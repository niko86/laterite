// Fail when `npm run sync-assets` would change a committed file (#1106).
//
// Every sync-assets output (the dictionary, the samples, the sensitive-heading
// list, the rules catalogue, the vendored icons) is a committed copy of a
// source elsewhere, refreshed by the predev/prebuild hook. Nothing compared the
// copies with what was committed, so the icons sat on an older lucide-static
// header than the lockfile pins and every local build rewrote all of them. The
// next drift would be just as quiet: a build fixes it on one laptop and nobody
// commits it.
//
// The check runs the real `sync-assets` script, not a list of its outputs, so
// a sync step added to that script is covered without editing this file. For
// the same reason it reads the WHOLE working tree afterwards rather than
// a list of output paths: there is no list to drift, and no path it skips.
//
// That only works on a clean tree, so a dirty one is refused, not judged: an
// uncommitted edit would read as drift. To prove the gate goes red, commit
// the hand edit first.

import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const webRoot = path.resolve(here, "..");

/** Every path git reports as changed or untracked, relative to the repo root. */
function changedPaths() {
  return execFileSync(
    "git",
    ["status", "--porcelain=v1", "--untracked-files=all"],
    { cwd: webRoot, encoding: "utf8" },
  )
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => `  ${line}`);
}

const before = changedPaths();
if (before.length > 0) {
  console.error(
    `check-sync-assets: the working tree is not clean (${before.length} path(s)), ` +
      `so drift cannot be told apart from your edits. Commit or set them aside first:\n` +
      before.join("\n"),
  );
  process.exit(2);
}

execFileSync("npm", ["run", "sync-assets"], { cwd: webRoot, stdio: "inherit" });

const drifted = changedPaths();
if (drifted.length > 0) {
  console.error(
    `check-sync-assets: sync-assets changed ${drifted.length} committed path(s):\n` +
      drifted.join("\n") +
      `\n\nRun \`npm run sync-assets\` in web/ and commit the result. If you edited ` +
      `one of these files by hand, edit its source instead; the sync overwrites the copy.`,
  );
  process.exit(1);
}

console.log("check-sync-assets: sync-assets left the working tree unchanged");
