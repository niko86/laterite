import laterite
import polars as pl

# Build valid AGS4 from your own per-group frames — columns are the AGS headings.
proj = pl.DataFrame({"PROJ_ID": ["LAT-DEMO"], "PROJ_NAME": ["Demo site"]})
loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.50, 13.75]})

res = laterite.build_ags4({"PROJ": proj, "LOCA": loca})  # default mode="autofix"
groups = laterite.read(data=res.bytes).groups
print("groups:", groups)
print("findings:", len(res.findings))

# You get back exactly the groups you supplied. AGS4 also mandates the metadata
# catalogs (TRAN/UNIT/TYPE), which your frames don't carry — so those are
# REPORTED rather than invented:
assert set(groups) == {"PROJ", "LOCA"}
assert {f["rule"] for f in res.findings} >= {
    "AGS Format Rule 14",  # TRAN
    "AGS Format Rule 15",  # UNIT
    "AGS Format Rule 17",  # TYPE
}

# Ask for them and UNIT and TYPE are derived from your columns. TRAN is not
# derivable — only you know who sent what to whom — so you state it, and a build
# that doesn't reports the gap instead of inventing a placeholder that would
# satisfy the rule while asserting a transmission that never happened. Opt-in
# either way, so nothing appears in your file that you didn't ask for.
full = laterite.build_ags4(
    {"PROJ": proj, "LOCA": loca},
    synthesise_metadata=True,
    tran_issue="1",
    tran_date="2026-07-30",
    tran_producer="Demo Producer",
    tran_recipient="Demo Recipient",
    tran_status="Final",
)
assert {"PROJ", "LOCA", "TRAN", "UNIT", "TYPE"}.issubset(
    laterite.read(data=full.bytes).groups
)
assert not full.findings
