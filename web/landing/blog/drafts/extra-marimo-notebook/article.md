# An AGS file as a notebook, with a map

_An optional companion to the series. Notebook: `boreholes_on_a_map.py`, beside this draft._

The scripts in this series print tables. That is fine for checking a file and
poor for exploring one. This is the same toolkit in a
[marimo](https://marimo.io) notebook: query the file with SQL, put its
boreholes on a map, pick one, and read its log.

```
uvx marimo edit --sandbox boreholes_on_a_map.py -- --file your_file.ags
```

One line, nothing installed first. The notebook lists what it needs at the
top, and `--sandbox` has uv build a throwaway environment with exactly that
in it. Use `marimo run` in place of `marimo edit` to open it as an app with
the code hidden.

laterite already has a longer marimo notebook, `examples/laterite_tour.py`,
which walks through the whole toolkit on a sample file. This one is narrower
and is meant for your own file: it does one job, from file to map to log.

## Why marimo

A marimo notebook is a plain Python file, so it diffs, reviews and emails
like any other script. And it is reactive: change the file path or pick a
different borehole, and every cell that depends on it reruns. There is no
"run all cells again and hope".

## The file, in DuckDB

laterite reads the file and hands marimo its DuckDB connection, so a SQL cell
sees every group as a table:

```python
ags = laterite.read(path.value)

holes = mo.sql(
    """
    SELECT l.LOCA_ID                       AS hole,
           l.LOCA_TYPE                     AS type,
           TRY_CAST(l.LOCA_NATE AS DOUBLE) AS easting,
           TRY_CAST(l.LOCA_NATN AS DOUBLE) AS northing,
           TRY_CAST(l.LOCA_FDEP AS DOUBLE) AS depth_m,
           count(g.LOCA_ID)                AS strata
    FROM LOCA l
    LEFT JOIN GEOL g USING (LOCA_ID)
    GROUP BY ALL
    ORDER BY hole
    """,
    engine=ags.connection,
)
```

`engine=ags.connection` is the whole trick. The result is a polars frame and
marimo shows it as a table you can sort, search and export. In the editor a
SQL cell is a proper SQL editor, with the tables listed down the side.

The `TRY_CAST` is there because this file, the Crossrail export from posts 1
and 4, declares its coordinates as text (post 2).

## From grid to map

AGS files give eastings and northings on a national grid. A web map wants
latitude and longitude. For a UK file that is British National Grid to WGS84,
and [pyproj](https://pyproj4.github.io/pyproj/) does it in three lines:

```python
from pyproj import Transformer

to_wgs84 = Transformer.from_crs("EPSG:27700", "EPSG:4326", always_xy=True)
lon, lat = to_wgs84.transform(eastings, northings)
```

Borehole F1, at 531538 E 181726 N, comes out at 51.5191° N, 0.1056° W: the
south end of Farringdon Road.

An AGS file rarely says which grid it uses, so the notebook has a dropdown
for it. The Fugro files from posts 2, 3 and 5 are in ETRS89 / UTM zone 31N,
and the same three lines put them in the North Sea.

## Pick a borehole

The map is a Plotly map wrapped in `mo.ui.plotly`, which makes the selection
available to Python. Draw a box round some boreholes, and the cells below
update: the record set from post 1, its strata, and the borehole log.

```python
frames = ags.at("LOCA", [choice.value]).frames()
plots.borehole_log(ags, choice.value)
```

That is the fan-out from post 1 and the log from its figure, now one click
away from a map.

## What I have and have not checked

The notebook runs from top to bottom on the Crossrail file with no errors,
and the table, the conversion and the log are right. I built it on a machine
that cannot reach a map tile server, so I have not seen the base map drawn
or tried selecting on it by hand. Try that before relying on it.

_Contains British Geological Survey materials ©NERC 2026. Run on laterite 0.17.1, marimo 0.26.0._
