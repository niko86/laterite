# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "marimo",
#     "polars>=1.40.1",
#     "pyarrow>=17",
#     "duckdb>=1.4.0",
#     "altair>=5.4",
#     "folium>=0.17",
#     "pyproj>=3.6",
#     "laterite==0.11.0",
# ]
# ///
"""laterite — an interactive tour (a marimo notebook).

Showcases the `laterite` AGS4 toolkit: born-typed reads, numbered-rule
validation, cross-group SQL, live charts, the `.ags.idx` certificate,
transport (zstd + age), materialise-to-DuckDB, and the multi-surface story
(Python / Node / CLI / DuckDB extension).

Runs live on marimo molab — https://github.com/niko86/laterite
"""

import marimo

app = marimo.App(width="medium")


@app.cell
def _():
    import hashlib
    import pathlib
    import tempfile
    import urllib.request

    import altair as alt
    import duckdb
    import folium
    import laterite
    import marimo as mo
    import polars as pl
    from pyproj import Transformer

    return (
        Transformer,
        alt,
        duckdb,
        folium,
        hashlib,
        laterite,
        mo,
        pathlib,
        pl,
        tempfile,
        urllib,
    )


@app.cell
def _(mo):
    mo.md(
        r"""
        # 🦀 laterite — a modern AGS4 toolkit

        A **Rust-backed** reader, writer and validator for the
        [AGS4](https://www.ags.org.uk/data-format/) geotechnical data format — a
        faster, **born-typed** drop-in for `python-ags4`, surfaced for **Python,
        Node.js, a CLI, the browser, and DuckDB**.

        This notebook is **live** — edit any cell and everything downstream
        re-runs. Pick a built-in dataset or **upload your own `.ags`** below, and
        the whole tour runs against it.

        `pip install laterite` · `npm install laterite` ·
        [GitHub](https://github.com/niko86/laterite)
        """
    )
    return


@app.cell
def _():
    # The rich default is fetched from the repo at runtime so it stays editable
    # as a plain file (swap in your own dataset there). Two tiny fixtures are
    # embedded for an offline fallback + the "watch the validator" demo.
    EXAMPLE_URL = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"

    EXCEL_AGS = """"GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","ID","X"
"DATA","123456","Excel fixture (hand-authored, MIT, ours)"

"GROUP","TRAN"
"HEADING","TRAN_ISNO","TRAN_AGS"
"UNIT","",""
"TYPE","X","X"
"DATA","1","4.1"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE","LOCA_NATN","LOCA_FDEP"
"UNIT","","","m","m","m"
"TYPE","ID","PA","2DP","3DP","2DP"
"DATA","Location_1","Boring","100000.01","5000000.001","50.11"
"DATA","Location_2","Boring","101000.01","5000000.100","50.22"

"GROUP","LLPL"
"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","LLPL_LL","LLPL_PL","LLPL_PI"
"UNIT","","m","","","","%","%",""
"TYPE","ID","2DP","X","PA","ID","2SF","XN","2SF"
"DATA","Location_1","1.00","1a","Bag","S1","55.1","20.3","34.8"
"DATA","Location_1","2.00","2a","Bag","S2","155.1","20.3","134.8"
"""

    CLEAN_AGS = """"GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","ID","X"
"DATA","P1","Clean minimal AGS4 fixture (hand-authored, MIT, ours)"

"GROUP","TRAN"
"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON"
"UNIT","","yyyy-mm-dd","","","","","",""
"TYPE","X","DT","X","X","X","X","X","X"
"DATA","1","2020-08-18","ACME Drilling Ltd","Draft","4.2","ACME Consulting","|","+"

"GROUP","UNIT"
"HEADING","UNIT_UNIT","UNIT_DESC"
"UNIT","",""
"TYPE","X","X"
"DATA","yyyy-mm-dd","year month day"

"GROUP","TYPE"
"HEADING","TYPE_TYPE","TYPE_DESC"
"UNIT","",""
"TYPE","X","X"
"DATA","ID","Unique identifier"
"DATA","X","Text"
"DATA","DT","Date and time"
"""
    return CLEAN_AGS, EXAMPLE_URL, EXCEL_AGS


