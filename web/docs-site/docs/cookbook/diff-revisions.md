# Diff two revisions

**Available in:** Python · Node · CLI · Browser

**When:** a resubmission lands and you need to know _what actually changed_ between
Rev A and Rev B — not a line diff, but a KEY-aware, type-aware delta.

=== "Python"

    ```python
    --8<-- "python/ex16_diff.py"
    ```

    ```text
    --8<-- "python/ex16_diff.out"
    ```

    `laterite.diff(a, b)` compares two AGS4 texts and returns a `RevisionDelta` — a
    per-group breakdown plus the `total_added` / `total_removed` / `total_changed`
    counts. It is **not** a text diff: rows are matched on each group's dictionary
    **KEY headings** (here `PROJ_ID`), so a row that moved or was reordered still
    lines up with its counterpart. The single edit above registers as one *changed*
    row keyed on `["LAT-DEMO"]`, carrying a cell that names the heading, its AGS
    `type`, and the `a`/`b` values.

    Because cells are compared through the [born-typed](../concepts/born-typed.md)
    value, only a genuine quantity change registers — `1.50` vs `1.5` on a `2DP`
    column is the same number and produces **no** delta, where a naive line diff
    would flag it.

    Walk the structure to drive a review: each `group` in `delta["groups"]` carries
    `code`, `key_headings`, `keyed` (whether the group has KEYs to match on), and a
    `rows` list. Each row has a `kind` (`added` / `removed` / `changed`), its `key`,
    and — for a changed row — a `cells` list of `{heading, type, a, b}`:

    ```python
    for group in delta["groups"]:
        for row in group["rows"]:
            if row["kind"] == "changed":
                for cell in row["cells"]:
                    print(group["code"], row["key"], cell["heading"], cell["a"], "→", cell["b"])
    ```

    **Gotcha:** matching is only as precise as the KEYs. A group with no KEY headings
    (`keyed` is `False`) falls back to positional comparison, so a genuine row
    insertion there can read as a cascade of *changed* rows rather than one *added*.
    Check `keyed` before trusting row identity on KEY-less groups.

=== "Node"

    ```js
    --8<-- "node/ex16_diff.mjs"
    ```

    ```text
    --8<-- "node/ex16_diff.out"
    ```

    `diff(a, b)` runs the same `laterite-ags4-diff` engine and returns the
    **byte-identical** `RevisionDelta` — the snake_case field names (`total_changed`,
    `key_headings`, `keyed`) match Python one-for-one, so the same walk drives a
    review in either language. One surface note: a bare `string` is a **path** in
    Node (`diff("a.ags", "b.ags")` compares two files), so pass a `Buffer` /
    `Uint8Array` — as above — when the revision only exists in memory. It's
    synchronous; no DuckDB peer needed.

=== "CLI"

    `lat diff <file> <other>` compares two revisions on disk and prints
    the per-group delta, exiting `0`:

    ```bash
    --8<-- "cli/diff_revisions.sh:cmd"
    ```

    ```text
    --8<-- "cli/diff_revisions.out"
    ```

    The CLI reports the **summary** shape — per group, `+added −removed ~changed`
    — off the same KEY-aware, type-aware engine, so the counts agree with every
    other surface. When you need the cell-level `a`/`b` detail shown in the
    Python/Node tabs, reach for `diff()` in a script.

=== "Browser"

    Open the [web app](../surfaces/browser.md)'s revision-diff tool and drop in
    Rev A and Rev B. The same KEY-aware, type-aware engine runs compiled to
    WebAssembly, entirely client-side — you get the per-group added/removed/changed
    breakdown with each changed cell's `a`/`b` values, and neither file leaves your
    machine.

Every door runs the same diff leaf and reports the same delta, so a `1.50` vs
`1.5` non-change stays a non-change everywhere. Reach for `lat diff` in a
resubmission gate, `diff()` in a pipeline when you need the cell-level detail, or
the browser tool for a quick visual compare.

See also: [Certify a file](./certify.md) ·
[Born typed](../concepts/born-typed.md)
