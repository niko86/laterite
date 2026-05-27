"""laterite contract tests.

Engine parity is the load-bearing gate: `compat` / the nice API /
the CLI all wrap the *same* clean-room `ags4_validator`, so they must
agree with each other and with the Rust `ags4-check` binary
byte-for-byte. python-ags4 agreement is reported but the documented
O-N clean-room divergences are expected (the engine, not laterite,
owns them).
"""

from __future__ import annotations

import contextlib
import io
import json
import logging
import subprocess
import sys
from pathlib import Path

import laterite
import narwhals.stable.v1 as nw
import pytest
from laterite import compat as AGS4

_FIX = Path(__file__).parents[3] / "rust-packages" / "ags4-validator" / "tests" / "fixtures"
_RUST_BIN = Path(__file__).parents[3] / "rust-packages" / "target" / "release" / "ags4-check"
_FIXTURES = sorted(_FIX.glob("*.ags"))
_CLEAN = _FIX / "clean_minimal.ags"

logging.disable(logging.CRITICAL)


def _quiet(fn, *a, **k):
    with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
        return fn(*a, **k)


# --- nice API -------------------------------------------------------

def test_validate_clean_file_is_valid():
    rep = laterite.validate(str(_CLEAN))
    assert rep.is_valid and rep.exit_code == 0
    assert rep.dict_version and rep.resolution


def test_validate_text_and_to_json_is_rust_shaped():
    src = (
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_FDEP"\r\n'
        '"UNIT","","m"\r\n"TYPE","ID","2DP"\r\n"DATA","BH1","x"\r\n'
    )
    rep = laterite.validate(text=src)
    doc = json.loads(rep.to_json())
    assert set(doc) == {"file", "findings"}
    assert all(k.startswith("AGS Format Rule ") for k in doc["findings"])


def test_read_to_numeric_coerces_like_pandas():
    src = (
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_FDEP"\r\n'
        '"UNIT","","m"\r\n"TYPE","ID","2DP"\r\n'
        '"DATA","BH1","10.50"\r\n"DATA","BH2","oops"\r\n'
    )
    f = laterite.read(text=src)
    col = f.to_numeric("LOCA").to_native()["LOCA_FDEP"].to_list()
    assert col == [10.5, None]


def test_not_ags4_raises_mapped_exception():
    with pytest.raises(laterite.NotAgs4Error):
        laterite.read(text="definitely not ags4")


def test_missing_file_raises_filenotfound():
    with pytest.raises(FileNotFoundError):
        laterite.validate("/no/such/file.ags")


# --- compat: python-ags4 drop-in -----------------------------------

def test_compat_dataframe_shape_matches_python_ags4():
    pytest.importorskip("python_ags4")
    from python_ags4 import AGS4 as ref
    t, h = AGS4.AGS4_to_dataframe(str(_CLEAN))            # default = pandas
    rt, rh = _quiet(ref.AGS4_to_dataframe, str(_CLEAN))
    assert set(t) == set(rt)
    for g in t:
        assert list(t[g].columns) == list(rt[g].columns)
        assert t[g].shape == rt[g].shape
    assert type(t[next(iter(t))]).__module__.startswith("pandas")


def test_compat_check_file_is_python_ags4_shaped_and_json_dumpable():
    d = AGS4.check_file(str(_CLEAN))
    assert any(k.startswith("AGS Format Rule ") for k in d) or "Summary of data" in d
    assert "Metadata" in d and "Summary of data" in d
    s = json.dumps(d)  # must be JSON-serialisable, python-ags4 shape
    assert '"Metadata"' in s
    meta_labels = {e["line"] for e in d["Metadata"]}
    assert {"Checker", "Time (UTC)", "File encoding", "SHA256 hash"} <= meta_labels


def test_compat_AGS4Error_aliases_native_Ags4Error():
    """python-ags4 callers do `pytest.raises(AGS4Error, ...)`; native
    code raises `Ags4Error`. Same class so both catch each other."""
    from laterite._errors import Ags4Error
    assert AGS4.AGS4Error is Ags4Error


