import laterite
import polars as pl

# Build valid AGS4 from your own per-group frames — columns are the AGS headings.
proj = pl.DataFrame({"PROJ_ID": ["LAT-DEMO"], "PROJ_NAME": ["Demo site"]})
loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.50, 13.75]})

res = laterite.build_ags4({"PROJ": proj, "LOCA": loca})  # default mode="autofix"
groups = laterite.read(data=res.bytes).groups
print("groups:", groups)
print("findings:", len(res.findings))

# Autofix synthesizes the mandatory metadata catalogs (TRAN/UNIT/TYPE), so a
# data-only build is valid in one call.
assert {"PROJ", "LOCA", "TRAN", "UNIT", "TYPE"}.issubset(groups)
assert not res.findings
