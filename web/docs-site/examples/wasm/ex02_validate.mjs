// what this shows: validate() over bytes, and the ONE rule for reading a
// finding's severity — an absent `severity` means "error".
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import init, { validate } from "@laterite/ags4-wasm";

await init({
  module_or_path: readFileSync(
    fileURLToPath(import.meta.resolve("@laterite/ags4-wasm/ags4_wasm_bg.wasm")),
  ),
});

// A dirty file: the DATA row is SHORT — fewer fields than HEADING declares (Rule 4).
const dirty =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"\r\n' +
  '"UNIT","","",""\r\n' +
  '"TYPE","ID","PA","2DP"\r\n' +
  '"DATA","BH01","BH"\r\n';

const report = validate(new TextEncoder().encode(dirty));

// The engine OMITS `severity` for errors rather than spelling it out, so the
// default is load-bearing: `?? "warning"` would silently reclassify every error
// in your UI. Resolve it in one place and call that everywhere.
const severityOf = (f) => f.severity ?? "error";

const counts = { error: 0, warning: 0, fyi: 0 };
for (const group of report.findings) {
  for (const item of group.items) counts[severityOf(item)] += 1;
}

console.log(report.dict_version, report.resolution, report.finding_count);
console.log(JSON.stringify(counts));

assert.equal(report.ok, false);
assert.equal(report.error, null); // parseable — findings, not a hard failure
assert.ok(counts.error > 0, "a short DATA row is a Rule 4 error");
