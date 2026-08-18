// No emoji in product UI (#406).
//
// The design system states this as a rule and it is one of the two rules it
// says are worth encoding in review rather than hoping for. The reason is tone:
// the audience is geotechnical engineers wiring AGS4 into pipelines, and a 🎉
// on a validation result reads as a consumer app congratulating someone for
// doing their job. It is also a legibility problem — emoji render differently
// on every platform and carry no meaning in greyscale or in a pasted issue.
//
// A SMALL SET OF UNICODE GLYPHS IS DELIBERATELY SANCTIONED and must keep
// working: the sun and moon on the theme toggle, the verdict marks, the ellipsis
// for elided rows, the arrows for paging, and the triangle for run. Several of
// those ARE `Extended_Pictographic`, so a bare property test would ban exactly
// the glyphs the system asks for — hence the allowlist rather than a stricter
// regex.
//
// Scope is the product surfaces: web/src and web/landing. Not design/ (vendored
// handoff bundles, not ours), not the docs (their own theme's problem).

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const webRoot = path.resolve(here, "..");

/** Sanctioned by the design system's ICONOGRAPHY section, plus the close mark. */
const ALLOWED = new Set([
  "☀", // theme toggle, light
  "☾", // theme toggle, dark
  "✓", // verdict: pass
  "✗", // verdict: fail
  "✕", // close / dismiss
  "ⓘ", // verdict: informational
  "⋯", // elided rows
  "←", // paging
  "→", // paging
  "▶", // run
  "↔", // round-trip, as in AGS4↔XLSX — the same textual arrow family as ← →
  "©", // Crown copyright attribution on the OSTN15 grid; legal text, not a glyph
  "︎", // text-presentation selector — the thing that keeps ☀ from being emoji
]);

// U+FE0F is the opposite selector: it forces EMOJI presentation on an otherwise
// textual glyph, which is the rule being dodged rather than followed.
const EMOJI_SELECTOR = "️";

const files = execFileSync(
  "git",
  ["ls-files", "src", "landing", "index.html"],
  { cwd: webRoot, encoding: "utf8" },
)
  .split("\n")
  .filter((f) => /\.(ts|tsx|js|mjs|css|html)$/.test(f))
  // Tests are not product UI, and an emoji in a fixture is usually the POINT:
  // agsline and fixpreview both pin how an astral-plane character survives the
  // tokenizer's byte/char walk. Banning those would delete the coverage.
  .filter((f) => !/\.test\.(ts|tsx)$/.test(f));

const pictographic = /\p{Extended_Pictographic}/u;
const violations = [];

for (const rel of files) {
  const text = readFileSync(path.join(webRoot, rel), "utf8");
  text.split("\n").forEach((line, i) => {
    for (const ch of line) {
      if (ALLOWED.has(ch)) continue;
      if (ch === EMOJI_SELECTOR || pictographic.test(ch)) {
        violations.push(
          `  ${rel}:${i + 1}  ${JSON.stringify(ch)} (U+${ch.codePointAt(0).toString(16).toUpperCase().padStart(4, "0")})`,
        );
      }
    }
  });
}

if (violations.length > 0) {
  console.error(
    `check-no-emoji: ${violations.length} emoji in product UI:\n` +
      violations.join("\n") +
      `\n\nThe design system bans emoji in product UI. If you need a glyph, use a ` +
      `vendored Lucide icon (src/shared/icons/icons.ts) — and if Lucide has no ` +
      `match, say so rather than hand-drawing or substituting one.`,
  );
  process.exit(1);
}

console.log(`check-no-emoji: ${files.length} file(s) clean`);
