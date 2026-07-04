// what this shows: the certify fast-path — a fresh .ags.idx cert lets validate() skip the rule engine.
import { read } from "laterite";
import assert from "node:assert/strict";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const tmp = mkdtempSync(join(tmpdir(), "laterite-docs-"));
const site = join(tmp, "site.ags");
copyFileSync("examples/sample_site.ags", site);

// certify() needs a prior clean validate() on the same handle; it mints <path>.ags.idx.
const idx = read(site).validate({ warnings: false }).certify();

// Re-reading with the fresh cert lets validate() resolve without running the rule engine.
const ags = read(site, { index: idx }).validate({ warnings: false });
console.log(ags.report.resolution);

assert.equal(ags.report.resolution, "certified");
rmSync(tmp, { recursive: true, force: true });
