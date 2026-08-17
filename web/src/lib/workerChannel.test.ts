import { describe, expect, it } from "vitest";
import {
  createChannel,
  EngineLoadError,
  type WorkerReply,
} from "./workerChannel";

// The channel's own behaviour, with a fake worker in place of a real one — the
// reason it is a module of its own (#357). What is under test here is the
// LIFECYCLE, not the protocol: when a worker is created, when its replies are
// correlated, and above all when it is dropped so the next request gets a fresh
// one. What a reply MEANS is `validatorClient`'s `settle`, injected below as a
// recorder.
//
// A dead engine that is never dropped is the failure this file exists for: the
// tab that needed it can report, but its retry posts into the same worker,
// re-reads the same settled rejection, and fails identically for ever. The
// browser half — that the retry really re-fetches, and that the other tabs
// carry on — is in web/e2e/app.spec.ts, which is the only place a real fetch
// can be blocked and released.

type Listener = (e: unknown) => void;

/** Just enough of a `Worker` to drive the channel: the two events it subscribes
 *  to, what it posted, and whether it was terminated. */
class FakeWorker {
  readonly posted: { msg: Record<string, unknown>; transfer?: unknown[] }[] =
    [];
  terminated = false;
  private readonly listeners = new Map<string, Set<Listener>>();

  addEventListener(type: string, fn: Listener): void {
    const set = this.listeners.get(type) ?? new Set<Listener>();
    set.add(fn);
    this.listeners.set(type, set);
  }

  removeEventListener(type: string, fn: Listener): void {
    this.listeners.get(type)?.delete(fn);
  }

  postMessage(msg: Record<string, unknown>, transfer?: unknown[]): void {
    this.posted.push({ msg, transfer });
  }

  terminate(): void {
    this.terminated = true;
  }

  /** Worker → main thread. */
  send(data: unknown): void {
    this.fire("message", { data });
  }

  /** The hard `error` event an unloadable or crashed worker fires. */
  crash(message: string): void {
    this.fire("error", { message });
  }

  private fire(type: string, e: unknown): void {
    // Copied: `onInit` removes itself while the set is being iterated.
    for (const fn of [...(this.listeners.get(type) ?? [])]) fn(e);
  }
}

interface TestPending {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
}

/** A channel over fake workers, plus the spawn log and the replies `settle`
 *  was handed. Every test needs all three, and the casts belong in one place. */
function harness() {
  const spawned: FakeWorker[] = [];
  const settled: { msg: WorkerReply; p: TestPending }[] = [];
  const channel = createChannel<TestPending>(
    () => {
      const w = new FakeWorker();
      spawned.push(w);
      return w as unknown as Worker;
    },
    (msg, p) => settled.push({ msg, p }),
  );
  return {
    channel,
    spawned,
    settled,
    last: () => spawned[spawned.length - 1]!,
  };
}

/** A pending entry plus the promise it settles, so a test can await either
 *  outcome the way a caller in `validatorClient` does. */
function pending() {
  let entry!: TestPending;
  const promise = new Promise<unknown>((resolve, reject) => {
    entry = { resolve, reject };
  });
  return { entry, promise };
}

const VALIDATE = {
  kind: "validate",
  bytes: new ArrayBuffer(0),
  dict: null,
  includeFyi: true,
  encoding: "utf-8",
  maxPerRule: null,
} as const;

