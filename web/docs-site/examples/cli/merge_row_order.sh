# Docs CLI example — write the merged rows in KEY order rather than first-seen
# order. The gate copies sample_site.ags under examples/; the script derives a
# second delivery that adds one borehole, BH03A, whose ID sorts between BH03 and
# BH04. In first-seen order it would land after BH14, because every row of the
# first delivery is seen before any row only the second one has — so this is the
# shape where the two orders differ. The TRAN stamp is the one
# merge_deliveries.sh passes, so the output is about the order and not about a
# missing stamp. The page shows only the [start:cmd] section; the gate
# byte-compares stdout to merge_row_order.out (the merged file lands at
# examples/merged.ags).
# expect-exit: 0
awk '{ print } /^"DATA","BH03","RC"/ { printf "\"DATA\",\"BH03A\",\"RC\",\"451352.10\",\"161994.02\",\"28.61\",\"30.10\"\r\n" }' \
    examples/sample_site.ags > examples/phase2.ags
# --8<-- [start:cmd]
lat merge examples/sample_site.ags examples/phase2.ags --out examples/merged.ags \
    --tran-issue 3 --tran-date 2024-01-15 \
    --tran-producer "Demo Producer" --tran-recipient "Demo Recipient" --tran-status Final \
    --row-order key
lat read examples/merged.ags LOCA --csv
# --8<-- [end:cmd]
