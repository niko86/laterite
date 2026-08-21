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
for cli, `laterite-node/dist` for node, the installed wheel for python,
`web/src/wasm` for wasm.

TWO SURFACES TAKE THEIR ARTIFACT FROM THE ENVIRONMENT, and both are the same
question asked twice: `LAT_BIN` for cli and `WASM_PKG_DIR` for wasm. Nightly's
`docs-vs-released-*` legs run these examples against the PUBLISHED artifact
rather than this tree's build, so which one was used is a measurement rather than
an assumption — and it is PRINTED either way: `main()` prints what `_lat()`
resolved, `_link_wasm()` prints the package it linked.
"""

from __future__ import annotations

import argparse
import functools
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "web" / "docs-site" / "docs"
EXAMPLES = ROOT / "web" / "docs-site" / "examples"
#: What the docs read from. `sample_site` is what the example FILES open by
#: path; `sample_strata` is the same file plus a GEOL group, and is what the
#: pages' narrative `delivery.ags` is seeded from — `sql-across-groups.md`
#: documents a three-way join through GEOL, so a fixture without it would
#: make a real, working capability look broken.
FIXTURE = ROOT / "examples" / "sample_site.ags"
DELIVERY_FIXTURE = ROOT / "examples" / "sample_strata.ags"
#: The filenames the docs NARRATE — names a reader is meant to substitute their
#: own file for. Seeded from the fixture so a page program can run without the
#: pages being rewritten to a repo path, which would cost the docs their voice
#: for the gate's convenience.
#:
#: Inputs only. `out.ags`, `merged.ags` and `clean.ags` are things a fence WRITES,
#: and pre-creating them would let a snippet that never wrote its output pass.
NARRATIVE_INPUTS = (
    "delivery.ags",
    "site.ags",
    "phase1.ags",
    "phase2.ags",
    "a.ags",
    "b.ags",
)


def seed_workdir(work: Path) -> None:
    """Give a page program the files its text names.

    Shared with `tests/test_docs_duckdb_examples.py` so both surfaces seed the
    same way — two runners preparing subtly different worlds is how a fence
    passes on one surface and fails on the other for reasons about neither.
    """
    (work / "examples").mkdir(exist_ok=True)
    shutil.copy(FIXTURE, work / "examples" / FIXTURE.name)
    for name in NARRATIVE_INPUTS:
        shutil.copy(DELIVERY_FIXTURE, work / name)


#: Where the built wasm package lands, and where Node must see it to resolve the
#: published name. wasm-pack names the package after the CRATE
#: (`laterite-ags4-wasm`); the publish step renames it. Node resolves a symlink by
#: PATH, not by the manifest's `name`, so this works against either.
WASM_PKG = ROOT / "web" / "src" / "wasm"
WASM_LINK = EXAMPLES / "wasm" / "node_modules" / "@laterite" / "ags4-wasm"

#: This checkout's Node package, and the symlink the node EXAMPLES resolve
#: `import … from "laterite"` through. Following the link rather than pointing
#: straight at the package is what makes a page program answer the released
#: package's question for free: `docs-vs-released-npm` re-points this one link at
#: what `npm install laterite` served, and everything downstream follows it.
NODE_PKG = ROOT / "rust-packages" / "laterite-node"
NODE_LINK = EXAMPLES / "node" / "node_modules" / "laterite"


@functools.cache
def _node_pkg() -> Path:
    """Which `laterite` a Node page program imports — printed, once.

    The same shape as `_lat()` and `_wasm_pkg()`, for the same reason: a gate
    whose subject depends on the caller's environment has to say which subject it
    got, or a green run means nothing in particular.
    """
    pkg = NODE_LINK.resolve() if NODE_LINK.exists() else NODE_PKG
    if not (pkg / "dist" / "index.mjs").exists():
        sys.exit(
            f"gen_doc_outputs: node page programs need a built package at {pkg} — "
            "run `npm run build:debug` in rust-packages/laterite-node first"
        )
    print(f"node package: {pkg}")
    return pkg


def _link_node(work: Path) -> None:
    """Put `laterite` where Node's resolver finds it from the page program.

    ESM resolution walks UP from the file, so the link has to sit in the temp
    directory the program is written into — the examples get theirs from the tree
    they live in, and a page program lives nowhere. NODE_PATH is ignored by ESM,
    which is why this is a symlink and not an env var.
    """
    (work / "node_modules").mkdir(exist_ok=True)
    (work / "node_modules" / "laterite").symlink_to(
        _node_pkg(), target_is_directory=True
    )


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

# ---------------------------------------------------------------------------
# The INPUT half of the same guarantee (#513).
#
# Everything above gates the `text` block that shows what an example PRINTS.
# Nothing gated the fence that shows what a reader RUNS — so a snippet could name
# a variable the page never binds and no gate could tell. Same shape deliberately:
# one convention with two halves, not two conventions.
#
# This is the STRUCTURAL half only. It classifies fences; it executes nothing.
# Running them is the next step, and it lands per surface behind this.
# ---------------------------------------------------------------------------

#: The opt-out, spelled to match `doc-output: skip` so there is one thing to learn.
CODE_SKIP_RE = re.compile(
    r"<!-- doc-code: skip(?:\s*[—-]\s*(?P<reason>[^\n]*?))?\s*-->"
)
#: Any fenced block, with the `doc-code` opt-out that may precede it.
CODE_FENCE_RE = re.compile(
    r"^(?P<skip>(?:[ \t]*<!-- doc-code:[^\n]*-->\n(?:[ \t]*\n)*)?)"
    r"(?P<indent>[ \t]*)```(?P<lang>\w+)\n"
    r"(?P<body>(?:.*?\n)*?)"
    r"(?P=indent)```\n",
    re.M,
)
#: Fence language -> the surface whose runner will execute it as part of a page
#: program. Membership IS the decision that a language is meant to be run; a
#: language absent from both this and EXCLUDED_LANGS is prose (json, yaml, text).
PAGE_SURFACE = {
    "python": "python",
    "sql": "duckdb",
    "js": "node",
    "javascript": "node",
    "ts": "node",
}
#: This script's own runner, named once so the routing table and the loop that
#: obeys it cannot drift apart through a typo in a string literal.
HERE = "--run-pages"
#: Fence language -> WHERE its page programs are executed today; `None` means
#: classified but not yet run. Two runners rather than one because the SQL half
#: needs a loaded extension and a live connection, which the pytest module
#: already owns — and because the `duckdb` Surface below shells out to the
#: DuckDB CLI, a binary `pip install duckdb` does not ship. Routing sql through
#: here would print "SKIPPED — duckdb not found" on every machine and read as a
#: gap where there is a gate.
#:
#: `ts` stays pending on purpose, and not for want of a runner. A TypeScript
#: fence's package is decided by the PAGE, not by the tag: the only one in the
#: corpus is a type-only import on `reference/wasm-api.md`, whose package is the
#: BROWSER one — running it under the node surface would answer a question nobody
#: asked. Mapping language to surface cannot express that, and inventing a
#: page-to-surface rule for a single type declaration would be machinery bought
#: for a case that does not exist yet. The census reports it as pending on every
#: run, which is the only claim that stays true either way; #519 carries the
#: three ways to close it, because the choice between them is a decision and not
#: an implementation detail.
PAGE_RUNNER: dict[str, str | None] = {
    "python": HERE,
    "sql": "tests/test_docs_duckdb_examples.py",
    "js": HERE,
    "javascript": HERE,
    "ts": None,
}
#: Languages that LOOK runnable and are deliberately never run. A `bash` fence on
#: a page is an install instruction — `pip install laterite[compat]` — and a gate
#: that executed one would rewrite the machine it runs on. The exclusion is the
#: reason they must opt out EXPLICITLY: silence here is indistinguishable from an
#: oversight, and the next person adding a runnable `lat …` fence would reasonably
#: assume it was covered.
EXCLUDED_LANGS = {"bash", "sh", "shell", "console"}


#: A named section inside an example file: `# --8<-- [start:code]` … `[end:code]`.
#: Pages include a SECTION so the machinery stays in the file and only the lesson
#: reaches the page — so a page program has to resolve the same slice MkDocs does.
SECTION_RE = (
    r"(?s)^[ \t]*#\s*--8<--\s*\[start:{name}\]\n(.*?)^[ \t]*#\s*--8<--\s*\[end:{name}\]"
)
#: The include target inside a fence body.
INCLUDE_IN_FENCE_RE = re.compile(r'--8<--\s*"([^"]+)"')


def _dedent(body: str, indent: str) -> str:
    """Strip a tabbed block's indent, or the source is a syntax error.

    Cookbook pages put fences inside `pymdownx.tabbed`, which indents them four
    spaces. Concatenating that verbatim yields `IndentationError` on the first
    line and would make every tabbed page fail for a reason that has nothing to
    do with the snippet.
    """
    if not indent:
        return body
    return "".join(
        line[len(indent) :] if line.startswith(indent) else line
        for line in body.splitlines(keepends=True)
    )


def resolve_include(ref: str) -> str | None:
    """The source a page's `--8<--` actually pulls in, section suffix honoured."""
    rel, _, section = ref.partition(":")
    path = EXAMPLES / rel
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8")
    if not section:
        return text
    m = re.search(SECTION_RE.format(name=re.escape(section)), text, re.M)
    return m.group(1) if m else None


