// The dispatch exists to be driven by EITHER engine build (#338): the tier-1
// engine in the always-on worker, the full one in the lazily-created worker
// that serves Explore and Excel. That only holds while the engine is a
// PARAMETER — the moment someone re-hardwires the import, a second worker
// silently runs the first worker's engine and the whole tiering is undone with
// nothing failing.
//
// Everything else here pins the half of this module the Playwright suite is
// BLIND to. e2e drives all thirteen ops through a real browser, so it proves
// the wasm computes the right answer — but it cannot see a leaked wasm handle,
// a dropped transfer list, or a `?? false` flipped to `?? true`. Delete the
// `finally` that frees the merge result and every e2e test still passes while
// the leak comes back. That is what these are for: handle lifecycle and the
// argument contract, the two things only a fake engine can observe.
import { describe, expect, it, vi } from "vitest";

import {
  createEngineDispatch,
  type EngineApi,
  type Reply,
} from "./engineDispatch";
import type { WorkerRes } from "./engineDispatch";
import type { ValidationReport } from "./validator";
import type { ParsedDataset } from "../wasm-full/ags4_wasm_full.js";

/** Only the members a given test actually exercises need to be real. */
const fakeEngine = (over: Partial<EngineApi> = {}): EngineApi =>
  ({
    validate: vi.fn(() => report()),
    certify: vi.fn(() => "{}"),
    compute_fixes: vi.fn(() => []),
    apply_fixes: vi.fn(() => new Uint8Array([1, 2])),
    read: vi.fn(() => fakeDataset()),
    diff: vi.fn(() => ({})),
    merge: vi.fn(() => fakeMergeResult()),
    censor: vi.fn(() => ({ text: "scrubbed", tally: {} })),
    build_ags4: vi.fn(() => ({})),
    ags4_to_xlsx: vi.fn(() => fakeExcelResult()),
    xlsx_to_ags4: vi.fn(() => fakeExcelResult()),
    ...over,
  }) as unknown as EngineApi;

const report = (over: Partial<ValidationReport> = {}) =>
  ({
    finding_count: 3,
    shown_count: 2,
    findings: [],
    ...over,
  }) as unknown as ValidationReport;

// Each factory takes its `free` mock rather than exposing one to read back off
// the result: asserting on `result.free` trips @typescript-eslint/unbound-method,
// and holding the mock directly names what the test is watching.
const fakeDataset = (
  over: Partial<Record<string, unknown>> = {},
  free = vi.fn(),
) =>
  ({
    group_codes: () => ["LOCA"],
    meta: () => ({ headings: [], units: [], types: [], sql_types: [] }),
    arrow_ipc: () => new Uint8Array([7, 8, 9]),
    free,
    ...over,
  }) as unknown as ParsedDataset;

const fakeMergeResult = (free = vi.fn()) => ({
  bytes: new Uint8Array([4, 5]),
  warnings_json: "[]",
  revisions_json: "[]",
  free,
});

const fakeExcelResult = (free = vi.fn()) => ({
  bytes: new Uint8Array([6, 7]),
  warnings: ["w"],
  sheets: 2,
  rows: 9,
  free,
});

/** A reply that records both the message and the transfer list, since the
 *  transfer list is half of what several ops promise. */
const collect = () => {
  const calls: { msg: WorkerRes; transfer?: Transferable[] }[] = [];
  const reply: Reply = (msg, transfer) => {
    calls.push({ msg, transfer });
  };
  // Indexed access is checked here (`noUncheckedIndexedAccess`), and "no reply
  // was made" is a failure worth naming rather than a `possibly undefined`.
  const at = (i: number) => {
    const call = calls[i];
    if (!call) throw new Error(`no reply at index ${i} (of ${calls.length})`);
    return call;
  };
  return { calls, reply, at, last: () => at(calls.length - 1) };
};

const validateReq = (over = {}) =>
  ({
    id: 1,
    kind: "validate",
    bytes: new ArrayBuffer(0),
    dict: null,
    includeFyi: false,
    encoding: "utf-8",
    maxPerRule: null,
    ...over,
  }) as Parameters<ReturnType<typeof createEngineDispatch>>[0];

/** wasm has no clock, so `checkedAt` is always the caller's — fixed here so a
 *  test can assert on it. */
