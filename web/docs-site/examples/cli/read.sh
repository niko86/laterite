# Docs CLI example — dump one group's rows as CSV (raw file cells). The gate
# copies sample_site.ags under examples/; byte-compares stdout to read.out.
# expect-exit: 0
# --8<-- [start:cmd]
lat read examples/sample_site.ags LOCA --csv
# --8<-- [end:cmd]