def page_program(md: str, *langs: str) -> tuple[str, int]:
    """One page's fences of `langs`, concatenated in document order.

    Document order, ignoring tab boundaries, because that is the order a reader
    meets them — and it is what makes a page's include and a continuation further
    down ONE program. That pairing is the whole point: the continuations refer to
    names the include bound, so running them apart proves nothing.

    SEVERAL tags, because a language can be spelled more than one way in a fence
    and a reader does not experience ```js and ```javascript as two languages.
    Building one program per TAG would hand the second half to Node without the
    first half's imports, and fail for a reason about this tool rather than about
    the page.

    Returns the source and how many INLINE fences it contains. Zero inline means
    the page is only includes, which `test_docs_examples.py` already runs as
    files; executing it again here would add cost and no coverage.
    """
    parts: list[str] = []
    inline = 0
    for m in CODE_FENCE_RE.finditer(md):
        if m.group("lang") not in langs or CODE_SKIP_RE.search(m.group("skip")):
            continue
        body = _dedent(m.group("body"), m.group("indent"))
        if body.lstrip().startswith("--8<--"):
            ref = INCLUDE_IN_FENCE_RE.search(body)
            src = resolve_include(ref.group(1)) if ref else None
            if src is not None:
                parts.append(src)
            continue
        inline += 1
        parts.append(body)
    return "\n".join(parts), inline


