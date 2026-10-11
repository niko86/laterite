# Docs CLI example — rewrite an ABBR code one delivery spelled its own way.
# The gate copies sample_site.ags under examples/; the script derives a second
# delivery whose producer coded cable-percussion boreholes `SA` where the first
# delivery and the edition's standard list say `CP`, in the ABBR row and in every
# LOCA row that used it. LOCA_TYPE is not a KEY heading, so the rewrite cannot
# collide two rows — a SAMP_TYPE rewrite would, because SAMP_TYPE is part of
# SAMP's KEY, and merge refuses that (exit 6). The mapping file is written in the
# [start:cmd] section so the page shows its shape; it lands at
# examples/recode.json. The ABBR listing before and after is what makes the
# rewrite visible: `SA` is in the input and not in the output. The TRAN stamp
# is the one merge_deliveries.sh passes, so the output is about the code and
# not about a missing stamp. The gate byte-compares stdout to merge_recode.out
# (the merged file lands at examples/merged.ags).
# expect-exit: 0
sed -e 's/"LOCA_TYPE","CP","Cable percussion boring"/"LOCA_TYPE","SA","Shell and auger boring"/' \
    -e 's/,"CP",/,"SA",/' \
    examples/sample_site.ags > examples/phase2.ags
# --8<-- [start:cmd]
lat read examples/phase2.ags ABBR --csv
printf '%s\n' '{"LOCA_TYPE": {"SA": "CP"}}' > examples/recode.json
lat merge examples/sample_site.ags examples/phase2.ags --out examples/merged.ags \
    --tran-issue 3 --tran-date 2024-01-15 \
    --tran-producer "Demo Producer" --tran-recipient "Demo Recipient" --tran-status Final \
    --recode examples/recode.json
lat read examples/merged.ags ABBR --csv
# --8<-- [end:cmd]
