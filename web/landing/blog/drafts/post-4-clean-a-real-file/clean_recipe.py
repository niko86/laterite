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

Frames = dict[str, pl.DataFrame]
PerHeading = dict[str, dict[str, str]]


def report(ags: laterite.Ags4File, label: str) -> laterite.Report:
    """Print the error count and the findings per rule, and hand the report back."""
    found = ags.validate().report
    per_rule = {
        rule.replace("AGS Format Rule ", "R"): len(rows)
        for rule, rows in sorted(found.by_rule().items())
        if "Warning" not in rule
    }
    print(f"{label:<38} {found.errors:>5}  {per_rule}")
    return found


def rebuild(frames: Frames, units: PerHeading, types: PerHeading) -> laterite.Ags4File:
    """Write the groups back out as AGS4, keeping the source's own UNIT and TYPE rows.

    Left to itself `build_ags4` fills UNIT and TYPE from the dictionary, which
    is right for data you authored and wrong for a file whose declarations you
    mean to keep.
    """
    built = laterite.build_ags4(frames, units=units, types=types, mode="report")
    return laterite.read(built.text)


def add_rows(frames: Frames, group: str, rows: list[dict[str, str | None]]) -> None:
    """Append rows to a group, leaving every heading not named blank."""
    blank = dict.fromkeys(frames[group].columns)
    full = [{**blank, **row} for row in rows]
    extra = pl.DataFrame(full, schema=frames[group].schema)
    frames[group] = pl.concat([frames[group], extra], how="vertical")


def main(source: str, target: str) -> None:
    raw = laterite.read(source)
    report(raw, "as sent")

    # 1. The mechanical part. On this file that is the line endings, which is
    #    most of what the validator found.
    safe = raw.fix()
    report(safe, "1. fix()")

    frames: Frames = {g: safe[g] for g in safe.groups}
    units: PerHeading = {
        g: dict(zip(safe.headings(g), safe.units(g), strict=True)) for g in safe.groups
    }
    types: PerHeading = {
        g: dict(zip(safe.headings(g), safe.types(g), strict=True)) for g in safe.groups
    }

    # 2. Rows that say the same thing twice. ABBR defines some codes twice, the
    #    second copy differing only in a remark, so the first is kept. BKFL and
    #    WSTG repeat whole rows.
    frames["ABBR"] = frames["ABBR"].unique(
        subset=["ABBR_HDNG", "ABBR_CODE"], keep="first", maintain_order=True
    )
    for group in ("BKFL", "WSTG"):
        frames[group] = frames[group].unique(maintain_order=True)
    report(rebuild(frames, units, types), "2. duplicates removed")

    # 3. The dates are written 2009-02-18 and declared dd/mm/yyyy. The values
    #    are unambiguous, so it is the declaration that is corrected.
    for heading in ("LOCA_STAR", "LOCA_ENDD"):
        units["LOCA"][heading] = "yyyy-mm-dd"
    report(rebuild(frames, units, types), "3. date unit corrected")

    # 4. UNIT rows with no description. None of them is used in this file, so
    #    they are dropped; a unit that was used would need describing instead.
    used = {unit for per_group in units.values() for unit in per_group.values() if unit}
    undescribed = pl.col("UNIT_DESC").is_null() | (pl.col("UNIT_DESC") == "")
    unused = undescribed & ~pl.col("UNIT_UNIT").is_in(sorted(used))
    frames["UNIT"] = frames["UNIT"].filter(~unused)
    report(rebuild(frames, units, types), "4. unused, undescribed units dropped")

    # 5. TRAN never declares the record-link characters. With "+" declared as
    #    the concatenator, "CP+RC" reads as CP and RC, both of which ABBR defines.
    frames["TRAN"] = frames["TRAN"].with_columns(
        TRAN_DLIM=pl.lit("|"), TRAN_RCON=pl.lit("+")
    )
    units["TRAN"] |= {"TRAN_DLIM": "", "TRAN_RCON": ""}
    types["TRAN"] |= {"TRAN_DLIM": "X", "TRAN_RCON": "X"}
    report(rebuild(frames, units, types), "5. TRAN_DLIM and TRAN_RCON declared")

    # 6. Codes and units that are used but never defined. The GEOL_LEG
    #    description is a placeholder: the source does not say what 407 is.
    add_rows(
        frames,
        "ABBR",
        [
            {
                "ABBR_HDNG": "DICT_TYPE",
                "ABBR_CODE": "HEADING",
                "ABBR_DESC": "Flag: heading",
            },
            {"ABBR_HDNG": "DICT_STAT", "ABBR_CODE": "KEY", "ABBR_DESC": "Key"},
            {"ABBR_HDNG": "DICT_STAT", "ABBR_CODE": "OTHER", "ABBR_DESC": "Other"},
            {
                "ABBR_HDNG": "GEOL_LEG",
                "ABBR_CODE": "407",
                "ABBR_DESC": "Undefined in the source",
            },
        ],
    )
    # The rebuild gives a blank unit the dictionary's, so those need a UNIT row.
    add_rows(
        frames,
        "UNIT",
        [
            {"UNIT_UNIT": "bar", "UNIT_DESC": "bar"},
            {"UNIT_UNIT": "hh:mm:ss", "UNIT_DESC": "hours minutes seconds"},
            {
                "UNIT_UNIT": "yyyy-mm-ddThh:mm",
                "UNIT_DESC": "year month day hours minutes",
            },
        ],
    )
    report(rebuild(frames, units, types), "6. codes and units defined")

    # 7. The columns BGS added have names the format does not allow. Give each
    #    a conforming name and a DICT row saying what it is.
    renames = {
        "PROJ": {"BGS_PROJ_ID": ("PROJ_BGS", "BGS project identifier")},
        "LOCA": {"BGS_LOCA_ID": ("LOCA_BGS", "BGS location identifier")},
        "BKFL": {
            "BKFL_TOP_TEXT": ("BKFL_TOPT", "Top of backfill, as text"),
            "BKFL_BASE_TEXT": ("BKFL_BAST", "Base of backfill, as text"),
        },
        "CDIA": {"CDIA_DIAM_TEXT": ("CDIA_DIAT", "Casing diameter, as text")},
    }
    for group, columns in renames.items():
        for old, (new, meaning) in columns.items():
            if old not in frames[group].columns:
                continue
            frames[group] = frames[group].rename({old: new})
            units[group][new] = units[group].pop(old)
            types[group][new] = types[group].pop(old)
            declared = {
                "DICT_TYPE": "HEADING",
                "DICT_GRP": group,
                "DICT_HDNG": new,
                "DICT_STAT": "OTHER",
                "DICT_DTYP": "X",
                "DICT_DESC": meaning,
            }
            add_rows(frames, "DICT", [declared])
    report(rebuild(frames, units, types), "7. BGS columns renamed and declared")

    # 8. A stale dictionary row: BKFL's key was HOLE_ID in AGS3, and the file
    #    now keys it on LOCA_ID.
    stale = (pl.col("DICT_GRP") == "BKFL") & (pl.col("DICT_HDNG") == "HOLE_ID")
    frames["DICT"] = frames["DICT"].filter(~stale)
    clean = rebuild(frames, units, types)
    left = report(clean, "8. stale DICT key row removed")

    for finding in left.findings.filter(
        ~pl.col("rule").str.contains("Warning")
    ).iter_rows(named=True):
        print("   left:", finding["rule"], finding["group"], finding["desc"])
    print(f"   {left.warnings} warning(s) remain")
    clean.save(target)
    print(f"   saved {target}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
