import { describe, expect, it } from "vitest";
import {
  splitAgsFields,
  quoteAgsField,
  agsLine,
  groupBlock,
  alignBlock,
  fixBlock,
  type GroupBlock,
} from "./agsline";

// Runs in the WASM lane (vitest.wasm.config.ts): splitAgsFields/quoteAgsField
// are now backed by the tiny tokenizer wasm (laterite-dev#533), init'd from disk in the
// lane's setup file. The tokenizer's invariants are pinned AUTHORITATIVELY in
// Rust (laterite-ags4-parse's `display_spans.rs` proptest); the checks below
// double as a cross-check that the wasm boundary hands JS the right shape
// (camelCase valueStart/valueEnd) and that the byte→code-point conversion the
// wasm adapter now performs is correct, and exercise the browser-only display
// helpers (groupBlock/alignBlock/fixBlock) against it.
//
// Tokens carry offsets only. The Rust side proves its bounds tile the line in
// BYTES; this lane is the only place the CODE-POINT offsets JS actually
// receives get checked, which is why the tiling test below lives here and not
// only in Rust.

const CASES = [
  '"HEADING","LOCA_ID","LOCA_NATE"',
  '"DATA","a","b ""c"" d",""',
  "no,quotes,here",
  '"single"',
  "",
  '"DATA","BH01",', // trailing comma (a real short/over-padded row shape)
  '"DATA",   "spaced"  ,"x"', // stray whitespace between comma and quote
  '"DATA","emoji 😀 here","z"', // astral char — must not split a surrogate pair
  '"a",,"c"', // empty unquoted middle field
  ",,", // all-empty fields
  '"x""y"', // a field that is just an escaped quote
];

describe("splitAgsFields — token ranges tile the line", () => {
  // The same proposition the retired `.text` reassembly asserted — that the
  // tokens cover the line with no gap and no overlap — stated directly against
  // the offsets instead of via a materialised copy of each token.
  it.each(CASES)("[start,end) ranges rebuild %j exactly", (raw) => {
    const cps = Array.from(raw);
    const fields = splitAgsFields(raw);
    expect(fields.map((f) => cps.slice(f.start, f.end).join("")).join("")).toBe(
      raw,
    );
  });

  // Reassembly alone would still pass if every range were shifted by a
  // compensating amount, so pin the seam explicitly.
  it.each(CASES)("ranges are contiguous and start at 0 for %j", (raw) => {
    const fields = splitAgsFields(raw);
    expect(fields[0]!.start).toBe(0);
    for (let i = 1; i < fields.length; i++) {
      expect(fields[i]!.start).toBe(fields[i - 1]!.end);
    }
    expect(fields[fields.length - 1]!.end).toBe(Array.from(raw).length);
  });
});

describe("splitAgsFields — inner-value bounds", () => {
  it.each(CASES)(
    "inner value never spills past token / onto quote+comma for %j",
    (raw) => {
      const cps = Array.from(raw);
      for (const f of splitAgsFields(raw)) {
        expect(f.valueStart).toBeGreaterThanOrEqual(f.start);
        expect(f.valueEnd).toBeLessThanOrEqual(f.end);
        expect(f.valueStart).toBeLessThanOrEqual(f.valueEnd);
        // The inner value must exclude the surrounding quote and trailing comma
        // (only constrained when the inner value is non-empty).
        const nonEmpty = f.valueStart < f.valueEnd;
        expect(nonEmpty && cps[f.valueStart] === '"').toBe(false);
        expect(nonEmpty && cps[f.valueEnd - 1] === ",").toBe(false);
      }
    },
  );
});

