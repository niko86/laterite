// what this shows: buildAgs4(entries) — valid AGS4 from your own per-group row data (headings as keys).
import { buildAgs4, read } from "laterite";
import assert from "node:assert/strict";

// Each group's rows are plain objects whose KEYS are the AGS headings.
const proj = [{ PROJ_ID: "LAT-DEMO", PROJ_NAME: "Demo site" }];
const loca = [
  { LOCA_ID: "BH01", LOCA_GL: 12.5 },
  { LOCA_ID: "BH02", LOCA_GL: 13.75 },
];

// A Map (or Array of [code, rows]) — group order is preserved, so put PROJ first.
const res = buildAgs4(
  new Map([
    ["PROJ", proj],
    ["LOCA", loca],
  ]),
); // default mode "autofix"
const groups = read(res.bytes).groups;
console.log("groups:", groups);
console.log("findings:", res.findings.length);

// You get back exactly the groups you supplied. AGS4 also mandates the metadata
// catalogs (TRAN/UNIT/TYPE), which your rows don't carry — so those are REPORTED
// rather than invented:
assert.deepEqual(groups, ["PROJ", "LOCA"]);
const rules = new Set(res.findings.map((f) => f.rule));
for (const r of [
  "AGS Format Rule 14",
  "AGS Format Rule 15",
  "AGS Format Rule 17",
]) {
  assert.ok(rules.has(r), `expected ${r}`);
}

// Ask for them and UNIT and TYPE are derived from your columns. TRAN is not
// derivable — only you know who sent what to whom — so you state it, and a
// build that doesn't reports the gap instead of inventing a placeholder that
// would satisfy the rule while asserting a transmission that never happened.
// Opt-in either way, so nothing appears in your file that you didn't ask for.
const full = buildAgs4(
  new Map([
    ["PROJ", proj],
    ["LOCA", loca],
  ]),
  {
    synthesiseMetadata: true,
    tranIssue: "1",
    tranDate: "2026-07-30",
    tranProducer: "Demo Producer",
    tranRecipient: "Demo Recipient",
    tranStatus: "Final",
  },
);
const fullGroups = read(full.bytes).groups;
assert.ok(
  ["PROJ", "LOCA", "TRAN", "UNIT", "TYPE"].every((c) => fullGroups.includes(c)),
);
assert.equal(full.findings.length, 0);
