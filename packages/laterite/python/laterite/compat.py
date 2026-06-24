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
"""

from __future__ import annotations

import datetime
import hashlib
import os
import warnings
from pathlib import Path
from typing import Any

from . import _compat_desc
from . import _laterite_native as _native
from ._errors import Ags4Error, BadDictError, raise_for
from ._frames import (
    get_default_backend,
    materialize,
    resolve_backend,
    set_default_backend,
)

# Honest identity strings. python-ags4 callers reach `__version__` via
# the parity-runner shim's `python_ags4.__version__` (and laterite
# users via `laterite.compat.__version__`). The `+local` segment is
# PEP 440 "local version" syntax — parseable as semver-ish,
# machine-comparable via `packaging.version`, and unambiguous about
# provenance.
#
# `PYTHON_AGS4_COMPAT` is the same parity-pin exposed as a
# programmatic constant. Downstream tools that want to know "which
# python-ags4 was this parity-tested against?" should read this
# directly rather than parsing the local-version segment of
# `__version__`. The two stay in sync (a paired test enforces this).
#
# Phase-1 of the migration path documented in COMPAT.md: today both
# the local-version pin AND the constant exist. When laterite matures
# (phase 2), the pin will drop out of `__version__` while the constant
# stays. Phase 3 drops the constant too. See COMPAT.md →
# "Versioning migration path".
#
# Why not just claim `__version__ = "1.2.0"` (which would close 5
# parity tests): downstream tools reading the JSON report or
# `__version__` would silently believe they were talking to
# python-ags4. A misidentified validator is much harder to debug than
# a parity test failing.
PYTHON_AGS4_COMPAT = "1.2.0"
__version__ = f"0.5.1+compat.python-ags4.{PYTHON_AGS4_COMPAT}"

# Human-readable Metadata.Checker — same intent, prose form.
_CHECKER_STRING = (
    "laterite 0.5.1 — compat: python-ags4 1.2.0 — clean-room laterite_ags4_validator engine"
)

# python-ags4 maps these version strings → bundled standard dict files;
# we map them to the engine's --dict-version. A *path* argument is
# inverted back to an edition via the bundled filename (mirrors
# tools/py_ags4_check_json.py's _DICT_BY_EDITION); anything else is a
# non-bundled external dict → BadDictError (O-28, never silent).
_VERSION_STRINGS = {"4.2", "4.1.1", "4.1", "4.0.4", "4.0.3", "4.0"}
# python-ags4 ships these in the package; the no-`v` form appears in
# their own tests (`Standard_dictionary_{dict_version}.ags` f-string
# pattern, line 35 of their tests/test_ags4.py) — accept both.
_DICT_FILE_TO_EDITION = {
    "Standard_dictionary_v4_0_3.ags": "4.0.3",
    "Standard_dictionary_v4_0_4.ags": "4.0.4",
    "Standard_dictionary_v4_1.ags": "4.1",
    "Standard_dictionary_v4_1_1.ags": "4.1.1",
    "Standard_dictionary_v4_2.ags": "4.2",
    "Standard_dictionary_4_0_3.ags": "4.0.3",
    "Standard_dictionary_4_0_4.ags": "4.0.4",
    "Standard_dictionary_4_1.ags": "4.1",
    "Standard_dictionary_4_1_1.ags": "4.1.1",
    "Standard_dictionary_4_2.ags": "4.2",
}


def set_backend(name: str) -> None:
    """Set the process-wide compat dataframe backend
    (``pandas`` | ``polars`` | ``pyarrow``)."""
    set_default_backend(name)


def get_backend() -> str:
    return get_default_backend()


# --- readers --------------------------------------------------------


def _primitives(filepath_or_buffer: Any, encoding: str) -> dict:
    if hasattr(filepath_or_buffer, "read"):
        text = filepath_or_buffer.read()
        if isinstance(text, bytes):
            text = text.decode(encoding, errors="replace")
        return raise_for(_native.parse_primitives(text=text))
    return raise_for(_native.parse_primitives(path=str(filepath_or_buffer)))


def _strict_pre_check(filepath_or_buffer: Any, encoding: str) -> None:
    """Compat-only pre-parse pass mirroring python-ags4's hard raises.

    The native laterite parser is deliberately lenient — duplicate
    GROUP declarations are silently merged, ragged DATA rows pass
    through, etc. — because those are findings to report, not crashes.
    python-ags4 raises hard. Compat callers expect the hard raise.

    Scans the file's raw text once and raises ``Ags4Error`` on:
    - duplicate group declarations (same GROUP code on two lines);
    - DATA rows whose field count differs from the HEADING row's.
    """
    if hasattr(filepath_or_buffer, "read"):
        text = filepath_or_buffer.read()
        if isinstance(text, bytes):
            text = text.decode(encoding, errors="replace")
        # Rewind file-like buffers so the subsequent parse can re-read.
        if hasattr(filepath_or_buffer, "seek"):
            try:
                filepath_or_buffer.seek(0)
            except OSError, ValueError:
                pass
    else:
        with open(filepath_or_buffer, encoding=encoding, errors="replace") as fh:
            text = fh.read()

    import csv
    import io

    seen_groups: dict[str, int] = {}
    heading_count: int | None = None
    current_group: str | None = None
    # csv.reader handles quoted commas correctly (AGS4 Rule 5 requires
    # data fields to be quoted, so embedded commas are common in DESC
    # columns). A naïve split() would mis-count fields and fire false
    # ragged-row raises on legitimate files.
    reader = csv.reader(io.StringIO(text), strict=False)
    for lineno, fields in enumerate(reader, start=1):
        if not fields or not any(f.strip() for f in fields):
            continue
        descriptor = fields[0]
        if descriptor == "GROUP" and len(fields) >= 2:
            grp = fields[1]
            if grp in seen_groups:
                raise Ags4Error(
                    f'"{grp}" group duplicated in Line {lineno} '
                    f"(first seen on Line {seen_groups[grp]}); "
                    "therefore please combine all duplicate groups."
                )
            seen_groups[grp] = lineno
            current_group = grp
            heading_count = None
        elif descriptor == "HEADING":
            heading_count = len(fields)
        elif descriptor == "DATA" and heading_count is not None:
            if len(fields) != heading_count:
                raise Ags4Error(
                    f"Line {lineno} of group {current_group!r} does not "
                    "have the same number of entries as the HEADING row."
                )


def _rename_dups(headings: list[str], rename: bool, group: str) -> list[str]:
    """python-ags4's duplicate-heading handling (AGS4.py:143-165):
    default renames ``X`` → ``X_1``, ``X_2``…; ``rename=False`` raises."""
    if len(headings) == len(set(headings)):
        return headings
    if not rename:
        raise Ags4Error(f"HEADER row in {group} has duplicate entries")
    seen: dict[str, int] = {}
    out: list[str] = []
    for h in headings:
        if h not in seen:
            seen[h] = 0
            out.append(h)
        else:
            seen[h] += 1
            out.append(f"{h}_{seen[h]}")
    return out


def AGS4_to_dict(
    filepath_or_buffer: Any,
    encoding: str = "utf-8",
    get_line_numbers: bool = False,
    rename_duplicate_headers: bool = True,
):
    """Load an AGS4 file to ``(data, headings[, line_numbers])`` — the
    python-ags4 ``AGS4_to_dict`` shape: ``data[GROUP]`` maps each
    column (``HEADING`` first) to ``[unit, type, *data]``."""
    _strict_pre_check(filepath_or_buffer, encoding)
    p = _primitives(filepath_or_buffer, encoding)
    data: dict[str, dict[str, list[str]]] = {}
    headings: dict[str, list[str]] = {}
    line_numbers: dict[str, dict[str, Any]] = {}

    for code in p["group_order"]:
        g = p["groups"][code]
        cols = [
            "HEADING",
            *_rename_dups(list(g["headings"]), rename_duplicate_headers, code),
        ]
        n = len(cols)
        unit = (["UNIT", *g["units"]] + [""] * n)[:n]
        typ = (["TYPE", *g["types"]] + [""] * n)[:n]
        rows = [(["DATA", *r["values"]] + [""] * n)[:n] for r in g["rows"]]

        hdr = list(cols)
        if get_line_numbers:
            hdr = [*cols, "line_number"]
            unit = [*unit, g["unit_line"]]
            typ = [*typ, g["type_line"]]
            rows = [[*row, r["line"]] for row, r in zip(rows, g["rows"], strict=False)]

        d: dict[str, list] = {c: [] for c in hdr}
        for rec in (unit, typ, *rows):
            for i, c in enumerate(hdr):
                d[c].append(rec[i])
        data[code] = d
        headings[code] = hdr
        line_numbers[code] = {
            "GROUP": g["group_line"],
            "HEADING": g["heading_line"] if g["heading_line"] is not None else "-",
        }

    if get_line_numbers:
        return data, headings, line_numbers
    return data, headings


def AGS4_to_dataframe(
    filepath_or_buffer: Any,
    encoding: str = "utf-8",
    get_line_numbers: bool = False,
    rename_duplicate_headers: bool = True,
    only_groups: list[str] | None = None,
    backend: str | None = None,
):
    """Load an AGS4 file to ``(tables, headings[, line_numbers])``.

    ``tables[GROUP]`` is a dataframe in the configured backend (default
    pandas) with a leading ``HEADING`` column and ``UNIT``/``TYPE``
    pseudo-rows then ``DATA`` — byte-identical *shape* to python-ags4's
    ``AGS4_to_dataframe`` (the contract ``ags5_ags4/codec.py`` relies
    on: ``df.iloc[2:]`` + dropping ``HEADING``)."""
    be = resolve_backend(backend)
    out = AGS4_to_dict(
        filepath_or_buffer, encoding, get_line_numbers, rename_duplicate_headers
    )
    if get_line_numbers:
        data, headings, line_numbers = out
    else:
        data, headings = out

    keys = only_groups if only_groups else list(data)
    tables: dict[str, Any] = {}
    for k in keys:
        cols = headings[k]
        rows = (
            list(zip(*[data[k][c] for c in cols], strict=False))
            if data[k][cols[0]]
            else []
        )
        import polars as pl

        df = pl.DataFrame(
            {c: [row[i] for row in rows] for i, c in enumerate(cols)}
            if rows
            else {c: [] for c in cols}
        )
        tables[k] = materialize(df, be)

    if get_line_numbers:
        return tables, headings, line_numbers
    return tables, headings


def AGS4_to_dataframe_AGS3(*_a, **_k):
    """The clean-room engine deliberately refuses AGS3 rather than
    silently validating it against an AGS4 schema (O-30)."""
    from ._errors import UnsupportedEditionError

    raise UnsupportedEditionError(
        "AGS3 is not supported (clean-room refuses it rather than silently "
        "validating against an AGS4 schema — see O-30)"
    )


# --- writer ---------------------------------------------------------


def _to_polars(df: Any):
    """Any python-ags4-shaped frame (pandas / polars) -> polars, without
    narwhals — compat's cross-backend shim now branches on the native type.
    A polars frame passes through; a pandas frame converts via from_pandas."""
    import polars as pl

    if isinstance(df, pl.DataFrame):
        return df
    return pl.from_pandas(df) if hasattr(df, "to_numpy") else pl.DataFrame(df)


def dataframe_to_AGS4(
    tables: dict[str, Any],
    headings: dict[str, list[str]],
    filepath: str | Path,
    mode: str = "w",
    index: bool = False,
    encoding: str = "utf-8",
    warnings: bool = True,
) -> None:
    """Write python-ags4-shaped frames back to a spec-correct AGS4
    file (CRLF, every field quoted, ``"``→``""``; blank line between
    groups). Any backend in (pandas / polars).

    The per-cell stringify is **all-Rust**: each frame is normalised to
    polars and column-selected to the ``headings`` order, then handed to
    the native emitter as a pyarrow-free Arrow capsule — Rust reproduces
    python-ags4's ``"" if v is None else str(v)`` and serialises (the
    column names become the HEADING line; data rows carry their tag in
    the ``"HEADING"`` column, exactly as stored)."""
    blocks: list[tuple[str, Any]] = []
    for code, df in tables.items():
        nf = _to_polars(df)
        cols = [c for c in (headings.get(code) or []) if c in nf.columns] or list(nf.columns)
        blocks.append((code, nf.select(cols)))
    text = _native.emit_ags4_compat(blocks)
    m = "ab" if mode == "a" else "wb"
    with open(filepath, m) as f:
        f.write(text.encode(encoding))


# --- numeric coercion ----------------------------------------------


def convert_to_numeric(dataframe: Any) -> Any:
    """python-ags4 ``convert_to_numeric`` parity: coerce columns whose
    TYPE row contains ``DP|MC|SF|SCI`` to numeric (bad cells → null),
    drop the UNIT+TYPE rows, reset index. Pandas in / pandas out."""
    import polars as pl

    # Normalise to polars ONCE up front (same pattern as convert_to_text
    # and the DATA branch below). Reading the TYPE row off the native
    # frame directly was the bug: a filtered pandas frame keeps its
    # original index, so positional `tr[c][0]` raised KeyError on the
    # default pandas backend. Polars positional indexing is safe.
    pf = _to_polars(dataframe)

    type_row = pf.filter(pl.col("HEADING") == "TYPE")
    numeric: list[str] = []
    if type_row.height:
        for c in pf.columns:
            if c == "HEADING":
                continue
            t = str(type_row[c][0])
            if any(tok in t for tok in ("DP", "MC", "SF", "SCI")):
                numeric.append(c)

    data = pf.filter(pl.col("HEADING") == "DATA")
    if numeric:
        data = data.with_columns(
            pl.col(c).cast(pl.Float64, strict=False) for c in numeric
        )
    return materialize(data, get_default_backend())


def _try_dict_version(dictionary: Any) -> str | None:
    """Non-raising variant of ``_dict_version_arg``: returns the
    edition string for a recognised version arg or bundled file
    basename, else ``None`` (so the caller can try a different
    interpretation — e.g. an external AGS4 dict file)."""
    if dictionary is None:
        return None
    s = str(dictionary)
    if s in _VERSION_STRINGS:
        return s
    base = os.path.basename(s)
    return _DICT_FILE_TO_EDITION.get(base)


def _unit_type_from_external_dict_file(
    path: str,
) -> dict[str, tuple[str, str]]:
    """Parse an AGS4 file containing a DICT group; return
    ``{heading_name: (unit, type)}``. Bundled standard dicts ship in
    the same AGS4 format, so this also works on
    ``python_ags4/Standard_dictionary_*.ags``-style paths that the
    caller passes directly without going through the
    edition-string handling.
    """
    data, _ = AGS4_to_dict(path)
    dict_group = data.get("DICT")
    if dict_group is None:
        return {}
    headings = dict_group.get("DICT_HDNG", [])
    units = dict_group.get("DICT_UNIT", [])
    types = dict_group.get("DICT_DTYP", [])
    out: dict[str, tuple[str, str]] = {}
    # Row 0 is UNIT (in AGS4_to_dict shape — see compat.py:101),
    # row 1 is TYPE; DATA rows start at index 2.
    for i in range(2, len(headings)):
        h = headings[i]
        if not h:
            continue
        u = units[i] if i < len(units) else ""
        t = types[i] if i < len(types) else ""
        out[h] = (u, t)
    return out


def _resolve_dict_unit_type(dataframe: Any, edition: str) -> dict[str, tuple[str, str]]:
    """Look up UNIT/TYPE for the columns in ``dataframe`` from the
    bundled standard dictionary for ``edition``.

    Infers the AGS4 group code from the columns' 4-letter prefix
    (Rule 19's heading shape is ``GRP_FLD``). Returns a
    ``{heading: (unit, type)}`` map covering only standard headings —
    non-standard ones are left for the file's own DICT group (which
    `convert_to_text` won't have here either, by definition).
    """
    cols = _columns_of(dataframe)
    code = _infer_group_code(cols)
    if code is None:
        return {}
    return dict(_native.dict_group_unit_type(edition, code))


def _columns_of(dataframe: Any) -> list[str]:
    """Cross-backend column-name lookup. pandas/polars expose
    ``.columns``; pyarrow Tables use ``.column_names``; fall back to
    ``.to_pydict().keys()`` for anything else."""
    if hasattr(dataframe, "columns"):
        return list(dataframe.columns)
    if hasattr(dataframe, "column_names"):
        return list(dataframe.column_names)
    if hasattr(dataframe, "to_pydict"):
        return list(dataframe.to_pydict().keys())
    raise TypeError(f"cannot enumerate columns of {type(dataframe).__name__}")


def _infer_group_code(cols: list[str]) -> str | None:
    """The 4-letter group prefix shared by data columns. Returns ``None``
    if columns disagree or there are none — caller falls back to leaving
    UNIT/TYPE alone."""
    for c in cols:
        if c == "HEADING":
            continue
        if len(c) >= 5 and c[4] == "_" and c[:4].isupper():
            return c[:4]
    return None


def _inject_unit_type_from_dict(
    pf: Any, dict_unit_type: dict[str, tuple[str, str]]
) -> Any:
    """Insert / overwrite UNIT and TYPE rows from the dictionary lookup.

    python-ags4's behaviour: when the dictionary is supplied, its
    UNIT/TYPE *override* the existing rows (or create them if the frame
    has none). DATA cells stay untouched.
    """
    import polars as pl

    cols = pf.columns
    heading = pf["HEADING"].to_list() if "HEADING" in cols else []
    has_unit = "UNIT" in heading
    has_type = "TYPE" in heading

    def cell(col: str, marker: str) -> str:
        if col == "HEADING":
            return marker
        pair = dict_unit_type.get(col)
        if pair is None:
            return ""
        return pair[0] if marker == "UNIT" else pair[1]

    unit_row = {c: cell(c, "UNIT") for c in cols}
    type_row = {c: cell(c, "TYPE") for c in cols}

    # Build a fresh frame: UNIT row, TYPE row, then DATA rows from the
    # input. Existing UNIT/TYPE rows in the input are dropped (the
    # dictionary's values win, matching python-ags4 behaviour).
    if has_unit or has_type:
        pf = pf.filter(~pl.col("HEADING").is_in(["UNIT", "TYPE"]))
    prefix = pl.DataFrame([unit_row, type_row])
    return pl.concat([prefix, pf], how="vertical_relaxed")


def convert_to_text(dataframe: Any, dictionary: str | None = None) -> Any:
    """Format numeric columns back to AGS4 text precision per their
    TYPE (``nDP`` → fixed decimals, ``nSCI`` → scientific, ``nSF`` →
    significant figures). UNIT/TYPE rows pass through unchanged.

    ``dictionary`` accepts either a bundled-edition version string
    (``"4.2"`` / ``"4.1.1"`` / ``"4.1"`` / ``"4.0.4"`` / ``"4.0.3"``) or
    the bundled-dict file basename (``"Standard_dictionary_v4_1.ags"``
    / ``"Standard_dictionary_4_1.ags"``). When supplied, UNIT/TYPE rows
    are recovered from the standard dictionary for columns whose
    headings are defined there — useful after
    :func:`convert_to_numeric` (which drops UNIT/TYPE). Genuinely
    external dict files (not one of the bundled paths) still raise
    :class:`BadDictError` per O-28.
    """
    import polars as pl

    # Recover UNIT/TYPE rows from the dictionary when supplied.
    # Order: version string → bundled basename → external AGS4 dict
    # file. External files are parsed via the regular AGS4 reader (the
    # bundled dicts ship as .ags themselves — same shape).
    dict_unit_type: dict[str, tuple[str, str]] | None = None
    if dictionary is not None:
        edition = _try_dict_version(dictionary)
        if edition is not None:
            if edition == "4.0":
                edition = "4.0.4"
            dict_unit_type = _resolve_dict_unit_type(dataframe, edition)
        elif os.path.isfile(str(dictionary)):
            dict_unit_type = _unit_type_from_external_dict_file(str(dictionary))
        else:
            # No recognised version, no readable file — still O-28 (but
            # now narrower than before — version strings + bundled paths
            # + real DICT-shaped files all work).
            raise BadDictError(
                f"dictionary {dictionary!r} is neither a bundled "
                f"version string {sorted(_VERSION_STRINGS)} nor an "
                "AGS4 dict file we can read."
            )
    pf = _to_polars(dataframe)
    # Capture which columns were numeric in the *input* — after the
    # UNIT/TYPE inject below, polars will unify dtypes to String (the
    # inject adds string rows). python-ags4 only reformats columns that
    # are numeric (its `f"{x:.2f}"` TypeErrors on strings and the
    # outer except leaves the column alone). We get the same semantics
    # by gating on the pre-inject dtype.
    _NUMERIC_DTYPES = (
        pl.Float32,
        pl.Float64,
        pl.Int8,
        pl.Int16,
        pl.Int32,
        pl.Int64,
        pl.UInt8,
        pl.UInt16,
        pl.UInt32,
        pl.UInt64,
    )
    numeric_cols = {c for c in pf.columns if pf[c].dtype in _NUMERIC_DTYPES}
    if dict_unit_type is not None:
        pf = _inject_unit_type_from_dict(pf, dict_unit_type)
    type_row = pf.filter(pl.col("HEADING") == "TYPE")
    if type_row.height == 0:
        raise Ags4Error(
            "Cannot convert to text as UNIT and/or TYPE row(s) are missing."
        )

    def fmt(v: Any, t: str) -> str:
        if v is None or v == "":
            return ""
        try:
            x = float(v)
        except TypeError, ValueError:
            return str(v)
        if "DP" in t:
            return f"{x:.{int(t.strip('DP') or 0)}f}"
        if "SCI" in t:
            return f"{x:.{int(t.strip('SCI') or 0)}E}"
        if "SF" in t:
            return _format_sf(x, t)
        return str(v)

    out_cols = {}
    for c in pf.columns:
        if c == "HEADING":
            out_cols[c] = pf[c].to_list()
            continue
        t = str(type_row[c][0])
        col = pf[c].to_list()
        hdr = pf["HEADING"].to_list()
        is_numeric = c in numeric_cols
        out_cols[c] = [
            (fmt(v, t) if is_numeric else ("" if v is None else str(v)))
            if h == "DATA"
            else ("" if v is None else str(v))
            for v, h in zip(col, hdr, strict=False)
        ]
    return materialize(pl.DataFrame(out_cols), get_default_backend())


# --- validation -----------------------------------------------------


def _dict_version_arg(standard_AGS4_dictionary: Any) -> str | None:
    if standard_AGS4_dictionary is None:
        return None
    s = str(standard_AGS4_dictionary)
    if s in _VERSION_STRINGS:
        return "4.0.4" if s == "4.0" else s
    base = os.path.basename(s)
    if base in _DICT_FILE_TO_EDITION:
        return _DICT_FILE_TO_EDITION[base]
    raise BadDictError(
        f"external standard dictionary {s!r} is not a bundled edition "
        "(O-28): pass None for TRAN_AGS auto-detect or a bundled version "
        f"string {sorted(_VERSION_STRINGS)}"
    )


# python-ags4 1.2.0's edition selection (check.py STANDARD_DICT_FILES + the
# LATEST_DICT_VERSION fallback): an exact string in the table wins; anything
# else — missing / bare "4" / unknown — falls back to 4.1.1. NB "4.0" maps to
# 4.0.3 (the older patch); laterite picks 4.0.4 (O-30), which is what the O-42
# divergence turns on. Mirrored here so compat can WARN (never silently diverge)
# when its auto-resolved edition differs — without running python-ags4.
_PYAGS4_STANDARD_DICT = {
    "4.0": "4.0.3",
    "4.0.3": "4.0.3",
    "4.0.4": "4.0.4",
    "4.1": "4.1",
    "4.1.1": "4.1.1",
    "4.2": "4.2",
}


def _python_ags4_edition(tran_ags: Any) -> str:
    """The edition python-ags4 1.2.0 would validate `tran_ags` against."""
    if tran_ags is None:
        return "4.1.1"  # python: TRAN_AGS missing -> LATEST_DICT_VERSION
    return _PYAGS4_STANDARD_DICT.get(str(tran_ags).strip(), "4.1.1")


def check_file(
    filepath_or_buffer: Any,
    standard_AGS4_dictionary: Any = None,
    rename_duplicate_headers: bool = True,
    encoding: str = "utf-8",
    match_python_ags4_wording: bool = True,
):
    """Validate an AGS4 file. Returns the **python-ags4-shaped dict**
    (``"AGS Format Rule N"`` keys plus ``General`` / ``Summary of
    data`` / ``Metadata``) so ``json.dumps`` of it matches
    python-ags4's. Rule findings come from the clean-room engine.

    With ``match_python_ags4_wording=True`` (default) the rule ``desc``
    strings are translated into python-ags4's phrasings — what callers
    porting from python-ags4 want. Pass ``False`` to see laterite's
    own (more precise) wording, which is what the native API
    (``laterite.Validator``) and the Rust ``lat-check`` CLI return."""
    dv = _dict_version_arg(standard_AGS4_dictionary)

    is_path = not hasattr(filepath_or_buffer, "read")
    # compat defaults `include_fyi=True` — python-ags4 emits FYI keys
    # (`"FYI"`, `"FYI (Related to Rule 16)"`, …) and tests assert on
    # them. The native API + Rust CLI keep the engine default off,
    # preserving the existing terse-by-default behaviour there.
    if is_path:
        r = raise_for(
            _native.run_check(
                path=str(filepath_or_buffer),
                dict_version=dv,
                check_files=True,
                include_fyi=True,
                encoding=encoding,
            )
        )
        p = raise_for(
            _native.parse_primitives(path=str(filepath_or_buffer), encoding=encoding)
        )
    else:
        text = filepath_or_buffer.read()
        if isinstance(text, bytes):
            text = text.decode(encoding, errors="replace")
        r = raise_for(_native.run_check(text=text, dict_version=dv, include_fyi=True))
        p = raise_for(_native.parse_primitives(text=text))

    # Transparency (#190 / O-30 / O-42): laterite resolves a few ambiguous
    # TRAN_AGS strings to a DIFFERENT edition than python-ags4 would (e.g.
    # "4.0" -> 4.0.4 here vs python's 4.0.3, "4" -> 4.0.4 vs 4.1.1). On an
    # AUTO-resolved file (no explicit dict), WARN when the chosen edition
    # differs so a drop-in caller isn't silently surprised by a divergent
    # verdict. Deliberately NOT added to the returned dict — that preserves
    # python-ags4 output-parity (the 122 oracle); a strict-fidelity mode that
    # replicates python's choice is tracked as future work.
    if standard_AGS4_dictionary is None:
        _lat_ed = r.get("dict_version")
        _py_ed = _python_ags4_edition(p.get("tran_ags"))
        if _lat_ed and _lat_ed != _py_ed:
            warnings.warn(
                f"laterite resolved TRAN_AGS={p.get('tran_ags')!r} to edition "
                f"{_lat_ed}; python-ags4 1.2.0 would use {_py_ed} (OBSERVATIONS "
                f"O-30/O-42). Verdicts may differ on edition-specific rules "
                f"(e.g. Rule 9 headings, Rule 10c parent links).",
                UserWarning,
                stacklevel=2,
            )

    ags_errors: dict[str, list[dict]] = {}

    def add(rule: str, line: Any, group: str, desc: str) -> None:
        ags_errors.setdefault(rule, []).append(
            {"line": line, "group": group, "desc": desc}
        )

    for f in r["findings"]:
        desc = f["desc"]
        if match_python_ags4_wording:
            desc = _compat_desc.translate(f["rule"], f["group"], desc)
        add(f["rule"], f["line"] if f["line"] is not None else "-", f["group"], desc)

    if "AGS Format Rule 1" in ags_errors:
        add(
            "General",
            "",
            "",
            "AGS4 Rule 1 is interpreted as allowing both standard ASCII "
            "characters (Unicode code points 0-127) and extended ASCII "
            "characters (Unicode code points 160-255).",
        )

    # Summary of data (python-ags4 check.get_data_summary parity).
    groups = list(p["group_order"])
    tran = p.get("tran_ags")
    add(
        "Summary of data",
        "",
        "",
        f'TRAN_AGS: "{tran}"' if tran else "TRAN_AGS: Not found",
    )
    add(
        "Summary of data",
        "",
        "",
        f"{len(groups)} groups identified in file: {' '.join(groups)}",
    )
    empty = [c for c in groups if not p["groups"][c]["rows"]]
    if empty:
        add(
            "Summary of data",
            "",
            "",
            f"{len(empty)} group(s) do not have any data: {' '.join(empty)}",
        )
    if "LOCA" in p["groups"]:
        add(
            "Summary of data",
            "",
            "",
            f"{len(p['groups']['LOCA']['rows'])} data row(s) in LOCA group",
        )
    add("Summary of data", "", "", f"Optional DICT group present? {'DICT' in groups}")
    add("Summary of data", "", "", f"Optional FILE group present? {'FILE' in groups}")

    # Metadata (same labels/order python-ags4 check.add_meta_data emits).
    if is_path:
        fp = Path(filepath_or_buffer)
        add("Metadata", "File Name", "", fp.name)
        add("Metadata", "File Size", "", f"{int(fp.stat().st_size / 1024)} kB")
    add("Metadata", "Checker", "", _CHECKER_STRING)
    # Always emit Dictionary — python-ags4 does (via TRAN_AGS auto-detect
    # when no explicit dict was passed). Matching the *position* matters:
    # `test_sha256_hash` asserts SHA lands at `Metadata[6]`, which only
    # holds when Dictionary is present at [3].
    if r.get("dict_version"):
        add(
            "Metadata",
            "Dictionary",
            "",
            f"Standard_dictionary_v{r['dict_version'].replace('.', '_')}.ags",
        )
    add(
        "Metadata",
        "Time (UTC)",
        "",
        datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%d %H:%M:%S"),
    )
    add("Metadata", "File encoding", "", encoding)

    sha = hashlib.sha256()
    if is_path:
        with open(
            filepath_or_buffer, newline="", encoding=encoding, errors="replace"
        ) as fh:
            for line in fh:
                sha.update(line.encode(encoding))
    add("Metadata", "SHA256 hash", "", sha.hexdigest())

    return ags_errors


# --- Stage 2b: Rust-backed AGS4 ↔ XLSX conversion -------------------


def AGS4_to_excel(
    input_file: str,
    output_file: str,
    encoding: str = "utf-8",
    rename_duplicate_headers: bool = True,
    sorting_strategy: str | None = None,
) -> None:
    """Convert an AGS4 file to an XLSX spreadsheet, one sheet per group.

    Rust-backed via `laterite_excel::ags4_to_excel` (uses
    `rust_xlsxwriter`); openpyxl never enters the dep graph. Output
    matches python-ags4's column layout: HEADING column first, AGS
    heading names as headers, UNIT / TYPE / DATA pseudo-rows
    preserved, column widths `min(max(13, max_str_len + 1), 75)`.

    Args:
        input_file: Path to the AGS4 file.
        output_file: Path to write the XLSX to.
        encoding: Accepted for python-ags4 API compat; ignored — the
            Rust parser auto-detects UTF-8 / Latin-1 (Rule 1).
        rename_duplicate_headers: Accepted for python-ags4 API compat;
            ignored — the Rust parser handles dup headings.
        sorting_strategy: ``None`` (preserve source order),
            ``"dictionary"``, ``"alphabetical"``, or ``"hierarchical"``.
            When set, calls `sort_groups` before writing.

    Raises:
        AGS4Error: if the input contains no valid AGS4 group data.
    """
    del encoding, rename_duplicate_headers  # python-ags4 API parity

    ordered_keys: list[str] | None = None
    if sorting_strategy is not None:
        # Replicate python-ags4's behaviour: parse, sort the in-memory
        # table dict, then pass the ordered keys down to the Rust
        # writer which re-parses the file and writes in that order.
        # The double-parse is cheap (Rust); keeping sort logic in
        # Python lets us reuse the existing `sort_groups`.
        tables, _ = AGS4_to_dataframe(input_file)
        ordered_keys = list(sort_groups(tables, sorting_strategy).keys())

    try:
        _native.ags4_to_excel(
            str(input_file),
            str(output_file),
            ordered_keys,
        )
    except RuntimeError as exc:
        if "No valid AGS4 data" in str(exc):
            raise AGS4Error(str(exc)) from exc
        raise


def excel_to_AGS4(
    input_file: str,
    output_file: str,
    format_numeric_columns: bool = True,
    dictionary: str | None = None,
) -> None:
    """Convert an XLSX (AGS4-shaped) back to an AGS4 file.

    Rust-backed via `laterite_excel::excel_to_ags4` (uses `calamine`).
    Each worksheet with a ``HEADING`` column becomes one AGS4 group.
    Columns not matching Rule 19's ``[A-Z0-9]{4}_[A-Z0-9]{1,4}`` regex
    are dropped (with a warning); rows whose HEADING isn't UNIT /
    TYPE / DATA are dropped.

    Args:
        input_file: Path to the XLSX file.
        output_file: Path to write the AGS4 file to.
        format_numeric_columns: When True (default), DATA cells are
            re-formatted to their column's TYPE precision (``<N>DP``,
            ``<N>SCI``, ``<N>SF``) so floats from XLSX don't lose
            trailing zeros. Done in Rust via
            ``laterite_excel::apply_type_formatting``.
        dictionary: Optional bundled-edition version string or AGS4
            dict-file path. When provided, the XLSX is first converted
            to AGS4 via Rust, then the dictionary's UNIT/TYPE rows
            override the XLSX-provided ones and DATA cells are
            reformatted to the dict's TYPE precision per
            :func:`convert_to_text`. Mirrors python-ags4's
            ``excel_to_AGS4(dictionary=...)`` behaviour.

    Raises:
        AGS4Error: if no sheets had a HEADING column.
        BadDictError: if ``dictionary`` is supplied but is neither a
            bundled version string nor an AGS4 dict file we can read.
    """
    try:
        _native.excel_to_ags4(
            str(input_file),
            str(output_file),
            bool(format_numeric_columns),
        )
    except RuntimeError as exc:
        if "No valid AGS4 data" in str(exc):
            raise AGS4Error(str(exc)) from exc
        raise

    if dictionary is not None:
        # Post-process: parse the freshly-written AGS4, apply the dict
        # via convert_to_text (which now handles version strings AND
        # external AGS4 dict files), then write back. Keeps the Rust
        # path dict-agnostic — the heavy lifting is `convert_to_text`'s
        # `_inject_unit_type_from_dict` + `_format_sf` logic.
        _apply_dict_to_ags4_file(str(output_file), dictionary)


def _apply_dict_to_ags4_file(path: str, dictionary: Any) -> None:
    """In-place rewrite of an AGS4 file with UNIT/TYPE from a
    dictionary (version string or external dict file). Used by
    ``excel_to_AGS4(dictionary=...)`` after the Rust XLSX→AGS4 step.
    """
    tables, headings = AGS4_to_dataframe(path)
    formatted: dict[str, Any] = {}
    for code, df in tables.items():
        # convert_to_numeric → convert_to_text round-trip pulls
        # text-formatted DATA cells back through the dict's precisions.
        num = convert_to_numeric(df)
        formatted[code] = convert_to_text(num, dictionary=dictionary)
    # Re-emit AGS4 from the formatted tables, preserving group order.
    dataframe_to_AGS4(formatted, headings, path)


# --- F2c-follow-up Stage 2a: python-ags4 surface completion ----------
#
# Five public helpers + one exception class python-ags4 exposes that
# the original compat shim didn't mirror. Ported from
# python_ags4.AGS4 (1.2.0) with identical names, signatures, and
# output shapes so `from python_ags4 import AGS4` callers see no
# behaviour change beyond the Rust-backed validator + parser.


# `AGS4Error` is the python-ags4-style alias for laterite's native
# `Ags4Error`. Same class — `pytest.raises(AGS4Error)` matches code
# that raises `Ags4Error`, and vice versa. Renaming the native class
# would break laterite's API; aliasing here is the right boundary.
AGS4Error = Ags4Error


def get_TRAN_AGS(tables: dict[str, Any]) -> str | None:
    """Return ``TRAN_AGS`` (the AGS4 edition string) from a tables dict.

    Mirrors ``python_ags4.check.get_TRAN_AGS``. Reads the
    ``TRAN['TRAN_AGS']`` cell on the first DATA row. Returns ``None``
    if the TRAN group is absent — AGS4 Rule 14 would catch that
    separately during ``check_file``.
    """
    try:
        tran = tables["TRAN"]
    except KeyError:
        return None
    # Cross-backend: pandas exposes .loc + .HEADING; polars/pyarrow need
    # a different path. narwhals would help but adds an import; the
    # branch on duck-typed attrs is simpler and matches how the rest of
    # compat handles backend pluralism.
    if hasattr(tran, "loc") and hasattr(tran, "HEADING"):  # pandas
        rows = tran.loc[tran.HEADING.eq("DATA"), "TRAN_AGS"].values
        return str(rows[0]) if len(rows) else None
    if hasattr(tran, "filter") and hasattr(tran, "to_dicts"):  # polars
        rows = tran.filter(tran["HEADING"] == "DATA").get_column("TRAN_AGS").to_list()
        return str(rows[0]) if rows else None
    # pyarrow / unknown: fall back to dict-of-lists conversion.
    try:
        d = tran.to_pydict()  # pyarrow Table
        for h, v in zip(d.get("HEADING", []), d.get("TRAN_AGS", []), strict=False):
            if h == "DATA":
                return str(v)
    except AttributeError:
        pass
    return None


# --- JSON-helper functions (Stage 7a) -------------------------------
#
# python-ags4 ships four `utils.get_<X>_table_from_json_file` helpers
# that read the AGS-DFWG-Web ASG4 JSON schema files and return the
# corresponding AGS4 group as a Pandas DataFrame. These are
# re-implemented (not copied — python-ags4 is LGPL-3.0) to produce
# the same column shape + row ordering python-ags4's tests assert
# via `assert_frame_equal`. They pair with
# `convert_to_text(dictionary='4.1')` from Stage 6d for the standard
# pipe-through pattern in `utils.test_get_*_table_from_json_file`.


_DICT_STATUS_MAP = {
    "*": "KEY",
    "R": "REQUIRED",
    "*R": "KEY+REQUIRED",
    "R*": "KEY+REQUIRED",
    "": "OTHER",
    "Deprecated": "DEPRECATED",
}


def _valid_dict_version(version: str) -> None:
    if version not in ("4.0", "4.1", "4.2"):
        raise AGS4Error(
            "Invalid version number. Only '4.0' and '4.1' are valid entries."
        )


def get_DICT_table_from_json_file(filepath: Any) -> Any:
    """Read the AGS-DFWG JSON groups+headings schema and return a
    DICT-shaped Pandas DataFrame.

    Mirrors ``python_ags4.utils.get_DICT_table_from_json_file``.
    HEADING-row and GROUP-row entries are both emitted; deprecated
    groups are marked ``DICT_STAT='DEPRECATED'``; descriptions have
    embedded line-breaks stripped and double-quotes coerced to
    single-quotes (AGS4 Rule 5 round-tripping).
    """
    import pandas as pd

    heading_rows = pd.read_json(filepath).rename(
        columns={
            "group": "DICT_GRP",
            "heading": "DICT_HDNG",
            "suggested_type": "DICT_DTYP",
            "description": "DICT_DESC",
            "suggested_unit": "DICT_UNIT",
            "example": "DICT_EXMP",
        }
    )
    heading_rows = heading_rows.assign(
        HEADING="DATA",
        DICT_TYPE="HEADING",
        DICT_STAT=heading_rows["heading_status"].map(_DICT_STATUS_MAP),
        DICT_PGRP="",
        DICT_REM="",
        FILE_FSET="",
        in_group_order=heading_rows["in_group_order"].astype("int"),
        group_order=heading_rows["group_order"].astype("int"),
    )

    # One row per group (the first heading row's metadata + the
    # group's own description + parent).
    group_rows = (
        heading_rows.groupby("DICT_GRP")
        .first()
        .reset_index()
        .drop("DICT_DESC", axis=1)
        .rename(columns={"group_description": "DICT_DESC"})
    )
    group_rows = group_rows.assign(
        HEADING="DATA",
        DICT_TYPE="GROUP",
        DICT_HDNG="",
        DICT_STAT="",
        DICT_DTYP="",
        DICT_UNIT="",
        DICT_EXMP="",
        DICT_PGRP=group_rows["parent"],
        FILE_FSET="",
        in_group_order=0,
    )

    unit_and_type_rows = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE"],
            "DICT_TYPE": ["", "PA"],
            "DICT_GRP": ["", "X"],
            "DICT_HDNG": ["", "X"],
            "DICT_STAT": ["", "PA"],
            "DICT_DTYP": ["", "PT"],
            "DICT_DESC": ["", "X"],
            "DICT_UNIT": ["", "PU"],
            "DICT_EXMP": ["", "X"],
            "DICT_PGRP": ["", "X"],
            "DICT_REM": ["", "X"],
            "FILE_FSET": ["", "X"],
            "group_order": [-1, 0],
        }
    )

    DICT = pd.concat([unit_and_type_rows, heading_rows, group_rows])

    # Deprecated GROUPs carry the marker on the GROUP row only.
    deprecated_mask = DICT["group_status"].eq("Deprecated") & DICT["DICT_TYPE"].eq(
        "GROUP"
    )
    DICT.loc[deprecated_mask, "DICT_STAT"] = "DEPRECATED"

    DICT = (
        DICT.sort_values(by=["group_order", "in_group_order"])
        .loc[
            :,
            [
                "HEADING",
                "DICT_TYPE",
                "DICT_GRP",
                "DICT_HDNG",
                "DICT_STAT",
                "DICT_DTYP",
                "DICT_DESC",
                "DICT_UNIT",
                "DICT_EXMP",
                "DICT_PGRP",
                "DICT_REM",
                "FILE_FSET",
            ],
        ]
        .reset_index(drop=True)
    )

    # Description hygiene: strip embedded line-breaks (some entries
    # have CRLF/LF inside the description text) and coerce embedded
    # double-quotes to single — the in-file standard dict uses single
    # quotes for embedded quoting to keep Rule 5 quoting simple.
    DICT["DICT_DESC"] = (
        DICT["DICT_DESC"]
        .str.replace(r"(\r\n)", "", regex=True)
        .str.replace(r"(\n)", "", regex=True)
        .str.replace(r'(")', "'", regex=True)
    )

    return DICT


def get_ABBR_table_from_json_file(
    filepath: Any,
    filepath_ELRG: Any = None,
    version: str = "4.1",
) -> Any:
    """Read the AGS-DFWG JSON abbreviations list (optionally joined
    with a separate ELRG-codes file) and return an ABBR-shaped
    Pandas DataFrame for the given AGS4 ``version``.

    Mirrors ``python_ags4.utils.get_ABBR_table_from_json_file``.
    """
    import pandas as pd

    _valid_dict_version(version)

    data_rows = (
        pd.read_json(filepath)
        .rename(
            columns={
                "Group": "ABBR_HDNG",
                "Code": "ABBR_CODE",
                "Description": "ABBR_DESC",
            }
        )
        .assign(HEADING="DATA", ABBR_LIST=version, ABBR_REM="", FILE_FSET="")
        .query(
            "Version.str.contains(@version) & Status.str.contains('Approved', case=False)"
        )
        .sort_values(by=["ABBR_HDNG", "ABBR_CODE"])
    )

    unit_and_type_rows = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE"],
            "ABBR_HDNG": ["", "X"],
            "ABBR_CODE": ["", "X"],
            "ABBR_DESC": ["", "X"],
            "ABBR_LIST": ["", "X"],
            "ABBR_REM": ["", "X"],
            "FILE_FSET": ["", "X"],
        }
    )

    if filepath_ELRG is not None:
        elrg_codes = (
            pd.read_json(filepath_ELRG)
            .rename(
                columns={
                    "Code": "ABBR_CODE",
                    "code": "ABBR_CODE",
                    "description": "ABBR_DESC",
                }
            )
            .assign(
                HEADING="DATA",
                ABBR_HDNG="ELRG_CODE",
                ABBR_LIST=version,
                ABBR_REM="",
                FILE_FSET="",
            )
            .query(
                "version.str.contains(@version) & status.str.contains('Approved', case=False)"
            )
            .sort_values(by=["ABBR_HDNG", "ABBR_CODE"])
        )
    else:
        elrg_codes = pd.DataFrame()

    ABBR = pd.concat([unit_and_type_rows, data_rows, elrg_codes])
    return ABBR.loc[
        :,
        [
            "HEADING",
            "ABBR_HDNG",
            "ABBR_CODE",
            "ABBR_DESC",
            "ABBR_LIST",
            "ABBR_REM",
            "FILE_FSET",
        ],
    ].reset_index(drop=True)


def get_TYPE_table_from_json_file(filepath: Any, version: str = "4.1") -> Any:
    """Read the AGS-DFWG JSON types list and return a TYPE-shaped
    Pandas DataFrame for the given AGS4 ``version``.

    Mirrors ``python_ags4.utils.get_TYPE_table_from_json_file``.
    """
    import pandas as pd

    _valid_dict_version(version)

    data_rows = (
        pd.read_json(filepath)
        .rename(columns={"Type": "TYPE_TYPE", "Desc": "TYPE_DESC"})
        .assign(HEADING="DATA", FILE_FSET="")
        .query("Version.str.contains(@version)")
        .sort_values(by=["TYPE_TYPE", "TYPE_DESC"])
    )

    unit_and_type_rows = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE"],
            "TYPE_TYPE": ["", "X"],
            "TYPE_DESC": ["", "X"],
            "FILE_FSET": ["", "X"],
        }
    )

    TYPE = pd.concat([unit_and_type_rows, data_rows])
    return TYPE.loc[:, ["HEADING", "TYPE_TYPE", "TYPE_DESC", "FILE_FSET"]].reset_index(
        drop=True
    )


def get_UNIT_table_from_json_file(filepath: Any, version: str = "4.1") -> Any:
    """Read the AGS-DFWG JSON units list and return a UNIT-shaped
    Pandas DataFrame for the given AGS4 ``version``.

    Mirrors ``python_ags4.utils.get_UNIT_table_from_json_file``.
    Duplicates on ``UNIT_UNIT`` are dropped (keep-first), and rows
    are sorted case-insensitively (matches the in-file standard
    dictionary's stable ordering).
    """
    import pandas as pd

    _valid_dict_version(version)

    data_rows = (
        pd.read_json(filepath)
        .rename(columns={"Unit": "UNIT_UNIT", "Description": "UNIT_DESC"})
        .assign(HEADING="DATA", UNIT_REM="", FILE_FSET="")
        .query(
            "Version.str.contains(@version) & Status.str.contains('Approved', case=False)"
        )
        .drop_duplicates(subset="UNIT_UNIT", keep="first")
        # Case-insensitive sort — the in-file standard dict groups
        # `mm` and `MM` together, with uppercase reverse-precedence
        # of the default `str` compare.
        .sort_values(by=["UNIT_UNIT", "UNIT_DESC"], key=lambda x: x.str.lower())
    )

    unit_and_type_rows = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE"],
            "UNIT_UNIT": ["", "X"],
            "UNIT_DESC": ["", "X"],
            "UNIT_REM": ["", "X"],
            "FILE_FSET": ["", "X"],
        }
    )

    UNIT = pd.concat([unit_and_type_rows, data_rows])
    return UNIT.loc[
        :, ["HEADING", "UNIT_UNIT", "UNIT_DESC", "UNIT_REM", "FILE_FSET"]
    ].reset_index(drop=True)


def format_numeric_column(dataframe: Any, column_name: str, TYPE: str) -> Any:
    """Format a column to the specified AGS4 TYPE and return as string.

    Mirrors ``python_ags4.AGS4.format_numeric_column``. The dataframe
    is mutated on a copy; the original is left untouched.

    Args:
        dataframe: Pandas DataFrame holding AGS4 group data (with a
            ``HEADING`` column and the AGS4 UNIT/TYPE/DATA pseudo-rows).
        column_name: Column to format.
        TYPE: AGS4 TYPE specifier — ``"<N>DP"``, ``"<N>SCI"``, or
            ``"<N>SF"``. Anything else passes through unchanged.

    Returns:
        A new DataFrame with ``column_name`` reformatted on DATA rows
        only. Pandas only (compat's primary backend); polars users
        should switch backends or do the formatting themselves.
    """
    df = dataframe.copy()
    col = column_name

    # Same pandas-FutureWarning workaround python-ags4 uses: cast to
    # object before assigning string values into a numeric column.
    df[col] = df[col].astype("object")

    try:
        if "DP" in TYPE:
            i = int(TYPE.strip("DP"))
            mask = (df.HEADING == "DATA") & df[col].notna()
            df.loc[mask, col] = df.loc[mask, col].apply(lambda x: f"{x:.{i}f}")
        elif "SCI" in TYPE:
            i = int(TYPE.strip("SCI"))
            mask = (df.HEADING == "DATA") & df[col].notna()
            df.loc[mask, col] = df.loc[mask, col].apply(lambda x: f"{x:.{i}E}")
        elif "SF" in TYPE:
            mask = (df.HEADING == "DATA") & df[col].notna()
            df.loc[mask, [col]] = df.loc[mask, [col]].map(lambda x: _format_sf(x, TYPE))
    except ValueError, TypeError:
        # python-ags4 silently logs and returns the unmodified frame
        # when a column has non-numeric entries. Match that behaviour
        # rather than letting the user see a traceback.
        pass

    return df


def _format_sf(value: float, TYPE: str) -> str:
    """Significant-figure formatter — helper for `format_numeric_column`.

    Mirrors ``python_ags4.AGS4._format_SF``. Kept as a top-level
    underscore-prefixed helper so the public surface stays clean.
    """
    from math import floor, log10

    if value == 0:
        return f"{value}"

    i = int(TYPE.strip("SF")) - 1 - int(floor(log10(abs(value))))
    if i < 0:
        return f"{round(value, i):.0f}"
    return f"{value:.{i}f}"


def count_errors(ags_errors: dict) -> tuple[int, int, int]:
    """Count errors / warnings / FYI messages in a check result.

    Mirrors ``python_ags4.AGS4.count_errors``. Categorises by the
    well-known key prefixes the validator uses for its findings:
    ``"AGS Format Rule"`` and ``"Validator Process Error"`` count as
    errors, anything with ``"Warning"`` in the key as a warning,
    anything with ``"FYI"`` as an FYI.

    Args:
        ags_errors: Output dict from ``check_file``.

    Returns:
        ``(error_count, warnings_count, fyi_count)``.
    """
    error_count = 0
    warnings_count = 0
    fyi_count = 0
    for key, val in ags_errors.items():
        if ("AGS Format Rule" in key) or ("Validator Process Error" in key):
            error_count += len(val)
        elif "Warning" in key:
            warnings_count += len(val)
        elif "FYI" in key:
            fyi_count += len(val)
    return error_count, warnings_count, fyi_count


def write_error_report(
    ags_errors: dict,
    output_file: str | None,
    show_warnings: bool = False,
    show_fyi: bool = False,
) -> None:
    """Save a human-readable error report to disk.

    Mirrors ``python_ags4.AGS4.write_error_report`` byte-for-byte:
    same line endings (``\\r\\n``), same wrapping at width 100, same
    section order (Metadata → summary → General → Summary of data →
    AGS Format Rule keys → Validator Process Error → Warning → FYI).

    ``output_file=None`` is a no-op (matches python-ags4's ``except
    TypeError: pass`` swallow).
    """
    if output_file is None:
        return

    import textwrap

    error_count, warnings_count, fyi_count = count_errors(ags_errors)

    try:
        with open(output_file, "w", newline="", encoding="utf-8") as f:
            if "Metadata" in ags_errors:
                for entry in ags_errors["Metadata"]:
                    f.write(f"""{entry["line"] + ":":<12} {entry["desc"]}\r\n""")
                f.write("\r\n")

            if error_count == 0:
                f.write("All checks passed!\r\n\r\n")
            elif (
                "AGS Format Rule 3" in ags_errors
                and "AGS3" in ags_errors["AGS Format Rule 3"][0]["desc"]
            ):
                f.write("Checking aborted as AGS3 files are not supported!\r\n\r\n")
            else:
                f.write(f"{error_count} error(s) found in file!\r\n\r\n")

            if show_warnings:
                f.write(f"{warnings_count} warning(s) returned.\r\n\r\n")
            if show_fyi:
                f.write(f"{fyi_count} FYI message(s) returned.\r\n\r\n")

            if "General" in ags_errors:
                f.write("General:")
                for entry in ags_errors["General"]:
                    msg = "\r\n  ".join(textwrap.wrap(entry["desc"], width=100))
                    f.write(f"""\r\n  {msg}\r\n""")
                f.write("\r\n")

            if "Summary of data" in ags_errors:
                f.write("Summary of data:\r\n")
                for entry in ags_errors["Summary of data"]:
                    msg = "\r\n  ".join(textwrap.wrap(entry["desc"], width=100))
                    f.write(f"""  {msg}\r\n""")
                f.write("\r\n")

            for key in [k for k in ags_errors if "AGS Format Rule" in k]:
                f.write(f"{key}:\r\n")
                for entry in ags_errors[key]:
                    f.write(
                        f"""  Line {entry["line"]:<8} {entry["group"].strip('"'):<7} """
                        f"""{entry["desc"]}\r\n"""
                    )
                f.write("\r\n")

            for key in [k for k in ags_errors if "Validator Process Error" in k]:
                f.write(f"{key}:\r\n")
                for entry in ags_errors[key]:
                    f.write(
                        f"""  Line {entry["line"]:<8} {entry["group"].strip('"'):<7} """
                        f"""{entry["desc"]}\r\n"""
                    )
                f.write("\r\n")

            if show_warnings:
                for key in [k for k in ags_errors if "Warning" in k]:
                    f.write(f"{key}:\r\n")
                    for entry in ags_errors[key]:
                        f.write(
                            f"""  Line {entry["line"]:<8} {entry["group"].strip('"'):<7} """
                            f"""{entry["desc"]}\r\n"""
                        )
                    f.write("\r\n")

            if show_fyi:
                for key in [k for k in ags_errors if "FYI" in k]:
                    f.write(f"{key}:\r\n")
                    for entry in ags_errors[key]:
                        f.write(
                            f"""  Line {entry["line"]:<8} {entry["group"].strip('"'):<7} """
                            f"""{entry["desc"]}\r\n"""
                        )
                    f.write("\r\n")

    except FileNotFoundError:
        # Match python-ags4: log and continue rather than raise. Use
        # warnings so the message is visible without configuring
        # logging.
        import warnings

        warnings.warn(
            f"write_error_report: could not write to {output_file!r} "
            "(parent directory missing). Report not saved.",
            stacklevel=2,
        )


def sort_groups(tables: dict, sorting_strategy: str = "dictionary") -> dict:
    """Reorder ``tables`` keys by group order.

    Mirrors ``python_ags4.AGS4.sort_groups`` with one practical
    difference: laterite sources the group dictionary from its own
    Rust-authoritative ``laterite.registry.GROUPS`` rather than
    re-parsing an AGS4 dictionary file. Output order is the same for
    the 92 standard groups; project-specific groups present in the
    input's ``DICT`` table are appended in their dictionary order.

    Args:
        tables: ``dict[str, DataFrame]`` (output from
            ``AGS4_to_dataframe``).
        sorting_strategy: one of ``"dictionary"``, ``"alphabetical"``,
            ``"hierarchical"``.

    Returns:
        Same dict re-ordered. Groups not in the dictionary are
        appended at the end in alphabetical order (a warning is
        emitted, matching python-ags4).
    """
    from .registry import GROUPS as _LATERITE_GROUPS

    if sorting_strategy == "alphabetical":
        group_list: list[str] = sorted(tables.keys())

    elif sorting_strategy == "dictionary":
        # python-ags4's bundled standard dictionary lists groups in a
        # specific order: PROJ, then six ancillaries (ABBR, DICT,
        # FILE, TRAN, TYPE, UNIT) in declaration order, then every
        # other group alphabetically. Replicate that order so
        # callers porting from python-ags4 see the same output.
        _DICT_HEADER = (
            "PROJ",
            "ABBR",
            "DICT",
            "FILE",
            "TRAN",
            "TYPE",
            "UNIT",
        )
        header_set = set(_DICT_HEADER)
        alphabetical_rest = sorted(c for c in _LATERITE_GROUPS if c not in header_set)
        group_list = [*_DICT_HEADER, *alphabetical_rest]
        dict_tbl = tables.get("DICT")
        if dict_tbl is not None:
            for code in _extract_project_groups_from_dict(dict_tbl):
                if code not in group_list:
                    group_list.append(code)

    elif sorting_strategy == "hierarchical":
        # Mirror python-ags4's seed-and-descend: start with the
        # documented top-level group list (PROJ first, then the
        # ancillary singletons in a fixed order), then walk PROJ's
        # children recursively via the parent → children map. The
        # seed order is part of python-ags4's user-visible contract;
        # the alternative — auto-deriving from parent=None — gives a
        # different but valid order that breaks downstream tests.
        SEED_ROOTS = (
            "PROJ",
            "TRAN",
            "ABBR",
            "DICT",
            "FILE",
            "TYPE",
            "UNIT",
            "LBSG",
            "LBST",
            "PREM",
            "STND",
        )
        group_list = list(SEED_ROOTS)

        # Build parent → children map from the standard registry
        # (the canonical 92) plus any project-specific groups in
        # tables['DICT'].
        by_parent: dict[str | None, list[str]] = {}
        for code, descriptor in _LATERITE_GROUPS.items():
            by_parent.setdefault(descriptor.parent, []).append(code)
        dict_tbl = tables.get("DICT")
        if dict_tbl is not None:
            for code, parent in _extract_project_group_parents(dict_tbl):
                if code not in _LATERITE_GROUPS:
                    by_parent.setdefault(parent, []).append(code)

        # Recursive descent from PROJ — picks up every descendant in
        # parent-ordered (DICT-list) order.
        def _visit(code: str) -> None:
            for child in by_parent.get(code, []):
                if child not in group_list:
                    group_list.append(child)
                    _visit(child)

        _visit("PROJ")

    else:
        raise ValueError(
            f"sort_groups: unknown sorting_strategy {sorting_strategy!r}; "
            f"expected one of 'dictionary', 'alphabetical', 'hierarchical'"
        )

    # Assemble. Unknown groups (not in the registry, not in the input
    # DICT) get appended in alphabetical order with a warning.
    sorted_tables = {code: tables[code] for code in group_list if code in tables}
    leftover = sorted(set(tables.keys()) - set(sorted_tables.keys()))
    if leftover:
        import warnings

        for code in leftover:
            warnings.warn(
                f"sort_groups: appending {code!r} at the end — not found "
                f"in the dictionary or its parent group is not defined.",
                stacklevel=2,
            )
            sorted_tables[code] = tables[code]

    return sorted_tables


def _extract_project_groups_from_dict(dict_table: Any) -> list[str]:
    """Pull the ``DICT_GRP`` codes declared as ``DICT_TYPE == 'GROUP'``
    from an input file's DICT table. Pandas-friendly; works with
    polars too via ``.to_pandas()`` if needed (not used today since
    sort_groups is only called from compat which is pandas-default)."""
    if hasattr(dict_table, "loc"):  # pandas DataFrame
        rows = dict_table.loc[dict_table["DICT_TYPE"] == "GROUP"]
        if hasattr(rows, "DICT_GRP"):
            return list(rows["DICT_GRP"].tolist())
    # Polars / other backends: skip silently (returns no project groups,
    # which means sort_groups falls back to the standard order — fine).
    return []


def _extract_project_group_parents(dict_table: Any) -> list[tuple[str, str | None]]:
    """As above but also return the parent code per project group."""
    if hasattr(dict_table, "loc"):
        rows = dict_table.loc[dict_table["DICT_TYPE"] == "GROUP"]
        if hasattr(rows, "DICT_GRP") and hasattr(rows, "DICT_PGRP"):
            out = []
            for grp, pgrp in zip(
                rows["DICT_GRP"].tolist(),
                rows["DICT_PGRP"].tolist(),
                strict=False,
            ):
                # Empty parent strings → root (None) per AGS4 convention.
                out.append((grp, pgrp if pgrp else None))
            return out
    return []


__all__ = [
    "__version__",
    "PYTHON_AGS4_COMPAT",
    "AGS4Error",
    "AGS4_to_dict",
    "AGS4_to_dataframe",
    "AGS4_to_dataframe_AGS3",
    "AGS4_to_excel",
    "dataframe_to_AGS4",
    "convert_to_numeric",
    "convert_to_text",
    "check_file",
    "count_errors",
    "excel_to_AGS4",
    "format_numeric_column",
    "get_ABBR_table_from_json_file",
    "get_DICT_table_from_json_file",
    "get_TRAN_AGS",
    "get_TYPE_table_from_json_file",
    "get_UNIT_table_from_json_file",
    "set_backend",
    "get_backend",
    "sort_groups",
    "write_error_report",
]
