"""Do the docs site's SQL examples run against the extension a reader installs?

Every other DuckDB check in this project runs against a LOCALLY BUILT extension.
That answers "our build works", which is a different claim from "the published one
does" — the community pipeline has its own build, its own toolchain image and its
own DuckDB target, and `laterite-duckdb` is a separate repo on its own version line
with deliberately no joint release train (`ags-wiki/design/dec-duckdb-extension.md`).
So nothing was asking the reader's question, and the reader is not on our build:
they are on `INSTALL laterite_ags4 FROM community`.

The gap is not hypothetical. Building the equivalent npm leg is what surfaced a
`peerDependencies` range matching zero published versions, which had made `sql()`
and `at()` unreachable for every npm consumer since publication. No per-PR gate
could see it, because per-PR gates test the tree.

WHICH EXTENSION IS UNDER TEST IS A MEASUREMENT, NOT AN ASSUMPTION — the same rule
`gen_doc_outputs.py:_lat()` follows, and for the same reason: a gate whose subject
depends on the caller's environment must say which subject it got. Two env vars
choose, and the choice is printed:

    LATERITE_DUCKDB_COMMUNITY=1     INSTALL … FROM community — the PUBLISHED build
    LATERITE_DUCKDB_EXT=<path>      LOAD that file — a local build

Neither set and the module skips, so per-PR CI still only include-checks these
files (`mkdocs build --strict` + `check_paths`). `docs-vs-released-duckdb` in
`.github/workflows/nightly.yml` sets the first. Locally::

    LATERITE_DUCKDB_COMMUNITY=1 uv run pytest tests/test_docs_duckdb_examples.py -q -rs

`_`-prefixed files are include-only — `_install.sql` is the `INSTALL … FROM
community` boilerplate the pages show, and running it is this module's job rather
than an example's.

The private dev satellite has an on-demand twin of this file that takes the
local-build path, wired into its `compliance-report.yml` after building the
extension from source.  cadence: compliance-report Two gates, two artifacts, one question each; this is the one that runs in
the repo the docs live in.

A LOAD failure in local-build mode is ABI drift between pip `duckdb` and the
extension's pinned C-API — a visible skip, not a doc break. In community mode it is
not survivable and goes red: if the published extension cannot load into a current
DuckDB, no reader can run anything on this page.

Each example runs from a temp dir seeded with a copy of the shared fixture, never
the repo tree: `certify_ags` mints `.ags.idx` sidecars, and a stale sidecar would
flip `validate_ags` onto its certified fast path.
"""

from __future__ import annotations

import functools
import os
import re
import shutil
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parents[1]
_SQL_DIR = _REPO / "web" / "docs-site" / "examples" / "duckdb"
_SQL = sorted(p for p in _SQL_DIR.glob("*.sql") if not p.name.startswith("_"))
_FIXTURE = _REPO / "examples" / "sample_site.ags"

_EXT = os.environ.get("LATERITE_DUCKDB_EXT", "")
_COMMUNITY = os.environ.get("LATERITE_DUCKDB_COMMUNITY", "") not in ("", "0")

# `needs_env` because `duckdb` is not in the buildless `repo-gates` job's env —
# the import is function-local so collection would survive, but the marker is what
# `tests/test_build_marker_faithful.py` reads, and a rule enforced by inspection is
# worth more than one enforced by whether today's import happens to be lazy.
pytestmark = [
    pytest.mark.needs_env,
    pytest.mark.skipif(
        not (_COMMUNITY or (_EXT and Path(_EXT).exists())),
        reason="set LATERITE_DUCKDB_COMMUNITY=1 (published) or LATERITE_DUCKDB_EXT=<path> (local build)",
    ),
]


def test_example_library_is_non_empty() -> None:
    # Guard against a glob that silently matches nothing: a moved directory would
    # make every example "pass" by not running, and this suite would go green
    # while testing the empty set. Counted by discovery on purpose — the number is
    # never written down anywhere it could go stale.
    assert _SQL, f"no docs DuckDB examples found under {_SQL_DIR}"


@pytest.fixture(scope="module")
def con():
    import duckdb

    if _COMMUNITY:
        c = duckdb.connect()
        c.execute("INSTALL laterite_ags4 FROM community")
        c.execute("LOAD laterite_ags4")
    else:
        c = duckdb.connect(config={"allow_unsigned_extensions": "true"})
        try:
            c.execute(f"LOAD '{Path(_EXT).resolve().as_posix()}'")
        except Exception as e:  # pip-duckdb vs extension C-API drift, not a doc break
            pytest.skip(f"extension load failed (ABI drift?): {e}")

    row = c.execute(
        "SELECT extension_version, install_mode FROM duckdb_extensions()"
        " WHERE extension_name = 'laterite_ags4'"
    ).fetchone()
    # The LOAD above succeeded, so the row exists — but `fetchone()` is
    # `… | None` and a bare unpack would raise a TypeError naming the tuple
    # rather than the missing extension, three lines from the cause.
    assert row is not None, "laterite_ags4 loaded but duckdb_extensions() omits it"
    version, mode = row
    print(
        f"\nunder test: laterite_ags4 {version} ({mode}) on duckdb {duckdb.__version__}"
    )
    return c