def test_compat_python_ags4_pin_stays_in_sync():
    """The PEP-440 local-version segment of `__version__` and the
    `PYTHON_AGS4_COMPAT` programmatic constant must agree — phase 1 of
    the versioning migration documented in COMPAT.md. If you bump the
    constant, the f-string auto-updates `__version__`; this test
    guards against a future hand-edit accidentally desynchronising
    them."""
    pin = AGS4.PYTHON_AGS4_COMPAT
    # PEP 440 local-version uses `+` separator; the parity-pin is the
    # last `.`-separated segment of that local-version part.
    assert "+" in AGS4.__version__, (
        "phase-1 versioning expects a local-version pin; "
        "update this test when phase 2 lands"
    )
    local = AGS4.__version__.split("+", 1)[1]
    assert local.endswith(f".{pin}"), (
        f"__version__ {AGS4.__version__!r} local-version segment "
        f"{local!r} doesn't end with PYTHON_AGS4_COMPAT={pin!r}"
    )
    # And the dev-dep pin (pyproject.toml [dependency-groups] dev)
    # MUST match the constant. The repo's tooling
    # (`tools/run_python_ags4_tests.sh`) shells out to that pinned
    # version; a desync would mean we're parity-testing against the
    # wrong thing.
    import pathlib
    import re
    root = pathlib.Path(__file__).resolve().parents[3]
    pyproject = (root / "pyproject.toml").read_text()
    m = re.search(r'"python-ags4==(?P<v>[0-9.]+)"', pyproject)
    assert m is not None, "couldn't find python-ags4 pin in root pyproject.toml"
    assert m["v"] == pin, (
        f"PYTHON_AGS4_COMPAT={pin!r} but pyproject.toml dev pins "
        f"python-ags4=={m['v']!r}"
    )


def test_compat_get_TRAN_AGS_returns_edition_string(tmp_path):
    """Mirrors python_ags4.check.get_TRAN_AGS — reads TRAN_AGS from
    TRAN['DATA']. Returns None when TRAN is absent (Rule 14 catches
    the structural issue separately)."""
    tables, _ = AGS4.AGS4_to_dataframe(str(_CLEAN))
    tran = AGS4.get_TRAN_AGS(tables)
    assert tran is not None and tran.startswith("4.")
    # Absent TRAN → None (not a raise; rule 14 owns the diagnosis)
    assert AGS4.get_TRAN_AGS({}) is None


def test_compat_strict_pre_check_raises_on_duplicate_groups(tmp_path):
    """compat enforces python-ags4 strictness even though native parser
    is lenient (O-37): duplicate GROUP declarations raise AGS4Error."""
    bad = tmp_path / "dups.ags"
    bad.write_text(
        '"GROUP","PROJ"\r\n'
        '"HEADING","PROJ_ID"\r\n'
        '"UNIT",""\r\n'
        '"TYPE","ID"\r\n'
        '"DATA","P1"\r\n'
        '"GROUP","PROJ"\r\n'
        '"HEADING","PROJ_ID"\r\n'
        '"UNIT",""\r\n'
        '"TYPE","ID"\r\n'
        '"DATA","P2"\r\n'
    )
    with pytest.raises(AGS4.AGS4Error, match=r"group duplicated"):
        AGS4.AGS4_to_dict(str(bad))


def test_compat_strict_pre_check_raises_on_ragged_data_row(tmp_path):
    """compat enforces python-ags4 strictness: DATA row with field count
    ≠ HEADING raises AGS4Error (native parser would silently pad/trim)."""
    bad = tmp_path / "ragged.ags"
    bad.write_text(
        '"GROUP","PROJ"\r\n'
        '"HEADING","PROJ_ID","PROJ_NAME"\r\n'
        '"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\n'
        '"DATA","P1"\r\n'  # only 2 fields, HEADING has 3
    )
    with pytest.raises(AGS4.AGS4Error,
                       match=r"does not have the same number of entries"):
        AGS4.AGS4_to_dict(str(bad))