@app.cell
def _(mo):
    source = mo.ui.dropdown(
        options={
            "Example site — 14 boreholes (rich, for the charts)": "example",
            "Minimal — a clean validation pass": "clean",
            "Has errors — watch the validator work": "errors",
        },
        value="Example site — 14 boreholes (rich, for the charts)",
        label="Built-in data",
    )
    upload = mo.ui.file(
        kind="button", filetypes=[".ags"], label="…or upload your own .ags"
    )
    mo.callout(mo.hstack([source, upload], justify="start", gap=2), kind="info")
    return source, upload


@app.cell
def _(CLEAN_AGS, EXAMPLE_URL, EXCEL_AGS, mo, source, upload, urllib):
    if upload.value:
        sample_bytes = upload.value[0].contents
        source_label = upload.value[0].name
    elif source.value == "clean":
        sample_bytes, source_label = CLEAN_AGS.encode(), "clean_minimal.ags"
    elif source.value == "errors":
        sample_bytes, source_label = EXCEL_AGS.encode(), "excel_source.ags"
    else:
        try:
            sample_bytes = urllib.request.urlopen(EXAMPLE_URL, timeout=10).read()
            source_label = "sample_site.ags"
        except Exception:
            sample_bytes = EXCEL_AGS.encode()
            source_label = "excel_source.ags (offline fallback)"
    mo.md(f"**Active source:** `{source_label}` · {len(sample_bytes):,} bytes")
    return (sample_bytes,)


@app.cell
def _(hashlib, pathlib, sample_bytes, tempfile):
    # A few APIs (validate, certify, the DuckDB extension) take a path, so write
    # the active bytes to a content-hashed temp file. The only fs write we do.
    _digest = hashlib.sha1(sample_bytes).hexdigest()[:12]
    ags_path = str(pathlib.Path(tempfile.gettempdir()) / f"laterite_tour_{_digest}.ags")
    pathlib.Path(ags_path).write_bytes(sample_bytes)
    return (ags_path,)


@app.cell
def _(laterite, sample_bytes):
    ags = laterite.read(data=sample_bytes)
    return (ags,)


@app.cell
def _(ags, mo):
    mo.vstack(
        [
            mo.md(
                """
                ## 1 · Read → born-typed tables

                `laterite.read()` returns every group as a **polars DataFrame**
                with real dtypes decoded from the AGS `TYPE` row — a `2DP`
                heading is an `f64`, a `DT` a `datetime`, an `ID` a `str`.
                python-ags4 hands you everything as strings.
                """
            ),
            mo.ui.tabs({g: ags.table(g) for g in ags.groups}),
        ]
    )
    return


@app.cell
def _(mo):
    warnings_sw = mo.ui.switch(value=True, label="warnings")
    fyi_sw = mo.ui.switch(value=False, label="FYI")
    mo.hstack(
        [mo.md("**Severity tiers** — errors always; "), warnings_sw, fyi_sw],
        justify="start",
        gap=1,
    )
    return fyi_sw, warnings_sw


@app.cell
def _(ags_path, fyi_sw, laterite, mo, warnings_sw):
    report = laterite.validate(ags_path, warnings=warnings_sw.value, fyi=fyi_sw.value)
    _counts = {"error": 0, "warning": 0, "fyi": 0}
    if report.count:
        for _row in report.findings.group_by("severity").len().iter_rows():
            _counts[_row[0]] = _row[1]
    _kpis = mo.hstack(
        [
            mo.stat(
                # errors + warnings are the gate; FYI is advisory and never fails it
                "PASS"
                if (_counts["error"] == 0 and _counts["warning"] == 0)
                else "FAIL",
                label="Validity",
                caption=f"dict {report.dict_version}",
                bordered=True,
            ),
            mo.stat(report.count, label="Findings", bordered=True),
            mo.stat(_counts["error"], label="Errors", bordered=True),
            mo.stat(_counts["warning"], label="Warnings", bordered=True),
        ]
    )
    _body = (
        mo.ui.table(report.findings, selection=None, pagination=True)
        if report.count
        else mo.callout("No findings — clean file. ✅", kind="success")
    )
    mo.vstack(
        [
            mo.md(
                "## 2 · Validate — every numbered AGS4 rule\n\nClean-room rules "
                "written from the spec. Toggle the tiers above."
            ),
            _kpis,
            _body,
        ]
    )
    return (report,)


