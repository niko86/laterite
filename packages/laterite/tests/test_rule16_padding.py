"""#1012 — a PA value that fails Rule 16 only because of surrounding whitespace.

Two halves, both pinned here on the delivery shape the issue reported (a padded
SAMP_TYPE repeated down SAMP → TRIG/LNMC):

* the advisory: a separate ``FYI (Related to Rule 16)`` naming the code the value
  matches once trimmed (O-57), with the Rule 16 error itself left word-for-word;
* the repair: ``fix`` trims exactly those values, in the safe tier, in every
  group that repeats them.

The fixture is built clean and then padded by byte substitution, because
``build_ags4``'s default autofix runs the same safe fixes and would trim the
padding before the file ever reached ``validate``.
"""

from __future__ import annotations

import io

import laterite
import polars as pl
import pytest

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI16 = "FYI (Related to Rule 16)"
R16 = "AGS Format Rule 16"


def _clean(second: str = "D", first: str = "U", *, same_keys: bool = False) -> bytes:
    """With ``same_keys`` the two samples share every KEY value but SAMP_TYPE."""
    k = {
        "LOCA_ID": ["BH1", "BH1"],
        "SAMP_TOP": [1.0, 1.0] if same_keys else [1.0, 2.0],
        "SAMP_REF": ["1", "1"] if same_keys else ["1", "2"],
        "SAMP_TYPE": [first, second],
        "SAMP_ID": ["S1", "S1"] if same_keys else ["S1", "S2"],
    }
    spec = {
        "SPEC_REF": ["1", "1"],
        "SPEC_DPTH": [1.0, 1.0] if same_keys else [1.0, 2.0],
    }
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": pl.DataFrame(k),
        "TRIG": pl.DataFrame(
            {**k, **spec, "TRIG_TYPE": ["UU", "UU"], "TRIG_COND": ["UNDISTURBED"] * 2}
        ),
        "LNMC": pl.DataFrame({**k, **spec, "LNMC_MC": [11.3, 14.1]}),
    }
    return laterite.build_ags4(
        frames, dict_version="4.1.1", synthesise_metadata=True, tran=STAMP
    ).bytes


def _padded(clean_code: str, padded_code: str) -> bytes:
    """The second sample's SAMP_TYPE rewritten in all three groups."""
    data = _clean(clean_code)
    needle = f'"{clean_code}","S2"'.encode()
    assert data.count(needle) == 3, "SAMP + TRIG + LNMC each carry the key"
    return data.replace(needle, f'"{padded_code}","S2"'.encode())


def _bd_defined() -> bytes:
    """A clean file whose ABBR defines both B and D (and TRAN_RCON is "+")."""
    # Build with "B+D" so the synthesised ABBR carries B and D as separate codes.
    return _clean("B+D")


def _rows(report, rule: str) -> list[str]:
    return [d["desc"] for d in report.findings.to_dicts() if d["rule"] == rule]


def _padding_fyis(report) -> list[str]:
    return [d for d in _rows(report, FYI16) if "surrounding whitespace" in d]


def test_the_padded_fixture_is_clean_before_padding():
    """Positive control: everything below is caused by the padding alone."""
    assert laterite.validate(_clean()).findings.height == 0
    assert laterite.validate(_bd_defined()).findings.height == 0


def test_fyi_names_heading_padded_value_and_defined_code():
    data = _padded("D", " D")
    rep = laterite.validate(data, fyi=True)
    err = 'Abbreviation " D" under SAMP_TYPE is not defined in the ABBR group.'
    assert _rows(rep, R16) == [err] * 3  # SAMP, TRIG, LNMC — unchanged
    fyi = _padding_fyis(rep)
    assert len(fyi) == 3
    assert all("SAMP_TYPE" in d and '" D"' in d and '"D"' in d for d in fyi), fyi


def test_without_fyi_the_findings_are_unchanged():
    data = _padded("D", " D")
    quiet = laterite.validate(data)
    assert _padding_fyis(quiet) == []
    assert set(quiet.findings["rule"].to_list()) == {R16}


def test_each_padded_concatenated_part_gets_its_own_fyi():
    data = _padded("B+D", "B + D")
    fyi = _padding_fyis(laterite.validate(data, fyi=True))
    # Two padded parts, each repeated once per group.
    assert sum('"B "' in d for d in fyi) == 3, fyi
    assert sum('" D"' in d for d in fyi) == 3, fyi


def test_no_fyi_when_the_trimmed_value_is_undefined_too():
    data = _padded("D", " X")
    rep = laterite.validate(data, fyi=True)
    assert _rows(rep, R16)
    assert _padding_fyis(rep) == []


@pytest.mark.parametrize(
    ("clean", "padded", "repaired"),
    [("D", " D", "D"), ("B+D", "B + D", "B+D")],
)
def test_fix_trims_and_leaves_nothing_new(clean, padded, repaired):
    data = _padded(clean, padded)
    fixed = laterite.fix(data=data)  # the SAFE tier: no risky=True
    assert [a["kind"] for a in fixed.applied] == ["trim_abbreviation"]
    after = laterite.validate(fixed.bytes, fyi=True)
    assert after.findings.height == 0, after.findings.to_dicts()
    # The fix restores exactly the clean file, every copy of the key included.
    assert fixed.bytes == _clean(clean)
    assert fixed.bytes.count(f'"{repaired}","S2"'.encode()) == 3


def test_fix_leaves_a_cell_with_an_undefined_part_untouched():
    # " D" alone is rescuable; " X" is not, so the whole cell stays as it is.
    data = _padded("B+D", " D+ X")
    fixed = laterite.fix(data=data, risky=True)
    assert not any(a["kind"] == "trim_abbreviation" for a in fixed.applied)
    assert fixed.bytes.count(b'" D+ X","S2"') == 3
    assert '" X"' in "".join(_rows(laterite.validate(fixed.bytes), R16))


def test_compat_keeps_python_ags4_rule_16_wording():
    data = _padded("D", " D")
    from laterite import compat

    out = compat.check_file(io.StringIO(data.decode()))
    descs = [d["desc"] for d in out[R16]]
    assert descs and all(
        d.startswith('" D" under SAMP_TYPE in ')
        and d.endswith(" not found in ABBR group.")
        for d in descs
    ), descs


def test_fix_withholds_a_trim_that_would_merge_two_keys():
    """Two samples identical but for SAMP_TYPE " D" vs "D": trimming would give
    them one KEY tuple in every group, so the padded cells must stay."""
    clean = _clean("D", "B", same_keys=True)
    # SAMP_ID repeats, which Rule 8's ID-uniqueness flags (O-11); the KEY
    # tuples themselves are distinct, so no Rule 10a yet.
    start = set(laterite.validate(clean).findings["rule"].to_list())
    assert "AGS Format Rule 10a" not in start, start
    needle = b'"B","S1"'
    assert clean.count(needle) == 3
    data = clean.replace(needle, b'" D","S1"')
    fixed = laterite.fix(data=data, risky=True)
    assert not any(a["kind"] == "trim_abbreviation" for a in fixed.applied)
    assert fixed.bytes.count(b'" D","S1"') == 3
    rules = set(laterite.validate(fixed.bytes).findings["rule"].to_list())
    assert R16 in rules
    assert "AGS Format Rule 10a" not in rules