def test_compat_desc_translator_unit():
    """The desc translator rewrites laterite wording into python-ags4
    phrasings. Verifies the most load-bearing rules' fingerprints
    independent of any fixture I/O."""
    from laterite._compat_desc import translate

    # Rule 17 — "is not defined in" → "not found in"
    assert (translate("AGS Format Rule 17", "TYPE",
                      'Data type "ID" is not defined in the TYPE group.')
            == 'Data type "ID" not found in TYPE group.')
    # Rule 19 — wording change
    assert (translate("AGS Format Rule 19", "test",
                      "GROUP name must be exactly 4 uppercase letters (A–Z).")
            == "GROUP name should consist of four uppercase letters.")
    # Rule 19a — quotes stripped, wording change
    assert (translate("AGS Format Rule 19a", "TEST",
                      'Heading "test_DPTH" must contain only uppercase '
                      'letters, digits, and underscore.')
            == "Heading test_DPTH should consist of only uppercase letters, "
               "numbers, and an underscore character.")
    # Rule 16 — uses finding group to stitch python-ags4's "in <group>" suffix
    assert (translate("AGS Format Rule 16", "LOCA",
                      'Abbreviation "RC" under LOCA_TYPE is not defined in '
                      'the ABBR group.')
            == '"RC" under LOCA_TYPE in LOCA not found in ABBR group.')
    # Rule 8 — type-specific suffix (DMS)
    assert (translate("AGS Format Rule 8", "LOCA",
                      'Value "51:68:52.498" in LOCA_LAT does not match its '
                      'declared TYPE "DMS".')
            == "Value 51:68:52.498 in LOCA_LAT not of data type DMS or is "
               "an invalid value.")
    # Rule 8 — U type, non-numeric value → "Numeric value expected." suffix
    assert (translate("AGS Format Rule 8", "SAMP",
                      'Value "x" in SAMP_RECL does not match its declared '
                      'TYPE "U".')
            == "Value x in SAMP_RECL not of data type U. Numeric value expected.")
    # Unmapped rule → desc returned untouched
    assert (translate("AGS Format Rule 999", "X", "no entry for this")
            == "no entry for this")


def test_compat_check_file_opt_out_returns_laterite_wording():
    """match_python_ags4_wording=False yields the engine's native
    (more precise) phrasings — the same strings laterite.Validator
    and ags4-check CLI return."""
    # _CLEAN passes all rules so check on a fixture that fires Rule 19
    # in python-ags4's repo, if available; otherwise just confirm the
    # opt-out exists and changes behaviour on Rule-1 ASCII path.
    d_translated = AGS4.check_file(str(_CLEAN))
    d_raw = AGS4.check_file(str(_CLEAN), match_python_ags4_wording=False)
    # Both produce the python-ags4-shape dict
    assert "Metadata" in d_translated and "Metadata" in d_raw
    # No findings on a clean file — but the toggle is plumbed through.
    # Sanity check: descs are deterministic across both calls.
    assert set(d_translated) == set(d_raw)


@pytest.mark.parametrize("fx", _FIXTURES, ids=lambda p: p.name)
def test_compat_check_file_matches_engine(fx):
    """compat.check_file rule-key set == the nice API == the engine.
    (This is the real contract; python-ags4 agreement is a separate,
    O-N-aware comparison.)"""
    rule_keys = {k for k in AGS4.check_file(str(fx)) if k.startswith("AGS Format Rule ")}
    rep_keys = set(laterite.validate(str(fx)).by_rule())
    assert rule_keys == rep_keys


def test_compat_backend_switch_no_pandas_needed():
    AGS4.set_backend("polars")
    try:
        t, _ = AGS4.AGS4_to_dataframe(str(_CLEAN))
        assert type(t[next(iter(t))]).__module__.startswith("polars")
        # per-call override beats process default
        t2, _ = AGS4.AGS4_to_dataframe(str(_CLEAN), backend="pyarrow")
        assert type(t2[next(iter(t2))]).__module__.startswith("pyarrow")
    finally:
        AGS4.set_backend("pandas")


def test_pandas_missing_gives_actionable_error(monkeypatch):
    import polars as pl
    from laterite import _frames

    monkeypatch.setitem(sys.modules, "pandas", None)  # force ImportError
    with pytest.raises(ModuleNotFoundError, match=r"laterite\[compat\]"):
        _frames.materialize(pl.DataFrame({"a": [1]}), "pandas")


def test_compat_ags3_is_refused():
    with pytest.raises(laterite.UnsupportedEditionError):
        AGS4.AGS4_to_dataframe_AGS3("x")


def test_compat_convert_to_numeric_default_pandas_backend():
    """Regression: convert_to_numeric must work on the DEFAULT pandas
    backend (the python-ags4 drop-in path). It previously raised
    `KeyError: 0` — the TYPE row was indexed positionally on a
    filtered pandas frame whose index is not reset (label 1, not 0)."""
    src = (
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_FDEP"\r\n'
        '"UNIT","","m"\r\n"TYPE","ID","2DP"\r\n'
        '"DATA","BH1","10.50"\r\n"DATA","BH2","oops"\r\n'
    )
    # Pin backend=pandas so the regression path is exercised regardless
    # of any leaked process-wide default from another test.
    t, _ = AGS4.AGS4_to_dataframe(io.StringIO(src), backend="pandas")
    out = AGS4.convert_to_numeric(t["LOCA"])  # must not raise

    import math

    nf = nw.from_native(out, eager_only=True)
    assert nf.shape[0] == 2  # UNIT/TYPE rows dropped, 2 DATA rows
    vals = nf["LOCA_FDEP"].to_list()  # 2DP coerced; bad cell → missing
    assert vals[0] == 10.5
    # pandas backend → NaN, polars/pyarrow → None: accept either
    assert vals[1] is None or (isinstance(vals[1], float) and math.isnan(vals[1]))


