"""Cross-file merge / append tests.

Two entrypoints support `append=True`:
- `write_ags5db(project, db, append=True)` — pseudo-key dedup against the
  existing DB. Same model graph re-written = same row count.
- `ags4_to_db(ags4, db, append=True)` — content-hash dedup. Same AGS4 file
  re-ingested = same row count; rows present in only one of two files merge in.
"""

from __future__ import annotations

from pathlib import Path

import duckdb
from laterite import GEOL, LLPL, LOCA, PROJ, SAMP, TREG, TREL, TRET
from laterite.ags5db import (
    read_db as read_ags5db,
)
from laterite.ags5db import (
    write_db as write_ags5db,
)


def _project_one() -> PROJ:
    """A small but non-trivial project used as the baseline fixture."""
    samp = SAMP(
        loca_id="BH01", samp_top=5.0, samp_ref="S1",
        samp_type="U", samp_id="SAMP001",
        llpls=[LLPL(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                    samp_type="U", samp_id="SAMP001",
                    spec_ref="S1", spec_dpth=5.0, llpl_ll=45)],
        tregs=[TREG(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                    samp_type="U", samp_id="SAMP001",
                    spec_ref="S1", spec_dpth=5.0, treg_type="CU",
                    trets=[TRET(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                                samp_type="U", samp_id="SAMP001",
                                spec_ref="S1", spec_dpth=5.0,
                                tret_tesn="1",
                                trels=[TREL(loca_id="BH01", samp_top=5.0,
                                            samp_ref="S1", samp_type="U",
                                            samp_id="SAMP001", spec_ref="S1",
                                            spec_dpth=5.0, tret_tesn="1",
                                            trel_mnum=1, trel_cell=350.0)
                                       for _ in range(3)])])])
    loca = LOCA(loca_id="BH01", loca_type="CP", samps=[samp])
    return PROJ(proj_id="MERGE", proj_name="merge test", locas=[loca])


def _project_two_with_extra_loca() -> PROJ:
    """Same PROJ + same LOCA + same SAMP as project_one, plus an extra LOCA
    that isn't in project_one."""
    base = _project_one()
    extra = LOCA(loca_id="BH02", loca_type="TP",
                 geols=[GEOL(loca_id="BH02", geol_top=0.0, geol_base=2.0,
                             geol_desc="Brown CLAY", geol_geol="CLAY")])
    return PROJ(proj_id="MERGE", proj_name="merge test",
                locas=[base.locas[0], extra])


def _row_count(db: Path, table: str) -> int:
    conn = duckdb.connect(str(db), read_only=True)
    try:
        return conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
    finally:
        conn.close()


def test_append_same_project_is_idempotent(tmp_path: Path) -> None:
    """Writing the same model graph twice with append=True yields the same
    row counts as a single write."""
    db = tmp_path / "merge.ags5db"
    write_ags5db(_project_one(), db)
    n_loca_before = _row_count(db, "g_loca")
    n_trel_before = _row_count(db, "g_trel")

    write_ags5db(_project_one(), db, append=True)

    assert _row_count(db, "g_proj") == 1
    assert _row_count(db, "g_loca") == n_loca_before
    assert _row_count(db, "g_samp") == 1
    assert _row_count(db, "g_trel") == n_trel_before


def test_append_adds_new_rows_only(tmp_path: Path) -> None:
    """A second project with one extra LOCA appends only that LOCA's rows."""
    db = tmp_path / "merge.ags5db"
    write_ags5db(_project_one(), db)
    write_ags5db(_project_two_with_extra_loca(), db, append=True)

    assert _row_count(db, "g_proj") == 1, "PROJ pseudo-key matches; one row"
    assert _row_count(db, "g_loca") == 2, "BH01 reused, BH02 new"
    assert _row_count(db, "g_samp") == 1, "BH01's sample is unchanged"
    assert _row_count(db, "g_geol") == 1, "BH02's geology is the only GEOL row"


def test_append_preserves_existing_uuids(tmp_path: Path) -> None:
    """A LOCA already on disk keeps its UUID across the append; children
    written in the second pass attach to the same UUID."""
    db = tmp_path / "merge.ags5db"
    write_ags5db(_project_one(), db)
    conn = duckdb.connect(str(db), read_only=True)
    try:
        loca_uuid_before = conn.execute(
            "SELECT id FROM g_loca WHERE loca_id = 'BH01'").fetchone()[0]
    finally:
        conn.close()

    write_ags5db(_project_two_with_extra_loca(), db, append=True)

    conn = duckdb.connect(str(db), read_only=True)
    try:
        loca_uuid_after = conn.execute(
            "SELECT id FROM g_loca WHERE loca_id = 'BH01'").fetchone()[0]
    finally:
        conn.close()
    assert loca_uuid_before == loca_uuid_after


def test_append_round_trips_via_read_ags5db(tmp_path: Path) -> None:
    """The merged DB rebuilds into a model graph with the union of both writes."""
    db = tmp_path / "merge.ags5db"
    write_ags5db(_project_one(), db)
    write_ags5db(_project_two_with_extra_loca(), db, append=True)

    rebuilt = read_ags5db(db)
    loca_ids = sorted(loc.loca_id for loc in rebuilt.locas)
    assert loca_ids == ["BH01", "BH02"]
    bh02 = next(loc for loc in rebuilt.locas if loc.loca_id == "BH02")
    assert len(bh02.geols) == 1


# `test_compact_shrinks_file_and_preserves_data` retired with F2c-3:
# `laterite.ags5db.write_db` always compacts (no compact=False knob), so
# the side-by-side comparison can't be expressed against the new writer.


# Test lives under packages/laterite-ags5/tests/ — repo root is three
# parents up.
# Committed synthetic multi-group fixture — always present, so this
# real-file append-idempotence check runs in CI (the old git-ignored
# large.ags made it silently skip).
_FIXTURE = (
    Path(__file__).resolve().parents[3]
    / "rust-packages" / "ags4-validator" / "tests" / "fixtures"
    / "synthetic_multigroup.ags"
)


def test_ags4_append_is_idempotent_on_synthetic_file(tmp_path: Path) -> None:
    """Re-ingesting the same AGS4 file with append=True must not duplicate rows.
    Content-hash dedup is the only way to get this right for groups whose
    auto-scaffolded KEYs aren't uniquely identifying (e.g. the many TREL rows
    sharing LOCA/SAMP keys)."""
    from laterite.ags5db import convert

    db = tmp_path / "merge.ags5db"
    convert(_FIXTURE, db)
    before: dict[str, int] = {}
    conn = duckdb.connect(str(db), read_only=True)
    try:
        for code in ("g_loca", "g_samp", "g_geol", "g_core",
                     "g_llpl", "g_treg", "g_tret", "g_trel"):
            before[code] = conn.execute(f"SELECT COUNT(*) FROM {code}").fetchone()[0]
    finally:
        conn.close()

    convert(_FIXTURE, db, append=True)
    after: dict[str, int] = {}
    conn = duckdb.connect(str(db), read_only=True)
    try:
        for code in before:
            after[code] = conn.execute(f"SELECT COUNT(*) FROM {code}").fetchone()[0]
    finally:
        conn.close()

    assert before == after, f"row counts changed after re-ingest: {before} -> {after}"
