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
// the same reason it reads the working tree afterwards through `git status`
// rather than a list of output paths: there is no list to drift.
//
// That reading has a scope, and it is not the whole tree (#1130). `git status`
// sees every tracked path and every untracked one, but never an IGNORED one, so
// a sync output written under an ignored path is not compared. Half of that is
// cheap to close: the ignored entries are listed before and after the sync, and
// one that APPEARS fails the check. The other half is not: rewriting an ignored
// path that already existed, or writing inside an ignored directory that
// already existed, changes no listing. Every run prints both numbers, pass or
// fail, so that blind spot is never behind a bare green line.
//
// That only works on a clean tree, so a dirty one is refused, not judged: an
// uncommitted edit would read as drift. To prove the gate goes red, commit
// the hand edit first.

import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const webRoot = path.resolve(here, "..");

function git(args) {
  return execFileSync("git", args, { cwd: webRoot, encoding: "utf8" });
}

/** Every path git reports as changed or untracked, relative to the repo root. */
function changedPaths() {
  return git(["status", "--porcelain=v1", "--untracked-files=all"])
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => `  ${line}`);
}

/**
 * The ignored entries, an ignored directory collapsed to one entry. Kept apart
 * from `changedPaths` because `--untracked-files=all` would expand every
 * ignored directory, node_modules included, into its files.
 */
function ignoredEntries() {
  return new Set(
    git([
      "status",
      "--porcelain=v1",
      "--ignored=traditional",
      "--untracked-files=normal",
    ])
      .split("\n")
      .filter((line) => line.startsWith("!! "))
      .map((line) => line.slice(3)),
  );
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

const top = git(["rev-parse", "--show-toplevel"]).trim();
const tracked = execFileSync("git", ["ls-files", "-z"], {
  cwd: top,
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
})
  .split("\0")
  .filter((p) => p.length > 0).length;
const ignoredBefore = ignoredEntries();

execFileSync("npm", ["run", "sync-assets"], { cwd: webRoot, stdio: "inherit" });

const drifted = changedPaths();
const appeared = [...ignoredEntries()].filter((p) => !ignoredBefore.has(p));

const scope =
  `  compared: ${tracked} tracked path(s), and any untracked path the sync created\n` +
  `  not compared: ${ignoredBefore.size} ignored entr${ignoredBefore.size === 1 ? "y" : "ies"} ` +
  `that existed before the sync (git status cannot see a rewrite under one); ` +
  `${appeared.length} new ignored entr${appeared.length === 1 ? "y" : "ies"} appeared`;

if (drifted.length > 0 || appeared.length > 0) {
  const parts = [];
  if (drifted.length > 0) {
    parts.push(
      `sync-assets changed ${drifted.length} committed path(s):\n` +
        drifted.join("\n") +
        `\n\nRun \`npm run sync-assets\` in web/ and commit the result. If you edited ` +
        `one of these files by hand, edit its source instead; the sync overwrites the copy.`,
    );
  }
  if (appeared.length > 0) {
    parts.push(
      `sync-assets wrote ${appeared.length} path(s) git ignores, so this check cannot ` +
        `compare them with anything committed:\n` +
        appeared.map((p) => `  !! ${p}`).join("\n") +
        `\n\nA sync output belongs in a tracked path. Commit it, or stop ignoring it.`,
    );
  }
  console.error(`check-sync-assets: ${parts.join("\n\n")}\n${scope}`);
  process.exit(1);
}

console.log(
  `check-sync-assets: OK — sync-assets left the working tree unchanged\n${scope}`,
);