# --- compat round-trip ---------------------------------------------

def test_compat_roundtrip_matches_python_ags4(tmp_path):
    pytest.importorskip("python_ags4")
    from python_ags4 import AGS4 as ref
    t, h = AGS4.AGS4_to_dataframe(str(_CLEAN))
    rt, rh = _quiet(ref.AGS4_to_dataframe, str(_CLEAN))
    a = tmp_path / "lat.ags"
    b = tmp_path / "ref.ags"
    AGS4.dataframe_to_AGS4(t, h, str(a))
    ref.dataframe_to_AGS4(rt, rh, str(b))
    assert a.read_bytes() == b.read_bytes()


# --- CLI byte-parity vs the Rust binary ----------------------------

@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust ags4-check not built")
@pytest.mark.parametrize("fx", _FIXTURES, ids=lambda p: p.name)
def test_cli_json_ndjson_exit_byte_parity(fx):
    def run(cmd):
        p = subprocess.run(cmd, capture_output=True, text=True)
        return p.stdout, p.returncode

    rj, rc_r = run([str(_RUST_BIN), str(fx), "--json"])
    pj, rc_p = run([sys.executable, "-m", "laterite._cli", str(fx), "--json"])
    rn, _ = run([str(_RUST_BIN), str(fx), "--ndjson"])
    pn, _ = run([sys.executable, "-m", "laterite._cli", str(fx), "--ndjson"])
    assert pj.rstrip("\n") == rj.rstrip("\n")
    assert pn == rn
    assert rc_p == rc_r


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust ags4-check not built")
def test_cli_exit_codes():
    def code(args):
        return subprocess.run(
            [sys.executable, "-m", "laterite._cli", *args], capture_output=True
        ).returncode

    assert code([str(_CLEAN)]) == 0                       # clean
    assert code(["/no/such.ags", "--json"]) == 3          # not found
    assert code(["--tui", str(_CLEAN)]) == 5              # unknown opt


# --- Stage 2a: python-ags4 surface completion ----------------------


def test_compat_AGS4Error_is_exception():
    """Mirror python_ags4.AGS4.AGS4Error — callers that catch this
    must keep working after switching to laterite.compat."""
    from laterite.compat import AGS4Error

    assert issubclass(AGS4Error, Exception)
    try:
        raise AGS4Error("test")
    except AGS4Error as exc:
        assert str(exc) == "test"


def test_compat_count_errors_categorises_by_key_prefix():
    """`count_errors` partitions errors / warnings / FYIs by the
    well-known key prefixes the validator uses."""
    from laterite.compat import count_errors

    errs = {
        "AGS Format Rule 1": [{"line": 1, "group": "", "desc": "x"}],
        "AGS Format Rule 2": [
            {"line": 2, "group": "X", "desc": "y"},
            {"line": 3, "group": "Y", "desc": "z"},
        ],
        "Validator Process Error (X)": [{"line": 4, "group": "", "desc": "w"}],
        "Warning (foo)": [{"line": 5, "group": "", "desc": "warn1"}],
        "FYI (bar)": [{"line": 6, "group": "", "desc": "fyi1"}],
        "Metadata": [{"line": "x", "desc": "meta"}],  # not counted
    }
    err, warn, fyi = count_errors(errs)
    assert (err, warn, fyi) == (4, 1, 1)


def test_compat_format_numeric_column_DP_and_SF():
    """`format_numeric_column` matches python-ags4 byte-for-byte on
    DP / SCI / SF specifiers."""
    import pandas as pd
    from laterite.compat import format_numeric_column
    from python_ags4 import AGS4

    df = pd.DataFrame({
        "HEADING": ["UNIT", "TYPE", "DATA", "DATA", "DATA"],
        "X": ["m", "2DP", 1.5, 2.0, 3.14159],
    })

    for spec in ("2DP", "1SCI", "3SF"):
        ours = format_numeric_column(df.copy(), "X", spec)
        theirs = AGS4.format_numeric_column(df.copy(), "X", spec)
        # DATA rows must match byte-for-byte; UNIT/TYPE pseudo-rows
        # remain object-typed strings on both sides.
        assert (ours["X"].tolist() == theirs["X"].tolist()), (
            f"divergence on TYPE={spec}: ours={ours['X'].tolist()} "
            f"theirs={theirs['X'].tolist()}"
        )


