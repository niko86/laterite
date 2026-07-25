"""Concurrency guard for the GIL release (#8).

The CPU-bound native read/validate entry points release the GIL for their pure-
Rust compute (`Python::detach`), so a second Python thread makes progress WHILE a
native call runs. Before #8 the GIL was held end-to-end and a concurrent thread
was starved to ~zero — which is exactly what would silently return if a future
edit dropped the `detach`. This is the regression guard for that.

Method (self-calibrating, no wall-clock threshold): a background thread spins a
counter. We measure its progress *during* a native call, then over an equal idle
`time.sleep` window (sleep releases the GIL, so that is the spinner's full rate).
GIL held ⇒ during ≈ 0 ≪ free. GIL released ⇒ during is a large fraction of free
(full on multi-core, time-shared on one). The 0.1 floor is deliberately forgiving.
"""

from __future__ import annotations

import threading
import time

import pytest
from laterite import _laterite_native as N


def _synthetic_ags(n_rows: int) -> str:
    head = "\r\n".join(
        [
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID","PROJ_NAME"',
            '"UNIT","",""',
            '"TYPE","ID","X"',
            '"DATA","GIL","concurrency guard"',
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


# Big enough that one native call takes tens of ms — the window the spinner runs
# in. Bytes so every call is pure CPU (no per-call file I/O).
_AGS = _synthetic_ags(30_000).encode("utf-8")


def _measure(call) -> tuple[int, int]:
    """Return (spinner progress during `call`, spinner progress over an equal
    idle window). A daemon thread spins a Python counter; we sample it around the
    native call and around a matched `time.sleep`."""
    counter = [0]
    stop = threading.Event()

    def spin() -> None:
        while not stop.is_set():
            counter[0] += 1

    t = threading.Thread(target=spin, daemon=True)
    t.start()
    try:
        time.sleep(0.02)  # let the spinner reach steady state
        base = counter[0]
        t0 = time.perf_counter()
        call()
        dt = time.perf_counter() - t0
        during = counter[0] - base
        # Matched GIL-free window: sleep releases the GIL, so this is full rate.
        base = counter[0]
        time.sleep(dt)
        free = counter[0] - base
    finally:
        stop.set()
        t.join()
    return during, free


@pytest.mark.parametrize(
    ("name", "call"),
    [
        ("run_check", lambda: N.run_check(data=_AGS)),
        ("parse_arrow", lambda: N.parse_arrow(data=_AGS)),
        ("parse_compat_arrow", lambda: N.parse_compat_arrow(data=_AGS)),
    ],
)
def test_gil_released_during_native_compute(name: str, call) -> None:
    during, free = _measure(call)
    assert free > 0, "spinner never ran — measurement setup is broken, not a GIL result"
    # Held ⇒ during ≈ 0. Released ⇒ during is a large fraction of the free rate.
    assert during > free * 0.1, (
        f"{name}: concurrent thread advanced {during} during the call vs {free} "
        f"idle ({during / free:.1%}) — the GIL appears held, not released"
    )
