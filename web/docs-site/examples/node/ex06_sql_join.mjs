// what this shows: drop to raw SQL to join across groups, count samples per location.
// Needs the optional @duckdb/node-api peer (npm i @duckdb/node-api) — read/validate/fix don't.
import { read } from "laterite";
import assert from "node:assert/strict";

const ags = read("examples/sample_site.ags");
const rows = await ags.sql(
  "SELECT l.LOCA_ID, count(*) n FROM SAMP s JOIN LOCA l USING (LOCA_ID) " +
    "GROUP BY 1 ORDER BY 1",
);
console.log(rows);
ags.close();

assert.ok(rows.length >= 1);
assert.ok(Object.hasOwn(rows[0], "n"));
