// what this shows: listRules() enumerates the numbered AGS4 rules; each RuleMeta carries severity + fixable.
import { listRules } from "laterite";
import assert from "node:assert/strict";

const rules = listRules();
console.log(rules.length);
console.log(Object.keys(rules[0]).sort());

// The mechanically-fixable rules, straight off the metadata — no hard-coded numbers.
const fixable = rules.filter((r) => r.fixable).map((r) => r.rule);
console.log("fixable:", fixable.join(", "));

assert.ok(rules.length >= 20);
assert.deepEqual(Object.keys(rules[0]).sort(), [
  "checks",
  "fixable",
  "observations",
  "rule",
  "severity",
  "title",
]);
assert.ok(fixable.includes("1")); // Rule 1 (character set) is mechanically fixable
