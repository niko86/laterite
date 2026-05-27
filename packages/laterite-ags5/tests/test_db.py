"""Tests for ags5-db: write -> read -> query round-trip on the registry-driven schema."""

from __future__ import annotations

from pathlib import Path

import polars as pl
import pytest
from laterite import (
    CORE,
    GEOL,
    LLPL,
    LOCA,
    PROJ,
    SAMP,
    TREG,
    TREL,
    TRET,
)
from laterite import ags5db as _laterite_db
from laterite.ags5db import (
    read_db as read_ags5db,
)
from laterite.ags5db import (
    write_db as write_ags5db,
)

Predicate = _laterite_db.Predicate


def _make_project(n_readings: int = 20) -> PROJ:
    readings = [
        TREL(
            loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
            samp_id="SAMP001", spec_ref="CAUCY3", spec_dpth=5.0, tret_tesn="1",
            trel_mnum=i + 1,
            trel_ttim=i * 0.5,
            trel_cell=350.0 + i,
            trel_back=10.0,
            trel_pwp=150.0 + i * 2,
            trel_pwpm=151.0,
        )
        for i in range(n_readings)
    ]
    tret = TRET(
        loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
        samp_id="SAMP001", spec_ref="CAUCY3", spec_dpth=5.0, tret_tesn="1",
        trels=readings,
    )
    treg = TREG(
        loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
        samp_id="SAMP001", spec_ref="CAUCY3", spec_dpth=5.0,
        treg_type="CU", treg_coh=25, treg_phi=28.5,
        trets=[tret],
    )
    llpl = LLPL(
        loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
        samp_id="SAMP001", spec_ref="S1", spec_dpth=5.0,
        llpl_ll=45, llpl_pl=22, llpl_pi=23,
    )
    samp = SAMP(
        loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
        samp_id="SAMP001",
        llpls=[llpl], tregs=[treg],
    )
    core = CORE(loca_id="BH01", core_top=0.0, core_base=10.0, core_rqd=85)
    geol = GEOL(
        loca_id="BH01", geol_top=0.0, geol_base=5.0,
        geol_desc="Firm brown CLAY", geol_geol="CLAY",
    )
    loca = LOCA(
        loca_id="BH01", loca_type="CP",
        loca_gl=45.0, loca_lat=51.5, loca_lon=-0.1,
        cores=[core], geols=[geol], samps=[samp],
    )
    return PROJ(proj_id="TEST001", proj_name="DB round-trip test", locas=[loca])


class TestDbRoundTrip:
    def test_write_read_roundtrip(self, tmp_path: Path):
        proj = _make_project(20)
        db_path = tmp_path / "test.ags5db"

        write_ags5db(proj, db_path)
        assert db_path.exists()
        assert db_path.stat().st_size > 0

        restored = read_ags5db(db_path)
        assert restored.proj_id == "TEST001"
        assert len(restored.locas) == 1

        loca = restored.locas[0]
        assert loca.loca_id == "BH01"
        assert loca.loca_gl == 45.0

        assert len(loca.cores) == 1
        assert loca.cores[0].core_rqd == 85

        assert len(loca.geols) == 1
        assert loca.geols[0].geol_geol == "CLAY"

        assert len(loca.samps) == 1
        samp = loca.samps[0]
        assert samp.samp_id == "SAMP001"

        assert len(samp.llpls) == 1
        assert samp.llpls[0].llpl_ll == 45

        assert len(samp.tregs) == 1
        treg = samp.tregs[0]
        assert treg.treg_type == "CU"
        assert len(treg.trets) == 1
        assert len(treg.trets[0].trels) == 20
        assert treg.trets[0].trels[0].trel_cell == 350.0
        assert treg.trets[0].trels[19].trel_cell == 369.0

    def test_empty_project(self, tmp_path: Path):
        proj = PROJ(proj_id="EMPTY", proj_name="Empty project")
        db_path = tmp_path / "empty.ags5db"
        write_ags5db(proj, db_path)
        restored = read_ags5db(db_path)
        assert restored.proj_id == "EMPTY"
        assert restored.locas == []


