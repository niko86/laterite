# Docs CLI example — the same file as validate_warning_tier.sh, read at the
# errors-only tier: the warning DISAPPEARS from the report. The exit code is 0
# either way, because a warning has never decided the verdict since #321 — this
# flag controls what you SEE, and `--warnings-as-errors` is the one that
# controls what FAILS. The pair exists to show those are two dials, not one.
# expect-exit: 0
sed 's/,"4\.1\.1",/,"4.9.9",/' examples/sample_site.ags > examples/sample_site_edition.ags
# --8<-- [start:cmd]
lat validate --no-warnings examples/sample_site_edition.ags
# --8<-- [end:cmd]
