"""laterite — a Rust-backed AGS4 reader / writer / validator.

The engine is the clean-room ``laterite_ags4_validator`` Rust crate exposed via PyO3
(``laterite._laterite_native``), with a Python-owned in-memory **DuckDB**
engine on top: a parsed AGS4 file becomes born-typed DuckDB tables, and
``ags[code]`` / ``ags.sql(...)`` read them back as **polars** (default) or
**pandas** frames — both pyarrow-free.

For a literal ``python_ags4`` swap-in use ``from laterite import compat as
AGS4``. For the CLI use ``lat-check`` (byte-faithful to the Rust binary).
"""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
from typing import Any, Self

import polars as pl

from . import _laterite_native as _native
from ._errors import (
    Ags4Error,
    BadDictError,
    NotAgs4Error,
    UnsupportedEditionError,
    raise_for,
)
from ._frames import ArrowStream, frame_from_arrow
from .registry import GROUPS as _GROUPS

# Re-export the typed-graph classes for ergonomic
# `from laterite import PROJ, LOCA, SAMP, ...`. The class objects live in the
# compiled Rust extension `_laterite_native`; this loop just aliases them at the
# package root. Type checkers follow the `.pyi` next to the .so.
_TYPED_CLASS_NAMES: tuple[str, ...] = tuple(sorted(_GROUPS))
for _code in _TYPED_CLASS_NAMES:
    globals()[_code] = getattr(_native, _code)
del _code

