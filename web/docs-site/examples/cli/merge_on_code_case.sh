# Docs CLI example — settle an ABBR code two deliveries spell in different
# letter case. The gate copies sample_site.ags under examples/; the script
# derives a second delivery that writes BH01's LOCA_TYPE, and the ABBR row
# defining it, as `cp` where the first delivery and the edition's standard list
# say `CP`. `CP` is the one standard code matching the pair ignoring case, so
# `standard` rewrites `cp` to it; the warning names the rewrite and the ABBR
# listing shows one row where `keep` would leave two. The TRAN stamp is the one
# merge_deliveries.sh passes, so the output is about the code and not about a
# missing stamp. The page shows only the [start:cmd] section; the gate
# byte-compares stdout to merge_on_code_case.out (the merged file lands at
# examples/merged.ags).
# expect-exit: 0
sed -e 's/"LOCA_TYPE","CP"/"LOCA_TYPE","cp"/' -e 's/"BH01","CP"/"BH01","cp"/' \
    examples/sample_site.ags > examples/phase2.ags
# --8<-- [start:cmd]
lat merge examples/sample_site.ags examples/phase2.ags --out examples/merged.ags \
    --tran-issue 3 --tran-date 2024-01-15 \
    --tran-producer "Demo Producer" --tran-recipient "Demo Recipient" --tran-status Final \
    --on-code-case standard
lat read examples/merged.ags ABBR --csv
# --8<-- [end:cmd]
