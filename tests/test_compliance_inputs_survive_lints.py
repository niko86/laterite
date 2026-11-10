"""A lint failure in a job that emits a compliance surface cannot also cost the surface.

The `compliance` job compares four surfaces, and three of them are emitted as a
side job of a lint job (`python`, `node`, `ts-lint`). When a check step ahead
of an emit fails, the emit is skipped, and `compliance` reports the surface as
MISSING. That second red points at the wrong thing. #1189 showed it: the
lucide icon drift failed ts-lint's sync-assets check, and `compliance` went red
with "MISSING SURFACES: wasm" over an engine nobody had touched.

So every step from the job's install to the surface upload must run unless the
install failed or the run was cancelled. That means
`if: ${{ !cancelled() && steps.<install>.outcome == 'success' }}`, or a step
with no check before it.

WHAT THIS CANNOT SEE: it reads `ci.yml` only, and only steps named
`compliance — emit …` plus the upload that follows them. A surface emitted any
other way is not checked. The number of emits it found is asserted.
"""

from __future__ import annotations

from pathlib import Path

import yaml

CI = Path(__file__).resolve().parents[1] / ".github" / "workflows" / "ci.yml"
EMIT = "compliance — emit"


def _jobs() -> dict:
    return yaml.safe_load(CI.read_text())["jobs"]


def _emit_jobs() -> dict[str, list[dict]]:
    return {
        name: job["steps"]
        for name, job in _jobs().items()
        if any(s.get("name", "").startswith(EMIT) for s in job.get("steps", []))
    }


def _guarded(step: dict) -> bool:
    cond = str(step.get("if", ""))
    return "!cancelled()" in cond and ".outcome == 'success'" in cond


def _is_check(step: dict) -> bool:
    """A step that can fail on the code: anything that is not setup or build."""
    name = step.get("name", "")
    return "run" in step and any(
        w in name for w in ("lint", "check", "eslint", "prettier", "unchanged")
    )


def test_every_surface_emit_was_found():
    # python, node and wasm. A fourth (rust) runs inside `compliance` itself.
    assert len(_emit_jobs()) == 3


def test_no_check_ahead_of_an_emit_can_skip_it():
    offenders = []
    for job, steps in _emit_jobs().items():
        emit = next(
            i for i, s in enumerate(steps) if s.get("name", "").startswith(EMIT)
        )
        upload = emit + 1
        checks_before = [s["name"] for s in steps[:emit] if _is_check(s)]
        if not checks_before:
            continue
        first_check = next(i for i, s in enumerate(steps) if _is_check(s))
        offenders.extend(
            f"{job}: '{s.get('name', s.get('uses'))}'"
            for s in steps[first_check + 1 : upload + 1]
            if not _is_check(s) and not _guarded(s)
        )
    assert not offenders, (
        "a check ahead of the compliance emit can skip these steps; guard each "
        "with `if: ${{ !cancelled() && steps.<install>.outcome == 'success' }}`:\n  "
        + "\n  ".join(offenders)
    )
