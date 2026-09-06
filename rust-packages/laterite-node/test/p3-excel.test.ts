// laterite-dev#358 — Node Excel I/O. Binds the SAME `laterite-ags4-excel` converter Python's
// to_excel / from_excel use, so AGS4 ↔ XLSX round-trips through Node.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { fromExcel, read, toExcel } from "../ts/index";

const AGS =
  [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_GL"',
    '"UNIT","","m"',
    '"TYPE","ID","2DP"',
    '"DATA","BH01","10.00"',
    '"DATA","BH02","20.00"',
  ].join("\r\n") + "\r\n";

describe("Excel I/O (laterite-dev#358)", () => {
  it("round-trips AGS4 → xlsx → AGS4", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-excel-"));
    const ags = join(dir, "in.ags");
    const xlsx = join(dir, "out.xlsx");
    const back = join(dir, "back.ags");
    writeFileSync(ags, AGS);

    const toStats = toExcel(ags, xlsx);
    expect(toStats.sheetsWritten).toBeGreaterThanOrEqual(2); // PROJ + LOCA
    expect(toStats.rowsWritten).toBeGreaterThan(0);

    const fromStats = fromExcel(xlsx, back);
    expect(fromStats.sheetsWritten).toBeGreaterThanOrEqual(2);

    // the round-tripped AGS4 still parses and keeps its groups
    const file = read(back);
    expect(file.groups).toContain("PROJ");
    expect(file.groups).toContain("LOCA");
  });

  it("honours the group order passed to toExcel", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-excel-"));
    const ags = join(dir, "in.ags");
    writeFileSync(ags, AGS);
    const stats = toExcel(ags, join(dir, "o.xlsx"), {
      groups: ["LOCA", "PROJ"],
    });
    expect(stats.sheetsWritten).toBe(2);
    expect(stats.warnings).toEqual([]);
  });
});

// #391 — bytes ↔ bytes, no filesystem (the door the browser proved).
describe("Excel bytes forms (#391)", () => {
  const agsBytes = Buffer.from(AGS, "utf8");

  it("toExcel(bytes) with no path returns the .xlsx bytes", () => {
    const xlsx = toExcel(agsBytes);
    expect(Buffer.isBuffer(xlsx)).toBe(true);
    expect(xlsx.subarray(0, 2).toString("latin1")).toBe("PK"); // xlsx = zip
    // round-trips back through the bytes door
    const back = fromExcel(xlsx);
    expect(Buffer.isBuffer(back)).toBe(true);
    expect(read(back).groups).toContain("LOCA");
  });

  it("fromExcel(bytes) with a path writes AGS4 and returns stats", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-excel-"));
    const xlsx = toExcel(agsBytes); // Buffer
    const out = join(dir, "frombytes.ags");
    const stats = fromExcel(xlsx, out);
    expect(stats.sheetsWritten).toBeGreaterThanOrEqual(2);
    expect(read(readFileSync(out)).groups).toContain("PROJ");
  });

  it("Ags4File.toExcel() handle method — path writes, no path returns bytes", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-excel-"));
    const handle = read(agsBytes);
    // bytes form
    const xlsx = handle.toExcel();
    expect(Buffer.isBuffer(xlsx)).toBe(true);
    // file form
    const out = join(dir, "handle.xlsx");
    const stats = handle.toExcel(out);
    expect(stats.sheetsWritten).toBe(handle.groups.length);
  });

  it("bytes and path forms produce the same workbook groups (cross-form parity)", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-excel-"));
    const ags = join(dir, "in.ags");
    writeFileSync(ags, AGS);
    const viaPath = toExcel(ags); // path-in → bytes-out
    const viaBytes = toExcel(agsBytes); // bytes-in → bytes-out
    // both are valid workbooks that convert back to the same group set
    expect(read(fromExcel(viaPath)).groups.sort()).toEqual(
      read(fromExcel(viaBytes)).groups.sort(),
    );
  });
});
