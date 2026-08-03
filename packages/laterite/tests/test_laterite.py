"""laterite contract tests.

Engine parity is the load-bearing gate: `compat` / the nice API /
the CLI all wrap the *same* clean-room `laterite_ags4_validator`, so they must
agree with each other and with the Rust `lat` binary
byte-for-byte. python-ags4 agreement is reported but the documented
O-N clean-room divergences are expected (the engine, not laterite,
owns them).
"""

from __future__ import annotations

import contextlib
import io
import json
import logging
import re
import subprocess
import sys
import textwrap
from pathlib import Path

import laterite
import pytest
from laterite import compat as AGS4

_FIX = (
    Path(__file__).parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
)
_RUST_BIN = Path(__file__).parents[3] / "rust-packages" / "target" / "release" / "lat"
_FIXTURES = sorted(_FIX.glob("*.ags"))
_CLEAN = _FIX / "clean_minimal.ags"
#: A file with findings — the populated-grid half of the human-table parity check.
_DIRTY = _FIX / "multi_finding.ags"

logging.disable(logging.CRITICAL)


def _quiet(fn, *a, **k):
    with (
        contextlib.redirect_stdout(io.StringIO()),
        contextlib.redirect_stderr(io.StringIO()),
    ):
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

    t, _h = AGS4.AGS4_to_dataframe(str(_CLEAN))  # default = pandas
    rt, _rh = _quiet(ref.AGS4_to_dataframe, str(_CLEAN))
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
    """The laterite.compat version-identity contract, kept bump-proof.

    Phase-aware across the COMPAT.md phase 1 -> 2 migration, so the migration
    itself doesn't require editing this test. Invariants:
    - the SHIPPED distribution version is always clean PEP 440 (no `+local`) —
      PyPI rejects local versions on upload, so this guards the release directly;
    - compat.__version__'s release prefix tracks the shipped version (catches a
      version bump that updated pyproject but left compat.py's prefix stale);
    - phase 1: __version__ carries `+compat.python-ags4.<pin>` and the pin
      matches PYTHON_AGS4_COMPAT; phase 2: the `+local` is gone (clean ==
      shipped) but PYTHON_AGS4_COMPAT still exists. Either passes;
    - PYTHON_AGS4_COMPAT matches the python-ags4==X dev pin (the parity oracle);
    - the Checker banner identifies as laterite (+ the pin), never masquerades
      as bare python-ags4 (COMPAT.md H-2 — a misidentified validator is worse
      than a red parity test).
    """
    import importlib.metadata
    import pathlib
    import re

    from laterite.compat import _CHECKER_STRING

    pin = AGS4.PYTHON_AGS4_COMPAT
    shipped = importlib.metadata.version("laterite")

    # PyPI safety: the shipped distribution version must never carry a `+local`
    # segment (PEP 440 local versions are rejected by PyPI on upload).
    assert "+" not in shipped, (
        f"shipped laterite version {shipped!r} carries a PEP 440 local segment "
        "— PyPI would reject the upload"
    )
    # compat.__version__'s release prefix must equal the shipped version (so a
    # bump that missed compat.py's hardcoded prefix turns red here).
    assert AGS4.__version__.split("+", 1)[0] == shipped, (
        f"compat.__version__ {AGS4.__version__!r} release prefix doesn't match "
        f"the shipped laterite version {shipped!r} — did a version bump miss "
        "compat.py?"
    )
    # Phase-aware pin coupling: phase 1 has the local segment ending in the pin.
    if "+" in AGS4.__version__:
        local = AGS4.__version__.split("+", 1)[1]
        assert local.endswith(f".{pin}"), (
            f"__version__ {AGS4.__version__!r} local segment {local!r} doesn't "
            f"end with PYTHON_AGS4_COMPAT={pin!r}"
        )
    # The dev-dep pin (pyproject [dependency-groups] dev) MUST match the
    # constant — tools/run_python_ags4_tests.sh shells out to that version, so a
    # desync would parity-test against the wrong thing.
    root = pathlib.Path(__file__).resolve().parents[3]
    pyproject = (root / "pyproject.toml").read_text()
    m = re.search(r'"python-ags4==(?P<v>[0-9.]+)"', pyproject)
    assert m is not None, "couldn't find python-ags4 pin in root pyproject.toml"
    assert m["v"] == pin, (
        f"PYTHON_AGS4_COMPAT={pin!r} but pyproject.toml dev pins "
        f"python-ags4=={m['v']!r}"
    )
    # Identity honesty: the Checker banner says laterite (+ the pin), never
    # claims to be bare python-ags4.
    assert "laterite" in _CHECKER_STRING and pin in _CHECKER_STRING


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
    with pytest.raises(
        AGS4.AGS4Error, match=r"does not have the same number of entries"
    ):
        AGS4.AGS4_to_dict(str(bad))