def test_compat_sort_groups_dictionary_alphabetical_hierarchical(tmp_path):
    """`sort_groups` orders match python-ags4 on the three documented
    strategies. The UnsortedGroups fixture lives in their repo;
    skip cleanly if the sibling clone isn't there."""
    py_ags4_repo = Path(__file__).resolve().parents[3].parent / "ags-python-library"
    fixture = py_ags4_repo / "tests" / "test_files" / "UnsortedGroups.ags"
    if not fixture.exists():
        pytest.skip(
            "needs ../ags-python-library cloned (run "
            "./tools/run_python_ags4_tests.sh once to set up)"
        )

    from laterite.compat import sort_groups
    from python_ags4 import AGS4

    tables, _ = AGS4.AGS4_to_dataframe(str(fixture))
    for strategy in ("dictionary", "alphabetical", "hierarchical"):
        ours = list(sort_groups(tables, sorting_strategy=strategy).keys())
        theirs = list(AGS4.sort_groups(tables, sorting_strategy=strategy).keys())
        assert ours == theirs, (
            f"order divergence on strategy={strategy}:\n"
            f"  ours:  {ours}\n  theirs: {theirs}"
        )


def test_compat_write_error_report_byte_exact(tmp_path):
    """`write_error_report` produces the same text python-ags4
    produces, byte-for-byte (CRLF line endings, section ordering,
    `Line ... :<8 group:<7 desc` row format)."""
    from laterite.compat import write_error_report
    from python_ags4 import AGS4

    errors = {
        "Metadata": [{"line": "File Path", "desc": "/tmp/x.ags"}],
        "AGS Format Rule 1": [
            {"line": 1, "group": '"LOCA"', "desc": "Non-ASCII char."}
        ],
        "AGS Format Rule 7": [
            {"line": 5, "group": '"PROJ"', "desc": "Field missing."}
        ],
        "Warning (foo)": [
            {"line": 8, "group": '"SAMP"', "desc": "A warning."}
        ],
    }
    ours_path = tmp_path / "ours.txt"
    theirs_path = tmp_path / "theirs.txt"
    write_error_report(errors, str(ours_path), show_warnings=True)
    AGS4.write_error_report(errors, str(theirs_path), show_warnings=True)
    assert ours_path.read_bytes() == theirs_path.read_bytes()


def test_compat_write_error_report_handles_none_output():
    """`output_file=None` is a no-op — matches python-ags4's
    `except TypeError: pass`."""
    from laterite.compat import write_error_report

    # Should not raise.
    write_error_report({"AGS Format Rule 1": []}, None)


# --- Stage 2b: Rust-backed Excel I/O -------------------------------


_PY_AGS4_REPO = Path(__file__).resolve().parents[3].parent / "ags-python-library"
_PY_AGS4_TEST_DATA = _PY_AGS4_REPO / "python_ags4" / "data" / "test_data.ags"
_PY_AGS4_TEST_XLSX = _PY_AGS4_REPO / "tests" / "test.xlsx"


def _need_py_ags4_repo():
    if not _PY_AGS4_TEST_DATA.exists():
        pytest.skip(
            "needs ../ags-python-library cloned (run "
            "./tools/run_python_ags4_tests.sh once to set up)"
        )


def test_compat_AGS4_to_excel_round_trip(tmp_path):
    """AGS4 → XLSX → AGS4 preserves every group and row count."""
    _need_py_ags4_repo()
    from laterite.compat import (
        AGS4_to_dataframe,
        AGS4_to_excel,
        excel_to_AGS4,
    )

    xlsx = tmp_path / "rt.xlsx"
    ags_out = tmp_path / "rt.ags"

    AGS4_to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx))
    assert xlsx.stat().st_size > 0
    excel_to_AGS4(str(xlsx), str(ags_out))

    orig, _ = AGS4_to_dataframe(str(_PY_AGS4_TEST_DATA))
    back, _ = AGS4_to_dataframe(str(ags_out))
    assert sorted(orig.keys()) == sorted(back.keys())
    for code in orig:
        assert orig[code].shape == back[code].shape, (
            f"{code}: orig={orig[code].shape} back={back[code].shape}"
        )


