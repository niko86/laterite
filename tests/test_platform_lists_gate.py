"""The platform-list gate: a doc listing platforms must list what release.yml builds.

Pinned against the shape of the defect it exists for: linux-arm64-gnu joined the
Node release and four documents kept listing three. Every skip is tested beside
its positive control, because a skip nobody tests is a place a stale list sits
under a green tick.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

from collections import Counter

import pytest
from _tools import load_tool

NODE = {
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
}
JOBS = {
    "build-wheels": NODE,
    "build-binaries": NODE - {"aarch64-unknown-linux-gnu"},
    "build-node": NODE,
}


@pytest.fixture(scope="module")
def gate():
    return load_tool("check_platform_lists")


def _scan(gate, text: str) -> tuple[list[str], Counter]:
    tally: Counter = Counter()
    return gate.scan("doc.md", text, NODE, JOBS, tally), tally


def test_the_stale_three_is_caught(gate):
    """The README line that shipped to npm after linux-arm64-gnu was added."""
    (finding,), _ = _scan(
        gate,
        "Prebuilt binaries ship for linux-x64-gnu, darwin-arm64, and win32-x64-msvc.\n",
    )
    assert finding.startswith("doc.md:1:")
    assert "linux-arm64-gnu" in finding


def test_the_full_list_passes(gate):
    findings, tally = _scan(
        gate, "linux-x64-gnu, linux-arm64-gnu, darwin-arm64 and win32-x64-msvc\n"
    )
    assert findings == []
    assert tally["lists"] == 1


def test_scoped_names_and_brace_lists_are_read(gate):
    assert _scan(
        gate, "`@laterite/native-darwin-arm64`, `@laterite/native-win32-x64-msvc`\n"
    )[0]
    full = "`@laterite/native-{darwin-arm64,linux-x64-gnu,linux-arm64-gnu,win32-x64-msvc}`\n"
    assert _scan(gate, full)[0] == []


def test_a_list_spans_its_paragraph_not_its_line(gate):
    """RELEASING-node.md lists three on one line and the fourth in a note below,
    inside the same paragraph."""
    text = "For each of linux-x64-gnu, darwin-arm64,\nwin32-x64-msvc: do X.\nLater, linux-arm64-gnu too.\n"
    assert _scan(gate, text)[0] == []
    split = "linux-x64-gnu, darwin-arm64, win32-x64-msvc\n\nLater, linux-arm64-gnu.\n"
    assert _scan(gate, split)[0]


def test_one_mention_is_not_a_list(gate):
    findings, tally = _scan(
        gate, "The arm64 package (`@laterite/native-linux-arm64-gnu`).\n"
    )
    assert findings == []
    assert tally["single"] == 1


def test_a_triple_list_must_match_some_release_job(gate):
    binaries = (
        "x86_64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-pc-windows-msvc\n"
    )
    assert _scan(gate, binaries)[0] == []
    (finding,), _ = _scan(gate, "x86_64-unknown-linux-gnu and aarch64-apple-darwin\n")
    assert "no release job's set" in finding


@pytest.mark.parametrize(
    "text",
    [
        "ships three per-platform native packages\n",
        "its three `@laterite/native-*` pins\n",
        "the 3 native addons\n",
    ],
)
def test_a_wrong_count_is_caught(gate, text):
    (finding,), _ = _scan(gate, text)
    assert "napi.targets has 4" in finding


def test_a_right_count_and_a_rule_both_pass(gate):
    assert _scan(gate, "the four native packages\n")[0] == []
    assert _scan(gate, "one native package per `napi.targets` entry\n")[0] == []


def test_fenced_code_is_skipped(gate):
    assert _scan(gate, "```\nlinux-x64-gnu darwin-arm64\n```\n")[0] == []


def test_historical_marker_exempts_the_paragraph(gate):
    text = "0.10.0 shipped linux-x64-gnu, darwin-arm64, win32-x64-msvc. <!-- historical -->\n"
    findings, tally = _scan(gate, text)
    assert findings == []
    assert tally["historical"] == 1


def test_matrix_targets_reads_each_job_block(gate):
    workflow = (
        "jobs:\n"
        "  build-wheels:\n    strategy:\n      matrix:\n        include:\n"
        "          - os: a\n            target: aarch64-apple-darwin\n"
        "          # - target: x86_64-apple-darwin\n"
        "  build-binaries:\n    steps:\n      - target: x86_64-pc-windows-msvc\n"
        "        with:\n          target: ${{ matrix.target }}\n"
        "  build-node:\n    x:\n      - target: x86_64-unknown-linux-gnu\n"
    )
    assert gate.matrix_targets(workflow) == {
        "build-wheels": {"aarch64-apple-darwin"},
        "build-binaries": {"x86_64-pc-windows-msvc"},
        "build-node": {"x86_64-unknown-linux-gnu"},
    }


def test_the_real_release_matrix_parses(gate):
    jobs = gate.matrix_targets(gate.RELEASE.read_text(encoding="utf-8"))
    assert all(jobs[j] for j in gate.JOBS)
    assert jobs["build-node"] <= gate.NAPI_NAMES.keys()
