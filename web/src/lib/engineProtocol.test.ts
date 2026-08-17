import { describe, expect, it } from "vitest";
import { settle, type Pending, type WorkerReply } from "./engineProtocol";

// The reply protocol on its own, importable at last (#380): every kind-pair
// maps a wire reply to the value the pane's promise resolves with, and the
// mismatch branch — a protocol bug's only witness — rejects with a message
// naming both sides. None of this was reachable while `settle` lived in
// `validatorClient`, whose module-scope worker spawn makes the whole file
// unimportable here (`ReferenceError: Worker is not defined` — proven in #380).
//
// Each case drives the REAL union member, not a stub shape: a wire field
// renamed in `engineDispatch` breaks these tests at compile time, which is half
// of what they are for.

/** Run one settle and capture how the pending settled. */
function outcome(msg: WorkerReply, kind: Pending["kind"]) {
  let resolved: unknown;
  let rejected: Error | undefined;
  settle(msg, {
    kind,
    resolve: (v: never) => {
      resolved = v;
    },
    reject: (e: Error) => {
      rejected = e;
    },
  } as unknown as Pending);
  return { resolved, rejected };
}

describe("settle", () => {
  it("report → the validation report itself", () => {
    const report = { findings: [], counts: {} } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "report", report },
      "report",
    );
    expect(resolved).toBe(report);
  });

  it("cert → the certificate JSON string", () => {
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "cert", json: '{"v":2}' },
      "cert",
    );
    expect(resolved).toBe('{"v":2}');
  });

  it("gzip → bytes plus the uncapped report meta", () => {
    const bytes = new ArrayBuffer(4);
    const meta = { total: 9 } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "gzip", bytes, report: meta },
      "gzip",
    );
    expect(resolved).toEqual({ bytes, meta });
  });

  it("fixes → the fix list", () => {
    const fixes = [{ label: "pad" }] as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "fixes", fixes },
      "fixes",
    );
    expect(resolved).toBe(fixes);
  });

  it("applied → the fixed bytes, wrapped for the caller", () => {
    const buf = Uint8Array.from([1, 2, 3]).buffer;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "applied", bytes: buf },
      "applied",
    );
    expect(resolved).toEqual(Uint8Array.from([1, 2, 3]));
  });

  it("parsed → the group metadata", () => {
    const groups = [{ code: "LOCA" }] as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "parsed", groups },
      "parsed",
    );
    expect(resolved).toBe(groups);
  });

  it("arrow → the IPC bytes, wrapped for the caller", () => {
    const buf = Uint8Array.from([9]).buffer;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "arrow", code: "LOCA", bytes: buf },
      "arrow",
    );
    expect(resolved).toEqual(Uint8Array.from([9]));
  });

  it("revisionDelta → the delta", () => {
    const delta = { groups: [] } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "revisionDelta", delta },
      "revisionDelta",
    );
    expect(resolved).toBe(delta);
  });

  it("mergeResult → bytes plus the PARSED warning and revision audits", () => {
    // The one pair that transforms rather than hands over: the worker ships the
    // audits as JSON text so the merge bytes transfer without a structured
    // clone of two arrays, and settle is where they become objects again.
    const bytes = new ArrayBuffer(2);
    const { resolved } = outcome(
      {
        id: 1,
        ok: true,
        kind: "mergeResult",
        bytes,
        warningsJson:
          '[{"kind":"recency","group":"LOCA","heading":null,"message":"m"}]',
        revisionsJson:
          '[{"group":"PROJ","key":["P1"],"changed":["PROJ_NAME"],"winnerFile":1}]',
      },
      "mergeResult",
    );
    expect(resolved).toEqual({
      bytes,
      warnings: [
        { kind: "recency", group: "LOCA", heading: null, message: "m" },
      ],
      revisions: [
        { group: "PROJ", key: ["P1"], changed: ["PROJ_NAME"], winnerFile: 1 },
      ],
    });
  });

  it("dictionary → the standard dict", () => {
    const dict = { edition: "4.1.1" } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "dictionary", dict },
      "dictionary",
    );
    expect(resolved).toBe(dict);
  });

  it("censor → the scrubbed text plus the tally", () => {
    const tally = { pseudonymised: 3 } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "censor", text: "clean", tally },
      "censor",
    );
    expect(resolved).toEqual({ text: "clean", tally });
  });

  it("toAgs4 → the export result", () => {
    const result = { bytes: new ArrayBuffer(0) } as never;
    const { resolved } = outcome(
      { id: 1, ok: true, kind: "toAgs4", result },
      "toAgs4",
    );
    expect(resolved).toBe(result);
  });

  it("excel → bytes plus warnings and the sheet/row counts", () => {
    const bytes = new ArrayBuffer(8);
    const { resolved } = outcome(
      {
        id: 1,
        ok: true,
        kind: "excel",
        bytes,
        warnings: ["w"],
        sheets: 3,
        rows: 42,
      },
      "excel",
    );
    expect(resolved).toEqual({ bytes, warnings: ["w"], sheets: 3, rows: 42 });
  });

  it("a kind mismatch rejects, naming both sides", () => {
    const { resolved, rejected } = outcome(
      { id: 1, ok: true, kind: "cert", json: "{}" },
      "report",
    );
    expect(resolved).toBeUndefined();
    expect(rejected).toBeInstanceOf(Error);
    expect(rejected?.message).toBe(
      "unexpected cert response for report request",
    );
  });

  it("never resolves AND rejects for any pair", () => {
    // The property behind every case above, swept across the whole matrix:
    // exactly one of resolve/reject fires, whatever meets whatever.
    const replies: WorkerReply[] = [
      { id: 1, ok: true, kind: "cert", json: "{}" },
      { id: 1, ok: true, kind: "censor", text: "t", tally: {} as never },
    ];
    const kinds: Pending["kind"][] = [
      "report",
      "cert",
      "gzip",
      "fixes",
      "applied",
      "parsed",
      "arrow",
      "revisionDelta",
      "mergeResult",
      "censor",
      "dictionary",
      "toAgs4",
      "excel",
    ];
    for (const msg of replies) {
      for (const kind of kinds) {
        const { resolved, rejected } = outcome(msg, kind);
        expect(resolved !== undefined || rejected !== undefined).toBe(true);
        expect(resolved !== undefined && rejected !== undefined).toBe(false);
      }
    }
  });
});