const CHECKED_AT = "2026-08-16T00:00:00Z";

const certifyReq = (over = {}) =>
  ({
    id: 1,
    kind: "certify",
    bytes: new ArrayBuffer(0),
    dict: null,
    encoding: "utf-8",
    checkedAt: CHECKED_AT,
    ...over,
  }) as Parameters<ReturnType<typeof createEngineDispatch>>[0];

// These three watch the injection itself, not the op they drive it with. They
// used `dictionary` until #349 retired it; `certify` carries them now because
// it is the same shape of witness — a core op every build has, one call in and
// one reply out — and nothing here is about certification.
describe("engine injection", () => {
  it("routes an op to the engine it was constructed with", async () => {
    const certify = vi.fn(() => '{"v":2}');
    const { at, reply } = collect();

    const dispatch = createEngineDispatch(fakeEngine({ certify }), reply);
    await dispatch(certifyReq({ id: 7 }));

    expect(certify).toHaveBeenCalledWith(
      expect.any(Uint8Array),
      expect.objectContaining({ checkedAt: CHECKED_AT }),
    );
    expect(at(0).msg).toEqual({
      id: 7,
      ok: true,
      kind: "cert",
      json: '{"v":2}',
    });
  });

  it("gives two dispatches genuinely separate engines", async () => {
    // The property the second worker depends on. If the engine were captured at
    // module scope instead of per-dispatch, both of these would hit the same fn.
    const a = vi.fn(() => '{"engine":"a"}');
    const b = vi.fn(() => '{"engine":"b"}');

    const dispatchA = createEngineDispatch(fakeEngine({ certify: a }), () => {});
    const dispatchB = createEngineDispatch(fakeEngine({ certify: b }), () => {});

    await dispatchA(certifyReq({ id: 1 }));
    await dispatchB(certifyReq({ id: 2 }));

    expect(a).toHaveBeenCalledTimes(1);
    expect(b).toHaveBeenCalledTimes(1);
  });

  it("lets an engine error propagate for the caller to report", async () => {
    // The worker's own try/catch turns a throw into `{ ok: false }`. The
    // dispatch must not swallow it first, or that reply never happens.
    const boom = vi.fn(() => {
      throw new Error("engine exploded");
    });
    const dispatch = createEngineDispatch(
      fakeEngine({ certify: boom }),
      () => {},
    );

    await expect(dispatch(certifyReq())).rejects.toThrow(/engine exploded/);
  });
});

// Tier 1 is a real engine that genuinely lacks `arrow` and `excel` (#355), which
// is what makes the precache 4.5 MiB smaller. The tier boundary is enforced at
// compile time where each worker names the ops it can serve, and by routing in
// `validatorClient.ts` — so these ops should never arrive here. If one does, the
// failure has to name the op and the reason: an engine build is not something a
// reader can see from "undefined is not a function" inside a wasm shim.
describe("an engine build without arrow or excel", () => {
  // The eight ops tier 1 serves — the same list `validator.worker.ts` passes.
  const tier1Engine = () => {
    const {
      validate,
      certify,
      compute_fixes,
      apply_fixes,
      diff,
      merge,
      censor,
      build_ags4,
    } = fakeEngine();
    return {
      validate,
      certify,
      compute_fixes,
      apply_fixes,
      diff,
      merge,
      censor,
      build_ags4,
    };
  };

  it("still serves every op it does have", async () => {
    const { at, reply } = collect();
    const dispatch = createEngineDispatch(tier1Engine(), reply);

    await dispatch(certifyReq());

    expect(at(0).msg).toMatchObject({ id: 1, ok: true, kind: "cert" });
  });

  it("names `read` when asked to parse", async () => {
    const dispatch = createEngineDispatch(tier1Engine(), () => {});

    await expect(
      dispatch({
        id: 1,
        kind: "parse",
        bytes: new ArrayBuffer(0),
        encoding: "utf-8",
      }),
    ).rejects.toThrow(/no read\(\).*tier 1/);
  });

  it("names the Excel doors when asked to convert, in either direction", async () => {
    const dispatch = createEngineDispatch(tier1Engine(), () => {});

    await expect(
      dispatch({ id: 1, kind: "excelExport", bytes: new ArrayBuffer(0) }),
    ).rejects.toThrow(/no ags4_to_xlsx\(\).*tier 1/);
    await expect(
      dispatch({
        id: 2,
        kind: "excelImport",
        bytes: new ArrayBuffer(0),
        formatNumeric: true,
      }),
    ).rejects.toThrow(/no xlsx_to_ags4\(\).*tier 1/);
  });
});

