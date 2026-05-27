"""laterite — a Rust-backed AGS4 reader / writer / validator.

The engine is the clean-room ``ags4_validator`` Rust crate exposed via
PyO3 (``laterite._laterite_native``). This module is the *nice* native
API: tabular results are `narwhals <https://narwhals-api.readthedocs.io>`_
frames over eager Polars, so callers target polars / pandas / pyarrow
without laterite choosing for them.

For a literal ``python_ags4`` swap-in use ``from laterite import
compat as AGS4``. For the CLI use ``ags4-check`` (byte-faithful to the
Rust binary).
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import narwhals.stable.v1 as nw

from . import _laterite_native as _native
from ._errors import (
    Ags4Error,
    BadDictError,
    NotAgs4Error,
    UnsupportedEditionError,
    raise_for,
)
from ._frames import polars_string_frame
from .registry import GROUPS as _GROUPS

# Re-export the typed-graph classes for ergonomic
# `from laterite import PROJ, LOCA, SAMP, ...`. The class objects
# live in the compiled Rust extension `_laterite_native`; this loop
# just aliases them at the package root. Type checkers follow the
# `.pyi` next to the .so for autocomplete / type validation.
#
# F2c-1: replaces the previous-only path of
# `from ags5_models import PROJ, ...` which retires with ags5-models.
_TYPED_CLASS_NAMES: tuple[str, ...] = tuple(sorted(_GROUPS))
for _code in _TYPED_CLASS_NAMES:
    globals()[_code] = getattr(_native, _code)
del _code

__all__ = [
    "validate",
    "read",
    "write",
    "dict_for",
    "Report",
    "Ags4File",
    "Ags4Error",
    "NotAgs4Error",
    "UnsupportedEditionError",
    "BadDictError",
    *_TYPED_CLASS_NAMES,
]

# AGS TYPE codes whose columns are numeric (python-ags4 convert_to_numeric
# parity: it coerces columns whose TYPE contains any of these tokens).
_NUMERIC_TOKENS = ("DP", "MC", "SF", "SCI")


def _split_source(source: Any, text: str | None) -> tuple[str | None, str | None]:
    if text is not None:
        return None, text
    if source is None:
        raise TypeError("provide a file path or text=")
    return str(source), None


class Report:
    """Outcome of :func:`validate`. The findings frame is narwhals;
    ``to_json`` / ``to_ndjson`` are byte-faithful to the Rust
    ``ags4-check`` binary."""

    __slots__ = ("_r",)

    def __init__(self, r: dict) -> None:
        self._r = r

    @property
    def file(self) -> str:
        return self._r["file"]

    @property
    def dict_version(self) -> str:
        return self._r["dict_version"]

    @property
    def resolution(self) -> str:
        return self._r["resolution"]

    @property
    def count(self) -> int:
        return self._r["count"]

    @property
    def is_valid(self) -> bool:
        return self._r["count"] == 0

    @property
    def exit_code(self) -> int:
        return self._r["exit_code"]

    @property
    def findings(self):
        """Narwhals frame: ``rule, line, group, desc`` (one row per
        finding; ``line`` is a nullable Int64)."""
        import polars as pl

        items = self._r["findings"]
        df = pl.DataFrame(
            {
                "rule": pl.Series([f["rule"] for f in items], dtype=pl.String),
                "line": pl.Series(
                    [f["line"] for f in items], dtype=pl.Int64
                ),
                "group": pl.Series([f["group"] for f in items], dtype=pl.String),
                "desc": pl.Series([f["desc"] for f in items], dtype=pl.String),
            }
        )
        return nw.from_native(df, eager_only=True)

    def by_rule(self) -> dict[str, list[dict]]:
        """``{"AGS Format Rule N": [{line, group, desc}, ...]}`` —
        the spec-rule grouping (sorted, like the Rust BTreeMap)."""
        out: dict[str, list[dict]] = {}
        for f in self._r["findings"]:
            out.setdefault(f["rule"], []).append(
                {"line": f["line"], "group": f["group"], "desc": f["desc"]}
            )
        return out

    def to_json(self) -> str:
        """``{file, findings:{"AGS Format Rule N":[{line,group,desc}]}}``
        — byte-identical to ``ags4-check --json``."""
        return self._r["json"]

    def to_ndjson(self) -> str:
        """One flat ``{rule,line,group,desc}`` per line — byte-identical
        to ``ags4-check --ndjson``."""
        return self._r["ndjson"]

    def __repr__(self) -> str:
        v = "valid" if self.is_valid else f"{self.count} finding(s)"
        return f"<Report {self.file!r} {v} dict={self.dict_version}>"


class Ags4File:
    """A parsed AGS4 file. Tables are narwhals frames of DATA rows
    (string dtype — AGS4 is a text format); UNIT/TYPE/HEADING live as
    side metadata, not pseudo-rows (use ``compat`` for the python-ags4
    HEADING-column shape)."""

    __slots__ = ("_p",)

    def __init__(self, parsed: dict) -> None:
        self._p = parsed

    @property
    def groups(self) -> list[str]:
        return list(self._p["group_order"])

    @property
    def tran_ags(self) -> str | None:
        return self._p.get("tran_ags")

    def _g(self, code: str) -> dict:
        try:
            return self._p["groups"][code]
        except KeyError:
            raise KeyError(f"group {code!r} not in file") from None

    def headings(self, code: str) -> list[str]:
        return list(self._g(code)["headings"])

    def units(self, code: str) -> list[str]:
        return list(self._g(code)["units"])

    def types(self, code: str) -> list[str]:
        return list(self._g(code)["types"])

    def line_numbers(self, code: str) -> list[int]:
        return [r["line"] for r in self._g(code)["rows"]]

    def __contains__(self, code: str) -> bool:
        return code in self._p["groups"]

    def __getitem__(self, code: str):
        g = self._g(code)
        df = polars_string_frame(
            list(g["headings"]), [list(r["values"]) for r in g["rows"]]
        )
        return nw.from_native(df, eager_only=True)

    table = __getitem__

    def to_numeric(self, code: str):
        """Narwhals frame with DP/SF/SCI/MC columns cast to Float64
        (non-numeric cells → null — python-ags4 ``errors='coerce'``
        parity)."""
        g = self._g(code)
        headings = list(g["headings"])
        types = list(g["types"])
        frame = self[code]
        numeric = [
            h
            for h, t in zip(headings, types, strict=False)
            if any(tok in t for tok in _NUMERIC_TOKENS)
        ]
        if not numeric:
            return frame
        import polars as pl

        df = frame.to_native().with_columns(
            pl.col(c).cast(pl.Float64, strict=False) for c in numeric
        )
        return nw.from_native(df, eager_only=True)

    def _matrix(self, code: str) -> list[list[str]]:
        g = self._g(code)
        headings = list(g["headings"])
        n = len(headings)
        matrix = [["HEADING", *headings]]
        matrix.append(["UNIT", *(list(g["units"]) + [""] * n)[:n]])
        matrix.append(["TYPE", *(list(g["types"]) + [""] * n)[:n]])
        for r in g["rows"]:
            vals = list(r["values"])
            vals = (vals + [""] * n)[:n]
            matrix.append(["DATA", *vals])
        return matrix

    def to_ags4_text(self) -> str:
        """Reconstruct spec-correct AGS4 text (CRLF, every field quoted,
        ``"``→``""``) for every group, in file order."""
        groups = [(c, self._matrix(c)) for c in self._p["group_order"]]
        return _native.emit_ags4(groups)

    def write(self, path: str | Path) -> Path:
        path = Path(path)
        path.write_bytes(self.to_ags4_text().encode("utf-8"))
        return path

    def __repr__(self) -> str:
        return f"<Ags4File groups={len(self.groups)} tran_ags={self.tran_ags!r}>"


def validate(
    source: Any = None,
    *,
    text: str | None = None,
    dict_version: str | None = None,
    warnings: bool = False,
    fyi: bool = False,
    check_files: bool = False,
) -> Report:
    """Validate an AGS4 file (path) or in-memory ``text=`` against the
    AGS4.1 rules. Raises for un-validatable input (missing / not AGS4 /
    unsupported edition); rule *violations* come back in the
    :class:`Report`."""
    path, txt = _split_source(source, text)
    r = _native.run_check(
        path=path,
        text=txt,
        dict_version=dict_version,
        include_warnings=warnings,
        include_fyi=fyi,
        check_files=check_files,
    )
    return Report(raise_for(r))


def read(source: Any = None, *, text: str | None = None) -> Ags4File:
    """Parse an AGS4 file (path) or in-memory ``text=`` into an
    :class:`Ags4File`."""
    path, txt = _split_source(source, text)
    p = _native.parse_primitives(path=path, text=txt)
    return Ags4File(raise_for(p))


def write(source: Ags4File, path: str | Path) -> Path:
    """Write an :class:`Ags4File` back to spec-correct AGS4. (Arbitrary
    dataframes go through :func:`laterite.compat.dataframe_to_AGS4`,
    the python-ags4 ``tables``/``headings`` contract.)"""
    if not isinstance(source, Ags4File):
        raise TypeError(
            "write() takes an Ags4File; use laterite.compat.dataframe_to_AGS4 "
            "for the python-ags4 tables/headings contract"
        )
    return source.write(path)


def dict_for(source: Any = None, *, text: str | None = None) -> tuple[str, str]:
    """``(edition, resolution)`` the engine would validate this file
    against — e.g. ``("4.1.1", "fallback")`` — without running rules."""
    path, txt = _split_source(source, text)
    p = raise_for(_native.parse_primitives(path=path, text=txt))
    return _native.resolve_dict(p.get("tran_ags"), None)
