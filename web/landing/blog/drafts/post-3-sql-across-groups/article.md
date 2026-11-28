# Ask the file a question

_Post 3 of 5. Script: `sql_across_groups.py`, beside this draft._

![The same results coloured by soil unit](by_unit.png)

Post 1 pulled a borehole out of a file. Post 2 showed the columns arrive as
numbers. This post is about asking questions, starting with an easy one and
ending with one that AGS makes surprisingly hard.

The data is the Fugro campaign from post 2: 53 Atterberg results from the
Hollandse Kust (zuid) wind farm zone.

## A question about one group

Which results are high plasticity?

Everything needed is in one group, `LLPL`. laterite has its own query for
this. You say which rows you want and which columns:

```python
import laterite

ags = laterite.read("your_file.ags")

high = (
    ags.query("SELECT * FROM LLPL")
    .filter("LLPL_LL >= 50")
    .select("LOCA_ID", "SAMP_TOP", "LLPL_LL", "LLPL_PI")
    .frame()
)
```

```
┌───────────────┬──────────┬─────────┬─────────┐
│ LOCA_ID       ┆ SAMP_TOP ┆ LLPL_LL ┆ LLPL_PI │
╞═══════════════╪══════════╪═════════╪═════════╡
│ HKZ3-BH01-SA  ┆ 16.0     ┆ 57.0    ┆ 32.0    │
│ HKZ3-BH01-SA  ┆ 17.0     ┆ 72.0    ┆ 42.0    │
│ HKZ3-BH01-SA  ┆ 31.5     ┆ 55.0    ┆ 28.0    │
│ …             ┆ …        ┆ …       ┆ …       │
│ HKZ3-BH24-SA  ┆ 13.0     ┆ 62.0    ┆ 33.0    │
└───────────────┴──────────┴─────────┴─────────┘
```

Ten rows. Three things to notice:

- `LLPL_LL >= 50` compares numbers, because the column is a number (post 2).
- Nothing runs until `.frame()`. You can build a query up a step at a time,
  and branch from a shared start without one branch disturbing another.
- `.frame()` gives you polars. `.to_pandas()` gives you pandas.

For a lot of everyday work, that is all you need.

## A question about two groups

Which soil unit did each of those ten come from?

Now the answer is not in `LLPL`. The unit is recorded against the strata, in
`GEOL`, and AGS holds no link between a lab result and a stratum. A result
knows its borehole and its sample depth. A stratum knows its borehole, its
top and its base. To put them together you match on both: same hole, and the
sample depth falls inside the stratum.

That is a join, and joins are what SQL is for. laterite exposes every group
in the file as a table named by its four-letter code:

```python
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
```

```
┌───────────────┬──────────┬─────────┬─────────┬──────┐
│ LOCA_ID       ┆ SAMP_TOP ┆ LLPL_LL ┆ LLPL_PI ┆ unit │
╞═══════════════╪══════════╪═════════╪═════════╪══════╡
│ HKZ3-BH01-SA  ┆ 16.0     ┆ 57.0    ┆ 32.0    ┆ B2   │
│ HKZ3-BH01-SA  ┆ 17.0     ┆ 72.0    ┆ 42.0    ┆ B2   │
│ HKZ3-BH01-SA  ┆ 31.5     ┆ 55.0    ┆ 28.0    ┆ C2   │
│ …             ┆ …        ┆ …       ┆ …       ┆ …    │
│ HKZ3-BH24-SA  ┆ 13.0     ┆ 62.0    ┆ 33.0    ┆ B2   │
└───────────────┴──────────┴─────────┴─────────┴──────┘
```

Read it from the middle: take the results (`LLPL`), attach the strata
(`GEOL`) where the hole matches and the depth falls between top and base,
keep the high ones.

The engine underneath is DuckDB. It ships with laterite, so there is nothing
to install, no database to set up and nothing to load. Fugro put its soil
unit in `GEOL_STAT`; your file may use `GEOL_GEOL` or `GEOL_FORM`.

## A question about the whole site

Once the join is written, the bigger question is a small change: not the ten
high results, but all 53, counted and averaged by unit.

```python
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
```

```
┌──────┬───────┬─────────┬─────────┐
│ unit ┆ tests ┆ mean_ll ┆ mean_pi │
╞══════╪═══════╪═════════╪═════════╡
│ B1   ┆ 2     ┆ 56.5    ┆ 32.5    │
│ B2   ┆ 12    ┆ 48.3    ┆ 24.3    │
│ C2   ┆ 33    ┆ 37.8    ┆ 18.3    │
│ D    ┆ 6     ┆ 44.3    ┆ 23.0    │
└──────┴───────┴─────────┴─────────┘
```

That table is the chart at the top of this post, in numbers.

