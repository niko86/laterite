# Pull one borehole record set

**Available in:** Python · Node · DuckDB · Browser

Fan out from a `LOCA` location to everything that hangs off it — samples, tests,
the lot — as a dict of typed frames keyed by group code. Reach for this when you
want a single borehole's whole story, not one group at a time.

First, see *which* groups fan out for the boreholes you picked — `.at(code, ids)`
returns a query whose `.groups` lists the related group set:

```python
--8<-- "python/ex03_at_fanout_groups.py"
```

```text
['LOCA', 'SAMP', 'LLPL']
```

`.at("LOCA", ["BH01", "BH02"])` walks the dictionary's parent graph down from
`LOCA` and keeps only the groups that actually carry rows for those locations.
`q.groups` is the manifest — `LOCA` itself, `SAMP` (samples), `LLPL` (Atterberg
limits) — so you know what's coming before you materialise anything.

Then call `.frames()` to materialise the record set as `{group_code: frame}`:

```python
--8<-- "python/ex04_at_frames.py"
```

```text
['LLPL', 'LOCA', 'SAMP']
4
```

`frames` is a plain dict of **born-typed** polars frames — pull one out by its
4-letter code (`frames["SAMP"]`, not `q["SAMP"]`). Each frame is already typed
straight from the AGS TYPE row, and each is row-filtered to just the boreholes
you asked for, so `frames["SAMP"]` here is the four samples taken on `BH01`.

When to use it: building a per-location report, exporting one hole's data, or
feeding a downstream model that wants the whole record set at once. The dict
keys are exactly `q.groups`, so you can iterate the manifest and grab frames in
one pass. Gotcha: `.at(...)` is a fan-out, not a join — each group stays a
separate frame keyed on its own KEY heading. If you want the groups *joined*
into one wide result, run SQL across them instead.

See also: [SQL across groups](./sql-across-groups.md) ·
[Born-typed](../concepts/born-typed.md)
