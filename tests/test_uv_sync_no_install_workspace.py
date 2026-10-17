"""CI jobs that build laterite with maturin sync dependencies only (#1117).

`laterite` is a workspace member with a maturin build backend, so a plain
`uv sync` compiles the PyO3 crate to install it. In the jobs pinned here a
maturin step follows and builds laterite again (`maturin develop --uv`, which
also installs it), or builds the release wheel (`maturin build`), so the uv
build was thrown away. On a cold hosted runner it was the slowest step in the
job. These jobs therefore pass `--no-install-workspace`.

Two ways to undo it without noticing: dropping the flag, and a `uv run` without
`--no-sync` in the same job, which re-syncs the project and compiles laterite
again. Both are checked.

WHAT THIS CANNOT SEE: only the jobs named in `SITES`. Every other job that runs
`uv sync` is listed on every run, with its count. Among them are `python` and
`docs` in ci.yml, which download a prebuilt cdylib instead of running maturin
and still need uv to install laterite, because tests read its distribution
metadata (`importlib.metadata.version("laterite")`).
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

import pytest
import yaml

REPO = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO / ".github" / "workflows"

# (workflow, job) -> the maturin command that must follow the sync.
SITES: dict[tuple[str, str], str] = {
    ("ci.yml", "build-ext"): "maturin develop --uv",
    ("parity.yml", "parity"): "maturin develop --uv",
    ("nightly.yml", "output-values"): "maturin develop --uv",
    ("deploy-validator.yml", "build-deploy"): "maturin develop --uv",
    ("ci.yml", "wheel-smoke"): "maturin build",
}

UV_SYNC = re.compile(r"\buv sync\b[^\n;&|)]*")
UV_RUN = re.compile(r"\buv run\b[^\n;&|)]*")


def _commands(job: dict[str, Any]) -> list[str]:
    """Every non-comment shell line the job runs, in step order."""
    lines: list[str] = []
    for step in job.get("steps", []):
        if isinstance(step, dict) and "run" in step:
            text = step["run"].replace("\\\n", " ")
            lines += [
                ln.strip()
                for ln in text.splitlines()
                if ln.strip() and not ln.strip().startswith("#")
            ]
    return lines


def _jobs() -> dict[tuple[str, str], list[str]]:
    out: dict[tuple[str, str], list[str]] = {}
    for path in sorted(WORKFLOWS.glob("*.yml")):
        wf: dict[str, Any] = yaml.safe_load(path.read_text(encoding="utf-8"))
        for name, job in (wf.get("jobs") or {}).items():
            out[(path.name, name)] = _commands(job)
    return out


JOBS = _jobs()


def test_report_the_uv_sync_jobs_this_does_not_pin(
    capsys: pytest.CaptureFixture[str],
) -> None:
    unpinned = sorted(
        f"{wf}:{job}"
        for (wf, job), cmds in JOBS.items()
        if (wf, job) not in SITES and any(UV_SYNC.search(c) for c in cmds)
    )
    with capsys.disabled():
        print(
            f"\n[uv sync scope] pinned {len(SITES)} jobs; "
            f"{len(unpinned)} other job(s) run uv sync unchecked: "
            + ", ".join(unpinned)
        )
    assert set(SITES) <= set(JOBS), sorted(set(SITES) - set(JOBS))


@pytest.mark.parametrize("site", sorted(SITES), ids=lambda s: f"{s[0]}:{s[1]}")
def test_sync_installs_dependencies_only(site: tuple[str, str]) -> None:
    cmds = JOBS[site]
    syncs = [(i, m.group(0)) for i, c in enumerate(cmds) if (m := UV_SYNC.search(c))]
    assert syncs, f"{site} no longer runs uv sync; update SITES"
    for _, sync in syncs:
        assert "--no-install-workspace" in sync.split(), (
            f"{site}: `{sync}` builds laterite through its maturin backend, and "
            f"the job's `{SITES[site]}` builds it again; pass --no-install-workspace"
        )
    builds = [i for i, c in enumerate(cmds) if SITES[site] in c]
    assert builds, f"{site} no longer runs `{SITES[site]}`; update SITES"
    assert builds[-1] > syncs[0][0], (
        f"{site}: `{SITES[site]}` must run after the sync, or nothing installs laterite"
    )


@pytest.mark.parametrize("site", sorted(SITES), ids=lambda s: f"{s[0]}:{s[1]}")
def test_no_uv_run_re_syncs_the_workspace(site: tuple[str, str]) -> None:
    for cmd in JOBS[site]:
        for m in UV_RUN.finditer(cmd):
            assert "--no-sync" in m.group(0).split(), (
                f"{site}: `{m.group(0)}` re-syncs the project, which installs "
                "laterite and compiles it again; pass --no-sync"
            )