describe("splitAgsFields — field count + values", () => {
  it("splits a HEADING row into one token per heading", () => {
    const f = splitAgsFields('"HEADING","LOCA_ID","LOCA_NATE"');
    expect(f).toHaveLength(3);
  });

  it("recovers the inner value, unescaping doubled quotes structurally", () => {
    const f = splitAgsFields('"DATA","b ""c"" d"');
    const innerOf = (
      raw: string,
      x: { valueStart: number; valueEnd: number },
    ) => Array.from(raw).slice(x.valueStart, x.valueEnd).join("");
    // valueStart/valueEnd span the raw (still-escaped) inner content.
    expect(innerOf('"DATA","b ""c"" d"', f[1]!)).toBe('b ""c"" d');
  });

  it("absorbs a trailing comma into the preceding token (the documented rule)", () => {
    // The tokenizer is offset-preserving for highlighting, not a field-count
    // oracle: a trailing comma rides along with its token rather than opening
    // a new empty field. The lossless invariant (covered above) still holds.
    const f = splitAgsFields('"DATA","BH01",');
    expect(f).toHaveLength(2);
    const raw = '"DATA","BH01",';
    expect(Array.from(raw).slice(f[1]!.start, f[1]!.end).join("")).toBe(
      '"BH01",',
    );
  });

  it("an empty line is a single empty field (never zero fields)", () => {
    const f = splitAgsFields("");
    expect(f).toHaveLength(1);
    expect(f[0]!.start).toBe(0);
    expect(f[0]!.end).toBe(0);
  });
});

describe("quoteAgsField / agsLine", () => {
  it("wraps and doubles internal quotes", () => {
    expect(quoteAgsField("plain")).toBe('"plain"');
    expect(quoteAgsField('a"b')).toBe('"a""b"');
    expect(quoteAgsField("")).toBe('""');
  });

  it("round-trips a value through quote → split → inner value", () => {
    for (const v of ["BH01", "a, b", 'has "quote"', ""]) {
      const line = agsLine(["DATA", v]);
      const f = splitAgsFields(line);
      const inner = Array.from(line)
        .slice(f[1]!.valueStart, f[1]!.valueEnd)
        .join("");
      // The structural inner value is the escaped form; un-doubling recovers v.
      expect(inner.replace(/""/g, '"')).toBe(v);
    }
  });

  it("agsLine joins quoted fields with commas", () => {
    expect(agsLine(["GROUP", "LOCA"])).toBe('"GROUP","LOCA"');
  });
});

// --- groupBlock: reconstruct the enclosing GROUP block -----------------------
//
// groupBlock scans UP to the "GROUP",… opener and DOWN to the next blank / next
// GROUP row. lineTag() (private) is exercised through it (and through alignBlock
// /fixBlock). These cover the boundary-finding + the windowRows() data-row
// windowing the block then goes through.

// A small LOCA block: GROUP/HEADING/UNIT/TYPE headers + 3 DATA rows.
const SMALL_BLOCK = [
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_TYPE"',
  '"UNIT","",""',
  '"TYPE","ID","PA"',
  '"DATA","BH01","TP"',
  '"DATA","BH02","CP"',
  '"DATA","BH03","RC"',
];

