# Validate

```python
--8<-- "python/ex02_validate.py"
```

```text
--8<-- "python/ex02_validate.out"
```

`read(path).validate()` runs the numbered-rules engine and hands back the
[`Ags4File`](./read.md) — so it chains. The verdict itself rides on `.report`:
`is_valid` and `count` are the headline, while `dict_version` and `resolution`
tell you _which_ AGS edition the rules came from.

!!! note "The edition selects itself"
    You never pass an edition. `dict_version` is read straight from the file's
    `TRAN_AGS` field; `resolution='exact'` means that edition was matched
    on the nose (otherwise laterite falls back to the nearest dictionary it
    ships). Here the file declared 4.1, so the 4.1.1 rules ran.

Because `.validate()` returns the file, you keep going on the same handle —
query it, slice a group, or emit it — without re-reading from disk.

Next → [Query across groups](./query.md)
