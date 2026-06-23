// Keep web/public/samples/*.ags in lock-step with their single source of truth:
// the validator's test fixtures (rust-packages/laterite-ags4-validator/tests/
// fixtures). The web's Validate "load a sample" feature serves a SUBSET of those
// fixtures as static assets; they used to be a hand-copy that could silently
// drift from the fixtures the engine is actually tested against. This refreshes
// each committed sample from its matching fixture (predev/prebuild hook), and
// fails loudly if a web sample has no fixture of the same name.

import { copyFileSync, existsSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const samplesDir = path.resolve(here, "../public/samples");
const fixturesDir = path.resolve(
  here,
  "../../rust-packages/laterite-ags4-validator/tests/fixtures",
);

// The committed sample filenames define the set the web serves; each must mirror
// the validator fixture of the same name.
for (const name of readdirSync(samplesDir).filter((f) => f.endsWith(".ags"))) {
  const src = path.join(fixturesDir, name);
  if (!existsSync(src)) {
    throw new Error(
      `web/public/samples/${name} has no matching validator fixture at ` +
        `${path.relative(process.cwd(), src)} — rename it or add the fixture.`,
    );
  }
  copyFileSync(src, path.join(samplesDir, name));
}
console.log(
  `[sync-samples] refreshed web/public/samples from ${path.relative(process.cwd(), fixturesDir)}`,
);