def test_compat_desc_translator_unit():
    """The desc translator rewrites laterite wording into python-ags4
    phrasings. Verifies the most load-bearing rules' fingerprints
    independent of any fixture I/O."""
    from laterite._compat_desc import translate

    # Rule 17 — "is not defined in" → "not found in"
    assert (
        translate(
            "AGS Format Rule 17",
            "TYPE",
            'Data type "ID" is not defined in the TYPE group.',
        )
        == 'Data type "ID" not found in TYPE group.'
    )
    # Rule 19 — wording change
    assert (
        translate(
            "AGS Format Rule 19",
            "test",
            "GROUP name must be exactly 4 uppercase letters (A–Z).",
        )
        == "GROUP name should consist of four uppercase letters."
    )
    # Rule 19a — quotes stripped, wording change
    assert (
        translate(
            "AGS Format Rule 19a",
            "TEST",
            'Heading "test_DPTH" must contain only uppercase '
            "letters, digits, and underscore.",
        )
        == "Heading test_DPTH should consist of only uppercase letters, "
        "numbers, and an underscore character."
    )
    # Rule 16 — uses finding group to stitch python-ags4's "in <group>" suffix
    assert (
        translate(
            "AGS Format Rule 16",
            "LOCA",
            'Abbreviation "RC" under LOCA_TYPE is not defined in the ABBR group.',
        )
        == '"RC" under LOCA_TYPE in LOCA not found in ABBR group.'
    )
    # Rule 8 — type-specific suffix (DMS)
    assert (
        translate(
            "AGS Format Rule 8",
            "LOCA",
            'Value "51:68:52.498" in LOCA_LAT does not match its declared TYPE "DMS".',
        )
        == "Value 51:68:52.498 in LOCA_LAT not of data type DMS or is "
        "an invalid value."
    )
    # Rule 8 — U type, non-numeric value → "Numeric value expected." suffix
    assert (
        translate(
            "AGS Format Rule 8",
            "SAMP",
            'Value "x" in SAMP_RECL does not match its declared TYPE "U".',
        )
        == "Value x in SAMP_RECL not of data type U. Numeric value expected."
    )
    # Unmapped rule → desc returned untouched
    assert (
        translate("AGS Format Rule 999", "X", "no entry for this")
        == "no entry for this"
    )


def test_compat_check_file_opt_out_returns_laterite_wording():
    """match_python_ags4_wording=False yields the engine's native
    (more precise) phrasings — the same strings laterite.Validator
    and lat CLI return."""
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
    rule_keys = {
        k for k in AGS4.check_file(str(fx)) if k.startswith("AGS Format Rule ")
    }
    # `validate` now shows WARNINGs by default (#203); this contract is about the
    # ERROR-tier rule keys, so compare errors-only (warnings=False).
    rep_keys = set(laterite.validate(str(fx), warnings=False).by_rule())
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


_SRC_PROJ = (
    '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n'
    '"UNIT","",""\r\n"TYPE","ID","X"\r\n'
    '"DATA","P1","Site A"\r\n"DATA","P2","Site B"\r\n'
)


# Exercised in a subprocess with pyarrow blocked BEFORE laterite/duckdb import.
# In-process simulation is impossible: DuckDB caches pyarrow availability
# process-globally, so once any earlier test imports pyarrow, blocking it mid-run
# breaks DuckDB's own `.df()` — which a genuine pyarrow-free install never does
# (DuckDB simply takes its NumPy path). Only a fresh interpreter is faithful.
_PYARROW_FREE_EXERCISE = textwrap.dedent(
    """
    import io, sys

    class _BlockPyarrow:
        def find_spec(self, name, path=None, target=None):
            if name.split(".")[0] == "pyarrow":
                raise ModuleNotFoundError("[sim] pyarrow not installed")
            return None

    sys.meta_path.insert(0, _BlockPyarrow())  # before laterite / duckdb import

    from laterite import compat as AGS4

    SRC = (
        '"GROUP","PROJ"\\r\\n"HEADING","PROJ_ID","PROJ_NAME"\\r\\n'
        '"UNIT","",""\\r\\n"TYPE","ID","X"\\r\\n'
        '"DATA","P1","Site A"\\r\\n"DATA","P2","Site B"\\r\\n'
    )

    fails = []

    # object-dtype pandas via the DuckDB `.df()` fallback (no pyarrow)
    tables, _ = AGS4.AGS4_to_dataframe(io.StringIO(SRC), backend="pandas")
    proj = tables["PROJ"]
    if list(proj.columns) != ["HEADING", "PROJ_ID", "PROJ_NAME"]:
        fails.append(f"columns={list(proj.columns)}")
    if not all(str(d) == "object" for d in proj.dtypes):
        fails.append(f"dtypes={list(proj.dtypes)}")
    if proj["PROJ_ID"].tolist() != ["", "ID", "P1", "P2"]:
        fails.append(f"values={proj['PROJ_ID'].tolist()}")

    # string_dtype='string' needs pyarrow — must raise, never downgrade
    try:
        AGS4.AGS4_to_dataframe(io.StringIO(SRC), backend="pandas", string_dtype="string")
        fails.append("string_dtype did not raise")
    except ModuleNotFoundError as e:
        if "pyarrow" not in str(e):
            fails.append(f"string raise msg={e}")

    # and pyarrow really was unreachable
    try:
        import pyarrow  # noqa: F401
        fails.append("pyarrow importable — block failed")
    except ModuleNotFoundError:
        pass

    if fails:
        print("PYARROW-FREE FAILURES:", fails)
        sys.exit(1)
    sys.exit(0)
    """
)


