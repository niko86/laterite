"""Mirrors ``python_ags4.AGS4``.

All 13 public names upstream's ``AGS4.py`` exposes, re-exported from
``laterite.compat``'s implementation so that
``from python_ags4.AGS4 import AGS4Error`` — which real third-party code
writes, not just ``from python_ags4 import AGS4`` — ports by changing one
token. A flat module could not serve that import shape at all.

Nothing is implemented here: this file IS the compatibility contract, and
``_impl`` is where the code lives.
"""

from ._impl import (
    AGS4_to_dataframe,
    AGS4_to_dict,
    AGS4_to_excel,
    AGS4Error,
    check_file,
    convert_to_numeric,
    convert_to_text,
    count_errors,
    dataframe_to_AGS4,
    excel_to_AGS4,
    format_numeric_column,
    sort_groups,
    write_error_report,
)

__all__ = [
    "AGS4Error",
    "AGS4_to_dataframe",
    "AGS4_to_dict",
    "AGS4_to_excel",
    "check_file",
    "convert_to_numeric",
    "convert_to_text",
    "count_errors",
    "dataframe_to_AGS4",
    "excel_to_AGS4",
    "format_numeric_column",
    "sort_groups",
    "write_error_report",
]