describe("groupBlock", () => {
  it("collects the whole small block from a header-row hit (≤ DATA_MAX rows shown whole)", () => {
    const blk = groupBlock(SMALL_BLOCK, 2)!; // hit on the HEADING row
    expect(blk).not.toBeNull();
    expect(blk.rows.map((r) => r.n)).toEqual([1, 2, 3, 4, 5, 6, 7]);
    expect(blk.rows.find((r) => r.hit)!.n).toBe(2); // the HEADING row is the hit
    expect(blk.rows.some((r) => r.ellipsis !== undefined)).toBe(false);
  });

  it("scans up from a DATA-row hit to the GROUP opener", () => {
    const blk = groupBlock(SMALL_BLOCK, 6)!; // hit on the 2nd DATA row
    expect(blk.rows[0]!.raw).toBe('"GROUP","LOCA"');
    expect(blk.rows.find((r) => r.hit)!.raw).toBe('"DATA","BH02","CP"');
  });

  it("stops the down-scan at a blank line (block boundary)", () => {
    const lines = [...SMALL_BLOCK, "", '"GROUP","SAMP"', '"HEADING","SAMP_ID"'];
    const blk = groupBlock(lines, 5)!;
    // The LOCA block ends at the blank line — the SAMP rows are not included.
    expect(blk.rows.every((r) => !r.raw.includes("SAMP"))).toBe(true);
    expect(blk.rows.map((r) => r.n)).toEqual([1, 2, 3, 4, 5, 6, 7]);
  });

  it("stops the down-scan at the next GROUP row (no blank separator)", () => {
    const lines = [...SMALL_BLOCK, '"GROUP","SAMP"', '"HEADING","SAMP_ID"'];
    const blk = groupBlock(lines, 1)!; // hit the GROUP opener itself
    expect(blk.rows.map((r) => r.raw)).toEqual(SMALL_BLOCK);
  });

  it("returns null for an out-of-range line", () => {
    expect(groupBlock(SMALL_BLOCK, 0)).toBeNull();
    expect(groupBlock(SMALL_BLOCK, SMALL_BLOCK.length + 1)).toBeNull();
  });

  it("returns null when there is no GROUP row above the hit", () => {
    // A hit with a blank line directly above it has no enclosing block.
    const lines = ["", '"DATA","orphan"'];
    expect(groupBlock(lines, 2)).toBeNull();
  });

  it("returns null when the up-scan hits a blank before any GROUP row", () => {
    const lines = ['"GROUP","LOCA"', "", '"DATA","BH01"'];
    // Line 3's up-scan hits the blank at line 2 first → no block.
    expect(groupBlock(lines, 3)).toBeNull();
  });

  describe("windowRows (DATA windowing for large groups)", () => {
    // 12 DATA rows (> DATA_MAX = 8) forces windowing: a data-row hit keeps
    // DATA_CONTEXT(=2) either side; a header hit keeps DATA_SAMPLE(=5) leading.
    const header = [
      '"GROUP","LOCA"',
      '"HEADING","LOCA_ID"',
      '"UNIT",""',
      '"TYPE","ID"',
    ];
    const data = Array.from({ length: 12 }, (_, i) => `"DATA","BH${i}"`);
    const big = [...header, ...data];

    it("windows around a data-row hit: headers + ±2 data rows + two ellipses", () => {
      const blk = groupBlock(big, 4 + 7)!; // hit the 7th data row (BH6, line 11)
      const tags = blk.rows.map((r) =>
        r.ellipsis !== undefined ? `…${r.ellipsis}` : r.raw,
      );
      // 4 headers, leading ellipsis (4 omitted: BH0..BH3), BH4..BH8, trailing ellipsis (3).
      expect(tags).toEqual([
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        "…4",
        '"DATA","BH4"',
        '"DATA","BH5"',
        '"DATA","BH6"',
        '"DATA","BH7"',
        '"DATA","BH8"',
        "…3",
      ]);
      expect(blk.rows.find((r) => r.hit)!.raw).toBe('"DATA","BH6"');
    });

    it("windows a header-row hit to the first DATA_SAMPLE(5) rows + one trailing ellipsis", () => {
      const blk = groupBlock(big, 2)!; // hit the HEADING row (no data hit)
      const dataRows = blk.rows.filter((r) => r.raw.startsWith('"DATA"'));
      expect(dataRows.map((r) => r.raw)).toEqual([
        '"DATA","BH0"',
        '"DATA","BH1"',
        '"DATA","BH2"',
        '"DATA","BH3"',
        '"DATA","BH4"',
      ]);
      const ell = blk.rows.filter((r) => r.ellipsis !== undefined);
      expect(ell).toHaveLength(1); // only trailing (start === 0 → no leading)
      expect(ell[0]!.ellipsis).toBe(7); // 12 - 5
    });

    it("no leading ellipsis when the hit is within DATA_CONTEXT of the first data row", () => {
      const blk = groupBlock(big, 4 + 1)!; // hit BH0 (first data row)
      const ell = blk.rows.filter((r) => r.ellipsis !== undefined);
      // start = max(0, 0-2) = 0 → no leading gap; only a trailing one.
      expect(ell.map((r) => r.ellipsis)).toEqual([data.length - 1 - 2]);
    });
  });
});

// --- alignBlock: space-pad fields into columns -------------------------------

