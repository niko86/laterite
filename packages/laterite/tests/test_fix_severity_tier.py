"""Batch C (#294): fix()'s residual re-validation runs at the errors+warnings
tier — the same default validate() uses — rather than the errors-only (Node) /
errors+FYI (Python) tiers that had drifted apart across surfaces.

Proven via the O-44 unrecognised-TRAN_AGS *warning*: it survives into the fix
residual only when warnings are on."""

from __future__ import annotations

import laterite as L

# An unrecognised TRAN_AGS ("9.9") raises the O-44 Rule 14 WARNING; the bare-LF
# line endings raise a Rule 2a fixable ERROR so the fixer actually applies
# something. After the fix, the warning is what's left over — and it must reach
# the residual, which it does only at the errors+warnings tier (the old
# errors-only / errors+FYI tiers dropped it).
_SRC = (
    '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n'
    '"GROUP","TRAN"\n"HEADING","TRAN_AGS"\n"UNIT",""\n"TYPE","X"\n"DATA","9.9"\n'
)

_RULE_14_WARNING = "Warning (Related to Rule 14)"


def _severities(findings) -> set:
    return {f.get("severity") for f in findings}


def test_fix_residual_reports_at_the_errors_plus_warnings_tier():
    res = L.fix(text=_SRC)
    assert res.fixes_applied >= 1, "fixture should apply the Rule 2a fix"
    # The residual carries the warning tier — the O-44 Rule 14 warning — proving
    # the re-validation runs at errors+warnings, not errors-only / errors+FYI.
    assert "warning" in _severities(res.findings)
    assert any(f["rule"] == _RULE_14_WARNING for f in res.findings)


def test_chained_and_free_fix_share_the_residual_tier():
    """The fluent Ags4File.fix() delegates to the free fix(), so their residual
    tier is identical — pinned so the two paths can't drift."""
    free = L.fix(text=_SRC).findings
    report = L.read(text=_SRC).fix().fix_report
    assert report is not None
    assert _severities(free) == _severities(report.findings)
    assert "warning" in _severities(report.findings)
