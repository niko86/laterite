// The property #366 is about, held at the seam where it lives: two requests for
// the same URL whose first download is still in flight must become ONE strategy
// pass, with the second released only once the first's CACHE WRITE has settled.
// Released at headers instead, the joiner's own strategy pass misses the cache
// and starts download #2 — the exact bug — so the done-not-response distinction
// gets its own case here rather than a comment.
//
// The strategy is faked: workbox's real CacheFirst needs a service-worker
// global scope no unit environment has, and the wrapper's whole contract is
// "handleAll once per flight, then let the strategy answer from its cache" —
// which a fake can hold better, because it can distinguish the response
// settling from the cache write settling, and a real cache cannot be told to
// hold a write open.
import { describe, expect, it } from "vitest";

import { coalesce } from "./swCoalesce";

interface Deferred<T> {
  promise: Promise<T>;
  resolve: (v: T) => void;
  reject: (e: unknown) => void;
}

function deferred<T>(): Deferred<T> {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** `deferred` for the value-less done promise — its own shape because the
 *  generic form would need `Deferred<void>`, and `void` as a type argument is
 *  whitelisted for `Promise` alone here. */
function deferredDone(): {
  promise: Promise<void>;
  resolve: () => void;
  reject: (e: unknown) => void;
} {
  let resolve!: () => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<void>((res, rej) => {
    resolve = () => {
      res();
    };
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** One controllable strategy pass: its response and its done (the cache write)
 *  settle when the test says so, not together. */
interface Flight {
  url: string;
  response: Deferred<Response>;
  done: ReturnType<typeof deferredDone>;
}

function fakeStrategy() {
  const flights: Flight[] = [];
  return {
    flights,
    handleAll(options: {
      request: Request;
    }): [Promise<Response>, Promise<void>] {
      const flight: Flight = {
        url: options.request.url,
        response: deferred<Response>(),
        done: deferredDone(),
      };
      flights.push(flight);
      return [flight.response.promise, flight.done.promise];
    },
  };
}

const req = (url: string) => ({ request: new Request(url) });

/** Let queued microtasks run, so "the joiner has (not) advanced" is observed
 *  rather than assumed. */
const settle = () => new Promise<void>((r) => setTimeout(r, 0));

describe("coalesce", () => {
  it("passes a lone request straight through to the strategy", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    const p = handler(req("https://app/engine.wasm"));
    expect(strategy.flights).toHaveLength(1);

    const response = new Response("bytes");
    strategy.flights[0]!.response.resolve(response);
    strategy.flights[0]!.done.resolve();
    await expect(p).resolves.toBe(response);
  });

  it("joins an in-flight download instead of starting a second one", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    const leader = handler(req("https://app/engine.wasm"));
    const joiner = handler(req("https://app/engine.wasm"));
    await settle();

    // One network flight, however many requesters.
    expect(strategy.flights).toHaveLength(1);

    strategy.flights[0]!.response.resolve(new Response("leader"));
    strategy.flights[0]!.done.resolve();
    await settle();

    // The joiner re-enters the strategy AFTER the write settled — in the real
    // worker that second pass is a cache hit, not a network fetch.
    expect(strategy.flights).toHaveLength(2);
    const served = new Response("from-cache");
    strategy.flights[1]!.response.resolve(served);
    strategy.flights[1]!.done.resolve();

    await expect(leader).resolves.toBeInstanceOf(Response);
    await expect(joiner).resolves.toBe(served);
  });

  it("releases the joiner on the CACHE WRITE settling, not on the response", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    void handler(req("https://app/engine.wasm"));
    const joiner = handler(req("https://app/engine.wasm"));

    // Headers arrive (the response settles) while the body is still streaming
    // to the cache. Releasing here is the bug this module exists to close: the
    // joiner would miss the cache and re-download.
    strategy.flights[0]!.response.resolve(new Response("leader"));
    await settle();
    expect(strategy.flights).toHaveLength(1);

    strategy.flights[0]!.done.resolve();
    await settle();
    expect(strategy.flights).toHaveLength(2);
    strategy.flights[1]!.response.resolve(new Response("from-cache"));
    strategy.flights[1]!.done.resolve();
    await expect(joiner).resolves.toBeInstanceOf(Response);
  });

  it("never coalesces requests for different URLs", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    void handler(req("https://app/engine.wasm")).catch(() => {});
    void handler(req("https://app/other.wasm")).catch(() => {});
    await settle();

    expect(strategy.flights.map((f) => f.url)).toEqual([
      "https://app/engine.wasm",
      "https://app/other.wasm",
    ]);
  });

  it("lets the joiner fetch for itself when the leader fails", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    const failure = new Error("network gone");
    const leader = handler(req("https://app/engine.wasm"));
    // Settled by hand rather than via `expect(...).rejects`, which must be
    // awaited where it is written; the catch attached HERE also keeps the
    // runner from reporting the mid-test rejection as unhandled.
    let leaderOutcome: unknown = null;
    const leaderSettled = leader.catch((e: unknown) => {
      leaderOutcome = e;
    });
    const joiner = handler(req("https://app/engine.wasm"));

    strategy.flights[0]!.response.reject(failure);
    strategy.flights[0]!.done.reject(failure);
    await settle();

    // The joiner is released and retries as its own flight — an honest
    // refetch, not a doubled one: nothing was downloaded the first time.
    expect(strategy.flights).toHaveLength(2);
    const recovered = new Response("second-attempt");
    strategy.flights[1]!.response.resolve(recovered);
    strategy.flights[1]!.done.resolve();

    await leaderSettled;
    expect(leaderOutcome).toBe(failure);
    await expect(joiner).resolves.toBe(recovered);
  });

  it("forgets a settled flight — the next request is a fresh pass", async () => {
    const strategy = fakeStrategy();
    const handler = coalesce(strategy);

    const first = handler(req("https://app/engine.wasm"));
    strategy.flights[0]!.response.resolve(new Response("one"));
    strategy.flights[0]!.done.resolve();
    await first;
    await settle();

    void handler(req("https://app/engine.wasm"));
    await settle();
    // A fresh strategy pass (a cache hit in the real worker) — not a wait on
    // a flight that no longer exists.
    expect(strategy.flights).toHaveLength(2);
  });
});
