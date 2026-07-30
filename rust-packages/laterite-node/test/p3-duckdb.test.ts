// P3 — the optional DuckDB layer (Appender ingest, row-object output): sql()
// cross-group JOINs, at()/AgsSubset key-filtering, the raw connection escape
// hatch. Requires the optional @duckdb/node-api peer (a devDependency here).
import { existsSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Table } from "apache-arrow";
import { describe, expect, it, vi } from "vitest";
import { Ags4File, read, toDuckdb } from "../ts/index";

// Every case below stands up its own DuckDB instance, and the first one in a
// cold checkout also downloads an extension — work that is not bounded by
// vitest's 5s default. One `describe` here already carried its own 60s
// override; the other four did not, and CI duly timed one of them out at
// 5000ms on a busy runner while the diff under test touched no Node code.
// The headroom is a property of the engine this file drives, so it is set once
// for the file rather than remembered per block.
vi.setConfig({ testTimeout: 60_000 });

/** The slice of the raw `@duckdb/node-api` connection the persistence checks use
 * — reached via `Ags4File.connection`, so the tests never import the optional
 * peer directly (the library is the only door to it). */
type RawCon = {
  run(sql: string): Promise<unknown>;
  runAndReadAll(
    sql: string,
  ): Promise<{ getRowObjectsJS(): Record<string, unknown>[] }>;
};

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
});

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

describe("toDuckdb() — persist the keyed relational store", () => {
  // PROJ (root) + LOCA + SAMP — a parent chain, so the persisted keys can be
  // exercised across a cross-group JOIN.
  const AGS_REL =
    '"GROUP","PROJ"\r\n' +
    '"HEADING","PROJ_ID"\r\n' +
    '"UNIT",""\r\n' +
    '"TYPE","ID"\r\n' +
    '"DATA","P1"\r\n' +
    AGS; // LOCA (2 rows) + SAMP (3 rows)

  const outPath = (name: string) =>
    join(mkdtempSync(join(tmpdir(), "lat-duckdb-")), name);

  it("writes one keyed table per group and the keys still JOIN", async () => {
    const out = outPath("store.duckdb");
    using ags = read(undefined, { text: AGS_REL });
    const stats = await ags.toDuckdb(out);
    expect(stats).toEqual({
      path: out,
      tables_written: 3,
      rows_written: 1 + 2 + 3,
    });
    expect(existsSync(out)).toBe(true);

    // Attach the file we just wrote back into the live engine and read it — this
    // proves it persisted, WITHOUT importing the optional peer directly.
    const con = (await ags.connection) as RawCon;
    await con.run(`ATTACH '${out}' AS chk (READ_ONLY)`);
    const names = (
      await con.runAndReadAll(
        "SELECT table_name AS n FROM duckdb_tables() WHERE database_name = 'chk' ORDER BY 1",
      )
    )
      .getRowObjectsJS()
      .map((r) => r.n);
    expect(names).toEqual(["LOCA", "PROJ", "SAMP"]);
    // SAMP leads with the two content-addressed key columns...
    const cols = (await con.runAndReadAll('DESCRIBE chk."SAMP"'))
      .getRowObjectsJS()
      .map((r) => r.column_name);
    expect(cols.slice(0, 2)).toEqual(["_id", "_parent_id"]);
    // ...and the persisted keys resolve a cross-group JOIN.
    const joined = (
      await con.runAndReadAll(
        "SELECT s.SAMP_ID FROM chk.SAMP s JOIN chk.LOCA l ON s._parent_id = l._id ORDER BY s.SAMP_ID",
      )
    )
      .getRowObjectsJS()
      .map((r) => r.SAMP_ID);
    expect(joined).toEqual(["S1", "S2", "S3"]);
  });

  it("is faithful to the in-memory relational layer, row for row", async () => {
    const out = outPath("faithful.duckdb");
    using ags = read(undefined, { text: AGS_REL });
    await ags.toDuckdb(out);
    const con = (await ags.connection) as RawCon;
    await con.run(`ATTACH '${out}' AS chk (READ_ONLY)`);
    for (const code of ["PROJ", "LOCA", "SAMP"]) {
      // Persisted MINUS in-memory, both directions: any difference (incl. the
      // synthetic keys) leaves a row. Tag rows with the code so a failure names
      // the offending group (vitest's expect takes no message arg).
      const diff = (
        await con.runAndReadAll(
          `SELECT '${code}' AS g, * FROM (
             (SELECT * FROM chk."${code}" EXCEPT SELECT * FROM "${code}")
             UNION ALL (SELECT * FROM "${code}" EXCEPT SELECT * FROM chk."${code}"))`,
        )
      ).getRowObjectsJS();
      expect(diff).toEqual([]);
    }
  });

  it("refuses to overwrite an existing database", async () => {
    const out = outPath("once.duckdb");
    using a = read(undefined, { text: AGS_REL });
    await a.toDuckdb(out);
    using b = read(undefined, { text: AGS_REL });
    await expect(b.toDuckdb(out)).rejects.toThrow(/fresh database/);
  });

  it("groups= selects a subset", async () => {
    const out = outPath("subset.duckdb");
    using ags = read(undefined, { text: AGS_REL });
    const stats = await ags.toDuckdb(out, { groups: ["SAMP", "PROJ"] });
    expect(stats.tables_written).toBe(2);
    const con = (await ags.connection) as RawCon;
    await con.run(`ATTACH '${out}' AS chk (READ_ONLY)`);
    const names = (
      await con.runAndReadAll(
        "SELECT table_name AS n FROM duckdb_tables() WHERE database_name = 'chk' ORDER BY 1",
      )
    )
      .getRowObjectsJS()
      .map((r) => r.n);
    expect(names).toEqual(["PROJ", "SAMP"]);
  });

  it("throws for an unknown group", async () => {
    const out = outPath("bad.duckdb");
    using ags = read(undefined, { text: AGS_REL });
    await expect(ags.toDuckdb(out, { groups: ["NOPE"] })).rejects.toThrow(
      /not in file/,
    );
  });

  it("the free toDuckdb matches the fluent method, byte for byte", async () => {
    const a = outPath("free.duckdb");
    const b = outPath("fluent.duckdb");
    await toDuckdb(Buffer.from(AGS_REL), a);
    using ags = read(undefined, { text: AGS_REL });
    await ags.toDuckdb(b);
    const con = (await ags.connection) as RawCon;
    await con.run(`ATTACH '${a}' AS ca (READ_ONLY)`);
    await con.run(`ATTACH '${b}' AS cb (READ_ONLY)`);
    const diff = (
      await con.runAndReadAll(
        `(SELECT * FROM ca.SAMP EXCEPT SELECT * FROM cb.SAMP)
         UNION ALL (SELECT * FROM cb.SAMP EXCEPT SELECT * FROM ca.SAMP)`,
      )
    ).getRowObjectsJS();
    expect(diff).toHaveLength(0);
  });

  it("the free toDuckdb leaves a caller's handle open (does not own it)", async () => {
    const out = outPath("passed-handle.duckdb");
    // Pass an already-read handle (owned=false): the free form must persist
    // without closing it, so the caller's handle stays usable afterwards.
    const ags = read(undefined, { text: AGS_REL });
    const stats = await toDuckdb(ags, out);
    expect(stats.tables_written).toBeGreaterThan(0);
    expect(ags.has("SAMP")).toBe(true);
    ags.close();
  });
});
