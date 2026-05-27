# laterite-ags5

Experimental AGS5 surface for [laterite](https://github.com/niko86/laterite).

`.ags5db` is a single-file DuckDB-backed format for AGS geotechnical
data — typed columns, content-addressed UUID7 keys, lossless AGS4
round-trip. This package ships the read / write / query API.

```bash
pip install "laterite[ags5]"
```

The `[ags5]` extra pulls this wheel alongside the base `laterite`
wheel. It links bundled DuckDB (~50 MB), which is why it's opt-in.
Plain `pip install laterite` stays light for AGS4-only workflows.

```python
from laterite.ags5db import convert, read_db, query

# AGS4 → .ags5db
convert("delivery.ags", "delivery.ags5db")

# Read a typed PROJ tree back
proj = read_db("delivery.ags5db")

# Query the file
rows = query("delivery.ags5db", group="LOCA", where=["LOCA_GL>50"])
```

**Pre-alpha.** The `.ags5db` format and Python API are subject to
change. AGS4 input/output (via convert/export) is stable; native
AGS5 authoring is the experimental piece.

Full docs at <https://github.com/niko86/laterite>.
