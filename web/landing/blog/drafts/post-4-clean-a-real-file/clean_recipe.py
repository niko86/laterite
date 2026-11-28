"""Clean one real AGS4 file by hand, after `.fix()` has done the mechanical part.

A worked example, not a general cleaner: the steps are specific to one BGS
export of Crossrail boreholes at Farringdon (project file 202002061142130.ags).

    python clean_recipe.py <as-sent.ags> <cleaned.ags>

Fetch the file with laterite-quarry:
    quarry bgs pull farringdon --bbox -0.108,51.518,-0.102,51.522
"""
import sys
import laterite
import polars as pl

def count(a, label):
    r = a.validate().report
    rules = {k.replace("AGS Format Rule ", "R"): len(v) for k, v in sorted(r.by_rule().items()) if "Warning" not in k}
    print(f"{label:<34} {r.errors:>5}  {rules}")
    return r

raw = laterite.read(sys.argv[1]); count(raw, "as sent")
safe = raw.fix(); count(safe, "1. fix()")

frames = {g: safe[g] for g in safe.groups}
units = {g: dict(zip(safe.headings(g), safe.units(g))) for g in safe.groups}
types = {g: dict(zip(safe.headings(g), safe.types(g))) for g in safe.groups}
def rebuild(label):
    built = laterite.build_ags4(frames, units=units, types=types, mode="report")
    out = laterite.read(built.text); return out, count(out, label)

# 2. rows that say the same thing twice
frames["ABBR"] = frames["ABBR"].unique(subset=["ABBR_HDNG", "ABBR_CODE"], keep="first", maintain_order=True)
for g in ("BKFL", "WSTG"):
    frames[g] = frames[g].unique(maintain_order=True)
rebuild("2. duplicates removed")

# 3. dates are ISO but declared dd/mm/yyyy: correct the declaration
for h in ("LOCA_STAR", "LOCA_ENDD"):
    units["LOCA"][h] = "yyyy-mm-dd"
rebuild("3. date unit corrected")

# 4. units: describe or drop
used = {u for g in units.values() for u in g.values() if u}
u = frames["UNIT"]
blank = u.filter(pl.col("UNIT_DESC").is_null() | (pl.col("UNIT_DESC") == ""))["UNIT_UNIT"].to_list()
print("   blank-description units used in this file:", [x for x in blank if x in used])
frames["UNIT"] = u.filter(~pl.col("UNIT_UNIT").is_in([x for x in blank if x not in used]))
out, rep = rebuild("4. unused, undescribed units dropped")
print("   R10a now:", rep.findings.filter(pl.col("rule") == "AGS Format Rule 10a").select("group", "line", "desc").rows())

# 5. the record-link characters the file never declared, so "CP+RC" reads as CP and RC
frames["TRAN"] = frames["TRAN"].with_columns(TRAN_DLIM=pl.lit("|"), TRAN_RCON=pl.lit("+"))
units["TRAN"] |= {"TRAN_DLIM": "", "TRAN_RCON": ""}; types["TRAN"] |= {"TRAN_DLIM": "X", "TRAN_RCON": "X"}
rebuild("5. TRAN_DLIM and TRAN_RCON declared")

# 6. codes used but never defined
def add_rows(group, rows):
    frames[group] = pl.concat([frames[group], pl.DataFrame(rows, schema=frames[group].schema)], how="vertical")
blank = dict.fromkeys(frames["ABBR"].columns)
add_rows("ABBR", [
    {**blank, "ABBR_HDNG": "DICT_TYPE", "ABBR_CODE": "HEADING", "ABBR_DESC": "Flag: heading"},
    {**blank, "ABBR_HDNG": "DICT_STAT", "ABBR_CODE": "KEY", "ABBR_DESC": "Key"},
    {**blank, "ABBR_HDNG": "DICT_STAT", "ABBR_CODE": "OTHER", "ABBR_DESC": "Other"},
    {**blank, "ABBR_HDNG": "GEOL_LEG", "ABBR_CODE": "407", "ABBR_DESC": "Legend code 407 (undefined in the source)"},
])
# and the units laterite supplied from the dictionary where the file's UNIT row was blank
add_rows("UNIT", [
    {"UNIT_UNIT": "bar", "UNIT_DESC": "bar"},
    {"UNIT_UNIT": "hh:mm:ss", "UNIT_DESC": "hours minutes seconds"},
    {"UNIT_UNIT": "yyyy-mm-ddThh:mm", "UNIT_DESC": "year month day hours minutes"},
])
rebuild("6. codes and units defined")

# 7. BGS's added columns: give them conforming names and say what they are
renames = {"PROJ": {"BGS_PROJ_ID": "PROJ_BGS"}, "LOCA": {"BGS_LOCA_ID": "LOCA_BGS"},
           "BKFL": {"BKFL_TOP_TEXT": "BKFL_TOPT", "BKFL_BASE_TEXT": "BKFL_BAST"}, "CDIA": {"CDIA_DIAM_TEXT": "CDIA_DIAT"}}
said = {"PROJ_BGS": "BGS project identifier", "LOCA_BGS": "BGS location identifier", "BKFL_TOPT": "Top of backfill, as text",
        "BKFL_BAST": "Base of backfill, as text", "CDIA_DIAT": "Casing diameter, as text"}
dblank = dict.fromkeys(frames["DICT"].columns)
for g, m in renames.items():
    m = {a: b for a, b in m.items() if a in frames[g].columns}
    frames[g] = frames[g].rename(m)
    for a, b in m.items():
        units[g][b] = units[g].pop(a); types[g][b] = types[g].pop(a)
        add_rows("DICT", [{**dblank, "DICT_TYPE": "HEADING", "DICT_GRP": g, "DICT_HDNG": b, "DICT_STAT": "OTHER", "DICT_DTYP": "X", "DICT_DESC": said[b]}])
out, rep = rebuild("7. BGS columns renamed and declared")

# 8. a stale dictionary row: BKFL's key was HOLE_ID in AGS3; the file now uses LOCA_ID
frames["DICT"] = frames["DICT"].filter(~((pl.col("DICT_GRP") == "BKFL") & (pl.col("DICT_HDNG") == "HOLE_ID")))
out, rep = rebuild("8. stale DICT key row removed")
for row in rep.findings.filter(~pl.col("rule").str.contains("Warning")).select("rule", "group", "line", "desc").rows(): print("   left:", row)
print("   warnings:", rep.warnings)
out.save(sys.argv[2]); print("   saved", sys.argv[2])

# what changed, by laterite's own key-aware diff, and what did not
d = raw.diff(out)
print("   diff:", type(d).__name__, [a for a in dir(d) if not a.startswith("_")][:14])
for g in ("LOCA", "GEOL", "SAMP", "ISPT"):
    print("   rows", g, raw[g].height, "->", out[g].height, "| same values:", raw[g].drop([c for c in raw[g].columns if c.startswith("BGS_")]).equals(out[g].drop([c for c in out[g].columns if c.endswith("_BGS")])))
