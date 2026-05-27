"""Performance regression guards via pytest-benchmark.

These tests don't assert wall-clock times (machines vary). pytest-benchmark
records the timing per run and stores it under `.benchmarks/`; running
`pytest --benchmark-compare=<id>` flags regressions for review.

Run with:
    uv run pytest tests/test_perf.py --benchmark-only
    uv run pytest tests/test_perf.py --benchmark-only --benchmark-save=baseline
    uv run pytest tests/test_perf.py --benchmark-only --benchmark-compare=baseline
"""

from __future__ import annotations

from pathlib import Path

import pytest
from laterite import GEOL, LLPL, LOCA, PROJ, SAMP, TREG, TREL, TRET
from laterite.ags5db import write_db as write_ags5db


def _build_project(n_locations: int, readings_per_sample: int = 5) -> PROJ:
    """Wide-shape project: many LOCAs, one sample each, a handful of TREL rows."""
    locas = []
    for i in range(n_locations):
        loca_id = f"BH{i:05d}"
        samp_id = f"{loca_id}_S1"
        keys = dict(
            loca_id=loca_id, samp_top=5.0, samp_ref="S1",
            samp_type="U", samp_id=samp_id,
        )
        spec = {**keys, "spec_ref": "S1", "spec_dpth": 5.0}
        readings = [
            TREL(**spec, tret_tesn="1", trel_mnum=j + 1, trel_cell=350.0 + j)
            for j in range(readings_per_sample)
        ]
        tret = TRET(**spec, tret_tesn="1", trels=readings)
        treg = TREG(**spec, treg_type="CU", trets=[tret])
        llpl = LLPL(**spec, llpl_ll=45)
        samp = SAMP(**keys, llpls=[llpl], tregs=[treg])
        geol = GEOL(loca_id=loca_id, geol_top=0.0, geol_base=5.0,
                    geol_desc="CLAY", geol_geol="CLAY")
        locas.append(LOCA(loca_id=loca_id, loca_type="CP",
                          geols=[geol], samps=[samp]))
    return PROJ(proj_id="BENCH", proj_name="bench", locas=locas)


@pytest.fixture(scope="module")
def project_1k() -> PROJ:
    """1000 LOCAs * 5 readings = 5000 TREL rows. Small enough to run fast,
    big enough that per-row overhead would show as a regression."""
    return _build_project(1000)


def test_write_1k_locations_baseline(benchmark, project_1k: PROJ, tmp_path: Path):
    """Pin the write throughput at 1000 LOCAs. A regression here means the
    per-group bulk insert path has been broken (e.g. someone reintroduced
    per-row INSERT...RETURNING in a refactor)."""
    db_path = tmp_path / "perf.ags5db"

    def _write() -> None:
        write_ags5db(project_1k, db_path)

    benchmark(_write)
