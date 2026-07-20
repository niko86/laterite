// Keep web/public/ags_dictionary.json in lock-step with the single source of
// truth, rust-packages/laterite-ags4-reference/data/ags_dictionary.json (the
// canonical multi-edition UNION, itself generated from the official AGS .ags
// dictionaries by tools/gen_dictionary.py). The web copy is a static asset Vite
// serves verbatim (every dict consumer fetches it at runtime — see
// src/lib/dict.ts). This runs as the `predev`/`prebuild` hook so dev + every
// deploy rebuild copy it fresh; a stale hand-copy can't drift in.

import { copyFileSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = path.resolve(
  here,
  "../../rust-packages/laterite-ags4-reference/data/ags_dictionary.json",
);
const dst = path.resolve(here, "../public/ags_dictionary.json");

// Fail loudly rather than ship a stale dictionary: if the source can't be read
// or isn't valid JSON, stop the build.
const raw = readFileSync(src, "utf8");
JSON.parse(raw); // throws on malformed source
copyFileSync(src, dst);
console.log(
  `[sync-dict] ${path.relative(process.cwd(), src)} → ${path.relative(process.cwd(), dst)}`,
);
