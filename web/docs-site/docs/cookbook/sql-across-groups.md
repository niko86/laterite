# SQL across groups

**When:** you need a real join across two or more groups — counts, lookups,
aggregates — that a single-group query can't express. Drop to SQL.

```python
--8<-- "python/ex06_sql_join.py"
```

```text
shape: (14, 2)
┌─────────┬─────┐
│ LOCA_ID ┆ n   │
│ ---     ┆ --- │
│ str     ┆ i64 │
╞═════════╪═════╡
│ BH01    ┆ 4   │
│ BH02    ┆ 2   │
│ BH03    ┆ 3   │
│ BH04    ┆ 4   │
│ BH05    ┆ 2   │
│ …       ┆ …   │
│ BH10    ┆ 3   │
│ BH11    ┆ 3   │
│ BH12    ┆ 3   │
│ BH13    ┆ 4   │
│ BH14    ┆ 4   │
└─────────┴─────┘
```

`.sql(...)` exposes every group in the file as a DuckDB table named by its
four-letter code, so `SAMP s JOIN LOCA l USING (LOCA_ID)` joins on the shared
key with no setup — the columns are already born-typed, so `count(*)`, `GROUP
BY`, and numeric comparisons mean what they say. The join key is the AGS
parent/child link: `LOCA_ID` lives on every `SAMP` row and points back at its
borehole.

`.sql()` is a **terminal** — it returns a `DuckDBPyRelation`, not an `AgsQuery`.
Finish it by materialising into the frame library you want:

- `.pl()` → polars (shown above)
- `.df()` → pandas
- `.arrow()` → an Arrow table

One variation — fold a third group in with another `JOIN` and pull real
columns instead of a count:

```python
rel = ags.sql(
    "SELECT l.LOCA_ID, s.SAMP_REF, g.GEOL_GEOL "
    "FROM SAMP s JOIN LOCA l USING (LOCA_ID) JOIN GEOL g USING (LOCA_ID, SAMP_TOP)"
)
df = rel.df()
```

**Gotcha:** `.sql()` is the escape hatch, not the guard-railed builder. Use
[`.query()`](./filter-select.md) for narrowing and selecting within one group —
it keeps you in the typed `AgsQuery` chain and stays lazy until a terminal. Reach
for `.sql()` only when you need a cross-group join or aggregate. Both share the
same DuckDB engine, so the type fidelity is identical.

See also: [Filter & select](./filter-select.md) ·
[Borehole record set](./borehole-record-set.md)
