# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite[pyarrow]==0.11.0"]
# ///
"""Docs example — run it with `uv run ex06_sql_join.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing,
and the fixture arm that makes its repo-relative path resolve outside a
checkout.

The header asks for `[pyarrow]`, and the dependency is not laterite's. `.sql()`
returns a DuckDB relation, and DuckDB's `.pl()` / `.arrow()` materialisers
import pyarrow themselves; laterite's own `.frame()` / `.to_polars()` stay
pyarrow-free. A bare pin runs green in any environment that happens to have
pyarrow and hands a reader `ModuleNotFoundError` at line `rel.pl()`.
"""

import urllib.request
from pathlib import Path

_FIXTURE = Path("examples/sample_site.ags")
_RAW = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"
if not _FIXTURE.exists():
    # Cold only for a reader running this outside the repo: in a checkout (and in
    # CI, cwd = repo root) the file is already there and this arm never executes,
    # so the gates stay offline. Fetching it — rather than rewriting the example
    # to an absolute path — is what keeps the text on the page the text you would
    # actually type.
    _FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    _FIXTURE.write_bytes(urllib.request.urlopen(_RAW, timeout=30).read())

# --8<-- [start:code]
# what this shows: drop to raw SQL to join across groups, count samples per location.
import laterite

rel = laterite.read("examples/sample_site.ags").sql(
    "SELECT l.LOCA_ID, count(*) n FROM SAMP s JOIN LOCA l USING (LOCA_ID) "
    "GROUP BY 1 ORDER BY 1"
)

# rel is a DuckDBPyRelation (terminal); materialise to polars with .pl().
df = rel.pl()
print(df)

assert hasattr(rel, "pl")
assert df.height >= 1 and "n" in df.columns
# --8<-- [end:code]
