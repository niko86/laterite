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
const res = buildAgs4(new Map([["PROJ", proj], ["LOCA", loca]])); // default mode "autofix"
const groups = read(res.bytes).groups;
console.log("groups:", groups);
console.log("findings:", res.findings.length);

// Autofix synthesizes the mandatory metadata catalogs (TRAN/UNIT/TYPE), so a
// data-only build is valid in one call.
assert.ok(["PROJ", "LOCA", "TRAN", "UNIT", "TYPE"].every((c) => groups.includes(c)));
assert.equal(res.findings.length, 0);