@app.cell
def _(mo):
    mo.md(
        "## 3 · Charts — real geotech, straight from born-typed columns\n\n"
        "**Site plan** — boreholes on a Leaflet map. `LOCA_NATE` / `LOCA_NATN` "
        "(British National Grid, OSGB36) are projected to WGS84 with the same "
        "Helmert transform the browser app uses (~5 m, no grid download). "
        "Pan / zoom and hover a pin."
    )
    return


@app.cell
def _(Transformer, ags, folium, mo):
    _have_site = "LOCA" in ags.groups and {"LOCA_NATE", "LOCA_NATN"} <= set(
        ags["LOCA"].columns
    )
    if _have_site:
        _loca = ags["LOCA"]
        # OSGB36 British National Grid -> WGS84 (Helmert 7-param), matching
        # web/src/lib/coords.ts: ~5 m, no OSTN15 grid download needed.
        _osgb = (
            "+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 "
            "+y_0=-100000 +ellps=airy +towgs84=446.448,-125.157,542.06,0.15,"
            "0.247,0.842,-20.489 +units=m +no_defs"
        )
        _to_wgs84 = Transformer.from_crs(_osgb, "EPSG:4326", always_xy=True)
        _rows = []
        for _r in _loca.iter_rows(named=True):
            _lon, _lat = _to_wgs84.transform(_r["LOCA_NATE"], _r["LOCA_NATN"])
            _rows.append((_r.get("LOCA_ID"), _lat, _lon, _r.get("LOCA_FDEP")))
        _lat0 = sum(r[1] for r in _rows) / len(_rows)
        _lon0 = sum(r[2] for r in _rows) / len(_rows)
        _map = folium.Map(
            location=[_lat0, _lon0], zoom_start=15, tiles="CartoDB positron"
        )
        for _id, _lat, _lon, _fdep in _rows:
            folium.CircleMarker(
                [_lat, _lon],
                radius=8,
                color="#5a1a14",
                weight=1,
                fill=True,
                fill_color="#b2342a",
                fill_opacity=0.85,
                tooltip=f"{_id} — {_fdep} m deep",
            ).add_to(_map)
        _out = _map  # marimo renders folium's _repr_html_
    else:
        _out = mo.md("*(load a file with `LOCA` easting/northing for the site map)*")
    _out
    return


@app.cell
def _(alt, ags, mo, pl):
    _have_pl = "LLPL" in ags.groups and {"LLPL_LL", "LLPL_PI"} <= set(
        ags["LLPL"].columns
    )
    if _have_pl:
        _ll = ags["LLPL"]
        _tt = [
            c for c in ("LOCA_ID", "SAMP_TOP", "LLPL_LL", "LLPL_PI") if c in _ll.columns
        ]
        _pts = (
            alt.Chart(_ll)
            .mark_circle(size=90, opacity=0.7, color="#b2342a")
            .encode(
                x=alt.X(
                    "LLPL_LL:Q",
                    title="Liquid limit, LL (%)",
                    scale=alt.Scale(domain=[0, 100]),
                ),
                y=alt.Y(
                    "LLPL_PI:Q",
                    title="Plasticity index, PI (%)",
                    scale=alt.Scale(domain=[0, 70]),
                ),
                tooltip=_tt,
            )
        )
        # Casagrande A-line: PI = 0.73 (LL - 20)
        _aline = (
            alt.Chart(pl.DataFrame({"LL": [20.0, 110.0], "PI": [0.0, 65.7]}))
            .mark_line(color="#444", strokeDash=[6, 4])
            .encode(x="LL:Q", y="PI:Q")
        )
        _out = mo.vstack(
            [
                mo.md(
                    "**Casagrande plasticity chart** — `LLPL_LL` × `LLPL_PI` with "
                    "the A-line (dashed). Both axes are born-typed `f64`."
                ),
                mo.ui.altair_chart((_pts + _aline).properties(height=360)),
            ]
        )
    else:
        _out = mo.md(
            "*(no `LLPL` plasticity data in this file — try the example site)*"
        )
    _out
    return


