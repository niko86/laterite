// what this shows: build a typed PROJ graph (objects, not tables) and emit valid AGS4 from it.
import { buildAgs4, read, PROJ, LOCA } from "laterite";
import assert from "node:assert/strict";

// A typed PROJ graph — children attach via the `locas` array.
const p = new PROJ({
  PROJ_ID: "LAT-DEMO",
  PROJ_NAME: "Built from a typed graph",
});
p.locas.push(new LOCA({ LOCA_ID: "BH01", LOCA_GL: 12.5 })); // attach via push …
new PROJ({
  PROJ_ID: "P2",
  locas: [new LOCA({ LOCA_ID: "BH02", LOCA_GL: 13.75 })],
}); // … or the ctor field

// buildAgs4 walks the graph depth-first (#214), emits only the headings you set,
// and autofix synthesizes the metadata catalogs — a sparse graph builds valid in one call.
const res = buildAgs4(p);
const groups = read(res.bytes).groups;
console.log("groups:", groups);
console.log("findings:", res.findings.length);

assert.ok(
  ["PROJ", "LOCA", "TRAN", "UNIT", "TYPE"].every((c) => groups.includes(c)),
);
assert.equal(res.findings.length, 0); // a valid file, no caveats
