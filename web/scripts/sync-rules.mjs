// Keep web/public/rules-catalogue.json in lock-step with the single source of
// truth, rust-packages/laterite-ags4-reference/data/rules_meta.json (the
// editorial rule metadata the validator embeds and exposes via --list-rules).
// The RuleExplainer tool fetches this static asset at runtime to render the
// plain-English rule reference — the SAME 27 rules the engine emits, so the
// web copy can't drift back to phantom rules (a no-op "12", a folded "16a") or
// stale severities/fixable flags. Runs as a `predev`/`prebuild` hook, and a
// vitest gate (src/lib/rulesCatalogue.test.ts) re-checks the copy in CI.

import { copyFileSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = path.resolve(
  here,
  "../../rust-packages/laterite-ags4-reference/data/rules_meta.json",
);
const dst = path.resolve(here, "../public/rules-catalogue.json");

// Fail loudly rather than ship a stale catalogue: stop the build on a missing
// or malformed source.
const raw = readFileSync(src, "utf8");
JSON.parse(raw); // throws on malformed source
copyFileSync(src, dst);
console.log(
  `[sync-rules] ${path.relative(process.cwd(), src)} → ${path.relative(process.cwd(), dst)}`,
);
