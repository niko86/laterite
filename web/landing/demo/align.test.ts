/* The aligned view's contract (#620): a display-only re-spacing whose one
 * load-bearing property is that it changes NOTHING a finding jump depends
 * on — same line count, same line indices, blank lines untouched. The
 * seeded delivery is the fixture where it matters, so the round-trip runs
 * against the real text the pane shows, not a toy.
 */

import { describe, expect, it } from "vitest";
import seededText from "./seeded-delivery.ags?raw";
import { alignLines, splitFields } from "./align";

const SEEDED_LINES = seededText.split("\r\n");

/** The inverse of the pad: alignment only ever inserts spaces between a
 *  closing quote and the comma (or nothing) after it, so stripping exactly
 *  those must recover the raw line — value-internal spaces sit inside the
 *  quotes and survive. */
const unpad = (line: string) => line.replace(/" +(?=,|$)/g, '"');

describe("splitFields — the landing's own comma walk", () => {
  it("splits on commas outside quotes only", () => {
    expect(splitFields('"DATA","a,b","c"')).toEqual(['"DATA"', '"a,b"', '"c"']);
  });

  it("keeps a doubled quote's comma quoted", () => {
    expect(splitFields('"DATA","x""y","z"')).toEqual([
      '"DATA"',
      '"x""y"',
      '"z"',
    ]);
  });

  it("has no fields for a blank line", () => {
    expect(splitFields("")).toEqual([]);
  });
});

describe("alignLines — same lines, wider", () => {
  const aligned = alignLines(SEEDED_LINES);

  it("keeps the line count and every blank line", () => {
    expect(aligned).toHaveLength(SEEDED_LINES.length);
    const blanksAt = (xs: readonly string[]) =>
      xs.flatMap((line, i) => (line === "" ? [i] : []));
    expect(blanksAt(aligned)).toEqual(blanksAt(SEEDED_LINES));
  });

  it("only inserts pad spaces — stripping them recovers the raw text", () => {
    for (const [i, line] of aligned.entries()) {
      expect(unpad(line)).toBe(SEEDED_LINES[i]);
    }
  });

  it("aligns every block's columns on the seeded delivery", () => {
    // The visible contract: within a block, each comma-outside-quotes sits
    // at the same character index on every line deep enough to have it.
    let block: string[] = [];
    const check = () => {
      const commaAt = (line: string): number[] => {
        const at: number[] = [];
        let inQuote = false;
        for (let i = 0; i < line.length; i++) {
          if (line[i] === '"') inQuote = !inQuote;
          else if (line[i] === "," && !inQuote) at.push(i);
        }
        return at;
      };
      const cols = block.map(commaAt);
      const deepest = cols.reduce((m, c) => Math.max(m, c.length), 0);
      for (let c = 0; c < deepest; c++) {
        const seen = new Set(cols.filter((x) => c < x.length).map((x) => x[c]));
        expect(seen.size, `column ${c} splits at one index`).toBe(1);
      }
      block = [];
    };
    for (const line of alignLines(SEEDED_LINES)) {
      if (line === "") check();
      else block.push(line);
    }
    check();
  });

  it("pads no line's last field", () => {
    for (const line of alignLines(SEEDED_LINES)) {
      expect(line).not.toMatch(/ $/);
    }
  });

  it("aligns blocks independently", () => {
    // Two blocks with wildly different widths must not share columns: the
    // narrow block keeps its own spacing rather than inheriting the wide
    // one's.
    const lines = ['"GROUP","AB"', '"X","YYYYYY"', "", '"GROUP","CDEF"'];
    const out = alignLines(lines);
    expect(out[0]).toBe('"GROUP","AB"');
    expect(out[1]).toBe('"X"    ,"YYYYYY"');
    expect(out[3]).toBe('"GROUP","CDEF"');
  });
});
