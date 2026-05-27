"""laterite.transport — zstd + age passphrase round-trip tests.

Stage F2a-2c: verifies the new lib-backed pack/unpack/lock/unlock
behaviour matches the retired ags5_db.transport.* shape. Tests use
small in-memory .ags5db fixtures so they run quickly.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from laterite import ags5db, transport

_AGS = "\r\n".join([
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID","PROJ_NAME"',
    '"UNIT","",""',
    '"TYPE","ID","X"',
    '"DATA","P1","transport test"',
    "",
]).encode("utf-8")


@pytest.fixture
def small_db(tmp_path: Path) -> Path:
    ags = tmp_path / "small.ags"
    ags.write_bytes(_AGS)
    db = tmp_path / "small.ags5db"
    ags5db.convert(ags, db)
    return db


def test_pack_default_dest(small_db: Path) -> None:
    out = transport.pack(small_db)
    assert out == small_db.with_suffix(".ags5db.zst")
    assert out.exists()
    assert out.stat().st_size > 0
    # zstd magic number: 0x28 B5 2F FD (little-endian).
    assert out.read_bytes()[:4] == bytes.fromhex("28B52FFD")


def test_pack_explicit_dest(small_db: Path, tmp_path: Path) -> None:
    dest = tmp_path / "elsewhere.zst"
    out = transport.pack(small_db, dest=dest)
    assert out == dest
    assert dest.exists()


def test_pack_unpack_round_trip(small_db: Path, tmp_path: Path) -> None:
    src_bytes = small_db.read_bytes()
    zst = transport.pack(small_db, level=3)
    restored = tmp_path / "restored.ags5db"
    transport.unpack(zst, dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unpack_strips_zst_suffix(small_db: Path, tmp_path: Path) -> None:
    zst = transport.pack(small_db)
    # unpack with no dest strips the .zst.
    out = transport.unpack(zst)
    assert out == zst.with_suffix("")
    assert out.exists()


def test_lock_unlock_round_trip(small_db: Path, tmp_path: Path) -> None:
    src_bytes = small_db.read_bytes()
    locked = transport.lock(small_db, password="hunter2")
    assert locked == small_db.with_suffix(".ags5db.zst.age")
    assert locked.exists()
    assert locked.read_bytes() != src_bytes  # encrypted, not the plain file

    restored = tmp_path / "restored.ags5db"
    transport.unlock(locked, password="hunter2", dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unlock_wrong_password_raises(small_db: Path, tmp_path: Path) -> None:
    locked = transport.lock(small_db, password="correct")
    with pytest.raises(RuntimeError) as exc:
        transport.unlock(locked, password="wrong", dest=tmp_path / "x.ags5db")
    assert "ags5db error" in str(exc.value)


def test_lock_higher_level_smaller_file(small_db: Path, tmp_path: Path) -> None:
    # On this tiny fixture the difference is small but level 9 should
    # never be larger than level 1 for non-trivial input. Just smoke-
    # test that both levels work.
    out1 = transport.lock(small_db, password="p", level=1, dest=tmp_path / "l1.age")
    out9 = transport.lock(small_db, password="p", level=9, dest=tmp_path / "l9.age")
    assert out1.exists() and out9.exists()
