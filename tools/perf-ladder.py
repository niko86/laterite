#!/usr/bin/env python3
"""Materialise the forge size ladder for the cross-surface perf matrix.

The per-surface harnesses — `laterite-ags4-perf` (rust) today, the Node/wasm/
CLI lanes to follow (#823-#825) — all read one ladder manifest
(`output/perf-ladder/manifest.json`) naming the rungs to measure. This script
writes it. The rungs themselves are the python lane's fixtures: the same
forge invocation, the same SHA pins, the same on-disk files
(`tools/bench-vs-python-ags4.py` owns that machinery and this script imports
it), so every surface measures byte-identical data and the disk holds one
copy.

Pinning is inherited too: a rung whose bytes drift from
`tools/readme-bench-fixtures.json` is a hard error here, exactly as in the
bench. If forge changed ON PURPOSE, re-pin (and re-measure) via
`tools/bench-vs-python-ags4.py --update-manifest`, then re-run this.

Manifest schema `laterite-perf-ladder/1`:

    schema      the string above; bump on shape changes
    generated   UTC timestamp of the write
    forge       {scaffold, seed} — the fixed generator invocation
    rungs       [{label, path, bytes, sha256}] — absolute paths, size-ordered

Consumers read `label` + `path` and may ignore the rest (provenance). The
rungs are left PLAIN on disk — the harnesses read them — so a full default
ladder holds a few hundred MB; `--repack` re-packs the manifest's rungs
(zstd, via laterite.transport) once every harness has run.

The default ladder stops at the 265MB rung: past it, memory columns are
refusals by the campaign's cap (epic #820 decision 7) and the machine spends
minutes on time-only cells. Ask for `--rungs 5MB,...,524MB` deliberately.

Usage:
    uv run python tools/perf-ladder.py                    # default rungs
    uv run python tools/perf-ladder.py --rungs 5MB,25MB
    uv run python tools/perf-ladder.py --repack           # reclaim the disk
"""

from __future__ import annotations

import argparse
import datetime
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
DEFAULT_OUT = REPO / "output" / "perf-ladder" / "manifest.json"
DEFAULT_RUNGS = ["5MB", "25MB", "100MB", "265MB"]

LADDER_SCHEMA = "laterite-perf-ladder/1"


def load_bench() -> Any:
    """Import `tools/bench-vs-python-ags4.py` as a module — the hyphenated
    name is not importable, and `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "bench_vs_python_ags4", REPO / "tools" / "bench-vs-python-ags4.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["bench_vs_python_ags4"] = mod
    spec.loader.exec_module(mod)
    return mod


def build_manifest(
    paths: dict[str, Path], pins: dict[str, Any], scaffold: str, seed: int
) -> dict[str, Any]:
    """Assemble the ladder manifest (pure; the schema in the docstring)."""
    rungs = [
        {
            "label": label,
            "path": str(path.resolve()),
            "bytes": pins[label]["bytes"],
            "sha256": pins[label]["sha256"],
        }
        for label, path in paths.items()
    ]
    rungs.sort(key=lambda r: r["bytes"])
    return {
        "schema": LADDER_SCHEMA,
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "forge": {"scaffold": scaffold, "seed": seed},
        "rungs": rungs,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--rungs",
        default=",".join(DEFAULT_RUNGS),
        help="comma-separated forge sizes (default: %(default)s)",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="manifest to write (default: %(default)s)",
    )
    ap.add_argument(
        "--repack",
        action="store_true",
        help="re-pack the manifest's rungs (zstd) instead of building — run "
        "it after the harnesses to reclaim the disk",
    )
    args = ap.parse_args()

    bench = load_bench()

    if args.repack:
        if not args.out.exists():
            print(
                f"error: no manifest at {args.out} — nothing to repack", file=sys.stderr
            )
            return 1
        manifest = json.loads(args.out.read_text())
        bench.repack({r["label"]: Path(r["path"]) for r in manifest["rungs"]})
        return 0

    rungs = [r.strip() for r in args.rungs.split(",") if r.strip()]
    paths = {label: bench.fixture(label) for label in rungs}
    # Dies loudly on drift — comparing surfaces against different data would
    # look like a perf change and would not be one.
    pins = bench.check_manifest(paths, update=False)

    manifest = build_manifest(paths, pins, bench.SCAFFOLD, bench.SEED)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(manifest, indent=2) + "\n")

    total = sum(r["bytes"] for r in manifest["rungs"])
    print(
        f"ladder written: {len(manifest['rungs'])} rungs, "
        f"{total / 1e6:.0f} MB plain → {args.out.relative_to(REPO)}"
    )
    print("next, the rust leg (from the repo root):")
    print(
        "  cargo run --release --manifest-path rust-packages/Cargo.toml "
        "-p laterite-ags4-perf"
    )
    print("then aggregate:  uv run python tools/perf-matrix.py")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
