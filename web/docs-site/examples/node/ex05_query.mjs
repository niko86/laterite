// what this shows: sql() as Node's equivalent of Python's lazy query builder — filter + select live in the SQL.
// Needs the optional @duckdb/node-api peer (npm i @duckdb/node-api) — read/validate/fix don't.
import { read } from "laterite";
import assert from "node:assert/strict";

const ags = read("examples/sample_site.ags");

// Node has one query door — sql() — not a lazy .query()/.filter()/.select() chain:
// narrow rows and columns in the statement itself. The dtype IS the AGS type, so
// `LOCA_GL > 28` compares numbers, not strings.
const rows = await ags.sql(
  "SELECT LOCA_ID, LOCA_TYPE, LOCA_GL FROM LOCA WHERE LOCA_GL > 28 ORDER BY LOCA_ID",
);
console.log(rows);
ags.close();

assert.equal(rows.length, 7);
assert.deepEqual(Object.keys(rows[0]), ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"]);
assert.ok(rows.every((r) => r.LOCA_GL > 28)); // numeric compare, not lexical
