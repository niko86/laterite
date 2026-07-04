"""Batch D: the enumerated-string params are `Literal` types, each gated against
its source of truth so the type can never drift from what the code accepts.

(#294 Batch D — `Backend` / `XnMode` / `BuildMode` / `Edition`, the same
discipline as `FixableRule`: a hand-written `Literal` is only safe if a test
proves it equals the runtime set.)
"""

from __future__ import annotations

import json
import typing
from pathlib import Path

import laterite as L
import polars as pl

_DICT = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-core"
    / "data"
    / "ags_dictionary.json"
)


def test_backend_literal_matches_the_runtime_set():
    """`Backend` == the tuple `read(backend=)` validates against."""
    assert set(typing.get_args(L.Backend)) == set(L._BACKENDS)


def test_xnmode_literal_matches_the_runtime_set():
    """`XnMode` == the tuple `read(xn=)` validates against."""
    assert set(typing.get_args(L.XnMode)) == set(L._XN_MODES)


def test_edition_literal_matches_the_dictionary_editions():
    """`Edition` (the `dict_version=` values) == the bundled dictionary's own
    `editions` list — so a newly bundled edition forces the type to update."""
    editions = json.loads(_DICT.read_text())["editions"]
    assert set(typing.get_args(L.Edition)) == set(editions)


def test_buildmode_literal_matches_the_engine_modes():
    """Every `BuildMode` value is accepted by the emit engine, and an unknown
    mode is rejected — gates the type against the Rust `EmitMode` parser."""
    frame = {"PROJ": pl.DataFrame({"PROJ_ID": ["P1"]})}

    def accepts(mode: str) -> bool:
        try:
            L.build_ags4(frame, mode=mode)
            return True
        except Exception as e:  # strict may raise a *violation* — still "accepted"
            return "unknown mode" not in str(e).lower()

    for mode in typing.get_args(L.BuildMode):
        assert accepts(mode), f"engine rejected documented BuildMode {mode!r}"
    assert not accepts("nope"), "engine should reject an unknown mode"