#: A statement that ASKS for rows, so "returned none" is worth reporting. The
#: DDL a page opens with (`INSTALL`, `LOAD`) returns nothing by definition.
RETURNS_ROWS_RE = re.compile(
    r"^\s*(SELECT|WITH|FROM|VALUES|SHOW|DESCRIBE|PIVOT)\b", re.I
)


def sql_statements(src: str) -> list[tuple[str, bool]]:
    """Split a SQL page program into (statement, asks-for-rows) pairs.

    Here rather than in `tests/test_docs_duckdb_examples.py` because that module
    only runs where the extension installs, and neither the split nor the
    row-asking judgement needs a database — putting them here is what lets the
    buildless lane exercise them.

    The split is a plain `;` scan: a semicolon inside a string literal would cut
    a statement in half. No documented example contains one, and the failure is
    loud rather than silent (the halves are not valid SQL), so a SQL parser
    would be machinery bought against a fault that announces itself.

    Leading `--` comments are stepped over rather than treated as the statement.
    Dropping a chunk that merely STARTS with one drops the query underneath it,
    and introducing a query with a comment is how these pages teach — the first
    example on `duckdb/index.md` is one, and it silently never ran.
    """
    out: list[tuple[str, bool]] = []
    for raw in src.split(";"):
        stmt = raw.strip()
        body = "\n".join(_after_the_preamble(stmt.splitlines()))
        if not body.strip():
            continue  # blank, or comment-only: nothing to execute
        out.append((stmt, RETURNS_ROWS_RE.match(body) is not None))
    return out


def _after_the_preamble(lines: list[str]) -> list[str]:
    """Where the statement really starts: past any leading blank or `--` lines."""
    i = 0
    while i < len(lines) and (
        not lines[i].strip() or lines[i].lstrip().startswith("--")
    ):
        i += 1
    return lines[i:]


