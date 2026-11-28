"""Join lab results to the stratum each sample came from.

uv run --with laterite sql_across_groups.py your_file.ags
"""

import sys

import laterite
import polars as pl

ags = laterite.read(sys.argv[1])

# Every group is a table named by its four-letter code.
by_unit = ags.sql("""
    SELECT g.GEOL_STAT            AS unit,
           a.ABBR_DESC            AS soil,
           count(*)               AS tests,
           round(avg(l.LLPL_LL), 1) AS mean_ll,
           round(avg(l.LLPL_PI), 1) AS mean_pi
    FROM LLPL l
    JOIN GEOL g
      ON g.LOCA_ID   = l.LOCA_ID
     AND l.SAMP_TOP >= g.GEOL_TOP
     AND l.SAMP_TOP <  g.GEOL_BASE
    LEFT JOIN ABBR a
      ON a.ABBR_HDNG = 'GEOL_LEG'
     AND a.ABBR_CODE = g.GEOL_LEG
    GROUP BY ALL
    ORDER BY unit, tests DESC
""")

# .sql() hands back a DuckDB relation. This turns it into polars on a base
# install; .pl() would need pyarrow.
print(pl.DataFrame(by_unit))

# Did every result find a stratum? A join drops the ones that did not.
joined = ags.sql("""
    SELECT count(*) FROM LLPL l
    JOIN GEOL g
      ON g.LOCA_ID = l.LOCA_ID
     AND l.SAMP_TOP >= g.GEOL_TOP AND l.SAMP_TOP < g.GEOL_BASE
""").fetchall()[0][0]
print(f"{joined} of {ags['LLPL'].height} results matched a stratum")