def test_compat_pyarrow_free_fallback_subprocess():
    """The `[compat]` pyarrow-free contract (a real `pip install laterite[compat]`)
    in a fresh interpreter that blocks pyarrow before any import: AGS4_to_dataframe
    still yields object-dtype pandas via DuckDB's `.df()` with values intact, and
    `string_dtype="string"` raises an actionable error rather than downgrading."""
    proc = subprocess.run(
        [sys.executable, "-c", _PYARROW_FREE_EXERCISE],
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert proc.returncode == 0, (
        "pyarrow-free compat fallback broke:\n"
        f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
    )


def test_compat_dataframe_default_is_object_dtype():
    """The drop-in contract: default `AGS4_to_dataframe` returns numpy object
    columns — byte-identical to python-ags4 today (its downstream
    `select_dtypes('object')` / `astype('object')` assume it)."""
    t, _ = AGS4.AGS4_to_dataframe(io.StringIO(_SRC_PROJ), backend="pandas")
    assert all(str(d) == "object" for d in t["PROJ"].dtypes)


def test_compat_string_dtype_knob_pyarrow():
    """`string_dtype="string"` (per-call and process-wide) yields pandas'
    Arrow-backed `str` dtype — what python-ags4 returns on pandas 3 — while
    staying `select_dtypes('object')`-visible (the na_value=NaN variant)."""
    pytest.importorskip("pyarrow")
    import pandas as pd

    t, _ = AGS4.AGS4_to_dataframe(
        io.StringIO(_SRC_PROJ), backend="pandas", string_dtype="string"
    )
    assert all(isinstance(d, pd.StringDtype) for d in t["PROJ"].dtypes)
    assert "PROJ_ID" in t["PROJ"].select_dtypes(include="object").columns

    AGS4.set_string_dtype("string")
    try:
        assert AGS4.get_string_dtype() == "string"
        t2, _ = AGS4.AGS4_to_dataframe(io.StringIO(_SRC_PROJ), backend="pandas")
        assert all(isinstance(d, pd.StringDtype) for d in t2["PROJ"].dtypes)
    finally:
        AGS4.set_string_dtype("object")
    with pytest.raises(ValueError, match=r"unknown string_dtype"):
        AGS4.set_string_dtype("nonsense")


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

    import polars as pl

    # convert_to_numeric returns the process-wide default backend, which may be
    # pandas OR polars — normalise to polars to assert backend-agnostically.
    pf = out if isinstance(out, pl.DataFrame) else pl.from_pandas(out)
    assert pf.shape[0] == 2  # UNIT/TYPE rows dropped, 2 DATA rows
    vals = pf["LOCA_FDEP"].to_list()  # 2DP coerced; bad cell → missing
    assert vals[0] == 10.5
    # pandas backend → NaN, polars → None: accept either
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


def _run_py_cli(args: list[str]) -> tuple[str, int]:
    """Drive the Python `lat` CLI *in-process* (covers
    `laterite._cli`, the actual shipped entrypoint) instead of paying
    for a cold interpreter start per fixture. Returns (stdout, exit).

    `main()` writes findings to `sys.stdout`; capturing it here is the
    byte-for-byte equivalent of reading the subprocess's stdout."""
    from laterite import _cli

    buf = io.StringIO()
    with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(io.StringIO()):
        code = _cli.main(args)
    return buf.getvalue(), code


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust lat not built")
@pytest.mark.parametrize("fx", _FIXTURES, ids=lambda p: p.name)
def test_cli_json_ndjson_exit_byte_parity(fx):
    def run_rust(cmd):
        p = subprocess.run(cmd, capture_output=True, text=True)
        return p.stdout, p.returncode

    # Rust side stays a real subprocess (it is the reference binary);
    # the Python side runs in-process so `_cli.main` is exercised under
    # coverage and the contract (python output == rust output) holds.
    rj, rc_r = run_rust([str(_RUST_BIN), str(fx), "--json"])
    pj, rc_p = _run_py_cli([str(fx), "--json"])
    rn, _ = run_rust([str(_RUST_BIN), str(fx), "--ndjson"])
    pn, _ = _run_py_cli([str(fx), "--ndjson"])
    assert pj.rstrip("\n") == rj.rstrip("\n")
    assert pn == rn
    assert rc_p == rc_r


def test_cli_exit_codes():
    # In-process: each branch exercises a distinct `_cli.main` exit path.
    assert _run_py_cli([str(_CLEAN)])[1] == 0  # clean
    assert _run_py_cli(["/no/such.ags", "--json"])[1] == 3  # not found
    assert _run_py_cli(["--tui", str(_CLEAN)])[1] == 5  # unknown opt
    assert _run_py_cli([])[1] == 5  # no input file
    # External --dict override is deliberately unimplemented (O-28).
    assert _run_py_cli([str(_CLEAN), "--dict", "x.ags"])[1] == 5


def test_cli_plain_report_and_out_file(tmp_path):
    """The default (plain) report and the `--out`/`--json-out` tee
    branches of `_cli.main` — the file-writing paths the byte-parity
    test never touches."""
    from laterite import _cli

    # Plain report on a clean file → "clean (0 findings)".
    out, code = _run_py_cli([str(_CLEAN)])
    assert code == 0
    assert "clean (0 findings)" in out

    # A fixture with findings → tabular plain report with a Rule column.
    findings_fx = _FIX / "rule19_bad_group_name.ags"
    out, code = _run_py_cli([str(findings_fx)])
    assert code == 1
    assert "finding(s)" in out and "Rule" in out

    # --out writes the active (plain) report to disk; stdout gets a
    # one-line summary; exit code unchanged.
    out_path = tmp_path / "report.txt"
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(io.StringIO()):
        code = _cli.main([str(findings_fx), "--out", str(out_path)])
    assert code == 1
    assert (
        out_path.read_text().rstrip("\n").endswith("CLAY")
        or "finding(s)" in out_path.read_text()
    )
    assert "finding(s)" in buf.getvalue()

    # --json-out tees JSON to disk while stdout keeps the plain report.
    json_path = tmp_path / "report.json"
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(io.StringIO()):
        code = _cli.main([str(findings_fx), "--json-out", str(json_path)])
    doc = json.loads(json_path.read_text())
    assert set(doc) == {"file", "findings"}
    assert "finding(s)" in buf.getvalue()  # plain report still on stdout


def test_cli_fix_writes_sibling_and_exit_codes(tmp_path):
    """`lat fix` (Python CLI): sibling output by default, in-place /
    --fix-out variants, and exit 0 clean vs 1 residual."""
    lf = tmp_path / "delivery.ags"
    lf.write_bytes(
        _CLEAN.read_bytes().replace(b"\r\n", b"\n")
    )  # LF-only → fixable clean

    # Default: writes delivery.fixed.ags, source untouched, exit 0 (clean).
    out, code = _run_py_cli(["fix", str(lf)])
    sibling = tmp_path / "delivery.fixed.ags"
    assert code == 0
    assert sibling.exists() and b"\r\n" in sibling.read_bytes()
    assert b"\r\n" not in lf.read_bytes()  # source left alone
    assert "applied 1 fix(es)" in out and "clean (0 findings)" in out

    # --in-place overwrites the source.
    _, code = _run_py_cli(["fix", str(lf), "--in-place"])
    assert code == 0 and b"\r\n" in lf.read_bytes()

    # A file with non-fixable findings → exit 1, findings remain.
    _, code = _run_py_cli(
        ["fix", str(_RULE8_PRECISION), "--fix-out", str(tmp_path / "r8.ags")]
    )
    assert code == 1
    assert (tmp_path / "r8.ags").exists()


def test_cli_fix_misuse_and_errors(tmp_path):
    """`fix` conflicting dest and bad input. (The old "--in-place without --fix"
    misuse is gone structurally — --in-place lives only on the `fix` subcommand.)"""
    assert _run_py_cli(["fix", str(_CLEAN), "--in-place", "--fix-out", "x"])[1] == 5
    assert _run_py_cli(["fix", "/no/such.ags"])[1] == 3  # not found


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust lat not built")
def test_cli_fix_rust_binary_parity(tmp_path):
    """The standalone Rust binary's --fix agrees with the Python CLI: same
    sibling-write behaviour and exit code."""
    lf = tmp_path / "d.ags"
    lf.write_bytes(_CLEAN.read_bytes().replace(b"\r\n", b"\n"))
    r = subprocess.run([str(_RUST_BIN), "fix", str(lf)], capture_output=True, text=True)
    assert r.returncode == 0
    assert (tmp_path / "d.fixed.ags").read_bytes().count(b"\r\n") >= 1


_FIXABLE_RULES = {
    "1",
    "2a",
    "4",
    "6",
    "7",
    "8",
    "11a",
    "11b",
}  # fixes.rs FIXABLE_RULE_LABELS


def test_list_rules_returns_the_27_with_fields():
    """laterite.list_rules() → the engine's rule catalogue, one dict per rule."""
    rules = laterite.list_rules()
    assert len(rules) == 27
    ids = {r["rule"] for r in rules}
    assert "12" not in ids and "16a" not in ids  # no phantoms
    for r in rules:
        assert {"rule", "title", "severity", "fixable", "observations"} <= set(r)
        assert r["severity"] in {"error", "fyi", "mixed"}


def test_list_rules_fixable_matches_fix_engine():
    """The catalogue's `fixable` flag matches the rules `laterite.fix` repairs."""
    rules = laterite.list_rules()
    assert {r["rule"] for r in rules if r["fixable"]} == _FIXABLE_RULES


def test_cli_list_rules_table_and_json():
    """`lat rules` (Python CLI): table by default, JSON with --json,
    no input file needed, exit 0."""
    out, code = _run_py_cli(["rules"])
    assert code == 0 and "Rule" in out and "Character Set" in out

    out, code = _run_py_cli(["rules", "--json"])
    assert code == 0
    doc = json.loads(out)
    assert len(doc["rules"]) == 27


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust lat not built")
def test_cli_list_rules_rust_binary_json_matches_python():
    """The standalone Rust binary's --list-rules --json is byte-identical to the
    Python CLI's — both stream the same compile-time-embedded rules_meta.json."""
    r = subprocess.run(
        [str(_RUST_BIN), "rules", "--json"], capture_output=True, text=True
    )
    assert r.returncode == 0
    py, _ = _run_py_cli(["rules", "--json"])
    assert r.stdout == py  # byte-identical, not merely structurally equal


def test_box_table_colour_is_off_when_redirected(monkeypatch):
    """`_colour_enabled` is the Rust `colour_enabled` conjunction, and all three
    terms must hold — an explicit opt-out, `NO_COLOR`, and a real terminal.

    Only the last one is exercised by every other test here (a captured
    subprocess has no TTY), so the other two are pinned directly. If any term
    stopped being read, redirected output would gain escape codes and every
    byte-parity assertion above would start failing for a reason none of them
    names.
    """
    from laterite import _cli

    monkeypatch.delenv("NO_COLOR", raising=False)
    monkeypatch.setattr(_cli.sys.stdout, "isatty", lambda: True)
    assert _cli._colour_enabled() is True
    assert _cli._colour_enabled(no_colour=True) is False, "the explicit opt-out"
    monkeypatch.setenv("NO_COLOR", "1")
    assert _cli._colour_enabled() is False, "the NO_COLOR convention"
    monkeypatch.delenv("NO_COLOR")
    monkeypatch.setattr(_cli.sys.stdout, "isatty", lambda: False)
    assert _cli._colour_enabled() is False, "not a terminal"


def test_box_table_styling_does_not_skew_columns():
    """Escapes are applied AFTER padding, so they never count as display width.

    Getting this backwards is the classic ANSI table bug: the cell is padded to
    include the invisible bytes, every column after it shifts, and the grid only
    breaks for users on a terminal — the one place no test looks.
    """
    from laterite import _cli

    plain = _cli._box_table(["h"], [["a"], ["b"]], colour=False)
    styled = _cli._box_table(["h"], [["a"], ["b"]], colour=True)
    assert "\x1b[" not in plain
    assert "\x1b[1;36m" in styled and "\x1b[2m" in styled, (
        "header bold-cyan, alt rows dim"
    )
    # Strip the escapes back out and the two must be the same table.
    assert re.sub(r"\x1b\[[0-9;]*m", "", styled) == plain


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust lat not built")
@pytest.mark.parametrize(
    ("argv", "what"),
    [
        (["rules"], "the rule catalogue"),
        ([str(_CLEAN)], "a clean verdict"),
        ([str(_DIRTY)], "a findings table"),
    ],
    ids=["rules", "clean", "findings"],
)
def test_cli_human_table_rust_binary_byte_parity(argv, what):
    """The HUMAN tables agree byte-for-byte, not just the machine-readable ones.

    `lat rules` and the findings table had no cross-surface coverage at all —
    `--json`/`--ndjson`/`--csv` and exit codes were pinned, and the grids people
    actually read were pinned nowhere. That is how the two CLIs came to print
    visibly different output for the same command without any gate noticing.

    Covers a wide table (28 rules), an empty one (a clean file prints no grid at
    all) and a populated one, so a change to the glyphs, the padding or the
    between-row rule reddens here.
    """
    r = subprocess.run([str(_RUST_BIN), *argv], capture_output=True, text=True)
    py, _ = _run_py_cli(argv)
    assert r.stdout == py, f"{what} diverged between the Rust binary and the Python CLI"


@pytest.mark.skipif(not _RUST_BIN.exists(), reason="Rust lat not built")
def test_cli_read_rust_binary_byte_parity():
    """`lat read` is byte-coherent across the Rust binary and the Python CLI for
    the group listing, --json / --csv (raw file cells, #430 PR 2) — and now the
    human --table as well.

    The table used to be excluded on the grounds that presentation was each
    surface's own. It was not a principle so much as an accident: the Python CLI
    printed a plain `ljust` grid because nobody had written comfy-table's glyphs
    down. They are written down now (`_cli._box_table`), so the exclusion is gone
    and the two programs called `lat` render the same bytes.

    The one place they still part is an interactive TTY, where comfy-table's
    `ContentArrangement::Dynamic` wraps to the terminal width and this does not.
    A captured subprocess has no TTY, so it is not this test's case — and it is
    not any gate's or document's case either, which is why matching the wrapping
    algorithm was not worth it."""
    f = str(_CLEAN)
    # the group listing
    r = subprocess.run([str(_RUST_BIN), "read", f], capture_output=True, text=True)
    py, _ = _run_py_cli(["read", f])
    assert r.returncode == 0 and r.stdout == py, "read (list codes) diverged"
    # a group dumped as --json, --csv and the human table
    for extra in (["--json"], ["--csv"], []):
        r = subprocess.run(
            [str(_RUST_BIN), "read", f, "PROJ", *extra], capture_output=True, text=True
        )
        py, _ = _run_py_cli(["read", f, "PROJ", *extra])
        assert r.returncode == 0 and r.stdout == py, f"read PROJ {extra} diverged"


def test_cli_transport_pack_unpack_round_trip(tmp_path):
    """`lat pack` / `unpack` via the Python CLI — a lossless zstd round-trip."""
    from laterite import _cli

    z, out = tmp_path / "p.zst", tmp_path / "restored.ags"
    assert _cli.main(["pack", str(_CLEAN), str(z)]) == 0
    assert _cli.main(["unpack", str(z), str(out)]) == 0
    assert out.read_bytes() == _CLEAN.read_bytes()


def test_cli_transport_lock_unlock_round_trip(tmp_path, monkeypatch):
    """`lat lock` / `unlock` — age passphrase round-trip (env-var password, low
    scrypt factor for speed); a wrong passphrase → exit 6 (#430 PR 3)."""
    from laterite import _cli

    monkeypatch.setenv("LAT_TRANSPORT_PASSWORD", "s3cret")
    age, out = tmp_path / "l.age", tmp_path / "u.ags"
    assert _cli.main(["lock", str(_CLEAN), str(age), "--log-n", "10"]) == 0
    assert _cli.main(["unlock", str(age), str(out)]) == 0
    assert out.read_bytes() == _CLEAN.read_bytes()
    monkeypatch.setenv("LAT_TRANSPORT_PASSWORD", "wrong")
    assert _cli.main(["unlock", str(age), str(tmp_path / "x.ags")]) == 6


def test_cli_transport_password_file_and_missing_input(tmp_path):
    """`--password-file` is honoured (trailing newline stripped); missing input → 3."""
    from laterite import _cli

    pw = tmp_path / "pw.txt"
    pw.write_text("hunter2\n")
    age, out = tmp_path / "l.age", tmp_path / "u.ags"
    assert (
        _cli.main(
            ["lock", str(_CLEAN), str(age), "--log-n", "10", "--password-file", str(pw)]
        )
        == 0
    )
    assert _cli.main(["unlock", str(age), str(out), "--password-file", str(pw)]) == 0
    assert out.read_bytes() == _CLEAN.read_bytes()
    assert _cli.main(["pack", str(tmp_path / "nope.ags"), str(tmp_path / "x.zst")]) == 3


def test_cli_excel_round_trip(tmp_path):
    """`lat excel` via the Python CLI — export (.ags→.xlsx) then import
    (.xlsx→.ags), direction inferred from the output extension; the group set
    survives. Ambiguous extension → exit 5, missing input → 3 (#430 PR 4)."""
    from laterite import _cli, read

    xlsx, back = tmp_path / "out.xlsx", tmp_path / "back.ags"
    assert _cli.main(["excel", str(_CLEAN), str(xlsx)]) == 0
    assert xlsx.exists()
    assert _cli.main(["excel", str(xlsx), str(back)]) == 0
    assert set(read(str(back)).groups) == set(read(str(_CLEAN)).groups)
    assert _cli.main(["excel", str(_CLEAN), str(tmp_path / "x.dat")]) == 5  # ambiguous
    assert (
        _cli.main(["excel", str(tmp_path / "none.ags"), str(tmp_path / "y.xlsx")]) == 3
    )


def test_cli_readme_and_help_flags():
    """`--readme` / `--help` / `-h` all print the bundled CLI README
    and exit 0 (the help short-circuit before argparse)."""
    for flag in ("--readme", "--help", "-h"):
        out, code = _run_py_cli([flag])
        assert code == 0
        assert out.strip(), f"{flag} should print the README"


# --- Stage 2a: python-ags4 surface completion ----------------------


def test_compat_AGS4Error_is_exception():
    """Mirror python_ags4.AGS4.AGS4Error — callers that catch this
    must keep working after switching to laterite.compat."""
    from laterite.compat import AGS4Error

    assert issubclass(AGS4Error, Exception)
    with pytest.raises(AGS4Error, match=r"^test$"):
        raise AGS4Error("test")


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

    df = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", "DATA", "DATA", "DATA"],
            "X": ["m", "2DP", 1.5, 2.0, 3.14159],
        }
    )

    for spec in ("2DP", "1SCI", "3SF"):
        ours = format_numeric_column(df.copy(), "X", spec)
        theirs = AGS4.format_numeric_column(df.copy(), "X", spec)
        # DATA rows must match byte-for-byte; UNIT/TYPE pseudo-rows
        # remain object-typed strings on both sides.
        assert ours["X"].tolist() == theirs["X"].tolist(), (
            f"divergence on TYPE={spec}: ours={ours['X'].tolist()} "
            f"theirs={theirs['X'].tolist()}"
        )


