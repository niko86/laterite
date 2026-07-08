# Docs CLI example — `--fix` applies the SAFE mechanical repairs and writes the
# result where --fix-out points (or a sibling <file>.fixed.ags). The dirty file
# (a short DATA row, Rule 4) is minted here; the page shows only the command.
# expect-exit: 1
printf '%s\r\n' \
  '"GROUP","LOCA"' \
  '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"' \
  '"UNIT","","",""' \
  '"TYPE","ID","PA","2DP"' \
  '"DATA","BH01","BH"' \
  > delivery.ags
# --8<-- [start:cmd]
lat fix delivery.ags --fix-out repaired.ags
# --8<-- [end:cmd]
