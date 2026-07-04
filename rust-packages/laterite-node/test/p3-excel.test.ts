// #358 — Node Excel I/O. Binds the SAME `laterite-excel` converter Python's
// to_excel / from_excel use, so AGS4 ↔ XLSX round-trips through Node.
import { mkdtempSync, writeFileSync } from "node:fs";
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

describe("Excel I/O (#358)", () => {
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
    const stats = toExcel(ags, join(dir, "o.xlsx"), { groups: ["LOCA", "PROJ"] });
    expect(stats.sheetsWritten).toBe(2);
    expect(stats.warnings).toEqual([]);
  });
});
