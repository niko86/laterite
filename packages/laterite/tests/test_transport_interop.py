"""laterite.transport ↔ pyrage age-interop — the envelope is *standard age*.

Our own ``lock``/``unlock`` round-trip only proves we are self-consistent. These
tests prove the stronger claim the docstrings make ("interoperable with
``pyrage``"): the ``.zst.age`` envelope is a real age passphrase file, verified
against the INDEPENDENT ``pyrage`` binding (Rust ``age`` crate via PyO3), not our
own decryptor.

``lock`` layers zstd THEN age, so ``pyrage.decrypt`` of a lock output yields the
inner **zstd frame** — which our ``unpack`` decompresses; conversely
``pyrage.encrypt`` of our ``pack`` output opens with our ``unlock``.

``pyrage`` is a HARD dev dependency (abi3 wheels for macOS + manylinux + win),
imported at module top with no ``importorskip`` — a skipped oracle would prove
nothing, so it must actually run in CI.

**Low ``log_n`` keeps it fast.** Each op runs scrypt; at the production factor 18
that is hundreds of ms → many seconds on a slow shared runner. Our-side locks pass
``log_n=_TEST_LOG_N`` (a cheap factor) — pyrage's decryptor reads whatever factor
the header declares, so interop is unaffected. Only the pyrage→ours leg can't be
sped up: pyrage's ``passphrase.encrypt`` hard-codes **log_N = 20** with no knob.

**Work-factor asymmetry (real, documented).** age's decryptor caps the accepted
factor by *machine speed* (anti-DoS), so a pyrage(20) envelope opens with our
``unlock`` on a fast host but can be rejected ("Excessive work parameter") on a
slow one. That is age policy, not a format incompatibility — so the pyrage→ours
leg tolerates the cap rejection while still proving we PARSED the envelope.
"""

from __future__ import annotations

from pathlib import Path

import pyrage
import pyrage.passphrase as age
import pytest
from laterite import transport

_PW = "correct horse battery staple"
# Cheap scrypt factor for the our-side locks — well under any machine's decrypt
# cap and ~256× cheaper than the production 18. pyrage reads any declared factor.
_TEST_LOG_N = 10

# Representative payloads: empty (a real zstd/age edge), a small AGS-ish blob.
_PAYLOADS = [
    pytest.param(b"", id="empty"),
    pytest.param(b'"GROUP","PROJ"\r\n"DATA","P1"\r\n' * 4, id="ags-blob"),
]


@pytest.mark.parametrize("data", _PAYLOADS)
def test_our_lock_bytes_is_decryptable_by_pyrage(data: bytes) -> None:
    # OUR lock_bytes → THEIR age decrypt strips the age layer → OUR unpack the zstd.
    # The load-bearing proof: our envelope is standard age, read by an independent
    # implementation.
    sealed = transport.lock_bytes(data, password=_PW, log_n=_TEST_LOG_N)
    inner = age.decrypt(sealed, _PW)  # pyrage removes the age envelope
    assert inner[:4] == bytes.fromhex("28B52FFD")  # inner really is a zstd frame
    assert transport.unpack_bytes(inner) == data


def test_file_form_envelope_is_standard_age(tmp_path: Path) -> None:
    # The .zst.age FILE that lock() writes is likewise standard age — pyrage reads it.
    payload = b'"GROUP","PROJ"\r\n' * 8
    src = tmp_path / "delivery.ags"
    src.write_bytes(payload)
    locked = transport.lock(src, password=_PW, log_n=_TEST_LOG_N)
    assert locked == tmp_path / "delivery.ags.zst.age"
    inner = age.decrypt(locked.read_bytes(), _PW)
    assert transport.unpack_bytes(inner) == payload


def test_pyrage_sealed_envelope_is_recognised_by_our_unlock() -> None:
    # THEIR age encrypt over OUR zstd frame → OUR unlock. pyrage encrypts at
    # log_N 20 (no knob); age's decryptor caps work factor by machine speed, so on
    # a slow host our unlock may reject 20 as "Excessive work parameter". That is
    # age anti-DoS policy, not a format mismatch — so accept EITHER the round-trip
    # OR the work-factor rejection, but never a NotPassphrase / malformed-envelope
    # error (which would mean we failed to parse pyrage's scrypt file).
    data = b'"GROUP","LOCA"\r\n"DATA","BH1"\r\n'
    sealed = age.encrypt(transport.pack_bytes(data), _PW)
    try:
        assert transport.unlock_bytes(sealed, password=_PW) == data
    except RuntimeError as exc:
        assert "work parameter" in str(exc), (
            f"pyrage envelope must parse as our scrypt file; only the work-factor "
            f"cap may reject it, got: {exc}"
        )


def test_pyrage_rejects_a_wrong_passphrase_on_our_envelope() -> None:
    # The negative leg: a wrong passphrase against our envelope fails in pyrage too
    # (proves the scrypt work-factor + recipient stanza are genuinely age, not a
    # no-op our own unlock might tolerate).
    sealed = transport.lock_bytes(b"secret payload", password=_PW, log_n=_TEST_LOG_N)
    with pytest.raises(pyrage.DecryptError):
        age.decrypt(sealed, "the wrong passphrase")
