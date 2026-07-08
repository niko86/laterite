# Validate a delivery

**Available in:** Python · Node · DuckDB · CLI · Browser

Run the numbered AGS Format Rules over a transfer file and get a verdict —
in-process from Python or Node, in SQL from DuckDB, at the shell with
`lat`, or drag-and-drop in the browser.

=== "Python"

    ```python
    --8<-- "python/ex02_validate.py"
    ```

    ```text
    is_valid=True count=0 dict_version='4.1.1' resolution='exact'
    ```

    `read(path).validate()` runs the rule engine and returns the
    [`Ags4File`](../learn/read.md), so it chains; the verdict rides on `.report`.
    `is_valid` and `count` are the headline. `dict_version` and `resolution`
    tell you *which* edition the rules came from — both read straight off the
    file: the edition is taken from `TRAN_AGS`, and `resolution='exact'` means
    that edition was matched on the nose (otherwise laterite falls back to the
    nearest dictionary it ships). You never pass an edition.

    Because `.validate()` hands the file back, keep going on the same handle —
    query it, slice a group, or emit it — without re-reading from disk.

=== "Node"

    ```js
    --8<-- "node/ex02_validate.mjs"
    ```

    ```text
    isValid=true count=0 dictVersion=4.1.1 resolution=exact
    ```

    Same engine, camelCase verbs: the free `validate()` returns a `Report`
    whose `isValid` / `count` / `dictVersion` / `resolution` mirror the Python
    properties one-for-one, and `report.toJson()` is byte-identical to
    `lat validate --json`, so a Node service and a CI gate can share downstream
    tooling. For the chaining style, `read(path).validate()` returns the
    [`Ags4File`](../node/index.md) with the verdict on `.report` — exactly the
    Python shape.

=== "DuckDB"

    ```sql
    --8<-- "duckdb/_install.sql"
    ```

    ```sql
    --8<-- "duckdb/ex02_validate.sql"
    ```

    `validate_ags()` is a table function: **one row per finding**, so a clean
    file returns zero rows and the verdict composes in plain SQL —
    `count(*) = 0` is your gate, `WHERE rule = '4'` is your triage. `group` and
    `desc` are quoted because they're SQL keywords. The edition is picked from
    the file's `TRAN_AGS` exactly as on every other surface; pass
    `dict_version := '4.2'` only to override it. See the
    [DuckDB function reference](../reference/duckdb-functions.md).

=== "CLI"

    A clean file prints one line and exits `0`:

    ```bash
    --8<-- "cli/validate_clean.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_clean.out"
    ```

    A file with problems prints a count line, then a findings table — one row
    per violation — and exits `1`. File-level findings (a missing group) carry
    no line, shown as `-`:

    ```bash
    --8<-- "cli/validate_findings.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_findings.out"
    ```

    Add `--json` for machine-readable findings keyed by rule. The exit code is
    unchanged — `0` for an empty `findings`, `1` when it's populated — so a CI
    gate can branch without parsing stdout:

    ```bash
    --8<-- "cli/validate_json.sh:cmd"
    ```

    ```text
    --8<-- "cli/validate_json.out"
    ```

=== "Browser"

    Open the [web app](../surfaces/browser.md) and drag your file into the
    **Validate** pane. The same numbered-rules engine runs compiled to
    WebAssembly, entirely client-side — your file never leaves your machine,
    which makes it safe for confidential ground-investigation data. Findings
    arrive grouped by rule with the resolved edition and severity tiers, and
    you can carry the same file straight into the **Fix** or **Explore** panes.

Every door runs the same rule engine and picks the dictionary edition from the
file's `TRAN_AGS` — the Python/Node `report`, DuckDB's rows, and the CLI's
`--json` carry identical findings. Reach for the CLI in a build gate (exit `1`
fails the build); reach for `.validate()` when you want to keep working with
the parsed file in the same process.

See also: [lat CLI](../reference/cli.md) · [Certify a file](./certify.md)
