"""#1009 — ABBR codes that differ only by letter case.

Rule 16 looks codes up exactly, so ``Undisturbed`` and ``UNDISTURBED`` under one
heading are two rows that each define their own spelling, and no rule fires; an
importer that keys ABBR case-insensitively rejects the second as a duplicate.
Pinned on the issue's own delivery shape:

* the collision FYI (O-58), whatever the codes' standard status;
* the "did you mean" hint on the non-standard-code FYI (O-43);
* both opt-in: without ``fyi=True`` the findings and the verdict are untouched.
"""

from __future__ import annotations

import laterite
import polars as pl

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI16 = "FYI (Related to Rule 16)"


def _build(trig_cond: tuple[str, str]) -> bytes:
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
        frames, dict_version="4.1.1", synthesise_metadata=True, tran=STAMP
    ).bytes


def _fyi16(report) -> list[str]:
    return [d["desc"] for d in report.findings.to_dicts() if d["rule"] == FYI16]


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
