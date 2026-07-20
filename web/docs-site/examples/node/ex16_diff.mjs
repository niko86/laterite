// what this shows: diff(a, b) — a KEY-aware, type-aware revision diff between two AGS4 revisions.
import { diff } from "laterite";
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";

// Two revisions of the same submission, differing in one PROJ cell (PROJ_NAME).
// A bare string is a PATH in Node, so pass bytes for an in-memory revision.
const baseline = readFileSync("examples/sample_site.ags");
const revision = Buffer.from(
  baseline
    .toString("utf8")
    .replace(
      "laterite demo site (synthetic starter - replace me)",
      "laterite demo site (Rev B)",
    ),
);
assert.ok(!revision.equals(baseline));

// diff() returns a RevisionDelta: per-group row/heading deltas + counts. Rows are
// matched by the group's dictionary KEY headings, and cells are compared through
// the typed value — so only a genuine quantity change registers. The shape is
// byte-identical to Python / wasm / `lat-check --diff`.
const delta = diff(baseline, revision);

const proj = delta.groups.find((g) => g.code === "PROJ");
const changed = proj.rows.filter((r) => r.kind === "changed");

console.log(
  "totals:",
  delta.total_added,
  delta.total_removed,
  delta.total_changed,
);
console.log("PROJ key headings:", proj.key_headings);
console.log("changed row key:", changed[0].key);
console.log("changed cell:", changed[0].cells[0]);

assert.equal(delta.total_changed, 1);
assert.deepEqual(proj.key_headings, ["PROJ_ID"]);
assert.equal(proj.keyed, true);
assert.deepEqual(changed[0].key, ["LAT-DEMO"]); // the PROJ_ID value
const cell = changed[0].cells[0];
assert.equal(cell.heading, "PROJ_NAME");
assert.equal(cell.type, "X");
assert.equal(cell.b, "laterite demo site (Rev B)");
