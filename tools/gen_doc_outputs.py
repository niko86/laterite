#!/usr/bin/env python3
"""Render the docs site's example-output blocks from what the examples actually print (#164).

THE DEFECT. A docs page single-sources its code by reference and then hand-writes
what that code prints:

    ```python
    --8<-- "python/ex09a_build_from_frames.py"
    ```

    ```text
    groups: ['PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE']
    findings: 0
    ```

The example cannot drift — it is included, not copied. The output block is prose,
and nothing compared it to stdout. On 2026-07-24 metadata synthesis went opt-in;
the examples were updated and assert the new behaviour, six output blocks across
three pages were not, and the pages went on claiming a result the code no longer
produced. It surfaced only because a downstream consumer built against the page
and reported the contradiction.

THE FIX, AND WHY IT IS NOT "REGENERATE THE PROSE". The obvious repair is to splice
fresh stdout into the `text` fence and gate it. That leaves a COPY in the Markdown,
correct only for as long as the gate keeps re-running — and 51 blocks come from ~32
distinct examples, so several examples are documented on three pages and every copy
has to be kept in step.

The CLI examples in this same tree already solved it properly: the output lives in
a `.out` file beside the example and the page `--8<--`-includes THAT. MkDocs
resolves it at build time, so there is no copy to drift — one artifact per example,
included wherever it is documented. This generator brings python and node onto that
footing and keeps every surface's `.out` honest:

    gen_doc_outputs.py                 run the examples, write .out, point blocks at them
    gen_doc_outputs.py --check         CI gate: re-run and byte-compare (writes nothing)
    gen_doc_outputs.py --surface cli   one surface only

WHAT --check ACTUALLY ASSERTS. Three things, because a gate that only checked the
first would be satisfied by a page that quietly stopped participating:

  1. every committed `.out` matches what the example prints NOW;
  2. every example-output block in the docs is either an include or a declared
     opt-out — a block that reverts to hand-written prose FAILS;
  3. an opt-out carries a REASON. `<!-- doc-output: skip -->` with nothing after it
     is rejected: the point of an escape hatch is that using it is on the record.

Non-determinism is the honest reason to opt out (a timestamp, a duration, an
absolute path). It is also the reason NOT to silently exclude: a block excluded
without a stated reason is indistinguishable from one nobody has looked at.

BLOCKS WITH NO EXAMPLE are counted and reported, never touched. They are prose
illustrations — `pip install` chatter and the like — with no source to run. The
count is printed so a hand-written block appearing where an example belongs is
visible rather than absorbed.

Run: `uv run --no-sync python tools/gen_doc_outputs.py` (stdlib only). Needs the
surfaces it is asked for to be built: `lat` on PATH or in the workspace target dir
for cli, `laterite-node/dist` for node, the installed wheel for python.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "web" / "docs-site" / "docs"
EXAMPLES = ROOT / "web" / "docs-site" / "examples"

#: Where the built wasm package lands, and where Node must see it to resolve the
#: published name. wasm-pack names the package after the CRATE
#: (`laterite-ags4-wasm`); the publish step renames it. Node resolves a symlink by
#: PATH, not by the manifest's `name`, so this works against either.
WASM_PKG = ROOT / "web" / "src" / "wasm"
WASM_LINK = EXAMPLES / "wasm" / "node_modules" / "@laterite" / "ags4-wasm"

#: A `--8<--` include fence followed by the `text` fence documenting its output.
#: Both fences carry the tab-block indent (`=== "Python"`), which is captured so
#: the rewrite puts it back — Material's tabbed content is indentation-sensitive.
BLOCK_RE = re.compile(
    r"^(?P<indent>[ \t]*)```(?P<lang>\w+)\n"
    r"(?P=indent)--8<-- \"(?P<include>[^\"]+)\"\n"
    r"(?P=indent)```\n"
    r"(?P<gap>(?:[ \t]*\n)*)"
    r"(?P<skip>(?:(?P=indent)<!-- doc-output:[^\n]*-->\n(?:[ \t]*\n)*)?)"
    r"(?P=indent)```text\n"
    r"(?P<body>(?:.*?\n)*?)"
    r"(?P=indent)```\n",
    re.M,
)
SKIP_RE = re.compile(r"<!-- doc-output: skip(?:\s*[—-]\s*(?P<reason>[^\n]*?))?\s*-->")
#: Any `text` fence, so blocks with no example can be counted rather than assumed away.
TEXT_FENCE_RE = re.compile(r"^[ \t]*```text\n", re.M)
#: `# expect-exit: N` in a CLI example — several demonstrate a *failing* run.
EXPECT_EXIT_RE = re.compile(r"^#\s*expect-exit:\s*(\d+)\s*$", re.M)


@dataclass
class Surface:
    """One example tree: how to find its examples and how to run one."""

    name: str
    pattern: str
    #: argv for an example, given its path. `lat` resolves at call time.
    argv: Callable[[Path], list[str]]
    #: Included in a default run. duckdb is not: its examples need the built
    #: DuckDB extension, which the dev satellite gates monthly, off the PR path.
    #: cadence: compliance-report
    default: bool = True
    requires: str = ""
    env: dict[str, str] = field(default_factory=dict)


def _lat() -> str:
    """Which `lat` the CLI examples run — deliberately NOT "whatever is on PATH".

    There are two programs called `lat`. The docs describe the shipped Rust binary
    (`laterite-cli`); the `laterite` wheel ALSO installs a Python `lat` console
    script (`laterite._cli`), so any venv with the wheel in it has one on PATH.
    They render differently — the Rust one draws comfy-table UTF-8 grids, the
    Python one plain ASCII — so a PATH-first lookup silently rewrote every
    committed `.out` the first time this ran under `uv run`, from a *different
    program* than the one the pages document.

    So: `LAT_BIN` if set (CI pins it), else this checkout's release build, else
    PATH. The resolved path is printed, because a gate whose subject depends on
    the caller's environment must at least say what it measured.
    """
    if env := os.environ.get("LAT_BIN"):
        return env
    built = ROOT / "rust-packages" / "target" / "release" / "lat"
    return str(built) if built.exists() else (shutil.which("lat") or "lat")


SURFACES = {
    s.name: s
    for s in (
        Surface("python", "ex*.py", lambda f: [sys.executable, str(f)]),
        Surface("node", "ex*.mjs", lambda f: ["node", str(f)]),
        # bash, not sh: the examples use bash-isms, and `lat` is put on PATH via
        # env rather than hard-coded so the committed .out stays machine-neutral.
        Surface("cli", "*.sh", lambda f: ["bash", str(f)]),
        # The browser package, run headless. The examples import it by its
        # PUBLISHED name (`@laterite/ags4-wasm`) so they read exactly as a
        # consumer writes them; `_link_wasm` puts a symlink where Node's resolver
        # will find it. Off by default: it needs `wasm-pack build`, which the e2e
        # job does and a plain `uv run` does not.
        Surface("wasm", "ex*.mjs", lambda f: ["node", str(f)], default=False),
        Surface(
            "duckdb",
            "ex*.sql",
            lambda f: [
                "duckdb",
                "-init",
                str(EXAMPLES / "duckdb" / "_install.sql"),
                "-c",
                f".read {f}",
            ],
            default=False,
            requires="duckdb",
        ),
    )
}


def _link_wasm() -> None:
    """Put the built wasm package where Node resolves `@laterite/ags4-wasm`.

    The examples import the published name rather than a relative path, because a
    reader copying one should get working code, not a path into this repo. The
    same trick `docs-examples.test.ts` uses for the node examples — idempotent,
    gitignored, and pointing at whatever `wasm-pack build` last produced.
    """
    if not (WASM_PKG / "ags4_wasm.js").exists():
        sys.exit(
            f"gen_doc_outputs: surface wasm needs a built package at "
            f"{WASM_PKG.relative_to(ROOT)} — run `npm run build:wasm` in web/ first"
        )
    WASM_LINK.parent.mkdir(parents=True, exist_ok=True)
    if WASM_LINK.is_symlink() or WASM_LINK.exists():
        WASM_LINK.unlink()
    WASM_LINK.symlink_to(WASM_PKG, target_is_directory=True)


def run_example(surface: Surface, path: Path) -> str:
    """Stdout of one example, from the repo root.

    Every tree's examples name fixtures as `examples/sample_site.ags` — repo-root
    relative — so cwd is the repo root for all of them, matching the runtime gates
    (`tests/test_docs_examples.py`, `laterite-node/test/docs-examples.test.ts`).
    """
    env = dict(os.environ, **surface.env)
    if surface.name == "cli":
        env["PATH"] = str(Path(_lat()).parent) + os.pathsep + env["PATH"]
    proc = subprocess.run(
        surface.argv(path),
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=300,
        env=env,
    )
    expected = EXPECT_EXIT_RE.search(path.read_text(encoding="utf-8", errors="replace"))
    want = int(expected.group(1)) if expected else 0
    if proc.returncode != want:
        sys.exit(
            f"gen_doc_outputs: {surface.name}/{path.name} exited {proc.returncode}, expected {want}.\n"
            f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
        )
    return proc.stdout


def out_path(example: Path) -> Path:
    return example.with_suffix(".out")


def include_ref(example: Path) -> str:
    """The `--8<--` target for an example's output, as a page must write it."""
    return f"{example.parent.name}/{out_path(example).name}"


