// #294 Batch E (fluent layer) — the chained verbs on Ags4File: `.validate()`
// (chainable, report on `.report`), `.fix()` (→ new repaired handle, on
// `.fixReport`), `.diff()` (→ RevisionDelta). They reuse the free fns with the
// handle's RETAINED read source, so line numbers + encoding match the original.
import { describe, expect, it } from "vitest";
import { Ags4File, type Report, read } from "../ts/index";

const CLEAN = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID","PROJ_NAME"',
  '"UNIT","",""',
  '"TYPE","ID","X"',
  '"DATA","P1","Demo"',
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_GL"',
  '"UNIT","","m"',
  '"TYPE","ID","2DP"',
  '"DATA","BH01","12.30"',
  "",
].join("\r\n");

// LOCA_GL "1.0" under a 2DP heading is a Rule 8 reformat AutoFix repairs to "1.00".
const FIXABLE = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID"',
  '"UNIT",""',
  '"TYPE","ID"',
  '"DATA","P1"',
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_GL"',
  '"UNIT","","m"',
  '"TYPE","ID","2DP"',
  '"DATA","BH01","1.0"',
  "",
].join("\r\n");

describe("Ags4File.validate — chainable, report on .report", () => {
  it("returns the same handle and lands the Report on .report", () => {
    const ags = read(undefined, { text: CLEAN });
    expect(ags.report).toBeUndefined(); // not validated yet
    const same = ags.validate();
    expect(same).toBe(ags); // chainable — returns this
    const rep = ags.report as Report;
    expect(typeof rep.count).toBe("number");
    expect(rep.count).toBe(rep.findings.length);
    // Matches the free validate() over the same source.
    expect(rep.count).toBe(
      read(undefined, { text: CLEAN }).validate().report?.count,
    );
  });

  it("honours the warnings gate like the free fn", () => {
    const withWarnings =
      read(undefined, { text: CLEAN }).validate().report?.count ?? 0;
    const errorsOnly =
      read(undefined, { text: CLEAN }).validate({ warnings: false }).report
        ?.count ?? 0;
    expect(errorsOnly).toBeLessThanOrEqual(withWarnings);
  });
});

describe("Ags4File.fix — → new repaired handle, report on .fixReport", () => {
  it("repairs the Rule 8 value and rides the FixResult on .fixReport", () => {
    const ags = read(undefined, { text: FIXABLE });
    const repaired = ags.fix();
    expect(repaired).toBeInstanceOf(Ags4File);
    expect(repaired).not.toBe(ags); // a new handle, non-destructive
    expect(repaired.fixReport?.fixesApplied).toBeGreaterThan(0);
    expect(repaired.fixReport?.applied[0]).toMatchObject({
      rule: "AGS Format Rule 8",
    });
    // The repaired document carries the canonical 2DP value.
    expect(repaired.text).toContain('"DATA","BH01","1.00"');
    // …and the original handle is untouched.
    expect(ags.text).toContain('"DATA","BH01","1.0"');
  });

  it("chains read → fix → validate → save-able bytes", () => {
    const repaired = read(undefined, { text: FIXABLE }).fix().validate();
    expect(repaired.report).toBeDefined();
    expect(Buffer.isBuffer(repaired.bytes)).toBe(true);
  });
});

describe("Ags4File.diff — baseline = self", () => {
  it("agrees with the free diff and reports the changed cell", () => {
    const revision = CLEAN.replace(
      '"DATA","BH01","12.30"',
      '"DATA","BH01","13.30"',
    );
    const delta = read(undefined, { text: CLEAN }).diff(Buffer.from(revision));
    expect(delta.total_changed).toBe(1);
    const loca = delta.groups.find((g) => g.code === "LOCA");
    const changed = loca?.rows.find((r) => r.kind === "changed");
    expect(changed?.cells[0]).toMatchObject({
      heading: "LOCA_GL",
      a: "12.30",
      b: "13.30",
    });
    // no diff against itself
    expect(
      read(undefined, { text: CLEAN }).diff(Buffer.from(CLEAN)).total_changed,
    ).toBe(0);
  });
});
