#!/usr/bin/env python3
"""Reproduce the README's performance tables.

The README quotes laterite-vs-python-ags4 speedups for validation and for both
read paths. Until now nothing in the repo *ran* that comparison — the fixtures
were reproducible (`forge scale` is deterministic for a given size + seed) but
the measurement was not, so the headline numbers rested on a run nobody else
could repeat. This is that run.

Fixture stability
-----------------
`forge scale --size N --scaffold wide --seed 0` is byte-identical across
machines, so the rungs are already fixed. What is NOT fixed is forge itself: a
change to the generator silently produces different data, and the numbers then
move for reasons that have nothing to do with the engine. So each rung's
SHA-256 is pinned in `tools/readme-bench-fixtures.json` and verified before
timing. A drifted fixture is a hard error, not a warning — comparing against
different data is worse than not comparing.

Use `--update-manifest` deliberately, when the generator was *meant* to change,
and re-measure the whole table in the same run.

Disk
----
The rungs total ~900 MB plain, most of it the two largest. Between runs they are
kept zstd-packed via `laterite.transport` — the shipped pack/unpack — and
unpacked on demand. AGS4 is extremely compressible, so the resting footprint is
a fraction of the plain size.

This dogfoods the transport layer usefully rather than decoratively: the SHA-256
check runs against the UNPACKED bytes, so every benchmark run is also a
byte-exact round-trip test of `pack`/`unpack`. A lossy round trip would fail as
fixture drift.

Usage
-----
    uv run python tools/bench-vs-python-ags4.py                 # default rungs
    uv run python tools/bench-vs-python-ags4.py --rungs 5MB,25MB
    uv run python tools/bench-vs-python-ags4.py --runs 10
    uv run python tools/bench-vs-python-ags4.py --update-manifest

Needs `python-ags4` importable (the comparison target) and the release `lat`
toolchain built; both are checked before any timing starts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import statistics
import subprocess
import sys
import time
from functools import partial
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable
    from types import ModuleType

REPO = Path(__file__).resolve().parent.parent
OUT_DIR = REPO / "output" / "readme-bench"
MANIFEST = REPO / "tools" / "readme-bench-fixtures.json"
FORGE = REPO / "rust-packages" / "target" / "release" / "laterite-ags4-forge"

# The README's rungs. Seed 0 and the `wide` scaffold everywhere — the point is a
# FIXED file, not a varied one.
DEFAULT_RUNGS = ["5MB", "25MB", "100MB"]
SEED = 0
SCAFFOLD = "wide"


def die(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ensure_forge() -> None:
    if FORGE.exists():
        return
    print("building laterite-ags4-forge (release) — first run only...")
    subprocess.run(
        ["cargo", "build", "--release", "-p", "laterite-ags4-forge"],
        cwd=REPO / "rust-packages",
        check=True,
    )


def fixture(size: str) -> Path:
    """Generate, unpack or reuse one rung. Deterministic for a given size + seed."""
    from laterite.transport import unpack

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUT_DIR / f"readme-{size}.ags"
    packed = path.with_suffix(".ags.zst")
    if not path.exists() and packed.exists():
        print(f"  unpacking {packed.name} ...")
        unpack(packed)
    if not path.exists():
        ensure_forge()
        print(f"  generating {path.name} ...")
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
    return path


def check_manifest(paths: dict[str, Path], update: bool) -> None:
    """Pin each rung's bytes, so generator drift is loud rather than silent."""
    recorded = json.loads(MANIFEST.read_text()) if MANIFEST.exists() else {}
    actual = {
        size: {"sha256": sha256(p), "bytes": p.stat().st_size}
        for size, p in paths.items()
    }

    if update:
        recorded.update(actual)
        MANIFEST.write_text(json.dumps(recorded, indent=2, sort_keys=True) + "\n")
        print(f"manifest updated: {MANIFEST.relative_to(REPO)}")
        return

    drifted = [
        size
        for size, got in actual.items()
        if size in recorded and recorded[size]["sha256"] != got["sha256"]
    ]
    if drifted:
        die(
            f"fixture drift on {', '.join(drifted)} — forge no longer produces the "
            f"bytes these numbers were measured against.\n"
            f"       Timing against different data would look like a perf change "
            f"and would not be one.\n"
            f"       If the generator was meant to change, re-run with "
            f"--update-manifest and re-measure the whole table."
        )
    missing = [s for s in actual if s not in recorded]
    if missing:
        print(
            f"note: no pinned hash yet for {', '.join(missing)} "
            f"(run --update-manifest to pin)"
        )


def repack(paths: dict[str, Path]) -> None:
    """Leave the fixtures packed: same bytes, a fraction of the disk.

    Deliberately after timing, never before — the plain file is what gets timed,
    and unpacking mid-run would put decompression inside a measurement.
    """
    from laterite.transport import pack

    saved = 0
    for path in paths.values():
        if not path.exists():
            continue
        plain = path.stat().st_size
        out = pack(path)
        saved += plain - out.stat().st_size
        path.unlink()
    if saved:
        print(
            f"\nfixtures repacked — {saved / 1e6:.0f} MB reclaimed "
            f"(unpacked automatically on the next run)"
        )


