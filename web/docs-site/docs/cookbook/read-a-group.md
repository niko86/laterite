# Read a group at the shell

**Available in:** Python · Node · DuckDB · CLI · [Browser](../surfaces/browser.md)

Dump one group's rows straight from a file — no engine run, no import. At the
shell, `lat read` gives you the raw cells as a table, CSV, or JSON; omit the
group to list the file's group codes.

=== "CLI"

    ```bash
    --8<-- "cli/read.sh:cmd"
    ```

    ```text
    --8<-- "cli/read.out"
    ```

    `lat read <file> <GROUP>` dumps the group's **raw file cells** — faithful to
    the bytes (`26.20` stays `26.20`, not `26.2`). `--csv` and `--json` are
    byte-identical whether you run the native binary, `uvx --from laterite lat`,
    or `npx laterite`; the default `--table` is a human view. Omit `<GROUP>` to
    list the file's group codes, one per line.

=== "Python"

    ```python
    import laterite

    loca = laterite.read("delivery.ags").table("LOCA")   # a born-typed frame
    print(loca)
    ```

    The library hands you a **born-typed** frame — numeric columns are real
    numbers, the typed complement to the CLI's raw dump. `lat read` in the Python
    console-script mirrors the binary's raw output for `--csv`/`--json`; reach for
    the library `table()` when you want the typed frame. See
    [Get a group as a typed frame](./get-typed-frame.md).

See also: [Get a typed frame](./get-typed-frame.md) · [Filter & select](./filter-select.md)
