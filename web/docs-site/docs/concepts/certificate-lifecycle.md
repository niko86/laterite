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

`certify()` needs a prior clean `validate()` on the same handle. It writes
`<path>.ags.idx` next to the file. Re-reading with `index=` hands that cert back;
because the file's content hash still matches the one baked into the cert,
`validate()` returns immediately and `report.certified` is `True` instead of
re-deriving the verdict from the rules. `resolution` answers a different
question (which dictionary edition judged the file), so the output above reads
`True exact`, not `certified`.

The check is exact: the cert vouches for _those_ bytes only. Edit one character
and the hash no longer matches, so the cert is ignored and the rule engine runs
as normal: a stale cert can never pass a changed file.

!!! warning "It vouches error-clean only"
    A certificate is minted from an error-clean validate (note `warnings=False`
    above). It says "this file has no rule **errors**". It does **not** capture
    warnings or `fyi`-tier findings, which bypass it entirely. If you need to
    surface warnings, run a normal `.validate()`; the fast-path is for confirming
    error-cleanliness on a file you've already cleared.

## Two doors, and they cost different amounts

`read(index=…)` hands the cert to a **handle**, so the file is parsed either way
(building a handle is what parsing is for), and the cert saves the rule engine
alone. Reach for it when you want the data as well as the verdict.

When the verdict is *all* you want, name the cert on `validate` instead:

=== "Python"

    <!-- doc-code: skip — needs a cert from the example above, which mints it in a temp dir; the shape is the lesson -->
    ```python
    laterite.validate("delivery.ags", index="delivery.ags.idx")
    ```

=== "Node"

    <!-- doc-code: skip — needs a cert from the example above, which mints it in a temp dir; the shape is the lesson -->
    ```javascript
    validate("delivery.ags", { index: "delivery.ags.idx" });
    ```

=== "CLI"

    <!-- doc-code: skip — illustrative `lat` usage over placeholder paths; the CLI examples that ARE executed live in examples/cli/ (#513) -->
    ```bash
    lat validate delivery.ags --index delivery.ags.idx
    ```

A vouched cert lets the engine answer from the bytes' hash and the stamp, so it
skips the **parse** as well as the rules. That is a different operation, not a
faster spelling of the same one: there is no handle afterwards, and nothing to
query.

!!! warning "Naming a cert is an assertion"
    Both doors raise `StaleCertError` if the named cert's size / SHA-256 do not
    match the file, and they raise *before* the rule engine runs. The point of
    naming one is to skip that work, so a mismatch reported afterwards would cost
    exactly what you were trying to save. A cert that genuinely belongs to these
    bytes but cannot answer *this* question (a different engine, a tier it never
    measured, `check_files`) is not an error: the rules run and
    `report.revalidate_reason` says why.

The cert is also the same byte index that powers sliced reads (pull one group's
rows without parsing the whole file). One sidecar, two payoffs.

See also: [Certify a clean file](../cookbook/certify.md) ·
[Validate a delivery](../cookbook/validate-a-delivery.md)
