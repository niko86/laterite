#!/usr/bin/env python3
"""The M1 cross-OS go/no-go probe (#833): does releasing each compat group's
Arrow table as its frame materialises reduce the operation's peak memory
*on this machine*?

Why this exists as its own instrument: #831 proved the release is real at the
allocator level but invisible to peak RSS on the darwin ledger machine —
`MADV_FREE`/`MADV_FREE_REUSABLE` never leave residency there without memory
pressure, so held and released peaks are indistinguishable. Most real users
run hosted Linux or Windows, where the purge primitives genuinely decommit
(`MADV_DONTNEED` / `VirtualFree(MEM_DECOMMIT)`), so the landing decision is
measured on those OSes instead of argued from darwin.

The claim family: numbers from this probe are a THIRD claim, labelled — one
fresh child per cell reading its own peak (`ru_maxrss` on POSIX, ctypes
`GetProcessMemoryInfo().PeakWorkingSetSize` on Windows, where `resource`
does not exist). They never share a table with the darwin lane
(`tools/perf-results/python-lane.json`) or with the dhat diagnosis numbers —
the campaign's rule 8 (`ags-wiki/concepts/perf-campaign.md`).

Three cells per rung:

- ``held``      today's shipped semantics — the public ``AGS4_to_dataframe``,
                every Arrow table live until return.
- ``released``  the M1 fix, loop-equivalent: pop each group's table as its
                frame materialises. (Loop-equivalence was validated in #831:
                held children reproduced the lane's committed cells within
                0.3%, so the fix does not need to be landed to be measured.)
- ``purge``     ``released`` plus a forced ``mi_collect(true)`` per group.
                The purge call resolves in order: a ``purge_native_heap``
                hook if the checked-out ref carries one → a ctypes
                ``mi_collect`` lookup on the native module (ELF may export
                what Mach-O hid) → a recorded ``purge-unavailable`` refusal,
                never a silent skip.

The pandas hop is the SHIPPED default: this probe expects a pyarrow-free
venv (the `[compat]` install shape) and records which hop actually ran —
a pyarrow-present venv is a recorded fact, not an error, but the go/no-go
verdict is only meaningful on the shipped hop.

Verdict rule (printed, and recorded in the JSON): the campaign's candidate
floor — released (or purge) cutting ≥ 5% off held's peak on a rung is a GO
on this OS; anything under it means #831's decline generalises here.

Usage:
    python tools/perf_probe_m1.py                       # default rungs 25MB,265MB
    python tools/perf_probe_m1.py --rungs 25MB --out output/perf-probe/local.json
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import platform
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

REPO = Path(__file__).resolve().parents[1]
# The probe carries its OWN corpus pin, not the README bench's. Its results
# are a separate claim family (campaign rule 8), so all it needs is byte
# identity ACROSS the two probe OSes — and the README manifest predates
# emit-output-changing work (Rule 5 re-quoting and after), so a fresh forge
# can no longer reproduce those bytes anywhere (#873). Own manifest, own
# cache directory: the README bench's cached rungs are a different corpus
# under the same rung names, and sharing a path would flag one corpus as
# drift against the other's pin.
OUT_DIR = REPO / "output" / "perf-probe-fixtures"
MANIFEST = REPO / "tools" / "perf-probe-fixtures.json"
FORGE = (
    REPO
    / "rust-packages"
    / "target"
    / "release"
    / ("laterite-ags4-forge.exe" if sys.platform == "win32" else "laterite-ags4-forge")
)
DEFAULT_RUNGS = ["25MB", "265MB"]
SEED = 0
SCAFFOLD = "wide"
# The campaign's candidate floor (rule 10), the go/no-go line for this probe.
GO_FLOOR = 0.05

MODES = ("held", "released", "purge")


def die(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(1)


# --- fixtures (same determinism + drift contract as bench-vs-python-ags4) ---


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def fixture(size: str) -> Path:
    """Generate (or reuse) one SHA-pinned rung; drift against the committed
    manifest is a hard error — a probe against different bytes would read as
    an OS difference and be a generator difference."""
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUT_DIR / f"probe-{size}.ags"
    packed = path.with_suffix(".ags.zst")
    if not path.exists() and packed.exists():
        from laterite.transport import unpack

        unpack(packed)
    if not path.exists():
        if not FORGE.exists():
            print("building laterite-ags4-forge (release) ...", flush=True)
            subprocess.run(
                ["cargo", "build", "--release", "-p", "laterite-ags4-forge"],
                cwd=REPO / "rust-packages",
                check=True,
            )
        print(f"generating {path.name} ...", flush=True)
        subprocess.run(
            [
                str(FORGE),
                "scale",
                "--size",
                size,
                "--scaffold",
                SCAFFOLD,
                "--seed",
                str(SEED),
                "--out",
                str(path),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
        )
    recorded = json.loads(MANIFEST.read_text()) if MANIFEST.exists() else {}
    actual = sha256(path)
    if size in recorded and recorded[size]["sha256"] != actual:
        die(
            f"fixture drift on {size} — forge no longer produces the pinned "
            f"bytes; a probe against different data is not a probe of the OS. "
            f"pinned {recorded[size]['sha256']} ({recorded[size]['bytes']} "
            f"bytes), got {actual} ({path.stat().st_size} bytes)."
        )
    return path


# --- the instrument -------------------------------------------------------


def peak_bytes() -> tuple[int, str]:
    """This process's peak memory so far, plus the instrument's name.

    POSIX: `ru_maxrss` — bytes on darwin, KiB on Linux (getrusage(2) differs
    by lineage). Windows has no `resource` module; the equivalent high-water
    is `PeakWorkingSetSize` from `GetProcessMemoryInfo`, read via ctypes so
    the probe carries no extra dependency.
    """
    if sys.platform == "win32":
        import ctypes
        import ctypes.wintypes as wt

        class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
            _fields_ = [
                ("cb", wt.DWORD),
                ("PageFaultCount", wt.DWORD),
                ("PeakWorkingSetSize", ctypes.c_size_t),
                ("WorkingSetSize", ctypes.c_size_t),
                ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                ("PagefileUsage", ctypes.c_size_t),
                ("PeakPagefileUsage", ctypes.c_size_t),
            ]

        counters = PROCESS_MEMORY_COUNTERS()
        counters.cb = ctypes.sizeof(counters)
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        # Without declared signatures ctypes marshals through c_int, which
        # truncates the 64-bit pseudo-handle GetCurrentProcess returns —
        # every call then fails ERROR_INVALID_HANDLE (run 33585274632, all
        # four Windows measurement cells).
        kernel32.GetCurrentProcess.restype = wt.HANDLE
        kernel32.K32GetProcessMemoryInfo.argtypes = [
            wt.HANDLE,
            ctypes.POINTER(PROCESS_MEMORY_COUNTERS),
            wt.DWORD,
        ]
        kernel32.K32GetProcessMemoryInfo.restype = wt.BOOL
        ok = kernel32.K32GetProcessMemoryInfo(
            kernel32.GetCurrentProcess(), ctypes.byref(counters), counters.cb
        )
        if not ok:
            raise OSError(ctypes.get_last_error(), "K32GetProcessMemoryInfo failed")
        return int(counters.PeakWorkingSetSize), "PeakWorkingSetSize"
    import resource

    raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return (raw if sys.platform == "darwin" else raw * 1024), "ru_maxrss"


# --- the worker: one mode, one fresh process ------------------------------


def resolve_purge() -> tuple[Callable[[], object] | None, str]:
    """The forced-purge call, or None with the reason it is unreachable."""
    from laterite import _laterite_native as _native

    hook = getattr(_native, "purge_native_heap", None)
    if hook is not None:
        return hook, "native purge_native_heap hook"
    import ctypes

    try:
        lib = ctypes.CDLL(_native.__file__)
        collect = lib.mi_collect
    except (OSError, AttributeError):
        return None, "no hook on this ref and mi_collect not exported"
    collect.argtypes = [ctypes.c_bool]
    collect.restype = None
    return (lambda: collect(True)), "ctypes mi_collect on the native module"


def worker_main(spec_json: str) -> int:
    spec = json.loads(spec_json)
    mode, path = spec["mode"], spec["path"]

    from laterite import _frames
    from laterite.compat import _impl

    hop = "pyarrow" if _frames._pyarrow_available() else "duckdb"
    purge_via = None

    if mode == "held":
        tables, headings = _impl.AGS4_to_dataframe(path)
    else:
        purge = None
        if mode == "purge":
            purge, purge_via = resolve_purge()
            if purge is None:
                Path(spec["result_path"]).write_text(
                    json.dumps({"refusal": "purge-unavailable", "detail": purge_via})
                )
                return 0
        # The M1 fix, loop-equivalent (see the module docstring for why this
        # measures the fix without landing it).
        p = _impl._compat_arrow(path, "utf-8", None)
        _impl._strict_check_native(p)
        groups = p["groups"]
        headings = {
            c: ["HEADING", *_impl._rename_dups(list(groups[c]["headings"]), True, c)]
            for c in p["group_order"]
            if c in groups
        }
        mat = _frames.compat_materializer("pandas", "object")
        tables = {}
        for code in list(headings):
            g = groups[code]
            tables[code] = mat(g.pop("table"), headings[code])
            if purge is not None:
                purge()
        del p, groups

    n_groups = len(tables)
    peak, instrument = peak_bytes()
    Path(spec["result_path"]).write_text(
        json.dumps(
            {
                "peak_bytes": peak,
                "instrument": instrument,
                "hop": hop,
                "groups": n_groups,
                "purge_via": purge_via,
            }
        )
    )
    return 0


# --- the parent -----------------------------------------------------------


def measure(mode: str, path: Path) -> dict:
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False, dir=OUT_DIR) as tf:
        result_path = Path(tf.name)
    spec = {"mode": mode, "path": str(path), "result_path": str(result_path)}
    proc = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--worker", json.dumps(spec)],
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        if proc.returncode != 0:
            tail = (proc.stderr or "").strip().splitlines()[-3:]
            return {
                "refusal": "failed",
                "detail": " | ".join(tail) or f"exit {proc.returncode}",
            }
        return json.loads(result_path.read_text())
    finally:
        result_path.unlink(missing_ok=True)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rungs", default=",".join(DEFAULT_RUNGS))
    ap.add_argument(
        "--out",
        type=Path,
        default=REPO / "output" / "perf-probe" / f"{sys.platform}.json",
    )
    ap.add_argument("--worker", help=argparse.SUPPRESS)
    args = ap.parse_args()

    if args.worker:
        return worker_main(args.worker)

    try:
        import laterite  # noqa: F401
    except ImportError:
        die("laterite not importable — install the built wheel into this venv first")

    rungs = [s.strip() for s in args.rungs.split(",") if s.strip()]
    paths = {s: fixture(s) for s in rungs}

    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
        cwd=REPO,
    )
    doc: dict = {
        "schema": "laterite-m1-cross-os-probe/1",
        "claim_family": (
            "third family (#833): per-mode fresh-child peak memory on this OS; "
            "never shares a table with the darwin lane or dhat numbers"
        ),
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "commit": commit.stdout.strip() if commit.returncode == 0 else "unknown",
        "platform": platform.platform(),
        "arch": platform.machine(),
        "go_floor": GO_FLOOR,
        "rungs": {},
    }

    verdicts: list[str] = []
    for size, path in paths.items():
        cells = {mode: measure(mode, path) for mode in MODES}
        file_bytes = path.stat().st_size
        doc["rungs"][size] = {"file_bytes": file_bytes, "cells": cells}
        held = cells["held"].get("peak_bytes")
        line = f"[{size}]"
        for mode in MODES:
            c = cells[mode]
            if "refusal" in c:
                line += f"  {mode}: REFUSED ({c['refusal']}: {c['detail']})"
                continue
            line += f"  {mode}: {c['peak_bytes'] / 1e6:.0f} MB ({c['peak_bytes'] / file_bytes:.2f}x)"
            if mode != "held" and held:
                cut = (held - c["peak_bytes"]) / held
                verdict = "GO" if cut >= GO_FLOOR else "below floor"
                line += f" [{cut:+.1%} vs held: {verdict}]"
                verdicts.append(f"{size}/{mode}: {cut:+.1%} ({verdict})")
        print(line, flush=True)

    doc["verdicts"] = verdicts
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(doc, indent=1) + "\n")
    print(f"\nresults written: {args.out}")
    # The probe exists to produce verdicts. A run where every cell was
    # refused wrote a diagnosable artifact but measured nothing — a green
    # job over it would read as "Windows measured" when Windows did not
    # (run 33585274632's windows leg concluded success on four refusals).
    if not verdicts:
        print("error: no verdicts — every measurement cell was refused", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
