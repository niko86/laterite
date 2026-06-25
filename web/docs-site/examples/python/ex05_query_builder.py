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
frame = q.frame()          # handle's default backend (polars)
pl_df = q.to_polars()      # always polars
pd_df = q.to_pandas()      # always pandas
rel = q.relation()         # the lazy DuckDBPyRelation (not yet materialised)

print(pl_df)

# Projection is honoured exactly as declared.
assert frame.columns == ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"]
assert pl_df.columns == ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"]

# Each terminal returns its expected type.
assert isinstance(frame, pl.DataFrame)
assert isinstance(pl_df, pl.DataFrame)
assert isinstance(pd_df, pd.DataFrame)
assert isinstance(rel, DuckDBPyRelation)
