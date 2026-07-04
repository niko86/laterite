# Python — `laterite`

```bash
pip install laterite
```

Python is the **fullest surface** — every capability in the [matrix](index.md#what-each-door-can-do)
lands here first. The deep content lives in the rest of this site:

- **[Learn](../learn/index.md)** — the step-by-step tutorial (install → read → validate → query → produce).
- **[Cookbook](../cookbook/index.md)** — task recipes.
- **[Python API reference](../reference/api.md)** — the full generated API.

This page is just the map of what's **unique to Python** — the parts no other
surface has.

## The fluent handle

```python
import laterite

laterite.read("delivery.ags").validate().save("checked.ags")
```

`read()` returns an `Ags4File` you keep chaining — `validate`, `fix`, `diff`,
`certify`, query — on one handle. See [the fluent model](../concepts/fluent-model.md).

## Only in Python

| Feature | What it is |
|---|---|
| **python-ags4 compat** | `from laterite import compat as AGS4` — a drop-in for the `python_ags4` API, so existing scripts run unchanged |
| **Excel I/O** | `to_excel()` / `from_excel()` — round-trip AGS4 ↔ XLSX |
| **`AgsQuery`** | a lazy, chainable query view (`.query()` / `.filter()` / `.sql()`) over the groups |
| **`registry`** | the typed group graph — `GROUPS`, `child_groups`, the KEY chain — as importable Python |
| **transport** | `pack` / `lock` — zstd + age-passphrase file envelopes (also on Node) |

Everything else — read, validate, build_ags4, fix, diff, certify — is the
[shared vocabulary](index.md#the-shared-vocabulary), documented once and mirrored
on the other surfaces.