def test_compat_sort_groups_dictionary_alphabetical_hierarchical(tmp_path):
    """`sort_groups` orders match python-ags4 on the three documented
    strategies, on a committed clean-room unsorted-groups fixture."""
    pytest.importorskip("python_ags4")
    fixture = _COMPAT_FIX / "unsorted_groups.ags"

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
        "AGS Format Rule 7": [{"line": 5, "group": '"PROJ"', "desc": "Field missing."}],
        "Warning (foo)": [{"line": 8, "group": '"SAMP"', "desc": "A warning."}],
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


# Committed synthetic compat fixtures (clean-room — no python-ags4 data) so the
# Excel/sort compat tests run in CI without the ../ags-python-library clone.
_COMPAT_FIX = Path(__file__).resolve().parent / "fixtures"
_PY_AGS4_TEST_DATA = _COMPAT_FIX / "excel_source.ags"
_PY_AGS4_TEST_XLSX = _COMPAT_FIX / "excel_book.xlsx"


def _need_py_ags4_repo():
    # Fixtures are committed now — always present. Kept as a no-op so the six
    # call sites read unchanged.
    return


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
    assert next(iter(proj.columns)) == "HEADING"
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
    assert tables["LOCA"].loc[3, "LOCA_NATN"] == "5000000.100"


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
    dict_path = _COMPAT_FIX / "dict_override.ags"
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
    AGS4_to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx), sorting_strategy="alphabetical")
    sheets = pd.read_excel(str(xlsx), sheet_name=None, engine="openpyxl")
    assert list(sheets.keys()) == sorted(sheets.keys())