@pytest.mark.parametrize("sql_file", _SQL, ids=lambda p: p.stem)
def test_docs_duckdb_example_runs(
    con, sql_file: Path, tmp_path: Path, monkeypatch
) -> None:
    # DuckDB resolves relative paths against the process cwd at execute time, so
    # seeding a temp cwd keeps the page's literal `examples/sample_site.ags` real
    # without the example knowing it is under test.
    (tmp_path / "examples").mkdir()
    shutil.copy(_FIXTURE, tmp_path / "examples" / _FIXTURE.name)
    monkeypatch.chdir(tmp_path)

    text = sql_file.read_text()
    rows = con.execute(text).fetchall()

    m = re.search(r"^-- expect-rows: (\d+)$", text, re.MULTILINE)
    assert m, (
        f"{sql_file.name}: no `-- expect-rows: N` annotation. Without one this test"
        " asserts only that the SQL parses, which a published extension serving"
        " zero rows would satisfy."
    )
    assert len(rows) == int(m.group(1)), (
        f"{sql_file.name}: {len(rows)} row(s), expected {m.group(1)}:\n{rows}"
    )


# --- SQL page programs (#513 step 3) -----------------------------------------
#
# The example FILES above are one half. The other half is the SQL typed directly
# onto a page rather than included from one — every statement a reader copies out
# of the prose — and until now nothing ran a single one of them.
#
# Built by `gen_doc_outputs.page_program`, the same function the Python runner
# uses, so a page's fences concatenate the same way on every surface. Executed
# HERE rather than in that tool because it documents itself as stdlib-only and
# runs in a buildless lane — importing `duckdb` there would break both. This file
# already owns the connection, the extension-mode reporting and the env gating.

_GEN = _REPO / "tools" / "gen_doc_outputs.py"
_DOCS = _REPO / "web" / "docs-site" / "docs"


@functools.cache
def _gen_doc_outputs():
    """Load the builder without importing `tools` as a package.

    Registered in `sys.modules` before execution because the module defines a
    dataclass, and `dataclasses` resolves the defining module by name — an
    unregistered one gives `AttributeError: 'NoneType' object has no attribute
    '__dict__'` at import time rather than anything about dataclasses.
    """
    import importlib.util

    spec = importlib.util.spec_from_file_location("gen_doc_outputs", _GEN)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["gen_doc_outputs"] = mod
    spec.loader.exec_module(mod)
    return mod


def _page_program(md: str, lang: str) -> tuple[str, int]:
    return _gen_doc_outputs().page_program(md, lang)


def _sql_pages() -> list[Path]:
    return [
        p
        for p in sorted(_DOCS.rglob("*.md"))
        if _page_program(p.read_text(encoding="utf-8"), "sql")[1]
    ]


_SQL_PAGES = _sql_pages()


def test_sql_page_library_is_non_empty() -> None:
    """Zero pages would make every case below vacuous — the same guard the
    example library carries, for the same reason."""
    assert _SQL_PAGES, "no docs page has an inline SQL fence"


@pytest.mark.parametrize(
    "page", _SQL_PAGES, ids=lambda p: p.relative_to(_DOCS).as_posix()
)
def test_sql_page_program_runs(page: Path, con, tmp_path: Path, monkeypatch) -> None:
    """A page's SQL fences, concatenated and run as one script.

    "Does not raise" is the bar, as for the Python programs. It is weaker here
    than the `-- expect-rows` the example files carry — a query can bind and
    return nothing — but it still catches the class that matters: a column or
    group the extension does not have. The join this found on
    `cookbook/sql-across-groups.md` failed exactly that way.

    Zero-row statements are counted and reported rather than failed, so the
    weaker guarantee is visible instead of assumed.
    """
    # The SAME seeding the Python runner uses, from the same function — two
    # runners preparing subtly different worlds is how a fence passes on one
    # surface and fails on the other for reasons about neither.
    _gen_doc_outputs().seed_workdir(tmp_path)
    monkeypatch.chdir(tmp_path)

    src, _ = _page_program(page.read_text(encoding="utf-8"), "sql")
    ran = empty = queried = 0
    for stmt, asks_for_rows in _gen_doc_outputs().sql_statements(src):
        rows = con.execute(stmt).fetchall()
        ran += 1
        # Only a statement that ASKS for rows can suspiciously return none, and
        # counting the `INSTALL`/`LOAD` boilerplate inflated both halves of the
        # ratio — a report that cries wolf on its own preamble is one nobody
        # reads the day it means something.
        if not asks_for_rows:
            continue
        queried += 1
        if not rows:
            empty += 1
    # Printed pass or fail, because the row audit LOOKS AT LESS than it runs, and
    # this line is the only place that gap is visible. Reporting solely on a
    # nonzero `empty` would make a page whose statements are all boilerplate
    # indistinguishable from one whose every query finds rows.
    print(
        f"\n{page.relative_to(_DOCS)}: {ran} statement(s) ran, {queried} asked for "
        f"rows, {empty} of those returned none"
    )
