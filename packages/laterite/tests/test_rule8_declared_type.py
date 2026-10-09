"""#1013 — a standard heading declared with a different TYPE from the dictionary.

Rule 8 judges values against the file's own TYPE row, so ``LNMC_MC`` declared
``1DP`` where the 4.1.1 dictionary says ``X`` validates clean; the difference
only bites once the file is combined with one typed the dictionary's way.
Pinned on the issue's own repro:

* one ``FYI (Related to Rule 8)`` per heading, naming the edition and its type;
* a precision-only difference counts; matching and user-defined headings are silent;
* the edition named is the one validation resolved, an explicit one included;
* opt-in: without ``fyi=True`` the findings and the verdict are untouched.
"""

from __future__ import annotations

import laterite
import polars as pl

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI8 = "FYI (Related to Rule 8)"


def _crlf(text: str) -> bytes:
    return text.replace("\n", "\r\n").encode()


def _build() -> str:
    k = {
        "LOCA_ID": ["BH1", "BH1"],
        "SAMP_TOP": [1.0, 2.0],
        "SAMP_REF": ["1", "2"],
        "SAMP_TYPE": ["U", "D"],
        "SAMP_ID": ["S1", "S2"],
    }
    spec = {"SPEC_REF": ["1", "1"], "SPEC_DPTH": [1.0, 2.0]}
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_GL": [10.0]}),
        "SAMP": pl.DataFrame(k),
        "LNMC": pl.DataFrame({**k, **spec, "LNMC_MC": ["11.3", "14.1"]}),
    }
    return (
        laterite.build_ags4(
            frames, dict_version="4.1.1", synthesise_metadata=True, tran=STAMP
        )
        .bytes.decode()
        .replace("\r\n", "\n")
    )


def _retype(text: str, old_line: str, new_line: str, new_type: str) -> bytes:
    """Swap one TYPE row and define the code it introduces in the TYPE group."""
    assert text.count(old_line) == 1, old_line
    text = text.replace(old_line, new_line)
    text = text.replace(
        '"DATA","ID","ID"\n', f'"DATA","ID","ID"\n"DATA","{new_type}","{new_type}"\n'
    )
    return _crlf(text)


LNMC_TYPES = '"TYPE","ID","2DP","X","PA","ID","X","2DP","X"\n'


def _mc_as_1dp() -> bytes:
    return _retype(
        _build(), LNMC_TYPES, '"TYPE","ID","2DP","X","PA","ID","X","2DP","1DP"\n', "1DP"
    )


def _fyi8(report) -> list[str]:
    return [d["desc"] for d in report.findings.to_dicts() if d["rule"] == FYI8]


def test_the_issue_repro_names_heading_type_edition_once():
    rep = laterite.validate(_mc_as_1dp(), fyi=True)
    assert _fyi8(rep) == [
        "LNMC.LNMC_MC is declared 1DP; the 4.1.1 dictionary type is X."
    ]
    assert rep.is_valid


def test_a_precision_difference_counts():
    text = _build()
    data = laterite.validate(
        _retype(
            text.replace('"DATA","BH1","10.00"', '"DATA","BH1","10.000"'),
            '"TYPE","ID","2DP"\n',
            '"TYPE","ID","3DP"\n',
            "3DP",
        ),
        fyi=True,
    )
    assert _fyi8(data) == [
        "LOCA.LOCA_GL is declared 3DP; the 4.1.1 dictionary type is 2DP."
    ]


def test_dictionary_typed_and_user_defined_headings_are_silent():
    assert _fyi8(laterite.validate(_crlf(_build()), fyi=True)) == []
    # A heading the standard does not define has no dictionary type to differ
    # from, whatever it declares.
    text = _build().replace('"LNMC_MC"\n', '"LNMC_ZZZZ"\n')
    rep = laterite.validate(
        _retype(
            text, LNMC_TYPES, '"TYPE","ID","2DP","X","PA","ID","X","2DP","1DP"\n', "1DP"
        ),
        fyi=True,
    )
    assert _fyi8(rep) == []


def test_an_explicit_edition_is_the_one_named():
    # LNMC_MC was typed MC in 4.0.3, so the dictionary-typed 4.1.1 file differs
    # from that edition, and the message says which edition it compared with.
    rep = laterite.validate(_crlf(_build()), fyi=True, dict_version="4.0.3")
    assert "LNMC.LNMC_MC is declared X; the 4.0.3 dictionary type is MC." in _fyi8(rep)


def test_without_fyi_nothing_changes():
    plain = laterite.validate(_crlf(_build()))
    retyped = laterite.validate(_mc_as_1dp())
    assert _fyi8(retyped) == []
    assert retyped.findings.to_dicts() == plain.findings.to_dicts()
    assert retyped.is_valid == plain.is_valid
    assert laterite.validate(_mc_as_1dp(), fyi=True).is_valid == retyped.is_valid


def test_native_only_not_through_compat():
    """O-59: the drop-in withholds this one FYI and nothing else."""
    import io

    from laterite import compat as AGS4

    # The repro plus a drifted ABBR description, so a Rule 16 FYI rides along
    # to show the filter is this label alone, not the FYI tier.
    drifted = _mc_as_1dp().replace(b"Small disturbed sample", b"Small bag")
    assert _fyi8(laterite.validate(drifted, fyi=True)) == [
        "LNMC.LNMC_MC is declared 1DP; the 4.1.1 dictionary type is X."
    ]
    errors = AGS4.check_file(io.StringIO(drifted.decode()))
    assert FYI8 not in errors
    assert "FYI (Related to Rule 16)" in errors


# --- the KEY-heading tier (O-59, promoted after O-52's TYPE-row trigger) ------

WARN8 = "Warning (Related to Rule 8)"
LNMC_LOCA_AS_X = '"TYPE","X","2DP","X","PA","ID","X","2DP","X"\n'


def _loca_as_x() -> bytes:
    """LNMC's KEY ``LOCA_ID`` declared ``X``; ``X`` is in the TYPE group already."""
    text = _build()
    assert text.count(LNMC_TYPES) == 1
    return _crlf(text.replace(LNMC_TYPES, LNMC_LOCA_AS_X))


def _warn8(report) -> list[dict]:
    return [d for d in report.findings.to_dicts() if d["rule"] == WARN8]


def test_a_key_heading_departure_warns_by_default():
    # No fyi=True: warnings are on by default, and this one rides that tier,
    # because a KEY column typed its own way is what python-ags4's Rule 10c
    # rejects once the parent's TYPE row disagrees (O-52).
    rep = laterite.validate(_loca_as_x())
    assert [d["desc"] for d in _warn8(rep)] == [
        "LNMC.LOCA_ID is a KEY heading declared X; the 4.1.1 dictionary type is ID."
    ]
    assert {d["severity"] for d in _warn8(rep)} == {"warning"}
    assert _fyi8(rep) == []
    assert rep.is_valid
    # A warning, so only the -Werror dial lets it decide anything.
    assert not laterite.validate(_loca_as_x(), warnings_as_errors=True).is_valid


def test_no_warnings_hides_the_key_warning_rather_than_demoting_it():
    rep = laterite.validate(_loca_as_x(), warnings=False, fyi=True)
    assert _warn8(rep) == []
    assert _fyi8(rep) == []


def test_the_key_warning_is_native_only_too():
    import io

    from laterite import compat as AGS4

    errors = AGS4.check_file(io.StringIO(_loca_as_x().decode()))
    assert WARN8 not in errors
    assert not any("Rule" in k for k in errors), sorted(errors)
