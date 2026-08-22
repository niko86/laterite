/* The seeded delivery as an editable model, and back to AGS4 text (#396–#398).
 *
 * Pure functions over plain data, with no Solid and no DOM, for two reasons.
 * The obvious one is that they are testable in the node lane. The load-bearing
 * one is that `emit()` is the half the ENGINE sees: the page validates the text
 * this produces, so if emit and the tables disagree the findings point at lines
 * the reader is not looking at. Keeping it a function of the model alone is what
 * makes that impossible rather than merely unlikely.
 *
 * Only DATA rows are editable. GROUP / HEADING / UNIT / TYPE are the file's
 * declaration of its own shape, and letting a reader edit a TYPE row would let
 * them change what Rule 8 compares against — which turns "the value is wrong"
 * into "the schema is wrong" and teaches the opposite of the lesson.
 */

import seeded from "./seeded-delivery.ags?raw";

export type Row = readonly string[];

export type Group = {
  readonly code: string;
  /** The HEADING row, minus its leading "HEADING" cell. */
  readonly headings: readonly string[];
  readonly units: readonly string[];
  readonly types: readonly string[];
  /** DATA rows, each already stripped of its leading "DATA" cell. */
  readonly rows: readonly Row[];
};

export type Delivery = readonly Group[];

/** AGS4 is CRLF and every field is double-quoted, with a literal quote doubled.
 *  Parsing is deliberately narrow — this reads the file THIS page ships, not
 *  arbitrary AGS4. The tolerant tokenizer is the engine's job. */
