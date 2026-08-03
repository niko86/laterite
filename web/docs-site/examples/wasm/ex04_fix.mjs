// what this shows: the two-step repair — compute_fixes() proposes, apply_fixes()
// rewrites. They are separate so you can show the user what will change first.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import init, {
  compute_fixes,
  apply_fixes,
  validate,
} from "@laterite/ags4-wasm";

await init({
  module_or_path: readFileSync(
    fileURLToPath(import.meta.resolve("@laterite/ags4-wasm/ags4_wasm_bg.wasm")),
  ),
});

// The DATA row is SHORT — three fields where HEADING declares four (Rule 4).
const dirty = new TextEncoder().encode(
  '"GROUP","LOCA"\r\n' +
    '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"\r\n' +
    '"UNIT","","",""\r\n' +
    '"TYPE","ID","PA","2DP"\r\n' +
    '"DATA","BH01","BH"\r\n',
);

const rulesIn = (report) => report.findings.map((g) => g.rule).sort();

// Each fix carries its `kind`, the `rule` it answers, the `line` and a `risk` —
// which is why this is a separate call: the app renders them as a reviewable
// before/after list rather than rewriting the file behind the user's back.
const fixes = compute_fixes(dirty);
console.log(fixes.map((f) => `${f.kind}(${f.risk})`).join(" "));

// `apply_fixes` takes the ledger back, so you can hand it a SUBSET — whatever
// the user actually ticked — not just everything that was proposed.
const repaired = apply_fixes(dirty, null, fixes);

console.log("before:", rulesIn(validate(dirty)).join(" "));
console.log("after: ", rulesIn(validate(repaired)).join(" "));

// Rule 4 is gone. The rest are the mandatory catalogs this fragment never had —
// and repair will not invent them, any more than the emitter will: a PROJ or a
// TRAN is an authorial fact, so it is REPORTED, not fabricated. "Fixed" here
// means "the defects a machine can settle are settled", not "now valid".
assert.ok(fixes.some((f) => f.kind === "pad_short_row"));
assert.ok(rulesIn(validate(dirty)).includes("AGS Format Rule 4"));
assert.ok(!rulesIn(validate(repaired)).includes("AGS Format Rule 4"));
assert.equal(validate(repaired).ok, false); // still missing PROJ / TRAN / UNIT / TYPE
