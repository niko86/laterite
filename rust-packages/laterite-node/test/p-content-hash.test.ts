// Optional `_content_hash` Arrow column (#448, Node — PR D1 of the rollout).
//
// `_id` fingerprints a row's IDENTITY (its KEY chain); `_content_hash`
// fingerprints its VALUE. Two deliveries of borehole BH02 with a corrected
// level share an `_id` and differ here. Off by default (`read(...,
// { contentHash: true })` opts in) — the plain table stays byte-identical
// without it.
//
// SAME fixture as the Python `test_content_hash.py` (`_D1`/`_D2`), and the
// SAME golden UUIDv8 hash values, pinned from a release build of the wheel —
// Node and Python both route through the one shared
// `keychain::group_content_hashes`, so a match here IS the cross-surface
// parity proof (mirrors how `p3-content-keys.test.ts` pins `test_content_keys.py`'s
// `_id` goldens).
import { describe, expect, it } from "vitest";
import { read } from "../ts/index";

// Two deliveries of one project.
//   D1: BH01 (GL 10.00), BH02 (GL 12.00).            NO LOCA_REM column at all.
//   D2: BH01 UNCHANGED but the level is re-emitted "10.0" (formatting only),
//       BH02 REVISED (12.00 -> 12.75), BH03 new.     PLUS a blank LOCA_REM column.
const D1 =
  '"GROUP","PROJ"\r\n' +
  '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","X"\r\n' +
  '"DATA","P100","Demo"\r\n' +
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"\r\n' +
  '"UNIT","","m","m"\r\n' +
  '"TYPE","ID","2DP","2DP"\r\n' +
  '"DATA","BH01","523400.00","10.00"\r\n' +
  '"DATA","BH02","523500.00","12.00"\r\n';
const D2 =
  '"GROUP","PROJ"\r\n' +
  '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","X"\r\n' +
  '"DATA","P100","Demo"\r\n' +
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL","LOCA_REM"\r\n' +
  '"UNIT","","m","m",""\r\n' +
  '"TYPE","ID","2DP","2DP","X"\r\n' +
  '"DATA","BH01","523400.00","10.0",""\r\n' +
  '"DATA","BH02","523500.00","12.75",""\r\n' +
  '"DATA","BH03","523600.00","9.25",""\r\n';

// Pinned from `laterite.read(text=D1/D2, content_hash=True, keys=True)["LOCA"]`
// on a release build of the Python wheel (packages/laterite).
const D1_BH01_ID = "6cd3aa0e-5673-8c1b-a2a0-9da0bab584d6";
const D1_BH01_HASH = "1bd2eb52-b18a-8427-b176-86d9881b1119";
const D1_BH02_ID = "261f9272-23f4-83c0-bffb-e6afda5008e1";
const D1_BH02_HASH = "462fff84-6fb9-8631-a5d1-e728db23568d";
const D2_BH02_HASH = "28b39176-eb14-8a42-86b9-c9e7b6b74e39";
const D2_BH03_ID = "d9d229d2-554e-8f54-9b6c-803d8a0e1a01";
const D2_BH03_HASH = "accb778f-a86b-88c7-b28d-344a6e61c631";

/** Read one row's `column` value by `LOCA_ID`, from an already-decoded Table. */
function cellFor(
  table: {
    numRows: number;
    getChild(name: string): { get(i: number): unknown } | null;
  },
  locaId: string,
  column: string,
): unknown {
  const ids = table.getChild("LOCA_ID")!;
  const values = table.getChild(column)!;
  for (let i = 0; i < table.numRows; i++) {
    if (ids.get(i) === locaId) return values.get(i);
  }
  throw new Error(`${locaId} not found`);
}

