"""A stamp is held to `required_part`, measured from the published baseline (#1147).

#1128 is the specimen: it stamped `laterite-ags4-emit` 0.20.5 — a patch — while
moving emit to `validator ^0.21`. emit's API names validator types, so the
move was breaking and `required_part` already said minor (#1073). Nothing held
the stamp to that: coherence asks only whether requirements can be satisfied,
and "publish owed" asks only whether the version is on the registry. The
nightly published 0.20.5 unattended, and `laterite` 0.15.0 stopped compiling on
a fresh resolution.

Everything here is pure over a typed report; no test touches the network.
"""

from __future__ import annotations

import json
from dataclasses import asdict, replace

from _tools import default_crate, load_tool, report_of

rs = load_tool("release_status")
ec = load_tool("engine_cut")

DEFAULT = default_crate(rs)

EMIT = "laterite-ags4-emit"
HOSTOPTS = "laterite-ags4-hostopts"


def _stamp(crate: str, live: str, stamped: str, part: str, **kw):
    """A crate stamped ahead of the registry, measured from its publish commit."""
    fields = {
        "crate": crate,
        "version": stamped,
        "registry_state": "owed",
        "registry_latest": live,
        "published_live": live,
        "delta_baseline": f"publish {live}",
        "baseline_kind": "publish",
        "part_required": part,
    }
    return replace(DEFAULT, **(fields | kw))


def _1128():
    """The #1128 tree against the registry as it stood (emit 0.20.4, hostopts 0.3.3)."""
    return report_of(
        rs,
        _stamp(EMIT, "0.20.4", "0.20.5", "minor"),
        _stamp(HOSTOPTS, "0.3.3", "0.3.4", "patch", code_changed=True),
    )


def test_emit_0_20_5_is_an_under_bump_and_hostopts_0_3_4_is_not():
    reading = rs.stamp_reading(_1128())
    assert [s.crate for s in reading.under] == [EMIT]
    assert reading.under[0].stamped_part == "patch"
    assert reading.under[0].required == "minor"
    assert HOSTOPTS not in [s.crate for s in reading.under]


def test_an_over_bump_passes_and_is_reported():
    """A minor where a patch was owed spends a number; it breaks nobody."""
    reading = rs.stamp_reading(
        report_of(rs, _stamp(HOSTOPTS, "0.3.3", "0.4.0", "patch", code_changed=True))
    )
    assert reading.under == []
    assert [(s.crate, s.stamped_part, s.required) for s in reading.over] == [
        (HOSTOPTS, "minor", "patch")
    ]


def test_a_stamp_carrying_exactly_the_owed_part_is_held():
    reading = rs.stamp_reading(
        report_of(rs, _stamp(HOSTOPTS, "0.3.3", "0.3.4", "patch", code_changed=True))
    )
    assert (reading.under, reading.over) == ([], [])
    assert [s.crate for s in reading.held] == [HOSTOPTS]


def test_a_stamp_with_no_publish_commit_baseline_is_skipped_and_named():
    """A stamp-only baseline may already be on the registry: judge nothing."""
    crate = _stamp(EMIT, "0.20.4", "0.20.5", "minor", baseline_kind="stamp")
    reading = rs.stamp_reading(report_of(rs, crate))
    assert reading.under == []
    assert [name for name, _ in reading.skipped] == [EMIT]


def test_a_first_publish_is_skipped_and_named():
    crate = replace(DEFAULT, crate=EMIT, registry_state="new", published_live="")
    reading = rs.stamp_reading(report_of(rs, crate))
    assert [name for name, _ in reading.skipped] == [EMIT]


def test_an_unreachable_crate_is_reported_not_judged():
    crate = replace(DEFAULT, crate=EMIT, registry_state="unknown", published_live="")
    reading = rs.stamp_reading(report_of(rs, crate))
    assert (reading.under, reading.skipped) == ([], [])
    assert reading.unreachable == [EMIT]


def test_a_level_crate_is_counted_and_carries_no_stamp():
    reading = rs.stamp_reading(report_of(rs, DEFAULT))
    assert (reading.under, reading.over, reading.held) == ([], [], [])
    assert reading.level == [DEFAULT.crate]


def test_an_unmeasured_delta_leaves_a_patch_stamp_unmeasured():
    """+0 -0 from a render that never ran is no reading: the minor may be owed."""
    crate = _stamp(
        EMIT,
        "0.20.4",
        "0.20.5",
        "patch",
        code_changed=True,
        api_delta_source="unavailable",
    )
    reading = rs.stamp_reading(report_of(rs, crate))
    assert (reading.under, reading.held) == ([], [])
    assert [name for name, _ in reading.unmeasured] == [EMIT]


def test_an_unmeasured_delta_cannot_rescue_a_stamp_the_measured_signals_sink():
    """A dep move already demands the minor; no API delta can lower it."""
    crate = _stamp(EMIT, "0.20.4", "0.20.5", "minor", api_delta_source="unavailable")
    assert [s.crate for s in rs.stamp_reading(report_of(rs, crate)).under] == [EMIT]


