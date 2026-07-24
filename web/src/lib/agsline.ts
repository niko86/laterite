// GROUP-block reconstruction + column alignment for the validate/fix views.
//
// The offset-preserving tokenizer + field quoter this builds on
// (`splitAgsFields` / `quoteAgsField` / `agsLine`) now live in `./tokenizer`,
// backed by the shared Rust leaves through a tiny wasm (#533, part of the #527
// arc) — the browser no longer carries its own copy. This module keeps the
// browser-only DISPLAY logic (block reconstruction, DATA windowing, column
// alignment, fix-preview pairing) that CONSUMES those tokens. The tokenizer
// surface is re-exported here so existing importers of `./agsline` are unchanged.

import { fieldSlice, splitAgsFields } from "./tokenizer";

export {
  splitAgsFields,
  quoteAgsField,
  agsLine,
  tokenizerReady,
} from "./tokenizer";
export type { AgsField } from "./tokenizer";

// Every `!` below is a non-null assertion on an array index that the loop bounds
// or an above-checked length PROVE in range (each is justified inline). Under
// noUncheckedIndexedAccess TypeScript still types those reads `T | undefined`;
// the assertions faithfully express hand-verified invariants across this index-
// dense alignment/windowing code, so no-non-null-assertion is disabled file-wide
// here (#615) — same rationale as linediff.ts.
/* eslint-disable @typescript-eslint/no-non-null-assertion */

// --- GROUP-block reconstruction + column alignment ---
//
// The ±1 snippet in `FindingsView` is fine for a single misbehaving line,
// but positional CSV misalignment (Rules 7/4/8/10b) is only eyeballable
// when the whole GROUP block is laid out as space-aligned columns. The
// `FindingsView` holds the entire file `lines()`, so we can reconstruct
// the enclosing block around any hit line and render it aligned.

/** First field's inner value (tag) of an AGS4 line, e.g. `"GROUP"` →
 *  `GROUP`. Empty for a blank line. Used to find block boundaries. */
function lineTag(raw: string): string {
  const fields = splitAgsFields(raw);
  if (fields.length === 0) return "";
  const f = fields[0]!; // length !== 0 checked above → in-bounds.
  return fieldSlice(Array.from(raw), f.valueStart, f.valueEnd);
}

export interface GroupBlock {
  /** A block row. `ellipsis` (when set) is a gap marker standing in for
   *  `ellipsis` omitted DATA rows — `raw` is empty for these. `variant`
   *  (when set) marks a before/after pair in a fix preview: `del` = the row
   *  as it is now, `ins` = the row with the fix applied. */
  rows: {
    n: number;
    raw: string;
    hit: boolean;
    ellipsis?: number;
    variant?: "del" | "ins";
  }[];
}

// A GROUP block is scanned per-finding (and `groupBlock` tokenizes every
// row it walks), so an enclosing block of tens of thousands of rows would
// make the aligned view O(group-size) × visible-findings — the "aligned
// mode locks up at 10k errors" symptom. Cap each scan direction: the hit is
// always within the window (`top` is at most MAX_BLOCK_SCAN above `line1`,
// the down-scan reaches at most MAX_BLOCK_SCAN below `top`), so the only
// casualty past the cap is an undercounted trailing "⋯ N more" on a
// pathologically large group — cosmetic, and far better than a freeze.
const MAX_BLOCK_SCAN = 2000;

/**
 * Reconstruct the GROUP block enclosing 1-based `line1`: scan UP to the
 * `"GROUP",…` row that opens the block and DOWN to the next blank line or
 * next `"GROUP"` row, collecting every row in between (and the GROUP row).
 * `hit` marks the row at `line1`. Returns `null` if no enclosing GROUP row
 * is found within {@link MAX_BLOCK_SCAN} (the caller falls back to the raw
 * snippet).
 */
export function groupBlock(lines: string[], line1: number): GroupBlock | null {
  if (line1 < 1 || line1 > lines.length) return null;

  // Up: find the GROUP row at or above the hit line. Stop early on a blank
  // line (block boundary) — a hit with no GROUP above it isn't in a block.
  let top = -1;
  for (let n = line1; n >= 1 && line1 - n <= MAX_BLOCK_SCAN; n--) {
    const raw = lines[n - 1] ?? "";
    if (raw.trim() === "") break;
    if (lineTag(raw) === "GROUP") {
      top = n;
      break;
    }
  }
  if (top < 0) return null;

  // Down: from the GROUP row, collect until the next blank line or the
  // next GROUP row (exclusive), bounded by the scan cap.
  const all: GroupBlock["rows"] = [];
  for (let n = top; n <= lines.length && n - top <= MAX_BLOCK_SCAN; n++) {
    const raw = lines[n - 1] ?? "";
    if (n > top && (raw.trim() === "" || lineTag(raw) === "GROUP")) break;
    all.push({ n, raw, hit: n === line1 });
  }
  return { rows: windowRows(all) };
}

// A group can hold hundreds of DATA rows; dumping all of them for a
// heading/cell finding buries the point. Keep the structural header rows
// (GROUP/HEADING/UNIT/TYPE — they define the column layout) plus a bounded
// window of DATA rows: centred on the hit when the finding is on a data
// row, else the first few for column context. Each omitted run collapses
// to one `ellipsis` marker row.
const HEADER_TAGS = new Set(["GROUP", "HEADING", "UNIT", "TYPE"]);
const DATA_CONTEXT = 2; // DATA rows kept either side of a data-row hit
const DATA_SAMPLE = 5; // DATA rows kept for a header-row hit (no data hit)
const DATA_MAX = 8; // show every DATA row when there are no more than this

