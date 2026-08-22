/* The demo's model has one invariant that matters more than the rest: the text
 * the engine validates must be the text the tables describe (#396–#398).
 *
 * If emit() and the tables disagree, every finding points at a line the reader
 * is not looking at, and the page's whole argument — "here is the file, here is
 * what the engine thinks of it" — becomes a coincidence. So the round-trip is
 * pinned against the committed fixture rather than against a hand-written
 * sample: the fixture is what ships, and it is the thing gated on the Python
 * side by tests/test_landing_demo_delivery.py.
 */

import { describe, expect, it } from "vitest";
import seededText from "./seeded-delivery.ags?raw";
import {
  SEEDED,
  addRow,
  deleteGroup,
  deleteRow,
  emit,
  groupOfLine,
  lineOfRow,
  parse,
  restoreGroup,
  seededFinalDepth,
  setCell,
} from "./delivery";
import { DEMO_GROUPS, keyHeadings } from "./schema";

describe("parse", () => {
  it("reads every group in the seeded delivery", () => {
    expect(SEEDED.map((g) => g.code)).toEqual([
      "PROJ",
      "TRAN",
      "LOCA",
      "SAMP",
      "LLPL",
      "UNIT",
      "TYPE",
      "ABBR",
    ]);
  });

  it("draws LLPL at nine columns, which is the whole layout constraint", () => {
    const llpl = SEEDED.find((g) => g.code === "LLPL")!;
    expect(llpl.headings).toHaveLength(9);
    expect(llpl.rows.every((r) => r.length === 9)).toBe(true);
  });

  it("keeps the heading, unit and type rows aligned with the data", () => {
    for (const g of SEEDED) {
      expect(g.units).toHaveLength(g.headings.length);
      expect(g.types).toHaveLength(g.headings.length);
      for (const row of g.rows) expect(row).toHaveLength(g.headings.length);
    }
  });
});

describe("emit", () => {
  it("round-trips the committed fixture byte for byte", () => {
    // The one that guarantees the findings line up with the rendered file.
    expect(emit(SEEDED)).toBe(seededText);
  });

  it("skips lines that arrive before any GROUP", () => {
    // AGS4 files in the wild open with a GROUP line, but a truncated paste into
    // the demo would not. Dropping the orphaned rows beats attributing them to
    // whichever group happens to come next.
    const stray = parse(
      '"DATA","x"\r\n"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n',
    );
    expect(stray).toHaveLength(1);
    expect(stray[0]?.code).toBe("PROJ");
    expect(stray[0]?.rows).toHaveLength(0);
  });

  it("re-quotes a value containing a quote", () => {
    const one = parse(
      '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","a""b"\r\n',
    );
    expect(one.at(0)?.rows.at(0)?.at(0)).toBe('a"b');
    expect(emit(one)).toContain('"a""b"');
  });
});

describe("lineOfRow", () => {
  it("points at the line the emitted file actually puts the row on", () => {
    const lines = emit(SEEDED).split("\r\n");
    for (const g of SEEDED) {
      for (let i = 0; i < g.rows.length; i++) {
        const at = lineOfRow(SEEDED, g.code, i);
        const row = g.rows[i];
        if (!row) throw new Error("row disappeared");
        expect(lines[at - 1]).toBe(
          ['"DATA"', ...row.map((c) => `"${c}"`)].join(","),
        );
      }
    }
  });

  it("agrees with the validator's line for the seeded Rule 8 defect", () => {
    // The literal the engine reports for the bad LOCA_GL — pinned because the
    // page bands that line from this function, so a drift here would band the
    // wrong row while the finding text stayed correct. The number lives ONLY
    // in the assertion; tests/test_landing_demo_delivery.py holds the other
    // half of the contract against the real validator.
    expect(lineOfRow(SEEDED, "LOCA", 0)).toBe(17);
  });

  it("answers -1 for a group the delivery does not carry", () => {
    // Deliberately out of range rather than a plausible number: the output pane
    // bands whatever line this returns, and a silently-wrong line would mark a
    // row the finding is not about — worse than marking nothing.
    expect(lineOfRow(SEEDED, "GEOL", 0)).toBe(-1);
  });
});