def test_an_unmeasured_delta_cannot_sink_a_minor_stamp():
    """On 0.x a minor is the most any signal asks, so the stamp is covered.

    Held, not over: whether the minor was owed is exactly what went unmeasured.
    """
    crate = _stamp(EMIT, "0.20.4", "0.21.0", "patch", api_delta_source="unavailable")
    reading = rs.stamp_reading(report_of(rs, crate))
    assert (reading.unmeasured, reading.over) == ([], [])
    assert [s.crate for s in reading.held] == [EMIT]


# --- the PR gate: `release_status.py --check-stamps` ---


def _check_stamps(monkeypatch, capsys, report) -> tuple[int, str]:
    monkeypatch.setattr(rs, "collect", lambda **kw: report)
    monkeypatch.setattr(rs.sys, "argv", ["release_status.py", "--check-stamps"])
    code = rs.main()
    return code, capsys.readouterr().out


def test_the_gate_fails_on_1128_and_names_the_under_bump(monkeypatch, capsys):
    code, out = _check_stamps(monkeypatch, capsys, _1128())
    assert code == 1
    assert "UNDER-BUMP" in out
    assert f"{EMIT} 0.20.4 -> 0.20.5" in out
    assert HOSTOPTS not in out.split("UNDER-BUMP", 1)[1].splitlines()[0]


def test_the_gate_passes_an_over_bump_and_names_it(monkeypatch, capsys):
    report = report_of(rs, _stamp(HOSTOPTS, "0.3.3", "0.4.0", "patch"))
    code, out = _check_stamps(monkeypatch, capsys, report)
    assert code == 0
    assert f"over-bump   {HOSTOPTS} 0.3.3 -> 0.4.0" in out


def test_an_unreachable_crate_passes_the_gate_as_unmeasured(monkeypatch, capsys):
    report = report_of(rs, replace(DEFAULT, registry_state="unknown"))
    code, out = _check_stamps(monkeypatch, capsys, report)
    assert code == 0
    assert f"unreachable {DEFAULT.crate}" in out


def test_every_run_prints_checked_skipped_and_unreachable(monkeypatch, capsys):
    report = report_of(
        rs,
        _stamp(HOSTOPTS, "0.3.3", "0.3.4", "patch"),
        _stamp(EMIT, "0.20.4", "0.20.5", "minor", baseline_kind="stamp"),
        replace(DEFAULT, crate="laterite-ags4-diff", registry_state="unknown"),
        DEFAULT,
    )
    _, out = _check_stamps(monkeypatch, capsys, report)
    assert (
        "stamps: 1 checked, 1 skipped, 1 unreachable, 0 unmeasured, "
        "1 level with the registry" in out
    )


# --- the nightly: `engine_cut.py --publish-owed` refuses the whole set ---

VALIDATOR = "laterite-ags4-validator"


def _publish_owed(monkeypatch, capsys, tmp_path, report) -> tuple[int, str, str]:
    path = tmp_path / "status.json"
    path.write_text(json.dumps(asdict(report)))
    monkeypatch.setattr(ec.sys, "argv", ["engine_cut.py", "--publish-owed", str(path)])
    code = ec.main()
    out = capsys.readouterr()
    return code, out.out, out.err


def _validator_owed():
    """validator 0.21.0, stamped as a minor and owed — what #1128's night dispatched."""
    return _stamp(
        VALIDATOR, "0.20.1", "0.21.0", "minor", cut_action="publish", tier="engine"
    )


def test_1128s_night_refuses_the_whole_owed_set(monkeypatch, capsys, tmp_path):
    """The cut owed emit a BUMP, yet the dispatch published it beside validator."""
    emit = _stamp(EMIT, "0.20.4", "0.20.5", "minor", cut_action="bump")
    report = report_of(rs, emit, _validator_owed())
    code, out, err = _publish_owed(monkeypatch, capsys, tmp_path, report)
    assert code == ec.EXIT_PUBLISH_REFUSED
    assert out == ""  # nothing to dispatch: not even validator, which was fine
    assert "UNDER-BUMP" in err
    assert EMIT in err


def test_an_unreachable_crate_refuses_the_whole_owed_set(monkeypatch, capsys, tmp_path):
    dark = replace(DEFAULT, crate=EMIT, registry_state="unknown", cut_action="none")
    report = report_of(rs, dark, _validator_owed())
    code, out, err = _publish_owed(monkeypatch, capsys, tmp_path, report)
    assert (code, out) == (ec.EXIT_PUBLISH_REFUSED, "")
    assert f"unreachable {EMIT}" in err


def test_a_clean_owed_set_dispatches_and_prints_its_counts(
    monkeypatch, capsys, tmp_path
):
    report = report_of(rs, _validator_owed(), DEFAULT)
    code, out, err = _publish_owed(monkeypatch, capsys, tmp_path, report)
    assert (code, out) == (0, f"{VALIDATOR}\n")
    assert "stamps: 1 checked, 0 skipped, 0 unreachable" in err
