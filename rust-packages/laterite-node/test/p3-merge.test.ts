// The Node `merge()` — N-way reconciliation over the SAME shared
// `laterite-ags4-merge` leaf Python / `lat merge` use, so the values match
// `test_merge.py`. Fixtures are shared with it deliberately (one behaviour, one
// set of reference values across surfaces).
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { MergeConflictError, merge, read } from "../ts/index";

// Two deliveries of one project, LOCA keyed on LOCA_ID.
//   file A: BH1 (NATE 100.00, GL 10.00), BH2 (NATE 200.00, GL 20.00)
//   file B: BH1 (NATE 100.00, GL 11.50 — real GL revision, identical NATE),
//           BH3 (NATE 300.00, GL 30.00 — new row)
// B also RE-TYPES LOCA_NATE 2DP -> X: the TYPE conflict strict mode rejects.
const A = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID","PROJ_NAME"',
  '"UNIT","",""',
  '"TYPE","ID","X"',
  '"DATA","P1","Demo"',
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
  '"UNIT","","m","m"',
  '"TYPE","ID","2DP","2DP"',
  '"DATA","BH1","100.00","10.00"',
  '"DATA","BH2","200.00","20.00"',
  "",
].join("\r\n");
const B = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID","PROJ_NAME"',
  '"UNIT","",""',
  '"TYPE","ID","X"',
  '"DATA","P1","Demo"',
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
  '"UNIT","","m","m"',
  '"TYPE","ID","X","2DP"',
  '"DATA","BH1","100.00","11.50"',
  '"DATA","BH3","300.00","30.00"',
  "",
].join("\r\n");

describe("merge → MergeResult", () => {
  it("returns bytes that re-parse as AGS4, plus the audit arrays", () => {
    const res = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
    });
    expect(res.bytes).toBeInstanceOf(Uint8Array);
    expect(res.text.startsWith('"GROUP"')).toBe(true);
    expect(Array.isArray(res.warnings)).toBe(true);
    expect(Array.isArray(res.revisions)).toBe(true);
    // The merged bytes are valid AGS4 (emit re-validates), so read() accepts them.
    expect(() => read(res.bytes)).not.toThrow();
  });

  it("unions every borehole (BH2 A-only and BH3 B-only both survive)", () => {
    const { text } = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
    });
    for (const bh of ["BH1", "BH2", "BH3"]) expect(text).toContain(`"${bh}"`);
  });

  it("resolves a KEY conflict by argument order (BH1 GL becomes B's 11.50)", () => {
    const { text } = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
    });
    // B's BH1 GL (11.50) wins over A's (10.00).
    expect(text).toContain('"11.50"');
    expect(text).not.toContain('"10.00"');
  });

  it("reports the real GL revision only, not the type-widened equal NATE", () => {
    const res = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
    });
    const locaRevs = res.revisions.filter((r) => r.group === "LOCA");
    expect(locaRevs.length).toBe(1);
    expect(locaRevs[0]?.key).toEqual(["BH1"]);
    expect(locaRevs[0]?.changed).toEqual(["LOCA_GL"]);
    expect(locaRevs[0]?.winnerFile).toBe(1);
  });

  it("throws MergeConflictError on a strict TYPE conflict (exit 6)", () => {
    let err: unknown;
    try {
      merge([Buffer.from(A), Buffer.from(B)]);
    } catch (e) {
      err = e;
    }
    expect(err).toBeInstanceOf(MergeConflictError);
    expect((err as MergeConflictError).exitCode).toBe(6);
    expect((err as Error).message).toContain("LOCA_NATE");
  });

  it("synthesises a merge-TRAN from tranIssue + tranDate", () => {
    const res = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
      tranIssue: "9",
      tranDate: "2024-05-01",
      tranProducer: "Merger",
    });
    expect(res.text).toContain('"GROUP","TRAN"');
    expect(res.text).toContain('"9"');
  });

  it("accepts path, bytes, and Ags4File inputs interchangeably", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-merge-"));
    const aPath = join(dir, "a.ags");
    const bPath = join(dir, "b.ags");
    writeFileSync(aPath, A);
    writeFileSync(bPath, B);
    const fromPaths = merge([aPath, bPath], { onTypeClash: "widen" });
    const fromBytes = merge([Buffer.from(A), Buffer.from(B)], {
      onTypeClash: "widen",
    });
    const fromHandles = merge(
      [read(undefined, { text: A }), read(undefined, { text: B })],
      {
        onTypeClash: "widen",
      },
    );
    for (const r of [fromPaths, fromBytes, fromHandles]) {
      expect(r.revisions.filter((x) => x.group === "LOCA").length).toBe(1);
    }
  });

  it("throws RangeError for fewer than two sources", () => {
    expect(() => merge([Buffer.from(A)])).toThrow(RangeError);
  });

  // #501 — TYPE has a universal absorber (`X`); UNIT has none. So a unit clash is
  // fatal in EVERY mode, unlike a type clash. Merge used to take the first
  // non-empty UNIT and discard the other silently, labelling `10500.00` mm as
  // metres — and since both are valid `2DP` numbers, nothing downstream could
  // catch it.
  it("throws MergeConflictError on conflicting UNITs, in every mode", () => {
    const inM = [
      '"GROUP","PROJ"',
      '"HEADING","PROJ_ID"',
      '"UNIT",""',
      '"TYPE","ID"',
      '"DATA","P1"',
      '"GROUP","LOCA"',
      '"HEADING","LOCA_ID","LOCA_GL"',
      '"UNIT","","m"',
      '"TYPE","ID","2DP"',
      '"DATA","BH01","10.00"',
      "",
    ].join("\r\n");
    const inMm = inM
      .replace('"UNIT","","m"', '"UNIT","","mm"')
      .replace('"DATA","BH01","10.00"', '"DATA","BH02","10500.00"');

    for (const onTypeClash of ["error", "widen", "promote"] as const) {
      let caught: unknown;
      try {
        merge([Buffer.from(inM), Buffer.from(inMm)], { onTypeClash });
      } catch (e) {
        caught = e;
      }
      expect(caught).toBeInstanceOf(MergeConflictError);
      const err = caught as MergeConflictError;
      expect(err.exitCode).toBe(6);
      expect(err.message).toContain("LOCA_GL");
      expect(err.message).toContain("will not convert units");
    }
  });
});

