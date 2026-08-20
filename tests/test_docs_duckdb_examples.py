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

The private dev satellite has a monthly twin of this file that takes the local-build
path, wired into its `compliance-report.yml` after building the extension from
source. Two gates, two artifacts, one question each; this is the one that runs in
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

import os
import re
import shutil
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

    version, mode = c.execute(
        "SELECT extension_version, install_mode FROM duckdb_extensions()"
        " WHERE extension_name = 'laterite_ags4'"
    ).fetchone()
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
