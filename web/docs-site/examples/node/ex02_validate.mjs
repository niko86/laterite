import { validate } from "laterite";
import assert from "node:assert/strict";

// Validate a file. The Report carries the verdict plus which edition the rules
// came from — read straight off the file's TRAN_AGS; you never pass an edition.
const report = validate("examples/sample_site.ags");
console.log(
  `isValid=${report.isValid} count=${report.count} ` +
    `dictVersion=${report.dictVersion} resolution=${report.resolution}`,
);

assert.equal(report.isValid, true);
assert.equal(report.count, 0);
assert.equal(report.dictVersion, "4.1.1"); // auto-selected from TRAN_AGS
assert.equal(report.resolution, "exact");
