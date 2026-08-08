// The optional-peer range must admit a version that actually exists.
//
// `peerDependencies: { "@duckdb/node-api": ">=1.5.0" }` matched ZERO of the 56
// published versions of that package, and shipped that way in laterite@0.10.1.
// Every `@duckdb/node-api` release carries a prerelease suffix (`1.5.5-r.3`,
// `1.5.3-r.3`, …), and semver excludes a prerelease from a range unless some
// comparator shares its major.minor.patch AND carries a prerelease of its own.
// `>=1.5.0` has no prerelease component, so it matched nothing — and neither did
// `>=1.5.0-0`, nor even `*`.
//
// What that cost a reader, all three reproduced against the published package:
//
//   npm i laterite && npm i @duckdb/node-api   ETARGET — no matching version.
//                                              This is the EXACT command the
//                                              library's own error text prints.
//   npm i laterite @duckdb/node-api@1.5.3-r.3  peer silently dropped, exit 0.
//   npm i @duckdb/node-api@… && npm i laterite peer actively REMOVED.
//
// So `sql()` and `at()` — two documented features, with a helpful error pointing
// at an install command that could not work — were unreachable for anyone
// installing from npm. Three of the docs examples fail against the released
// package for this reason alone.
//
// WHY NO EXISTING GATE SAW IT. `npm ci` installs from the committed lockfile,
// which pins the devDependency directly; the peer range is never consulted. Every
// test, the docs-example gates included, ran against a tree where the peer was
// present for a reason unrelated to the range being correct.
//
// This asserts the one thing that would have caught it, offline and without the
// registry: THE RANGE WE PUBLISH MUST ADMIT THE VERSION WE DEVELOP AGAINST. If
// those two disagree, the range cannot be right — whatever else it may match.
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import semver from "semver";
import { expect, it } from "vitest";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));

function read(rel: string): { version?: string; [k: string]: unknown } {
  return JSON.parse(readFileSync(join(pkgDir, rel), "utf8"));
}

it("the optional-peer range admits the version we develop against", () => {
  const pkg = read("package.json") as {
    peerDependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
  };
  const range = pkg.peerDependencies?.["@duckdb/node-api"];
  if (!range) throw new Error("peerDependencies['@duckdb/node-api'] is gone");

  // The version npm actually resolved, not the range we asked for — that is what
  // every test in this package has been running against.
  const installed = read("node_modules/@duckdb/node-api/package.json").version;
  if (!installed) throw new Error("the peer is not installed; run `npm ci`");

  if (!semver.satisfies(installed, range)) {
    throw new Error(
      `peerDependencies declares "${range}", which does NOT admit ${installed} — ` +
        "the version this package is built and tested against. A consumer running " +
        "the documented `npm install @duckdb/node-api` gets ETARGET. Note that a " +
        "range without a prerelease component (`>=1.5.0`, or even `*`) matches NO " +
        "prerelease version, and every @duckdb/node-api release is a prerelease.",
    );
  }
  expect(semver.satisfies(installed, range)).toBe(true);
});

it("the dev range and the peer range agree about the same version", () => {
  // Belt and braces: if the devDependency drifts to a line the peer range does
  // not cover, we would again be testing something users cannot install — the
  // failure above, arrived at from the other side.
  const pkg = read("package.json") as {
    devDependencies?: Record<string, string>;
  };
  const devRange = pkg.devDependencies?.["@duckdb/node-api"];
  const installed = read("node_modules/@duckdb/node-api/package.json").version;
  if (!devRange || !installed)
    throw new Error("dev range or peer install missing");
  if (!semver.satisfies(installed, devRange)) {
    throw new Error(
      `devDependencies declares "${devRange}" but ${installed} is installed`,
    );
  }
  expect(semver.satisfies(installed, devRange)).toBe(true);
});
