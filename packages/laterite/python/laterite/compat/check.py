"""Mirrors ``python_ags4.check``.

Upstream's ``check.py`` is 43 functions, but 41 of them are ``rule_*`` /
``fyi_*`` implementations of individual numbered AGS4 rules — laterite
implements those in Rust (``laterite-ags4-validator``) and exposes the verdict
through :func:`laterite.compat.AGS4.check_file`, so a drop-in user never calls
them. They are listed by identity in ``compat-surface-gaps.json`` rather than
excluded wholesale, so a NEW ``rule_*`` appearing upstream surfaces as a gap.

What is mirrored here is the part a caller actually reaches for: ``get_TRAN_AGS``,
the two module-level dictionary constants, and the incidental re-exports upstream's
own module carries (``AGS4Error``, ``format_numeric_column`` are importable from
``python_ags4.check`` because it imports them, so they are importable from here).
"""

from ._impl import (
    AGS4Error,
    format_numeric_column,
    get_TRAN_AGS,
)

# Mirrored VERBATIM from python_ags4 1.2.0 (`check.py:39-50`), pinned by
# `PYTHON_AGS4_COMPAT`. Two things about these values are load-bearing and
# easy to "correct" into a divergence:
#
#   * `LATEST_DICT_VERSION` is 4.1.1, NOT the newest key below. Upstream ships a
#     4.2 dictionary and still defaults to 4.1.1 when TRAN_AGS is missing or
#     unrecognised, so a file with no TRAN_AGS validates against 4.1.1 there.
#     `_impl._python_ags4_edition` already reproduces that fallback; this constant
#     is the same decision made visible.
#   * The filenames are IDENTIFIERS here, not paths. laterite generates its
#     dictionaries from `ags_dictionary.json` and ships no `.ags` dictionary
#     files, so joining these to a package directory resolves to nothing. They
#     are mirrored because callers pass and compare the *strings* —
#     `_impl._DICT_FILE_TO_EDITION` maps every one of them back to an edition.
STANDARD_DICT_FILES = {
    "4.0": "Standard_dictionary_v4_0_3.ags",
    "4.0.3": "Standard_dictionary_v4_0_3.ags",
    "4.0.4": "Standard_dictionary_v4_0_4.ags",
    "4.1": "Standard_dictionary_v4_1.ags",
    "4.1.1": "Standard_dictionary_v4_1_1.ags",
    "4.2": "Standard_dictionary_v4_2.ags",
}

LATEST_DICT_VERSION = "4.1.1"

__all__ = [
    "LATEST_DICT_VERSION",
    "STANDARD_DICT_FILES",
    "AGS4Error",
    "format_numeric_column",
    "get_TRAN_AGS",
]
