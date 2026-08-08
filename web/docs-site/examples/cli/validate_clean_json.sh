# Docs CLI example — `--json` on a CLEAN file: `findings` is an empty object,
# exit 0. Distinct from validate_json.sh, which shows the same flag over a file
# WITH findings; `learn/install.md` makes the empty-object point specifically,
# and the two outputs share no lines.
# expect-exit: 0
# --8<-- [start:cmd]
lat validate examples/sample_site.ags --json
# --8<-- [end:cmd]