def example_key(include: str) -> str:
    """An include target reduced to the example it names, section suffix dropped.

    A page may include a SECTION of an example (`file.sh:cmd`, `file.py:code`)
    rather than the whole file, so the machinery lives in the file and only the
    lesson reaches the page. This was previously handled by registering the `:cmd`
    spelling literally alongside the bare one — which silently stopped working the
    moment a second section name existed: an unrecognised spelling falls through
    the `not in wanted` branch below as "a surface this run didn't ask for", so
    adding `:code` to three pages dropped the matched-block count from 38 to 35
    and reported success. Splitting on `:` covers every section name there will
    ever be, and cannot quietly under-count.
    """
    return include.split(":", 1)[0]


def scan(md: str) -> list[re.Match[str]]:
    return list(BLOCK_RE.finditer(md))


def rewrite_page(md: str, wanted: dict[str, str]) -> str:
    """Point every example-output block at its `.out`, leaving opt-outs alone."""

    def sub(m: re.Match[str]) -> str:
        ref = wanted.get(example_key(m.group("include")))
        if ref is None or SKIP_RE.search(m.group("skip")):
            return m.group(0)
        i = m.group("indent")
        return (
            f'{i}```{m.group("lang")}\n{i}--8<-- "{m.group("include")}"\n{i}```\n'
            f'{m.group("gap")}{m.group("skip")}{i}```text\n{i}--8<-- "{ref}"\n{i}```\n'
        )

    return BLOCK_RE.sub(sub, md)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="fail on drift; write nothing")
    ap.add_argument(
        "--check-pages",
        action="store_true",
        help="the structural half only: no example is run, so nothing needs building",
    )
    ap.add_argument(
        "--surface",
        action="append",
        choices=sorted(SURFACES),
        help="limit to one surface (repeatable); default is every surface not needing extra tooling",
    )
    args = ap.parse_args()

    # `default` marks a surface whose examples need extra tooling to RUN. That is
    # irrelevant to --check-pages, which runs nothing — so the structural half
    # covers EVERY surface unless one is named. Scoping it to the runnable ones
    # would leave the wasm blocks unchecked in the only lane that can always fire.
    chosen = (
        [SURFACES[n] for n in args.surface]
        if args.surface
        else list(SURFACES.values())
        if args.check_pages
        else [s for s in SURFACES.values() if s.default]
    )
    # A surface's tooling is only needed to RUN its examples; --check-pages reads
    # Markdown and needs none of it.
    for s in chosen:
        if args.check_pages:
            continue
        if s.requires and not shutil.which(s.requires):
            sys.exit(f"gen_doc_outputs: surface {s.name} needs `{s.requires}` on PATH")
        if s.name == "wasm":
            _link_wasm()

    # example include-path -> its .out include-path, for every surface in play
    wanted: dict[str, str] = {}
    examples: list[tuple[Surface, Path]] = []
    for s in chosen:
        found = sorted((EXAMPLES / s.name).glob(s.pattern))
        if not found:
            sys.exit(
                f"gen_doc_outputs: no {s.pattern} under {(EXAMPLES / s.name).relative_to(ROOT)} — "
                "an empty glob would make this gate vacuous"
            )
        for f in found:
            examples.append((s, f))
            # Keyed on the bare example; `example_key` strips any section
            # suffix a page used. Only the output side is ever rewritten.
            wanted[f"{s.name}/{f.name}"] = include_ref(f)

    # --check-pages asserts only what the Markdown can say for itself: that every
    # block still points at its .out, or opts out with a reason. It runs no example,
    # so it needs nothing built and can live in an UNFILTERED lane — which matters,
    # because a doc page can be edited without touching any path that would fire the
    # jobs holding the built surfaces. The two halves catch different regressions:
    # this one catches a block being hand-written back in, --check catches the code
    # changing under a block that is still correctly wired.
    stale: list[Path] = []
    if not args.check_pages:
        if any(s.name == "cli" for s in chosen):
            print(f"cli examples run: {_lat()}")
        print(f"running {len(examples)} example(s) across {len(chosen)} surface(s)…")
        produced = {f: run_example(s, f) for s, f in examples}
        stale = [
            f
            for f, text in produced.items()
            if not out_path(f).exists() or out_path(f).read_text() != text
        ]
        if not args.check:
            for f in stale:
                out_path(f).write_text(produced[f])

    # Page rewrite + the two structural assertions.
    changed_pages: list[Path] = []
    hand_written: list[str] = []
    unreasoned: list[str] = []
    covered = orphan = 0
    for page in sorted(DOCS.rglob("*.md")):
        md = page.read_text()
        blocks = scan(md)
        orphan += len(TEXT_FENCE_RE.findall(md)) - len(blocks)
        for m in blocks:
            if example_key(m.group("include")) not in wanted:
                continue  # a surface this run didn't ask for
            covered += 1
            skip = SKIP_RE.search(m.group("skip"))
            if skip:
                if not (skip.group("reason") or "").strip():
                    unreasoned.append(f"{page.relative_to(DOCS)}: {m.group('include')}")
                continue
            if not m.group("body").strip().startswith("--8<--"):
                hand_written.append(f"{page.relative_to(DOCS)}: {m.group('include')}")
        new = rewrite_page(md, wanted)
        if new != md:
            changed_pages.append(page)
            if not (args.check or args.check_pages):
                page.write_text(new)

    if args.check_pages:
        problems = [
            f"{h} is hand-written — it must include its .out or declare a skip"
            for h in hand_written
        ] + [f"{u} opts out with no reason" for u in unreasoned]
        if problems:
            for p_ in problems:
                print(f"  {p_}")
            sys.exit(
                f"gen_doc_outputs: {len(problems)} example-output block(s) are not wired to "
                "their example.\nRun: uv run --no-sync python tools/gen_doc_outputs.py"
            )
        print(
            f"gen_doc_outputs: {covered} output block(s) are wired to an example "
            f"({orphan} block(s) have no example and are not gated)"
        )
        return

    if args.check:
        problems: list[str] = []
        problems += [f"{f.parent.name}/{out_path(f).name} is stale" for f in stale]
        problems += [
            f"{h} is hand-written — it must include its .out or declare a skip"
            for h in hand_written
        ]
        problems += [f"{u} opts out with no reason" for u in unreasoned]
        problems += [f"{p.relative_to(DOCS)} would be rewritten" for p in changed_pages]
        if problems:
            for p_ in problems:
                print(f"  {p_}")
            sys.exit(
                f"gen_doc_outputs: {len(problems)} problem(s). The docs' example output is "
                "generated from what the examples print.\n"
                "Run: uv run --no-sync python tools/gen_doc_outputs.py"
            )
        print(
            f"gen_doc_outputs: {covered} output block(s) match their examples "
            f"({orphan} block(s) have no example and are not gated)"
        )
        return

    print(
        f"gen_doc_outputs: wrote {len(stale)} .out file(s), rewrote {len(changed_pages)} page(s); "
        f"{covered} block(s) covered, {orphan} block(s) have no example"
    )


if __name__ == "__main__":
    main()
