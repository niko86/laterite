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

import os
from collections.abc import Mapping
from pathlib import Path
from typing import TYPE_CHECKING, Any, Self

import polars as pl

from . import _laterite_native as _native
from ._errors import (
    Ags4Error,
    BadDictError,
    NotAgs4Error,
    StaleCertError,
    UnsupportedEditionError,
    raise_for,
)
from ._frames import ArrowStream, frame_from_arrow
from .registry import GROUPS as _GROUPS
from .registry import child_groups as _child_groups

# Re-export the typed-graph classes for ergonomic
# `from laterite import PROJ, LOCA, SAMP, ...`. The class objects live in the
# compiled Rust extension `_laterite_native`; this loop aliases them onto the
# package root at runtime.
_TYPED_CLASS_NAMES: tuple[str, ...] = tuple(sorted(_GROUPS))
for _code in _TYPED_CLASS_NAMES:
    globals()[_code] = getattr(_native, _code)
del _code

# The runtime loop above is invisible to static analysers, so without this they
# flag `from laterite import PROJ` as an unknown symbol — no IDE autocomplete and
# spurious type errors. Re-importing the classes here (type-check time only,
# never executed) makes them statically visible; the star is restricted to the
# typed-graph classes by `_laterite_native.pyi`'s generated `__all__`.
if TYPE_CHECKING:
    from ._laterite_native import *  # noqa: F403

__all__ = [
    "validate",
    "read",
    "source",
    "to_excel",
    "from_excel",
    "fix",
    "FixResult",
    "build_ags4",
    "BuildResult",
    "dict_for",
    "list_rules",
    "diff",
    "Report",
    "Ags4File",
    "AgsQuery",
    "Ags4Error",
    "NotAgs4Error",
    "UnsupportedEditionError",
    "BadDictError",
    "StaleCertError",
    *_TYPED_CLASS_NAMES,
]

# Frame backends `read(..., backend=)` / `ags[code]` can return. Both are
# pyarrow-free — polars via the Arrow capsule, pandas via DuckDB's NumPy
# `.df()`. Arrow tables are deliberately NOT offered here; reach for
# `ags.connection` (with your own pyarrow) if you want them.
_BACKENDS = ("polars", "pandas")

# XN read modes for `read(..., xn=)`. AGS XN ("numeric, may carry a non-numeric
# qualifier like NP / <5 / >100") is parsed byte-faithfully as a string by
# default; "numeric" gives a Float64 read-side view (qualifiers → null). A fuller
# bidirectional/typed XN treatment is future work (see issue tracker).
_XN_MODES = ("string", "numeric")


def _looks_like_ags_text(s: str) -> bool:
    """Does this str look like AGS4 *content* rather than a path? A real AGS4
    file's first non-blank line is always a quoted GROUP record (Rule 3), and a
    path never contains a newline — either signal means text."""
    head = s.lstrip("﻿ \r\n\t")[:8]
    return head.startswith('"GROUP"') or "\n" in s[:512]


def _resolve_source(
    source: Any = None,
    *,
    path: str | os.PathLike[str] | None = None,
    text: str | None = None,
    data: bytes | bytearray | memoryview | None = None,
) -> tuple[str | None, str | None, bytes | None]:
    """Normalise any input — a path, a file-like, raw bytes, or AGS4 text — to the
    ``(path, text, data)`` triple the native parser takes (exactly one non-None).

    A single positional ``source`` is sniffed: a file-like (``.read()``) or bytes
    route to ``data``/``text``; a str/PathLike is a **path** if it exists on disk
    (the unambiguous case, checked first) else sniffs as AGS4 text. The explicit
    ``path=`` / ``text=`` / ``data=`` keywords bypass the sniff for an ambiguous
    input (e.g. a file literally named ``"GROUP",PROJ``)."""
    explicit = [
        k for k, v in (("path", path), ("text", text), ("data", data)) if v is not None
    ]
    if len(explicit) > 1:
        raise TypeError(
            f"pass only one of path= / text= / data= (got {', '.join(explicit)})"
        )
    if path is not None:
        return str(path), None, None
    if text is not None:
        return None, text, None
    if data is not None:
        return None, None, bytes(data)
    if source is None:
        raise TypeError(
            "provide a source (path / text / bytes / file-like) or path=/text=/data="
        )
    if hasattr(source, "read"):  # file-like (io.BytesIO / io.StringIO / open file)
        buf = source.read()
        return (None, buf, None) if isinstance(buf, str) else (None, None, bytes(buf))
    if isinstance(source, (bytes, bytearray, memoryview)):
        return None, None, bytes(source)
    s = os.fspath(source) if isinstance(source, os.PathLike) else source
    if isinstance(s, str):
        try:
            if os.path.exists(s):  # a path that exists on disk is unambiguous
                return s, None, None
        except OSError:
            pass  # interior NUL / over-long → not a usable path; fall through
        except ValueError:
            pass  # path-check raised on a non-path string → treat as content below
        if _looks_like_ags_text(s):
            return None, s, None
        return s, None, None  # treat as a (missing) path — the parser raises NotFound
    raise TypeError(f"unsupported source type {type(source).__name__}")


def _source_bytes(source: Any) -> bytes:
    """Resolve any diff input — a path, AGS4 text, raw bytes, file-like, or an
    :class:`Ags4File` — to the raw bytes the native ``diff_files`` parses."""
    if isinstance(source, Ags4File):
        return source.bytes
    p, txt, raw = _resolve_source(source)
    if raw is not None:
        return raw
    if txt is not None:
        return txt.encode("utf-8")
    from pathlib import Path

    return Path(p).read_bytes()


