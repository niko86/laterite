// Runnable-guarantee gate for the docs' LITERAL JavaScript, which nothing runs.
//
// `docs-examples.test.ts` beside this file, and `gen_doc_outputs --surface node`,
// both key off the `--8<--` include: they cover the 18 fences a page includes
// from `web/docs-site/examples/node/`. Eleven more are hand-typed straight into
// the Markdown, and the npm landing page (`../README.md`) carries four — the page
// a reader sees *before* `npm install`. None of that was executed by anything.
//
// The Python twin of this gate (`tests/test_docs_snippets.py`) found, on the day
// it landed, a parameter that had never existed in either library. The Node
// static gate (`tests/test_docs_node_api.py`) found `report.ok`, which is
// `undefined`, in an `if (!file.report.ok)` that made a repair branch run on
// clean files. Both were hand-typed prose beside correct, executed examples.
//
// WHY VITEST AND NOT pytest. Executing these needs `dist/` and the gitignored
// `node_modules/laterite` symlink that makes a literal `import … from "laterite"`
// resolve through the real package `exports` — ESM ignores NODE_PATH, and
// self-reference cannot work outside the package directory. Only the `node` CI
// job builds `dist/`. A pytest version would skip in CI, which is what
// `test_docs_node_api.py` explicitly refused to accept ("the same as not having
// it"). That gate stays, and the division is not cosmetic: THIS ONE CATCHES
// CALLS, THAT ONE CATCHES READS. A bare `report.someTypo` evaluates to
// `undefined` and logs — it does not throw — so no amount of executing finds it;
// `buildAgs4({…})` with the wrong shape throws and no amount of name-resolution
// finds that. Falsification proved the split rather than assuming it, and
// exposed a hole in the process: the static gate scanned `docs/node/` only, so
// the npm landing page was covered by neither. Its scope now includes it.
//
// A PAGE IS ONE PROGRAM, in document order, with `--8<--` includes replayed —
// because that is how a reader reads it, and because Node's fragments are chained
// handles (`read(…).validate(…)`) that only mean anything with their predecessor
// in scope. Concatenating ESM has one wrinkle Python did not: a binding may not be
// declared twice, and pages repeat both imports and handles across fences. `weld`
// reconciles that — imports unioned per module, re-declarations turned into
// assignment — without smoothing away a genuine conflict.
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, expect, it } from "vitest";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const repoRoot = resolve(pkgDir, "../..");
const docsDir = join(repoRoot, "web", "docs-site", "docs");
const exampleDir = join(repoRoot, "web", "docs-site", "examples");
const fixture = join(repoRoot, "examples", "sample_site.ags");

/** An indented-or-not js/ts fence. Material's tabbed content indents its blocks. */
const FENCE = /^([ \t]*)```(js|ts)\n([\s\S]*?)^\1```$/gm;
const INCLUDE = /--8<-- "([^"]+)"/;
/** Same shape as `gen_doc_outputs`'s `doc-output: skip`, reason equally required. */
const SKIP = /<!-- doc-snippet: skip\s*[—-]\s*[^\n]*?-->/g;

/** Placeholder paths -> the copied fixture, so a snippet that only lacked a file runs. */
const PATHS: Record<string, string> = {
  "delivery.ags": "examples/sample_site.ags",
  "phase1.ags": "examples/sample_site.ags",
  "phase2.ags": "examples/sample_site.ags",
};

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory()
      ? walk(join(dir, e.name))
      : e.name.endsWith(".md")
        ? [join(dir, e.name)]
        : [],
  );
}