@app.cell
def _(ags, mo):
    _groups = set(ags.groups)
    if {"LOCA", "LLPL"} <= _groups:
        _default = (
            "SELECT l.LOCA_ID, l.LOCA_GL, p.SAMP_TOP, p.LLPL_LL, p.LLPL_PI\n"
            "FROM LLPL p JOIN LOCA l USING (LOCA_ID)\n"
            "ORDER BY p.LLPL_PI DESC\n"
            "LIMIT 20"
        )
    else:
        _g = ags.groups[0]
        _default = f'SELECT * FROM "{_g}" LIMIT 20'
    sql_editor = mo.ui.code_editor(value=_default, language="sql", label="")
    mo.vstack(
        [
            mo.md(
                "## 4 · Explore — real SQL across groups\n\n`ags.sql(...)` runs a "
                "**DuckDB** engine over the born-typed columns — joins, filters, "
                "aggregates. No pandas in the path. Edit and re-run:"
            ),
            sql_editor,
        ]
    )
    return (sql_editor,)


@app.cell
def _(ags, mo, sql_editor):
    try:
        _res = ags.sql(sql_editor.value).pl()
        _out = mo.ui.table(_res, pagination=True)
    except Exception as _e:  # bad SQL shouldn't break the tour
        _out = mo.callout(f"Query error: {_e}", kind="warn")
    _out
    return


@app.cell
def _(ags, mo):
    # registry-driven related-group fan-out
    _out = mo.md("")
    if "LOCA" in ags.groups:
        _ids = ags["LOCA"]["LOCA_ID"].head(1).to_list()
        if _ids:
            _q = ags.at("LOCA", _ids)
            _out = mo.vstack(
                [
                    mo.md(
                        f"## 5 · Fan-out — one borehole, its whole record\n\n"
                        f"`ags.at('LOCA', {_ids})` pulls `{_ids[0]}` **and every "
                        f"related group** (the registry knows the key chain):"
                    ),
                    mo.ui.tabs(dict(_q.frames().items())),
                ]
            )
    _out
    return


@app.cell
def _(mo):
    mo.md(
        """
        ---
        ## 6 · Beyond python-ags4

        The things a string-based pandas shim can't do.
        """
    )
    return


@app.cell
def _(ags_path, laterite, mo):
    # The .ags.idx certificate: validate once, certify, then an errors-only
    # re-open skips the rule engine entirely (resolution == "certified").
    try:
        _h = laterite.read(ags_path)
        _h.validate()
        if _h.report.is_valid:
            _idx = _h.certify()
            _c = laterite.read(ags_path, index=str(_idx))
            _c.validate(warnings=False)  # errors-only engages the cert skip
            _out = mo.vstack(
                [
                    mo.md("### 🔖 Certificate + index (`.ags.idx`)"),
                    mo.hstack(
                        [
                            mo.stat(
                                _c.report.resolution,
                                label="Re-validate",
                                caption="engine skipped",
                                bordered=True,
                            ),
                            mo.stat(
                                "PASS" if _c.report.is_valid else "FAIL",
                                label="Verdict",
                                bordered=True,
                            ),
                        ],
                        justify="start",
                    ),
                    mo.md(
                        "`read(p).validate().certify()` mints a sibling **`.ags.idx`** — "
                        "a signed validity certificate **+ byte-offset index**. A later "
                        "`read(p, index=…)` returns a verdict **without re-running the "
                        "rules**, and can slice a single group's bytes (locally or over "
                        "an HTTP range request). python-ags4 re-checks everything, every time."
                    ),
                ]
            )
        else:
            _out = mo.callout(
                "`certify()` records a *passed* validation — this file has findings, "
                "so there's nothing to certify. Try the clean sample.",
                kind="neutral",
            )
    except Exception as _e:
        _out = mo.callout(f"certify demo unavailable: {_e}", kind="warn")
    _out
    return


@app.cell
def _(ags_path, mo, pathlib):
    # transport: zstd compression + age passphrase encryption, single-file
    from laterite import transport

    _src = pathlib.Path(ags_path)
    _z = transport.pack(ags_path)  # -> <path>.zst
    _ratio = _z.stat().st_size / max(_src.stat().st_size, 1)
    _a = transport.lock(ags_path, password="demo-passphrase")  # -> <path>.zst.age
    mo.vstack(
        [
            mo.md("### 📦 Transport — pack (zstd) & lock (age)"),
            mo.hstack(
                [
                    mo.stat(
                        f"{_src.stat().st_size:,} B", label="Original", bordered=True
                    ),
                    mo.stat(
                        f"{_z.stat().st_size:,} B",
                        label="pack() .zst",
                        caption=f"{_ratio:.0%} of source",
                        bordered=True,
                    ),
                    mo.stat(
                        f"{_a.stat().st_size:,} B",
                        label="lock() .zst.age",
                        caption="encrypted",
                        bordered=True,
                    ),
                ],
                justify="start",
            ),
            mo.md(
                "`transport.pack` / `unpack` (zstd) and `lock` / `unlock` (zstd **+ "
                "age** passphrase encryption) work on any file. **`pack` is a "
                "single-file envelope** — it does *not* bundle the `.ags.idx`; the "
                "index is a regenerable sidecar (`certify()` re-mints it in ms)."
            ),
        ]
    )
    return


