"""`--dict` custom-dictionary overlay (#568) on the Python surface.

The overlay lets a delivery carry a bespoke group (here `XTRA`, hung off the standard
`SAMP`) and still validate as first-class, instead of being flagged unknown. These tests
assert the OUTPUT — the count of `XTRA` findings, `report.certified`, and the
`revalidate_reason` token — never that the flag merely parses. The distinction is the
whole feature: a flag that is accepted and dropped looks identical to one that works,
right up until the findings differ.

Referenced, not copied: the dictionary + delivery fixtures are the same ones the Rust
E2E test (`custom_dict.rs`) exercises. A second copy is a second thing to drift.
"""

from __future__ import annotations

import json
from pathlib import Path

import laterite
import pytest
from laterite import _cli

_FIX = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
)
_CUSTOM = _FIX / "custom_dict"
_DELIVERY = _CUSTOM / "delivery_with_xtra.ags"
_DICT_JSON = _CUSTOM / "xtra.dict.json"
_DICT_AGS = _CUSTOM / "xtra.dict.ags"
#: TRAN_AGS 4.2, validates error-clean — the file the cert round-trip mints over.
_CLEAN = _FIX / "clean_minimal.ags"


def _xtra_findings(report) -> int:
    """Findings that reference the bespoke XTRA group."""
    df = report.findings
    return df.filter((df["group"] == "XTRA") | df["desc"].str.contains("XTRA")).height


# --- library: the overlay makes a bespoke group first-class -----------------


def test_bundled_dictionary_flags_the_unknown_group():
    r = laterite.validate(str(_DELIVERY))
    assert _xtra_findings(r) > 0, (
        "the bundled dictionary must flag the unknown XTRA group"
    )


def test_dictionary_path_json_makes_xtra_valid():
    r = laterite.validate(str(_DELIVERY), dictionary=str(_DICT_JSON))
    assert _xtra_findings(r) == 0
    # A purely-additive dict overlays the latest edition; it does not become a replacement.
    assert r.dict_version == "4.2"


def test_dictionary_ags_and_json_and_bytes_agree():
    """The `.ags` spelling of the dict, its JSON twin, and raw bytes are one dictionary."""
    n_json = laterite.validate(str(_DELIVERY), dictionary=str(_DICT_JSON)).count
    n_ags = laterite.validate(str(_DELIVERY), dictionary=str(_DICT_AGS)).count
    n_bytes = laterite.validate(
        str(_DELIVERY), dictionary=_DICT_JSON.read_bytes()
    ).count
    assert n_json == n_ags == n_bytes
    # And the overlay strictly reduces findings vs the bundled dictionary.
    assert n_json < laterite.validate(str(_DELIVERY)).count


def test_dict_replace_contradicts_dict_version():
    with pytest.raises(laterite.BadDictError):
        laterite.validate(
            str(_DELIVERY),
            dictionary=str(_DICT_JSON),
            dict_replace=True,
            dict_version="4.1",
        )


def test_bad_dictionary_raises_bad_dict():
    with pytest.raises(laterite.BadDictError):
        laterite.validate(str(_DELIVERY), dictionary=str(_CUSTOM / "nope.json"))


# --- Report.revalidate_reason: the cert records which dictionary judged ------
# (O-48 record-not-contract — a dict mismatch REVALIDATES, it never hard-fails.)


def test_revalidate_reason_is_none_without_a_certificate():
    r = laterite.validate(str(_DELIVERY), dictionary=str(_DICT_JSON))
    assert r.revalidate_reason is None


def test_matching_config_uses_the_certificate(tmp_path):
    src = tmp_path / "clean.ags"
    src.write_bytes(_CLEAN.read_bytes())
    cert = laterite.read(str(src)).certify()
    r = laterite.read(str(src), index=str(cert)).validate().report
    assert r.certified is True
    assert r.revalidate_reason is None


def test_adding_a_dict_to_a_bare_cert_revalidates(tmp_path):
    src = tmp_path / "clean.ags"
    src.write_bytes(_CLEAN.read_bytes())
    cert = laterite.read(str(src)).certify()  # minted WITHOUT a custom dict
    r = (
        laterite.read(str(src), index=str(cert))
        .validate(dictionary=str(_DICT_JSON))
        .report
    )
    assert r.certified is False
    assert r.revalidate_reason == "dictionary_changed"


def test_certify_stamps_the_dict_and_a_matching_read_is_certified(tmp_path):
    src = tmp_path / "clean.ags"
    src.write_bytes(_CLEAN.read_bytes())
    # Mint a cert AGAINST the custom dict, then read back with the same dict.
    cert = laterite.read(str(src)).certify(dictionary=str(_DICT_JSON))
    same = (
        laterite.read(str(src), index=str(cert))
        .validate(dictionary=str(_DICT_JSON))
        .report
    )
    assert same.certified is True
    # A bare read of a dict-stamped cert cannot inherit the verdict — it revalidates.
    bare = laterite.read(str(src), index=str(cert)).validate().report
    assert bare.certified is False
    assert bare.revalidate_reason == "dictionary_changed"


# --- uvx CLI: --dict / --dict-replace, faithful to the native binary --------


def test_cli_validate_dict_overlay_removes_xtra_findings(tmp_path, capsys):
    rc = _cli.main(["validate", str(_DELIVERY), "--dict", str(_DICT_JSON), "--json"])
    out = capsys.readouterr().out
    findings = json.loads(out)["findings"]
    xtra = sum(1 for vs in findings.values() for f in vs if f.get("group") == "XTRA")
    assert xtra == 0
    # exit reflects the residual (non-XTRA) findings, not a parse failure.
    assert rc in (0, 1)


def test_cli_dict_replace_contradicts_dict_version(capsys):
    rc = _cli.main(
        [
            "validate",
            str(_DELIVERY),
            "--dict",
            str(_DICT_JSON),
            "--dict-replace",
            "--dict-version",
            "4.1",
        ]
    )
    assert rc == 5
    assert "replace" in capsys.readouterr().err.lower()


def test_cli_bad_dict_is_exit_5(capsys):
    rc = _cli.main(["validate", str(_DELIVERY), "--dict", str(_CUSTOM / "nope.json")])
    assert rc == 5


def test_cli_fix_accepts_dict(tmp_path, capsys):
    out = tmp_path / "fixed.ags"
    rc = _cli.main(
        ["fix", str(_DELIVERY), "--dict", str(_DICT_JSON), "--fix-out", str(out)]
    )
    # 0 clean / 1 residual — either way the fix ran and wrote a file, not a dict refusal.
    assert rc in (0, 1)
    assert out.is_file()
