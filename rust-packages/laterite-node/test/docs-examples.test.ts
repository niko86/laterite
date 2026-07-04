// Runnable-guarantee gate for the docs site's Node snippets (#373) — the Node
// twin of tests/test_docs_examples.py. Executes every
// web/docs-site/examples/node/*.mjs as a real `node` subprocess from the repo
// root (so "examples/sample_site.ags" relative paths resolve). Each example
// ends in node:assert assertions, so a changed return shape / property name /
// printed format turns a doc snippet red HERE. The doc pages `--8<--`-include
// these exact files: page and test are the same bytes.
//
// Module resolution: the examples contain the literal user text
// `import { … } from "laterite"`. A node_modules/laterite symlink beside them
// (created idempotently below, gitignored) points at this package, so ESM
// resolution walks up from the example file, hits the symlink, and follows the
// real package.json `exports` to dist/index.mjs — which then executes at its
// TRUE path (Node realpaths by default), so apache-arrow and the `#native`
// imports-map resolve from this package exactly as in a published install.
// (Self-reference can't work — the examples live outside the package dir — and
// NODE_PATH is ignored by ESM; this is the tools/pack-smoke.mjs trick.)
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, symlinkSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, expect, it } from "vitest";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const repoRoot = resolve(pkgDir, "../..");
const exampleDir = join(repoRoot, "web", "docs-site", "examples", "node");

beforeAll(() => {
  // The examples import the built package surface, not ../ts — same reason
  // pack-smoke exists: exercise what an installed `laterite` actually resolves.
  if (!existsSync(join(pkgDir, "dist", "index.mjs"))) {
    throw new Error("dist/index.mjs missing — run `npm run build:debug` first");
  }
  mkdirSync(join(exampleDir, "node_modules"), { recursive: true });
  const link = join(exampleDir, "node_modules", "laterite");
  if (!existsSync(link)) symlinkSync(pkgDir, link, "dir");
});

const examples = readdirSync(exampleDir).filter((f) => f.endsWith(".mjs"));

it("example library is non-empty", () => {
  // Guard against a glob that silently matches nothing (a moved dir would make
  // every example "pass" by not running) — mirrors test_docs_examples.py.
  expect(examples.length).toBeGreaterThan(0);
});

it.each(examples)(
  "%s runs",
  (name) => {
    // Throws (failing the test, with captured output) on any non-zero exit.
    execFileSync(process.execPath, [join(exampleDir, name)], {
      cwd: repoRoot,
      encoding: "utf8",
    });
  },
  30_000, // fresh node + debug-addon load per example
);
