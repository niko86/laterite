# Severity tiers

Not every finding is fatal. laterite grades each one **error** / **warning** /
**fyi**, and — like a compiler — the default report shows errors _and_ warnings,
keeps the informational tier opt-in, and **only errors decide the verdict**.

- **error** — a real Format Rule violation (missing TRAN, short DATA row, an
  abbreviation not declared in ABBR). These make a file invalid.
- **warning** — predicts a downstream _surprise_: a consumer may silently get
  something other than the author meant (a malformed DICT whose custom headings
  nothing can resolve, an unrecognised `TRAN_AGS` edition that sends the whole
  file to the wrong dictionary). Shown by default, and **does not fail**.
- **fyi** — informational only (e.g. extended-ASCII under Rule 1). Hidden unless
  you ask for it, and never fails.

## Two dials, not one

What you **see** and what **fails** are separate settings — the distinction a
compiler draws between a warning and `-Werror`.

| | shows | fails the run |
|---|---|---|
| default | errors + warnings | errors only |
| `--no-warnings` | errors only | errors only |
| `--warnings-as-errors` | errors + warnings | errors **and** warnings |
| `--show-fyi` | adds fyi | unchanged — fyi never fails |

`--no-warnings` and `--warnings-as-errors` contradict each other, so passing both
is rejected rather than silently resolved.

## A warning is reported, and the file still passes

A file whose only blemish is an out-of-range `TRAN_AGS` edition (`4.9.9`) carries
a single **warning** — nothing in the error tier — so it is reported and the run
succeeds:

```bash
--8<-- "cli/validate_warning_tier.sh:cmd"
```

```text
--8<-- "cli/validate_warning_tier.out"
```

Drop the warning tier and the _same file_ reads clean. The exit code was already
`0` — what changed is only what the report shows:

```bash
--8<-- "cli/validate_no_warnings.sh:cmd"
```

```text
--8<-- "cli/validate_no_warnings.out"
```

`--no-warnings` is errors-only; the default keeps warnings; `--show-fyi` adds the
informational tier on top. See [the CLI reference](../reference/cli.md) for the
full flag list.

## The same dials in Python

`validate()` exposes the display tiers as keyword args — `warnings=` (default
`True`) and `fyi=` (default `False`) — plus `warnings_as_errors=` (default
`False`) for the verdict:

```python
--8<-- "python/ex18_severity_tiers.py:code"
```

```text
--8<-- "python/ex18_severity_tiers.out"
```

Three lines, `count · warnings · is_valid` each time: the warning is shown and
the file passes; `warnings=False` hides it and the verdict does not move;
`warnings_as_errors=True` leaves the report alone and flips the verdict.

Same dials as the CLI — `validate(warnings=False)` mirrors `--no-warnings`,
`validate(fyi=True)` mirrors `--show-fyi`, `validate(warnings_as_errors=True)`
mirrors `--warnings-as-errors`. On [`.report`](../learn/validate.md), `count`
reflects whichever tiers you asked for and `errors`/`warnings`/`fyi` split it;
`is_valid` and `exit_code` carry the verdict and are read from the engine, so
they cannot disagree.

!!! warning "Tiers and the certificate fast-path"
    `certify()` measures **every** tier, and the certificate records what each
    one returned. A cert can therefore shortcut any tier it both *measured* and
    found *clean* — not just errors. So a plain `validate()` takes the cert
    short-circuit, and so does `validate(warnings=True, fyi=True)` when the cert
    recorded those tiers clean. What forces a full re-run is asking a question
    the cert cannot fully answer: a tier it never measured, or one it measured
    and found **dirty** — the findings themselves are not stored, only the
    verdict, so the engine has to run to hand them back.

See also: [Validate](../learn/validate.md) · [Certificate lifecycle](./certificate-lifecycle.md)
