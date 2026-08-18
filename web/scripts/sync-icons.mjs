// Keep web/src/shared/icons/*.svg in lock-step with their single source of
// truth: the `lucide-static` package. Same contract as sync-samples.mjs — the
// COMMITTED filenames define the set we carry, and each is refreshed from the
// upstream icon of the same name; an icon upstream has dropped or renamed fails
// loudly rather than silently keeping a stale copy.
//
// The icons are VENDORED — committed files, bundled by Vite — and not loaded
// from a CDN, which is the one thing the design system flags about its own Icon
// component. That matters here more than it does for the system: the app is a
// PWA with a precache, so a CDN icon set is a validator full of unlabelled
// buttons the first time someone opens it on a train. `lucide-static` is a
// devDependency for exactly this reason — it feeds this script and never enters
// the bundle.
//
// Upstream ships ~2000 icons; we carry the working set and nothing else. To add
// one: name it in src/shared/icons/icons.ts, run `npm run sync-icons`, commit
// the .svg it writes. To find out whether Lucide HAS a match, look in
// node_modules/lucide-static/icons — and if it does not, say so rather than
// hand-drawing a replacement (the system is explicit about that).

import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.resolve(here, "../src/shared/icons");
const upstreamDir = path.resolve(here, "../node_modules/lucide-static/icons");
const manifest = path.resolve(iconsDir, "icons.ts");

if (!existsSync(upstreamDir)) {
  throw new Error(
    `lucide-static is not installed (looked in ${path.relative(process.cwd(), upstreamDir)}) — run npm ci first.`,
  );
}
mkdirSync(iconsDir, { recursive: true });

// The manifest is the set. Parsing its import specifiers rather than importing
// it keeps this a plain node script — the manifest is TypeScript, and the names
// are the one thing here that must not have a second definition.
const wanted = [
  ...readFileSync(manifest, "utf8").matchAll(
    /from "\.\/([a-z0-9-]+)\.svg\?raw"/g,
  ),
].map((m) => m[1]);

if (wanted.length === 0) {
  throw new Error(
    `no icons found in ${path.relative(process.cwd(), manifest)} — has its import shape changed?`,
  );
}

for (const name of wanted) {
  const src = path.join(upstreamDir, `${name}.svg`);
  if (!existsSync(src)) {
    throw new Error(
      `icons.ts names "${name}", which lucide-static ${JSON.parse(readFileSync(path.resolve(here, "../node_modules/lucide-static/package.json"), "utf8")).version} does not ship. ` +
        `It may have been renamed upstream — check node_modules/lucide-static/icons and update the manifest.`,
    );
  }
  writeFileSync(path.join(iconsDir, `${name}.svg`), readFileSync(src));
}

// Anything vendored but no longer named is dead weight in the precache.
const orphans = readdirSync(iconsDir)
  .filter((f) => f.endsWith(".svg"))
  .filter((f) => !wanted.includes(f.slice(0, -4)));
if (orphans.length > 0) {
  throw new Error(
    `vendored icons no longer named in icons.ts: ${orphans.join(", ")} — delete them, or add them back to the manifest.`,
  );
}

console.log(
  `sync-icons: ${wanted.length} icon(s) refreshed from lucide-static`,
);
