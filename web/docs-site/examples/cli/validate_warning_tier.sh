# Docs CLI example — the WARNING tier: a file whose only blemish is an
# out-of-range TRAN_AGS edition carries one warning and no errors, exit 1.
# `concepts/severity-tiers.md` hand-wrote this table (and its `--no-warnings`
# twin) against a placeholder `site.ags`, in plain ASCII pipes rather than the
# box-drawing the Rust binary actually emits.
#
# The edition string is unique in the fixture, so a targeted sed cannot hit a
# DICT row or a date by accident; sed also keeps the CRLF that AGS4 Rule 2a
# requires, where a read_text()/write_text() rewrite would strip it and bury the
# one interesting finding under a Rule 2a per line.
# expect-exit: 1
sed 's/,"4\.1\.1",/,"4.9.9",/' examples/sample_site.ags > examples/sample_site_edition.ags
# --8<-- [start:cmd]
lat validate examples/sample_site_edition.ags
# --8<-- [end:cmd]
