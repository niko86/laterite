# Fix a dirty file

Mechanically repair a non-conforming AGS4 file — non-destructively — into a fresh handle.

```python
--8<-- "python/ex15_fix.py"
```

```text
pad_short_row
```

`.fix()` runs the safe-repair pass over a file you've already
[`read`](../learn/read.md) and returns a **new** `Ags4File` — the original handle
is left untouched (`fixed is not dirty`). What it changed rides on
`fixed.fix_report.applied`, a list of `{"kind": …}` records: here the lone `DATA`
row had only 3 fields where the `HEADING` declared 4, so the fixer padded it to
width and recorded `kind == "pad_short_row"`.

Read the kinds off the report to see every repair that was applied:

```python
kinds = [a["kind"] for a in fixed.fix_report.applied]
```

These are *safe* repairs — width/whitespace/structural defects that have one
unambiguous correction. Anything judgemental (a wrong value, a missing KEY) is
left for you; `.fix()` will not invent data. Because it returns the file, you
chain straight on — re-`.validate()` the fixed handle, or emit it.

## From the command line

`lat-check FILE --fix` is the CLI equivalent: it applies the same safe-repair
pass and writes the corrected file out, reporting the kinds it touched. Reach for
it when the file is the unit of work and you don't need the Python handle.

See also: [CLI reference](../reference/cli.md) · [Validate in Python](../learn/validate.md).