@app.cell
def _(ags, duckdb, mo, pathlib, tempfile):
    # Materialise AGS4 -> a real .duckdb database any DuckDB tool can query.
    try:
        _store_path = str(pathlib.Path(tempfile.gettempdir()) / "laterite_site.duckdb")
        pathlib.Path(_store_path).unlink(missing_ok=True)
        _store = duckdb.connect(_store_path)
        for _g in ags.groups:
            _df = ags.table(_g)  # born-typed polars
            # register explicitly — duckdb's implicit local-scan doesn't reach
            # the cell frame in every sandbox (e.g. molab)
            _store.register("_src_df", _df)
            _store.execute(f'CREATE TABLE "{_g}" AS SELECT * FROM _src_df')
        _store.unregister("_src_df")
        _store.close()
        _chk = duckdb.connect(_store_path, read_only=True)
        _tables = [r[0] for r in _chk.execute("SHOW TABLES").fetchall()]
        _size = pathlib.Path(_store_path).stat().st_size
        _out = mo.vstack(
            [
                mo.md("### 🗄️ Materialise → a portable `.duckdb`"),
                mo.hstack(
                    [
                        mo.stat(len(_tables), label="Tables", bordered=True),
                        mo.stat(f"{_size:,} B", label=".duckdb size", bordered=True),
                    ],
                    justify="start",
                ),
                mo.md(
                    "Every born-typed group `CTAS`-ed into a file-backed DuckDB "
                    "database — now queryable by **any** DuckDB client, no laterite "
                    "needed. Host it and `ATTACH 'https://…/site.duckdb'` to query "
                    "it remotely. Tables: " + ", ".join(f"`{t}`" for t in _tables)
                ),
            ]
        )
    except Exception as _e:
        _out = mo.callout(f"materialise demo unavailable: {_e}", kind="warn")
    _out
    return


@app.cell
def _(ags_path, duckdb, mo):
    # The native DuckDB extension. May not load on every sandbox (it's pinned to
    # a DuckDB ABI), so degrade gracefully and just show the SQL.
    _sql = (
        "INSTALL laterite_ags4 FROM community;\n"
        "LOAD laterite_ags4;\n\n"
        "-- born-typed columns + deterministic _id / _parent_id keys\n"
        "SELECT loca_id, loca_gl FROM read_ags('site.ags', 'LOCA');\n\n"
        "-- validate in SQL, with the warning tier\n"
        "SELECT * FROM validate_ags('site.ags', warnings := true);\n\n"
        "-- remote files, reading ONE group via HTTP range requests on site.ags.idx\n"
        "LOAD httpfs;\n"
        "SELECT * FROM read_ags('https://example.com/site.ags', 'LOCA');"
    )
    _ext_ok = False
    try:
        _con = duckdb.connect(config={"allow_unsigned_extensions": "true"})
        _con.execute("INSTALL laterite_ags4 FROM community")
        _con.execute("LOAD laterite_ags4")
        _ext_ok = True
    except Exception as _e:
        _err = _e
    if _ext_ok:
        _groups = _con.execute(
            'SELECT "group", n_rows, n_headings FROM ags_groups(?)', [ags_path]
        ).pl()
        _loca = _con.execute(
            "SELECT * FROM read_ags(?, 'LOCA') LIMIT 10", [ags_path]
        ).pl()
        _body = mo.ui.tabs({"ags_groups()": _groups, "read_ags(…, 'LOCA')": _loca})
    else:
        _body = mo.vstack(
            [
                mo.callout(
                    "The `laterite_ags4` extension didn't load in this sandbox "
                    "(it's pinned to a specific DuckDB ABI). Here's the SQL it runs:",
                    kind="neutral",
                ),
                mo.ui.code_editor(_sql, language="sql", disabled=True),
            ]
        )
    mo.vstack(
        [
            mo.md(
                "### 🦆 DuckDB extension — AGS4 as SQL\n\n`INSTALL laterite_ags4 "
                "FROM community` adds `read_ags` / `validate_ags` / `certify_ags` to "
                "DuckDB itself — including **remote** files (httpfs) that read a "
                "single group via HTTP range requests on the remote `.ags.idx`."
            ),
            _body,
        ]
    )
    return


