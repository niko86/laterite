# laterite

laterite reads, validates, queries and **produces** AGS4 geotechnical data. Files come back as
**born-typed** polars frames (the polars dtype _is_ the AGS type), wired into a fluent, chainable
API. One engine drives Python, the `lat` CLI, Node and DuckDB, and it's a drop-in for
python-ags4, rebuilt on a Rust core for speed.

<!-- doc-code: skip — installs packages; a gate that ran it would rewrite its own environment -->
```bash
pip install laterite
```

laterite is **[in beta](reference/support.md)**. The engine is tested; what it hasn't
had is your files. [Tell us how it goes](feedback.md).

## In one breath

```python
--8<-- "python/ex02_validate.py:code"
```

```text
--8<-- "python/ex02_validate.out"
```

`read(...)` gives you an `Ags4File`; `.validate()` runs the numbered-rules engine and hands the file
straight back so the chain keeps flowing, with the verdict on `.report`. The dictionary edition
(`4.1.1`) is picked automatically from the file's `TRAN_AGS` row: no flags, no guessing.

!!! tip
    Every frame is born typed. A `2DP` column is a polars `Float64`, a date is a `Date`, an `ID` is a
    `String`. So `.query(...)`, `.sql(...)` and plain polars all see real types, not text.

## Where to go next

- **New here? → [Learn](learn/install.md)**: install, then read → validate → query → produce, one step at a time.
- **Need to get something done? → [Cookbook](cookbook/index.md)**: task-shaped recipes you can lift wholesale.
- **Show me what it can do? → [Chaining](chaining/index.md)**: the fluent API end to end, one chain at a time.
- **Looking up a function? → [Reference](reference/cheatsheet.md)**: the cheatsheet and the `lat` CLI.
