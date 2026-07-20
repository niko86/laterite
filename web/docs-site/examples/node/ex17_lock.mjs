// what this shows: age-encrypted transport round-trip — lock a file with a passphrase, unlock it, prove byte-identical.
import { transport } from "laterite";
import assert from "node:assert/strict";
import { copyFileSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const tmp = mkdtempSync(join(tmpdir(), "laterite-docs-"));
const src = join(tmp, "site.ags");
copyFileSync("examples/sample_site.ags", src);

// lock = zstd pack + age passphrase encrypt (scrypt KDF + ChaCha20-Poly1305).
transport.lock(
  src,
  join(tmp, "site.ags.zst.age"),
  "correct horse battery staple",
);
transport.unlock(
  join(tmp, "site.ags.zst.age"),
  join(tmp, "restored.ags"),
  "correct horse battery staple",
);

const original = readFileSync(src);
const restored = readFileSync(join(tmp, "restored.ags"));
console.log(`round-trip byte-identical: ${original.equals(restored)}`);

// Encryption is transparent to the payload: unlock restores the exact bytes.
assert.ok(original.equals(restored));
rmSync(tmp, { recursive: true, force: true });
