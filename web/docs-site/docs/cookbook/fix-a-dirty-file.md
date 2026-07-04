# Fix a dirty file

**Available in:** Python · Node · CLI · Browser

Mechanically repair a non-conforming AGS4 file — non-destructively — into a
fresh handle.

=== "Python"

    ```python
    --8<-- "python/ex15_fix.py"
    ```

    ```text
    pad_short_row
    ```

    `.fix()` runs the safe-repair pass over a file you've already
    [`read`](../learn/read.md) and returns a **new** `Ags4File` — the original
    handle is left untouched (`fixed is not dirty`). What it changed rides on
    `fixed.fix_report.applied`, a list of `{"kind": …}` records: here the lone
    `DATA` row had only 3 fields where the `HEADING` declared 4, so the fixer
    padded it to width and recorded `kind == "pad_short_row"`.

    Read the kinds off the report to see every repair that was applied:

    ```python
    kinds = [a["kind"] for a in fixed.fix_report.applied]
    ```

    Because it returns the file, you chain straight on — re-`.validate()` the
    fixed handle, or emit it.

=== "Node"

    ```js
    --8<-- "node/ex15_fix.mjs"
    ```

    ```text
    pad_short_row
    ```

    Same behaviour, camelCase: `.fix()` returns a **new**
    [`Ags4File`](../node/index.md) and the record of what changed rides on
    `.fixReport.applied` (`{kind, label, rule, line?, risk}` per fix), with
    what *couldn't* be mechanically repaired left in `.fixReport.findings`.
    Write the repaired bytes out with `fixed.save(path)`, or keep chaining.
    The free-function form `fix(path)` returns the `FixResult` directly when
    you don't need the handle. Pass `{ risky: true }` to add the
    intent-guessing repairs on top of the safe pass.

=== "CLI"

    ```bash
    --8<-- "cli/fix_dirty.sh:cmd"
    ```

    ```text
    --8<-- "cli/fix_dirty.out"
    ```

    `--fix` applies the same safe-repair pass and writes the repaired file
    where `--fix-out` points (omit it for a sibling `<file>.fixed.ags`, or use
    `--in-place` to overwrite the source). The summary names each applied kind
    and counts what remains for a human — here the padded row was fixed, while
    the missing `TRAN`/`UNIT`/`TYPE` groups can't be invented, so the exit code
    stays `1`. Add `--fix-risky` for the intent-guessing tier.

=== "Browser"

    Open the [web app](../surfaces/browser.md) and drop your file on the
    **Fix** pane. It computes the available repairs, shows a before/after diff
    so you can review each one, and hands back the repaired file as a download
    — the same safe-fix engine as every other surface, running entirely in
    your browser.

These are *safe* repairs — width/whitespace/structural defects that have one
unambiguous correction. Anything judgemental (a wrong value, a missing KEY) is
left for you; fix will not invent data.

See also: [CLI reference](../reference/cli.md) · [Validate a delivery](./validate-a-delivery.md)