## You do not need to be fluent in SQL

I am not. That used to be a real barrier: the join above is not something
most geotechnical engineers would write from memory.

It matters much less now, because writing SQL from a plain description is
something LLMs are very good at. What they need from you is the shape of the
data, and laterite will print that for you:

```python
for code in ("LLPL", "GEOL"):
    headings = zip(ags.headings(code), ags.types(code), strict=True)
    print(code + ":", ", ".join(f"{name} ({kind})" for name, kind in headings))
```

```
LLPL: LOCA_ID (ID), SAMP_TOP (2DP), SAMP_REF (X), … LLPL_LL (1DP), LLPL_PL (1DP), LLPL_PI (1DP), …
GEOL: LOCA_ID (ID), GEOL_TOP (2DP), GEOL_BASE (2DP), GEOL_DESC (X), … GEOL_STAT (X), …
```

Paste that in with the question in your own words:

> These are two tables from an AGS4 geotechnical file, queried with DuckDB.
> LLPL holds Atterberg limit results; GEOL holds strata, with the soil unit
> in GEOL_STAT. Write a query giving the number of results and the mean
> liquid limit and plasticity index for each soil unit. A result belongs to
> the stratum in the same borehole whose depth range contains its SAMP_TOP.

The last sentence is the part only you can supply. The model knows SQL. You
know that a sample belongs to the stratum it was taken from, and that in AGS
you have to say so.

## Check the answer

Whoever wrote the query, check it. A join silently drops anything that does
not match, so a result whose depth falls in no stratum vanishes from the
table without a word. Count both sides:

```python
joined = sum(pl.DataFrame(by_unit)["tests"])
print(f"{joined} of {ags['LLPL'].height} results matched a stratum")
# 53 of 53 results matched a stratum
```

Two more things to know:

- `.sql()` returns a DuckDB relation. On a plain `pip install laterite`,
  turn it into polars with `pl.DataFrame(relation)`. DuckDB's own `.pl()`
  and `.df()` need pyarrow and pandas, which laterite does not install for
  you.
- If the file declares its depths as text (post 2), the depth comparison
  will not run as written. Cast inside the query:
  `TRY_CAST(l.SAMP_TOP AS DOUBLE) >= g.GEOL_TOP`.

## One step further

A little more SQL goes a long way. The summary above says how plastic each
unit is. It does not say what the unit is made of. That is in the file too:
each stratum carries a legend code, and the file's own abbreviation table,
`ABBR`, says what the code means. Join it in as a third table:

```python
by_soil = ags.sql("""
    SELECT g.GEOL_STAT              AS unit,
           a.ABBR_DESC              AS soil,
           count(*)                 AS tests,
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
```

Four lines more than before, and legend code `201` reads `CLAY`:

```
┌──────┬────────────┬───────┬─────────┬─────────┐
│ unit ┆ soil       ┆ tests ┆ mean_ll ┆ mean_pi │
╞══════╪════════════╪═══════╪═════════╪═════════╡
│ B1   ┆ SAND       ┆ 2     ┆ 56.5    ┆ 32.5    │
│ B2   ┆ CLAY       ┆ 8     ┆ 50.1    ┆ 25.5    │
│ B2   ┆ Sandy CLAY ┆ 3     ┆ 36.0    ┆ 16.3    │
│ B2   ┆ SAND       ┆ 1     ┆ 70.0    ┆ 38.0    │
│ C2   ┆ CLAY       ┆ 18    ┆ 39.9    ┆ 20.2    │
│ C2   ┆ Silty SAND ┆ 6     ┆ 37.0    ┆ 17.3    │
│ C2   ┆ SAND       ┆ 4     ┆ 33.0    ┆ 14.8    │
│ C2   ┆ Sandy CLAY ┆ 4     ┆ 32.5    ┆ 13.0    │
│ C2   ┆ null       ┆ 1     ┆ 45.0    ┆ 27.0    │
│ D    ┆ CLAY       ┆ 6     ┆ 44.3    ┆ 23.0    │
└──────┴────────────┴───────┴─────────┴─────────┘
```

One statement, three groups, and the table already tells you something the
summary hid. Fourteen of the 53 results sit in strata logged as sand, and one
in a stratum with no legend code at all. Atterberg limits on a sand are worth
a second look: clay laminae within it, or a sample depth sitting on a
boundary. That is a question for the engineer, and the query is what put it
in front of them.

It is `LEFT JOIN` for `ABBR` so that a stratum with no legend code is kept
and shows as `null`, not dropped.

## The figure

The same chart as post 2, with the joined frame and one more argument:

```python
import outcrop

outcrop.plasticity_chart(frame, by="unit")
```

_Data: Netherlands Enterprise Agency (RVO) / Fugro. Run on laterite 0.17.1._
