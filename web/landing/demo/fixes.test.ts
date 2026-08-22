/* The pure half of per-table autofix (#530): scoping the engine's fix list to
 * one group, and deciding which findings the fixer will never touch. Synthetic
 * fix records here — the real ones come from the wasm engine, whose shape the
 * FixLike type narrows to exactly what these functions read.
 */

import { describe, expect, it } from "vitest";
import { SEEDED, lineOfRow } from "./delivery";
import { fixesForGroup, isManual } from "./fixes";

const fix = (rule: string, line: number | null) => ({ rule, line });

describe("fixesForGroup", () => {
  it("keeps only the fixes whose anchor line falls inside the group's block", () => {
    const locaLine = lineOfRow(SEEDED, "LOCA", 0);
    const sampLine = lineOfRow(SEEDED, "SAMP", 0);
    const fixes = [
      fix("AGS Format Rule 8", locaLine),
      fix("AGS Format Rule 8", sampLine),
    ];
    expect(fixesForGroup(fixes, SEEDED, "LOCA")).toEqual([
      fix("AGS Format Rule 8", locaLine),
    ]);
    expect(fixesForGroup(fixes, SEEDED, "TRAN")).toEqual([]);
  });

  it("assigns a whole-file fix (null line) to NO group — no table's button may apply it", () => {
    const fixes = [fix("AGS Format Rule 1", null)];
    for (const g of SEEDED) {
      expect(fixesForGroup(fixes, SEEDED, g.code)).toEqual([]);
    }
  });
});

describe("isManual", () => {
  it("a finding is manual exactly when no fix shares its (rule, line) identity", () => {
    const locaLine = lineOfRow(SEEDED, "LOCA", 0);
    const fixes = [fix("AGS Format Rule 8", locaLine)];
    expect(isManual({ rule: "AGS Format Rule 8", line: locaLine }, fixes)).toBe(
      false,
    );
    // Same rule, different line: the fix resolves THAT finding, not this one.
    expect(
      isManual({ rule: "AGS Format Rule 8", line: locaLine + 1 }, fixes),
    ).toBe(true);
    expect(isManual({ rule: "AGS Format Rule 16", line: 3 }, fixes)).toBe(true);
  });

  it("matches whole-file findings against whole-file fixes by rule", () => {
    const fixes = [fix("AGS Format Rule 1", null)];
    expect(isManual({ rule: "AGS Format Rule 1", line: null }, fixes)).toBe(
      false,
    );
    expect(isManual({ rule: "AGS Format Rule 14", line: null }, fixes)).toBe(
      true,
    );
  });
});
