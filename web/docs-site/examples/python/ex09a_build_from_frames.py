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

# Ask for them and they're derived from your data — UNIT and TYPE from the
# columns, TRAN as a placeholder you overwrite. Opt-in, so nothing appears in
# your file that you didn't ask for.
full = laterite.build_ags4({"PROJ": proj, "LOCA": loca}, synthesise_metadata=True)
assert {"PROJ", "LOCA", "TRAN", "UNIT", "TYPE"}.issubset(
    laterite.read(data=full.bytes).groups
)
assert not full.findings
