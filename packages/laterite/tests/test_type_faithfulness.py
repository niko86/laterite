"""Type-faithfulness: the declared hints must hold at **runtime**.

Astral ``ty`` (the CI gate, configured in ``[tool.ty]``) proves the hints are
internally consistent *statically*; this asserts the complementary half — that
real calls actually RETURN the declared types, ACCEPT every documented
``Literal`` value, REJECT unknown kwargs, and that the chained ``Ags4File``
methods honour their ``-> Self`` contract. A static checker reads the ``.pyi``;
it can't notice a ``#[pyclass]`` method whose Rust return drifts from that stub,
nor prove ``validate()`` hands back the *same* handle. This does.

Same drift-gate family as ``test_typed_choices`` (a ``Literal`` vs its runtime
set) and ``test_free_chained_parity`` (free ↔ chained args): two representations
of one contract, failing when they diverge. (#303 Phase 1 — the discipline the
content-keys work ships under: a feature is not "typed" until a test proves the
type at runtime.)
"""

from __future__ import annotations

import typing
from pathlib import Path

import laterite as L
import polars as pl
import pytest

# The hand-authored clean fixture the rest of the suite uses — it validates with
# no findings, so the validate → .report → certify chain runs end to end.
_CLEAN = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
    / "clean_minimal.ags"
)
_FRAME = {"PROJ": pl.DataFrame({"PROJ_ID": ["P1"]})}


# --- declared return types hold on real results ---------------------------


def test_read_returns_ags4file():
    assert isinstance(L.read(path=_CLEAN), L.Ags4File)


def test_free_validate_returns_report():
    assert isinstance(L.validate(_CLEAN), L.Report)


def test_build_ags4_returns_buildresult():
    assert isinstance(L.build_ags4(_FRAME), L.BuildResult)


def test_report_property_returns_report():
    assert isinstance(L.read(path=_CLEAN).validate().report, L.Report)


def test_certify_returns_path(tmp_path):
    out = L.read(path=_CLEAN).validate().certify(tmp_path / "c.ags.idx")
    assert isinstance(out, Path)


# --- the chained `-> Self` contract: identity, not just type --------------


def test_chained_validate_returns_self():
    f = L.read(path=_CLEAN)
    assert f.validate() is f, "Ags4File.validate() -> Self must return the SAME handle"


def test_context_manager_enter_returns_self():
    f = L.read(path=_CLEAN)
    with f as g:
        assert g is f, "Ags4File.__enter__() -> Self must return the SAME handle"


# --- every documented Literal value is accepted by its param --------------
# read() is lazy (no frame materialised), so a backend whose optional dep is
# absent still returns an Ags4File — the acceptance check needs no extra installs.


@pytest.mark.parametrize("backend", typing.get_args(L.Backend))
def test_read_accepts_every_backend(backend):
    assert isinstance(L.read(path=_CLEAN, backend=backend), L.Ags4File)


@pytest.mark.parametrize("xn", typing.get_args(L.XnMode))
def test_read_accepts_every_xnmode(xn):
    assert isinstance(L.read(path=_CLEAN, xn=xn), L.Ags4File)


# --- unknown kwargs are rejected (the signature isn't hiding a **kwargs) ---


def test_read_rejects_unknown_kwarg():
    with pytest.raises(TypeError):
        L.read(path=_CLEAN, nope=1)


def test_free_validate_rejects_unknown_kwarg():
    with pytest.raises(TypeError):
        L.validate(_CLEAN, nope=1)


def test_build_ags4_rejects_unknown_kwarg():
    with pytest.raises(TypeError):
        L.build_ags4(_FRAME, nope=1)
