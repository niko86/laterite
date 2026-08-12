"""``from laterite import compat as AGS4`` — a drop-in for
``from python_ags4 import AGS4``, backed by the clean-room Rust engine.

Backend-configurable. The default is **pandas** (a literal
python-ags4 swap-in returns pandas frames), switchable so ``compat``
can return polars / pyarrow with no pandas installed at all:

    laterite.compat.set_backend("polars")          # process-wide
    AGS4.AGS4_to_dataframe("f.ags", backend="polars")   # per call
    # or env LATERITE_COMPAT_BACKEND=polars

``check_file`` returns the **python-ags4-shaped dict** (rule keys plus
``Metadata`` / ``Summary of data`` / ``General``) so its
``json.dumps`` matches python-ags4's. Validator semantics — and the
deliberate divergences (AGS3 refusal O-30, ``errors='replace'`` →
Rule 1 O-32, ``rename_duplicate_headers`` default O-8) — defer to the
upstream docs: https://gitlab.com/ags-data-format-wg/ags-python-library

**Module shapes mirror upstream's**, so every import shape a real
python-ags4 user writes ports by changing one token, ``python_ags4``
→ ``laterite.compat``::

    from laterite.compat import AGS4              # the submodule
    from laterite.compat.AGS4 import AGS4Error    # third-party code does this
    from laterite.compat.check import get_TRAN_AGS
    from laterite.compat.utils import get_DICT_table_from_json_file
    from laterite.compat.data import load_test_data

The flat namespace below is unchanged and stays the primary surface —
the submodules re-export from it, not the other way round. The engine
lives in ``_impl``; the submodules are the compatibility contract.

Two deliberate non-mirrors: ``ags4_cli`` (laterite ships ``lat``, a
different CLI — see ``compat-surface-gaps.json``), and there is no
top-level ``python_ags4`` import name in this wheel, which is a
permanent non-goal rather than an omission — two distributions owning
``site-packages/python_ags4/`` is a packaging hazard, not a feature.
"""

# Reachable on the flat module before it became a package, because the body
# imported them from `.._errors` — and `BadDictError` is documented
# (`COMPAT.md`'s error-handling section shows `from laterite.compat import
# BadDictError`). Dropping them would have been a silent public-surface break in
# a change whose whole claim is that it is additive, so they are re-exported
# explicitly rather than by accident this time.
from .._errors import Ags4Error, BadDictError
from . import AGS4, check, data, utils
from ._impl import (
    PYTHON_AGS4_COMPAT,
    AGS4_to_dataframe,
    AGS4_to_dict,
    AGS4_to_excel,
    AGS4Error,
    __version__,
    check_file,
    convert_to_numeric,
    convert_to_text,
    count_errors,
    dataframe_to_AGS4,
    excel_to_AGS4,
    format_numeric_column,
    get_ABBR_table_from_json_file,
    get_backend,
    get_DICT_table_from_json_file,
    get_string_dtype,
    get_TRAN_AGS,
    get_TYPE_table_from_json_file,
    get_UNIT_table_from_json_file,
    set_backend,
    set_string_dtype,
    sort_groups,
    write_error_report,
)

__all__ = [
    "AGS4",
    "PYTHON_AGS4_COMPAT",
    "AGS4Error",
    "AGS4_to_dataframe",
    "AGS4_to_dict",
    "AGS4_to_excel",
    "Ags4Error",
    "BadDictError",
    "__version__",
    "check",
    "check_file",
    "convert_to_numeric",
    "convert_to_text",
    "count_errors",
    "data",
    "dataframe_to_AGS4",
    "excel_to_AGS4",
    "format_numeric_column",
    "get_ABBR_table_from_json_file",
    "get_DICT_table_from_json_file",
    "get_TRAN_AGS",
    "get_TYPE_table_from_json_file",
    "get_UNIT_table_from_json_file",
    "get_backend",
    "get_string_dtype",
    "set_backend",
    "set_string_dtype",
    "sort_groups",
    "utils",
    "write_error_report",
]
