// what this shows: the certify fast-path — a fresh .ags.idx cert lets validate() skip the rule engine.
import { read } from "laterite";
import assert from "node:assert/strict";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const tmp = mkdtempSync(join(tmpdir(), "laterite-docs-"));
const site = join(tmp, "site.ags");
copyFileSync("examples/sample_site.ags", site);

// certify() runs the validation itself and mints <path>.ags.idx for an error-clean file.
const idx = read(site).certify();

// Re-reading with the fresh cert lets validate() answer without running the rule engine.
const ags = read(site, { index: idx }).validate({ warnings: false });
// `certified` says the ENGINE was skipped; `dictVersion` still says which dictionary judged it.
console.log(ags.report.certified, ags.report.dictVersion);

assert.equal(ags.report.certified, true);
rmSync(tmp, { recursive: true, force: true });
