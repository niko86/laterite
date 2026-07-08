# Docs CLI example — export AGS4 to an Excel workbook, import it back, and prove
# the round-trip by reading a group. Direction is inferred from the output
# extension (.xlsx ⇒ export, .ags ⇒ import). The gate byte-compares stdout.
# expect-exit: 0
# --8<-- [start:cmd]
lat excel examples/sample_site.ags site.xlsx   # → Excel workbook (summary on stderr)
lat excel site.xlsx round-tripped.ags          # → back to AGS4 (import)
lat read round-tripped.ags | head -3           # prove it: the first group codes
# --8<-- [end:cmd]
