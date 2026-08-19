// Carry the shared token layer onto the docs site (#401).
//
// Same contract as sync-icons.mjs: the COMMITTED output is the artefact, this
// script is how it is refreshed, and CI runs `--check` so a shared-layer edit
// that nobody re-synced fails loudly instead of leaving docs.laterite.dev on
// last month's palette.
//
// WHY A COPY AT ALL. The app and the apex both build with Vite and resolve
// `@shared/styles/tokens.css` through it. MkDocs has no bundler — it copies
// `docs/` verbatim — so it can follow neither the relative `@import` chain nor
// the Fontsource package specifiers inside fonts.css. The choice is a copy or a
// second hand-written palette, and a generated copy is the one that cannot
// disagree with its source.
//
// WHAT IT EMITS
//   docs/stylesheets/tokens.css  — one bundle: the @font-face blocks with their
//                                  urls repointed, then every stylesheet
//                                  tokens.css composes, in tokens.css's order.
//   docs/fonts/*.woff2|woff      — exactly the files those faces name.
//
// ONE bundle rather than a mirrored @import tree because the browser resolves
// `@import` with a serial round trip per file, and this is chrome — it blocks
// first paint. The order is read from tokens.css rather than restated, so a file
// added to the shared layer arrives here without anyone remembering to add it.
//
// The dark rule is rewritten onto Material's `data-md-color-scheme` attribute
// (see docs-tokens.mjs). That is what lets the docs run the same dark VALUES as
// the other two surfaces while Material's own palette toggle keeps working.

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  localImports,
  packageImports,
  rewriteFontUrls,
  toMaterialScheme,
} from "./docs-tokens.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const web = path.resolve(here, "..");
const sharedDir = path.join(web, "src/shared/styles");
const nodeModules = path.join(web, "node_modules");
const docsDir = path.join(web, "docs-site/docs");
const bundlePath = path.join(docsDir, "stylesheets/tokens.css");
const fontsDir = path.join(docsDir, "fonts");

// Relative to the bundle, which sits in stylesheets/ beside docs/fonts/.
const FONT_URL_PREFIX = "../fonts/";
const check = process.argv.includes("--check");

const BANNER = `/* GENERATED FILE — DO NOT EDIT.
 *
 * The shared token layer (#394), carried onto the docs site by
 * web/scripts/sync-docs-tokens.mjs. The source is web/src/shared/styles/ — edit
 * there and re-run \`npm run sync-docs-tokens\`; CI runs it with --check.
 *
 * The one transform applied on the way: the shared layer's \`.dark\` rule becomes
 * Material's \`[data-md-color-scheme="slate"]\`, so the docs render the same dark
 * values the app and the apex do and Material's palette toggle still drives them.
 */
`;

if (!existsSync(nodeModules)) {
  throw new Error(
    `node_modules is absent (looked in ${path.relative(process.cwd(), nodeModules)}) — run npm ci first.`,
  );
}

// ── Build the bundle ────────────────────────────────────────────────────────
const entry = readFileSync(path.join(sharedDir, "tokens.css"), "utf8");
const parts = [BANNER];
const wanted = new Set();
let darkRewrites = 0;

for (const name of localImports(entry)) {
  const source = readFileSync(path.join(sharedDir, name), "utf8");
  parts.push(`\n/* ── src/shared/styles/${name} ── */\n`);

  if (name === "fonts.css") {
    // The @import lines ARE the content here — resolve each through
    // node_modules and inline the faces they carry.
    const specs = packageImports(source);
    if (!specs.length) {
      throw new Error(
        `${name} declares no Fontsource imports — the docs would ship no faces.`,
      );
    }
    // Keep the prose: it explains why the families are self-hosted at all, and
    // this bundle is where a docs reader-of-source lands.
    parts.push(source.slice(0, source.indexOf("@import")).trimEnd(), "\n");
    for (const spec of specs) {
      const file = path.join(nodeModules, spec);
      const { css, files } = rewriteFontUrls(
        readFileSync(file, "utf8"),
        FONT_URL_PREFIX,
      );
      files.forEach((f) =>
        wanted.add(path.join(path.dirname(file), "files", f)),
      );
      parts.push(css.trimEnd(), "\n");
    }
    continue;
  }

  // Every file carrying a dark set gets the rewrite. This used to insist on
  // exactly ONE such file, which was true when colors.css was the only one —
  // but the count was never the thing worth protecting. What the guard is for
  // is the silent no-op: a layer that reaches the docs with no dark rule at
  // all looks like a stylesheet still loading. charts.css (#434) legitimately
  // carries its own dark values, so the invariant is at least one, below.
  if (/^\.dark(?=\s*\{)/m.test(source)) {
    parts.push(toMaterialScheme(source));
    darkRewrites += 1;
  } else {
    parts.push(source);
  }
}

if (darkRewrites === 0) {
  throw new Error(
    "no shared stylesheet carries a dark set — the docs would ship light-only.",
  );
}

const bundle = parts.join("");
const fontFiles = [...wanted].sort();

// ── Write, or check ─────────────────────────────────────────────────────────
const rel = (p) => path.relative(web, p);
const stale = [];

const bundleDrifted =
  !existsSync(bundlePath) || readFileSync(bundlePath, "utf8") !== bundle;
if (bundleDrifted) stale.push(rel(bundlePath));

const expectedNames = new Set(fontFiles.map((f) => path.basename(f)));
for (const src of fontFiles) {
  const dest = path.join(fontsDir, path.basename(src));
  if (!existsSync(dest) || !readFileSync(dest).equals(readFileSync(src))) {
    stale.push(rel(dest));
  }
}
// An orphan is drift too: a family swapped out leaves its old faces behind, and
// they are dead weight nothing ever requests again.
const present = existsSync(fontsDir)
  ? readdirSync(fontsDir).filter((f) => /\.woff2?$/.test(f))
  : [];
const orphans = present.filter((f) => !expectedNames.has(f));
stale.push(...orphans.map((f) => `${rel(path.join(fontsDir, f))} (orphan)`));

if (check) {
  if (stale.length) {
    console.error(
      `docs token bundle is stale — re-run \`npm run sync-docs-tokens\`:\n` +
        stale.map((s) => `  ${s}`).join("\n"),
    );
    process.exit(1);
  }
  console.log(
    `docs tokens in sync (${fontFiles.length} font files, ${bundle.length} bytes of CSS).`,
  );
} else {
  mkdirSync(path.dirname(bundlePath), { recursive: true });
  mkdirSync(fontsDir, { recursive: true });
  writeFileSync(bundlePath, bundle);
  for (const src of fontFiles) {
    copyFileSync(src, path.join(fontsDir, path.basename(src)));
  }
  for (const f of orphans) rmSync(path.join(fontsDir, f));
  console.log(
    `wrote ${rel(bundlePath)} + ${fontFiles.length} font files` +
      (orphans.length ? ` (pruned ${orphans.length} orphan)` : ""),
  );
}
