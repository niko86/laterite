"""What merge does when two files declare the same heading with different types.

uv run --with laterite type_clash.py

Builds two one-borehole files in memory, so it needs no data.
"""

import laterite
import polars as pl


def one_hole(hole: str, depth: str, declared: str) -> str:
    """A minimal AGS4 file: one LOCA row, its final depth declared as given."""
    loca = pl.DataFrame({"LOCA_ID": [hole], "LOCA_FDEP": [depth]})
    types = {"LOCA": {"LOCA_FDEP": declared}}
    return laterite.build_ags4({"LOCA": loca}, types=types, mode="report").text


phase_1 = one_hole("BH01", "12.50", "2DP")  # depths to two decimal places
phase_2 = one_hole("BH02", "8.5", "1DP")  # the next contractor used one


def depth_column(result: laterite.MergeResult) -> str:
    merged = laterite.read(result.bytes)
    declared = dict(zip(merged.headings("LOCA"), merged.types("LOCA"), strict=True))
    values = merged["LOCA"]["LOCA_FDEP"].to_list()
    return f"declared {declared['LOCA_FDEP']}, read as {values}"


# By default merge will not choose for you.
try:
    laterite.merge(phase_1, phase_2)
except laterite.MergeConflictError as stopped:
    print("stopped:", str(stopped).split(". ")[0])

# promote: keep the column a number, at the finer of the two precisions.
print(
    "promote:", depth_column(laterite.merge(phase_1, phase_2, on_type_clash="promote"))
)

# widen: fall back to text, every value exactly as it was written.
print("widen:  ", depth_column(laterite.merge(phase_1, phase_2, on_type_clash="widen")))

# A number against text has no number to keep, so both settings give text, and
# neither says so in the warnings. Compare the types yourself.
phase_3 = one_hole("BH03", "about 9", "X")  # and one typed its depths as text
quiet = laterite.merge(phase_1, phase_3, on_type_clash="promote")
print("promote, number against text:", depth_column(quiet))
print("warnings:", [warning["kind"] for warning in quiet.warnings])


def types_changed(before: str, after: laterite.Ags4File) -> list[str]:
    """Headings whose declared type in ``after`` is not what ``before`` declared."""
    source = laterite.read(before)
    changed = []
    for group in source.groups:
        was = dict(zip(source.headings(group), source.types(group), strict=True))
        now = dict(zip(after.headings(group), after.types(group), strict=True))
        changed += [f"{h}: {was[h]} -> {now[h]}" for h in was if now.get(h) != was[h]]
    return changed


print("changed:", types_changed(phase_1, laterite.read(quiet.bytes)))