describe("wasm handle lifecycle", () => {
  // Every one of these frees a handle the GC cannot reclaim. A leak is
  // invisible to e2e — the op returns the right answer either way — and grows
  // with use, which is exactly how the merge leak survived to ship.

  it("frees the merge result after reading its getters", async () => {
    const free = vi.fn();
    const result = fakeMergeResult(free);
    const dispatch = createEngineDispatch(
      fakeEngine({
        merge: vi.fn(() => result) as unknown as EngineApi["merge"],
      }),
      () => {},
    );

    await dispatch({
      id: 1,
      kind: "merge",
      aBytes: new ArrayBuffer(0),
      bBytes: new ArrayBuffer(0),
      encoding: "utf-8",
      onTypeClash: "error",
      tran: null,
    });

    expect(free).toHaveBeenCalledTimes(1);
  });

  it("still frees the merge result when a getter throws", async () => {
    // The `finally`. Without it a getter throwing turns one bug into two: the
    // op fails AND the handle leaks. Nothing else in the suite reaches this.
    const free = vi.fn();
    const result = {
      bytes: new Uint8Array([1]),
      get warnings_json(): string {
        throw new Error("getter exploded");
      },
      revisions_json: "[]",
      free,
    };
    const dispatch = createEngineDispatch(
      fakeEngine({
        merge: vi.fn(() => result) as unknown as EngineApi["merge"],
      }),
      () => {},
    );

    await expect(
      dispatch({
        id: 1,
        kind: "merge",
        aBytes: new ArrayBuffer(0),
        bBytes: new ArrayBuffer(0),
        encoding: "utf-8",
        onTypeClash: "error",
        tran: null,
      }),
    ).rejects.toThrow(/getter exploded/);
    expect(free).toHaveBeenCalledTimes(1);
  });

  it("frees the excel result in both directions", async () => {
    const exportFree = vi.fn();
    const importFree = vi.fn();
    const exported = fakeExcelResult(exportFree);
    const imported = fakeExcelResult(importFree);
    const dispatch = createEngineDispatch(
      fakeEngine({
        ags4_to_xlsx: vi.fn(
          () => exported,
        ) as unknown as EngineApi["ags4_to_xlsx"],
        xlsx_to_ags4: vi.fn(
          () => imported,
        ) as unknown as EngineApi["xlsx_to_ags4"],
      }),
      () => {},
    );

    await dispatch({ id: 1, kind: "excelExport", bytes: new ArrayBuffer(0) });
    await dispatch({
      id: 2,
      kind: "excelImport",
      bytes: new ArrayBuffer(0),
      formatNumeric: true,
    });

    expect(exportFree).toHaveBeenCalledTimes(1);
    expect(importFree).toHaveBeenCalledTimes(1);
  });

  it("frees the previous dataset before parsing a new one", async () => {
    // Residency is meant to stay at ONE dataset. Drop this and a session that
    // opens five files holds five parsed datasets in wasm memory.
    const firstFree = vi.fn();
    const secondFree = vi.fn();
    const first = fakeDataset({}, firstFree);
    const second = fakeDataset({}, secondFree);
    const read = vi.fn().mockReturnValueOnce(first).mockReturnValueOnce(second);
    const dispatch = createEngineDispatch(
      fakeEngine({ read: read as unknown as EngineApi["read"] }),
      () => {},
    );

    const parse = (id: number) =>
      dispatch({
        id,
        kind: "parse",
        bytes: new ArrayBuffer(0),
        encoding: "utf-8",
      });
    await parse(1);
    expect(firstFree).not.toHaveBeenCalled();
    await parse(2);

    expect(firstFree).toHaveBeenCalledTimes(1);
    expect(secondFree).not.toHaveBeenCalled();
  });

  it("keeps parsed-dataset state per dispatch, not shared", async () => {
    // `arrowIpc` reads the dataset the preceding `parse` stored. Two workers
    // must not see each other's — Explore's dataset belongs to Explore's worker.
    // Parsing in A must therefore leave B with nothing, which is only true while
    // the dataset lives in the closure rather than at module scope.
    const read = vi.fn(() => fakeDataset());
    const a = createEngineDispatch(
      fakeEngine({ read: read as unknown as EngineApi["read"] }),
      () => {},
    );
    const b = createEngineDispatch(
      fakeEngine({ read: read as unknown as EngineApi["read"] }),
      () => {},
    );

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
});

describe("the request → engine argument contract", () => {
  // Every `?? undefined` / `?? false` below is load-bearing: the engine reads an
  // absent option as "use the default" and would read `null` as a value. These
  // assert the whole options object, so a field silently dropped in a future
  // edit fails here rather than in a browser six ops later.

  it("normalises validate's absent options to the engine's defaults", async () => {
    const validate = vi.fn(() => report());
    const dispatch = createEngineDispatch(
      fakeEngine({ validate: validate as unknown as EngineApi["validate"] }),
      () => {},
    );

    await dispatch(validateReq());

    expect(validate).toHaveBeenCalledWith(expect.any(Uint8Array), {
      dictVersion: undefined,
      // Stated, not assumed: the display decision is this app's to make.
      warnings: true,
      fyi: false,
      encoding: "utf-8",
      maxPerRule: undefined,
      dictionary: undefined,
      dictReplace: false,
    });
  });

  it("passes validate's options through when they are set", async () => {
    const validate = vi.fn(() => report());
    const dictBytes = new Uint8Array([9]);
    const dispatch = createEngineDispatch(
      fakeEngine({ validate: validate as unknown as EngineApi["validate"] }),
      () => {},
    );

    await dispatch(
      validateReq({
        dict: "4.1.1",
        includeFyi: true,
        maxPerRule: 5,
        dictBytes,
        dictReplace: true,
      }),
    );

    expect(validate).toHaveBeenCalledWith(expect.any(Uint8Array), {
      dictVersion: "4.1.1",
      warnings: true,
      fyi: true,
      encoding: "utf-8",
      maxPerRule: 5,
      dictionary: dictBytes,
      dictReplace: true,
    });
  });

  it("defaults arrowIpc's key and hash columns to off, and honours them on", async () => {
    // #303 / #448 — both opt-in. Defaulting either to true would ship the
    // internal `_id`/`_content_hash` columns into every group grid.
    const arrow_ipc = vi.fn(() => new Uint8Array([1]));
    const read = vi.fn(() => fakeDataset({ arrow_ipc }));
    const dispatch = createEngineDispatch(
      fakeEngine({ read: read as unknown as EngineApi["read"] }),
      () => {},
    );
    await dispatch({
      id: 1,
      kind: "parse",
      bytes: new ArrayBuffer(0),
      encoding: "utf-8",
    });

    await dispatch({ id: 2, kind: "arrowIpc", code: "LOCA" });
    expect(arrow_ipc).toHaveBeenLastCalledWith("LOCA", false, false);

    await dispatch({
      id: 3,
      kind: "arrowIpc",
      code: "LOCA",
      keys: true,
      contentHash: true,
    });
    expect(arrow_ipc).toHaveBeenLastCalledWith("LOCA", true, true);
  });

  it("normalises the nullable options of diff, merge, certify and toAgs4", async () => {
    const diff = vi.fn(() => ({}));
    const merge = vi.fn(() => fakeMergeResult());
    const certify = vi.fn(() => "{}");
    const build_ags4 = vi.fn(() => ({}));
    const dispatch = createEngineDispatch(
      fakeEngine({
        diff: diff as unknown as EngineApi["diff"],
        merge: merge as unknown as EngineApi["merge"],
        certify: certify as unknown as EngineApi["certify"],
        build_ags4: build_ags4 as unknown as EngineApi["build_ags4"],
      }),
      () => {},
    );

    await dispatch({
      id: 1,
      kind: "revisionDiff",
      aBytes: new ArrayBuffer(0),
      bBytes: new ArrayBuffer(0),
      encoding: "utf-8",
      maxRowsPerGroup: null,
    });
    expect(diff).toHaveBeenCalledWith(
      expect.any(Uint8Array),
      expect.any(Uint8Array),
      {
        encoding: "utf-8",
        maxRowsPerGroup: undefined,
      },
    );

    await dispatch({
      id: 2,
      kind: "merge",
      aBytes: new ArrayBuffer(0),
      bBytes: new ArrayBuffer(0),
      encoding: "utf-8",
      onTypeClash: "promote",
      tran: null,
    });
    expect(merge).toHaveBeenCalledWith(
      expect.any(Uint8Array),
      expect.any(Uint8Array),
      {
        encoding: "utf-8",
        onTypeClash: "promote",
        tran: undefined,
      },
    );

    await dispatch({
      id: 4,
      kind: "certify",
      bytes: new ArrayBuffer(0),
      dict: null,
      encoding: "utf-8",
      checkedAt: "2026-08-16T00:00:00Z",
    });
    expect(certify).toHaveBeenCalledWith(expect.any(Uint8Array), {
      dictVersion: undefined,
      encoding: "utf-8",
      checkedAt: "2026-08-16T00:00:00Z",
      dictionary: undefined,
      dictReplace: false,
    });

    await dispatch({
      id: 5,
      kind: "toAgs4",
      groupsJson: "[]",
      edition: null,
      mode: "strict",
    });
    expect(build_ags4).toHaveBeenCalledWith("[]", {
      dictVersion: undefined,
      mode: "strict",
    });
  });

  it("hands computeFixes its dictionary positionally, null and all", async () => {
    // The one op that does NOT normalise: `compute_fixes` takes the dict as a
    // positional argument and reads `null` itself, where every options-object op
    // above needs `undefined`. Pinned because the asymmetry looks like an
    // oversight and a helpful `?? undefined` here would change what the engine
    // receives.
    const compute_fixes = vi.fn(() => []);
    const { at, reply } = collect();
    const dispatch = createEngineDispatch(
      fakeEngine({
        compute_fixes: compute_fixes as unknown as EngineApi["compute_fixes"],
      }),
      reply,
    );

    await dispatch({
      id: 1,
      kind: "computeFixes",
      bytes: new ArrayBuffer(0),
      dict: null,
      encoding: "utf-8",
    });

    expect(compute_fixes).toHaveBeenCalledWith(
      expect.any(Uint8Array),
      null,
      "utf-8",
    );
    expect(at(0).msg).toEqual({ id: 1, ok: true, kind: "fixes", fixes: [] });
  });

  it("forwards the censor policy as the engine's option object", async () => {
    const censor = vi.fn(() => ({ text: "scrubbed", tally: {} }));
    const { at, reply } = collect();
    const dispatch = createEngineDispatch(
      fakeEngine({ censor: censor as unknown as EngineApi["censor"] }),
      reply,
    );

    await dispatch({
      id: 1,
      kind: "censor",
      bytes: new ArrayBuffer(0),
      sensitiveJson: "{}",
      selectedCodes: ["LOCA"],
      token: "***",
      dropCustom: true,
      includeFreetext: false,
    });

    expect(censor).toHaveBeenCalledWith(expect.any(Uint8Array), "{}", {
      selectedCodes: ["LOCA"],
      token: "***",
      dropCustom: true,
      includeFreetext: false,
    });
    expect(at(0).msg).toMatchObject({ kind: "censor", text: "scrubbed" });
  });

  it("substitutes an empty schema for a group the engine has no meta for", async () => {
    // `meta()` returning null must not put `undefined` headings into the reply —
    // the grid reads these straight.
    const read = vi.fn(() => fakeDataset({ meta: () => null }));
    const { at, reply } = collect();
    const dispatch = createEngineDispatch(
      fakeEngine({ read: read as unknown as EngineApi["read"] }),
      reply,
    );

    await dispatch({
      id: 1,
      kind: "parse",
      bytes: new ArrayBuffer(0),
      encoding: "utf-8",
    });

    expect(at(0).msg).toEqual({
      id: 1,
      ok: true,
      kind: "parsed",
      groups: [
        { code: "LOCA", headings: [], units: [], types: [], sql_types: [] },
      ],
    });
  });
});

describe("byte-returning ops transfer their buffer", () => {
  // A missing transfer list is silent: the message still arrives, having COPIED
  // a payload that can run to hundreds of MB. Nothing observable fails, which is
  // why only an assertion on the list itself catches it.
  const byteOps: [
    string,
    Parameters<ReturnType<typeof createEngineDispatch>>[0],
  ][] = [
    [
      "applied",
      {
        id: 1,
        kind: "applyFixes",
        bytes: new ArrayBuffer(0),
        encoding: "utf-8",
        fixes: [],
      },
    ],
    [
      "mergeResult",
      {
        id: 2,
        kind: "merge",
        aBytes: new ArrayBuffer(0),
        bBytes: new ArrayBuffer(0),
        encoding: "utf-8",
        onTypeClash: "error",
        tran: null,
      },
    ],
    ["excel", { id: 3, kind: "excelExport", bytes: new ArrayBuffer(0) }],
  ];

  it.each(byteOps)(
    "transfers the exact buffer it replies with (%s)",
    async (kind, req) => {
      const { reply, last } = collect();
      const dispatch = createEngineDispatch(fakeEngine(), reply);

      await dispatch(req);

      const { msg, transfer } = last();
      expect(msg).toMatchObject({ kind });
      const bytes = (msg as { bytes: ArrayBuffer }).bytes;
      // Not `toEqual` — the transferred handle must BE the replied buffer, or the
      // main thread receives a detached one.
      expect(transfer).toEqual([bytes]);
    },
  );

  it("transfers a freshly sliced buffer, not a view onto wasm memory", async () => {
    // `.slice()` before `.buffer` is what makes the transfer safe: transferring
    // a view onto the wasm heap would detach the engine's own memory.
    const wasmHeap = new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7]);
    const view = wasmHeap.subarray(2, 5);
    const { reply, last } = collect();
    const dispatch = createEngineDispatch(
      fakeEngine({
        apply_fixes: vi.fn(() => view) as unknown as EngineApi["apply_fixes"],
      }),
      reply,
    );

    await dispatch({
      id: 1,
      kind: "applyFixes",
      bytes: new ArrayBuffer(0),
      encoding: "utf-8",
      fixes: [],
    });

    const bytes = (last().msg as { bytes: ArrayBuffer }).bytes;
    expect(bytes).not.toBe(wasmHeap.buffer);
    expect(bytes.byteLength).toBe(3);
    expect(new Uint8Array(bytes)).toEqual(new Uint8Array([2, 3, 4]));
  });

  it("pulls arrow IPC for the requested group and transfers it", async () => {
    const { reply, last } = collect();
    const dispatch = createEngineDispatch(fakeEngine(), reply);
    await dispatch({
      id: 1,
      kind: "parse",
      bytes: new ArrayBuffer(0),
      encoding: "utf-8",
    });

    await dispatch({ id: 2, kind: "arrowIpc", code: "LOCA" });

    const { msg, transfer } = last();
    expect(msg).toMatchObject({ id: 2, ok: true, kind: "arrow", code: "LOCA" });
    expect(transfer).toEqual([(msg as { bytes: ArrayBuffer }).bytes]);
  });
});

