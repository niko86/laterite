"""laterite.transport — zstd + age round-trip tests (content-agnostic, base-only).

transport works on ANY file (#111-B), so these run on a plain AGS4 ``.ags`` file.
They also cover the ``src``/``db`` deprecation shim and the early non-path
``TypeError``.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from laterite import transport

if TYPE_CHECKING:
    from pathlib import Path

_AGS = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","transport test"',
        "",
    ]
).encode("utf-8")


@pytest.fixture
def ags_file(tmp_path: Path) -> Path:
    # transport is content-agnostic — a plain AGS4 file, no format conversion.
    f = tmp_path / "delivery.ags"
    f.write_bytes(_AGS)
    return f


def test_pack_default_dest_on_ags4(ags_file: Path) -> None:
    out = transport.pack(ags_file)
    assert out == ags_file.with_suffix(".ags.zst")  # any-file: <src>.zst
    assert out.exists() and out.stat().st_size > 0
    # zstd magic number: 0x28 B5 2F FD (little-endian).
    assert out.read_bytes()[:4] == bytes.fromhex("28B52FFD")


def test_pack_unpack_round_trip(ags_file: Path, tmp_path: Path) -> None:
    # Folds in the old presence-only `test_pack_explicit_dest`: pack to an EXPLICIT
    # dest, then round-trip through it — a stronger check than "the dest file
    # exists" (it also proves the bytes at that dest actually decompress).
    src_bytes = ags_file.read_bytes()
    dest = tmp_path / "elsewhere.zst"
    zst = transport.pack(ags_file, level=3, dest=dest)
    assert zst == dest and dest.exists()
    restored = tmp_path / "restored.ags"
    transport.unpack(zst, dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unpack_strips_zst_suffix(ags_file: Path) -> None:
    zst = transport.pack(ags_file)
    out = transport.unpack(zst)  # no dest → strips .zst
    assert out == zst.with_suffix("") and out.exists()


#: Cheap scrypt work factor for tests whose subject is NOT the KDF strength.
#: At the shipped factor 18 each derivation is memory-hard by design (256 MiB),
#: which a laptop absorbs and a CI runner does not — these tests ran 27-102s
#: there against under a second locally. The envelope carries its factor in the
#: header, so every property tested here (round-trip, error paths, form parity,
#: zstd levels) is factor-independent. The shipped default still has two
#: round-trip proofs that pay full price on purpose:
#: `test_lock_unlock_round_trip` below, and the docs example `ex17_lock.py`.
#: (Same pattern and constant as test_transport_interop.py's _TEST_LOG_N.)
_TEST_LOG_N = 10


def test_lock_unlock_round_trip(ags_file: Path, tmp_path: Path) -> None:
    # Deliberately at the SHIPPED work factor — the one test here that proves
    # the default tier round-trips, and asserts the header pins factor 18.
    # Every other test in this file dials `log_n` down (see _TEST_LOG_N above).
    src_bytes = ags_file.read_bytes()
    locked = transport.lock(ags_file, password="hunter2")
    assert locked == ags_file.with_suffix(".ags.zst.age")
    assert locked.exists()
    sealed = locked.read_bytes()
    assert sealed != src_bytes  # encrypted, not the plain file
    stanza = next(
        line for line in sealed[:200].split(b"\n") if line.startswith(b"-> scrypt ")
    )
    assert int(stanza.rsplit(b" ", 1)[1]) == 18  # the shipped default tier
    restored = tmp_path / "restored.ags"
    transport.unlock(locked, password="hunter2", dest=restored)
    assert restored.read_bytes() == src_bytes


def test_unlock_wrong_password_raises(ags_file: Path, tmp_path: Path) -> None:
    locked = transport.lock(ags_file, password="correct", log_n=_TEST_LOG_N)
    with pytest.raises(RuntimeError) as exc:
        transport.unlock(locked, password="wrong", dest=tmp_path / "x.ags")
    assert "laterite error" in str(exc.value)


def test_lock_both_levels_work(ags_file: Path, tmp_path: Path) -> None:
    out1 = transport.lock(
        ags_file, password="p", level=1, log_n=_TEST_LOG_N, dest=tmp_path / "l1.age"
    )
    out9 = transport.lock(
        ags_file, password="p", level=9, log_n=_TEST_LOG_N, dest=tmp_path / "l9.age"
    )
    assert out1.exists() and out9.exists()


def test_lock_bytes_honours_log_n_and_still_round_trips() -> None:
    # The `log_n` knob sets the scrypt work factor in the age header. Pin an
    # explicit factor, read it back out of the ASCII `-> scrypt <salt> <log_N>`
    # stanza, and confirm the envelope still opens with our own unlock.
    sealed = transport.lock_bytes(_AGS, password="pw", log_n=12)
    stanza = next(
        line for line in sealed[:200].split(b"\n") if line.startswith(b"-> scrypt ")
    )
    assert int(stanza.rsplit(b" ", 1)[1]) == 12  # the factor we asked for
    assert transport.unlock_bytes(sealed, password="pw") == _AGS
    # The default tier's header pin (18) lives in test_lock_unlock_round_trip,
    # the one test that pays the shipped factor — not duplicated here.


@pytest.mark.parametrize("bad", [0, 21, 25])
def test_lock_rejects_out_of_range_log_n(bad: int) -> None:
    # >20 would make a file the browser age decoder refuses; 0 is invalid. The
    # guard lives once in Rust (encrypt_with_passphrase) so both bytes + file
    # forms inherit it.
    with pytest.raises(RuntimeError, match="log_n must be"):
        transport.lock_bytes(_AGS, password="pw", log_n=bad)


# --- in-memory (bytes) forms ------------------------------------------------


def test_pack_bytes_round_trip() -> None:
    packed = transport.pack_bytes(_AGS * 20)
    assert packed[:4] == bytes.fromhex("28B52FFD")  # zstd magic
    assert len(packed) < len(_AGS * 20)  # repetitive → shrinks
    assert transport.unpack_bytes(packed) == _AGS * 20


def test_lock_bytes_round_trip_and_wrong_password() -> None:
    sealed = transport.lock_bytes(_AGS, password="hunter2", log_n=_TEST_LOG_N)
    assert sealed != _AGS  # encrypted
    assert transport.unlock_bytes(sealed, password="hunter2") == _AGS
    with pytest.raises(RuntimeError):
        transport.unlock_bytes(sealed, password="wrong")


def test_bytes_and_file_forms_interoperate(ags_file: Path, tmp_path: Path) -> None:
    # The parity guarantee: a lock_bytes blob opens with the file unlock, and a
    # file lock opens with unlock_bytes — same envelope either way.
    data = ags_file.read_bytes()

    (tmp_path / "b.zst.age").write_bytes(
        transport.lock_bytes(data, password="pw", log_n=_TEST_LOG_N)
    )
    assert transport.unlock(tmp_path / "b.zst.age", password="pw").read_bytes() == data

    locked = transport.lock(ags_file, password="pw", log_n=_TEST_LOG_N)
    assert transport.unlock_bytes(locked.read_bytes(), password="pw") == data


# --- non-path guard ---------------------------------------------------------


def test_non_path_arg_raises_actionable_typeerror() -> None:
    # Passing e.g. an Ags4File (or any non-path) fails early, naming the fix.
    with pytest.raises(TypeError, match=r"\.save\(path\)"):
        transport.pack(object())
    with pytest.raises(TypeError, match=r"required positional argument: 'src'"):
        transport.pack()
