"""Ask an AGS4 file a question: first with laterite's own query, then with SQL.

uv run --with laterite sql_across_groups.py your_file.ags
"""

import sys

import laterite
import polars as pl

ags = laterite.read(sys.argv[1])

# 1. One group, laterite's own query: say which rows and which columns.
high = (
    ags.query("SELECT * FROM LLPL")
    .filter("LLPL_LL >= 50")
    .select("LOCA_ID", "SAMP_TOP", "LLPL_LL", "LLPL_PI")
    .frame()
)
print(high)

# 2. Two groups: which soil unit did each of those come from? That needs the
#    strata as well, so it is a join, and a join is SQL.
in_unit = ags.sql("""
    SELECT l.LOCA_ID, l.SAMP_TOP, l.LLPL_LL, l.LLPL_PI, g.GEOL_STAT AS unit
    FROM LLPL l
    JOIN GEOL g
      ON g.LOCA_ID   = l.LOCA_ID
     AND l.SAMP_TOP >= g.GEOL_TOP
     AND l.SAMP_TOP <  g.GEOL_BASE
    WHERE l.LLPL_LL >= 50
    ORDER BY l.LOCA_ID, l.SAMP_TOP
""")
print(pl.DataFrame(in_unit))

# 3. The same join, summed up: every result, counted and averaged by unit.
by_unit = ags.sql("""
    SELECT g.GEOL_STAT              AS unit,
           count(*)                 AS tests,
           round(avg(l.LLPL_LL), 1) AS mean_ll,
           round(avg(l.LLPL_PI), 1) AS mean_pi
    FROM LLPL l
    JOIN GEOL g
      ON g.LOCA_ID   = l.LOCA_ID
     AND l.SAMP_TOP >= g.GEOL_TOP
     AND l.SAMP_TOP <  g.GEOL_BASE
    GROUP BY ALL
    ORDER BY unit
""")
print(pl.DataFrame(by_unit))

# Did every result find a stratum? A join drops the ones that did not.
joined = sum(pl.DataFrame(by_unit)["tests"])
print(f"{joined} of {ags['LLPL'].height} results matched a stratum")

# What to paste into an LLM when you want it to write the SQL for you: each
# group's headings, with the type the file declares for each.
for code in ("LLPL", "GEOL"):
    headings = zip(ags.headings(code), ags.types(code), strict=True)
    print(code + ":", ", ".join(f"{name} ({kind})" for name, kind in headings))
