# Dictionary auto-selection

laterite validates against the right AGS edition without you naming it. The
edition is read from the file's `TRAN_AGS` row and resolved to a bundled
dictionary (4.0.3 … 4.2). `dict_for(path)` reports the decision as a
`(version, reason)` tuple so you can see _why_ a file landed on a given edition.

```python
--8<-- "python/ex14_rules_dict.py"
```

```text
--8<-- "python/ex14_rules_dict.out"
```

The last line is the part to read here: `dict_for("examples/sample_site.ags")`
returns `('4.1.1', 'exact')`. The `TRAN_AGS` cell named `4.1.1`, that matched a
bundled edition exactly, so the `reason` is `"exact"` — validation will run
against the 4.1.1 rules and dictionary.

When `TRAN_AGS` is missing, malformed, or names an edition laterite doesn't
carry, the `reason` changes (a fallback to the nearest known edition rather than
`"exact"`) — so you always get a usable dictionary plus a record of how it was
chosen.

## Forcing an edition

Auto-selection is a default, not a cage. Pass `dict_version=` to pin validation
to a specific edition regardless of the `TRAN_AGS` row — useful when a delivery
mislabels its edition, or when you want to test a file against a newer spec:

```python
laterite.read("delivery.ags").validate(dict_version="4.1.1")
```

!!! note "One dictionary, many editions"
    All editions are projected from a single union dictionary at build time, so
    every bundled edition is available in-process — no download, no per-edition
    install. See [Born-typed reads](born-typed.md) for how the same dictionary
    drives column typing.

See also: [List the validator's rules](../cookbook/list-rules.md) ·
[Validate](../learn/validate.md)