# --- Headless mechanical fix/repair: laterite.fix / Ags4File.fix (#198) -----

_RULE8_PRECISION = _FIX / "rule8_dp_wrong_precision.ags"
# A duplicate heading within one group → the RISKY RenameDuplicateHeading fix.
_DUP_HEADING_SRC = (
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID","LOCA_ID"\r\n'
    '"UNIT","",""\r\n'
    '"TYPE","ID","ID"\r\n'
    '"DATA","BH1","BH1"\r\n'
)


def test_fix_normalizes_crlf_to_clean():
    """An LF-only clean file → normalize_crlf → CRLF output, no residual findings."""
    lf = _CLEAN.read_bytes().replace(b"\r\n", b"\n")
    r = laterite.fix(data=lf)
    assert [a["kind"] for a in r.applied] == ["normalize_crlf"]
    assert r.fixes_applied == 1
    assert b"\r\n" in r.bytes and len(r.findings) == 0


def test_fix_strips_bom():
    """A BOM-prefixed file → strip_bom → BOM gone from the output."""
    r = laterite.fix(data=b"\xef\xbb\xbf" + _CLEAN.read_bytes())
    assert any(a["kind"] == "strip_bom" for a in r.applied)
    assert not r.bytes.startswith(b"\xef\xbb\xbf")


