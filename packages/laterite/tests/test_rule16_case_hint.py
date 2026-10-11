"""#1033 — a PA value that matches a declared ABBR code apart from letter case.

The lookup stays exact in every edition, so the file still fails Rule 16; a
4.2 file's error only gains a sentence naming the declared code and why it
still fires. Earlier editions, and compat, keep the plain wording.

The fixture is built clean (LOCA_TYPE "CP", declared as "CP") and then
lower-cased in the LOCA row alone by byte substitution, so ABBR still declares
"CP".
"""

from __future__ import annotations

import io

import laterite
import polars as pl
import pytest

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
R16 = "AGS Format Rule 16"
PLAIN = 'Abbreviation "cp" under LOCA_TYPE is not defined in the ABBR group.'


def _clean(edition: str) -> bytes:
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_TYPE": ["CP"]}),
    }
    return laterite.build_ags4(
        frames, dict_version=edition, synthesise_metadata=True, tran=STAMP
    ).bytes


def _variant(edition: str, value: str = "cp") -> bytes:
    data = _clean(edition)
    needle = b'"BH1","CP"'
    assert data.count(needle) == 1, "only the LOCA row carries the value"
    return data.replace(needle, f'"BH1","{value}"'.encode())


def _rule_16(data: bytes) -> list[dict]:
    rep = laterite.validate(data, fyi=True)
    return [d for d in rep.findings.to_dicts() if d["rule"] == R16]


@pytest.mark.parametrize("edition", ["4.0.4", "4.1.1", "4.2"])
def test_the_clean_fixture_passes(edition):
    """Positive control: the finding below is caused by the case change alone."""
    assert laterite.validate(_clean(edition)).findings.height == 0


def test_4_2_case_only_mismatch_names_the_declared_code():
    rows = _rule_16(_variant("4.2"))
    assert len(rows) == 1, rows
    desc = rows[0]["desc"]
    assert desc.startswith(PLAIN), desc
    for needle in ('"CP"', "4.2", "python-ags4", "on_code_case", "recode"):
        assert needle in desc, (needle, desc)
    assert rows[0]["group"] == "LOCA"
    assert not laterite.validate(_variant("4.2")).is_valid


def test_4_2_unrelated_code_keeps_the_plain_message():
    rows = _rule_16(_variant("4.2", "zz"))
    assert [r["desc"] for r in rows] == [PLAIN.replace('"cp"', '"zz"')]


@pytest.mark.parametrize("edition", ["4.0.4", "4.1.1"])
def test_before_4_2_the_message_is_unchanged(edition):
    assert [r["desc"] for r in _rule_16(_variant(edition))] == [PLAIN]


def test_compat_keeps_python_ags4_wording_on_a_4_2_file():
    from laterite import compat

    out = compat.check_file(io.StringIO(_variant("4.2").decode()))
    assert [d["desc"] for d in out[R16]] == [
        '"cp" under LOCA_TYPE in LOCA not found in ABBR group.'
    ]
