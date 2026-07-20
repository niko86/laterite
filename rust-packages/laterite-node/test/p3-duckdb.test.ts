// P3 — the optional DuckDB layer (Appender ingest, row-object output): sql()
// cross-group JOINs, at()/AgsSubset key-filtering, the raw connection escape
// hatch. Requires the optional @duckdb/node-api peer (a devDependency here).
import type { Table } from "apache-arrow";
import { describe, expect, it } from "vitest";
import { Ags4File, read } from "../ts/index";

const AGS =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL","LOCA_CKED","LOCA_STAR"\r\n' +
  '"UNIT","","m","",""\r\n' +
  '"TYPE","ID","2DP","YN","DT"\r\n' +
  '"DATA","BH01","12.30","Y","2023-02-22"\r\n' +
  '"DATA","BH02","","N","2023-03-01"\r\n' +
  '"GROUP","SAMP"\r\n' +
  '"HEADING","LOCA_ID","SAMP_ID","SAMP_TOP"\r\n' +
  '"UNIT","","","m"\r\n' +
  '"TYPE","ID","ID","2DP"\r\n' +
  '"DATA","BH01","S1","1.50"\r\n' +
  '"DATA","BH01","S2","3.00"\r\n' +
  '"DATA","BH02","S3","2.00"\r\n';

describe("sql() — cross-group JOIN, row objects", () => {
  it("joins SAMP to LOCA and returns JS-native typed values", async () => {
    using ags = read(undefined, { text: AGS });
    const rows = await ags.sql(
      `SELECT s.SAMP_ID, s.SAMP_TOP, l.LOCA_GL, l.LOCA_CKED, l.LOCA_STAR
       FROM SAMP s JOIN LOCA l USING (LOCA_ID) ORDER BY s.SAMP_ID`,
    );
    expect(rows.map((r) => r.SAMP_ID)).toEqual(["S1", "S2", "S3"]);
    // 2DP → JS number; empty cell (BH02) → real null.
    expect(rows[0]!.LOCA_GL).toBe(12.3);
    expect(rows[2]!.LOCA_GL).toBeNull();
    // YN → boolean.
    expect(rows[0]!.LOCA_CKED).toBe(true);
    expect(rows[2]!.LOCA_CKED).toBe(false);
    // DT → JS Date (both rows have a date; only BH02's LOCA_GL was empty).
    expect(rows[0]!.LOCA_STAR).toBeInstanceOf(Date);
    expect((rows[0]!.LOCA_STAR as Date).toISOString()).toBe(
      "2023-02-22T00:00:00.000Z",
    );
    expect((rows[2]!.LOCA_STAR as Date).toISOString()).toBe(
      "2023-03-01T00:00:00.000Z",
    );
  });

  it("a WHERE pushes into the engine", async () => {
    using ags = read(undefined, { text: AGS });
    const rows = await ags.sql(
      `SELECT COUNT(*) AS n FROM SAMP WHERE LOCA_ID = 'BH01'`,
    );
    expect(Number(rows[0]!.n)).toBe(2);
  });
});

describe("sql({ arrow: true }) — opt-in arrow-js Table output", () => {
  // Loads the `arrow` community extension on first use (needs network once).
  it("returns a born-typed arrow-js Table", async () => {
    using ags = read(undefined, { text: AGS });
    const table: Table = await ags.sql(
      `SELECT l.LOCA_ID, l.LOCA_GL, l.LOCA_STAR FROM LOCA l ORDER BY l.LOCA_ID`,
      { arrow: true },
    );
    expect(table.numRows).toBe(2);
    expect(table.getChild("LOCA_GL")!.type.toString()).toMatch(/Float64/);
    expect(table.getChild("LOCA_STAR")!.type.toString()).toMatch(/Timestamp/);
    expect(table.getChild("LOCA_GL")!.get(0)).toBe(12.3);
    expect(table.getChild("LOCA_GL")!.get(1)).toBeNull(); // BH02's empty cell
    expect(table.getChild("LOCA_ID")!.get(0)).toBe("BH01");
  });

  it("at().table(code, { arrow: true }) is also a Table", async () => {
    using ags = read(undefined, { text: AGS });
    const table = await ags.at("LOCA", ["BH01"]).table("SAMP", { arrow: true });
    expect(table.numRows).toBe(2); // BH01's two samples
    expect(table.getChild("SAMP_ID")!.type.toString()).toMatch(/Utf8/);
  });
}, 60_000); // first-run extension download headroom

describe("at() / AgsSubset — key filtering across related groups", () => {
  it("filters every related group to the chosen LOCA_IDs", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("LOCA", ["BH01"]);
    expect(sub.groups.sort()).toEqual(["LOCA", "SAMP"]); // both carry LOCA_ID
    const frames = await sub.frames();
    expect(frames.LOCA!.map((r) => r.LOCA_ID)).toEqual(["BH01"]);
    expect(frames.SAMP!.map((r) => r.SAMP_ID)).toEqual(["S1", "S2"]); // BH01's two samples
  });

  it("an empty value list matches nothing", async () => {
    using ags = read(undefined, { text: AGS });
    expect(await ags.at("LOCA", []).table("SAMP")).toEqual([]);
  });

  it("chaining accumulates filters (LOCA_ID AND SAMP_ID on SAMP)", async () => {
    using ags = read(undefined, { text: AGS });
    // LOCA_ID ∈ {BH01,BH02} AND SAMP_ID ∈ {S1,S3} → S1 (BH01) + S3 (BH02).
    const rows = await ags
      .at("LOCA", ["BH01", "BH02"])
      .at("SAMP", ["S1", "S3"])
      .table("SAMP");
    expect(rows.map((r) => r.SAMP_ID).sort()).toEqual(["S1", "S3"]);
    // LOCA only carries LOCA_ID, so the SAMP_ID filter is ignored there.
    const loca = await ags
      .at("LOCA", ["BH01", "BH02"])
      .at("SAMP", ["S1", "S3"])
      .table("LOCA");
    expect(loca.map((r) => r.LOCA_ID).sort()).toEqual(["BH01", "BH02"]);
  });
});

describe("connection — raw escape hatch", () => {
  it("exposes the seeded @duckdb/node-api connection", async () => {
    using ags = read(undefined, { text: AGS });
    const con = await ags.connection;
    expect(con).toBeDefined();
    const reader = await (
      con as {
        runAndReadAll(s: string): Promise<{ getRowObjectsJS(): unknown[] }>;
      }
    ).runAndReadAll("SELECT COUNT(*) AS n FROM LOCA");
    expect(reader.getRowObjectsJS()).toHaveLength(1);
  });

  it("close() tears the engine down and is idempotent", async () => {
    const ags = read(undefined, { text: AGS });
    await ags.sql("SELECT 1");
    ags.close();
    ags.close(); // idempotent
    expect(ags).toBeInstanceOf(Ags4File);
  });
});