class Report:
    """Outcome of :func:`validate`. ``findings`` is a polars frame; ``to_json``
    / ``to_ndjson`` are byte-faithful to the Rust ``lat-check`` binary."""

    __slots__ = ("_r",)

    def __init__(self, r: dict) -> None:
        self._r = r

    @classmethod
    def from_cert(cls, cert, src=None) -> Report:
        """Synthesise a clean report from a fresh certificate — the engine-skipped
        outcome of ``.validate()`` on an ``index=``-certified file. :attr:`resolution`
        is the sentinel ``"certified"`` (the engine never emits it), :attr:`count`
        is 0, and the edition is the cert's. The clean verdict's provenance is the
        certificate's stamp (``cert.validator`` / ``cert.checked_at``)."""
        import json

        if src is not None and src[0] is not None:
            label = src[0]
        elif src is not None and src[2] is not None:
            label = "<bytes>"
        else:
            label = "<text>"
        return cls(
            {
                "ok": True,
                "file": label,
                "dict_version": cert.edition,
                "resolution": "certified",
                "count": 0,
                "exit_code": 0,
                "findings": [],
                "json": json.dumps({"file": label, "findings": {}}),
                "ndjson": "",
            }
        )

    @property
    def file(self) -> str:
        return self._r["file"]

    @property
    def dict_version(self) -> str:
        return self._r["dict_version"]

    @property
    def resolution(self) -> str:
        """How the edition was resolved — ``"exact"`` / ``"fallback"`` / ``"forced"``
        from the engine, or the sentinel ``"certified"`` when this verdict came from
        a fresh ``index=`` certificate (the rule engine was skipped)."""
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
        """Polars frame, one row per finding. Columns:

        - ``rule`` — ``"AGS Format Rule N"`` (or an ``"FYI …"`` key for FYI findings).
        - ``line`` — nullable ``Int64`` source line.
        - ``group`` / ``desc`` — the AGS group code and the human-readable message.
        - ``severity`` — ``"error"`` (default), ``"warning"`` or ``"fyi"``; lets a
          ``validate(warnings=True, fyi=True)`` run separate the tiers straight from
          the frame.
        - ``target`` — what the finding points at: ``"line"`` (default), ``"heading"``,
          ``"cell"`` or ``"group"``.
        - ``heading`` / ``field_index`` / ``data_row`` — the offending heading name,
          its 0-based column index, and the 0-based data-row index, when the finding
          pins them (else null).

        The within-line character span (``char_span``) is editor-oriented and lives on
        :meth:`by_rule` / :meth:`to_json` / :meth:`to_ndjson`, not this flat frame."""
        items = self._r["findings"]
        return pl.DataFrame(
            {
                "rule": pl.Series([f["rule"] for f in items], dtype=pl.String),
                "line": pl.Series([f["line"] for f in items], dtype=pl.Int64),
                "group": pl.Series([f["group"] for f in items], dtype=pl.String),
                "desc": pl.Series([f["desc"] for f in items], dtype=pl.String),
                # severity is omitted at the boundary when Error (the default), so
                # absent → "error"; warning/fyi findings carry it explicitly.
                "severity": pl.Series(
                    [f.get("severity", "error") for f in items], dtype=pl.String
                ),
                "target": pl.Series(
                    [f.get("target", "line") for f in items], dtype=pl.String
                ),
                "heading": pl.Series([f.get("heading") for f in items], dtype=pl.String),
                "field_index": pl.Series(
                    [f.get("field_index") for f in items], dtype=pl.Int64
                ),
                "data_row": pl.Series([f.get("data_row") for f in items], dtype=pl.Int64),
            }
        )

    def by_rule(self) -> dict[str, list[dict]]:
        """``{"AGS Format Rule N": [{line, group, desc, severity, ...}, ...]}`` — the
        spec-rule grouping (sorted, like the Rust BTreeMap). Each finding carries
        ``line`` / ``group`` / ``desc`` plus ``severity`` (always present — ``"error"``
        by default) and whatever location fields the finding pins (``target``,
        ``heading``, ``field_index``, ``data_row``, ``char_span``)."""
        out: dict[str, list[dict]] = {}
        for f in self._r["findings"]:
            # Pass every field through verbatim except `rule` (it's the dict key);
            # normalise `severity` so callers can read it unconditionally.
            item = {k: v for k, v in f.items() if k != "rule"}
            item.setdefault("severity", "error")
            out.setdefault(f["rule"], []).append(item)
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
    python-ags4 HEADING-column shape). ``save`` round-trips byte-faithfully
    from the retained Rust parse, independent of which groups were touched."""

    __slots__ = (
        "_backend",
        "_bytes",
        "_cert",
        "_con",
        "_fix_report",
        "_last_check_files",
        "_last_forced",
        "_p",
        "_registered",
        "_report",
        "_src",
        "_text",
        "_xn",
    )

    def __init__(
        self,
        parsed: dict,
        backend: str = "polars",
        *,
        xn: str = "string",
        _src: tuple[str | None, str | None, bytes | None] | None = None,
    ) -> None:
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
        if xn not in _XN_MODES:
            raise ValueError(f"xn must be one of {_XN_MODES} (got {xn!r})")
        self._p = parsed
        self._backend = backend
        # XN-column read mode: "string" (byte-faithful, default) or "numeric"
        # (XN-typed columns cast to Float64 in the engine; non-numeric tokens →
        # null). Read-side only — write-back stays byte-faithful from the parse.
        self._xn = xn
        self._con = None  # lazy DuckDB engine (first group access / sql / connection)
        self._registered: set[str] = set()  # groups loaded into _con
        # The (path, text, data) this handle was read from — lets chainable
        # `.validate()` run the rule engine on the ORIGINAL source (matching line
        # numbers), not a re-emit. None for a synthesised handle (falls back to emit).
        self._src = _src
        self._report: Report | None = None  # last `.validate()` outcome
        self._text: str | None = None  # memoised AGS4 re-emit (.text)
        self._bytes: bytes | None = None  # memoised UTF-8 of .text (.bytes)
        # A fresh `.ags.idx` certificate, set by `read(index=...)` only after it
        # matched this file's bytes — lets `.validate()` skip the rule engine.
        self._cert = None
        # The check profile of the most recent `.validate()` engine run — what
        # `.certify()` stamps into the cert so a later skip can match profiles.
        self._last_check_files = False
        self._last_forced = False
        # The FixResult from the `.fix()` that produced this handle (what was
        # applied + the residual findings); None unless this came from `.fix()`.
        self._fix_report: FixResult | None = None

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
            select = "*" if self._xn == "string" else self._xn_select(con, tmp, code)
            con.execute(f'CREATE TABLE "{code}" AS SELECT {select} FROM "{tmp}"')
        finally:
            con.unregister(tmp)
        self._registered.add(code)

    def _xn_select(self, con, tmp: str, code: str) -> str:
        """The projection for ``xn="numeric"``: ``* REPLACE (...)`` casting this
        group's XN-typed columns to DOUBLE — non-numeric AGS qualifiers (NP, <5,
        >100, …) become null via ``TRY_CAST``. Registry-driven and intersected
        with the columns actually present, so it's a no-op (``*``) for a group
        with no XN headings or a passthrough group absent from the dictionary."""
        g = _GROUPS.get(code)
        if g is None:
            return "*"  # dynamic / passthrough group — no dictionary XN info
        xn_headings = {h.name for h in g.headings if h.type == "XN"}
        if not xn_headings:
            return "*"
        present = {row[0] for row in con.execute(f'DESCRIBE SELECT * FROM "{tmp}"').fetchall()}
        cols = [c for c in present if c in xn_headings]
        if not cols:
            return "*"
        replace = ", ".join(f'TRY_CAST("{c}" AS DOUBLE) AS "{c}"' for c in cols)
        return f"* REPLACE ({replace})"

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

    def at(self, group: str, values) -> AgsQuery:
        """Filter to a parent entity's records — ``ags.at("LOCA", ["BH01", "BH02"])``
        returns an :class:`AgsQuery` whose ``sub[code]`` yields only the rows of each
        group whose ``{group}_ID`` (e.g. ``LOCA_ID``) is in ``values``, materialising
        only the matching rows (explore a huge file without a huge frame). Chain to
        narrow further (``.at("SAMP", […])``); ``sub.groups`` is the related groups and
        ``sub.frames()`` pulls them all at once. Groups carrying none of the keys pass
        through unfiltered. For a richer predicate, ``.query("SELECT …").filter("…")``."""
        return AgsQuery(self, filters=[(f"{group}_ID", list(values))])

    def query(self, sql: str) -> AgsQuery:
        """Start a lazy, chainable query over the file's groups by clean name — the
        fluent counterpart to :meth:`sql`. Where ``sql()`` hands back a raw DuckDB
        relation (ending the chain), ``query()`` returns an :class:`AgsQuery` you keep
        building (``.filter()``, ``.select()``, ``.at()``) and cash out with a terminal
        (``.frame()`` / ``.to_polars()`` / ``.to_pandas()`` / ``.relation()``)."""
        return AgsQuery(self, base=sql)

    def pipe(self, fn, /, *args, **kwargs):
        """Apply ``fn(self, *args, **kwargs)`` and return its result — a functional
        escape hatch to slot a custom step into a chain
        (``read(p).pipe(my_transform).save(out)``) without leaving the fluent flow."""
        return fn(self, *args, **kwargs)

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

    @property
    def text(self) -> str:
        """Spec-correct AGS4 as text (CRLF, every field quoted, ``"``→``""``) for
        every group in file order — byte-faithful to the source DATA values,
        re-emitted Rust-side from the retained parse (no per-cell rows cross the
        boundary on read). Memoised (the emit is O(size))."""
        if self._text is None:
            self._text = self._p["_handle"].emit()
        return self._text

    @property
    def bytes(self) -> bytes:
        """:attr:`text` encoded UTF-8 — the on-disk / wire form :meth:`save` writes.
        AGS4 is a text format, so ``bytes`` is just ``text.encode("utf-8")``. Memoised."""
        if self._bytes is None:
            self._bytes = self.text.encode("utf-8")
        return self._bytes

    def validate(
        self,
        *,
        dict_version: str | None = None,
        warnings: bool = True,
        fyi: bool = False,
        check_files: bool = False,
    ) -> Self:
        """Validate this file against the AGS4 rules and return ``self`` (chainable —
        ``read(p).validate().query(...)``); the outcome lands on :attr:`report`. Same
        engine as the module-level :func:`validate`, run on the source this handle was
        read from (so line numbers match the original file). A handle built without a
        retained source validates its spec-correct re-emit instead.

        Severity tiers track importance, like a compiler: **errors and WARNINGs show
        by default** (``warnings=True``); pass ``warnings=False`` to drop to
        errors-only, and ``fyi=True`` to add the low-signal FYI tier. (The ``compat``
        shim keeps its own python-ags4-faithful defaults, unaffected by this.)

        **Certificate short-circuit:** if this handle carries a fresh ``index=``
        certificate (from :func:`read`) **minted by the current validator engine**,
        and you ask for an errors-only check, the rule engine is skipped — the cert
        already proves the file validated clean — and :attr:`report` is the
        synthesised certified report (:meth:`Report.from_cert`). A cert from a
        *different/older* engine is re-validated, not trusted (its clean verdict may
        not reproduce under today's rules). Asking for more than the cert vouches for
        runs the engine — which now includes the **default** check, since a cert
        records only the error verdict, not the warning list; pass ``warnings=False``
        to engage the skip on a known-clean file."""
        if (
            self._cert is not None
            and self._cert.matches_native_validator()
            and self._cert.profile_covers(check_files, dict_version)
            and not warnings
            and not fyi
        ):
            self._report = Report.from_cert(self._cert, self._src)
            return self
        # Engine run — remember the check profile so a following `.certify()`
        # stamps it into the cert (errors-only default ⇒ both False).
        self._last_check_files = check_files
        self._last_forced = dict_version is not None
        if self._src is not None:
            path, txt, data = self._src
        else:
            path, txt, data = None, self.text, None
        r = _native.run_check(
            path=path,
            text=txt,
            data=data,
            dict_version=dict_version,
            include_warnings=warnings,
            include_fyi=fyi,
            check_files=check_files,
        )
        self._report = Report(raise_for(r))
        return self

    @property
    def report(self) -> Report:
        """The :class:`Report` from the most recent :meth:`validate` (raises if
        ``validate()`` has not been called on this handle yet)."""
        if self._report is None:
            raise AttributeError("call .validate() before reading .report")
        return self._report

    def _source_bytes(self) -> bytes:
        """The ORIGINAL source bytes this handle was read from — what a certificate
        indexes and fingerprints. NOT the spec-correct re-emit :attr:`bytes`, which
        can differ from a non-canonically-formatted on-disk file. A path re-reads the
        file; raw ``data=`` is returned as-is; ``text=`` is UTF-8-encoded; a
        synthesised handle (no retained source) falls back to the re-emit."""
        if self._src is None:
            return self.bytes
        path, text, data = self._src
        if path is not None:
            return Path(path).read_bytes()
        if data is not None:
            return data
        return text.encode("utf-8")  # type: ignore[union-attr]

    def certify(self, path: str | Path | None = None) -> Path:
        """Mint this file's ``.ags.idx`` validity **certificate** — a clean-validation
        proof plus a byte-offset index — and write it beside the file. REQUIRES a prior
        clean :meth:`validate`: ``certify`` *vouches for* a passed validation, it does
        not run one. Raises if :meth:`validate` was not called, or found finding(s); a
        later ``read(..., index=...)`` consumes the cert to skip re-validation.

        ``path`` defaults to ``<source>.idx`` (``delivery.ags`` → ``delivery.ags.idx``);
        a handle read from text/bytes has no source path, so pass ``path=`` explicitly.
        Returns the written ``Path``. The certificate indexes the original source bytes,
        which must be UTF-8 (the byte index rejects other encodings)."""
        if self._report is None:
            raise Ags4Error(
                "call .validate() before .certify() — certify records a passed "
                "validation, it does not run one"
            )
        if not self._report.is_valid:
            raise Ags4Error(
                f"cannot certify a file with {self._report.count} finding(s); fix "
                "them and re-validate clean first"
            )
        if path is None:
            src_path = self._src[0] if self._src is not None else None
            if src_path is None:
                raise Ags4Error(
                    "no source path to derive the .ags.idx location from; pass "
                    "certify(path=...) for a handle read from text/bytes"
                )
            path = f"{src_path}.idx"
        path = Path(path)
        from datetime import UTC, datetime

        checked_at = datetime.now(UTC).isoformat()
        cert = _native.Sidecar.assemble(
            self._source_bytes(),
            self._report.dict_version,
            checked_at,
            check_files=self._last_check_files,
            edition_forced=self._last_forced,
        )
        path.write_bytes(cert.to_json())
        return path

    def save(self, path: str | Path) -> Path:
        """Write spec-correct AGS4 to ``path`` (UTF-8 — :attr:`bytes`); returns the
        ``Path``. The inverse of :func:`read`."""
        path = Path(path)
        path.write_bytes(self.bytes)
        return path

    def to_excel(
        self, path: str | os.PathLike[str], *, groups: list[str] | None = None
    ) -> dict:
        """Write this file to an XLSX workbook — one sheet per group — and return the
        Rust writer's stats (``{"sheets_written", "rows_written", "warnings"}``).

        Rust-backed via ``laterite_excel`` (``rust_xlsxwriter``); openpyxl and
        pyarrow never enter the dep graph. Sheets carry the AGS HEADING / UNIT /
        TYPE / DATA layout. ``groups`` optionally fixes the sheet order (a subset or
        re-ordering of :attr:`groups`); default is source order. The workbook is
        written from this handle's spec-correct :attr:`bytes`, so it round-trips
        through :func:`from_excel` regardless of how the handle was read."""
        import tempfile

        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d) / "_to_excel.ags"
            tmp.write_bytes(self.bytes)
            return _excel_convert(_native.ags4_to_excel, str(tmp), os.fspath(path), groups)

    @property
    def fix_report(self) -> FixResult | None:
        """The :class:`FixResult` from the :meth:`fix` that produced this handle —
        what was applied and the findings that could **not** be mechanically fixed —
        or ``None`` for a handle not produced by :meth:`fix`."""
        return self._fix_report

    def fix(self, *, risky: bool = False) -> Ags4File:
        """Repair this file and return a new, repaired :class:`Ags4File` — the fluent
        transform, so ``read(path).fix().validate().save(out)`` reads as one chain.
        The **safe** mechanical fixes (CRLF / BOM / embedded-CR / short-row pad /
        numeric reformat / TRAN delimiter+concatenator rows) are applied; ``risky=True``
        also applies the intent-guessing ones (duplicate-heading rename, datetime
        canonicalisation, typography). The same engine the browser fix UI uses.

        Non-destructive — the source on disk is untouched; persist the repaired handle
        with :meth:`save`. The :class:`FixResult` — what was applied and the residual
        findings — rides on the returned handle's :attr:`fix_report`. The new handle
        inherits this one's ``backend`` / ``xn``. For the report itself (and the
        ``in_place=`` / ``out=`` write options) instead of a handle, call the free
        :func:`fix`."""
        if self._src is not None:
            p, t, d = self._src
        else:
            # A synthesised handle has no retained source — fix its re-emit.
            p, t, d = None, None, self.bytes
        report = fix(path=p, text=t, data=d, risky=risky)
        repaired = read(data=report.bytes, backend=self._backend, xn=self._xn)
        repaired._fix_report = report
        return repaired

    def diff(self, other: Any, *, dict_version: str | None = None) -> dict:
        """Compare this file (the **baseline**) against ``other`` (the **revision** —
        a path, AGS4 text, bytes, or another :class:`Ags4File`); returns the
        :func:`diff` ``RevisionDelta`` dict. Rows are matched by the group's KEY
        headings, cells compared through the typed value."""
        return diff(self, other, dict_version=dict_version)

    def __repr__(self) -> str:
        return (
            f"<Ags4File groups={len(self.groups)} backend={self._backend!r} "
            f"tran_ags={self.tran_ags!r}>"
        )


class AgsQuery:
    """A lazy, chainable view over an :class:`Ags4File`'s DuckDB engine — the single
    query type returned by :meth:`Ags4File.at` and :meth:`Ags4File.query`. Nothing
    runs until a terminal. Two modes share the type:

    **Multi-group fan-out** (from ``.at()``) — key-filter several related groups at
    once. ``q[code]`` materialises one group with every applicable ``.at()`` filter,
    ``q.frames()`` pulls all related groups, ``q.groups`` lists them. ``.at()`` filters
    accumulate (AND); a group not carrying a filter's key column passes through
    unfiltered. Filters are parameterised (no SQL injection on the value lists).

    **Single result** (from ``.query()``) — one relation out. ``.filter(pred)`` adds a
    SQL ``WHERE`` fragment, ``.select(*cols)`` projects, and any ``.at()`` filters
    narrow it; finish with ``.frame()`` (handle backend), ``.to_polars()``,
    ``.to_pandas()``, or ``.relation()`` (the raw DuckDB relation, to chain more SQL).

    ``.filter()`` / ``.select()`` / ``.query()`` build the single-result relation and
    don't apply to the multi-group accessors — mixing them raises rather than silently
    dropping a filter. Every chaining method returns a NEW ``AgsQuery`` (immutable
    builder); the parent handle's backend is inherited.

    Future: ``.filter()`` today takes a SQL predicate string (use ``.at()`` for
    parameterised value lists); a later release may also accept typed
    ``column=value`` keywords or column-expression objects for an
    injection-safe, discoverable form — see the reliquary / future-work register."""

    __slots__ = ("_base", "_filters", "_parent", "_predicates", "_projection")

    def __init__(
        self,
        parent: Ags4File,
        *,
        filters: list[tuple[str, list]] | None = None,
        base: str | None = None,
        predicates: list[str] | None = None,
        projection: list[str] | None = None,
    ) -> None:
        self._parent = parent
        self._filters = filters or []  # [(key_col, [values])] — .at(), parameterised IN
        self._base = base  # a .query() SQL string, or None
        self._predicates = predicates or []  # SQL WHERE fragments — .filter()
        self._projection = projection  # [col, ...] — .select(), or None

    # --- chaining (immutable builder; each returns a new AgsQuery) -------------

    def _with(self, **changes) -> AgsQuery:
        state = {
            "filters": self._filters,
            "base": self._base,
            "predicates": self._predicates,
            "projection": self._projection,
        }
        state.update(changes)
        return AgsQuery(self._parent, **state)

    def at(self, group: str, values) -> AgsQuery:
        """Add a parent-entity key filter (e.g. ``.at("LOCA", ["BH01"])``),
        parameterised; accumulates with AND."""
        return self._with(filters=[*self._filters, (f"{group}_ID", list(values))])

    def filter(self, predicate: str) -> AgsQuery:
        """Add a SQL ``WHERE`` fragment to the single-result relation (e.g.
        ``.filter("LOCA_GL > 100 AND LOCA_TYPE = 'CP'")``), AND-ed with any others.
        Finish with a terminal (``.frame()`` / …). For a parameterised value list use
        ``.at()``; you compose the SQL fragment yourself, so don't interpolate
        untrusted values into it."""
        return self._with(predicates=[*self._predicates, predicate])

    def select(self, *columns: str) -> AgsQuery:
        """Project the single-result relation to ``columns``."""
        return self._with(projection=list(columns))

    def query(self, sql: str) -> AgsQuery:
        """Set (or replace) the base SQL the single-result relation reads from."""
        return self._with(base=sql)

    def pipe(self, fn, /, *args, **kwargs):
        """Apply ``fn(self, *args, **kwargs)`` and return its result."""
        return fn(self, *args, **kwargs)

    # --- multi-group fan-out (from .at) ---------------------------------------

    def _guard_fanout(self) -> None:
        if self._base is not None or self._predicates or self._projection is not None:
            raise TypeError(
                "filter()/select()/query() build a single-result query — finish with "
                ".frame()/.to_polars()/.to_pandas()/.relation(); they don't apply to "
                "the multi-group q[code]/.frames() accessors. Use .at() for multi-group "
                "key filters."
            )

    def _key_where(self, cols: set[str]) -> tuple[str, list]:
        """The accumulated ``.at()`` filters as a WHERE clause + params, keeping only
        those whose key column is present in ``cols`` (others pass through)."""
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
        return (" AND ".join(clauses) if clauses else "TRUE"), params

    @property
    def groups(self) -> list[str]:
        """The related groups — those carrying at least one ``.at()`` filter's key."""
        self._guard_fanout()
        p = self._parent
        keys = {k for k, _ in self._filters}
        return [g for g in p.groups if keys.intersection(p.headings(g))]

    def __contains__(self, code: str) -> bool:
        return code in self._parent

    def __getitem__(self, code: str):
        """``code`` filtered by every applicable ``.at()`` key (groups carrying none
        of the keys pass through), materialised to the handle's backend."""
        self._guard_fanout()
        p = self._parent
        p._register(code)
        where, params = self._key_where(set(p.headings(code)))
        rel = p._engine().sql(
            f'SELECT * FROM "{code}" WHERE {where}', params=params or None
        )
        return p._materialize(rel)

    table = __getitem__

    def frames(self) -> dict:
        """``{group: frame}`` for every related group, each filtered — pull a
        location's whole related record set in one call."""
        self._guard_fanout()
        return {g: self[g] for g in self.groups}

    # --- single-result terminals (from .query / .filter / .select) ------------

    def relation(self):
        """The built DuckDB relation (lazy — ``.df()`` / ``pl.from_arrow`` to
        materialise, or chain more SQL). Requires a base set via :meth:`Ags4File.query`
        / :meth:`query`; ``.at()`` filters and ``.filter()`` predicates that apply to
        the base's columns narrow it, and ``.select()`` projects."""
        p = self._parent
        if self._base is None:
            raise TypeError(
                "no base to run — start a single-result query with .query('SELECT … "
                "FROM …'), or use the multi-group accessors q[code] / q.frames() "
                "after .at()."
            )
        p._register_all()
        engine = p._engine()
        cols = set(engine.sql(self._base).columns)  # schema only — doesn't execute
        key_where, params = self._key_where(cols)
        clauses = [key_where] if key_where != "TRUE" else []
        clauses.extend(f"({pred})" for pred in self._predicates)
        where = " AND ".join(clauses) if clauses else "TRUE"
        sel = ", ".join(f'"{c}"' for c in self._projection) if self._projection else "*"
        return engine.sql(
            f"SELECT {sel} FROM ({self._base}) AS _q WHERE {where}",
            params=params or None,
        )

    def frame(self):
        """Materialise the single-result relation to the handle's backend
        (polars / pandas)."""
        return self._parent._materialize(self.relation())

    def to_polars(self):
        """Materialise the single-result relation to a polars frame (pyarrow-free),
        regardless of the handle's default backend."""
        return frame_from_arrow(self.relation())

    def to_pandas(self):
        """Materialise the single-result relation to a pandas frame (DuckDB's NumPy
        ``.df()``, pyarrow-free), regardless of the handle's default backend."""
        return self.relation().df()

    def __repr__(self) -> str:
        bits: list[str] = []
        if self._filters:
            bits.append(
                "at[" + ", ".join(f"{k} in {v!r}" for k, v in self._filters) + "]"
            )
        if self._base:
            bits.append(f"query={self._base!r}")
        if self._predicates:
            bits.append("filter[" + " AND ".join(self._predicates) + "]")
        if self._projection is not None:
            bits.append(f"select{self._projection!r}")
        return f"<AgsQuery {' '.join(bits) or 'empty'}>"


def validate(
    source: Any = None,
    *,
    text: str | None = None,
    dict_version: str | None = None,
    warnings: bool = True,
    fyi: bool = False,
    check_files: bool = False,
) -> Report:
    """Validate an AGS4 file (path) or in-memory ``text=`` against the AGS4.1
    rules. Raises for un-validatable input (missing / not AGS4 / unsupported
    edition); rule *violations* come back in the :class:`Report`.

    Severity tiers track importance: **errors and WARNINGs show by default**
    (``warnings=True``) — pass ``warnings=False`` for errors-only, ``fyi=True`` to
    add the low-signal FYI tier. (The ``compat`` shim keeps its own
    python-ags4-faithful defaults.)"""
    path, txt, data = _resolve_source(source, text=text)
    r = _native.run_check(
        path=path,
        text=txt,
        data=data,
        dict_version=dict_version,
        include_warnings=warnings,
        include_fyi=fyi,
        check_files=check_files,
    )
    return Report(raise_for(r))


def read(
    source: Any = None,
    *,
    path: str | os.PathLike[str] | None = None,
    text: str | None = None,
    data: bytes | bytearray | memoryview | None = None,
    index: str | os.PathLike[str] | None = None,
    encoding: str | None = None,
    backend: str = "polars",
    xn: str = "string",
) -> Ags4File:
    """Read AGS4 — from a path, a file-like, raw bytes, or in-memory text — into
    an :class:`Ags4File` over an in-memory DuckDB engine. The inverse of
    :meth:`Ags4File.save`; to build AGS4 from your own data use :func:`build_ags4`.

    A single positional argument is auto-detected (path / file-like / bytes /
    AGS4 text); pass ``path=`` / ``text=`` / ``data=`` to be explicit for an
    ambiguous input. ``encoding`` (a WHATWG label, default UTF-8) applies to
    bytes / path input — text is already decoded. ``backend`` is the default
    frame type for ``ags[code]`` — ``"polars"`` (default) or ``"pandas"`` (both
    pyarrow-free).

    ``xn`` controls AGS ``XN``-typed columns (numeric values that may carry a
    non-numeric qualifier — ``NP`` / ``<5`` / ``>100``): ``"string"`` (default)
    keeps them byte-faithful as text; ``"numeric"`` casts them to ``Float64``
    across the whole handle (``ags[code]`` / ``sql`` / ``at``), non-numeric tokens
    becoming null. Read-side only — ``save`` / ``text`` / ``bytes`` stay
    byte-faithful regardless. (A fuller bidirectional XN treatment is future work.)

    ``index`` is the explicit path to this file's ``.ags.idx`` certificate (from
    :meth:`Ags4File.certify`). It is opt-in — there is no autodiscovery. When given,
    the cert is loaded and freshness-checked against the source bytes (format version
    + size + SHA-256); a **stale** cert raises :class:`StaleCertError` here (fail-fast
    — an explicit ``index=`` asserts the cert is for this file), while a **fresh** one
    is carried so a later default :meth:`Ags4File.validate` skips the rule engine."""
    p, txt, raw = _resolve_source(source, path=path, text=text, data=data)
    res = _native.parse_arrow(path=p, text=txt, data=raw, encoding=encoding)
    handle = Ags4File(raise_for(res), backend=backend, xn=xn, _src=(p, txt, raw))
    if index is not None:
        cert = _native.Sidecar.from_json(Path(index).read_bytes())
        if not cert.is_fresh_for(handle._source_bytes()):
            raise StaleCertError(
                f"certificate {os.fspath(index)!r} does not match the file it was "
                "read for (size/SHA-256 differ) — the source changed under it; "
                "rebuild it with read(...).validate().certify()"
            )
        handle._cert = cert
    return handle


# `source` is the fluent-chain entry name (`laterite.source(x).validate()…`);
# `read` is the plain-verb name. Same callable — one surface, two vocabularies.
source = read


def _excel_convert(fn, *args) -> dict:
    """Call a native AGS4↔XLSX conversion fn and normalise its outcome: the stats
    PyDict becomes a plain ``dict``; the engine's "no valid AGS4 data" RuntimeError
    is re-raised as :class:`NotAgs4Error` to match the rest of the read surface."""
    try:
        return dict(fn(*args))
    except RuntimeError as exc:
        if "No valid AGS4 data" in str(exc):
            raise NotAgs4Error(str(exc)) from exc
        raise


def to_excel(
    source: Any = None,
    output: str | os.PathLike[str] | None = None,
    *,
    path: str | os.PathLike[str] | None = None,
    text: str | None = None,
    data: bytes | bytearray | memoryview | None = None,
    groups: list[str] | None = None,
) -> dict:
    """Convert AGS4 to an XLSX workbook — one sheet per group — and return the Rust
    writer's stats (``{"sheets_written", "rows_written", "warnings"}``).

    Rust-backed via ``laterite_excel`` (``rust_xlsxwriter``); openpyxl and pyarrow
    never enter the dep graph. ``source`` is anything :func:`read` accepts (a path /
    file-like / bytes / AGS4 text) or an already-:func:`read` :class:`Ags4File`;
    ``output`` is the ``.xlsx`` path to write. ``groups`` optionally fixes the sheet
    order (a subset or re-ordering of the file's groups); default is source order."""
    if output is None:
        raise TypeError("to_excel() requires an output path (the .xlsx to write)")
    if isinstance(source, Ags4File):
        return source.to_excel(output, groups=groups)
    p, txt, raw = _resolve_source(source, path=path, text=text, data=data)
    if p is not None:
        # A real on-disk AGS4 file → one Rust pass straight to XLSX (no re-emit).
        return _excel_convert(_native.ags4_to_excel, os.fspath(p), os.fspath(output), groups)
    # text / bytes → parse then write from the spec-correct re-emit.
    return read(text=txt, data=raw).to_excel(output, groups=groups)


def from_excel(
    source: str | os.PathLike[str],
    output: str | os.PathLike[str] | None = None,
    *,
    format_numeric_columns: bool = True,
    backend: str = "polars",
    xn: str = "string",
) -> dict | Ags4File:
    """Convert an AGS4-shaped XLSX workbook to AGS4.

    Rust-backed via ``laterite_excel`` (``calamine``). Each worksheet with a
    ``HEADING`` column becomes one group; columns not matching Rule 19's heading
    pattern are dropped. With ``output`` given, writes an AGS4 file and returns the
    Rust converter's stats; with ``output=None`` (default), returns a parsed
    :class:`Ags4File` read straight from the conversion. ``format_numeric_columns``
    (default ``True``) re-formats DATA cells to their column's TYPE precision so
    floats from XLSX keep trailing zeros; ``backend`` / ``xn`` apply only to the
    returned-handle form."""
    src = os.fspath(source)
    if output is not None:
        return _excel_convert(
            _native.excel_to_ags4, src, os.fspath(output), bool(format_numeric_columns)
        )
    import tempfile

    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d) / "_from_excel.ags"
        _excel_convert(_native.excel_to_ags4, src, str(tmp), bool(format_numeric_columns))
        raw = tmp.read_bytes()
    # Read from bytes (not the now-deleted temp path) so the handle is self-contained
    # — its `.validate()` / `.certify()` don't depend on a vanished file.
    return read(data=raw, backend=backend, xn=xn)


