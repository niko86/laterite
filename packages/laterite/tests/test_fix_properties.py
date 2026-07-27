"""Property-based tests for the public fix / diff / build doors.

The Rust `fix_properties.rs` (PR 1) hammers the fix ENGINE's internals
(cell-preservation, the ASCII fold, the bounded fixpoint). This file is the
Python layer's complement: it exercises the *public fluent API* end-to-end over
the real vendored corpus and across encodings — the integration surface Rust
can't reach. It is deliberately lean; it does not re-derive the engine
invariants.

The headline property (`fix output is always valid UTF-8`) is what caught the
`encoding=None` non-UTF-8 leak that this PR's fix closes: the auto-sniffer
resolves an unlabelled non-UTF-8 file to UTF-8, and the no-op path used to return
the invalid bytes verbatim. Example-based tests missed it because they all passed
an explicit encoding or an ASCII fixture.
"""

from __future__ import annotations

from pathlib import Path

import laterite as L
import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st

_ROOT = Path(__file__).resolve().parents[3]  # repo root
_CORPUS = _ROOT / "rust-packages" / "laterite-ags4-forge" / "vendor" / "pyags4-tests"

# Fail LOUD if the corpus path is wrong. Without this, a mistyped directory makes
# every fixture read as "not parseable" and the whole suite degrades to an
# all-skip that stays green in CI too — which is exactly how this file silently
# stopped guarding the "fix output is always UTF-8" contract for a while.
if not _CORPUS.is_dir():
    raise RuntimeError(f"fix-properties corpus missing: {_CORPUS}")

# A curated, deterministic subset: the encoding trio (the non-UTF-8 exercise) plus
# a spread of rule fixtures whose faults the fixer actually touches (CRLF, BOM,
# short rows, numerics, typography, datetimes). Kept ~15 so the sweep stays well
# under a second, not the full 84 (the Rust sweep covers the whole corpus).
_SUBSET = [
    "4.1-rule1-utf8.ags",
    "4.1-rule1-latin1.ags",
    "4.1-rule1-cp1252.ags",
    "4.1-rule2.ags",
    "4.1-rule2b1.ags",
    "4.1-rule3.ags",
    "4.1-rule5.ags",
    "4.1-rule7-1.ags",
    "4.1-rule8-1.ags",
    "4.1-rule8-4.ags",
    "4.1-rule9-1.ags",
    "4.1-rule10-1.ags",
    "4.1-rule11-1.ags",
    "4.1-fyi16-1.ags",
]


def _parseable(path: Path, encoding: str | None) -> bool:
    """A fixture is in-scope only if it parses as AGS4 at all — the corpus holds
    negative fixtures (e.g. the tab-delimited `4.1-rule6_1.ags`) that are not AGS4
    and can't be mechanically repaired into it. Mirrors the Rust sweep's skip."""
    try:
        L.read(path=path, encoding=encoding)
    except Exception:
        return False
    else:
        return True


@pytest.mark.parametrize("name", _SUBSET, ids=lambda n: n.replace("4.1-", ""))
@pytest.mark.parametrize("encoding", [None, "cp1252"], ids=["auto", "cp1252"])
@pytest.mark.parametrize("risky", [False, True], ids=["safe", "risky"])
def test_fix_output_is_utf8_and_reparses(
    name: str, encoding: str | None, risky: bool
) -> None:
    """The contract the docstrings make unconditionally: `fix()` output is ALWAYS
    valid UTF-8 with no BOM, and re-parses. Run across the auto-sniff (`None`) and
    explicit-cp1252 doors — the `None`/non-UTF-8 combination is the one that
    regressed."""
    path = _CORPUS / name
    if not _parseable(path, encoding):
        pytest.skip(f"{name} is not parseable AGS4 under encoding={encoding!r}")
    r = L.fix(path=path, risky=risky, encoding=encoding)
    r.bytes.decode("utf-8")  # raises if not valid UTF-8
    assert not r.bytes.startswith(b"\xef\xbb\xbf"), "output must have no BOM"
    L.read(data=r.bytes)  # the repaired bytes re-parse


