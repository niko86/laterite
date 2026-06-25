# List the validation rules

Enumerate the validator's numbered rules in Python, and ask which dictionary
edition a file resolves to.

```python
--8<-- "python/ex14_rules_dict.py"
```

```text
27
['checks', 'fixable', 'observations', 'rule', 'severity', 'title']
('4.1.1', 'exact')
```

`laterite.list_rules()` returns one rich dict per numbered AGS4 rule — 27 of
them, covering `1`, `2`, `2a`, `2b`, `3` … through `20`. Each dict carries the
rule number (`rule`), a human `title`, a prose `checks` description, a
`severity` (`error`, or `mixed` where the rule flags some findings for
information only), a `fixable` flag (whether `.fix()` / `lat-check --fix` can
repair it mechanically), and the `observations` it relates to in the
[divergence catalogue](../concepts/dictionary-selection.md). It's the
programmatic mirror of the rule table the CLI prints — drive a docs page, a UI
legend, or a coverage check off it without hard-coding rule numbers.

`laterite.dict_for(path)` resolves a file to the dictionary edition the
validator will judge it against, as a `(version, reason)` tuple — here
`('4.1.1', 'exact')` because the file's `TRAN_AGS` declares 4.1.1 and that
edition is bundled. The `reason` tells you *how* the pick was made (an exact
match, a fallback, …); see [Dictionary selection](../concepts/dictionary-selection.md)
for the resolution order.

Filter the list the same way you'd filter any list of dicts — e.g. just the
mechanically-fixable rules:

```python
fixable = [r["rule"] for r in laterite.list_rules() if r["fixable"]]
```

The `severity` field is `mixed` for rules that emit both errors and
information-only findings (Rule 1 character-set, Rule 16 ABBR pick-lists, Rule
18 dictionary references); everything else is a hard `error`.

See also: [CLI reference](../reference/cli.md) · [Dictionary selection](../concepts/dictionary-selection.md)
