// Three fallbacks that only fire on the shapes a happy-path test never builds:
// a ragged row set, a bytes-in Excel source, and the CLI's exit-code mapping.
//
// Each is a `??` or a ternary whose second arm is the interesting one, and each
// fails silently rather than loudly — a misaligned column, a file read from the
// wrong place, an exit code a script branches on.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { buildAgs4, fromExcel, toExcel } from "../ts/index";

const AGS =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL"\r\n' +
  '"UNIT","","m"\r\n' +
  '"TYPE","ID","2DP"\r\n' +
  '"DATA","BH01","12.30"\r\n' +
  '"DATA","BH02","14.00"\r\n';

describe("rowsToTable over a ragged row set", () => {
  it("null-fills a key some rows omit rather than shifting the column", () => {
    // The columns are gathered from the union of every row's keys, then each row
    // contributes one value per column. A row that lacks a key must yield NULL
    // in its own slot — not be skipped, which would slide every later row's
    // value up a position and silently mis-attribute data between boreholes.
    const text = buildAgs4([
      [
        "LOCA",
        [
          { LOCA_ID: "BH01", LOCA_GL: "12.30" },
          { LOCA_ID: "BH02" }, // no LOCA_GL at all
          { LOCA_ID: "BH03", LOCA_GL: "15.00" },
        ],
      ],
    ]).text;

    const dataRows = text
      .split("\r\n")
      .filter((l) => l.startsWith('"DATA"'))
      .map((l) => l.split(",").map((c) => c.replace(/^"|"$/g, "")));

    expect(dataRows).toHaveLength(3);
    // BH02's missing value is an empty cell in ITS row …
    expect(dataRows[1]![1]).toBe("BH02");
    expect(dataRows[1]![2]).toBe("");
    // … and BH03's value stayed with BH03.
    expect(dataRows[2]![1]).toBe("BH03");
    expect(dataRows[2]![2]).not.toBe("");
  });

  it("keeps a column that only one row populates", () => {
    // The union-of-keys walk. If columns came from the FIRST row only, a field
    // introduced later would be dropped from the output entirely.
    const text = buildAgs4([
      ["LOCA", [{ LOCA_ID: "BH01" }, { LOCA_ID: "BH02", LOCA_GL: "14.00" }]],
    ]).text;
    expect(text).toContain("LOCA_GL");
    expect(text).toContain("14.00");
  });
});

describe("fromExcel's two source doors", () => {
  it("accepts a path and accepts bytes, and both produce the same AGS4", () => {
    // The `typeof source === "string" ? readFileSync(source) : source` arm. A
    // caller holding an uploaded buffer must not have to write it to disk first,
    // and the two doors must not diverge.
    const dir = mkdtempSync(join(tmpdir(), "laterite-ags4-excel-"));
    const xlsxPath = join(dir, "book.xlsx");
    const agsPath = join(dir, "in.ags");
    writeFileSync(agsPath, AGS);
    toExcel(agsPath, xlsxPath);

    const fromPath = fromExcel(xlsxPath);
    const fromBytes = fromExcel(readFileSync(xlsxPath));

    expect(fromPath.length).toBeGreaterThan(0);
    // Byte-identical: the source door must not change the output.
    expect(Buffer.compare(fromPath, fromBytes)).toBe(0);
    expect(fromBytes.toString("utf8")).toContain('"GROUP","LOCA"');
  });

  it("returns bytes when given no output path, and stats when given one", () => {
    // The two overloads share one body and differ on `agsPath === undefined`.
    const dir = mkdtempSync(join(tmpdir(), "laterite-ags4-excel-"));
    const xlsxPath = join(dir, "book.xlsx");
    const agsPath = join(dir, "in.ags");
    const outPath = join(dir, "out.ags");
    writeFileSync(agsPath, AGS);
    toExcel(agsPath, xlsxPath);

    const bytes = fromExcel(readFileSync(xlsxPath));
    expect(Buffer.isBuffer(bytes)).toBe(true);

    const stats = fromExcel(readFileSync(xlsxPath), outPath);
    expect(typeof stats).toBe("object");
    expect(Buffer.isBuffer(stats)).toBe(false);
    // …and the file it claims to have written is really there.
    expect(readFileSync(outPath, "utf8")).toContain('"GROUP","LOCA"');
  });
});
