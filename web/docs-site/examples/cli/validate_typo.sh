# Docs CLI example — one broken value: a findings table with a single Rule 8
# row, exit 1. `learn/install.md` documented this table by hand and it had
# drifted into the WRONG RENDERER: plain ASCII pipes (`Rule | Line | Group`),
# which is what the wheel's Python `lat` prints, on a page documenting the Rust
# binary — which draws comfy-table box-drawing. The row's content was right, so
# somebody ran it once; nothing re-ran it afterwards.
#
# `sed`, not a Python rewrite: the fixture is CRLF (AGS4 Rule 2a) and sed keeps
# the \r on the line, where read_text()/write_text() would normalise it and turn
# one finding into 157 Rule-2a violations.
# expect-exit: 1
sed 's/451105.75/not-a-number/' examples/sample_site.ags > examples/sample_site_typo.ags
# --8<-- [start:cmd]
lat validate examples/sample_site_typo.ags
# --8<-- [end:cmd]