/** Fences in document order; an include contributes its example's source. */
function fences(md: string): { kind: "include" | "literal"; src: string }[] {
  const text = readFileSync(md, "utf8");
  const out: { kind: "include" | "literal"; src: string }[] = [];
  for (const m of text.matchAll(FENCE)) {
    const indent = m[1] ?? "";
    const raw = m[3];
    if (raw === undefined) continue;
    const body = indent ? raw.replace(new RegExp(`^${indent}`, "gm"), "") : raw;
    const inc = INCLUDE.exec(body);
    if (inc) {
      const target = inc[1]?.split(":")[0];
      if (target === undefined) continue;
      // NODE examples only. `reference/wasm-api.md` includes from `wasm/`, and
      // replaying those here welded two wasm examples that each declare `init`
      // into one program — a SyntaxError from a surface this gate does not own.
      // The wasm tree has its own leg (`gen_doc_outputs --surface wasm`).
      if (!target.startsWith("node/")) continue;
      const f = join(exampleDir, target);
      if (existsSync(f))
        out.push({ kind: "include", src: readFileSync(f, "utf8") });
      continue;
    }
    // A skip marker in the 300 chars before the fence exempts it. Used for the
    // one `import type { … }` block: pure type syntax, not executable, and about
    // the wasm surface rather than this one.
    const before = text.slice(Math.max(0, m.index - 300), m.index);
    if (SKIP.test(before)) {
      SKIP.lastIndex = 0;
      continue;
    }
    SKIP.lastIndex = 0;
    out.push({ kind: "literal", src: body });
  }
  return out;
}

const IMPORT_NAMED = /^\s*import\s*\{([^}]*)\}\s*from\s*["']([^"']+)["'];?\s*$/;
const IMPORT_DEFAULT = /^\s*import\s+(\w+)\s+from\s*["']([^"']+)["'];?\s*$/;
/** `const foo = …` / `let foo = …` at top level. Destructuring is excluded on
 *  purpose — rewriting `const { a, b } = …` is more machinery than the docs need. */
const DECL = /^(const|let|var)\s+(\w+)(\s*=.*)$/;

/** Join fences into one ESM program that Node will accept.
 *
 * Pages repeat imports and handles across fences. In Python that is a rebind; in
 * ESM it is a SyntaxError, so the page would fail for a reason that says nothing
 * about the docs. Two collapses, and no more than two:
 *
 *   * imports are UNIONED PER MODULE, not deduped by line. `import { buildAgs4 }`
 *     and `import { PROJ, LOCA, buildAgs4 }` are different lines that share a
 *     binding, so line-equality is not enough — the npm README has exactly that
 *     pair and it was the first thing this gate hit.
 *   * a RE-declaration becomes an assignment, so `const file = …` appearing in two
 *     fences reads as the page means it — the handle rebuilt, later fences seeing
 *     the newer value. Dropping the second instead would silently run fence 2
 *     against fence 1's handle, which is the wrong answer arrived at quietly. */
function weld(parts: string[]): string {
  const named = new Map<string, Set<string>>();
  const defaults = new Map<string, string>();
  const declared = new Set<string>();
  const body: string[] = [];

  for (const raw of parts.join("\n").split("\n")) {
    const asNamed = IMPORT_NAMED.exec(raw);
    if (asNamed) {
      const [, specifiers = "", mod = ""] = asNamed;
      const names = named.get(mod) ?? new Set<string>();
      for (const n of specifiers
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean)) {
        names.add(n);
      }
      named.set(mod, names);
      continue;
    }
    const asDefault = IMPORT_DEFAULT.exec(raw);
    if (asDefault?.[1] && asDefault[2]) {
      defaults.set(asDefault[1], asDefault[2]);
      continue;
    }
    // A page that declares `file` twice is not a mistake — it is telling a story
    // where the handle is rebuilt a different way. Python allows the rebind and
    // this model was borrowed from there; ESM makes it a SyntaxError. So the
    // FIRST declaration becomes `let` and later ones become plain assignment,
    // which preserves exactly what the page means: later fences see the newer
    // value. Dropping the duplicate instead would silently run fence 2 against
    // fence 1's handle.
    const decl = DECL.exec(raw);
    if (decl?.[2] && decl[3] !== undefined) {
      const [, , name, rest] = decl;
      body.push(declared.has(name) ? `${name}${rest}` : `let ${name}${rest}`);
      declared.add(name);
      continue;
    }
    body.push(raw);
  }

  const head = [
    ...[...defaults].map(([name, mod]) => `import ${name} from "${mod}";`),
    ...[...named].map(
      ([mod, names]) => `import { ${[...names].join(", ")} } from "${mod}";`,
    ),
  ];
  return [...head, ...body].join("\n");
}