@app.cell
def _(laterite, mo, pl):
    # Produce valid AGS4 from data.
    _res = laterite.build_ags4(
        {
            "PROJ": pl.DataFrame(
                {"PROJ_ID": ["P1"], "PROJ_NAME": ["Built by laterite"]}
            ),
            "LOCA": pl.DataFrame(
                {"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.50, 13.75]}
            ),
        },
        dict_version="4.1.1",
        mode="autofix",
    )
    mo.vstack(
        [
            mo.md(
                "## 7 · Produce — build valid AGS4 from data\n\n`build_ags4(...)` "
                "writes spec-correct AGS4 from polars/pandas frames (or a typed "
                "`PROJ → LOCA` graph), autofixing as it goes."
            ),
            mo.ui.code_editor(_res.text, language="csv", disabled=True),
            mo.download(
                data=_res.bytes,
                filename="built_by_laterite.ags",
                label="⬇ download .ags",
            ),
        ]
    )
    return


@app.cell
def _(ags_path, mo):
    # python-ags4 drop-in
    _out = mo.md("")
    try:
        from laterite import compat as AGS4

        AGS4.set_backend("polars")  # no pandas needed
        _tables, _headings = AGS4.AGS4_to_dataframe(ags_path)
        _out = mo.vstack(
            [
                mo.md(
                    "## 8 · Drop-in for `python-ags4`\n\nSwap one import, keep your "
                    f"code. `compat.__version__` is `{AGS4.__version__}` — it honestly "
                    "identifies as laterite."
                ),
                mo.md(
                    "```diff\n"
                    "- from python_ags4 import AGS4\n"
                    "+ from laterite import compat as AGS4\n"
                    "```"
                ),
                mo.accordion(
                    {
                        f"AGS4_to_dataframe() → {len(_tables)} groups (python-ags4 shape)": mo.ui.tabs(
                            {g: _tables[g] for g in list(_tables)[:4]}
                        )
                    }
                ),
            ]
        )
    except Exception as _e:
        _out = mo.callout(f"compat demo unavailable: {_e}", kind="warn")
    _out
    return


@app.cell
def _(mo):
    mo.accordion(
        {
            "🟩 Node.js — same engine, JS API (runs in Node, not here)": mo.md(
                "```js\n"
                'import { read, validate, buildAgs4 } from "laterite";\n\n'
                'const ags = read("site.ags");        // path | bytes | { text }\n'
                'ags.groups;                           // ["PROJ", "LOCA", …]\n'
                'ags.table("LOCA").getChild("LOCA_GL")?.get(0);   // 12.3 (born-typed)\n\n'
                'const report = validate("site.ags");\n'
                "report.isValid;\n"
                "report.toJson();                      // byte-identical to lat validate --json\n"
                "```\n"
                "`npm install laterite` · a runnable script lives at "
                "`examples/node_tour.mjs`."
            ),
            "⌨️ CLI — `lat` (the Rust binary)": mo.md(
                "```bash\n"
                "pip install laterite          # installs the lat command\n\n"
                "lat validate site.ags         # human report; exit 0 clean / 1 findings\n"
                "lat validate site.ags --json  # machine-readable findings\n"
                "lat fix site.ags              # repair → sibling .fixed.ags\n"
                "lat diff old.ags new.ags      # KEY-aware revision delta\n"
                "```\n"
                "Exit codes: `0` clean · `1` findings · `3` unreadable · `4` not AGS4."
            ),
        }
    )
    return


@app.cell
def _(mo):
    mo.md(
        """
        ---
        **laterite** · one clean-room Rust AGS4 engine, every stack ·
        `pip install laterite` · `npm install laterite` ·
        `INSTALL laterite_ags4 FROM community`

        [GitHub](https://github.com/niko86/laterite) ·
        [PyPI](https://pypi.org/project/laterite/) ·
        [browser validator](https://app.laterite.dev/) ·
        built with [marimo](https://marimo.io) + molab 🦀
        """
    )
    return


if __name__ == "__main__":
    app.run()
