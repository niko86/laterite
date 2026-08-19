# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite[compat]==0.11.0"]
# ///
"""Docs example — run it with `uv run ex05_query_builder.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing,
and the fixture arm that makes its repo-relative path resolve outside a
checkout.
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
# what this shows: the lazy single-result AgsQuery builder + its four terminals.
import laterite
import pandas as pd
import polars as pl
from duckdb import DuckDBPyRelation

# Build a query lazily: nothing runs until a terminal is called.
q = (
    laterite.read("examples/sample_site.ags")
    .query("SELECT * FROM LOCA")
    .filter("LOCA_GL > 28")
    .select("LOCA_ID", "LOCA_TYPE", "LOCA_GL")
)

# Four terminals materialise the same lazy plan:
frame = q.frame()  # handle's default backend (polars)
pl_df = q.to_polars()  # always polars
pd_df = q.to_pandas()  # always pandas
rel = q.relation()  # the lazy DuckDBPyRelation (not yet materialised)

print(pl_df)

# Projection is honoured exactly as declared.
assert frame.columns == ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"]
assert pl_df.columns == ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"]

# Each terminal returns its expected type.
assert isinstance(frame, pl.DataFrame)
assert isinstance(pl_df, pl.DataFrame)
assert isinstance(pd_df, pd.DataFrame)
assert isinstance(rel, DuckDBPyRelation)
# --8<-- [end:code]