function splitLine(line: string): string[] {
  const out: string[] = [];
  let cell = "";
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line.charAt(i);
    if (ch === '"') {
      if (inQuotes && line.charAt(i + 1) === '"') {
        cell += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (ch === "," && !inQuotes) {
      out.push(cell);
      cell = "";
    } else {
      cell += ch;
    }
  }
  out.push(cell);
  return out;
}

export function parse(text: string): Delivery {
  const groups: Group[] = [];
  let current: {
    code: string;
    headings: string[];
    units: string[];
    types: string[];
    rows: string[][];
  } | null = null;

  for (const raw of text.split(/\r?\n/)) {
    if (!raw.trim()) continue;
    const cells = splitLine(raw);
    const [tag, ...rest] = cells;
    if (tag === "GROUP") {
      current = {
        code: rest[0] ?? "",
        headings: [],
        units: [],
        types: [],
        rows: [],
      };
      groups.push(current);
    } else if (!current) {
      continue;
    } else if (tag === "HEADING") {
      current.headings = rest;
    } else if (tag === "UNIT") {
      current.units = rest;
    } else if (tag === "TYPE") {
      current.types = rest;
    } else if (tag === "DATA") {
      current.rows.push(rest);
    }
  }
  return groups;
}

function quote(cell: string): string {
  return `"${cell.replaceAll('"', '""')}"`;
}

export function emit(delivery: Delivery): string {
  const lines: string[] = [];
  for (const g of delivery) {
    if (lines.length) lines.push("");
    lines.push(["GROUP", g.code].map(quote).join(","));
    lines.push(["HEADING", ...g.headings].map(quote).join(","));
    lines.push(["UNIT", ...g.units].map(quote).join(","));
    lines.push(["TYPE", ...g.types].map(quote).join(","));
    for (const row of g.rows) lines.push(["DATA", ...row].map(quote).join(","));
  }
  return lines.join("\r\n") + "\r\n";
}

/** The 1-based line number a group's Nth DATA row occupies in `emit()`'s output.
 *  The findings list uses this in reverse (line → cell) to jump, and the output
 *  pane uses it to band. Computed rather than tracked, so it cannot go stale
 *  when a row is added. */
export function lineOfRow(
  delivery: Delivery,
  code: string,
  rowIndex: number,
): number {
  let line = 0;
  for (const g of delivery) {
    if (line) line += 1; // the blank separator between groups
    if (g.code === code) return line + 5 + rowIndex; // GROUP/HEADING/UNIT/TYPE
    line += 4 + g.rows.length;
  }
  return -1;
}

/** The group whose block owns 1-based line `n`, or null for the separators
 *  and anything past the end. The same walk as lineOfRow — 4 declaration
 *  lines + the DATA rows, one blank between groups — kept adjacent so the two
 *  cannot drift apart. #530 uses it to scope the engine's fix list, whose
 *  records carry a line but not a group. */
export function groupOfLine(delivery: Delivery, n: number): string | null {
  let line = 0;
  for (const g of delivery) {
    if (line) line += 1; // the blank separator between groups
    const last = line + 4 + g.rows.length;
    if (n > line && n <= last) return g.code;
    line = last;
  }
  return null;
}

/** The value a single-line `<input>` would hold, so the clipboard handler can
 *  agree with every other entry path (#574).
 *
 *  Why this matters: one cell holding a terminator tears a DATA record in two,
 *  because `quote()` doubles an embedded quote and escapes nothing else, so the
 *  byte reaches the text raw. The native paths — the in-place editor, the
 *  carousel's field cards — are sanitized by the browser before we ever see the
 *  value. The handler reads the clipboard itself, so it is the one path that
 *  has to do it by hand.
 *
 *  The rule is TAKEN FROM the browser, not from the spec: an interior break
 *  becomes a SPACE (not nothing), a CRLF pair counts as one break, and a
 *  TRAILING RUN of breaks — however many — is dropped entirely. Reading HTML's
 *  value sanitization algorithm suggests stripping every break instead, and
 *  that is wrong for a paste. The `+` on the trailing match is the half that is
 *  easy to get wrong and impossible to notice: `"AB\n\n"` is `"AB"` to the
 *  browser and would be `"AB "` without it, and a trailing space in a KEY field
 *  orphans every row that references it — the same failure shape #574 exists to
 *  close. The landing e2e reads all of this off a live input, so the day an
 *  engine disagrees it fails there rather than drifting silently. (A LEADING
 *  break is not dropped; it becomes a space like any other interior one.)
 *
 *  Exported because the handler normalizes at the point of entry, where a
 *  reader can see it happen. Two call sites, ONE definition — the alternative
 *  is two rules that agree until someone edits one of them. */
export function singleLine(value: string): string {
  return value.replace(/(?:\r\n|[\r\n])+$/, "").replace(/\r\n|[\r\n]/g, " ");
}

/** Replace one cell, returning a new delivery — or the SAME delivery when the
 *  write changes nothing, so callers can use identity to skip a history entry
 *  for a no-op: the value already matches (Enter on an untouched editor, a
 *  click away), or the group, the row or the column is not there.
 *
 *  A write that cannot land wrote nothing before the bounds check either — the
 *  rebuild below simply matches no cell — so what the check buys is the undo
 *  step, not the data. Without it a miss cost the reader a history entry, and
 *  the Ctrl/Cmd+Z that followed appeared to do nothing at all (#581); a cell
 *  index captured before an async commit reaches that for real (#580).
 *
 *  The column bound is the target ROW's own length, not the group's heading
 *  count: a DATA row shorter than its HEADING row is a real AGS4 shape, and
 *  this write has never extended one. */
export function setCell(
  delivery: Delivery,
  code: string,
  rowIndex: number,
  colIndex: number,
  value: string,
): Delivery {
  // Normalize BEFORE the comparison below, or a paste that reduces to the
  // current value would spend an undo step on a change nobody can see.
  const next = singleLine(value);
  const target = delivery.find((g) => g.code === code)?.rows[rowIndex];
  if (!target || colIndex < 0 || colIndex >= target.length) return delivery;
  if (target[colIndex] === next) return delivery;
  return delivery.map((g) =>
    g.code !== code
      ? g
      : {
          ...g,
          rows: g.rows.map((row, i) =>
            i !== rowIndex
              ? row
              : row.map((cell, j) => (j === colIndex ? next : cell)),
          ),
        },
  );
}

/** Remove one DATA row, returning a new delivery — or the SAME delivery for a
 *  group or row that is not there, so callers can use identity to skip a
 *  history entry for a no-op. Declaration rows are untouchable by
 *  construction: `rows` holds DATA lines only. */
export function deleteRow(
  delivery: Delivery,
  code: string,
  rowIndex: number,
): Delivery {
  const group = delivery.find((g) => g.code === code);
  if (!group || rowIndex < 0 || rowIndex >= group.rows.length) return delivery;
  return delivery.map((g) =>
    g.code !== code
      ? g
      : { ...g, rows: g.rows.filter((_, i) => i !== rowIndex) },
  );
}

/** Append a row to `code`, inheriting the KEY values its parent determines.
 *
 * This is the format teaching itself: a new SAMP under BH01 does not make you
 * retype BH01, because the chain already says which LOCA it hangs off. The
 * inherited values are the parent's KEY headings that this group also carries —
 * which, in AGS4, is exactly what "child" means. */
export function addRow(
  delivery: Delivery,
  code: string,
  parentCode: string | null,
  keyHeadings: readonly string[],
): Delivery {
  const group = delivery.find((g) => g.code === code);
  if (!group) return delivery;

  const parent = parentCode
    ? delivery.find((g) => g.code === parentCode)
    : undefined;
  const source = parent?.rows.at(-1);

  const row = group.headings.map((heading) => {
    if (!parent || !source || !keyHeadings.includes(heading)) return "";
    const at = parent.headings.indexOf(heading);
    return at === -1 ? "" : (source[at] ?? "");
  });

  return delivery.map((g) =>
    g.code === code ? { ...g, rows: [...g.rows, row] } : g,
  );
}

/** Remove a whole group (#529) — the model half of the missing-group teach
 *  loop. Identity for a code that is not there, so a no-op burns no undo
 *  step. */
export function deleteGroup(delivery: Delivery, code: string): Delivery {
  return delivery.some((g) => g.code === code)
    ? delivery.filter((g) => g.code !== code)
    : delivery;
}

/** Put a deleted group back AS SEEDED — rows, position and all (#529).
 *
 * Restore is honest rather than magical: carrying the reader's edits through
 * a delete/restore cycle would need shadow state that can rot, so the
 * contract is the seed's rows, reinserted where the seed had the group
 * relative to whichever groups still stand — position matters, because every
 * finding's line number is derived from the emitted file. Identity when the
 * group is already present, or is not in the seed at all. */
export function restoreGroup(delivery: Delivery, code: string): Delivery {
  if (delivery.some((g) => g.code === code)) return delivery;
  const group = SEEDED.find((g) => g.code === code);
  if (!group) return delivery;
  const seededAt = SEEDED.indexOf(group);
  const order = new Map(SEEDED.map((g, i) => [g.code, i]));
  const at = delivery.findIndex(
    (g) => (order.get(g.code) ?? Number.POSITIVE_INFINITY) > seededAt,
  );
  const head = at === -1 ? delivery.length : at;
  return [...delivery.slice(0, head), group, ...delivery.slice(head)];
}

/** The seeded delivery, parsed once. The fixture is committed and gated by
 *  tests/test_landing_demo_delivery.py, which asserts the exact seeded-finding
 *  set this page narrates. */
export const SEEDED: Delivery = parse(seeded);

/** `LOCA_FDEP` for the first location — the total depth #399's rail runs to,
 *  read from the delivery rather than written into the rail. */
export function seededFinalDepth(delivery: Delivery = SEEDED): number {
  const loca = delivery.find((g) => g.code === "LOCA");
  const at = loca?.headings.indexOf("LOCA_FDEP") ?? -1;
  if (!loca || at === -1) return 0;
  return Number(loca.rows[0]?.[at] ?? 0) || 0;
}
