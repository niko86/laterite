/* The finding→cell mapping (#590), pinned on fixtures shaped exactly like
 * what the engine emits for the seeded delivery (verified against the
 * shipped CLI's --json for the same file). Rule 16's message TEXT is the
 * adapter's contract — the finding carries no heading and no row — and this
 * file pins the ADAPTER against that wording, not the wording against the
 * engine: these fixtures are copies, so an engine that rephrases the
 * message leaves this suite green. The e2e that runs the real wasm against
 * the seeded delivery ("the bad abbreviation lights its cell") is what goes
 * red on a rephrase. */

import { describe, expect, it } from "vitest";
import { abbreviationCells, abbreviationTarget, type Finding } from "./engine";

const rule16 = (over: Partial<Finding> = {}): Finding => ({
  rule: "AGS Format Rule 16",
  line: null,
  group: "SAMP",
  heading: null,
  dataRow: null,
  severity: "error",
  desc: 'Abbreviation "b" under SAMP_TYPE is not defined in the ABBR group.',
  ...over,
});

const HEADINGS = ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"];
const ROWS = [
  ["BH01", "1.50", "S1", "b", "BH01-S1"],
  ["BH01", "3.00", "S2", "D", "BH01-S2"],
  ["BH02", "2.00", "S1", "b", "BH02-S1"],
];

describe("abbreviationTarget", () => {
  it("reads the value and the heading out of Rule 16's prose", () => {
    expect(abbreviationTarget(rule16())).toEqual({
      value: "b",
      heading: "SAMP_TYPE",
    });
  });

  it("answers null for a cell-addressed finding — Rule 8 already knows its cell", () => {
    expect(
      abbreviationTarget(
        rule16({
          rule: "AGS Format Rule 8",
          heading: "LOCA_GL",
          dataRow: 1,
          desc: 'Value "11.8" in LOCA_GL does not match its declared TYPE "2DP".',
        }),
      ),
    ).toBeNull();
  });

  it("answers null for the row-pinned orphan — Rule 10c is the row grammar's, not this mapping's", () => {
    expect(
      abbreviationTarget(
        rule16({
          rule: "AGS Format Rule 10c",
          dataRow: 3,
          desc: "No parent entry in SAMP for KEY combination: BH02|4.50|S3|D|BH02-S3",
        }),
      ),
    ).toBeNull();
  });

  it("answers null when Rule 16's message no longer parses — the loud path is the test above failing", () => {
    expect(abbreviationTarget(rule16({ desc: "something else" }))).toBeNull();
  });
});

describe("abbreviationCells", () => {
  it("maps to EVERY cell carrying the value — one row would be a lie when two carry it", () => {
    expect(abbreviationCells(rule16(), HEADINGS, ROWS)).toEqual([
      { row: 0, col: 3 },
      { row: 2, col: 3 },
    ]);
  });

  it("maps to nothing when the named heading is not in this block", () => {
    expect(
      abbreviationCells(rule16(), ["LOCA_ID", "LOCA_GL"], [["BH01", "b"]]),
    ).toEqual([]);
  });

  it("maps to nothing when no cell carries the value any more — the fixed state", () => {
    const fixed = ROWS.map((r) => r.map((v) => (v === "b" ? "D" : v)));
    expect(abbreviationCells(rule16(), HEADINGS, fixed)).toEqual([]);
  });

  it("maps nothing for a finding that is not Rule 16's shape", () => {
    expect(abbreviationCells(rule16({ dataRow: 1 }), HEADINGS, ROWS)).toEqual(
      [],
    );
  });
});
