# Get one group as a typed frame

Pull a single AGS group straight into a polars frame whose dtypes already match
the group's `TYPE` row — no casting at the call site.

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

`ags["LOCA"]` is a born-typed polars `DataFrame`: the dtype *is* the `TYPE` row.
The `2DP` columns (`LOCA_NATE`, `LOCA_GL`) arrive as `Float64`; the `ID` column
(`LOCA_ID`) stays `String`. You can sort, add, and join immediately — no
`.cast()`, no `pd.to_numeric`.

`ags.table("LOCA")` is the same call by another name — use whichever reads
better at the call site.

**The three input doors** — `read` takes a path, or read from memory:

```python
laterite.read("delivery.ags")          # a path
laterite.read(text=ags_string)         # text already in hand
laterite.read(data=ags_bytes)          # raw bytes (e.g. an upload)
```

A missing group raises `KeyError`; check `"LOCA" in ags` first if the group may
be absent.

See also: [Read](../learn/read.md) · [Born-typed reads](../concepts/born-typed.md)