def dict_for(source: Any = None, *, text: str | None = None) -> tuple[str, str]:
    """``(edition, resolution)`` the engine would validate this file against —
    e.g. ``("4.1.1", "fallback")`` — without running rules."""
    path, txt, data = _resolve_source(source, text=text)
    p = raise_for(_native.parse_primitives(path=path, text=txt, data=data))
    return _native.resolve_dict(p.get("tran_ags"), None)


def list_rules() -> list[dict]:
    """The rule catalogue the engine enforces — one dict per AGS4 rule with
    ``rule`` (e.g. ``"10c"``), ``title``, ``checks`` (a plain-English summary),
    ``severity`` (``"error"`` / ``"fyi"`` / ``"mixed"``), ``fixable`` (whether
    :func:`fix` can repair it), and ``observations`` (the cited ``O-N`` divergence
    notes). Read-only and file-independent — sourced from the engine's gated rule
    metadata, so it always matches the rules ``validate`` actually runs."""
    import json

    return json.loads(_native.list_rules())["rules"]


class BuildResult:
    """The product of :func:`build_ags4`: the AGS4 ``bytes``, the validator
    ``findings`` on those bytes (post-fix in AutoFix mode), and the count of
    safe fixes applied. ``.text`` decodes the bytes; ``.save(path)`` writes them."""

    __slots__ = ("bytes", "findings", "fixes_applied")

    def __init__(self, data: bytes, findings: list[dict], fixes_applied: int) -> None:
        self.bytes = data
        self.findings = findings
        self.fixes_applied = fixes_applied

    @property
    def text(self) -> str:
        return self.bytes.decode("utf-8")

    def save(self, path: str | Path) -> Path:
        path = Path(path)
        path.write_bytes(self.bytes)
        return path

    def __repr__(self) -> str:
        return (
            f"<BuildResult {len(self.bytes)} bytes, "
            f"{len(self.findings)} finding(s), fixes_applied={self.fixes_applied}>"
        )


