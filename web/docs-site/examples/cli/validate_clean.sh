# Docs CLI example — a clean file: one line, exit 0. The page shows only the
# command between the snippet markers; the gate runs the whole script from a
# temp dir with the fixture copied under examples/ (so the path text is real).
# expect-exit: 0
# --8<-- [start:cmd]
lat validate examples/sample_site.ags
# --8<-- [end:cmd]