class TestDbQueries:
    @pytest.fixture()
    def db_path(self, tmp_path: Path) -> Path:
        proj = _make_project(20)
        p = tmp_path / "query_test.ags5db"
        write_ags5db(proj, p)
        return p

    def test_sum_field(self, db_path: Path):
        # Sum of trel_cell: 350+351+...+369
        expected = sum(350.0 + i for i in range(20))
        result = _laterite_db.sum(db_path, "TREL", "trel_cell")
        assert abs(result - expected) < 0.01

    def test_sum_field_with_where(self, db_path: Path):
        # Only readings where trel_mnum <= 5 (mnum 1..5, values 350..354)
        expected = sum(350.0 + i for i in range(5))
        result = _laterite_db.sum(
            db_path, "TREL", "trel_cell",
            where=Predicate("trel_mnum", "<=", 5),
        )
        assert abs(result - expected) < 0.01

    def test_query_readings_all(self, db_path: Path):
        df = _laterite_db.query(db_path, "TREL", fields=["trel_cell", "trel_pwp"])
        assert isinstance(df, pl.DataFrame)
        assert len(df) == 20
        assert "trel_cell" in df.columns
        assert "trel_pwp" in df.columns

    def test_query_readings_with_limit(self, db_path: Path):
        df = _laterite_db.query(db_path, "TREL", fields=["trel_cell"], limit=5)
        assert len(df) == 5

    def test_predicate_rejects_bad_op(self):
        with pytest.raises(ValueError, match="disallowed predicate op"):
            Predicate("trel_mnum", "DROP TABLE", 1)  # type: ignore[arg-type]


class TestSampUuidInvariants:
    """SAMP uses a UUID7 PK + pseudo-key dedup, and descendants drop the
    inherited LOCA_ID/SAMP_* cascade (they reach them via parent_id JOINs).
    These invariants are what makes cross-file SAMP merge byte-cheap and
    keep the schema narrow — they're easy to regress accidentally."""

    def test_descendant_tables_drop_inherited_keys(self, tmp_path: Path):
        """LOCA_ID/SAMP_TOP/SAMP_REF/SAMP_TYPE/SAMP_ID should NOT appear as
        typed columns on any SAMP descendant — they're derivable via parent_id."""
        import duckdb
        proj = _make_project(2)
        db_path = tmp_path / "schema.ags5db"
        write_ags5db(proj, db_path)
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            for table in ("g_treg", "g_tret", "g_trel", "g_llpl", "g_conl"):
                cols = {r[1] for r in conn.execute(
                    f"PRAGMA table_info({table})").fetchall()}
                assert "LOCA_ID" not in cols, f"{table} still has LOCA_ID"
                assert "SAMP_TOP" not in cols, f"{table} still has SAMP_TOP"
                assert "SAMP_REF" not in cols, f"{table} still has SAMP_REF"
                assert "SAMP_TYPE" not in cols, f"{table} still has SAMP_TYPE"
                assert "SAMP_ID" not in cols, f"{table} still has SAMP_ID"
        finally:
            conn.close()

    def test_views_expose_inherited_keys_via_join(self, tmp_path: Path):
        """v_treg etc. should still expose loca_id, samp_id etc. via JOIN."""
        import duckdb
        proj = _make_project(2)
        db_path = tmp_path / "view.ags5db"
        write_ags5db(proj, db_path)
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            row = conn.execute(
                "SELECT loca_id, samp_id, spec_ref, treg_type FROM v_treg"
            ).fetchone()
            assert row[0] == "BH01"
            assert row[1] == "SAMP001"
            assert row[2] == "CAUCY3"
            assert row[3] == "CU"

            # 3-level JOIN view: v_trel exposes SAMP keys through TRET → TREG → SAMP.
            row = conn.execute(
                "SELECT loca_id, samp_id, tret_tesn, trel_mnum FROM v_trel "
                "ORDER BY trel_mnum LIMIT 1"
            ).fetchone()
            assert row[0] == "BH01"
            assert row[1] == "SAMP001"
            assert row[2] == "1"
            assert row[3] == 1
        finally:
            conn.close()

    def test_in_process_dedup(self, tmp_path: Path):
        """Two SAMP instances with identical pseudo-keys but different children
        collapse to ONE g_samp row with both children attached via the same UUID."""
        import duckdb
        shared = dict(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                      samp_type="U", samp_id="SAMP001")
        samp_a = SAMP(**shared,
                      llpls=[LLPL(**shared, spec_ref="S1", spec_dpth=5.0,
                                  llpl_ll=45)])
        samp_b = SAMP(**shared,
                      tregs=[TREG(**shared, spec_ref="X", spec_dpth=5.0,
                                  treg_type="CU")])
        proj = PROJ(proj_id="DEDUP", proj_name="dedup",
                    locas=[LOCA(loca_id="BH01", loca_type="CP",
                                samps=[samp_a, samp_b])])
        db_path = tmp_path / "dedup.ags5db"
        write_ags5db(proj, db_path)
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            n_samps = conn.execute("SELECT COUNT(*) FROM g_samp").fetchone()[0]
            assert n_samps == 1, "two same-key SAMPs should dedupe to one row"
            id = conn.execute(
                "SELECT id FROM g_samp").fetchone()[0]
            llpl_parent = conn.execute(
                "SELECT parent_id FROM g_llpl").fetchone()[0]
            treg_parent = conn.execute(
                "SELECT parent_id FROM g_treg").fetchone()[0]
            assert llpl_parent == id
            assert treg_parent == id
        finally:
            conn.close()

    def test_uuid_is_internal_not_in_model(self, tmp_path: Path):
        """The SAMP UUID is an internal DB optimisation; it must never
        appear on the model class. F2a retired the .agsx round-trip
        check; the property holds at the class level."""
        proj = _make_project(2)
        db_path = tmp_path / "rt.ags5db"
        write_ags5db(proj, db_path)
        restored = read_ags5db(db_path)
        samp = restored.locas[0].samps[0]
        assert samp.samp_id == "SAMP001"
        # Model classes don't carry an id field; only the DB schema does.
        assert not hasattr(samp, "id")