// --- the type-clash lattice (#500) ----------------------------------------
// Same three modes, same vocabulary, same resolved values as `test_merge.py` and
// the Rust `promote.rs` — the shared leaf is the point.
describe("merge onTypeClash", () => {
  // LOCA_GL typed 2DP in one delivery, 5DP in the other.
  const DP2 = [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_GL"',
    '"UNIT","","m"',
    '"TYPE","ID","2DP"',
    '"DATA","BH01","10.00"',
    "",
  ].join("\r\n");
  const DP5 = DP2.replace('"TYPE","ID","2DP"', '"TYPE","ID","5DP"').replace(
    '"DATA","BH01","10.00"',
    '"DATA","BH02","20.12345"',
  );

  it("promote keeps the greatest nDP precision and zero-pads the coarser values", () => {
    const res = merge([Buffer.from(DP2), Buffer.from(DP5)], {
      onTypeClash: "promote",
    });
    expect(res.text).toContain('"TYPE","ID","5DP"');
    expect(res.text).toContain('"DATA","BH01","10.00000"'); // padded — no digit changed
    expect(res.text).toContain('"DATA","BH02","20.12345"'); // already 5DP — untouched
    const promoted = res.warnings.filter((w) => w.kind === "type_promoted");
    expect(promoted).toHaveLength(1);
    expect(promoted[0]?.heading).toBe("LOCA_GL");
  });

  it("widen throws the TYPE away instead (the contrast that motivates promote)", () => {
    const res = merge([Buffer.from(DP2), Buffer.from(DP5)], {
      onTypeClash: "widen",
    });
    expect(res.text).toContain('"TYPE","ID","X"');
    expect(res.text).toContain('"DATA","BH01","10.00"'); // bytes untouched
  });

  it("error is the default, and names BOTH escape hatches", () => {
    let caught: unknown;
    try {
      merge([Buffer.from(DP2), Buffer.from(DP5)]); // no onTypeClash
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(MergeConflictError);
    const msg = (caught as MergeConflictError).message;
    expect(msg).toContain("LOCA_GL");
    expect(msg).toContain("promote");
    expect(msg).toContain("widen");
  });

  it("never promotes significant figures — padding nSF would overstate precision", () => {
    const sf3 = DP2.replace('"TYPE","ID","2DP"', '"TYPE","ID","3SF"');
    const sf5 = DP2.replace('"TYPE","ID","2DP"', '"TYPE","ID","5SF"').replace(
      '"DATA","BH01","10.00"',
      '"DATA","BH02","20.123"',
    );
    const res = merge([Buffer.from(sf3), Buffer.from(sf5)], {
      onTypeClash: "promote",
    });
    expect(res.text).toContain('"TYPE","ID","X"');
    expect(res.warnings.filter((w) => w.kind === "type_promoted")).toHaveLength(
      0,
    );
  });

  it("rejects an unknown mode, listing the ones it accepts", () => {
    expect(() =>
      // deliberately off-contract: the native layer is the gate, not just the types
      merge([Buffer.from(DP2), Buffer.from(DP5)], {
        onTypeClash: "yolo" as unknown as "widen",
      }),
    ).toThrow(/yolo[\s\S]*error[\s\S]*widen[\s\S]*promote/);
  });
});
