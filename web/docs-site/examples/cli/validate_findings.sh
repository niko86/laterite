# Docs CLI example — a delivery with problems: count line + findings table,
# exit 1. The dirty file is minted here (CRLF, per AGS4 Rule 2a) so the gate
# runs hermetically; the page shows only the command between the markers.
# expect-exit: 1
printf '%s\r\n' \
  '"GROUP","PROJ"' \
  '"HEADING","PROJ_ID","PROJ_NAME"' \
  '"UNIT","",""' \
  '"TYPE","ID","X"' \
  '"DATA","121415"' \
  > delivery.ags
# --8<-- [start:cmd]
lat validate delivery.ags
# --8<-- [end:cmd]