describe("alignBlock", () => {
  it("pads every cell to its column's max width, preserving value offsets", () => {
    const block: GroupBlock = {
      rows: [
        { n: 1, raw: '"HEADING","LOCA_ID","LOCA_TYPE"', hit: false },
        { n: 2, raw: '"DATA","BH01","TP"', hit: true },
      ],
    };
    const aligned = alignBlock(block);
    expect(aligned.rows).toHaveLength(2);
    // Column 0: max("HEADING", "DATA",) widths → both cells equal width.
    const w0 = aligned.rows.map((r) => r.cells[0]!.padded.length);
    expect(w0[0]).toBe(w0[1]);
    // Column 1: '"LOCA_ID",' (10) vs '"BH01",' (7) → data cell padded to 10.
    expect(aligned.rows[1]!.cells[1]!.padded).toBe('"BH01",   '); // 7 + 3 spaces
    expect(aligned.rows[1]!.hit).toBe(true);
    // valueStart/valueEnd are rebased to be cell-relative (just inside the quote).
    const cell = aligned.rows[1]!.cells[1]!;
    expect(cell.padded.slice(cell.valueStart, cell.valueEnd)).toBe("BH01");
  });

  it("ragged rows: a short row's missing columns count as zero width", () => {
    const block: GroupBlock = {
      rows: [
        { n: 1, raw: '"DATA","a","bbbb"', hit: false },
        { n: 2, raw: '"DATA","a"', hit: false }, // only 2 fields
      ],
    };
    const aligned = alignBlock(block);
    // Row 2 has no 3rd cell; row 1's 3rd column width comes from itself only.
    expect(aligned.rows[1]!.cells).toHaveLength(2);
    expect(aligned.rows[0]!.cells).toHaveLength(3);
  });

  it("carries ellipsis rows through with no cells", () => {
    const block: GroupBlock = {
      rows: [
        { n: 1, raw: '"DATA","x"', hit: false },
        { n: 2, raw: "", hit: false, ellipsis: 5 },
        { n: 3, raw: '"DATA","y"', hit: false },
      ],
    };
    const aligned = alignBlock(block);
    expect(aligned.rows[1]).toMatchObject({
      n: 2,
      hit: false,
      cells: [],
      ellipsis: 5,
    });
  });

  it("preserves the variant flag through alignment (del/ins pairs)", () => {
    const block: GroupBlock = {
      rows: [
        { n: 5, raw: '"DATA","old"', hit: true, variant: "del" },
        { n: 5, raw: '"DATA","new"', hit: true, variant: "ins" },
      ],
    };
    const aligned = alignBlock(block);
    expect(aligned.rows.map((r) => r.variant)).toEqual(["del", "ins"]);
  });
});

// --- fixBlock: before/after pair inside the aligned block --------------------

describe("fixBlock", () => {
  it("splices a del/ins pair at the hit row, keeping surrounding context", () => {
    const blk = fixBlock(SMALL_BLOCK, 6, '"DATA","BH02","XX"')!;
    expect(blk).not.toBeNull();
    const hitPair = blk.rows.filter((r) => r.variant);
    expect(hitPair).toHaveLength(2);
    expect(hitPair[0]).toMatchObject({
      raw: '"DATA","BH02","CP"',
      variant: "del",
    });
    expect(hitPair[1]).toMatchObject({
      raw: '"DATA","BH02","XX"',
      variant: "ins",
    });
    // Non-hit rows are passed through unchanged (no variant).
    expect(
      blk.rows.find((r) => r.raw === '"GROUP","LOCA"')!.variant,
    ).toBeUndefined();
  });

  it("returns null when there's no enclosing GROUP block to anchor the fix", () => {
    const lines = ["", '"DATA","orphan","x"'];
    expect(fixBlock(lines, 2, '"DATA","orphan","y"')).toBeNull();
  });

  it("aligns the del/ins pair to the same column widths as the rest of the block", () => {
    const blk = fixBlock(
      SMALL_BLOCK,
      6,
      '"DATA","BH02","a much longer value"',
    )!;
    const aligned = alignBlock(blk);
    const variantRows = aligned.rows.filter((r) => r.variant);
    // Both del + ins share the block widths: every row's col-1 cell is the same width.
    const widths = aligned.rows
      .filter((r) => r.cells.length > 1)
      .map((r) => r.cells[1]!.padded.length);
    expect(new Set(widths).size).toBe(1);
    expect(variantRows.map((r) => r.variant)).toEqual(["del", "ins"]);
  });
});