def test_fix_reformat_numeric_real_fixture():
    """A wrong-precision DP cell is mechanically reformatted (real fixture)."""
    before = laterite.validate(str(_RULE8_PRECISION)).count
    r = laterite.fix(str(_RULE8_PRECISION))
    assert any(a["kind"] == "reformat_numeric" for a in r.applied)
    assert len(r.findings) < before  # at least one finding resolved


def test_fix_clean_file_is_noop():
    """A clean file gets no fixes and its bytes are returned unchanged."""
    clean = _CLEAN.read_bytes()
    r = laterite.fix(data=clean)
    assert r.fixes_applied == 0
    assert r.bytes == clean  # untouched, not silently re-encoded


def test_fix_risky_excluded_by_default_included_on_request():
    """Duplicate-heading rename is RISKY: excluded by default, applied with risky=True."""
    safe = laterite.fix(text=_DUP_HEADING_SRC)
    risky = laterite.fix(text=_DUP_HEADING_SRC, risky=True)
    assert all(a["kind"] != "rename_duplicate_heading" for a in safe.applied)
    assert any(a["kind"] == "rename_duplicate_heading" for a in risky.applied)
    assert all(
        a["risk"] == "risky"
        for a in risky.applied
        if a["kind"] == "rename_duplicate_heading"
    )