__all__ = [
    "validate",
    "read",
    "write",
    "emit_ags4",
    "EmitResult",
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

# Frame backends `read(..., backend=)` / `ags[code]` can return. Both are
# pyarrow-free — polars via the Arrow capsule, pandas via DuckDB's NumPy
# `.df()`. Arrow tables are deliberately NOT offered here; reach for
# `ags.connection` (with your own pyarrow) if you want them.
_BACKENDS = ("polars", "pandas")


def _split_source(source: Any, text: str | None) -> tuple[str | None, str | None]:
    if text is not None:
        return None, text
    if source is None:
        raise TypeError("provide a file path or text=")
    return str(source), None


class Report:
    """Outcome of :func:`validate`. ``findings`` is a polars frame; ``to_json``
    / ``to_ndjson`` are byte-faithful to the Rust ``lat-check`` binary."""

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
    def findings(self) -> pl.DataFrame:
        """Polars frame ``rule, line, group, desc`` (one row per finding;
        ``line`` is a nullable Int64)."""
        items = self._r["findings"]
        return pl.DataFrame(
            {
                "rule": pl.Series([f["rule"] for f in items], dtype=pl.String),
                "line": pl.Series([f["line"] for f in items], dtype=pl.Int64),
                "group": pl.Series([f["group"] for f in items], dtype=pl.String),
                "desc": pl.Series([f["desc"] for f in items], dtype=pl.String),
            }
        )

    def by_rule(self) -> dict[str, list[dict]]:
        """``{"AGS Format Rule N": [{line, group, desc}, ...]}`` — the spec-rule
        grouping (sorted, like the Rust BTreeMap)."""
        out: dict[str, list[dict]] = {}
        for f in self._r["findings"]:
            out.setdefault(f["rule"], []).append(
                {"line": f["line"], "group": f["group"], "desc": f["desc"]}
            )
        return out

    def to_json(self) -> str:
        """``{file, findings:{"AGS Format Rule N":[{line,group,desc}]}}`` —
        byte-identical to ``lat-check --json``."""
        return self._r["json"]

    def to_ndjson(self) -> str:
        """One flat ``{rule,line,group,desc}`` per line — byte-identical to
        ``lat-check --ndjson``."""
        return self._r["ndjson"]

    def __repr__(self) -> str:
        v = "valid" if self.is_valid else f"{self.count} finding(s)"
        return f"<Report {self.file!r} {v} dict={self.dict_version}>"


class Ags4File:
    """A parsed AGS4 file over a Python-owned in-memory **DuckDB** engine.

    Each group becomes a born-typed DuckDB table (a 2DP heading is Float64, an
    ID str, a non-conforming numeric cell is null). Access:

    - ``ags["LOCA"]`` → one group materialised to the handle's backend
      (**polars** by default, **pandas** if ``read(..., backend="pandas")``);
      both pyarrow-free. Groups load into the engine on first touch.
    - ``ags.sql("SELECT … WHERE …")`` → a **DuckDB relation** — cross-group
      joins + filter pushdown; finish with ``.df()`` / ``pl.from_arrow(rel)``.
    - ``ags.connection`` → the raw duckdb connection (every engine feature).

    UNIT/TYPE/HEADING are side metadata, not pseudo-rows (use ``compat`` for the
    python-ags4 HEADING-column shape). ``write`` round-trips byte-faithfully
    from the retained Rust parse, independent of which groups were touched."""

    __slots__ = ("_backend", "_con", "_p", "_registered")

    def __init__(self, parsed: dict, backend: str = "polars") -> None:
        # Guard the common `read(path)` vs `Ags4File(path)` mix-up: the ctor takes
        # the parsed mapping `parse_arrow` returns, not a path. Without this a bad
        # arg fails several calls later with a cryptic subscript error (#112).
        if (
            not isinstance(parsed, Mapping)
            or "groups" not in parsed
            or "group_order" not in parsed
        ):
            raise TypeError(
                "Ags4File() expects parsed primitives (a mapping with 'groups' and "
                f"'group_order'), not {type(parsed).__name__!r}. To load a file, use "
                "laterite.read(path)."
            )
        if backend not in _BACKENDS:
            raise ValueError(
                f"backend must be one of {_BACKENDS} (got {backend!r}); for Arrow "
                "tables use ags.connection with your own pyarrow."
            )
        self._p = parsed
        self._backend = backend
        self._con = None  # lazy DuckDB engine (first group access / sql / connection)
        self._registered: set[str] = set()  # groups loaded into _con

    # --- metadata (no engine spin-up) ----------------------------------------

    @property
    def groups(self) -> list[str]:
        return list(self._p["group_order"])

    @property
    def backend(self) -> str:
        return self._backend

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
        return list(self._g(code)["line_numbers"])

    def __contains__(self, code: str) -> bool:
        return code in self._p["groups"]

    # --- the Python-owned DuckDB engine --------------------------------------
    #
    # Spun up lazily on the first group access / sql() / .connection. Each group
    # loads ON DEMAND into a NATIVE DuckDB table (CTAS from the Rust-built Arrow)
    # — native storage, not a view over external Arrow, so joins/filters run in
    # DuckDB and don't push predicates into a pyarrow Arrow-scan (whose is_in
    # kernel trips on DuckDB's string_view strings).

    def _engine(self):
        if self._con is None:
            import duckdb

            self._con = duckdb.connect(":memory:")
            self._registered = set()
        return self._con

    def _register(self, code: str) -> None:
        con = self._engine()
        if code in self._registered:
            return
        table = self._p["_handle"].table_for(code)
        if table is None:
            raise KeyError(f"group {code!r} not in file")
        tmp = f"__arrow_{code}"
        con.register(tmp, table)  # the arro3 table's Arrow capsule (pyarrow-free)
        try:
            con.execute(f'CREATE TABLE "{code}" AS SELECT * FROM "{tmp}"')
        finally:
            con.unregister(tmp)
        self._registered.add(code)

    def _register_all(self) -> None:
        for code in self._p["group_order"]:
            self._register(code)

    def _materialize(self, rel):
        # polars via the Arrow capsule (pl.from_arrow) and pandas via DuckDB's
        # NumPy .df() are BOTH pyarrow-free; rel.pl() and polars->pandas would
        # both pull pyarrow, so we never take those.
        if self._backend == "pandas":
            return rel.df()
        return frame_from_arrow(rel)

    def __getitem__(self, code: str):
        """One group, materialised to the handle's backend (born-typed)."""
        self._register(code)
        return self._materialize(self._engine().sql(f'SELECT * FROM "{code}"'))

    table = __getitem__

    @property
    def connection(self):
        """The raw ``duckdb`` connection — every engine feature (parquet export,
        the relational API, Arrow via ``.arrow()``, …). Seeded with all of this
        file's groups under their clean names on first access."""
        self._register_all()
        return self._engine()

    def sql(self, query: str):
        """Run SQL over the file's groups by their clean names — e.g.
        ``ags.sql("SELECT * FROM LOCA JOIN SAMP USING (LOCA_ID) WHERE ...")`` —
        returning a **DuckDB relation**. The WHERE/SELECT push into the engine
        (filter a big file down before materialising); finish with ``.df()`` /
        ``pl.from_arrow(rel)`` or chain more SQL. A query may reference any
        group, so this registers them all."""
        self._register_all()
        return self._engine().sql(query)

    def at(self, group: str, values) -> _AgsSubset:
        """Filter to a parent entity's records — ``ags.at("LOCA", ["BH01", "BH02"])``
        returns a view whose ``sub[code]`` yields only the rows of each group whose
        ``{group}_ID`` (e.g. ``LOCA_ID``) is in ``values``, materialising only the
        matching rows (explore a huge file without a huge frame). Chain to narrow
        further (``.at("SAMP", […])``); ``sub.groups`` is the related groups and
        ``sub.frames()`` pulls them all at once. Groups carrying none of the keys
        pass through unfiltered. For any other predicate, use ``sql("... WHERE ...")``."""
        return _AgsSubset(self, [(f"{group}_ID", list(values))])

    def register(self, name: str, frame) -> None:
        """Register YOUR frame (polars / pandas / pyarrow / arro3) into the
        engine as ``name``, so ``sql()`` can join it against the AGS groups."""
        native = frame.to_native() if hasattr(frame, "to_native") else frame
        # Force DuckDB's pyarrow-free Arrow-capsule path for capsule-exposing
        # frames (polars / pyarrow / arro3); a bare polars frame would route
        # through polars.to_arrow() and pull in pyarrow.
        if hasattr(native, "__arrow_c_stream__"):
            native = ArrowStream(native)
        self._engine().register(name, native)

    def close(self) -> None:
        """Close the in-memory DuckDB engine if one was created. Idempotent.
        NOTE: relations from ``sql()`` become invalid once closed — materialise
        (``.df()`` / ``pl.from_arrow``) before closing."""
        if self._con is not None:
            self._con.close()
            self._con = None
            self._registered.clear()

    def __enter__(self) -> Self:
        return self

    def __exit__(self, *exc: object) -> bool:
        # __exit__ == close() — the duckdb convention (NOT sqlite3's commit).
        self.close()
        return False

    # No __del__ close on purpose: a relation from sql() holds the connection
    # alive by refcount, so a one-liner `read(p).sql(q).df()` works even with the
    # handle unbound. The connection is released when the handle AND any
    # outstanding relations are gone (or explicitly via close() / `with`).

    def to_numeric(self, code: str):
        """Group frame with DP/SF/SCI/MC columns coerced to numeric
        (non-numeric → null, python-ags4 ``errors='coerce'`` parity). Mostly
        redundant now — those columns are already born numeric — but kept for
        the compat shape; honours the handle's backend."""
        g = self._g(code)
        numeric = [
            h
            for h, t in zip(g["headings"], g["types"], strict=False)
            if any(tok in t for tok in _NUMERIC_TOKENS)
        ]
        frame = self[code]
        if not numeric:
            return frame
        if self._backend == "pandas":
            import pandas as pd

            return frame.assign(
                **{c: pd.to_numeric(frame[c], errors="coerce") for c in numeric}
            )
        return frame.with_columns(
            pl.col(c).cast(pl.Float64, strict=False) for c in numeric
        )

    def to_ags4_text(self) -> str:
        """Reconstruct spec-correct AGS4 text (CRLF, every field quoted,
        ``"``→``""``) for every group, in file order — byte-faithful to the
        source DATA values. Emitted Rust-side from the retained parse (the
        ``_handle``), so no per-cell rows cross the boundary on read."""
        return self._p["_handle"].emit()

    def write(self, path: str | Path) -> Path:
        path = Path(path)
        path.write_bytes(self.to_ags4_text().encode("utf-8"))
        return path

    def __repr__(self) -> str:
        return (
            f"<Ags4File groups={len(self.groups)} backend={self._backend!r} "
            f"tran_ags={self.tran_ags!r}>"
        )


class _AgsSubset:
    """A key-filtered view over an :class:`Ags4File`'s engine, returned by
    :meth:`Ags4File.at`. Filters **accumulate** by chaining
    (``at("LOCA", […]).at("SAMP", […])`` keeps both): ``sub[code]`` applies every
    filter whose key column is present in ``code`` (others are ignored), so groups
    carrying none of the keys pass through unfiltered. ``sub.groups`` lists the
    related groups (touched by a filter) and ``sub.frames()`` pulls them all at
    once. Inherits the parent handle's backend."""

    __slots__ = ("_filters", "_parent")

    def __init__(self, parent: Ags4File, filters: list[tuple[str, list]]) -> None:
        self._parent = parent
        self._filters = filters

    def at(self, group: str, values) -> _AgsSubset:
        """Narrow further by another entity's id (e.g. add a SAMP filter)."""
        return _AgsSubset(self._parent, [*self._filters, (f"{group}_ID", list(values))])

    @property
    def groups(self) -> list[str]:
        """The related groups — those carrying at least one filter's key column."""
        p = self._parent
        keys = {k for k, _ in self._filters}
        return [g for g in p.groups if keys.intersection(p.headings(g))]

    def __contains__(self, code: str) -> bool:
        return code in self._parent

    def __getitem__(self, code: str):
        """``code`` filtered by every applicable key (groups carrying none of the
        keys pass through), materialised to the handle's backend."""
        p = self._parent
        p._register(code)
        cols = set(p.headings(code))
        clauses: list[str] = []
        params: list = []
        for key, values in self._filters:
            if key not in cols:
                continue
            if not values:
                clauses.append("FALSE")  # an empty selection matches nothing
            else:
                clauses.append(f'"{key}" IN ({", ".join("?" * len(values))})')
                params.extend(values)
        where = " AND ".join(clauses) if clauses else "TRUE"
        rel = p._engine().sql(
            f'SELECT * FROM "{code}" WHERE {where}', params=params or None
        )
        return p._materialize(rel)

    table = __getitem__

    def frames(self) -> dict:
        """``{group: frame}`` for every related group, each filtered — pull a
        location's whole related record set in one call."""
        return {g: self[g] for g in self.groups}

    def __repr__(self) -> str:
        flt = ", ".join(f"{k} in {v!r}" for k, v in self._filters)
        return f"<Ags4File.at {flt} — {len(self.groups)} related group(s)>"


def validate(
    source: Any = None,
    *,
    text: str | None = None,
    dict_version: str | None = None,
    warnings: bool = False,
    fyi: bool = False,
    check_files: bool = False,
) -> Report:
    """Validate an AGS4 file (path) or in-memory ``text=`` against the AGS4.1
    rules. Raises for un-validatable input (missing / not AGS4 / unsupported
    edition); rule *violations* come back in the :class:`Report`."""
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


def read(
    source: Any = None, *, text: str | None = None, backend: str = "polars"
) -> Ags4File:
    """Parse an AGS4 file (path) or in-memory ``text=`` into an :class:`Ags4File`
    over an in-memory DuckDB engine. ``backend`` is the default frame type for
    ``ags[code]`` — ``"polars"`` (default) or ``"pandas"`` (both pyarrow-free)."""
    path, txt = _split_source(source, text)
    p = _native.parse_arrow(path=path, text=txt)
    return Ags4File(raise_for(p), backend=backend)


def write(source: Ags4File, path: str | Path) -> Path:
    """Write an :class:`Ags4File` back to spec-correct AGS4 — byte-faithful to
    the source DATA values (re-emitted Rust-side from the retained parse).
    Arbitrary dataframes go through :func:`laterite.compat.dataframe_to_AGS4`
    (the python-ags4 ``tables``/``headings`` contract)."""
    if not isinstance(source, Ags4File):
        raise TypeError(
            "write() takes an Ags4File; use laterite.compat.dataframe_to_AGS4 "
            "for the python-ags4 tables/headings contract"
        )
    return source.write(path)


def dict_for(source: Any = None, *, text: str | None = None) -> tuple[str, str]:
    """``(edition, resolution)`` the engine would validate this file against —
    e.g. ``("4.1.1", "fallback")`` — without running rules."""
    path, txt = _split_source(source, text)
    p = raise_for(_native.parse_primitives(path=path, text=txt))
    return _native.resolve_dict(p.get("tran_ags"), None)


class EmitResult:
    """The product of :func:`emit_ags4`: the AGS4 ``bytes``, the validator
    ``findings`` on those bytes (post-fix in AutoFix mode), and the count of
    safe fixes applied. ``.text`` decodes the bytes; ``.write(path)`` saves them."""

    __slots__ = ("bytes", "findings", "fixes_applied")

    def __init__(self, data: bytes, findings: list[dict], fixes_applied: int) -> None:
        self.bytes = data
        self.findings = findings
        self.fixes_applied = fixes_applied

    @property
    def text(self) -> str:
        return self.bytes.decode("utf-8")

    def write(self, path: str | Path) -> Path:
        path = Path(path)
        path.write_bytes(self.bytes)
        return path

    def __repr__(self) -> str:
        return (
            f"<EmitResult {len(self.bytes)} bytes, "
            f"{len(self.findings)} finding(s), fixes_applied={self.fixes_applied}>"
        )


def emit_ags4(
    groups: Mapping[str, Any] | list[tuple[str, Any]],
    *,
    edition: str = "4.1.1",
    mode: str = "autofix",
) -> EmitResult:
    """Build valid AGS4 from your own per-group data — the data→AGS4 door
    (the inverse of :func:`read`).

    ``groups`` maps each AGS group code to a frame (pandas **or** polars)
    whose **column names are the AGS headings** (e.g. ``LOCA_ID``, ``LOCA_GL``);
    UNIT/TYPE are filled from the chosen edition's standard dictionary. Order
    is preserved (pass an ordered mapping or a list of ``(code, frame)`` pairs;
    put ``PROJ`` first).

    The frame crosses into Rust zero-copy via the Arrow C-stream — **pyarrow-free
    for polars** (so this stays a base feature, no ``[compat]``) and for pandas
    ≥ 2.2; an older pandas routes through DuckDB (pandas only ships via
    ``[compat]``, which carries the deps). Each cell is formatted to its
    canonical AGS4 string. ``mode``:

    * ``"autofix"`` (default) — build, then apply the *safe* mechanical fixes
      (pad decimals, normalise, …); ``EmitResult.findings`` holds whatever
      couldn't be safely fixed (e.g. a missing required heading).
    * ``"report"`` — build unchanged, return findings for you to act on.
    * ``"strict"`` — raise if the output violates any error-severity rule.

    ``edition`` is one of ``4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2`` (default
    ``4.1.1``)."""
    import json

    items = list(groups.items()) if isinstance(groups, Mapping) else list(groups)
    # Hand each frame straight to Rust via its Arrow C-stream PyCapsule
    # (`__arrow_c_stream__`) — pyo3-arrow reads it with NO pyarrow, so the polars
    # path stays a base feature. polars always exposes the capsule; pandas does
    # from 2.2. Only an older pandas (no capsule) falls back through DuckDB — and
    # pandas ships solely via [compat], which carries pyarrow + duckdb, so that
    # branch never burdens a base polars user. (#111 base-surface audit: the old
    # code registered EVERY frame into DuckDB, whose polars ingest goes via
    # polars `.to_arrow()` → pyarrow, leaking [compat] into a base call.)
    tables = []
    con = None
    for i, (code, frame) in enumerate(items):
        if hasattr(frame, "__arrow_c_stream__"):
            tables.append((code, frame))
            continue
        if con is None:
            import duckdb

            con = duckdb.connect()
        name = f"_emit_{i}"
        con.register(name, frame)
        tables.append((code, con.sql(f'SELECT * FROM "{name}"')))
    data, findings_json, fixes = _native.emit_ags4_from_arrow(tables, edition, mode)
    by_rule: dict[str, list[dict]] = json.loads(findings_json)
    findings = [{"rule": rule, **f} for rule, items_ in by_rule.items() for f in items_]
    return EmitResult(data, findings, fixes)
