# Certify a clean file & skip re-validation

**Available in:** Python · Node · DuckDB · CLI · Browser

Mint an `.ags.idx` certificate from a clean validation, then reopen with it to
skip the rule engine.

=== "Python"

    ```python
    --8<-- "python/ex08_certify.py"
    ```

    ```text
    certified
    ```

    `.certify()` needs a prior **clean** [`.validate()`](../learn/validate.md)
    on the same handle — a passing verdict is the precondition for issuing a
    cert. It writes `<path>.ags.idx` next to the file: a validity certificate
    (the verdict plus a hash of the bytes it vouches for) and a byte-offset
    index of every group.

    Reopen with `read(path, index=...)` and the next `.validate()` resolves
    from the cert instead of running the numbered rules. You can see it took
    the fast path on `.report.resolution`: `"certified"` means the cert matched
    the file's current bytes, so the rule engine was skipped entirely. (A
    normal validate reports `"exact"` or a fallback edition — see
    [Validate](../learn/validate.md).)

=== "Node"

    ```js
    --8<-- "node/ex08_certify.mjs"
    ```

    ```text
    certified
    ```

    The same lifecycle, the same file format: `certify()` after a clean
    `validate()` mints `<path>.ags.idx`, and `read(path, { index })` +
    `validate()` resolves from it with `report.resolution === "certified"`.
    The cert wraps the one core `Sidecar`, so a Node-minted `.ags.idx` is
    byte-compatible with the ones Python, DuckDB and `lat-check` mint — any
    surface can consume any surface's cert.

=== "DuckDB"

    ```sql
    --8<-- "duckdb/ex08_certify.sql"
    ```

    `certify_ags()` validates and, on a clean result, writes the same
    `<path>.ags.idx` beside the file — one row back: `certified` (did it
    pass?), `groups` (how many were indexed), `errors`. Because the cert is the
    shared format, a nightly SQL job can pre-certify deliveries that Python or
    Node consumers then open on the fast path.

=== "CLI"

    ```bash
    --8<-- "cli/certify_emit_index.sh:cmd"
    ```

    ```text
    --8<-- "cli/certify_emit_index.out"
    ```

    `--emit-index` mints the certificate after the check comes back clean (a
    `note: certificate written to site.ags.idx` line reports where; point it
    elsewhere with `--index-out`). A dirty file gets no cert — the findings
    table and exit `1` come back instead.

=== "Browser"

    Validate a clean file in the [web app](../surfaces/browser.md) and it
    offers the `.ags.idx` certificate as a download — minted by the same wasm
    engine, byte-compatible with every other surface, so a file certified in
    the browser opens on the fast path in Python, Node, or DuckDB.

The cert is **content-bound**: if the file changes by a single byte, the hash
no longer matches and laterite silently falls back to a full validation — a
stale cert never yields a false "clean". So certify is a cache, not a trust
override: fast when the file is untouched, correct when it isn't.

See also: [Certificate lifecycle](../concepts/certificate-lifecycle.md) · [Validate](../learn/validate.md).
