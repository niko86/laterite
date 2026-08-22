#!/usr/bin/env python3
"""Type-check the docs' `js` programs against the shipped `.d.ts` (#565).

The page-program runner executes these programs, and "does not raise" cannot see
a page that runs the wrong branch: #518's example branched on `file.report.ok`
— a field `Report` does not have — so the expression was always `undefined`,
the else branch ran, and the runner passed a page demonstrating the opposite of
what its prose claimed. A type check refuses that expression. The defect class
this gate exists to catch is **a documented type that no longer exists**: a
renamed field, a changed signature, a return type that quietly became something
else.

**Resolution-only, not strict** (the decision on #565): no `noImplicitAny`, no
`strictNullChecks`. Full strict flags things that are not defects in an example
— implicit `any` on a placeholder, `possibly undefined` on a value the prose
just said is present. Failing when a name or signature does not resolve is the
whole ask; tightening further is a later argument with evidence.

Two corpora per leg, mirroring how the docs' code actually RUNS:

- **Assembled page programs** — pages with ≥1 inline `js` fence, concatenated
  include-and-continuations in document order via `gen_doc_outputs.page_program`
  — the SAME assembly the runner executes, shared rather than reimplemented so
  the two cannot drift. The runner's inline-only cut is mirrored too, because a
  pure-include page is not one program: its includes are INDEPENDENT files
  (concatenating `transport.md`'s two tabs redeclares every import).
- **The example files** (`examples/{node,wasm}/ex*.mjs`) — each its own module,
  exactly as `test_docs_examples.py` runs them. This is what covers the
  pure-include pages, `reference/wasm-api.md` among them.

Two legs, one per module specifier in the corpus, each against the SHIPPED type
surface (`package.json` `types`/`exports` — NOT the tracked-but-unshipped napi
`index.d.ts`):

- `laterite` → `rust-packages/laterite-node` (needs the tsup `dist/` built)
- `@laterite/ags4-wasm` → `web/src/wasm` (needs `wasm-pack build`; untracked)

A leg whose `.d.ts` is not built is REPORTED BY NAME with everything it would
have checked — never silently skipped — and naming it with `--leg` makes its
absence a failure, because CI asking for a leg it did not build is a broken
job, not a quiet day. Counts print on every run, zero included; zero items on
an available leg is itself a failure (an empty corpus means discovery broke).

Every leg run starts with a POSITIVE CONTROL — a program carrying exactly
#518's defect — that must go red before the real corpus is believed green, so
"the gate passed" can never mean "the gate checked nothing".

Known non-defects are suppressed through ALLOW below — each entry names its
diagnostic, carries its why, is printed on every run, and FAILS the run when it
stops matching anything (a suppression that outlives its diagnostic is a blind
spot with a green tick on it). `ts` fences are not checked: #519 classified the
corpus's one instance as a type-only non-program, and this gate does not
resurrect it.

    python tools/check_doc_types.py [--leg node|wasm] ...
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_doc_outputs as gdo

ROOT = gdo.ROOT
WASM_SPEC = "@laterite/ags4-wasm"


class Leg(NamedTuple):
    """One package the corpus imports: how pages spell it, where its shipped
    types live, which file's absence means "not built", and the symlink the
    runner resolves it through (`docs-vs-released-npm` re-points that link, so
    both gates answer about the same artifact)."""

    spec: str
    pkg: Path
    dts: str
    link: Path


LEGS: dict[str, Leg] = {
    "node": Leg("laterite", gdo.NODE_PKG, "dist/index.d.ts", gdo.NODE_LINK),
    "wasm": Leg(WASM_SPEC, gdo.WASM_PKG, "ags4_wasm.d.ts", gdo.WASM_LINK),
}


class Allow(NamedTuple):
    """One suppressed diagnostic: which leg it lives on, the item-key pattern,
    the TS code, and WHY it is a checker limit rather than a page defect."""

    leg: str
    pattern: str
    code: str
    why: str


#: Diagnostics that are CHECKER limits, not page defects. The corpus is written
#: for readers; the #565 decision forbids rewriting it to satisfy the checker,
#: so what the checker cannot infer is recorded here instead, visibly. An entry
#: whose leg WAS checked and that matched nothing FAILS the run — a suppression
#: must never outlive its diagnostic — while an entry whose leg did not run
#: this invocation is reported as unexercised, not stale: the `--leg wasm`
#: CI lane must not fail over node-leg entries it cannot see.
ALLOW: list[Allow] = [
    Allow(
        "node",
        r"ex05_query|filter-select",
        "TS2365",
        "sql() rows are honestly Record<string, unknown>; the example's bare "
        "numeric compare IS the page's point, and annotating it would rewrite "
        "the corpus for the checker",
    ),
    Allow(
        "node",
        r"ex09a_build_from_frames|build-from-frames",
        "TS2769",
        "a heterogeneous Map literal — tsc cannot unify the per-group row "
        "shapes without an annotation the reader does not need; buildAgs4 "
        "accepts exactly this shape at runtime",
    ),
]

#: Vite's `?url` asset import — `reference/wasm-api.md` shows
#: `import wasmUrl from "@laterite/ags4-wasm/ags4_wasm_bg.wasm?url"` for
#: bundler users. Not a real module (a bundler rewrites it to a string URL), so
#: it is declared here EXPLICITLY rather than left to fail resolution or be
#: silently dropped: a page adopting it type-checks as `string`, which is what
#: a bundler hands back (#565).
_URL_AMBIENT = (
    'declare module "*?url" {\n  const url: string;\n  export default url;\n}\n'
)

#: #518's defect, verbatim in kind — plus the ONE diagnostic code it must
#: produce. Any-nonzero would also accept a broken tsconfig as "red", and a
#: control that passes for the wrong reason warrants nothing.
_CONTROLS: dict[str, tuple[str, str]] = {
    "node": (
        'import { validate } from "laterite";\n'
        'const report = validate("delivery.ags");\n'
        "report.ok;\n",  # Report has no `ok` — the exact #518 expression
        "TS2339",  # property does not exist on type
    ),
    "wasm": (
        f'import {{ thisExportDoesNotExist565 }} from "{WASM_SPEC}";\n'
        "thisExportDoesNotExist565;\n",
        # TS2614, not TS2305: the wasm package HAS a default export, so tsc
        # diagnoses a bad named import as "did you mean the default" — the
        # control run itself corrected this expectation on its first firing.
        "TS2614",
    ),
}

#: One tsc diagnostic head: `path(line,col): error TSnnnn: message`.
_DIAG_RE = re.compile(r"^(?P<file>\S+?)\(\d+,\d+\): error (?P<code>TS\d+): ")
#: A location-less diagnostic (`error TS5083: Cannot read file …` — config and
#: harness failures carry no `path(line,col):` head). Always REAL, never
#: allowlisted: it is about this gate's own setup, not a page.
_HEADLESS_RE = re.compile(r"^error TS\d+: ")


def leg_pkg(leg: str) -> Path:
    lg = LEGS[leg]
    return lg.link.resolve() if lg.link.exists() else lg.pkg


def leg_available(leg: str) -> bool:
    return (leg_pkg(leg) / LEGS[leg].dts).exists()


def collect() -> dict[str, dict[str, str]]:
    """Both corpora, keyed `page <rel>` / `example <leg>/<name>` → source."""
    legs: dict[str, dict[str, str]] = {"node": {}, "wasm": {}}
    for page in sorted(gdo.DOCS.rglob("*.md")):
        src, inline = gdo.page_program(
            page.read_text(encoding="utf-8"), "js", "javascript"
        )
        # The runner's cut, for the runner's reason: zero inline means the page
        # is only includes, and those are independent files, not one program —
        # they are covered per-file below.
        if not inline or not src.strip():
            continue
        rel = str(page.relative_to(gdo.DOCS))
        legs["wasm" if WASM_SPEC in src else "node"][f"page {rel}"] = src
    for leg in legs:
        for f in sorted((gdo.EXAMPLES / leg).glob("ex*.mjs")):
            legs[leg][f"example {leg}/{f.name}"] = f.read_text(encoding="utf-8")
    return legs


def tsc_bin() -> str:
    """The TypeScript this repo already carries; PATH only as a last resort."""
    for cand in (
        ROOT / "rust-packages" / "laterite-node" / "node_modules" / ".bin" / "tsc",
        ROOT / "web" / "node_modules" / ".bin" / "tsc",
    ):
        if cand.exists():
            return str(cand)
    return shutil.which("tsc") or "tsc"


def type_roots() -> Path | None:
    """An `@types` dir carrying `node`, for the `node:*` builtin imports."""
    for cand in (
        ROOT / "rust-packages" / "laterite-node" / "node_modules" / "@types",
        ROOT / "web" / "node_modules" / "@types",
    ):
        if (cand / "node").exists():
            return cand
    return None


def _tsconfig(files: list[str], roots: Path | None) -> dict:
    opts: dict[str, bool | str | list[str]] = {
        # allowJs+checkJs: the corpus is `js`; nodenext follows the
        # package's `exports`/`types` map — the SHIPPED surface.
        "allowJs": True,
        "checkJs": True,
        "noEmit": True,
        "module": "nodenext",
        "moduleResolution": "nodenext",
        "target": "es2022",
        # Resolution-only (#565's decision).
        "strict": False,
        "skipLibCheck": True,
    }
    if roots is not None:
        opts["types"] = ["node"]
        opts["typeRoots"] = [str(roots)]
    return {"compilerOptions": opts, "files": files}


def _slug(key: str) -> str:
    return key.split(" ", 1)[1].replace("/", "__").replace(".md", "") + ".mjs"


def run_leg(
    leg: str, corpus: dict[str, str], tsc: str, allow_hits: dict[int, int]
) -> int:
    """Control first, then the corpus; returns the failure count."""
    spec = LEGS[leg].spec
    roots = type_roots()
    if roots is None:
        # Builtins go untyped rather than unresolvable — the laterite surface,
        # the gate's actual subject, is still fully checked.
        print(f"leg {leg}: no @types/node found — node builtins typed as any")
    failures = 0
    with tempfile.TemporaryDirectory() as td:
        work = Path(td)
        link_dir = work / "node_modules" / Path(spec).parent
        link_dir.mkdir(parents=True, exist_ok=True)
        (link_dir / Path(spec).name).symlink_to(leg_pkg(leg), target_is_directory=True)
        ambient = _URL_AMBIENT + ('declare module "node:*";\n' if roots is None else "")
        (work / "ambient.d.ts").write_text(ambient, encoding="utf-8")

        def check(names: list[str]) -> subprocess.CompletedProcess:
            cfg = work / "tsconfig.json"
            cfg.write_text(
                json.dumps(_tsconfig([*names, "ambient.d.ts"], roots)),
                encoding="utf-8",
            )
            return subprocess.run(
                [tsc, "--noEmit", "-p", str(cfg)],
                cwd=work,
                capture_output=True,
                text=True,
                timeout=300,
            )

        control_src, control_code = _CONTROLS[leg]
        (work / "_control.mjs").write_text(control_src, encoding="utf-8")
        cr = check(["_control.mjs"])
        control_out = cr.stdout or cr.stderr
        # The EXPECTED code, not any nonzero exit: a broken tsconfig also
        # exits 2, and a control red for the wrong reason warrants nothing.
        if cr.returncode == 0 or control_code not in control_out:
            print(
                f"leg {leg}: POSITIVE CONTROL FAILED — expected {control_code} "
                f"for #518's defect, got exit {cr.returncode}:"
            )
            for line in control_out.strip().splitlines():
                print(f"  {line}")
            return 1
        print(f"leg {leg}: positive control red with {control_code}, as it must be")

        names: dict[str, str] = {}  # slug -> corpus key
        for key, src in corpus.items():
            slug = _slug(key)
            (work / slug).write_text(src, encoding="utf-8")
            names[slug] = key
        r = check(sorted(names))
        # Partition diagnostics: allowlisted (printed, counted) vs real (fail).
        real: list[str] = []
        classified = 0
        current_allowed = False
        for line in (r.stdout or r.stderr).splitlines():
            if _HEADLESS_RE.match(line):
                # A config/harness failure — about this gate, never a page,
                # so it can never be allowlisted away.
                classified += 1
                current_allowed = False
                real.append(line)
                continue
            m = _DIAG_RE.match(line)
            if m is None:
                # Continuation of the previous diagnostic; follows its verdict.
                if not current_allowed and real:
                    real.append(f"  {line}")
                continue
            classified += 1
            item_key = names.get(Path(m.group("file")).name, m.group("file"))
            current_allowed = False
            for i, entry in enumerate(ALLOW):
                if entry.code == m.group("code") and re.search(entry.pattern, item_key):
                    allow_hits[i] = allow_hits.get(i, 0) + 1
                    current_allowed = True
                    break
            if not current_allowed:
                real.append(f"{item_key}: {line.split(': ', 1)[1]}")
        if r.returncode != 0 and classified == 0:
            # tsc failed and this parser recognised NOTHING it printed — green
            # here would be a blind spot with a tick on it, so the raw output
            # becomes the failure instead.
            failures += 1
            print(f"[unclassified-tsc-failure] leg {leg} (exit {r.returncode}):")
            for line in (r.stdout or r.stderr).strip().splitlines():
                print(f"  {line}")
        elif real:
            failures += 1
            print(f"[type-error] leg {leg}:")
            for line in real:
                print(f"  {line}")
    return failures


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--leg",
        action="append",
        choices=sorted(LEGS),
        help="require this leg (repeatable); its package missing becomes a "
        "failure instead of a reported skip. Default: check what is built.",
    )
    args = ap.parse_args()

    corpora = collect()
    print(
        "scope: the docs' `js` page programs (assembled, inline-bearing pages) "
        "+ the example files, resolution-only, against each package's shipped "
        "types; `ts` fences are type-only non-programs (#519) and the python "
        "pages are mkdocstrings' (#565)"
    )
    tsc = tsc_bin()
    print(f"tsc: {tsc}")

    failures = 0
    allow_hits: dict[int, int] = {}
    legs_checked: set[str] = set()
    for leg in sorted(LEGS):
        corpus = corpora[leg]
        n_pages = sum(1 for k in corpus if k.startswith("page "))
        n_files = len(corpus) - n_pages
        if not leg_available(leg):
            named = ", ".join(sorted(corpus)) or "none found"
            print(
                f"leg {leg}: UNAVAILABLE — {leg_pkg(leg) / LEGS[leg][2]} is "
                f"not built; NOT checked: {named}"
            )
            if args.leg and leg in args.leg:
                print(f"leg {leg}: required by --leg but not built — failing")
                failures += 1
            continue
        if args.leg and leg not in args.leg:
            print(f"leg {leg}: built but not requested — skipped by --leg")
            continue
        print(
            f"leg {leg}: {n_pages} page program(s) + {n_files} example "
            "file(s) to type-check"
        )
        if not corpus:
            print(f"leg {leg}: zero items found — discovery is broken")
            failures += 1
            continue
        legs_checked.add(leg)
        failures += run_leg(leg, corpus, tsc, allow_hits)

    for i, entry in enumerate(ALLOW):
        n = allow_hits.get(i, 0)
        if n:
            print(f"allowed {n}× {entry.code} ({entry.pattern}): {entry.why}")
        elif entry.leg not in legs_checked:
            # Not stale — unjudged. The `--leg wasm` lane cannot see a
            # node-leg diagnostic, and failing it over one would make the CI
            # split red by construction.
            print(
                f"allow ({entry.pattern}, {entry.code}): not exercised — "
                f"leg {entry.leg} was not checked this run"
            )
        else:
            print(
                f"STALE ALLOW ({entry.pattern}, {entry.code}): leg {entry.leg} "
                "ran and it matched nothing — delete the entry or the "
                "diagnostic it excused has moved"
            )
            failures += 1

    if failures:
        sys.exit(f"check_doc_types: {failures} FAILURE(S)")
    print("check_doc_types: OK")


if __name__ == "__main__":
    main()
