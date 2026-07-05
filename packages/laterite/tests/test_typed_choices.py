"""Batch D: the enumerated-string params are `Literal` types, each gated against
its source of truth so the type can never drift from what the code accepts.

(#294 Batch D — `Backend` / `XnMode` / `BuildMode` / `Edition`, the same
discipline as `FixableRule`: a hand-written `Literal` is only safe if a test
proves it equals the runtime set.)
"""

from __future__ import annotations

import json
import re
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


_INDEX_TS = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-node"
    / "ts"
    / "index.ts"
)


def test_node_fixablerule_union_matches_python():
    """The Node `FixableRule` TS union (`only`/`exclude` labels, #394) must equal
    Python's `FixableRule` — which is itself gated to the engine's `fixable_rules()`
    (test_fix_selection). So Node's typed choices can't drift from what the shared
    fix engine actually repairs, without hand-listing rules on the Node side."""
    src = _INDEX_TS.read_text(encoding="utf-8")
    m = re.search(r"export type FixableRule =\s*([^;]+);", src)
    assert m is not None, "FixableRule union not found in index.ts"
    node = set(re.findall(r'"([^"]+)"', m.group(1)))
    assert node == set(typing.get_args(L.FixableRule)), (
        f"Node FixableRule {sorted(node)} != Python {sorted(typing.get_args(L.FixableRule))} "
        "— update the Node union in index.ts (it mirrors the engine's fixable set)"
    )
