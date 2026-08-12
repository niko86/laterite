# Severity tiers

Not every finding is fatal. laterite grades each one **error** / **warning** /
**fyi**, and — like a compiler — the default report shows errors _and_ warnings,
keeps the informational tier opt-in, and lets you drop to errors only.

- **error** — a real Format Rule violation (missing TRAN, short DATA row, an
  abbreviation not declared in ABBR). These make a file invalid.
- **warning** — a soft signal: a malformed DICT, an unrecognised `TRAN_AGS`
  edition, nonstandard abbreviations. Shown by default; suppressible.
- **fyi** — informational only (e.g. extended-ASCII under Rule 1). Hidden unless
  you ask for it.

## The same file, two verdicts

A file whose only blemish is an out-of-range `TRAN_AGS` edition (`4.9.9`) carries
a single **warning** — nothing in the error tier:

```bash
--8<-- "cli/validate_warning_tier.sh:cmd"
```

```text
--8<-- "cli/validate_warning_tier.out"
```

Drop the warning tier and the _same file_ reads clean — and the exit code flips
from `1` to `0`:

```bash
--8<-- "cli/validate_no_warnings.sh:cmd"
```

```text
--8<-- "cli/validate_no_warnings.out"
```

`--no-warnings` is errors-only; the default keeps warnings; `--show-fyi` adds the
informational tier on top. See [the CLI reference](../reference/cli.md) for the
full flag list.

## The same dial in Python

`validate()` exposes the tiers as keyword args — `warnings=` (default `True`) and
`fyi=` (default `False`):

```python
--8<-- "python/ex18_severity_tiers.py:code"
```

```text
--8<-- "python/ex18_severity_tiers.out"
```

Same dial as the CLI: `validate(warnings=False)` mirrors `--no-warnings`, and
`validate(fyi=True)` mirrors `--show-fyi`. The verdict (`is_valid`, `count`) on
[`.report`](../learn/validate.md) reflects whichever tiers you asked for.

!!! warning "Tiers and the certificate fast-path"
    An `.ags.idx` [certificate](./certificate-lifecycle.md) vouches that a file
    is **error-clean** — that and only that. So a plain `validate()` (or
    `validate(warnings=False)`) can take the cert short-circuit and skip the rule
    engine. The moment you ask for a tier the cert never promised —
    `validate(warnings=True, fyi=True)` wanting the soft findings back — there's
    nothing cached to trust, so the call runs the full engine regardless of the
    cert. Warnings and fyi are computed fresh; the error verdict is the only thing
    a cert can shortcut.

See also: [Validate](../learn/validate.md) · [Certificate lifecycle](./certificate-lifecycle.md)