def _typed_group_code(obj: Any) -> str | None:
    """The AGS group code if ``obj`` is a typed-graph node, else ``None``.

    The Python twin of laterite-node's ``instanceof AgsGroup`` test. Node has a
    shared ``AgsGroup`` base; the compiled ``#[pyclass]`` types here don't (their
    MRO is just ``[PROJ, object]``), so we key off identity instead: a compiled
    group is its class name being a known code in the native module; a
    ``laterite.dynamic`` passthrough carries its code on ``_ags_code``."""
    code = getattr(type(obj), "_ags_code", None)
    if isinstance(code, str):
        return code
    cls = type(obj)
    if cls.__module__ == "laterite._laterite_native" and cls.__name__ in _GROUPS:
        return cls.__name__
    return None


def _typed_graph_to_items(root: Any) -> list[tuple[str, pl.DataFrame]]:
    """Walk a typed PROJ tree depth-first into ``(code, polars frame)`` pairs —
    the Python twin of laterite-node's ``walkTree``. Every *declared* heading
    becomes a column (null if unset); the registry's parent→child links drive
    recursion via the ``<child>s`` accessors, so only the PROJ-rooted subtree is
    walked. Root-metadata groups (TRAN/UNIT/TYPE/ABBR/DICT, parent ``None``) and
    orphaned subtrees (a child whose intermediate parent is absent) aren't part
    of the tree — emit's autofix re-adds the mandatory TRAN rows. Coverage for
    standard groups is identical to Node's walk by construction (same dictionary
    parent→child map); additionally — a Python-only superset, since Node has no
    passthrough surface — a custom group that ``read_typed`` hangs off a parent
    (a dynamic ``laterite.dynamic`` node on an undeclared ``<code>s`` accessor)
    is also carried, so a ``read_typed`` → ``build_ags4`` round trip is lossless."""
    buckets: dict[str, list[dict[str, Any]]] = {}

    def heading_names(code: str, node: Any) -> list[str]:
        desc = _GROUPS.get(code)
        if desc is not None:
            return [h.name for h in desc.headings]
        specs = getattr(type(node), "_ags_heading_specs", None)
        if specs:
            return [name for name, _type in specs]
        raise TypeError(f"build_ags4: cannot determine headings for group {code!r}")

    def visit(node: Any) -> None:
        code = _typed_group_code(node)
        if code is None:
            raise TypeError(
                "build_ags4: not a known typed AGS group instance "
                f"(got {type(node).__name__!r})"
            )
        names = heading_names(code, node)
        buckets.setdefault(code, []).append(
            {n: getattr(node, n.lower(), None) for n in names}
        )
        # Registry-declared children (standard parent→child links).
        declared = [f"{child.code.lower()}s" for child in _child_groups(code)]
        for accessor in declared:
            for kid in getattr(node, accessor, None) or []:
                visit(kid)
        # Passthrough children: read_typed hangs a *custom* group off its
        # parent's `<code>s` accessor via setattr (the registry doesn't know
        # that link), so it lives in the instance __dict__ rather than on a
        # declared accessor. Pick up any extra list of typed nodes so a
        # read_typed → build_ags4 round trip doesn't silently drop it.
        declared_set = set(declared)
        for attr, value in (getattr(node, "__dict__", None) or {}).items():
            if attr in declared_set or not isinstance(value, list) or not value:
                continue
            if _typed_group_code(value[0]) is not None:
                for kid in value:
                    visit(kid)

    visit(root)

    items: list[tuple[str, pl.DataFrame]] = []
    for code, rows in buckets.items():
        names = list(rows[0])  # every row of a group shares its heading set
        df = pl.DataFrame({n: [r[n] for r in rows] for n in names})
        # An all-None column infers as polars Null; the Arrow canonicaliser
        # wants a (possibly empty) string column, not a Null one — cast those.
        null_cols = [c for c, dtype in df.schema.items() if dtype == pl.Null]
        if null_cols:
            df = df.with_columns([pl.col(c).cast(pl.Utf8) for c in null_cols])
        items.append((code, df))
    return items


