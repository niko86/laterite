# Docs CLI example — package a file for transport, restore it, and prove the
# round-trip is lossless by reading a group back. Pack/unpack print their summary
# to stderr; the gate byte-compares stdout to transport.out.
# expect-exit: 0
# --8<-- [start:cmd]
lat pack examples/sample_site.ags site.ags.zst   # → compressed (summary on stderr)
lat unpack site.ags.zst restored.ags             # → restored, byte-identical
lat read restored.ags | head -3                  # prove it: the first group codes
# --8<-- [end:cmd]
