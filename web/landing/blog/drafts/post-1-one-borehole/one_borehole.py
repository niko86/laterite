"""Pull one borehole's whole record set out of an AGS4 file.

uv run --with laterite one_borehole.py your_file.ags [HOLE_ID]
"""

import sys

import laterite

ags = laterite.read(sys.argv[1])

# Which holes are in the file? Default to the first.
holes = ags["LOCA"]["LOCA_ID"].to_list()
hole = sys.argv[2] if len(sys.argv) > 2 else holes[0]
if hole not in holes:
    sys.exit(f"{hole} is not in this file. It holds: {', '.join(holes)}")

# Everything keyed on that hole. Nothing is read until .frames().
record = ags.at("LOCA", [hole])
print(record.groups)

frames = record.frames()
for code, frame in frames.items():
    print(f"{code}  {frame.height:>4} rows")

# Each frame is an ordinary polars frame.
strata = frames["GEOL"].sort("GEOL_TOP")
print(strata.select("GEOL_TOP", "GEOL_BASE", "GEOL_DESC"))

# PROJ has no LOCA_ID, so the fan-out leaves it out. A log header wants it.
frames["PROJ"] = ags["PROJ"]