describe("setCell", () => {
  it("replaces one cell and leaves the rest identical", () => {
    const next = setCell(SEEDED, "LOCA", 0, 2, "11.80");
    expect(emit(next)).toContain('"11.80"');
    expect(emit(next)).not.toContain('"11.8"');
    // Everything else survives — a naive rebuild would drop the group order.
    expect(next.map((g) => g.code)).toEqual(SEEDED.map((g) => g.code));
  });

  it("does not mutate the seeded delivery", () => {
    const before = emit(SEEDED);
    setCell(SEEDED, "LOCA", 0, 2, "99.99");
    expect(emit(SEEDED)).toBe(before);
  });

  it("answers the same delivery for a value that already matches", () => {
    // The undo contract rides on this identity: commit() skips history when
    // a mutation changed nothing, so an untouched editor's Enter must not
    // cost the reader an undo step (#525).
    const current = SEEDED.find((g) => g.code === "PROJ")?.rows[0]?.[0] ?? "";
    expect(setCell(SEEDED, "PROJ", 0, 0, current)).toBe(SEEDED);
  });
});

describe("addRow", () => {
  it("inherits the KEY values the parent chain determines", () => {
    const next = addRow(SEEDED, "SAMP", "LOCA", keyHeadings("SAMP"));
    const samp = next.find((g) => g.code === "SAMP")!;
    const added = samp.rows.at(-1)!;
    const locaIdAt = samp.headings.indexOf("LOCA_ID");

    // The last LOCA row is BH02, so a new SAMP hangs off BH02 without retyping.
    expect(added[locaIdAt]).toBe("BH02");
    // Its own KEY fields stay blank — they are the reader's to fill.
    expect(added[samp.headings.indexOf("SAMP_REF")]).toBe("");
  });

  it("leaves a rootless group's row entirely blank", () => {
    const next = addRow(SEEDED, "PROJ", null, keyHeadings("PROJ"));
    const proj = next.find((g) => g.code === "PROJ")!;
    expect(proj.rows.at(-1)!.every((c) => c === "")).toBe(true);
  });

  it("keeps the row the same width as the headings", () => {
    const next = addRow(SEEDED, "LLPL", "SAMP", keyHeadings("LLPL"));
    const llpl = next.find((g) => g.code === "LLPL")!;
    expect(llpl.rows.at(-1)).toHaveLength(llpl.headings.length);
  });
});

describe("the model and the generated schema agree", () => {
  it("renders every heading the dictionary slice defines, in its order", () => {
    for (const g of DEMO_GROUPS) {
      const parsed = SEEDED.find((x) => x.code === g.code)!;
      expect(parsed.headings).toEqual(g.headings.map((h) => h.name));
    }
  });

  it("has no KEY set for a group the demo does not draw", () => {
    // `addRow` passes this straight into the inherit step, so an empty list has
    // to mean "inherit nothing" rather than throwing on a group the slice omits.
    expect(keyHeadings("GEOL")).toEqual([]);
  });

  it("marks the same KEY set the dictionary does", () => {
    expect(keyHeadings("LLPL")).toEqual([
      "LOCA_ID",
      "SAMP_TOP",
      "SAMP_REF",
      "SAMP_TYPE",
      "SAMP_ID",
      "SPEC_REF",
      "SPEC_DPTH",
    ]);
  });
});

describe("seededFinalDepth", () => {
  it("reads the rail's total out of the delivery rather than a constant", () => {
    expect(seededFinalDepth()).toBe(25);
  });

  it("follows the fixture if the seeded depth changes", () => {
    const moved = setCell(
      SEEDED,
      "LOCA",
      0,
      SEEDED.find((g) => g.code === "LOCA")!.headings.indexOf("LOCA_FDEP"),
      "31.25",
    );
    expect(seededFinalDepth(moved)).toBe(31.25);
  });
});

describe("deleteRow", () => {
  it("removes exactly the named row and leaves every other group alone", () => {
    const next = deleteRow(SEEDED, "SAMP", 1);
    const samp = next.find((g) => g.code === "SAMP");
    const before = SEEDED.find((g) => g.code === "SAMP");
    expect(samp?.rows).toHaveLength((before?.rows.length ?? 0) - 1);
    expect(samp?.rows.map((r) => r[2])).toEqual(["S1", "S1"]);
    for (const g of next) {
      if (g.code === "SAMP") continue;
      expect(g).toBe(SEEDED.find((o) => o.code === g.code));
    }
  });

  it("only DATA rows: the group's declaration rows are untouchable by construction", () => {
    // deleteRow indexes into `rows`, which parse() built from DATA lines only —
    // pinned so a refactor storing declaration rows alongside them goes red.
    const next = deleteRow(SEEDED, "PROJ", 0);
    const proj = next.find((g) => g.code === "PROJ");
    expect(proj?.rows).toHaveLength(0);
    expect(proj?.headings).toEqual(
      SEEDED.find((g) => g.code === "PROJ")?.headings,
    );
    expect(emit(next)).toContain('"GROUP","PROJ"');
    expect(emit(next)).toContain('"TYPE","ID","X","X"');
  });

  it("answers the same delivery for a row or group that is not there", () => {
    expect(deleteRow(SEEDED, "PROJ", 5)).toBe(SEEDED);
    expect(deleteRow(SEEDED, "NOPE", 0)).toBe(SEEDED);
  });
});

