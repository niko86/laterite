"""#1009 — ABBR codes that differ only by letter case.

Rule 16 looks codes up exactly, so ``Undisturbed`` and ``UNDISTURBED`` under one
heading are two rows that each define their own spelling, and no rule fires; an
importer that keys ABBR case-insensitively rejects the second as a duplicate.
Pinned on the issue's own delivery shape:

* the collision FYI (O-58), whatever the codes' standard status;
* the "did you mean" hint on the non-standard-code FYI (O-43);
* both opt-in: without ``fyi=True`` the findings and the verdict are untouched.

Under 4.2, whose specification makes ABBR codes case-insensitive, the collision
is a WARNING instead (shown by default, never fatal) and the FYI is not raised.
"""

from __future__ import annotations

import laterite
import polars as pl

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI16 = "FYI (Related to Rule 16)"
WARN16 = "Warning (Related to Rule 16)"
COLLISION = "differ only by letter case"


def _build(trig_cond: tuple[str, str], edition: str = "4.1.1") -> bytes:
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
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": pl.DataFrame(k),
        "TRIG": pl.DataFrame(
            {**k, **spec, "TRIG_TYPE": ["UU", "UU"], "TRIG_COND": list(trig_cond)}
        ),
        "LNMC": pl.DataFrame({**k, **spec, "LNMC_MC": [11.3, 14.1]}),
    }
    return laterite.build_ags4(
        frames, dict_version=edition, synthesise_metadata=True, tran=STAMP
    ).bytes


def _fyi16(report) -> list[str]:
    return [d["desc"] for d in report.findings.to_dicts() if d["rule"] == FYI16]


def _collisions(report) -> list[tuple[str, str]]:
    return [
        (d["rule"], d["desc"])
        for d in report.findings.to_dicts()
        if COLLISION in d["desc"]
    ]


def test_case_variants_raise_the_collision_and_the_hint():
    rep = laterite.validate(_build(("Undisturbed", "UNDISTURBED")), fyi=True)
    # The spellings are named in ABBR row order, and build_ags4 writes the
    # synthesised ABBR rows sorted, so "UNDISTURBED" comes first.
    assert sorted(_fyi16(rep)) == [
        'TRIG_COND: abbreviation "Undisturbed" is declared in the ABBR group but '
        "is not a recognised standard abbreviation for TRIG_COND; "
        'did you mean "UNDISTURBED"?',
        'TRIG_COND: codes "UNDISTURBED" and "Undisturbed" differ only by letter '
        "case; some importers treat them as the same code.",
    ]


def test_without_fyi_the_case_variants_change_nothing():
    variant = laterite.validate(_build(("Undisturbed", "UNDISTURBED")))
    plain = laterite.validate(_build(("UNDISTURBED", "UNDISTURBED")))
    assert _fyi16(variant) == []
    assert variant.is_valid == plain.is_valid
    assert (
        variant.findings.select("rule").to_dicts()
        == plain.findings.select("rule").to_dicts()
    )


def test_under_4_2_the_collision_is_a_warning_with_or_without_fyi():
    data = _build(("Undisturbed", "UNDISTURBED"), edition="4.2")
    plain = laterite.validate(_build(("UNDISTURBED", "UNDISTURBED"), edition="4.2"))
    warning = [
        (
            WARN16,
            'TRIG_COND: codes "UNDISTURBED" and "Undisturbed" differ only by '
            "letter case; some importers treat them as the same code. AGS 4.2 "
            "treats ABBR codes as case-insensitive.",
        )
    ]
    for fyi in (False, True):
        rep = laterite.validate(data, fyi=fyi)
        # The WARNING replaces the FYI: never both for one collision.
        assert _collisions(rep) == warning, fyi
        # A warning is shown, not fatal.
        assert rep.is_valid == plain.is_valid, fyi
    # --no-warnings drops it, and no FYI stands in for it.
    assert _collisions(laterite.validate(data, warnings=False, fyi=True)) == []


def test_before_4_2_the_collision_stays_an_opt_in_fyi():
    data = _build(("Undisturbed", "UNDISTURBED"), edition="4.1.1")
    assert _collisions(laterite.validate(data)) == []
    assert _collisions(laterite.validate(data, fyi=True)) == [
        (
            FYI16,
            'TRIG_COND: codes "UNDISTURBED" and "Undisturbed" differ only by '
            "letter case; some importers treat them as the same code.",
        )
    ]