describe("_content_hash (#448, Node)", () => {
  it("default read carries no _content_hash column anywhere", async () => {
    using ags = read(undefined, { text: D1 });
    expect(ags.table("LOCA").schema.fields.map((f) => f.name)).not.toContain(
      "_content_hash",
    );
    // The relational sql() layer carries _id/_parent_id but must NOT gain a
    // third synthetic column.
    const rows = await ags.sql("SELECT * FROM LOCA LIMIT 1");
    expect(Object.keys(rows[0]!)).not.toContain("_content_hash");
    expect(Object.keys(rows[0]!)).toContain("_id"); // sanity: always-keyed relational layer
  });

  it("{ contentHash: true } adds a _content_hash column that survives the default (unkeyed) view", () => {
    const ags = read(undefined, { text: D1, contentHash: true });
    const table = ags.table("LOCA"); // default: no { keys: true }
    const names = table.schema.fields.map((f) => f.name);
    expect(names).not.toContain("_id"); // ids ARE stripped by default
    expect(names).not.toContain("_parent_id");
    expect(names).toContain("_content_hash"); // the hash is NOT stripped
    expect(typeof cellFor(table, "BH01", "_content_hash")).toBe("string");
  });

  it("golden hash values match Python for both deliveries", () => {
    const a = read(undefined, { text: D1, contentHash: true }).table("LOCA");
    const b = read(undefined, { text: D2, contentHash: true }).table("LOCA");
    expect(cellFor(a, "BH01", "_content_hash")).toBe(D1_BH01_HASH);
    expect(cellFor(a, "BH02", "_content_hash")).toBe(D1_BH02_HASH);
    // BH01 unchanged: a formatting-only reemit ("10.0" vs "10.00") and a new
    // blank LOCA_REM column do NOT change the hash (typed + blank-insensitive).
    expect(cellFor(b, "BH01", "_content_hash")).toBe(D1_BH01_HASH);
    // BH02 revised (12.00 -> 12.75): the hash MUST move.
    expect(cellFor(b, "BH02", "_content_hash")).toBe(D2_BH02_HASH);
    expect(cellFor(b, "BH03", "_content_hash")).toBe(D2_BH03_HASH);
  });

  it("a revised row shares its _id but not its _content_hash", () => {
    const a = read(undefined, { text: D1, contentHash: true }).table("LOCA", {
      keys: true,
    });
    const b = read(undefined, { text: D2, contentHash: true }).table("LOCA", {
      keys: true,
    });
    expect(cellFor(a, "BH01", "_id")).toBe(D1_BH01_ID);
    expect(cellFor(b, "BH01", "_id")).toBe(D1_BH01_ID); // same identity
    expect(cellFor(a, "BH02", "_id")).toBe(D1_BH02_ID);
    expect(cellFor(b, "BH02", "_id")).toBe(D1_BH02_ID); // same borehole → same identity
    expect(cellFor(a, "BH02", "_content_hash")).not.toBe(
      cellFor(b, "BH02", "_content_hash"),
    );
    expect(cellFor(b, "BH03", "_id")).toBe(D2_BH03_ID);
  });

  it("hashes are deterministic across independent reads", () => {
    const hashOf = (): unknown =>
      cellFor(
        read(undefined, { text: D1, contentHash: true }).table("LOCA"),
        "BH01",
        "_content_hash",
      );
    expect(hashOf()).toBe(hashOf());
    expect(hashOf()).toBe(D1_BH01_HASH);
  });

  it("sql() exposes _content_hash for querying, and DISTINCT ON it collapses value-identical rows", async () => {
    // Query each delivery through the engine (proves the schema-driven `register()`
    // carries the trailing synthetic column all the way into DuckDB) then combine the
    // two result sets in JS — same headline claim `test_content_hash.py` makes (a
    // shared `_id` for BH02 across BOTH deliveries, but a different `_content_hash`
    // after its revision): 3 distinct row IDENTITIES, 4 distinct row VALUES over 5
    // total rows across the two deliveries.
    using ags1 = read(undefined, { text: D1, contentHash: true });
    using ags2 = read(undefined, { text: D2, contentHash: true });
    const rowsA = await ags1.sql("SELECT _id, _content_hash FROM LOCA");
    const rowsB = await ags2.sql("SELECT _id, _content_hash FROM LOCA");
    const all = [...rowsA, ...rowsB];
    expect(all).toHaveLength(5);
    expect(new Set(all.map((r) => r._id)).size).toBe(3); // 3 boreholes — identity
    expect(new Set(all.map((r) => r._content_hash)).size).toBe(4); // BH01 dedups, BH02 revision doesn't
  });
});
