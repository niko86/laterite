/* #1024: the did-you-mean folds under the Rule 16 error it explains.
 *
 * Fixtures are copies of what the engine emits for the seeded delivery with
 * FYI on (verified against `lat validate --show-fyi` on seeded-delivery.ags):
 * the undefined "b" is reported against SAMP and against LLPL, and so is its
 * suggestion. These pin the PAIRING, not the wording: a rephrased message
 * leaves this suite green, and the landing e2e running the real wasm is what
 * goes red on one.
 *
 * Each surface hands `foldSuggestions` the slice it renders, so each gets a
 * test with its own slice: the panel (the whole report), the group strip
 * (heading-less findings of one group), the field card (the cell's findings
 * plus the group findings naming its heading) and the cell popover (the
 * cell's findings, suggestions looked up in the whole report).
 */

import { describe, expect, it } from "vitest";
import type { Finding } from "./engine";
import {
  foldSuggestions,
  suggestionTarget,
  suggestionText,
} from "./suggestions";

const error = (group: string, value = "b", heading = "SAMP_TYPE"): Finding => ({
  rule: "AGS Format Rule 16",
  line: null,
  group,
  heading: null,
  dataRow: null,
  severity: "error",
  desc: `Abbreviation "${value}" under ${heading} is not defined in the ABBR group.`,
});

const suggestion = (
  group: string,
  value = "b",
  heading = "SAMP_TYPE",
  code = "B",
): Finding => ({
  rule: "FYI (Related to Rule 16)",
  line: null,
  group,
  heading: null,
  dataRow: null,
  severity: "fyi",
  desc: `"${value}" under ${heading} is not defined; did you mean "${code}" (declared in ABBR)?`,
});

const rule8: Finding = {
  rule: "AGS Format Rule 8",
  line: 17,
  group: "LOCA",
  heading: "LOCA_GL",
  dataRow: 1,
  severity: "error",
  desc: 'Value "11.8" in LOCA_GL does not match its declared TYPE "2DP".',
};

/** Another Rule 16 FYI that is NOT a suggestion (O-58's case collision). */
const collision: Finding = {
  rule: "FYI (Related to Rule 16)",
  line: null,
  group: "ABBR",
  heading: null,
  dataRow: null,
  severity: "fyi",
  desc: 'SAMP_TYPE: codes "B" and "b" differ only by letter case; some importers treat them as the same code.',
};

/** The seed's report, in the engine's own order (errors, then FYIs). */
const REPORT = [
  rule8,
  error("SAMP"),
  error("LLPL"),
  suggestion("SAMP"),
  suggestion("LLPL"),
];

describe("suggestionTarget", () => {
  it("reads the value, the heading and the clause out of the suggestion", () => {
    expect(suggestionTarget(suggestion("SAMP"))).toEqual({
      value: "b",
      heading: "SAMP_TYPE",
      clause: 'did you mean "B" (declared in ABBR)?',
    });
  });

  it("answers null for the error it explains and for any other FYI", () => {
    expect(suggestionTarget(error("SAMP"))).toBeNull();
    expect(suggestionTarget(collision)).toBeNull();
  });

  it("stands the clause as a sentence under the error", () => {
    expect(suggestionText(suggestion("SAMP"))).toBe(
      'Did you mean "B" (declared in ABBR)?',
    );
  });
});

describe("the findings panel", () => {
  it("puts each suggestion under its own error and shows no card for it", () => {
    const cards = foldSuggestions(REPORT);
    expect(cards.map((c) => c.finding)).toEqual([
      rule8,
      error("SAMP"),
      error("LLPL"),
    ]);
    expect(cards.map((c) => c.suggestion)).toEqual([
      null,
      suggestion("SAMP"),
      suggestion("LLPL"),
    ]);
  });

  it("pairs by what the messages name, not by position", () => {
    // The two suggestions arrive in the opposite order to their errors, and a
    // second undefined value sits between them: a positional zip would hand
    // SAMP's error LLPL's suggestion and the "x" error a "b" suggestion.
    const cards = foldSuggestions([
      error("SAMP"),
      error("SAMP", "x"),
      error("LLPL"),
      suggestion("LLPL"),
      suggestion("SAMP", "x", "SAMP_TYPE", "X"),
      suggestion("SAMP"),
    ]);
    // Compared whole, group included: SAMP's and LLPL's suggestions carry
    // the same text, so a swap between groups is invisible in the prose.
    expect(cards).toEqual([
      { finding: error("SAMP"), suggestion: suggestion("SAMP") },
      {
        finding: error("SAMP", "x"),
        suggestion: suggestion("SAMP", "x", "SAMP_TYPE", "X"),
      },
      { finding: error("LLPL"), suggestion: suggestion("LLPL") },
    ]);
  });

  it("keeps any other FYI as its own card", () => {
    const cards = foldSuggestions([...REPORT, collision]);
    expect(cards.at(-1)).toEqual({ finding: collision, suggestion: null });
  });

  it("keeps a suggestion whose error is not on the surface as its own card", () => {
    // Never folded into nothing: a suggestion only disappears as a card when
    // the error it explains is shown on the same surface.
    const cards = foldSuggestions([suggestion("SAMP")]);
    expect(cards).toEqual([{ finding: suggestion("SAMP"), suggestion: null }]);
  });
});

describe("the group strip", () => {
  it("folds the group's suggestion under the group's error", () => {
    // findingsForGroup's slice: SAMP's heading-less findings.
    const slice = REPORT.filter(
      (f) => f.group === "SAMP" && f.heading === null,
    );
    expect(foldSuggestions(slice)).toEqual([
      { finding: error("SAMP"), suggestion: suggestion("SAMP") },
    ]);
  });
});

describe("the field card", () => {
  it("folds the suggestion under the error on the SAMP_TYPE card", () => {
    // RowCarousel's slice: the cell's findings (the Rule 16 error, mapped onto
    // the carrying cell) plus the group findings naming SAMP_TYPE, which is
    // how the suggestion reaches the card at all.
    // The same objects the report holds, as findingsForCell hands them over.
    const failing = REPORT.filter(
      (f) => f.rule === "AGS Format Rule 16" && f.group === "SAMP",
    );
    const named = REPORT.filter(
      (f) =>
        f.group === "SAMP" &&
        f.heading === null &&
        f.desc.includes("SAMP_TYPE") &&
        !failing.includes(f),
    );
    const cards = foldSuggestions([...failing, ...named]);
    expect(cards).toHaveLength(1);
    expect(cards[0]?.suggestion).toEqual(suggestion("SAMP"));
  });
});

describe("the cell popover", () => {
  it("finds the suggestion in the whole report when the cell's slice lacks it", () => {
    // findingsForCell maps the Rule 16 error onto the cell, but the
    // suggestion names no cell, so the popover passes the report as the pool.
    expect(foldSuggestions([error("LLPL")], REPORT)).toEqual([
      { finding: error("LLPL"), suggestion: suggestion("LLPL") },
    ]);
    expect(foldSuggestions([error("LLPL")])).toEqual([
      { finding: error("LLPL"), suggestion: null },
    ]);
  });
});
