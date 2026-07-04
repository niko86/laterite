# Docs CLI example — machine-readable findings keyed by rule; the exit code is
# unchanged (0 empty findings / 1 populated) so a CI gate can branch without
# parsing stdout. Same minted dirty file as validate_findings.sh.
# expect-exit: 1
printf '%s\r\n' \
  '"GROUP","PROJ"' \
  '"HEADING","PROJ_ID","PROJ_NAME"' \
  '"UNIT","",""' \
  '"TYPE","ID","X"' \
  '"DATA","121415"' \
  > delivery.ags
# --8<-- [start:cmd]
lat-check delivery.ags --json
# --8<-- [end:cmd]
