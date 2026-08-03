# Read & explore

```python
--8<-- "python/ex01_read_typed.py"
```

```text
--8<-- "python/ex01_read_typed.out"
```

`laterite.read` hands back a group as a **born-typed** polars frame. Index it
by 4-letter code (`ags["LOCA"]`) and you get a real DataFrame — already typed.
The dtype row _is_ the AGS TYPE row: `2DP` lands as `Float64`, `ID` as
`String`. No `.cast(...)`, no `pd.to_numeric`, no per-column cleanup. The
arithmetic just works because the column was numeric the moment it was read.

## Three doors in

`read` takes whichever form your data already has — there's no separate
loader to pick:

```python
laterite.read("examples/sample_site.ags")   # a path
laterite.read(text="GROUP,...\n...")          # an in-memory AGS4 string
laterite.read(data=raw_bytes)                  # raw bytes (an upload, a download)
```

All three return the same object, so the rest of your code doesn't care where
the file came from.

!!! tip
    Typing happens at read time straight from the dictionary, so a group that
    isn't in your file simply isn't a key. Iterate `ags` to see what's present.

Curious _how_ the dtype gets fixed from the AGS TYPE row? See
[how this works → born-typed](../concepts/born-typed.md).

Next → [Validate in Python](./validate.md)
