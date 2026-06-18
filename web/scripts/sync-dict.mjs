// Keep web/public/ags5_dictionary.json in lock-step with the single source of
// truth, rust-packages/laterite-ags4-core/data/ags5_dictionary.json. The web copy is a
// static asset Vite serves verbatim (the Dictionary tool fetches it at runtime),
// and it used to be a HAND-maintained byte copy with no sync — so a dictionary
// edit could land in the engine but not the site (or vice-versa). This runs as
// the `predev`/`prebuild` hook so dev + every deploy rebuild copy it fresh.

import { copyFileSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const src = path.resolve(here, "../../rust-packages/laterite-ags4-core/data/ags5_dictionary.json");
const dst = path.resolve(here, "../public/ags5_dictionary.json");

// Fail loudly rather than ship a stale dictionary: if the source can't be read
// or isn't valid JSON, stop the build.
const raw = readFileSync(src, "utf8");
JSON.parse(raw); // throws on malformed source
copyFileSync(src, dst);
console.log(`[sync-dict] ${path.relative(process.cwd(), src)} → ${path.relative(process.cwd(), dst)}`);