describe("groupOfLine", () => {
  it("maps every line of a group's block to its code, blanks to null", () => {
    // PROJ owns lines 1-5 (GROUP/HEADING/UNIT/TYPE/DATA); line 6 is the
    // separator; TRAN's block starts at 7. Derived from the same walk as
    // lineOfRow, and pinned against it so the two can never disagree.
    expect(groupOfLine(SEEDED, 1)).toBe("PROJ");
    expect(groupOfLine(SEEDED, 5)).toBe("PROJ");
    expect(groupOfLine(SEEDED, 6)).toBeNull();
    expect(groupOfLine(SEEDED, 7)).toBe("TRAN");
    for (const g of SEEDED) {
      for (let r = 0; r < g.rows.length; r++) {
        expect(groupOfLine(SEEDED, lineOfRow(SEEDED, g.code, r))).toBe(g.code);
      }
    }
  });

  it("answers null off the end of the file and on line 0", () => {
    expect(groupOfLine(SEEDED, 0)).toBeNull();
    expect(groupOfLine(SEEDED, 10_000)).toBeNull();
  });
});

describe("deleteGroup", () => {
  it("removes the whole group and leaves the survivors in file order", () => {
    const next = deleteGroup(SEEDED, "TRAN");
    expect(next.find((g) => g.code === "TRAN")).toBeUndefined();
    expect(next.map((g) => g.code)).toEqual(
      SEEDED.map((g) => g.code).filter((c) => c !== "TRAN"),
    );
  });

  it("answers the same delivery for a group that is not there", () => {
    expect(deleteGroup(SEEDED, "NOPE")).toBe(SEEDED);
  });
});

describe("restoreGroup", () => {
  it("round-trips: delete then restore emits the seeded file byte-for-byte", () => {
    // Position matters, not just membership — the group must come back where
    // the seed had it, or the emitted file (and every finding line number
    // derived from it) shifts.
    const next = restoreGroup(deleteGroup(SEEDED, "TRAN"), "TRAN");
    expect(emit(next)).toBe(emit(SEEDED));
  });

  it("returns the SEEDED rows, not the pre-delete edits", () => {
    // Restore-to-seeded is the honest contract (#529): keeping edits through
    // a delete/restore cycle would need shadow state that can rot.
    const edited = setCell(SEEDED, "TRAN", 0, 3, "Draft");
    const next = restoreGroup(deleteGroup(edited, "TRAN"), "TRAN");
    const tran = next.find((g) => g.code === "TRAN");
    const col = tran?.headings.indexOf("TRAN_STAT") ?? -1;
    expect(tran?.rows[0]?.[col]).toBe("Final");
  });

  it("inserts by seeded order relative to the groups still standing", () => {
    // With LOCA also gone, TRAN's seeded neighbour is missing — the insert
    // must fall back to the next surviving group, not throw or append.
    const gutted = deleteGroup(deleteGroup(SEEDED, "LOCA"), "TRAN");
    const next = restoreGroup(gutted, "TRAN");
    const codes = next.map((g) => g.code);
    expect(codes.indexOf("TRAN")).toBe(codes.indexOf("PROJ") + 1);
    expect(codes.indexOf("TRAN")).toBeLessThan(codes.indexOf("SAMP"));
  });

  it("any delete set, restored in any order, reproduces the seeded file", () => {
    // The property the insertion logic exists for: the delivery is always a
    // seed-order subsequence, so restores commute. Both orders, byte-for-byte.
    const gutted = deleteGroup(deleteGroup(SEEDED, "LOCA"), "TRAN");
    const locaFirst = restoreGroup(restoreGroup(gutted, "LOCA"), "TRAN");
    const tranFirst = restoreGroup(restoreGroup(gutted, "TRAN"), "LOCA");
    expect(emit(locaFirst)).toBe(emit(SEEDED));
    expect(emit(tranFirst)).toBe(emit(SEEDED));
  });

  it("answers the same delivery when the group is present or unknown", () => {
    expect(restoreGroup(SEEDED, "TRAN")).toBe(SEEDED);
    expect(restoreGroup(SEEDED, "NOPE")).toBe(SEEDED);
  });
});