def build_ags4(
    groups: Mapping[str, Any] | list[tuple[str, Any]] | Any,
    *,
    dict_version: str | None = None,
    mode: str = "autofix",
) -> BuildResult:
    """Build valid AGS4 from your own per-group data — the data→AGS4 door.
    Where :func:`read` loads an *existing* file, ``build_ags4`` *constructs* a new
    one (and autofixes + validates it); persist the result with ``BuildResult.save``.

    ``groups`` is either a **typed-graph root** — a ``PROJ`` instance with its
    children attached (``PROJ(...)``; ``proj.locas.append(LOCA(...))``), walked
    depth-first via the registry's parent→child links — OR a mapping / list of
    ``(code, frame)`` pairs where each frame (pandas **or** polars) has **column
    names that are the AGS headings** (e.g. ``LOCA_ID``, ``LOCA_GL``). This
    mirrors laterite-node's ``buildAgs4``, which accepts the same two shapes.
    UNIT/TYPE are filled from the chosen dictionary edition. Order is preserved
    (pass an ordered mapping or a list of ``(code, frame)`` pairs; put ``PROJ``
    first). A typed graph emits only its PROJ-rooted subtree — root-metadata
    groups (TRAN/UNIT/TYPE/ABBR/DICT) aren't children of PROJ, so use the
    ``(code, frame)`` form if you need to carry those.

    The frame crosses into Rust zero-copy via the Arrow C-stream — **pyarrow-free
    for polars** (so this stays a base feature, no ``[compat]``) and for pandas
    ≥ 2.2; an older pandas routes through DuckDB (pandas only ships via
    ``[compat]``, which carries the deps). Each cell is formatted to its
    canonical AGS4 string. ``mode``:

    * ``"autofix"`` (default) — build, then apply the *safe* mechanical fixes
      (pad decimals, normalise, …); ``BuildResult.findings`` holds whatever
      couldn't be safely fixed (e.g. a missing required heading).
    * ``"report"`` — build unchanged, return findings for you to act on.
    * ``"strict"`` — raise if the output violates any error-severity rule.

    ``dict_version`` is one of ``4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2`` (default
    ``4.1.1``)."""
    if dict_version is None:
        dict_version = "4.1.1"
    import json

    if _typed_group_code(groups) is not None:
        items = _typed_graph_to_items(groups)
    elif isinstance(groups, Mapping):
        items = list(groups.items())
    else:
        items = list(groups)
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
    data, findings_json, fixes = _native.emit_ags4_from_arrow(tables, dict_version, mode)
    by_rule: dict[str, list[dict]] = json.loads(findings_json)
    findings = [{"rule": rule, **f} for rule, items_ in by_rule.items() for f in items_]
    return BuildResult(data, findings, fixes)


