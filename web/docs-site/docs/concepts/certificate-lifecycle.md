# The `.ags.idx` certificate lifecycle

Validating a delivery runs the full rule engine. When you'll reopen the same
file, a **certificate** lets you skip that work: a clean validate mints an
`.ags.idx` sidecar recording a content hash + a byte index of every group. Reopen
with a _fresh, matching_ cert and `.validate()` resolves without re-running a
single rule.

```python
--8<-- "python/ex08_certify.py:code"
```

```text
--8<-- "python/ex08_certify.out"
```

`certify()` needs a prior clean `validate()` on the same handle — it writes
`<path>.ags.idx` next to the file. Re-reading with `index=` hands that cert back;
because the file's content hash still matches the one baked into the cert,
`validate()` returns immediately and `report.resolution == "certified"` instead
of re-deriving the verdict from the rules.

The check is exact: the cert vouches for _those_ bytes only. Edit one character
and the hash no longer matches, so the cert is ignored and the rule engine runs
as normal — a stale cert can never pass a changed file.

!!! warning "It vouches error-clean only"
    A certificate is minted from an error-clean validate (note `warnings=False`
    above). It says "this file has no rule **errors**" — it does **not** capture
    warnings or `fyi`-tier findings, which bypass it entirely. If you need to
    surface warnings, run a normal `.validate()`; the fast-path is for confirming
    error-cleanliness on a file you've already cleared.

The cert is also the same byte index that powers sliced reads (pull one group's
rows without parsing the whole file) — one sidecar, two payoffs.

See also: [Certify a clean file](../cookbook/certify.md) ·
[Validate a delivery](../cookbook/validate-a-delivery.md)