def _excerpt(err: str, keep: int = 4) -> str:
    """Both ends of a failure, because runtimes disagree about which one matters.

    A Python traceback ends with the exception; Node's stderr STARTS with it and
    ends in loader frames. Keeping only the tail — which this did — printed four
    lines of `node:internal/modules/esm/loader` and hid the `SyntaxError` that
    said what was actually wrong. Keeping both ends is language-neutral, and the
    elision says how much it dropped rather than trimming silently.
    """
    lines = [ln for ln in err.splitlines() if ln.strip()]
    if len(lines) <= keep * 2:
        return "\n    ".join(lines)
    gap = f"… {len(lines) - keep * 2} line(s) elided …"
    return "\n    ".join([*lines[:keep], gap, *lines[-keep:]])


def run_page_programs(surface: Surface, langs: list[str]) -> list[tuple[str, str]]:
    """Execute every page program for one surface. Returns (page, stderr) failures.

    Per SURFACE rather than per fence tag, because a surface can answer to more
    than one: ```js and ```javascript are one language to Node, and splitting a
    page between them would run the second half without the first half's imports.

    "Does not raise" is the bar. A continuation typically just prints, and
    demanding assertions would mean editing every one of them — the rewrite the
    one-program-per-page decision exists to avoid. It is still enough to catch
    what is actually broken here: a name the page never binds raises NameError.
    """
    failures: list[tuple[str, str]] = []
    ran = 0
    for page in sorted(DOCS.rglob("*.md")):
        src, inline = page_program(page.read_text(encoding="utf-8"), *langs)
        if not inline or not src.strip():
            continue
        ran += 1
        with tempfile.TemporaryDirectory() as td:
            work = Path(td)
            f = work / f"page_program{Path(surface.pattern).suffix}"
            f.write_text(src, encoding="utf-8")
            # A SEEDED cwd, not the repo root. Pages say `delivery.ags` — it is the
            # site's narrative filename, in 31 places — and rewriting every one to
            # the fixture path would trade the docs' voice for the gate's
            # convenience. Seeding the working directory instead keeps page text
            # and executed text identical, which is the property both existing
            # gates advertise; only the environment is prepared, exactly as
            # `test_docs_examples.py` prepares one by running from the repo root.
            #
            # It also removes a real hazard: `delivery.ags` EXISTS at the repo root
            # as a gitignored working artifact holding only PROJ, so running from
            # there gave `read-a-group.md` a KeyError on a missing LOCA rather than
            # the FileNotFoundError CI would have seen. A gate whose result depends
            # on an untracked file is not a gate.
            seed_workdir(work)
            if surface.prepare:
                surface.prepare(work)
            proc = subprocess.run(
                surface.argv(f),
                cwd=work,
                capture_output=True,
                text=True,
                timeout=300,
                env={**os.environ, **surface.env},
            )
        if proc.returncode != 0:
            failures.append((str(page.relative_to(DOCS)), proc.stderr.strip()))
    print(
        f"gen_doc_outputs: ran {ran} {surface.name} page program(s) "
        f"({'/'.join(langs)}); {len(failures)} failed"
    )
    if not ran:
        # Zero is the one result a green run cannot mean. `test_docs_examples.py`
        # guards its glob for exactly this reason — "a moved directory would make
        # every example pass by not running" — and the issue behind this work
        # names that guard as the precedent the page half had not inherited. A
        # fence-regex change, a routing-table typo, or a docs directory moving
        # would each empty this loop, and every one of them would exit 0.
        sys.exit(
            f"gen_doc_outputs: no {surface.name} page programs found "
            f"({'/'.join(langs)}) — the surface is routed to this runner, so "
            "finding none means discovery is broken, not that there is nothing "
            "to run"
        )
    return failures


def census_code_fences(md: str, page: str) -> tuple[dict[str, int], list[str]]:
    """Classify every code fence on one page, and report what cannot stand.

    Four states, mirroring the output half:

      included  the fence is an `--8<--` — it is a file, already gated;
      inline    typed on the page, in a language meant to run (executed once its
                page-program runner lands);
      skipped   opts out, with a reason;
      prose     a language nothing claims to run (json, yaml, diff).

    Two things fail. An opt-out with no reason, because an escape hatch whose use
    is not on the record is just a silence. And an EXCLUDED language that has not
    opted out, because "we never run bash" has to be said somewhere a reader of
    the page can find it.
    """
    counts = {"included": 0, "inline": 0, "skipped": 0, "prose": 0}
    problems: list[str] = []
    for m in CODE_FENCE_RE.finditer(md):
        lang = m.group("lang")
        if lang == "text":
            continue  # the output half's business
        where = f"{page}:{md[: m.start('indent')].count(chr(10)) + 1}"
        if m.group("body").lstrip().startswith("--8<--"):
            counts["included"] += 1
            continue
        skip = CODE_SKIP_RE.search(m.group("skip"))
        if skip:
            counts["skipped"] += 1
            if not (skip.group("reason") or "").strip():
                problems.append(f"{where}: `{lang}` opts out with no reason")
            continue
        if lang in EXCLUDED_LANGS:
            problems.append(
                f"{where}: `{lang}` is never executed — it must say so with "
                "`<!-- doc-code: skip — why -->`"
            )
            continue
        counts["inline" if lang in PAGE_SURFACE else "prose"] += 1
    return counts, problems