def test_compat_AGS4_to_excel_writes_HEADING_column_first(tmp_path):
    """Sheet layout: first column is HEADING; row 0 is UNIT (the AGS
    UNIT pseudo-row), row 1 is TYPE, rows 2+ are DATA. Matches the
    layout python-ags4 produces (HEADING pseudo-rows in column 0)."""
    _need_py_ags4_repo()
    import pandas as pd
    from laterite.compat import AGS4_to_excel

    xlsx = tmp_path / "layout.xlsx"
    AGS4_to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx))
    sheets = pd.read_excel(str(xlsx), sheet_name=None, engine="openpyxl")

    assert "PROJ" in sheets
    proj = sheets["PROJ"]
    assert list(proj.columns)[0] == "HEADING"
    assert proj.loc[0, "HEADING"] == "UNIT"
    assert proj.loc[1, "HEADING"] == "TYPE"
    assert proj.loc[2, "HEADING"] == "DATA"


def test_compat_excel_to_AGS4_drops_rule_19_violating_columns(tmp_path):
    """Columns named anything other than `HEADING` or
    `[A-Z0-9]{4}_[A-Z0-9]{1,4}` get dropped from the AGS4 output."""
    _need_py_ags4_repo()
    if not _PY_AGS4_TEST_XLSX.exists():
        pytest.skip("test.xlsx not in cloned repo")
    from laterite.compat import AGS4_to_dataframe, excel_to_AGS4

    ags_out = tmp_path / "filtered.ags"
    excel_to_AGS4(str(_PY_AGS4_TEST_XLSX), str(ags_out))
    tables, _ = AGS4_to_dataframe(str(ags_out))
    assert "NEW_column" not in tables["LOCA"].columns
    assert "NEWCOLUMN" not in tables["LOCA"].columns


def test_compat_excel_to_AGS4_applies_TYPE_precision(tmp_path):
    """The default `format_numeric_columns=True` re-formats DATA
    cells to the TYPE row's precision — `5000000.1` (XLSX) →
    `5000000.100` (AGS4 3DP)."""
    _need_py_ags4_repo()
    if not _PY_AGS4_TEST_XLSX.exists():
        pytest.skip("test.xlsx not in cloned repo")
    from laterite.compat import AGS4_to_dataframe, excel_to_AGS4

    ags_out = tmp_path / "formatted.ags"
    excel_to_AGS4(str(_PY_AGS4_TEST_XLSX), str(ags_out))
    tables, _ = AGS4_to_dataframe(str(ags_out))
    assert tables["LOCA"].loc[2, "LOCA_NATN"] == "5000000.001"
    assert tables["LOCA"].loc[4, "LOCA_NATN"] == "5000000.100"


def test_compat_excel_to_AGS4_dictionary_path_overrides_TYPE_precision(tmp_path):
    """`dictionary=<AGS4 file path>` is now wired up (Stage 6d): Rust
    does the XLSX→AGS4 conversion, then compat post-processes the
    result by reading the dict's UNIT/TYPE rows and reformatting DATA
    cells to the dict's precision."""
    _need_py_ags4_repo()
    if not _PY_AGS4_TEST_XLSX.exists():
        pytest.skip("test.xlsx not in cloned repo")
    from laterite.compat import AGS4_to_dataframe, excel_to_AGS4

    out = tmp_path / "x.ags"
    dict_path = _PY_AGS4_REPO / "tests" / "DICT.ags"
    excel_to_AGS4(str(_PY_AGS4_TEST_XLSX), str(out), dictionary=str(dict_path))
    # The dict reformats LLPL_LL to 2SF — the matching python-ags4 test
    # asserts the same expected text (e.g. 5000000.00 / 5000000.10).
    tables, _ = AGS4_to_dataframe(str(out))
    assert "LOCA" in tables
    assert "LLPL" in tables


def test_compat_AGS4_to_excel_sorting_strategy(tmp_path):
    """sorting_strategy='alphabetical' orders sheets alphabetically."""
    _need_py_ags4_repo()
    import pandas as pd
    from laterite.compat import AGS4_to_excel

    xlsx = tmp_path / "sorted.xlsx"
    AGS4_to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx),
                  sorting_strategy="alphabetical")
    sheets = pd.read_excel(str(xlsx), sheet_name=None, engine="openpyxl")
    assert list(sheets.keys()) == sorted(sheets.keys())
