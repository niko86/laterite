"""`tools/ci_failures.py` finds the job behind a check and the lines that explain it.

Pure parsing only: the network half (`gh`) is not exercised here.
"""

from __future__ import annotations

from _tools import load_tool

cf = load_tool("ci_failures")

LOG = """\
2026-11-08T00:53:12.7819559Z ##[group]Run set -euo pipefail
2026-11-08T00:53:12.7820000Z shell: /usr/bin/bash -e {0}
2026-11-08T00:53:12.7820001Z env:
2026-11-08T00:53:12.7820002Z   CARGO_TERM_COLOR: always
2026-11-08T00:53:12.7820003Z ##[endgroup]
2026-11-08T00:53:31.4600000Z \x1b[36;1mlaterite surfaces compared: 3\x1b[0m
2026-11-08T00:53:31.4610000Z !! MISSING SURFACES (1) — these produced no results at all: wasm
2026-11-08T00:53:31.4696238Z ##[error]Process completed with exit code 1.
2026-11-08T00:53:31.6000000Z Post job cleanup.
"""


def test_an_actions_link_yields_its_job_id():
    link = (
        "https://github.com/niko86/laterite/actions/runs/42430847540/job/128580293094"
    )
    assert cf.job_id(link) == "128580293094"


def test_a_non_actions_check_has_no_job():
    assert cf.job_id("https://app.codecov.io/gh/niko86/laterite/pull/1189") is None
    assert cf.job_id("") is None


def test_error_context_keeps_the_cause_and_drops_runner_noise():
    (block,) = cf.error_context(LOG, context=5)
    assert block == [
        "laterite surfaces compared: 3",
        "!! MISSING SURFACES (1) — these produced no results at all: wasm",
        "##[error]Process completed with exit code 1.",
    ]


def test_error_context_is_bounded():
    log = "\n".join(f"line {i}" for i in range(100)) + "\n##[error]boom"
    (block,) = cf.error_context(log, context=3)
    assert block == ["line 97", "line 98", "line 99", "##[error]boom"]


def test_a_log_without_an_error_line_yields_nothing():
    assert cf.error_context("all fine\n", context=5) == []