@dataclass
class Surface:
    """One example tree: how to find its examples and how to run one."""

    name: str
    pattern: str
    #: argv for an example, given its path. `lat` resolves at call time.
    argv: Callable[[Path], list[str]]
    #: Included in a default run. duckdb is not: its examples need the built
    #: DuckDB extension, which the dev satellite gates on-demand, off the PR path.
    #: cadence: compliance-report
    default: bool = True
    requires: str = ""
    env: dict[str, str] = field(default_factory=dict)
    #: What a PAGE PROGRAM's temp directory needs beyond the seeded fixtures, if
    #: anything. The example trees get this from where they sit in the repo — the
    #: node examples resolve `import … from "laterite"` through a `node_modules`
    #: symlink beside them — but a page program runs nowhere, so whatever the
    #: examples get for free has to be built for it.
    prepare: Callable[[Path], None] | None = None


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
        Surface("node", "ex*.mjs", lambda f: ["node", str(f)], prepare=_link_node),
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


def _wasm_pkg() -> Path:
    """Which wasm package the examples run against — not always this tree's build.

    The same shape as `_lat()`, for the same reason. Nightly's
    `docs-vs-released-wasm` leg asks whether the pages work against the package
    `npm install @laterite/ags4-wasm` actually serves, so the directory is an
    INPUT to this script rather than a constant: `WASM_PKG_DIR` if set (that leg
    pins it at the installed package), else this checkout's `wasm-pack` output.

    Nothing else about the wasm surface changes — the examples still import the
    published name, and the symlink below is still what makes that resolve.
    """
    if env := os.environ.get("WASM_PKG_DIR"):
        # RESOLVED, because the two things done with this path resolve relative
        # paths against DIFFERENT directories: the existence check below reads it
        # from the cwd, while the symlink target is looked up from the link's own
        # directory, deep under `web/docs-site/examples/`. A relative
        # `WASM_PKG_DIR` would pass the check and then write a dangling link —
        # every example dying with ERR_MODULE_NOT_FOUND directly under a printed
        # line saying the package was found.
        return Path(env).resolve()
    return WASM_PKG


