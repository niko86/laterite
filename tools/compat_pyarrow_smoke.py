"""Smoke test for an isolated ``pip install laterite[compat,pyarrow]``: the
opt-in accelerator. pyarrow IS importable; ``AGS4_to_dataframe`` still returns
object dtype by default (byte-identical drop-in), and ``string_dtype="string"``
now resolves to pandas' Arrow-backed ``str`` dtype (unreachable pyarrow-free).
Proves the extra resolves and both hops work in a clean install.
"""

import importlib.util
import io

import pandas as pd
import pyarrow as pa
from laterite import compat as AGS4

assert importlib.util.find_spec("pyarrow") is not None, "pyarrow should be present"

src = (
    '"GROUP","PROJ"\r\n'
    '"HEADING","PROJ_ID"\r\n'
    '"UNIT",""\r\n'
    '"TYPE","ID"\r\n'
    '"DATA","P1"\r\n'
)

# Default: still object dtype (the pyarrow `to_pandas` fast hop, byte-identical).
tables, _ = AGS4.AGS4_to_dataframe(io.StringIO(src))
proj = tables["PROJ"]
assert list(proj.columns) == ["HEADING", "PROJ_ID"], proj.columns
assert all(str(d) == "object" for d in proj.dtypes), list(proj.dtypes)

# Opt-in: pandas' Arrow-backed str dtype (the pandas-3 baseline).
tables_s, _ = AGS4.AGS4_to_dataframe(io.StringIO(src), string_dtype="string")
proj_s = tables_s["PROJ"]
assert all(isinstance(d, pd.StringDtype) for d in proj_s.dtypes), list(proj_s.dtypes)
# na_value=NaN variant stays matched by the object selector (drop-in contract).
assert list(proj_s.select_dtypes(include="object").columns) == [
    "HEADING",
    "PROJ_ID",
], "string_dtype='string' must remain object-selectable (na_value=NaN)"

print(
    f"[compat,pyarrow] OK — pandas {pd.__version__}, pyarrow "
    f"{pa.__version__}, object default + string_dtype='string' verified"
)
