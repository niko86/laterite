// what this shows: at(code, ids) fans out to a borehole's related groups; frames() materialises the record set.
// Needs the optional @duckdb/node-api peer (npm i @duckdb/node-api) — read/validate/fix don't.
import { read } from "laterite";
import assert from "node:assert/strict";

const ags = read("examples/sample_site.ags");

// at("LOCA", ids) walks the dictionary's parent graph down from LOCA and keeps
// only groups that carry rows for those locations; `.groups` is the manifest.
const q = ags.at("LOCA", ["BH01"]);
console.log(q.groups.sort());

// frames() returns { group: rows } for the whole related record set, each
// row-filtered to just the boreholes you asked for.
const frames = await q.frames();
console.log(Object.keys(frames).sort());
console.log(frames.SAMP.length);
ags.close();

assert.ok(q.groups.includes("LOCA") && q.groups.includes("SAMP"));
assert.ok(Array.isArray(frames.SAMP) && frames.SAMP.length >= 1);