def _link_wasm() -> None:
    """Put the wasm package where Node resolves `@laterite/ags4-wasm`.

    The examples import the published name rather than a relative path, because a
    reader copying one should get working code, not a path into this repo. The
    same trick `docs-examples.test.ts` uses for the node examples — idempotent,
    gitignored, and pointing at whatever `_wasm_pkg()` resolved. Which package
    that was is PRINTED: a gate whose subject depends on the caller's environment
    must say what it measured.
    """
    pkg = _wasm_pkg()
    if not (pkg / "ags4_wasm.js").exists():
        sys.exit(
            f"gen_doc_outputs: surface wasm needs a built package at {pkg} — "
            "run `npm run build:wasm` in web/ first, or point WASM_PKG_DIR at one"
        )
    print(f"wasm package: {pkg}")
    WASM_LINK.parent.mkdir(parents=True, exist_ok=True)
    # A REAL directory can sit here, not just the symlink this writes: anyone
    # pointing WASM_PKG_DIR at an installed package is the same person liable to
    # have run `npm install @laterite/ags4-wasm` in the examples tree first, and
    # `unlink()` on a directory raises instead of replacing it.
    if WASM_LINK.is_symlink() or WASM_LINK.is_file():
        WASM_LINK.unlink()
    elif WASM_LINK.is_dir():
        shutil.rmtree(WASM_LINK)
    WASM_LINK.symlink_to(pkg, target_is_directory=True)


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
        HERE,
        action="store_true",
        help="execute each page's fences as one program (nightly; needs the surface built)",
    )
    ap.add_argument(
        "--surface",
        action="append",
        choices=sorted(SURFACES),
        help="limit to one surface (repeatable); default is every surface not needing extra tooling",
    )
    args = ap.parse_args()

    if args.run_pages:
        # Nightly lane. The structural half (--check-pages) runs on every PR and
        # says which fences SHOULD run; this is what actually runs them.
        bad: list[tuple[str, str]] = []
        for surface_name in sorted(set(PAGE_SURFACE.values())):
            # Grouped by surface, then filtered to the tags routed HERE — the two
            # are not the same cut. `ts` reaches the node surface and is still
            # pending, so a surface can be half-claimed and the loop has to run
            # the claimed half rather than all or nothing.
            langs = sorted(
                lang
                for lang, s_name in PAGE_SURFACE.items()
                if s_name == surface_name and PAGE_RUNNER[lang] == HERE
            )
            if not langs:
                continue  # not this runner's surface; the census says whose
            if args.surface and surface_name not in args.surface:
                continue
            s = SURFACES[surface_name]
            if s.requires and not shutil.which(s.requires):
                print(
                    f"gen_doc_outputs: {surface_name} page programs SKIPPED — "
                    f"{s.requires} not found"
                )
                continue
            bad += [
                (f"{surface_name} · {p}", err) for p, err in run_page_programs(s, langs)
            ]
        if bad:
            for where, err in bad:
                print(f"\n  {where}\n    {_excerpt(err)}")
            sys.exit(
                f"\ngen_doc_outputs: {len(bad)} page program(s) do not run. A reader "
                "following the page hits this."
            )
        return

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
    code_counts = {"included": 0, "inline": 0, "skipped": 0, "prose": 0}
    code_problems: list[str] = []
    covered = orphan = 0
    for page in sorted(DOCS.rglob("*.md")):
        md = page.read_text()
        blocks = scan(md)
        orphan += len(TEXT_FENCE_RE.findall(md)) - len(blocks)
        # The input half runs over EVERY page on every invocation: it reads
        # Markdown and needs nothing built, so scoping it to the chosen surfaces
        # would leave fences unclassified in the only lane that always fires.
        c, p_ = census_code_fences(md, str(page.relative_to(DOCS)))
        for k, v in c.items():
            code_counts[k] += v
        code_problems += p_
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
        problems = (
            [
                f"{h} is hand-written — it must include its .out or declare a skip"
                for h in hand_written
            ]
            + [f"{u} opts out with no reason" for u in unreasoned]
            + code_problems
        )
        if problems:
            for p_ in problems:
                print(f"  {p_}")
            sys.exit(
                f"gen_doc_outputs: {len(problems)} doc block(s) are not wired to "
                "their example.\nRun: uv run --no-sync python tools/gen_doc_outputs.py"
            )
        print(
            f"gen_doc_outputs: {covered} output block(s) are wired to an example "
            f"({orphan} block(s) have no example and are not gated)"
        )
        # Say what the input half saw, pass or fail. `inline` is the number that
        # matters: fences a reader can copy which no gate has yet executed. It
        # falls as the per-surface runners land, and stating it is what stops
        # "structurally classified" reading as "known to run".
        print(
            f"gen_doc_outputs: {code_counts['included']} code fence(s) are includes, "
            f"{code_counts['inline']} inline, "
            f"{code_counts['skipped']} skipped with a reason, "
            f"{code_counts['prose']} prose"
        )
        # Which of the inline ones a runner actually executes, because "classified"
        # and "known to run" are different claims and the gap between them is the
        # thing worth watching. This half runs no examples, so it reports the
        # SHAPE of the coverage; `--run-pages` is what proves it.
        # Grouped BY RUNNER rather than by language: one line per language read
        # as five separate gates when it is two, and the question a reader has
        # here is "what runs this, and what runs nothing".
        by_runner: dict[str, list[str]] = {}
        for lang, where in sorted(PAGE_RUNNER.items()):
            by_runner.setdefault(where or "PENDING — nothing runs these", []).append(
                lang
            )
        print(
            "gen_doc_outputs: page programs execute in the nightly — "
            + "; ".join(f"{w}: {', '.join(ls)}" for w, ls in sorted(by_runner.items()))
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
