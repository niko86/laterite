// The dispatch exists to be driven by EITHER engine build (#338): the tier-1
// engine in the always-on worker, the full one in the lazily-created worker
// that serves Explore and Excel. That only holds while the engine is a
// PARAMETER — the moment someone re-hardwires the import, a second worker
// silently runs the first worker's engine and the whole tiering is undone with
// nothing failing.
//
// So this drives the dispatch with a fake engine and asserts the calls land on
// it. It is not coverage for its own sake: it is the one assertion that goes red
// if the engine stops being injectable, which is the entire point of #351.
import { describe, expect, it, vi } from "vitest";

import { createEngineDispatch, type EngineApi } from "./engineDispatch";
import type { WorkerRes } from "./engineDispatch";
import type { StandardDict } from "./validator";
import type { ParsedDataset } from "../wasm/ags4_wasm.js";

/** Only the members a given test actually exercises need to be real. */
const fakeEngine = (over: Partial<EngineApi> = {}): EngineApi => ({
  validate: vi.fn(),
  certify: vi.fn(),
  compute_fixes: vi.fn(() => []),
  apply_fixes: vi.fn(() => new Uint8Array()),
  read: vi.fn(),
  diff: vi.fn(),
  merge: vi.fn(),
  censor: vi.fn(),
  dictionary: vi.fn(),
  build_ags4: vi.fn(),
  ags4_to_xlsx: vi.fn(),
  xlsx_to_ags4: vi.fn(),
  ...over,
});

describe("engine dispatch", () => {
  it("routes an op to the engine it was constructed with", async () => {
    const dict = {
      ags_edition: "4.1.1",
      groups: [],
    } as unknown as StandardDict;
    const dictionary = vi.fn(() => dict);
    const replies: WorkerRes[] = [];

    const dispatch = createEngineDispatch(fakeEngine({ dictionary }), (m) =>
      replies.push(m),
    );
    await dispatch({ id: 7, kind: "dictionary", edition: "4.1.1" });

    expect(dictionary).toHaveBeenCalledWith("4.1.1");
    expect(replies).toEqual([{ id: 7, ok: true, kind: "dictionary", dict }]);
  });

  it("gives two dispatches genuinely separate engines", async () => {
    // The property the second worker depends on. If the engine were captured at
    // module scope instead of per-dispatch, both of these would hit the same fn.
    const a = vi.fn(() => ({ ags_edition: "4.1.1", groups: [] }));
    const b = vi.fn(() => ({ ags_edition: "4.2", groups: [] }));

    const dispatchA = createEngineDispatch(
      fakeEngine({ dictionary: a }),
      () => {},
    );
    const dispatchB = createEngineDispatch(
      fakeEngine({ dictionary: b }),
      () => {},
    );

    await dispatchA({ id: 1, kind: "dictionary", edition: null });
    await dispatchB({ id: 2, kind: "dictionary", edition: null });

    expect(a).toHaveBeenCalledTimes(1);
    expect(b).toHaveBeenCalledTimes(1);
  });

  it("keeps parsed-dataset state per dispatch, not shared", async () => {
    // `arrowIpc` reads the dataset the preceding `parse` stored. Two workers
    // must not see each other's — Explore's dataset belongs to Explore's worker.
    // Parsing in A must therefore leave B with nothing, which is only true while
    // the dataset lives in the closure rather than at module scope.
    const fakeDataset = {
      group_codes: () => ["LOCA"],
      meta: () => ({ headings: [], units: [], types: [], sql_types: [] }),
      arrow_ipc: () => new Uint8Array([1, 2, 3]),
      free: () => {},
    };
    const read = vi.fn(() => fakeDataset as unknown as ParsedDataset);

    const a = createEngineDispatch(fakeEngine({ read }), () => {});
    const b = createEngineDispatch(fakeEngine({ read }), () => {});

    await a({
      id: 1,
      kind: "parse",
      bytes: new ArrayBuffer(0),
      encoding: "utf-8",
    });
    // A can now serve a pull …
    await expect(
      a({ id: 2, kind: "arrowIpc", code: "LOCA" }),
    ).resolves.toBeUndefined();
    // … and B, which never parsed, still cannot.
    await expect(b({ id: 3, kind: "arrowIpc", code: "LOCA" })).rejects.toThrow(
      /parse first/,
    );
  });

  it("lets an engine error propagate for the caller to report", async () => {
    // The worker's own try/catch turns a throw into `{ ok: false }`. The
    // dispatch must not swallow it first, or that reply never happens.
    const boom = vi.fn(() => {
      throw new Error("engine exploded");
    });
    const dispatch = createEngineDispatch(
      fakeEngine({ dictionary: boom }),
      () => {},
    );

    await expect(
      dispatch({ id: 1, kind: "dictionary", edition: null }),
    ).rejects.toThrow(/engine exploded/);
  });
});
