# Born-typed reads

Every AGS group ships a `TYPE` row (`2DP`, `ID`, `DT`, …) declaring what each
column _is_. laterite reads that row and hands each column the matching polars
dtype, so the frame is typed at the door. No `.cast()`, no `pd.to_numeric`, no
guessing.

```python
--8<-- "python/ex01_read_typed.py:code"
```

```text
--8<-- "python/ex01_read_typed.out"
```

The `2DP` columns (`LOCA_NATE`, `LOCA_GL`) come back as `Float64`; the `ID`
column (`LOCA_ID`) stays `String`. The mapping follows the `TYPE` row: a `DT`
heading like `TRAN_DATE` reads as `Datetime(time_unit='us')` (this file's `LOCA`
carries no `DT` column, so it doesn't appear here).

!!! note "Why it matters"
    A born-typed frame means arithmetic, sorting, and joins just work. Add a
    depth to a ground level, sort boreholes by easting, join `SAMP` to `LOCA` on
    `LOCA_ID`. No per-column casting, and no silent string-vs-number bugs from
    AGS data arriving as text.

← Back to [Read](../learn/read.md)
