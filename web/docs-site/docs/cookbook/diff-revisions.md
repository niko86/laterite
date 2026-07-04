# Diff two revisions

**Available in:** Python · Node · Browser

**When:** a resubmission lands and you need to know *what actually changed* between
Rev A and Rev B — not a line diff, but a KEY-aware, type-aware delta.

```python
--8<-- "python/ex16_diff.py"
```

```text
totals: 0 0 1
PROJ key headings: ['PROJ_ID']
changed row key: ['LAT-DEMO']
changed cell: {'heading': 'PROJ_NAME', 'type': 'X', 'a': 'laterite demo site (synthetic starter - replace me)', 'b': 'laterite demo site (Rev B)'}
```

`laterite.diff(a, b)` compares two AGS4 texts and returns a `RevisionDelta` — a
per-group breakdown plus the `total_added` / `total_removed` / `total_changed`
counts. It is **not** a text diff: rows are matched on each group's dictionary
**KEY headings** (here `PROJ_ID`), so a row that moved or was reordered still
lines up with its counterpart. The single edit above registers as one *changed*
row keyed on `["LAT-DEMO"]`, carrying a cell that names the heading, its AGS
`type`, and the `a`/`b` values.

Because cells are compared through the [born-typed](../concepts/born-typed.md)
value, only a genuine quantity change registers — `1.50` vs `1.5` on a `2DP`
column is the same number and produces **no** delta, where a naive line diff
would flag it.

Walk the structure to drive a review: each `group` in `delta["groups"]` carries
`code`, `key_headings`, `keyed` (whether the group has KEYs to match on), and a
`rows` list. Each row has a `kind` (`added` / `removed` / `changed`), its `key`,
and — for a changed row — a `cells` list of `{heading, type, a, b}`:

```python
for group in delta["groups"]:
    for row in group["rows"]:
        if row["kind"] == "changed":
            for cell in row["cells"]:
                print(group["code"], row["key"], cell["heading"], cell["a"], "→", cell["b"])
```

**Gotcha:** matching is only as precise as the KEYs. A group with no KEY headings
(`keyed` is `False`) falls back to positional comparison, so a genuine row
insertion there can read as a cascade of *changed* rows rather than one *added*.
Check `keyed` before trusting row identity on KEY-less groups.

See also: [Certify a file](./certify.md) ·
[Born typed](../concepts/born-typed.md)