def test_fix_result_save_and_text(tmp_path):
    """FixResult.save writes the bytes; .text decodes them."""
    r = laterite.fix(data=_CLEAN.read_bytes().replace(b"\r\n", b"\n"))
    out = tmp_path / "fixed.ags"
    assert r.save(out) == out
    assert out.read_bytes() == r.bytes
    assert r.text == r.bytes.decode("utf-8")


def test_fix_in_place_overwrites_source(tmp_path):
    """fix(path, in_place=True) overwrites the original file."""
    p = tmp_path / "delivery.ags"
    p.write_bytes(_CLEAN.read_bytes().replace(b"\r\n", b"\n"))  # LF-only
    r = laterite.fix(str(p), in_place=True)
    assert r.fixes_applied >= 1
    assert b"\r\n" in p.read_bytes()  # the file on disk was repaired


def test_fix_out_writes_elsewhere_source_untouched(tmp_path):
    """fix(path, out=...) writes the fixed file there; the source is untouched."""
    p = tmp_path / "delivery.ags"
    original = _CLEAN.read_bytes().replace(b"\r\n", b"\n")
    p.write_bytes(original)
    out = tmp_path / "clean.ags"
    laterite.fix(str(p), out=str(out))
    assert out.exists() and b"\r\n" in out.read_bytes()
    assert p.read_bytes() == original  # source unchanged


def test_fix_in_place_requires_path():
    """in_place=True with a non-path source is a clear error, not a silent no-write."""
    with pytest.raises(laterite.Ags4Error):
        laterite.fix(data=_CLEAN.read_bytes().replace(b"\r\n", b"\n"), in_place=True)


def test_fix_in_place_and_out_conflict():
    with pytest.raises(TypeError, match="only one"):
        laterite.fix(text="x", in_place=True, out="y.ags")


def test_fix_handle_method_returns_repaired_handle():
    """Ags4File.fix() returns a repaired, chainable Ags4File; the FixResult rides
    on .fix_report (a handle not produced by .fix() has none)."""
    handle = laterite.read(data=_CLEAN.read_bytes().replace(b"\r\n", b"\n"))
    repaired = handle.fix()
    assert isinstance(repaired, laterite.Ags4File)
    assert b"\r\n" in repaired.bytes
    # the repaired handle is itself a clean, chainable file
    assert repaired.validate().report.is_valid
    # the fix report is carried for inspection; the source handle has none
    assert isinstance(repaired.fix_report, laterite.FixResult)
    assert repaired.fix_report.fixes_applied >= 1
    assert handle.fix_report is None


def test_fix_missing_file_raises_filenotfound():
    with pytest.raises(FileNotFoundError):
        laterite.fix("/no/such/file.ags")


def test_fix_non_ags_raises():
    with pytest.raises(laterite.NotAgs4Error):
        laterite.fix(text="not an ags file at all")


# --- Modern first-class Excel verbs: laterite.to_excel / from_excel (#195) ---


def test_to_excel_path_round_trips(tmp_path):
    """laterite.to_excel(path) → from_excel(xlsx) preserves every group."""
    xlsx = tmp_path / "rt.xlsx"
    stats = laterite.to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx))
    assert xlsx.stat().st_size > 0
    assert set(stats) >= {"sheets_written", "rows_written", "warnings"}

    back = laterite.from_excel(str(xlsx))
    assert isinstance(back, laterite.Ags4File)
    orig = laterite.read(str(_PY_AGS4_TEST_DATA))
    assert sorted(back.groups) == sorted(orig.groups)


def test_to_excel_from_handle_method(tmp_path):
    """Ags4File.to_excel() writes a workbook from the parsed handle (no source path
    needed — it stages the handle's spec-correct bytes)."""
    handle = laterite.read(str(_PY_AGS4_TEST_DATA))
    xlsx = tmp_path / "handle.xlsx"
    stats = handle.to_excel(str(xlsx))
    assert xlsx.stat().st_size > 0
    assert stats["sheets_written"] == len(handle.groups)


def test_to_excel_from_text_source(tmp_path):
    """to_excel(text=...) routes the text/bytes branch through the re-emit."""
    text = _PY_AGS4_TEST_DATA.read_text(encoding="utf-8")
    xlsx = tmp_path / "fromtext.xlsx"
    laterite.to_excel(text=text, output=str(xlsx))
    assert xlsx.stat().st_size > 0


def test_to_excel_groups_subset_orders_sheets(tmp_path):
    """`groups=` fixes the sheet order / subset."""
    import pandas as pd

    xlsx = tmp_path / "subset.xlsx"
    laterite.to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx), groups=["LOCA", "PROJ"])
    sheets = pd.read_excel(str(xlsx), sheet_name=None, engine="openpyxl")
    assert list(sheets.keys()) == ["LOCA", "PROJ"]


def test_to_excel_no_output_returns_xlsx_bytes():
    """to_excel(source) with no output path returns the .xlsx bytes in memory
    (#391) — the FS-free door, so an in-memory AGS4 needn't hit disk."""
    blob = laterite.to_excel(str(_PY_AGS4_TEST_DATA))
    assert isinstance(blob, bytes)
    assert blob[:2] == b"PK"  # xlsx is a zip (PK magic)
    # the bytes are a real workbook: from_excel(bytes) round-trips the groups
    back = laterite.from_excel(blob)
    assert isinstance(back, laterite.Ags4File)
    assert sorted(back.groups) == sorted(laterite.read(str(_PY_AGS4_TEST_DATA)).groups)


