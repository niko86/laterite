import { splitAgsFields } from "./agsline";
import type { SpanEdit } from "./validator";

// The geometry behind a fix's aligned before/after preview, factored out of
// FixesPanel so it's unit-testable (the "highlight EXACTLY the changed chars"
// promise is easy to break silently). Given a raw line and the span edit, this
// computes the spliced AFTER line, which field the edit lands in, and the
// cell-relative highlight ranges for the del (before) and ins (after) rows.

export interface FixHighlight {
  /** The line as it reads once the replacement is spliced into [start, end). */
  after: string;
  /** Index of the field the edit lands in, or -1 when it anchors inside no
   *  original field (e.g. an end-of-line append by a row-padding fix). */
  changedCol: number;
  /** Cell-relative [start, end) to highlight in the del (before) row, or null
   *  when there's no single enclosing field (an append). */
  delHl: [number, number] | null;
  /** Cell-relative [start, end) to highlight in the ins (after) row. */
  insHl: [number, number] | null;
  /** For an end-of-line append (changedCol < 0): the first appended field
   *  index in the AFTER line — those cells are highlighted wholesale (there's
   *  no original sub-span to paint). Null when it isn't an append. */
  appendFrom: number | null;
}

export function fixHighlight(line: string, edit: SpanEdit): FixHighlight {
  const cps = Array.from(line);
  const s = Math.max(0, Math.min(edit.start, cps.length));
  const e = Math.max(s, Math.min(edit.end, cps.length));
  const after =
    cps.slice(0, s).join("") + edit.replacement + cps.slice(e).join("");

  // Which field the edit lands in, matched by char offset (tag-agnostic).
  const fields = splitAgsFields(line);
  const changedCol = fields.findIndex(
    (f) => edit.start >= f.start && edit.start < f.end,
  );

  // The cell's padded text starts with the field token, so a raw-line offset
  // rebases by the field's `start`. del = the old span; ins = the replacement
  // span (same start, replacement length).
  let delHl: [number, number] | null = null;
  if (changedCol >= 0) {
    const f = fields[changedCol]; // findIndex ≥ 0 → in-bounds.
    if (f)
      delHl = [
        Math.max(0, edit.start - f.start),
        Math.max(0, edit.end - f.start),
      ];
  }

  const afterFields = splitAgsFields(after);
  let insHl: [number, number] | null = null;
  if (changedCol >= 0 && changedCol < afterFields.length) {
    const f = afterFields[changedCol]; // bounds checked in the guard.
    if (f) {
      const cs = Math.max(0, edit.start - f.start);
      insHl = [cs, cs + Array.from(edit.replacement).length];
    }
  }

  // Row-padding appends past end-of-line (inside no original field). Flag the
  // first appended field of the AFTER line so the ins row can highlight the new
  // cells wholesale rather than rendering them un-marked.
  let appendFrom: number | null = null;
  if (changedCol < 0 && edit.start >= cps.length) {
    const idx = afterFields.findIndex((f) => f.start >= edit.start);
    appendFrom = idx >= 0 ? idx : null;
  }

  return { after, changedCol, delHl, insHl, appendFrom };
}
