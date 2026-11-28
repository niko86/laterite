# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "marimo",
#     "laterite",
#     "laterite-outcrop[laterite]>=0.1.1",
#     "plotly",
#     "pyproj",
# ]
# ///
# ruff: noqa: ANN001, ANN202, B018
# A marimo cell takes its inputs by name and shows its last expression, so its
# signature is not an API and its bare final line is its output.
"""An AGS4 file as a notebook: query it with SQL, put its boreholes on a map,
click one, and read its log.

    uvx marimo edit --sandbox boreholes_on_a_map.py -- --file your_file.ags
"""

import marimo

__generated_with = "0.26.0"
app = marimo.App(width="medium")


@app.cell
def _():
    import laterite
    import marimo as mo
    import plotly.express as px
    import polars as pl
    from outcrop import ags as plots
    from pyproj import Transformer

    return Transformer, laterite, mo, pl, plots, px


@app.cell
def _(mo):
    mo.md("""
    # Boreholes on a map

    Point this at an AGS4 file. laterite reads it, DuckDB queries it, pyproj
    turns its grid coordinates into latitude and longitude, and the map lets
    you pick a borehole to look at.
    """)
    return


@app.cell
def _(mo):
    # Started with `-- --file path.ags`, the notebook opens on that file.
    path = mo.ui.text(
        value=mo.cli_args().get("file") or "your_file.ags",
        label="AGS4 file",
        full_width=True,
    )
    # An AGS4 file gives eastings and northings but rarely says in which grid.
    grid = mo.ui.dropdown(
        options={
            "British National Grid (EPSG:27700)": "EPSG:27700",
            "Irish Transverse Mercator (EPSG:2157)": "EPSG:2157",
            "ETRS89 / UTM zone 31N (EPSG:25831)": "EPSG:25831",
        },
        value="British National Grid (EPSG:27700)",
        label="The file's coordinates are in",
    )
    mo.vstack([path, grid])
    return grid, path


@app.cell
def _(laterite, mo, path):
    ags = laterite.read(path.value)
    report = ags.validate().report
    mo.md(
        f"**{len(ags.groups)} groups**, {ags['LOCA'].height} locations. "
        f"Validation: {report.errors:,} errors, {report.warnings} warnings."
    )
    return (ags,)


@app.cell
def _(mo):
    mo.md("""
    ## Ask it in SQL

    Every group in the file is a DuckDB table, and a marimo SQL cell can use
    laterite's own connection. This one lists each location with its depth
    and how many strata were logged in it. `TRY_CAST` is there because some
    files declare their coordinates as text.
    """)
    return


@app.cell
def _(ags, mo):
    holes = mo.sql(
        """
        SELECT l.LOCA_ID                         AS hole,
               l.LOCA_TYPE                       AS type,
               TRY_CAST(l.LOCA_NATE AS DOUBLE)   AS easting,
               TRY_CAST(l.LOCA_NATN AS DOUBLE)   AS northing,
               TRY_CAST(l.LOCA_FDEP AS DOUBLE)   AS depth_m,
               count(g.LOCA_ID)                  AS strata
        FROM LOCA l
        LEFT JOIN GEOL g USING (LOCA_ID)
        GROUP BY ALL
        ORDER BY hole
        """,
        engine=ags.connection,
    )
    return (holes,)


@app.cell
def _(mo):
    mo.md("""
    ## From grid to map

    A web map wants latitude and longitude (WGS84). The file has eastings and
    northings on a national grid. pyproj converts between the two.
    """)
    return


@app.cell
def _(Transformer, grid, holes, pl):
    to_wgs84 = Transformer.from_crs(grid.value, "EPSG:4326", always_xy=True)
    placed = holes.drop_nulls(["easting", "northing"])
    lon, lat = to_wgs84.transform(
        placed["easting"].to_list(), placed["northing"].to_list()
    )
    # Named series, so each hole gets its own value and not the whole list.
    on_map = placed.with_columns(pl.Series("lon", lon), pl.Series("lat", lat))
    return (on_map,)


@app.cell
def _(mo, on_map, px):
    figure = px.scatter_map(
        on_map,
        lat="lat",
        lon="lon",
        hover_name="hole",
        hover_data={
            "type": True,
            "depth_m": True,
            "strata": True,
            "lat": False,
            "lon": False,
        },
        color="type",
        zoom=15,
        height=480,
        map_style="open-street-map",
    )
    figure.update_traces(marker={"size": 13})
    figure.update_layout(margin={"l": 0, "r": 0, "t": 0, "b": 0})
    # Wrapped, the map reports what you select: use the box or lasso tool.
    hole_map = mo.ui.plotly(figure)
    hole_map
    return (hole_map,)


@app.cell
def _(hole_map, mo, on_map):
    # The map reports each selected point by its position, so match on that.
    # Two holes drilled on the same spot are both picked.
    spots = {
        (round(point["lat"], 7), round(point["lon"], 7))
        for point in (hole_map.points or [])
        # Before anything is selected the widget can report a whole trace.
        if isinstance(point.get("lat"), float) and isinstance(point.get("lon"), float)
    }
    picked = [
        hole
        for hole, lat, lon in on_map.select("hole", "lat", "lon").iter_rows()
        if (round(lat, 7), round(lon, 7)) in spots
    ]
    choice = mo.ui.dropdown(
        options=on_map["hole"].to_list(),
        value=picked[0] if picked else on_map["hole"][0],
        label="Borehole" + (f" ({len(picked)} selected on the map)" if picked else ""),
    )
    choice
    return (choice,)


@app.cell
def _(ags, choice, mo):
    frames = ags.at("LOCA", [choice.value]).frames()
    counts = ", ".join(
        f"{code} {frame.height}" for code, frame in frames.items() if frame.height
    )
    strata = (
        frames["GEOL"].sort("GEOL_TOP").select("GEOL_TOP", "GEOL_BASE", "GEOL_DESC")
    )
    mo.vstack([mo.md(f"### {choice.value}\n\nRows by group: {counts}"), strata])
    return


@app.cell
def _(ags, choice, plots):
    plots.borehole_log(ags, choice.value)
    return


if __name__ == "__main__":
    app.run()
