# Produce AGS4

```python
--8<-- "python/ex09a_build_from_frames.py"
```

```text
groups: ['PROJ', 'LOCA']
findings: 3
```

`build_ags4` takes a `{code: frame}` mapping — the columns are your AGS
headings — and constructs a file from exactly the groups you supplied.

AGS4 also mandates the metadata catalogs (`TRAN`, `UNIT`, `TYPE`, plus `ABBR`
for any `PA` pick-list codes), which your frames don't carry. Those are
**reported, not invented** — hence the three findings above, Rules 14/15/17.
`mode="autofix"` repairs what your input _contains_; it does not mint groups you
never wrote.

To get them, ask:

```python
res = laterite.build_ags4(
    {"PROJ": proj, "LOCA": loca},
    synthesise_metadata=True,
    tran_issue="1",
    tran_date="2026-07-30",
    tran_producer="Your Firm",
    tran_recipient="The Client",
    tran_status="Final",
)
```

`UNIT` and `TYPE` are derived from your columns. `TRAN` is not derivable — only
you know who sent what to whom — so you state it. Omit the stamp and no `TRAN`
is written and Rule 14 reports the gap, rather than a placeholder being invented
that would _satisfy_ the rule while asserting a transmission that never happened.
All five values are needed for a finding-free file, because `TRAN_PROD`,
`TRAN_RECV` and `TRAN_STAT` are REQUIRED headings.

Synthesis is opt-in on every surface (`synthesise_metadata=` in Python,
`{ synthesiseMetadata }` in Node, `synthesise_metadata` in the browser wasm
build) so nothing appears in your file that you didn't ask for.

## From a typed PROJ graph

```python
--8<-- "python/ex09b_build_from_typed_graph.py"
```

```text
groups: ['PROJ', 'LOCA']
findings: 3
```

The other door takes a typed graph: a `PROJ` with `LOCA` children attached via
`.locas.append(...)` or the `locas=[...]` constructor kwarg. `build_ags4` walks
it depth-first and — like the frames door — emits **only the headings you
set**, and only the groups you built. That's why a sparse graph builds clean:
nothing is invented, in your data columns or around them. `synthesise_metadata=True`
works here too, and reports the same Rules 14/15/17 without it. The managed child
collection is append-only, so reassigning `p.locas` raises `AttributeError` rather
than silently dropping the rows you built up.

!!! tip
Both doors return the same `BuildResult`. Inspect `res.text` / `res.bytes`
in memory, check `res.findings` for any caveats autofix couldn't resolve, or
`res.save("out.ags")` to persist a byte-faithful AGS4 file to disk.

→ See the whole fluent API assembled in the
[Chaining Showcase](../chaining/index.md).