class FixResult:
    """The product of :func:`fix` (and carried on :attr:`Ags4File.fix_report`): the
    repaired AGS4 ``bytes``, the ``findings`` that REMAIN after the fixes were
    applied, and ``applied`` — the list of fixes that were made (each ``{kind, label,
    rule, line, risk}``). ``fixes_applied`` is ``len(applied)``; ``.text`` decodes the
    bytes; ``.save(path)`` writes them. The fixed output is always UTF-8 with no
    BOM (so fixing a CRLF/encoding issue normalises both)."""

    __slots__ = ("applied", "bytes", "dict_version", "findings")

    def __init__(
        self, data: bytes, findings: list[dict], applied: list[dict], dict_version: str
    ) -> None:
        self.bytes = data
        self.findings = findings
        self.applied = applied
        self.dict_version = dict_version

    @property
    def fixes_applied(self) -> int:
        return len(self.applied)

    @property
    def text(self) -> str:
        return self.bytes.decode("utf-8")

    def save(self, path: str | Path) -> Path:
        path = Path(path)
        path.write_bytes(self.bytes)
        return path

    def __repr__(self) -> str:
        return (
            f"<FixResult {len(self.bytes)} bytes, applied={self.fixes_applied}, "
            f"{len(self.findings)} residual finding(s)>"
        )


