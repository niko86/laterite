# Excel ↔ AGS4

**Available in:** Python · Node · CLI · [Browser](../surfaces/browser.md)

Round-trip an AGS4 delivery through a spreadsheet (one sheet per group, with the
headings / units / types as header rows) for hand-editing or sharing with
non-AGS tools, then back to a compliant `.ags`.

=== "CLI"

    ```bash
    --8<-- "cli/excel.sh:cmd"
    ```

    ```text
    --8<-- "cli/excel.out"
    ```

    `lat excel <in> <out>` picks the direction from the **output** extension:
    `.xlsx` exports (AGS4 → Excel), `.ags` imports (Excel → AGS4). Force it with
    `--export` / `--import` when the extension is ambiguous (the sheets + rows
    summary prints to stderr; here the round-trip is proved by reading a group back
    out). On import, `--no-format-numeric` leaves numeric-looking columns as text.

=== "Python"

    ```python
    import laterite

    laterite.to_excel("delivery.ags", "delivery.xlsx")      # one sheet per group
    laterite.from_excel("delivery.xlsx", "round-tripped.ags")
    ```

    `to_excel` / `from_excel` are the library face. Pass `groups=[...]` to export
    a subset, or `format_numeric_columns=False` on import to keep cells as text.
    See the [Python API](../reference/api.md).

See also: [Read a group](./read-a-group.md) · [Build from frames](./build-from-frames.md)
