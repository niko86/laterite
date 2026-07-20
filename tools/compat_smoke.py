"""Smoke test for an isolated ``pip install laterite[compat]``: the drop-in
default is **pyarrow-free**. pyarrow must NOT be importable, and
``AGS4_to_dataframe`` must still return the python-ags4 shape with object-dtype
columns — proving the DuckDB ``.df()`` fallback path (the only environment that
exercises it, since every workspace job installs pyarrow via ``--all-extras``).
Run by ci.yml's wheel-smoke job in a clean ``[compat]`` venv.
"""

import importlib.util
import io

import pandas as pd
from laterite import compat as AGS4

# The invariant: `[compat]` alone does not pull pyarrow.
assert importlib.util.find_spec("pyarrow") is None, (
    "pyarrow is importable in a bare [compat] install — it must be an opt-in "
    "accelerator ([compat,pyarrow]/[all]/[pyarrow]), not a [compat] dependency."
)

src = (
    '"GROUP","PROJ"\r\n'
    '"HEADING","PROJ_ID"\r\n'
    '"UNIT",""\r\n'
    '"TYPE","ID"\r\n'
    '"DATA","P1"\r\n'
)
tables, _ = AGS4.AGS4_to_dataframe(io.StringIO(src))
proj = tables["PROJ"]
assert list(proj.columns) == ["HEADING", "PROJ_ID"], proj.columns
assert all(str(d) == "object" for d in proj.dtypes), list(proj.dtypes)
print(
    f"[compat] OK — pandas {pd.__version__}, pyarrow-free, "
    "object-dtype drop-in via DuckDB verified"
)