def fix(
    source: Any = None,
    *,
    path: str | os.PathLike[str] | None = None,
    text: str | None = None,
    data: bytes | bytearray | memoryview | None = None,
    dict_version: str | None = None,
    encoding: str | None = None,
    risky: bool = False,
    in_place: bool = False,
    out: str | os.PathLike[str] | None = None,
) -> FixResult:
    """Mechanically repair an existing AGS4 file and return a :class:`FixResult`.

    The same fix engine the browser uses, run headless: ``source`` is anything
    :func:`read` accepts (path / file-like / bytes / AGS4 text). The **safe**
    fixes (CRLF / BOM / embedded-CR / short-row pad / numeric reformat / the TRAN
    delimiter+concatenator rows) are always applied; ``risky=True`` also applies
    the intent-guessing ones (duplicate-heading rename, ``dd/mm`` datetime
    canonicalisation, smart-quote→ASCII typography). The result is re-validated, so
    ``FixResult.findings`` is what could **not** be mechanically fixed.

    Non-destructive by default — the fixed bytes come back on the result and are
    written only if you ask: ``in_place=True`` overwrites the source file (requires
    a path source), or ``out=<path>`` writes there. Otherwise call
    :meth:`FixResult.save`. The output is UTF-8 (fixing a non-UTF-8 file also
    normalises its encoding)."""
    import json

    if in_place and out is not None:
        raise TypeError("pass only one of in_place=True / out=<path>")
    p, txt, raw = _resolve_source(source, path=path, text=text, data=data)
    res = raise_for(
        _native.fix_file(
            path=p,
            text=txt,
            data=raw,
            dict_version=dict_version,
            encoding=encoding,
            include_risky=risky,
        )
    )
    by_rule: dict[str, list[dict]] = json.loads(res["findings_json"])
    findings = [{"rule": rule, **f} for rule, items_ in by_rule.items() for f in items_]
    result = FixResult(res["fixed"], findings, list(res["applied"]), res["dict_version"])

    if in_place:
        if p is None:
            raise Ags4Error(
                "fix(in_place=True) needs a path source to overwrite; pass a file "
                "path, or use out=<path> / FixResult.save(path)"
            )
        result.save(p)
    elif out is not None:
        result.save(out)
    return result


def diff(
    a: Any,
    b: Any,
    *,
    dict_version: str | None = None,
    encoding: str | None = None,
) -> dict:
    """Compare two AGS4 documents and return their **revision diff**.

    ``a`` (the baseline) and ``b`` (the revision) are each anything :func:`read`
    accepts — a path, AGS4 text, raw bytes, a file-like, or an :class:`Ags4File`.
    Returns a ``RevisionDelta`` dict — ``groups`` (a per-group list of row/heading
    deltas) plus ``groups_added`` / ``groups_removed`` and the
    ``total_added`` / ``total_removed`` / ``total_changed`` counts.

    Rows are matched by the group's dictionary **KEY** headings (not line order, so
    a re-sorted file still pairs the same boreholes), and cells are compared through
    the **typed** value — a formatting-only change (``"1.0"`` → ``"1.00"``) is not a
    diff. The dictionary edition is the revision's ``TRAN_AGS`` (force it with
    ``dict_version``). The same engine the browser's revision-diff tool and
    ``lat-check <a> --diff <b>`` use."""
    import json

    r = _native.diff_files(
        _source_bytes(a),
        _source_bytes(b),
        dict_version=dict_version,
        encoding=encoding,
    )
    return json.loads(raise_for(r)["delta_json"])
