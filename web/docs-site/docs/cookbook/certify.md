# Certify a clean file & skip re-validation

**Available in:** Python · Node · CLI · [Browser](../surfaces/browser.md)

Mint an `.ags.idx` certificate from a clean validation, then reopen with it to
skip the rule engine.

=== "Python"

    ```python
    --8<-- "python/ex08_certify.py:code"
    ```

    ```text
    --8<-- "python/ex08_certify.out"
    ```

    `.certify()` **runs the validation itself**, with every severity tier on,
    and records what the rules actually returned — no prior
    [`.validate()`](../learn/validate.md) is needed. It writes
    `<path>.ags.idx` next to the file: a validity certificate (the verdict plus
    a hash of the bytes it vouches for) and a byte-offset index of every group.

    Reopen with `read(path, index=...)` and the next `.validate()` resolves
    from the cert instead of running the numbered rules. Two separate fields
    tell you what happened: `.report.certified` is `True` when the cert matched
    the file's current bytes and the rule engine was skipped, and
    `.report.resolution` reports how the dictionary edition was resolved
    (`"exact"` or a fallback — see [Validate](../learn/validate.md)).

=== "Node"

    ```js
    --8<-- "node/ex08_certify.mjs"
    ```

    ```text
    --8<-- "node/ex08_certify.out"
    ```

    The same lifecycle, the same file format: `certify()` validates and mints
    `<path>.ags.idx` in one step, and `read(path, { index })` + `validate()`
    resolves from it with `report.certified === true`.
    The cert wraps the one core `Sidecar`, so a Node-minted `.ags.idx` is
    byte-compatible with the ones Python, DuckDB and `lat` mint — any
    surface can consume any surface's cert.

=== "DuckDB"

    The `laterite_ags4` DuckDB extension is a **read-only reader** — it doesn't
    mint certificates. Certify with `lat certify` or the library (the tabs
    above); `read_ags` then *consumes* the resulting `<path>.ags.idx` beside the
    file to range-read a single group's bytes instead of parsing the whole file.
    See the [DuckDB function reference](../reference/duckdb-functions.md).

=== "CLI"

    ```bash
    --8<-- "cli/certify_emit_index.sh:cmd"
    ```

    ```text
    --8<-- "cli/certify_emit_index.out"
    ```

    `certify` mints the certificate after the check comes back clean (a
    `certificate written to site.ags.idx` line reports where; point it
    elsewhere with `--out`). A file with error-severity findings gets no cert:
    a single `error:` line explains why and points you at
    `lat validate <file>` for the findings table, and the exit code is `1`.

The cert is **content-bound**: if the file changes by a single byte, the hash
no longer matches and laterite silently falls back to a full validation — a
stale cert never yields a false "clean". So certify is a cache, not a trust
override: fast when the file is untouched, correct when it isn't.

See also: [Certificate lifecycle](../concepts/certificate-lifecycle.md) · [Validate](../learn/validate.md).
