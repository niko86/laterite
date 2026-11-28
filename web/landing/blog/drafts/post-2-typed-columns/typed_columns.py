"""See how each column was typed, and use the numbers as numbers.

uv run --with laterite typed_columns.py your_file.ags
"""

import sys

import laterite
import polars as pl

ags = laterite.read(sys.argv[1])
limits = ags["LLPL"]

# What the file declares, beside what you got.
for heading, declared in zip(ags.headings("LLPL"), ags.types("LLPL"), strict=True):
    print(f"{heading:<10} {declared:<4} {limits.schema[heading]}")

# No casting: the limits are already floats.
print(limits.select(pl.col("LLPL_LL").mean().round(1), pl.col("LLPL_PI").max()))

high = limits.filter(pl.col("LLPL_LL") >= 50)
print(f"{high.height} of {limits.height} results are high plasticity or above")

# The honest part: a column declared as text is read as text.
if "LNMC" in ags.groups:
    declared = dict(zip(ags.headings("LNMC"), ags.types("LNMC"), strict=True))
    print(
        "LNMC_MC is declared",
        declared["LNMC_MC"],
        "and read as",
        ags["LNMC"].schema["LNMC_MC"],
    )
    water = ags["LNMC"].with_columns(pl.col("LNMC_MC").cast(pl.Float64, strict=False))
    print(water.select(pl.col("LNMC_MC").mean().round(1)))