@pytest.mark.parametrize("name", _SUBSET, ids=lambda n: n.replace("4.1-", ""))
@pytest.mark.parametrize("risky", [False, True], ids=["safe", "risky"])
def test_fix_is_a_bounded_fixpoint(name: str, risky: bool) -> None:
    """Re-fixing converges: after at most a few passes the bytes stop changing
    (single-pass idempotence is FALSE — nSF decade-crossers and deep duplicate
    headings need a second pass — but convergence within a small bound holds).
    A drifting fixer would loop here."""
    path = _CORPUS / name
    if not _parseable(path, None):
        pytest.skip(f"{name} is not parseable AGS4")
    cur = L.fix(path=path, risky=risky).bytes
    for _ in range(4):
        nxt = L.fix(data=cur, risky=risky).bytes
        if nxt == cur:
            break
        cur = nxt
    else:  # loop exhausted without break → never reached a fixed point
        pytest.fail(f"{name} (risky={risky}) did not converge within 4 passes")


# --- diff algebra (public `diff` door) --------------------------------------

_KEYED_DOC = """"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n\
"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_GL"\r\n"UNIT","","m"\r\n"TYPE","ID","2DP"\r\n{rows}"""


def _doc(rows: list[tuple[str, str]]) -> bytes:
    body = "".join(f'"DATA","{bh}","{gl}"\r\n' for bh, gl in rows)
    return _KEYED_DOC.format(rows=body).encode("utf-8")


# LOCA rows: a KEY id + a 2DP value. Unique ids so rows pair by KEY, not order.
_rows = st.lists(
    st.tuples(
        st.text(alphabet="ABCDEFGH0123456789", min_size=2, max_size=4),
        st.integers(-9999, 9999).map(lambda n: f"{n / 100:.2f}"),
    ),
    min_size=0,
    max_size=6,
    unique_by=lambda t: t[0],
)


@settings(max_examples=60, suppress_health_check=[HealthCheck.too_slow])
@given(rows=_rows)
def test_diff_is_reflexive(rows: list[tuple[str, str]]) -> None:
    """A document differs from itself by nothing — the identity every diff view
    depends on to report `no changes`."""
    d = L.diff(_doc(rows), _doc(rows))
    assert d["total_added"] == d["total_removed"] == d["total_changed"] == 0


@settings(max_examples=60, suppress_health_check=[HealthCheck.too_slow])
@given(a=_rows, b=_rows)
def test_diff_is_antisymmetric(
    a: list[tuple[str, str]], b: list[tuple[str, str]]
) -> None:
    """Swapping baseline and revision swaps additions and removals and leaves the
    change count fixed — rows matched by KEY, so a→b add == b→a remove."""
    ab = L.diff(_doc(a), _doc(b))
    ba = L.diff(_doc(b), _doc(a))
    assert ab["total_added"] == ba["total_removed"]
    assert ab["total_removed"] == ba["total_added"]
    assert ab["total_changed"] == ba["total_changed"]


# --- build 2DP canonicalisation (public `build_ags4` door) ------------------


@settings(max_examples=80, suppress_health_check=[HealthCheck.too_slow])
@given(
    vals=st.lists(
        st.floats(min_value=-1e6, max_value=1e6, allow_nan=False, allow_infinity=False),
        min_size=1,
        max_size=5,
    )
)
def test_build_2dp_is_canonical_and_reparse_clean(vals: list[float]) -> None:
    """A native float under a 2DP heading emits its canonical `%.2f` form with no
    fixing, and the emitted file re-reads clean of Rule 8 (the born-typed
    guarantee, over the float domain rather than a couple of examples)."""
    import polars as pl

    proj = pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["prop"]})
    loca = pl.DataFrame(
        {"LOCA_ID": [f"BH{i}" for i in range(len(vals))], "LOCA_GL": vals}
    )
    res = L.build_ags4({"PROJ": proj, "LOCA": loca})
    assert res.fixes_applied == 0, "native floats are canonical by construction"
    for v in vals:
        assert f'"{v:.2f}"' in res.text
    back = L.read(data=res.bytes).validate(warnings=True)
    assert "AGS Format Rule 8" not in back.report.by_rule()
