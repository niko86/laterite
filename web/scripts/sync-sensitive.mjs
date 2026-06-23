// Keep web/public/sensitive_headings.json in lock-step with the single source
// of truth, rust-packages/laterite-ags4-core/data/sensitive_headings.json
// (generated from ags_dictionary.json by tools/gen_sensitive_headings.py). The
// web Anonymiser fetches this static asset at runtime to decide which columns
// to pre-select for redaction — the SAME list the corpus `censor` tool uses, so
// the two anonymisers can't drift. Runs as a `predev`/`prebuild` hook.

import { copyFileSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = path.resolve(
  here,
  "../../rust-packages/laterite-ags4-core/data/sensitive_headings.json",
);
const dst = path.resolve(here, "../public/sensitive_headings.json");

// Fail loudly rather than ship a stale list: stop the build on a missing or
// malformed source.
const raw = readFileSync(src, "utf8");
JSON.parse(raw); // throws on malformed source
copyFileSync(src, dst);
console.log(
  `[sync-sensitive] ${path.relative(process.cwd(), src)} → ${path.relative(process.cwd(), dst)}`,
);
