"""`max_findings_per_rule` (#1089): a per-rule cap on what a `Report` KEEPS.

What a cap may change is the findings a report holds. What it may not change is what
the report concludes: the verdict, the per-tier counts and `count`. Every test here
compares a capped run with the uncapped run of the same file, so the uncapped engine is
the oracle and no expected count is written down by hand.
"""

from __future__ import annotations

import json
from pathlib import Path

import laterite as lat
import pytest
from laterite import _cli

RULE_2A = "AGS Format Rule 2a"

#: The shared clean fixture. With its CRLF line endings rewritten to LF it carries one
#: Rule 2a finding per line, which is the #1089 shape in miniature.
_CLEAN = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
    / "clean_minimal.ags"
).read_bytes()
LF = _CLEAN.replace(b"\r\n", b"\n")
#: Error-clean with one warning: an edition this build does not bundle.
WITH_WARNING = _CLEAN.replace(b'"4.2"', b'"4.9.9"')


@pytest.fixture
def lf(tmp_path):
    p = tmp_path / "lf.ags"
    p.write_bytes(LF)
    return p


def _verdict(r: lat.Report):
    return (r.count, r.errors, r.warnings, r.fyi, r.is_valid, r.exit_code)


def test_the_fixture_has_a_rule_worth_cutting(lf):
    full = lat.validate(lf)
    assert len(full.by_rule()[RULE_2A]) > 2
    assert WITH_WARNING != _CLEAN, "the warning fixture must differ from the clean one"


@pytest.mark.parametrize("cap", [0, 1, 2])
def test_a_cap_keeps_each_rules_first_n_and_every_count(lf, cap):
    full = lat.validate(lf, fyi=True)
    capped = lat.validate(lf, fyi=True, max_findings_per_rule=cap)
    assert _verdict(capped) == _verdict(full)
    expected = {
        rule: {"shown": cap, "total": len(items)}
        for rule, items in full.by_rule().items()
        if len(items) > cap
    }
    assert capped.truncated == expected
    for rule, items in full.by_rule().items():
        assert capped.by_rule().get(rule, []) == items[:cap], rule
    assert capped.findings.height == sum(
        min(len(v), cap) for v in full.by_rule().values()
    )


def test_zero_keeps_no_findings_and_the_serialisations_say_so(lf):
    full = lat.validate(lf)
    zero = lat.validate(lf, max_findings_per_rule=0)
    assert zero.findings.height == 0
    assert zero.count == full.count > 0
    doc = json.loads(zero.to_json())
    assert doc["findings"] == {}
    assert doc["truncated"] == zero.truncated
    lines = zero.to_ndjson().splitlines()
    assert [json.loads(x) for x in lines] == [{"truncated": zero.truncated}]


def test_an_uncapped_report_carries_no_truncation(lf):
    # Byte-identity with the pre-cap output: no `truncated` key, no summary record.
    for r in (lat.validate(lf), lat.validate(lf, max_findings_per_rule=10_000)):
        assert r.truncated == {}
        assert "truncated" not in json.loads(r.to_json())
        assert all("rule" in json.loads(x) for x in r.to_ndjson().splitlines())
    roomy = lat.validate(lf, max_findings_per_rule=10_000)
    assert roomy.to_json() == lat.validate(lf).to_json()
    assert roomy.to_ndjson() == lat.validate(lf).to_ndjson()


def test_the_handle_door_takes_the_cap_too(lf):
    full = lat.read(lf).validate().report
    r = lat.read(lf).validate(max_findings_per_rule=1).report
    assert r.truncated[RULE_2A] == {"shown": 1, "total": len(full.by_rule()[RULE_2A])}
    assert _verdict(r) == _verdict(full)


@pytest.mark.parametrize("bad", [-1, 1.5, True, "3"])
def test_a_cap_that_is_not_a_count_is_refused(lf, bad):
    with pytest.raises(ValueError, match="max_findings_per_rule"):
        lat.validate(lf, max_findings_per_rule=bad)


@pytest.fixture
def frozen_clock(monkeypatch):
    """Pin the mint's `checked_at`, so two certificates compare value for value."""
    import datetime as dt

    class _Frozen(dt.datetime):
        @classmethod
        def now(cls, tz=None):
            return cls(2026, 1, 1, tzinfo=tz)

    monkeypatch.setattr(dt, "datetime", _Frozen)


@pytest.mark.parametrize("cap", [0, 1])
def test_a_capped_run_mints_the_certificate_an_uncapped_one_does(
    tmp_path, frozen_clock, cap
):
    # A cert records counts, never findings. WITH_WARNING's one warning is exactly
    # what a cap of 0 does not keep, so it is the case that would show a difference.
    src = tmp_path / "warn.ags"
    src.write_bytes(WITH_WARNING)
    capped = lat.read(src).validate(fyi=True, max_findings_per_rule=cap).certify_bytes()
    alone = lat.read(src).certify_bytes()
    assert json.loads(capped) == json.loads(alone)
    sc = lat._laterite_native.Sidecar.from_json(capped)
    assert sc.errors == 0 and sc.warnings, "the fixture must carry a warning"


def test_lat_validate_passes_the_cap_through(lf, capsys):
    plain_code = _cli.main(["validate", str(lf), "--json"])
    full = json.loads(capsys.readouterr().out)
    code = _cli.main(["validate", str(lf), "--json", "--max-findings-per-rule", "1"])
    doc = json.loads(capsys.readouterr().out)
    assert code == plain_code
    assert len(doc["findings"][RULE_2A]) == 1
    assert doc["truncated"][RULE_2A] == {
        "shown": 1,
        "total": len(full["findings"][RULE_2A]),
    }


def test_lat_validate_plain_names_each_cut_rule(lf, capsys):
    _cli.main(["validate", str(lf), "--max-findings-per-rule", "0"])
    out = capsys.readouterr().out
    total = lat.validate(lf).count
    assert out.splitlines()[0].startswith(f"{lf}: {total} finding(s)")
    assert f"truncated: {RULE_2A} — 0 of " in out


def test_lat_validate_refuses_a_negative_cap(lf, capsys):
    # 5, the bad-arguments code all three launchers use.
    assert _cli.main(["validate", str(lf), "--max-findings-per-rule", "-1"]) == 5
    assert "--max-findings-per-rule" in capsys.readouterr().err
