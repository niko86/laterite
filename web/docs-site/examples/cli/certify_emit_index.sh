# Docs CLI example — after a clean check, --emit-index mints the <file>.ags.idx
# certificate beside the file (the same cert Python/Node mint via .certify()).
# The file is copied here so the cert never lands beside the shared fixture.
# expect-exit: 0
cp examples/sample_site.ags site.ags
# --8<-- [start:cmd]
lat certify site.ags
# --8<-- [end:cmd]
