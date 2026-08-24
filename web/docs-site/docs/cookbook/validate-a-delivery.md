# Validate a delivery

**Available in:** Python · Node · CLI · [Browser](../surfaces/browser.md)

Run the numbered AGS Format Rules over a transfer file and get a verdict:
in-process from Python or Node, in SQL from DuckDB, at the shell with
`lat`, or drag-and-drop in the browser.

=== "Python"

    ```python
    --8<-- "python/ex02_validate.py:code"
    ```

    ```text
    --8<-- "python/ex02_validate.out"
    ```

    `read(path).validate()` runs the rule engine and returns the
    [`Ags4File`](../learn/read.md), so it chains; the verdict rides on `.report`.
    `is_valid` is the headline verdict and reflects errors only, so a file whose
    findings are all warnings passes with a non-zero `count`. `dict_version` and
    `resolution` tell you *which* edition the rules came from. Both read straight
    off the file: the edition is taken from `TRAN_AGS`, and `resolution='exact'` means
    that edition was matched on the nose (otherwise laterite falls back to the
    nearest dictionary it ships). You never pass an edition.

    Because `.validate()` hands the file back, keep going on the same handle
    without re-reading from disk: query it, slice a group, or emit it.

=== "Node"

    ```js
    --8<-- "node/ex02_validate.mjs"
    ```

    ```text
    --8<-- "node/ex02_validate.out"
    ```

    Same engine, camelCase verbs: the free `validate()` returns a `Report`
    whose `isValid` / `count` / `dictVersion` / `resolution` mirror the Python
    properties one-for-one, and `report.toJson()` is byte-identical to
    `lat validate --json`, so a Node service and a CI gate can share downstream
    tooling. For the chaining style, `read(path).validate()` returns the
    [`Ags4File`](../node/index.md) with the verdict on `.report`, exactly the
    Python shape.

=== "DuckDB"

    The `laterite_ags4` DuckDB extension is a **read-only reader** with no
    `validate` function. Run the numbered rules with the [`lat`
    CLI](../reference/cli.md) or the `laterite` library (the Python/Node tabs
    above); the extension then *reads* the file and can *consume* an
    externally-minted `.ags.idx` (from `lat certify`) for fast single-group
    reads. See the [DuckDB function reference](../reference/duckdb-functions.md).

=== "CLI"

    A clean file prints one line and exits `0`:

    ```bash
    --8<-- "cli/validate_clean.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_clean.out"
    ```

    A file with problems prints a count line, then a findings table (one row
    per violation), and exits `1`. File-level findings (a missing group) carry
    no line, shown as `-`:

    ```bash
    --8<-- "cli/validate_findings.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_findings.out"
    ```

    Add `--json` for machine-readable findings keyed by rule. The exit code is
    unchanged (`0` when the file passed, `1` when it did not), so a CI gate can
    branch without parsing stdout. Note that the two are separate answers: only
    errors fail a run, so `findings` can be populated on an exit `0`. Add
    `--warnings-as-errors` if the gate should fail on warnings too:

    ```bash
    --8<-- "cli/validate_json.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_json.out"
    ```

Every validating door runs the same rule engine and picks the dictionary
edition from the file's `TRAN_AGS`; the Python/Node `report` and the CLI's
`--json` carry identical findings. Reach for the CLI in a build gate (exit `1`
fails the build); reach for `.validate()` when you want to keep working with
the parsed file in the same process.

How strict that gate is, is your call: see [severity
tiers](../concepts/severity-tiers.md) for the two dials.

See also: [lat CLI](../reference/cli.md) · [Certify a file](./certify.md)
