/* Display-only column alignment for the file pane (#620) — the webapp's
 * "Aligned columns" grammar, recomputed in the landing's own model.
 *
 * The app's alignBlock consumes the wasm-backed tokenizer, which must not
 * join the apex byte budget, so this module re-derives the same contract
 * from the one thing the landing's emitter guarantees: every field is
 * quoted, and the only commas that separate fields sit OUTSIDE quotes. The
 * grammar mirrors the app's — per-block column widths, each field padded on
 * the right — and the invariant that makes it safe as a VIEW is intra-line
 * padding only: same line count in and out, so finding line numbers keep
 * landing where they land on the raw bytes (#396's fidelity story is about
 * the RAW mode; this one is labelled a view).
 */

/** One emitted line's fields, quotes included, split on commas OUTSIDE
 *  quotes only — a comma or doubled quote inside a value must not split.
 *  A blank line has no fields. */
export function splitFields(line: string): string[] {
  if (line === "") return [];
  const fields: string[] = [];
  let start = 0;
  let inQuote = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      // A doubled quote inside a value flips this twice, netting to no
      // state change — so text after it is still correctly "inside".
      inQuote = !inQuote;
    } else if (ch === "," && !inQuote) {
      fields.push(line.slice(start, i));
      start = i + 1;
    }
  }
  fields.push(line.slice(start));
  return fields;
}

/** The aligned view of the pane's lines: blocks (runs of non-blank lines)
 *  are aligned independently, each field padded to its column's widest
 *  occupant, fields rejoined with the commas that split them. Blank lines
 *  pass through untouched and every line keeps its index — the property
 *  the unit lane pins, because it is the one that keeps finding jumps
 *  honest in both modes. */
export function alignLines(lines: readonly string[]): string[] {
  const out: string[] = [];
  let block: string[][] = [];
  const flush = () => {
    if (block.length === 0) return;
    // Character count, not code units — a value with an accent or a symbol
    // must not shift every column after it by a phantom pad.
    const width = (f: string) => Array.from(f).length;
    const widths: number[] = [];
    for (const fields of block) {
      fields.forEach((f, c) => {
        const w = width(f);
        if (w > (widths[c] ?? 0)) widths[c] = w;
      });
    }
    for (const fields of block) {
      out.push(
        fields
          .map((f, c) =>
            // A line's last field takes no pad: nothing follows it on ITS
            // line, so padding would only ship trailing spaces.
            c === fields.length - 1
              ? f
              : f + " ".repeat((widths[c] ?? 0) - width(f)),
          )
          .join(","),
      );
    }
    block = [];
  };
  for (const line of lines) {
    if (line === "") {
      flush();
      out.push(line);
    } else {
      block.push(splitFields(line));
    }
  }
  flush();
  return out;
}