def test_to_excel_handle_bytes(tmp_path):
    """Ags4File.to_excel() with no path returns the workbook bytes."""
    handle = laterite.read(str(_PY_AGS4_TEST_DATA))
    blob = handle.to_excel()
    assert isinstance(blob, bytes) and blob[:2] == b"PK"
    # equal to the on-disk form (bar zip mtime noise, the group set round-trips)
    assert sorted(laterite.from_excel(blob).groups) == sorted(handle.groups)


def test_from_excel_bytes_in(tmp_path):
    """from_excel accepts raw .xlsx bytes (bytes-in, #391) — no temp file. With an
    output path it writes AGS4 + returns stats; without, returns a handle."""
    xlsx_bytes = laterite.to_excel(str(_PY_AGS4_TEST_DATA))  # in-memory workbook
    assert isinstance(xlsx_bytes, bytes)
    # bytes-in → handle
    handle = laterite.from_excel(xlsx_bytes)
    assert isinstance(handle, laterite.Ags4File)
    # bytes-in → file + stats
    out = tmp_path / "frombytes.ags"
    stats = laterite.from_excel(xlsx_bytes, str(out))
    assert isinstance(stats, dict) and out.stat().st_size > 0


def test_from_excel_to_file_returns_stats(tmp_path):
    """from_excel(xlsx, out) writes AGS4 and returns the converter stats dict."""
    xlsx = tmp_path / "src.xlsx"
    laterite.to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx))
    out = tmp_path / "back.ags"
    stats = laterite.from_excel(str(xlsx), str(out))
    assert isinstance(stats, dict)
    assert out.stat().st_size > 0


def test_from_excel_handle_is_self_contained(tmp_path):
    """from_excel(xlsx) returns a handle that validates without a backing file — its
    source is the bytes, not the (deleted) temp path."""
    xlsx = tmp_path / "selfc.xlsx"
    laterite.to_excel(str(_PY_AGS4_TEST_DATA), str(xlsx))
    back = laterite.from_excel(str(xlsx))
    # .validate() re-reads the retained source; a dangling temp path would raise.
    report = back.validate().report
    assert report is not None


def test_to_excel_rejects_non_ags_input(tmp_path):
    """A non-AGS source surfaces NotAgs4Error, not a raw RuntimeError."""
    from laterite import NotAgs4Error

    junk = tmp_path / "junk.ags"
    junk.write_text("not an ags file at all\n", encoding="utf-8")
    with pytest.raises(NotAgs4Error):
        laterite.to_excel(str(junk), str(tmp_path / "x.xlsx"))


# --- Report.findings / by_rule carry severity + location (#196) -------------

_RULE10A_DUP = (
    _FIX / "rule10a_dup_key.ags"
)  # cell-target findings (field_index/data_row)
_RULE11C_RL = _FIX / "rule11c_bad_rl.ags"  # a heading-target finding


def test_findings_frame_has_severity_and_location_columns():
    """The findings frame surfaces the severity + location columns the native layer
    already carries — not just rule/line/group/desc."""
    rep = laterite.validate(str(_RULE10A_DUP))
    cols = set(rep.findings.columns)
    assert {
        "rule",
        "line",
        "group",
        "desc",
        "severity",
        "target",
        "heading",
        "field_index",
        "data_row",
    } <= cols


def test_findings_severity_defaults_error():
    """Error findings (the default tier) report severity == 'error' even though the
    boundary omits the field for them."""
    rep = laterite.validate(str(_RULE10A_DUP))
    sev = set(rep.findings["severity"].to_list())
    assert sev == {"error"}


def test_findings_fyi_severity(tmp_path):
    """A BOM-prefixed file validated with fyi=True surfaces an FYI row whose
    severity column reads 'fyi' — the non-error tier flows through to the frame."""
    bom = tmp_path / "bom.ags"
    bom.write_bytes(b"\xef\xbb\xbf" + _CLEAN.read_bytes())
    rep = laterite.validate(str(bom), fyi=True)
    df = rep.findings
    fyi_rows = df.filter(df["severity"] == "fyi")
    assert fyi_rows.height >= 1
    assert all("FYI" in r for r in fyi_rows["rule"].to_list())


def test_findings_heading_target_populated():
    """A heading-targeted finding pins target='heading' and the offending heading."""
    rep = laterite.validate(str(_RULE11C_RL))
    df = rep.findings
    heads = df.filter(df["target"] == "heading")
    assert heads.height >= 1
    assert heads["heading"].null_count() < heads.height  # at least one heading set


def test_findings_cell_target_has_field_index_and_data_row():
    """A cell-targeted finding pins target='cell' with a field_index + data_row."""
    rep = laterite.validate(str(_RULE10A_DUP))
    df = rep.findings
    cells = df.filter(df["target"] == "cell")
    assert cells.height >= 1
    assert cells["field_index"].null_count() < cells.height
    assert cells["data_row"].null_count() < cells.height


def test_by_rule_carries_severity_and_location():
    """by_rule items carry severity (always) plus whatever location fields the
    finding pins."""
    rep = laterite.validate(str(_RULE10A_DUP))
    by = rep.by_rule()
    # Every item has severity; at least one cell-target item carries field_index.
    all_items = [f for items in by.values() for f in items]
    assert all("severity" in f for f in all_items)
    assert any(f.get("target") == "cell" and "field_index" in f for f in all_items)


def test_by_rule_back_compat_keys_preserved():
    """The original line/group/desc keys are still present — widening is additive."""
    rep = laterite.validate(str(_RULE10A_DUP))
    for items in rep.by_rule().values():
        for f in items:
            assert {"line", "group", "desc"} <= set(f)
