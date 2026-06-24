"""laterite.transport — zstd + age round-trip tests (content-agnostic, base-only).

transport works on ANY file (#111-B), so these run on a plain AGS4 ``.ags`` file and
need no ``.ags5db`` / ``[ags5]`` surface. They also cover the ``src``/``db``
deprecation shim and the early non-path ``TypeError``.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from laterite import transport

_AGS = "\r\n".join([
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID","PROJ_NAME"',
    '"UNIT","",""',
    '"TYPE","ID","X"',
    '"DATA","P1","transport test"',
    "",
]).encode("utf-8")


@pytest.fixture
def ags_file(tmp_path: Path) -> Path:
    # transport is content-agnostic — a plain AGS4 file, no .ags5db conversion.
    f = tmp_path / "delivery.ags"
    f.write_bytes(_AGS)
    return f


def test_pack_default_dest_on_ags4(ags_file: Path) -> None:
    out = transport.pack(ags_file)
    assert out == ags_file.with_suffix(".ags.zst")   # any-file: <src>.zst
    assert out.exists() and out.stat().st_size > 0
    # zstd magic number: 0x28 B5 2F FD (little-endian).
    assert out.read_bytes()[:4] == bytes.fromhex("28B52FFD")


def test_pack_explicit_dest(ags_file: Path, tmp_path: Path) -> None:
    dest = tmp_path / "elsewhere.zst"
    out = transport.pack(ags_file, dest=dest)
    assert out == dest and dest.exists()


def test_pack_unpack_round_trip(ags_file: Path, tmp_path: Path) -> None:
    src_bytes = ags_file.read_bytes()
    zst = transport.pack(ags_file, level=3)
    restored = tmp_path / "restored.ags"
    transport.unpack(zst, dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unpack_strips_zst_suffix(ags_file: Path) -> None:
    zst = transport.pack(ags_file)
    out = transport.unpack(zst)  # no dest → strips .zst
    assert out == zst.with_suffix("") and out.exists()


def test_lock_unlock_round_trip(ags_file: Path, tmp_path: Path) -> None:
    src_bytes = ags_file.read_bytes()
    locked = transport.lock(ags_file, password="hunter2")
    assert locked == ags_file.with_suffix(".ags.zst.age")
    assert locked.exists()
    assert locked.read_bytes() != src_bytes  # encrypted, not the plain file
    restored = tmp_path / "restored.ags"
    transport.unlock(locked, password="hunter2", dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unlock_wrong_password_raises(ags_file: Path, tmp_path: Path) -> None:
    locked = transport.lock(ags_file, password="correct")
    with pytest.raises(RuntimeError) as exc:
        transport.unlock(locked, password="wrong", dest=tmp_path / "x.ags")
    assert "laterite error" in str(exc.value)


def test_lock_both_levels_work(ags_file: Path, tmp_path: Path) -> None:
    out1 = transport.lock(ags_file, password="p", level=1, dest=tmp_path / "l1.age")
    out9 = transport.lock(ags_file, password="p", level=9, dest=tmp_path / "l9.age")
    assert out1.exists() and out9.exists()


# --- non-path guard ---------------------------------------------------------

def test_non_path_arg_raises_actionable_typeerror() -> None:
    # Passing e.g. an Ags4File (or any non-path) fails early, naming the fix.
    with pytest.raises(TypeError, match=r"\.save\(path\)"):
        transport.pack(object())
    with pytest.raises(TypeError, match=r"required positional argument: 'src'"):
        transport.pack()