/** A temp cwd that can resolve BOTH `import "laterite"` and `examples/…ags`.
 *
 * The program file lives here with its own `node_modules/laterite` symlink, so
 * ESM resolution walks up and finds the real package; the fixture is copied under
 * `examples/` so the page's repo-relative path text is the text you would type.
 * Everything the snippets WRITE lands here too — the Python twin minted
 * `checked.ags` and friends into the working tree before it was run this way. */
function workdir(): string {
  const dir = mkdtempSync(join(tmpdir(), "docs-snippets-"));
  mkdirSync(join(dir, "node_modules"), { recursive: true });
  symlinkSync(pkgDir, join(dir, "node_modules", "laterite"), "dir");
  mkdirSync(join(dir, "examples"), { recursive: true });
  copyFileSync(fixture, join(dir, "examples", "sample_site.ags"));
  return dir;
}

const pages = [...walk(docsDir), join(pkgDir, "README.md")]
  .map((md) => ({ md, parts: fences(md) }))
  .filter((p) => p.parts.some((f) => f.kind === "literal"));

let dir: string;
beforeAll(() => {
  if (!existsSync(join(pkgDir, "dist", "index.mjs"))) {
    throw new Error("dist/index.mjs missing — run `npm run build:debug` first");
  }
  if (!existsSync(fixture)) {
    throw new Error(`the shared fixture is gone: ${fixture}`);
  }
  dir = workdir();
});

it("pages with literal fences are discovered", () => {
  // Zero is a bad witness: an empty list would make every case below vacuous.
  expect(pages.length).toBeGreaterThanOrEqual(4);
});

const ran: string[] = [];
const stuck: string[] = [];

it.each(pages.map((p) => [p.md.slice(repoRoot.length + 1), p] as const))(
  "%s — its documented JavaScript runs",
  (name, page) => {
    let program = weld(page.parts.map((f) => f.src));
    for (const [from, to] of Object.entries(PATHS)) {
      program = program.split(from).join(to);
    }
    const file = join(dir, `${name.replace(/[^\w]/g, "_")}.mjs`);
    writeFileSync(file, program);
    try {
      const stdout = execFileSync(process.execPath, [file], {
        cwd: dir,
        encoding: "utf8",
      });
      expect(typeof stdout).toBe("string");
      ran.push(name);
    } catch (e) {
      const err = String((e as { stderr?: string }).stderr ?? e);
      // A fragment missing its setup is a documentation shape, not a defect —
      // the same classification the Python twin makes. Anything naming a member
      // that does not exist is the defect this gate is for.
      const contextOnly =
        /ENOENT|is not defined|Cannot read properties of undefined \(reading '\w+'\) *$/.test(
          err,
        );
      if (!contextOnly) {
        throw new Error(`${name}: documented JavaScript failed\n${err}`, {
          cause: e,
        });
      }
      stuck.push(name);
    }
  },
  120_000,
);

it("every page actually executed — none fell back", () => {
  // The Python twin needed a percentage floor because its pages are full of
  // genuine statement-level fragments. Here every page runs, so the honest
  // ratchet is stricter than a floor: NONE may quietly stop running. A page that
  // legitimately cannot (the wasm type declarations, the arrow fence that
  // downloads a DuckDB extension) carries a `doc-snippet: skip — reason` marker
  // and never reaches this list — an escape hatch you have to justify in writing,
  // which is the point.
  if (stuck.length > 0) {
    throw new Error(
      `these pages stopped executing and fell back to nothing:\n  ${stuck.join("\n  ")}\n\n` +
        "Give the fences their setup, or mark the block `doc-snippet: skip — <reason>`.",
    );
  }
  expect(ran.length).toBeGreaterThanOrEqual(4);
});