def best_of(fn: Callable[[], object], runs: int) -> float:
    """Mean of `runs` warm timings, first run discarded (file cache, imports)."""
    fn()
    return statistics.mean(_time(fn) for _ in range(runs))


def _time(fn: Callable[[], object]) -> float:
    t0 = time.perf_counter()
    fn()
    return time.perf_counter() - t0


def warn_if_debug_build(laterite_mod: ModuleType) -> None:
    """Refuse to publish numbers measured against a debug wheel.

    A debug build of the native module is roughly 5x slower across every path,
    which looks exactly like a catastrophic regression and is not one. The tell
    is size: the release abi3 module is a few MB, a debug one is tens.
    """
    mod_file = laterite_mod.__file__
    if mod_file is None:
        return  # namespace package with no single file — nothing to size-check
    native = list(pathlib.Path(mod_file).parent.glob("*.so"))
    if not native:
        return
    mb = max(f.stat().st_size for f in native) / 1e6
    if mb > 25:
        die(
            f"the installed laterite native module is {mb:.0f} MB — that is a "
            f"DEBUG build.\n"
            f"       Every laterite timing would be ~5x slow and would read as a "
            f"regression that is not real.\n"
            f"       Rebuild first:  (cd packages/laterite && uv run --no-sync "
            f"maturin develop --release --uv)"
        )


def dist_version(name: str) -> str:
    try:
        return version(name)
    except PackageNotFoundError:
        return "?"


def fmt(seconds: float) -> str:
    return f"{seconds * 1000:.0f} ms" if seconds < 1 else f"{seconds:.1f} s"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--rungs",
        default=",".join(DEFAULT_RUNGS),
        help="comma-separated forge sizes (default: %(default)s)",
    )
    ap.add_argument("--runs", type=int, default=5, help="warm runs per cell")
    ap.add_argument(
        "--update-manifest",
        action="store_true",
        help="re-pin fixture hashes (use when forge changed ON PURPOSE)",
    )
    ap.add_argument(
        "--keep-plain",
        action="store_true",
        help="skip the post-run repack (faster repeat runs, ~900 MB on disk)",
    )
    args = ap.parse_args()

    try:
        from python_ags4 import AGS4 as UPSTREAM
    except ImportError:
        die(
            "python-ags4 not importable — it is the comparison target.\n"
            "       Install it into this environment first."
        )
    try:
        import laterite
        from laterite import compat as COMPAT
    except ImportError:
        die(
            "laterite not importable — build the wheel first:\n"
            "       (cd packages/laterite && uv run --no-sync maturin develop "
            "--release --uv)"
        )

    warn_if_debug_build(laterite)

    sizes = [s.strip() for s in args.rungs.split(",") if s.strip()]
    paths = {s: fixture(s) for s in sizes}
    check_manifest(paths, args.update_manifest)

    # Report both versions: a speedup is meaningless without knowing what it
    # was measured against, and neither package exposes __version__ reliably.
    print(
        f"\npython-ags4 {dist_version('python-ags4')} vs "
        f"laterite {dist_version('laterite')} — mean of {args.runs} warm runs\n"
    )

    rows: dict[str, list[tuple[str, float, float]]] = {
        "validate": [],
        "read_strings": [],
        "read_typed": [],
    }

    def upstream_typed(target: str) -> None:
        """python-ags4's route to typed columns: read, then convert per group."""
        tables, _ = UPSTREAM.AGS4_to_dataframe(target)
        for key in tables:
            UPSTREAM.convert_to_numeric(tables[key])

    # `partial` rather than a lambda: a closure over the loop variable would
    # bind late, so every rung would end up timing the last file.
    for path in paths.values():
        mb = path.stat().st_size / 1e6
        label = f"{mb:.1f} MB"
        p = str(path)
        print(f"[{label}] timing ...", flush=True)

        rows["validate"].append(
            (
                label,
                best_of(partial(UPSTREAM.check_file, p), args.runs),
                best_of(partial(laterite.validate, p), args.runs),
            )
        )
        rows["read_strings"].append(
            (
                label,
                best_of(partial(UPSTREAM.AGS4_to_dataframe, p), args.runs),
                best_of(partial(COMPAT.AGS4_to_dataframe, p), args.runs),
            )
        )
        rows["read_typed"].append(
            (
                label,
                best_of(partial(upstream_typed, p), args.runs),
                best_of(partial(laterite.read, p), args.runs),
            )
        )

    def table(title: str, left: str, right: str, key: str) -> None:
        print(f"\n**{title}**\n")
        print(f"| File | `{left}` | `{right}` | speedup |")
        print("|---:|---:|---:|:---:|")
        for label, a, b in rows[key]:
            print(f"| {label} | {fmt(a)} | {fmt(b)} | **{a / b:.1f}×** |")

    print("\n" + "=" * 62)
    print("README-format tables — paste into the Performance section")
    print("=" * 62)
    table("Validation", "python-ags4 check_file", "laterite.validate", "validate")
    table("Read, strings", "python-ags4 AGS4_to_dataframe", "compat", "read_strings")
    table(
        "Read, typed", "python-ags4 + convert_to_numeric", "laterite.read", "read_typed"
    )
    if not args.keep_plain:
        repack(paths)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
