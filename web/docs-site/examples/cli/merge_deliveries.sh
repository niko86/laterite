# Docs CLI example — reconcile two deliveries of one project into one file.
# The gate copies sample_site.ags under examples/; the script derives a second
# delivery (one PROJ_NAME edit, so merge has a revision to report) and merges the
# two, stamping the merged file's own transmission (--tran-issue/--tran-date).
# The page shows only the [start:cmd] section; the gate byte-compares stdout to
# merge_deliveries.out (the merged file lands at examples/merged.ags).
# expect-exit: 0
sed 's/synthetic starter - replace me/phase 2/' examples/sample_site.ags > examples/phase2.ags
# --8<-- [start:cmd]
lat merge examples/sample_site.ags examples/phase2.ags --out examples/merged.ags \
    --tran-issue 3 --tran-date 2024-01-15
# --8<-- [end:cmd]
