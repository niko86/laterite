# Learn laterite — start here

A linear, five-step tour of the library: install it, read an AGS4 file into typed
frames, validate against the numbered rules, query with the builder or raw SQL, and
produce byte-faithful AGS4 back out. Work through them in order — each page ends with a
**Next →** to the following step.

1. **[Install](./install.md)** — `pip install laterite`, the optional extras, and the
   `lat` CLI in one line.
2. **[Read](./read.md)** — `read()` a path, text, or bytes; AGS types arrive as polars
   dtypes (every column is [born typed](../concepts/born-typed.md)).
3. **[Validate](./validate.md)** — run the numbered-rules engine and read the
   `Report` of findings.
4. **[Query](./query.md)** — fan out a group with `.at()`, chain the lazy `AgsQuery`
   builder, or drop to `.sql()` over DuckDB.
5. **[Produce](./produce.md)** — `build_ags4()` from frames or a typed `PROJ` graph into a
   `BuildResult` you can `.save`.

!!! tip
In a hurry? The [cheatsheet](../reference/cheatsheet.md) is the whole API on one page,
and the [cookbook](../cookbook/index.md) has task-shaped recipes.

[Next → Install](./install.md)