function windowRows(all: GroupBlock["rows"]): GroupBlock["rows"] {
  const headers = all.filter((r) => HEADER_TAGS.has(lineTag(r.raw)));
  const data = all.filter((r) => !HEADER_TAGS.has(lineTag(r.raw)));
  if (data.length <= DATA_MAX) return all; // small group — show it whole.

  const hitIdx = data.findIndex((r) => r.hit);
  let start: number;
  let end: number; // inclusive index into `data`
  if (hitIdx >= 0) {
    start = Math.max(0, hitIdx - DATA_CONTEXT);
    end = Math.min(data.length - 1, hitIdx + DATA_CONTEXT);
  } else {
    start = 0;
    end = Math.min(data.length - 1, DATA_SAMPLE - 1);
  }

  const out: GroupBlock["rows"] = [...headers];
  // data.length > DATA_MAX here, and start/end are clamped into
  // [0, data.length−1] → every data[...] access below is in-bounds.
  if (start > 0)
    out.push({ n: data[0]!.n, raw: "", hit: false, ellipsis: start });
  for (let i = start; i <= end; i++) out.push(data[i]!);
  const trailing = data.length - 1 - end;
  if (trailing > 0)
    out.push({ n: data[end + 1]!.n, raw: "", hit: false, ellipsis: trailing });
  return out;
}

export interface AlignedCell {
  /** Padded display text for this cell (field text, space-padded to the
   *  column width). The trailing comma/quotes ride along as in the token. */
  padded: string;
  /** Char offset of the inner value WITHIN `padded`. */
  valueStart: number;
  /** Char offset one past the inner value within `padded`. */
  valueEnd: number;
}

export interface AlignedRow {
  n: number;
  hit: boolean;
  cells: AlignedCell[];
  /** When set, this is a gap standing in for `ellipsis` omitted DATA rows
   *  (no cells); the renderer shows a "⋯ N more" marker. */
  ellipsis?: number;
  /** Before/after marker in a fix preview (see {@link GroupBlock}). */
  variant?: "del" | "ins";
}

export interface AlignedBlock {
  rows: AlignedRow[];
}

/**
 * Lay out a {@link GroupBlock} as space-aligned columns: split every row
 * into fields, compute per-column max display width, and pad each field's
 * text to that width. The returned per-cell `valueStart/valueEnd` are
 * offsets WITHIN the padded cell so a field/inner-value highlight can be
 * located after alignment (the leading-quote/comma offsets are preserved
 * because padding is appended on the right).
 */
export function alignBlock(block: GroupBlock): AlignedBlock {
  // Ellipsis marker rows carry no fields and don't participate in width.
  const split = block.rows.map((r) =>
    r.ellipsis === undefined ? splitAgsFields(r.raw) : null,
  );
  const colCount = split.reduce(
    (m, fs) => (fs ? Math.max(m, fs.length) : m),
    0,
  );
  const widths: number[] = [];
  for (let c = 0; c < colCount; c++) {
    let w = 0;
    for (const fs of split) {
      if (!fs) continue;
      // A token's display width IS its offset span — no string needed.
      const len = c < fs.length ? fs[c]!.end - fs[c]!.start : 0;
      if (len > w) w = len;
    }
    widths[c] = w;
  }

  const rows: AlignedRow[] = block.rows.map((r, ri) => {
    if (r.ellipsis !== undefined) {
      return { n: r.n, hit: false, cells: [], ellipsis: r.ellipsis };
    }
    const fs = split[ri]!;
    // Split ONCE per row, not once per field — slicing each field from its own
    // Array.from(raw) would be quadratic in the row's width.
    const chars = Array.from(r.raw);
    const cells: AlignedCell[] = fs.map((f, c) => {
      const text = fieldSlice(chars, f.start, f.end);
      // c < fs.length ≤ colCount = widths.length → widths[c] is in-bounds.
      const pad = Math.max(0, widths[c]! - (f.end - f.start));
      // valueStart/valueEnd are token-relative; rebase onto the cell.
      return {
        padded: text + " ".repeat(pad),
        valueStart: f.valueStart - f.start,
        valueEnd: f.valueEnd - f.start,
      };
    });
    return { n: r.n, hit: r.hit, cells, variant: r.variant };
  });
  return { rows };
}

/**
 * Build the GROUP block enclosing a fix's edit, with the changed line shown
 * as a before/after pair: the current row (`variant: "del"`) immediately
 * followed by the row with `after` (the line after the replacement is
 * spliced in, `variant: "ins"`). Both share the block's column widths once
 * passed through {@link alignBlock}, so the change reads in proper aligned
 * context (GROUP/HEADING/UNIT/TYPE headers + nearby data rows). Returns
 * `null` when there is no enclosing GROUP block (caller falls back to the
 * single-line per-edit diff).
 */
export function fixBlock(
  lines: string[],
  line1: number,
  after: string,
): GroupBlock | null {
  const block = groupBlock(lines, line1);
  if (!block) return null;
  const rows: GroupBlock["rows"] = [];
  for (const r of block.rows) {
    if (r.hit && r.ellipsis === undefined) {
      rows.push({ ...r, variant: "del" });
      rows.push({ n: r.n, raw: after, hit: true, variant: "ins" });
    } else {
      rows.push(r);
    }
  }
  return { rows };
}

// The tiling invariant (every token's [start,end) range, concatenated, rebuilds `raw`)
// and the inner-value bounds are load-bearing — every fix preview and the
// Anonymiser/Coordinate tools rebuild lines from these tokens. They are now
// pinned authoritatively in Rust (`laterite-ags4-parse`'s `display_spans.rs`
// proptest), the single source the wasm tokenizer wraps — though only in BYTES:
// the code-point offsets JS receives come from a conversion in the wasm adapter,
// so the wasm test lane is where that unit is checked. The browser-side display
// logic above (groupBlock/alignBlock/fixBlock) keeps its own tests.
