# Born-typed reads

Every AGS group ships a `TYPE` row — `2DP`, `ID`, `DT`, … — declaring what each
column *is*. laterite reads that row and hands each column the matching polars
dtype, so the frame is typed at the door. No `.cast()`, no `pd.to_numeric`, no
guessing.

```python
--8<-- "python/ex01_read_typed.py"
```

```text
shape: (2, 3)
┌─────────┬───────────┬─────────┐
│ LOCA_ID ┆ LOCA_NATE ┆ LOCA_GL │
│ ---     ┆ ---       ┆ ---     │
│ str     ┆ f64       ┆ f64     │
╞═════════╪═══════════╪═════════╡
│ BH01    ┆ 451105.75 ┆ 23.68   │
│ BH02    ┆ 451235.21 ┆ 32.49   │
└─────────┴───────────┴─────────┘
{'LOCA_ID': 'String', 'LOCA_NATE': 'Float64', 'LOCA_GL': 'Float64'}
```

The `2DP` columns (`LOCA_NATE`, `LOCA_GL`) come back as `Float64`; the `ID`
column (`LOCA_ID`) stays `String`. The mapping follows the `TYPE` row — a `DT`
heading like `TRAN_DATE` reads as `Datetime(time_unit='us')` (LOCA has no `DT`
column, so it doesn't appear here).

!!! note "Why it matters"
    A born-typed frame means arithmetic, sorting, and joins just work. Add a
    depth to a ground level, sort boreholes by easting, join `SAMP` to `LOCA` on
    `LOCA_ID` — no per-column casting, and no silent string-vs-number bugs from
    AGS data arriving as text.

← Back to [Read](../learn/read.md)
