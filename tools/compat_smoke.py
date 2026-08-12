"""Smoke test for an isolated ``pip install laterite[compat]``: the drop-in
default is **pyarrow-free**. pyarrow must NOT be importable, and
``AGS4_to_dataframe`` must still return the python-ags4 shape with object-dtype
columns — proving the DuckDB ``.df()`` fallback path (the only environment that
exercises it, since every workspace job installs pyarrow via ``--all-extras``).
Run by nightly.yml's wheel-smoke job in a clean ``[compat]`` venv.
"""

import importlib.util
import io

import pandas as pd
from laterite import compat as AGS4

# The module SHAPES have to survive packaging too, not just the flat namespace:
# a missing `__init__.py` in the built wheel would leave every dev-tree import
# test green and break `from laterite.compat.AGS4 import …` for real users.
from laterite.compat.AGS4 import AGS4Error  # noqa: F401
from laterite.compat.data import load_test_data
from laterite.compat.utils import get_DICT_table_from_json_file  # noqa: F401

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

# The shipped sample must be readable through the same pyarrow-free path.
_tables, _ = load_test_data()
assert "LOCA" in _tables, sorted(_tables)
print(
    f"[compat] OK — pandas {pd.__version__}, pyarrow-free, "
    "object-dtype drop-in via DuckDB verified"
)
