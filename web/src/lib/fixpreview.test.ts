import { describe, expect, it } from "vitest";
import { fixHighlight } from "./fixpreview";
import type { SpanEdit } from "./validator";

const edit = (
  start: number,
  end: number,
  replacement: string,
  line = 1,
): SpanEdit => ({ line, start, end, replacement, expected: "" });

// The highlight spans are cell-RELATIVE (offset within the aligned cell's
// padded text, which starts at the field's leading quote). These pin down the
// "highlight exactly the changed chars" promise.

describe("fixHighlight — in-field reformat (Rule 8: 123.4 → 123.40)", () => {
  // Line: "DATA","BH01","123.4"  — the value 123.4 is at raw offsets 21..26.
  const line = '"DATA","BH01","123.4"';
  // Field 2 ("...","123.4") starts at offset 14; the value sits at 15..20.
  // The reformat edit replaces the inner value [15,20) with "123.40".
  const h = fixHighlight(line, edit(15, 20, "123.40"));

  it("splices the new value into the after line", () => {
    expect(h.after).toBe('"DATA","BH01","123.40"');
  });

  it("lands the edit in field 2", () => {
    expect(h.changedCol).toBe(2);
  });

  it("del span is cell-relative (rebased onto the field's leading quote)", () => {
    // field starts at 14, edit at [15,20) → cell-relative [1,6).
    expect(h.delHl).toEqual([1, 6]);
  });

  it("ins span covers exactly the replacement length", () => {
    // same start (1), replacement "123.40" is 6 chars → [1,7).
    expect(h.insHl).toEqual([1, 7]);
  });

  it("is not an append", () => {
    expect(h.appendFrom).toBeNull();
  });
});

describe("fixHighlight — row padding (append past end of line)", () => {
  // A short 2-field row padded to 4 fields: the edit anchors at end-of-line.
  const line = '"DATA","BH01"';
  const h = fixHighlight(line, edit(line.length, line.length, ',"",""'));

  it("appends the new cells in the after line", () => {
    expect(h.after).toBe('"DATA","BH01","",""');
  });

  it("has no single changed field (anchors past every field)", () => {
    expect(h.changedCol).toBe(-1);
    expect(h.delHl).toBeNull();
    expect(h.insHl).toBeNull();
  });

  it("flags the first appended field so the ins row highlights it wholesale", () => {
    // after = "DATA","BH01","","" → fields 2 and 3 are the appended ones.
    expect(h.appendFrom).toBe(2);
  });
});

describe("fixHighlight — astral chars don't desync offsets", () => {
  it("rebases correctly past a surrogate pair", () => {
    // "DATA","😀","x" — replace the x (a 1-char value) in field 2.
    const line = '"DATA","😀","x"';
    const cps = Array.from(line);
    const xPos = cps.lastIndexOf("x");
    const h = fixHighlight(line, edit(xPos, xPos + 1, "y"));
    expect(h.after).toBe('"DATA","😀","y"');
    expect(h.changedCol).toBe(2);
    expect(h.delHl).toEqual([1, 2]);
    expect(h.insHl).toEqual([1, 2]);
  });
});
