"""The wheel's ``lat`` must not depend on the console's encoding.

On Windows a cp1252 console or pipe hands Python a cp1252 text stream, and the
guide's ``→`` / ``↔`` glyphs have no cp1252 encoding — so ``lat --help`` and
``lat --readme`` died with a ``UnicodeEncodeError`` on the first flag a new user
reaches for, and any finding or error text carrying such a character did the
same. Python's UTF-8 mode (PEP 540) is the policy that cannot fail this way and
becomes the default in 3.15 (PEP 686); the wheel ships for 3.12+, so ``main()``
applies that policy to its two output streams itself. The Rust binary and the
Node launcher write bytes and never had the fault.

``capsys`` cannot stage the fault — its capture is UTF-8 already — so the
end-to-end check is a subprocess with ``PYTHONIOENCODING=cp1252``, the stream
Windows gives the launcher, reproducible on every OS. The in-process check
stages both streams as cp1252 wrappers and asserts the policy itself, stderr
included, which no command line reaches deterministically. Both were run red
against the unfixed ``main()`` before the fix landed.
"""

from __future__ import annotations

import io
import os
import subprocess
import sys
from typing import Any

import pytest
from laterite import _cli

#: A guide glyph outside cp1252 — the one the Windows report died on.
_ARROW = "↔"


@pytest.mark.parametrize("flag", ["--help", "--readme"])
def test_guide_survives_a_cp1252_stream(flag: str) -> None:
    env = {**os.environ, "PYTHONIOENCODING": "cp1252", "PYTHONUTF8": "0"}
    proc = subprocess.run(
        [sys.executable, "-m", "laterite._cli", flag],
        env=env,
        capture_output=True,
        timeout=120,
    )
    assert proc.returncode == 0, proc.stderr.decode("utf-8", "replace")
    assert b"UnicodeEncodeError" not in proc.stderr
    # UTF-8 on the wire whatever the console said — the bytes the Rust binary
    # emits for the same guide, so the three `lat` programs still agree.
    assert _ARROW in proc.stdout.decode("utf-8")


def test_stdio_policy_is_utf8_mode(monkeypatch: Any) -> None:
    out = io.TextIOWrapper(io.BytesIO(), encoding="cp1252", errors="strict")
    err = io.TextIOWrapper(io.BytesIO(), encoding="cp1252", errors="strict")
    monkeypatch.setattr(sys, "stdout", out)
    monkeypatch.setattr(sys, "stderr", err)

    assert _cli.main(["--readme"]) == 0
    out.flush()

    assert (out.encoding, out.errors) == ("utf-8", "surrogateescape")
    assert (err.encoding, err.errors) == ("utf-8", "backslashreplace")
    assert _ARROW in out.buffer.getvalue().decode("utf-8")
