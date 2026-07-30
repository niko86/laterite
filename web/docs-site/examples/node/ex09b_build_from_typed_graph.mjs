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

// buildAgs4 walks the graph depth-first (#214) and emits only the headings you
// set — so a sparse graph stays sparse.
const res = buildAgs4(p);
const groups = read(res.bytes).groups;
console.log("groups:", groups);
console.log("findings:", res.findings.length);

// The root-metadata groups (TRAN/UNIT/TYPE/ABBR/DICT) have no parent, so they
// are not part of a PROJ-rooted graph. They are reported, not invented:
assert.deepEqual(groups, ["PROJ", "LOCA"]);
assert.ok(res.findings.length > 0);

// `synthesiseMetadata` derives the ones that CAN be derived. PROJ, DICT and
// TRAN are never invented: a project identity, a schema extension and a record
// of transmission are yours to state. A guessed DICT parent would quietly
// mislead the relational checks, and a placeholder TRAN would satisfy the rule
// while asserting a transmission that never happened.
const full = buildAgs4(p, {
  synthesiseMetadata: true,
  tran: {
    issue: "1",
    date: "2026-07-30",
    producer: "Demo Producer",
    recipient: "Demo Recipient",
    status: "Final",
  },
});
const fullGroups = read(full.bytes).groups;
assert.ok(
  ["PROJ", "LOCA", "TRAN", "UNIT", "TYPE"].every((c) => fullGroups.includes(c)),
);
assert.equal(full.findings.length, 0); // valid in one call, because you asked
