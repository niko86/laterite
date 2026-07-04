// what this shows: zstd transport round-trip — pack a file to .zst, unpack it, prove byte-identical + smaller.
import { transport } from "laterite";
import assert from "node:assert/strict";
import { copyFileSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const tmp = mkdtempSync(join(tmpdir(), "laterite-docs-"));
const src = join(tmp, "site.ags");
copyFileSync("examples/sample_site.ags", src);

transport.pack(src, join(tmp, "site.ags.zst"));
transport.unpack(join(tmp, "site.ags.zst"), join(tmp, "restored.ags"));

const original = readFileSync(src);
const restored = readFileSync(join(tmp, "restored.ags"));
console.log(`original:   ${original.length} bytes`);
console.log(`compressed: ${statSync(join(tmp, "site.ags.zst")).size} bytes`);
console.log(`round-trip byte-identical: ${original.equals(restored)}`);

assert.ok(original.equals(restored));
assert.ok(statSync(join(tmp, "site.ags.zst")).size < original.length);
rmSync(tmp, { recursive: true, force: true });
