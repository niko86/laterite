"""Base read-path performance baselines via pytest-benchmark.

Tracks the operations the 1.0 Arrow engine redesign changes (see
``redacted-wiki/design/api-surface-1.0.md``): cold read, group-frame access,
numeric coercion, and validation (Rust-on-text — a control that stays
unchanged). They don't assert wall-clock (machines vary); pytest-benchmark
records timings and ``--benchmark-save`` / ``--benchmark-compare`` flag
regressions. Per pyproject's ``--benchmark-disable`` the bodies also run once
as plain functional tests in CI.

Phase-1 result (2000-row synthetic, release, median, vs the v0.2.0 primitives
path) — the Arrow boundary + the Rust write-back handle:
    group-access   404us ->  26us   (~16x; zero-copy capsule, was per-cell)
    read-cold      864us -> 660us   (~1.3x; parse+Arrow replaced the O(cells) dict)
    validate      1161us -> 1169us  (unchanged control)

Named test_perf_read.py (distinct from other per-surface benchmarks) —
pytest collects them in one session and duplicate basenames would clash. Run (use --benchmark-enable, since
pyproject sets --benchmark-disable in addopts; --benchmark-only conflicts):
    uv run pytest packages/laterite/tests/test_perf_read.py --benchmark-enable
    uv run pytest .../test_perf_read.py --benchmark-enable --benchmark-save=phase-1
    uv run pytest .../test_perf_read.py --benchmark-enable --benchmark-compare=phase-1
"""

from __future__ import annotations

from pathlib import Path

import laterite
import pytest

# A self-contained synthetic AGS4 file — meaningful size, no gitignored fixture, so
# CI runs it. ~2000 LOCA rows with two numeric (2DP) columns for the numeric path.
_N_ROWS = 2000


def _synthetic_ags(n_rows: int) -> str:
    head = "\r\n".join(
        [
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID","PROJ_NAME"',
            '"UNIT","",""',
            '"TYPE","ID","X"',
            '"DATA","BENCH","perf baseline"',
            '"GROUP","LOCA"',
            '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE","LOCA_FDEP"',
            '"UNIT","","","m","m"',
            '"TYPE","ID","PA","2DP","2DP"',
        ]
    )
    rows = "".join(
        f'\r\n"DATA","BH{i:05d}","CP","{523000 + i}.20","{(i % 50) + 0.25:.2f}"'
        for i in range(n_rows)
    )
    return head + rows + "\r\n"


@pytest.fixture(scope="module")
def ags_path(tmp_path_factory) -> Path:
    p = tmp_path_factory.mktemp("perf") / "synthetic.ags"
    p.write_text(_synthetic_ags(_N_ROWS), encoding="utf-8")
    return p


@pytest.fixture(scope="module")
def parsed(ags_path: Path) -> laterite.Ags4File:
    return laterite.read(str(ags_path))


@pytest.mark.benchmark
def test_read_cold_baseline(benchmark, ags_path: Path):
    """Cold read = full parse into an Ags4File. Phase 1: parse + typed Arrow
    build, with the raw parse kept Rust-side (the O(cells) primitives dict
    retired) — faster than the v0.2.0 primitives path."""
    benchmark(lambda: laterite.read(str(ags_path)))


@pytest.mark.benchmark
def test_group_access_baseline(benchmark, parsed: laterite.Ags4File):
    """Building one group's frame — now a zero-copy ingest of the Rust-built
    Arrow table (replacing the per-cell string-frame path); born-typed."""
    benchmark(lambda: parsed["LOCA"])


@pytest.mark.benchmark
def test_validate_baseline(benchmark, ags_path: Path):
    """Validation is Rust-on-text and stays unchanged at 1.0 — a control number."""
    benchmark(lambda: laterite.validate(str(ags_path)))


_LARGE = Path(__file__).resolve().parents[3] / "examples" / "output" / "large.ags"


@pytest.mark.benchmark
@pytest.mark.skipif(not _LARGE.exists(), reason="examples/output/large.ags absent")
def test_read_large_ags_baseline(benchmark):
    """Optional richer baseline on the real 23 MB / 69-group file (local only)."""
    benchmark(lambda: laterite.read(str(_LARGE)))
