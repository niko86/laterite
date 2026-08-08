# Docs CLI example — the same file as validate_warning_tier.sh, read at the
# errors-only tier: the warning disappears, the verdict is clean and the exit
# code flips from 1 to 0. That flip is the whole point of the page, and it is
# asserted here by `expect-exit` rather than described in prose.
# expect-exit: 0
sed 's/,"4\.1\.1",/,"4.9.9",/' examples/sample_site.ags > examples/sample_site_edition.ags
# --8<-- [start:cmd]
lat validate --no-warnings examples/sample_site_edition.ags
# --8<-- [end:cmd]
