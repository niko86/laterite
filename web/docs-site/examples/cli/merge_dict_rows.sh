# Docs CLI example — drop DICT rows that only restate the standard dictionary.
# The gate copies sample_site.ags under examples/; the script derives a second
# delivery carrying a DICT group whose one row redeclares LOCA_GL, a heading the
# standard dictionary already defines. Pruning leaves no row, so the merged file
# has no DICT group at all, which is what the group listing at the end shows;
# the listing of the input's DICT first is what makes the drop visible. The
# TRAN stamp is the one merge_deliveries.sh passes, so the output is about the
# DICT and not about a missing stamp. The page shows only the [start:cmd]
# section; the gate byte-compares stdout to merge_dict_rows.out (the merged
# file lands at examples/merged.ags).
# expect-exit: 0
cp examples/sample_site.ags examples/phase2.ags
printf '%s\r\n' \
  '' \
  '"GROUP","DICT"' \
  '"HEADING","DICT_TYPE","DICT_GRP","DICT_HDNG","DICT_STAT","DICT_DTYP","DICT_DESC","DICT_UNIT","DICT_EXMP","DICT_PGRP","DICT_REM","FILE_FSET"' \
  '"UNIT","","","","","","","","","","",""' \
  '"TYPE","PA","X","X","PA","PT","X","PU","X","X","X","X"' \
  '"DATA","HEADING","LOCA","LOCA_GL","OTHER","2DP","Ground level relative to datum","m","","","",""' \
  >> examples/phase2.ags
# --8<-- [start:cmd]
lat read examples/phase2.ags DICT --csv
lat merge examples/sample_site.ags examples/phase2.ags --out examples/merged.ags \
    --tran-issue 3 --tran-date 2024-01-15 \
    --tran-producer "Demo Producer" --tran-recipient "Demo Recipient" --tran-status Final \
    --dict-rows prune
lat read examples/merged.ags
# --8<-- [end:cmd]