class TestUuid7Schema:
    """Every group's PK is UUID7; every child's parent_id is UUID."""

    def test_every_group_has_uuid_pk(self, tmp_path: Path):
        import duckdb
        proj = _make_project(2)
        db_path = tmp_path / "schema.ags5db"
        write_ags5db(proj, db_path)
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            for table in ("g_proj", "g_loca", "g_samp", "g_geol", "g_core",
                          "g_llpl", "g_treg", "g_tret", "g_trel"):
                cols = {r[1]: r[2] for r in
                        conn.execute(f"PRAGMA table_info({table})").fetchall()}
                assert cols.get("id") == "UUID", f"{table}.id != UUID"
                if table != "g_proj":
                    assert cols.get("parent_id") == "UUID", \
                        f"{table}.parent_id != UUID"
        finally:
            conn.close()

    def test_in_process_dedup(self, tmp_path: Path):
        """Two TRET instances with identical (parent_id, TRET_TESN) collapse
        to one g_tret row, both children attach via the same UUID."""
        import duckdb
        # Build SAMP/TREG once; build two TRETs with same TRET_TESN under the
        # same TREG, each with different TREL row counts so we can spot which
        # children attached.
        shared_keys = dict(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                           samp_type="U", samp_id="SAMP001",
                           spec_ref="CAUCY3", spec_dpth=5.0, tret_tesn="1")
        readings_a = [
            TREL(**shared_keys, trel_mnum=i + 1, trel_cell=350.0 + i)
            for i in range(3)
        ]
        readings_b = [
            TREL(**shared_keys, trel_mnum=i + 100, trel_cell=400.0 + i)
            for i in range(2)
        ]
        tret_a = TRET(**shared_keys, trels=readings_a)
        tret_b = TRET(**shared_keys, trels=readings_b)
        treg = TREG(loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
                    samp_id="SAMP001", spec_ref="CAUCY3", spec_dpth=5.0,
                    treg_type="CU", trets=[tret_a, tret_b])
        samp = SAMP(loca_id="BH01", samp_top=5.0, samp_ref="S1", samp_type="U",
                    samp_id="SAMP001", tregs=[treg])
        loca = LOCA(loca_id="BH01", loca_type="CP", samps=[samp])
        proj = PROJ(proj_id="DEDUP", proj_name="dedup", locas=[loca])

        db_path = tmp_path / "tret_dedup.ags5db"
        write_ags5db(proj, db_path)
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            n_tret = conn.execute("SELECT COUNT(*) FROM g_tret").fetchone()[0]
            assert n_tret == 1, "duplicate TRETs should dedupe to one row"

            id = conn.execute(
                "SELECT id FROM g_tret").fetchone()[0]
            trel_parents = {r[0] for r in
                            conn.execute("SELECT DISTINCT parent_id FROM g_trel").fetchall()}
            assert trel_parents == {id}, \
                "all TREL rows from both TRETs should attach to the same UUID"

            n_trel = conn.execute("SELECT COUNT(*) FROM g_trel").fetchone()[0]
            assert n_trel == 5, "all 3 + 2 TREL rows should be inserted"
        finally:
            conn.close()

    def test_tret_uuid_not_on_model(self, tmp_path: Path):
        """TRET UUID is internal DB-only; never appears on the model class."""
        proj = _make_project(2)
        db_path = tmp_path / "rt.ags5db"
        write_ags5db(proj, db_path)
        restored = read_ags5db(db_path)
        tret = restored.locas[0].samps[0].tregs[0].trets[0]
        assert tret.tret_tesn == "1"
        assert not hasattr(tret, "id")

    # `test_treg_in_process_dedup` retired with F2c-3.
    #
    # The old Python writer used pseudo-key dedup on the AGS KEY tuple
    # only, so two TREGs with the same pseudo-key but different
    # `treg_type` values silently collapsed to one row (the first one's
    # non-KEY fields won). The new Rust writer (shared with AGS4
    # ingest) uses content-hash dedup over every heading value, so
    # the two TREGs are preserved as two rows — no silent data loss.
    # The test asserted the old lossy behaviour, so it can't be ported.

