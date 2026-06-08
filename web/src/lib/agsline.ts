// Offset-preserving AGS4 line tokenizer.
//
// AGS4 lines are comma-separated, double-quoted fields; a literal quote
// inside a field is escaped by doubling it (`""`). This splits a raw
// line into field tokens whose `.text` slices, concatenated in order,
// reproduce `raw` byte-for-byte (the lossless-reassembly invariant).
//
// Tokenization rule (consistent + lossless): each token spans its field
// content INCLUDING its surrounding quotes, plus the trailing comma
// delimiter that follows it (the last field has no trailing comma). Any
// stray whitespace/characters between a comma and the next quote ride
// along with the following token. Concatenating every `.text` therefore
// always rebuilds `raw`.
//
// Examples:
//   splitAgsFields('"HEADING","LOCA_ID"')
//     -> [{text:'"HEADING",',start:0,end:10},{text:'"LOCA_ID"',start:10,end:19}]
//   splitAgsFields('"a","b ""c"" d"')  // escaped quotes stay inside the field
//     -> two tokens; field 1 text is '"b ""c"" d"'
//   concat of every .text === raw   (always)

export interface AgsField {
  /** The raw slice for this field, including quotes + trailing comma. */
  text: string;
  /** Char offset (code points) where this token starts in `raw`. */
  start: number;
  /** Char offset (code points) one past this token's end. */
  end: number;
  /**
   * Char offset (code points) of the field's INNER value — the content
   * between the surrounding quotes (an unquoted field: its trimmed
   * content), excluding the quotes AND the trailing comma. This is the
   * range a field-level highlight should paint, not the whole token.
   * For an empty quoted field `valueStart === valueEnd`.
   */
  valueStart: number;
  /** Char offset (code points) one past the inner value's end. */
  valueEnd: number;
}

export function splitAgsFields(raw: string): AgsField[] {
  // Code-point aware: index over the spread array so astral chars don't
  // split a surrogate pair mid-offset.
  const chars = [...raw];
  const fields: AgsField[] = [];
  const n = chars.length;

  let i = 0;
  let tokenStart = 0;
  let inQuotes = false;
  // Inner-value bounds for the field currently being read. For a quoted
  // field these are set just inside the opening/closing quotes; for an
  // unquoted field they're derived from the token at push time.
  let valueStart = -1;
  let valueEnd = -1;

  // Push the token spanning [tokenStart, end). `valueStart/valueEnd` are
  // the inner-value bounds if a quote was seen; otherwise the trimmed
  // unquoted content (excluding the trailing comma at `end-1`, if any).
  const push = (end: number, hadComma: boolean) => {
    let vs = valueStart;
    let ve = valueEnd;
    if (vs < 0) {
      // Unquoted (or empty) field — inner value is the trimmed content
      // up to the trailing comma.
      const contentEnd = hadComma ? end - 1 : end;
      vs = tokenStart;
      ve = contentEnd;
      while (vs < ve && chars[vs] === " ") vs += 1;
      while (ve > vs && chars[ve - 1] === " ") ve -= 1;
    }
    fields.push({
      text: chars.slice(tokenStart, end).join(""),
      start: tokenStart,
      end,
      valueStart: vs,
      valueEnd: ve,
    });
    valueStart = -1;
    valueEnd = -1;
  };

  while (i < n) {
    const c = chars[i];
    if (inQuotes) {
      if (c === '"') {
        // A doubled quote ("") is an escaped literal quote, not a close.
        if (chars[i + 1] === '"') {
          i += 2;
          continue;
        }
        inQuotes = false;
        valueEnd = i; // content ends just before the closing quote.
      }
      i += 1;
      continue;
    }
    // Outside quotes:
    if (c === '"') {
      inQuotes = true;
      i += 1;
      valueStart = i; // content begins just inside the opening quote.
      valueEnd = i; // empty field defaults to a zero-width inner span.
    } else if (c === ",") {
      // The comma closes the current token (it rides along, per the rule).
      i += 1;
      push(i, true);
      tokenStart = i;
    } else {
      i += 1;
    }
  }

  // Trailing token (after the last comma, or the whole line if commaless).
  if (tokenStart < n || fields.length === 0) {
    push(n, false);
  }

  return fields;
}

/** Quote a raw value as an AGS4 field: wrap in double quotes, doubling any
 *  internal quote (the inverse of the inner-value unescaping above). */
export function quoteAgsField(value: string): string {
  return `"${value.replace(/"/g, '""')}"`;
}

/** Build one AGS4 line from raw field values: each quoted, comma-joined.
 *  `agsLine(["GROUP", "LOCA"])` → `"GROUP","LOCA"`. */
export function agsLine(values: string[]): string {
  return values.map(quoteAgsField).join(",");
}

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
  const f = fields[0];
  return [...raw].slice(f.valueStart, f.valueEnd).join("");
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
  if (start > 0)
    out.push({ n: data[0].n, raw: "", hit: false, ellipsis: start });
  for (let i = start; i <= end; i++) out.push(data[i]);
  const trailing = data.length - 1 - end;
  if (trailing > 0)
    out.push({ n: data[end + 1].n, raw: "", hit: false, ellipsis: trailing });
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
  const colCount = split.reduce((m, fs) => (fs ? Math.max(m, fs.length) : m), 0);
  const widths: number[] = [];
  for (let c = 0; c < colCount; c++) {
    let w = 0;
    for (const fs of split) {
      if (!fs) continue;
      const len = c < fs.length ? [...fs[c].text].length : 0;
      if (len > w) w = len;
    }
    widths[c] = w;
  }

  const rows: AlignedRow[] = block.rows.map((r, ri) => {
    if (r.ellipsis !== undefined) {
      return { n: r.n, hit: false, cells: [], ellipsis: r.ellipsis };
    }
    const fs = split[ri]!;
    const cells: AlignedCell[] = fs.map((f, c) => {
      const text = f.text;
      const pad = Math.max(0, widths[c] - [...text].length);
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

// The lossless-reassembly invariant (concat of every `.text` rebuilds `raw`)
// and the inner-value bounds are load-bearing — every fix preview and the
// Anonymiser/Coordinate tools rebuild lines from these tokens. They're
// enforced by the unit suite in `agsline.test.ts` (run in CI), which replaced
// an import-time dev-only console check that also false-flagged empty quoted
// fields (`""`, ubiquitous in AGS4) as "inner value includes a quote".