describe("createChannel", () => {
  it("spawns nothing until something asks", () => {
    const { channel, spawned } = harness();
    expect(channel.started()).toBe(false);
    expect(spawned).toHaveLength(0);

    channel.start();
    expect(channel.started()).toBe(true);
    expect(spawned).toHaveLength(1);
  });

  it("reuses one worker across start, ready and every post", () => {
    const { channel, spawned } = harness();
    channel.start();
    void channel.ready();
    channel.post(VALIDATE, new Uint8Array([1, 2]), pending().entry);
    channel.postBare({ kind: "arrowIpc", code: "LOCA" }, pending().entry);
    expect(spawned).toHaveLength(1);
  });

  it("correlates a reply to the request waiting on its id", async () => {
    const { channel, settled, last } = harness();
    const a = pending();
    const b = pending();
    channel.post(VALIDATE, new Uint8Array([1]), a.entry);
    channel.post(VALIDATE, new Uint8Array([2]), b.entry);

    const ids = last().posted.map((p) => p.msg.id as number);
    expect(new Set(ids).size).toBe(2); // ids are unique, and monotonic
    expect(ids[1]).toBe(ids[0]! + 1);

    last().send({ id: ids[1]!, ok: true, kind: "cert", json: "{}" });
    expect(settled).toHaveLength(1);
    expect(settled[0]!.p).toBe(b.entry);

    // The other request is still outstanding — a settled sibling doesn't
    // disturb it, and a second copy of the same reply finds nothing to settle.
    last().send({ id: ids[1]!, ok: true, kind: "cert", json: "{}" });
    expect(settled).toHaveLength(1);

    last().send({ id: ids[0]!, ok: false, error: "bad file" });
    await expect(a.promise).rejects.toThrow("bad file");
  });

  it("ignores a reply to an id nobody is waiting on", () => {
    const { channel, settled, last } = harness();
    channel.start();
    last().send({ id: 9999, ok: true, kind: "cert", json: "{}" });
    expect(settled).toHaveLength(0);
  });

  it("posts a COPY of the caller's bytes, and transfers that", () => {
    const { channel, last } = harness();
    const bytes = new Uint8Array([1, 2, 3]);
    channel.post(VALIDATE, bytes, pending().entry);

    const sent = last().posted[0]!;
    const copy = sent.msg.bytes as ArrayBuffer;
    expect(new Uint8Array(copy)).toEqual(bytes);
    expect(sent.transfer).toEqual([copy]);
    // The caller still owns its own buffer — the editor text and finding
    // snippets are decoded from it after the post.
    expect(bytes).toEqual(new Uint8Array([1, 2, 3]));
  });

  it("posts two independent copies for a dual-buffer request", () => {
    const { channel, last } = harness();
    const a = new Uint8Array([1]);
    const b = new Uint8Array([2]);
    channel.postDual(
      {
        kind: "revisionDiff",
        aBytes: new ArrayBuffer(0),
        bBytes: new ArrayBuffer(0),
        encoding: "utf-8",
        maxRowsPerGroup: null,
      },
      a,
      b,
      pending().entry,
    );

    const sent = last().posted[0]!;
    expect(new Uint8Array(sent.msg.aBytes as ArrayBuffer)).toEqual(a);
    expect(new Uint8Array(sent.msg.bBytes as ArrayBuffer)).toEqual(b);
    expect(sent.transfer).toEqual([sent.msg.aBytes, sent.msg.bBytes]);
  });

  it("posts a bare request with no transfer list", () => {
    const { channel, last } = harness();
    channel.postBare({ kind: "arrowIpc", code: "LOCA" }, pending().entry);
    expect(last().posted[0]!.transfer).toBeUndefined();
    expect(last().posted[0]!.msg.code).toBe("LOCA");
  });

  it("resolves ready when the engine comes up", async () => {
    const { channel, last } = harness();
    const ready = channel.ready();
    last().send({ type: "ready" });
    await expect(ready).resolves.toBeUndefined();
    expect(last().terminated).toBe(false);
  });

  it("retires the worker when its engine fails to load", async () => {
    const { channel, spawned, last } = harness();
    const ready = channel.ready();
    const inflight = pending();
    channel.post(VALIDATE, new Uint8Array([1]), inflight.entry);
    const dead = last();

    dead.send({ type: "initError", error: "TypeError: Failed to fetch" });

    // Reported, not hung — on the readiness promise AND on the request that was
    // waiting, both as the one error a retry can clear.
    await expect(ready).rejects.toThrow(EngineLoadError);
    await expect(inflight.promise).rejects.toThrow("Failed to fetch");
    await expect(inflight.promise).rejects.toBeInstanceOf(EngineLoadError);
    expect(dead.terminated).toBe(true);
    expect(channel.started()).toBe(false);
    expect(spawned).toHaveLength(1);
  });

  it("gives the next request a FRESH worker after a failed load", async () => {
    const { channel, spawned, settled, last } = harness();
    const first = pending();
    channel.post(VALIDATE, new Uint8Array([1]), first.entry);
    last().send({ type: "initError", error: "Failed to fetch" });
    await expect(first.promise).rejects.toThrow(EngineLoadError);

    // The retry. Without the retirement this posts into the dead worker, which
    // has already replied to everything it will ever reply to, and the promise
    // never settles — the retry that can never succeed, however long ago the
    // cause was fixed.
    const retry = pending();
    channel.post(VALIDATE, new Uint8Array([1]), retry.entry);
    expect(spawned).toHaveLength(2);
    expect(last()).not.toBe(spawned[0]);

    last().send({ type: "ready" });
    const id = last().posted[0]!.msg.id as number;
    last().send({ id, ok: true, kind: "cert", json: "{}" });
    expect(settled).toHaveLength(1);
    expect(settled[0]!.p).toBe(retry.entry);
  });

  it("lets a retired worker's late replies fall on the floor", async () => {
    const { channel, settled, spawned } = harness();
    const first = pending();
    channel.post(VALIDATE, new Uint8Array([1]), first.entry);
    const dead = spawned[0]!;
    const deadId = dead.posted[0]!.msg.id as number;

    dead.send({ type: "initError", error: "Failed to fetch" });
    await expect(first.promise).rejects.toThrow(EngineLoadError);

    // The worker answers each queued op with its own `{ ok: false }` after the
    // initError. Those arrive to a table that has already been failed and
    // cleared, so they settle nothing twice.
    dead.send({ id: deadId, ok: false, error: "Failed to fetch" });
    expect(settled).toHaveLength(0);
  });

  it("fails a crashed worker's in-flight requests", async () => {
    const { channel, spawned, last } = harness();
    const inflight = pending();
    channel.post(VALIDATE, new Uint8Array([1]), inflight.entry);
    last().crash("out of memory");

    await expect(inflight.promise).rejects.toThrow("out of memory");
    // NOT an EngineLoadError: the engine loaded, the worker died. Only the
    // load failure offers a retry, because only it has a cause that can be
    // fixed from outside the app.
    await expect(inflight.promise).rejects.not.toBeInstanceOf(EngineLoadError);
    // Dropping the handle here too is #363, which owns the change and its e2e.
    expect(channel.started()).toBe(true);
    expect(spawned).toHaveLength(1);
  });

  it("names a crash with no message", async () => {
    const { channel, last } = harness();
    const inflight = pending();
    channel.post(VALIDATE, new Uint8Array([1]), inflight.entry);
    last().crash("");
    await expect(inflight.promise).rejects.toThrow("engine worker crashed");
  });
});