describe("the validate reply shape", () => {
  it("returns the report object when gzip is not asked for", async () => {
    const { at, reply } = collect();
    const dispatch = createEngineDispatch(fakeEngine(), reply);

    await dispatch(validateReq());

    expect(at(0).msg).toMatchObject({ id: 1, ok: true, kind: "report" });
  });

  it("gzips the report and carries the counts alongside it", async () => {
    // The download path. The counts ride separately so the UI can label the
    // file without inflating a multi-hundred-MB payload to read two numbers —
    // if they stopped matching the report, the label would silently lie.
    const { reply, last } = collect();
    const dispatch = createEngineDispatch(
      fakeEngine({
        validate: vi.fn(() =>
          report({ finding_count: 41, shown_count: 12 }),
        ) as unknown as EngineApi["validate"],
      }),
      reply,
    );

    await dispatch(validateReq({ gzip: true }));

    const { msg, transfer } = last();
    expect(msg).toMatchObject({
      id: 1,
      ok: true,
      kind: "gzip",
      report: { finding_count: 41, shown_count: 12 },
    });
    const bytes = (msg as { bytes: ArrayBuffer }).bytes;
    expect(transfer).toEqual([bytes]);
    // Really gzip: the members check, not just "some bytes came back".
    expect(new Uint8Array(bytes).slice(0, 2)).toEqual(
      new Uint8Array([0x1f, 0x8b]),
    );
  });
});
