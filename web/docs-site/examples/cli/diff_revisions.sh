# Docs CLI example — compare two revisions: the KEY-aware/type-aware delta.
# The gate copies sample_site.ags under examples/; the script derives a second
# revision (one PROJ_NAME edit) so the page's command has something to diff
# against. The page shows only the [start:cmd] section; the gate byte-compares
# stdout to diff_revisions.out.
# expect-exit: 0
sed 's/synthetic starter - replace me/Rev B/' examples/sample_site.ags > examples/rev.ags
# --8<-- [start:cmd]
lat diff examples/sample_site.ags examples/rev.ags
# --8<-- [end:cmd]
