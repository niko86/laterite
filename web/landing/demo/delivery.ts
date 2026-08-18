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

/** Replace one cell, returning a new delivery. */
export function setCell(
  delivery: Delivery,
  code: string,
  rowIndex: number,
  colIndex: number,
  value: string,
): Delivery {
  return delivery.map((g) =>
    g.code !== code
      ? g
      : {
          ...g,
          rows: g.rows.map((row, i) =>
            i !== rowIndex
              ? row
              : row.map((cell, j) => (j === colIndex ? value : cell)),
          ),
        },
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

/** The seeded delivery, parsed once. The fixture is committed and gated by
 *  tests/test_landing_demo_delivery.py, which asserts it fails in exactly the
 *  four ways this page narrates. */
export const SEEDED: Delivery = parse(seeded);

/** `LOCA_FDEP` for the first location — the total depth #399's rail runs to,
 *  read from the delivery rather than written into the rail. */
export function seededFinalDepth(delivery: Delivery = SEEDED): number {
  const loca = delivery.find((g) => g.code === "LOCA");
  const at = loca?.headings.indexOf("LOCA_FDEP") ?? -1;
  if (!loca || at === -1) return 0;
  return Number(loca.rows[0]?.[at] ?? 0) || 0;
}
