"""Tests for the blob storage layer (CorePhotos + other binary attachments).

F2c-2: migrated to `laterite.ags5db.{write_db, attach_blobs,
BlobAttachment}`. The blob side-channel is now a two-step flow —
`write_db(proj, db)` lays down the structured rows, then
`attach_blobs(db, [...])` resolves target UUIDs via the file's views
and bulk-inserts into the `blob` table.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

import duckdb
import pytest
from laterite import CORE, GEOL, LOCA, PROJ, SAMP, TREG, TREL, TRET
from laterite.ags5db import (
    BlobAttachment,
    attach_blobs,
    list_blobs,
    write_db,
)


def _project_with_one_core_and_some_trel(n_readings: int = 10) -> PROJ:
    """A minimal PROJ with one CORE and one TREL chain so we can attach
    photos to the CORE and exercise the L-group raw CSV path."""
    keys = dict(loca_id="BH01", samp_top=5.0, samp_ref="S1",
                samp_type="U", samp_id="SAMP001")
    spec = {**keys, "spec_ref": "S1", "spec_dpth": 5.0}
    readings = [
        TREL(**spec, tret_tesn="1", trel_mnum=i + 1, trel_cell=350.0 + i)
        for i in range(n_readings)
    ]
    tret = TRET(**spec, tret_tesn="1", trels=readings)
    treg = TREG(**spec, treg_type="CU", trets=[tret])
    samp = SAMP(**keys, llpls=[], tregs=[treg])
    core = CORE(loca_id="BH01", core_top=0.0, core_base=5.0, core_rqd=85)
    geol = GEOL(loca_id="BH01", geol_top=0.0, geol_base=5.0,
                geol_desc="CLAY", geol_geol="CLAY")
    loca = LOCA(loca_id="BH01", loca_type="CP",
                cores=[core], geols=[geol], samps=[samp])
    return PROJ(proj_id="BLOB_TEST", proj_name="blob test", locas=[loca])


class TestBlobAttachmentBasics:
    def test_from_model_introspects_keys(self):
        core = CORE(loca_id="BH01", core_top=5.0, core_base=10.0)
        blob = BlobAttachment.from_model(
            core, kind="photo", data=b"fake-jpeg", filename="bh01_5m.jpg",
        )
        assert blob.target_code == "CORE"
        assert blob.target_keys == {"LOCA_ID": "BH01", "CORE_TOP": 5.0}
        assert blob.kind == "photo"
        assert blob.data == b"fake-jpeg"
        assert blob.mime_type == "image/jpeg"
        assert blob.filename == "bh01_5m.jpg"

    def test_from_model_rejects_unregistered_class(self):
        class NotAGroup:
            pass
        with pytest.raises(ValueError, match="not a registered AGS group"):
            BlobAttachment.from_model(
                NotAGroup(), kind="photo", data=b"", filename=None,
            )


class TestPhotoAttachRoundTrip:
    def test_attach_and_retrieve_photo(self, tmp_path: Path):
        proj = _project_with_one_core_and_some_trel()
        core = proj.locas[0].cores[0]
        photo_bytes = b"\xff\xd8\xff\xe0" + b"jpegdata" * 100
        blob = BlobAttachment.from_model(
            core, kind="photo", data=photo_bytes, filename="bh01_top.jpg",
        )

        db_path = tmp_path / "photo.ags5db"
        write_db(proj, db_path)
        n = attach_blobs(db_path, [blob])
        assert n == 1

        retrieved = list_blobs(db_path, parent_code="CORE", kind="photo")
        assert len(retrieved) == 1
        rec = retrieved[0]
        assert rec["mime_type"] == "image/jpeg"
        assert rec["filename"] == "bh01_top.jpg"
        # `list_blobs` returns metadata only; fetch bytes via DuckDB direct
        # since laterite.ags5db.sql doesn't bind parameters.
        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            (data,) = conn.execute(
                "SELECT data FROM blob WHERE id = ?", [rec["id"]],
            ).fetchone()
        finally:
            conn.close()
        assert bytes(data) == photo_bytes

    def test_sha256_persisted(self, tmp_path: Path):
        proj = _project_with_one_core_and_some_trel()
        core = proj.locas[0].cores[0]
        data = b"some-bytes"
        expected_sha = hashlib.sha256(data).hexdigest()

        blob = BlobAttachment.from_model(core, kind="photo", data=data, filename="x.png")
        db_path = tmp_path / "sha.ags5db"
        write_db(proj, db_path)
        attach_blobs(db_path, [blob])

        conn = duckdb.connect(str(db_path), read_only=True)
        try:
            row = conn.execute("SELECT sha256 FROM blob LIMIT 1").fetchone()
        finally:
            conn.close()
        assert row[0] == expected_sha

    def test_unknown_target_keys_raises(self, tmp_path: Path):
        proj = _project_with_one_core_and_some_trel()
        bogus = BlobAttachment(
            target_code="CORE",
            target_keys={"LOCA_ID": "BH99", "CORE_TOP": 0.0},  # BH99 doesn't exist
            kind="photo",
            data=b"x",
        )
        db_path = tmp_path / "missing.ags5db"
        write_db(proj, db_path)
        with pytest.raises(ValueError, match="no CORE row matches"):
            attach_blobs(db_path, [bogus])


class TestListBlobsFilters:
    @pytest.fixture()
    def db_with_blobs(self, tmp_path: Path) -> Path:
        proj = _project_with_one_core_and_some_trel()
        core = proj.locas[0].cores[0]
        samp = proj.locas[0].samps[0]
        blobs = [
            BlobAttachment.from_model(core, kind="photo", data=b"jpg-1", filename="a.jpg"),
            BlobAttachment.from_model(core, kind="photo", data=b"jpg-2", filename="b.jpg"),
            BlobAttachment.from_model(samp, kind="other", data=b"meta", filename="m.txt"),
        ]
        db_path = tmp_path / "filters.ags5db"
        write_db(proj, db_path)
        attach_blobs(db_path, blobs)
        return db_path

    def test_no_filter_returns_all(self, db_with_blobs: Path):
        assert len(list_blobs(db_with_blobs)) == 3

    def test_filter_by_parent_code(self, db_with_blobs: Path):
        cores = list_blobs(db_with_blobs, parent_code="CORE")
        assert len(cores) == 2
        assert all(b["parent_code"] == "CORE" for b in cores)

    def test_filter_by_kind(self, db_with_blobs: Path):
        photos = list_blobs(db_with_blobs, kind="photo")
        assert len(photos) == 2
        others = list_blobs(db_with_blobs, kind="other")
        assert len(others) == 1
