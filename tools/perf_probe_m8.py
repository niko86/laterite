#!/usr/bin/env python3
"""#893 M8 diagnosis probe (SPIKE BRANCH ONLY — never lands on main).

Variant children of the CLI read door (`lat read <rung> <G> --csv --out …`),
alone and paired, per the #848 co-peak rule: a slice's at-peak share is not
its contribution, so each re-materialisation is removed alone and then both
are, and only the measured deltas price anything.

The children are env-gated inside the spike `lat` build
(`laterite-cli/src/commands/read.rs` on this branch):

  base   the shipped path: span ParsedFile -> from_shared's whole-file
         HashMap<Arc<str>, String> rows-maps -> the dumped group's
         Vec<Vec<String>> projection -> rendered CSV.
  slab   LATERITE_M8_SPIKE=slab — the rows-map slab never builds; the
         projection + render build straight off the spans.
  proj   LATERITE_M8_SPIKE=proj — the map slab builds; the projection copy
         never does (row-by-row render off the maps).
  both   LATERITE_M8_SPIKE=both — spans -> CSV text directly.

Instrument: peak RSS of a fresh child per run (`os.wait4` ru_maxrss — this
child's own, never RUSAGE_CHILDREN), the diagnosis family (epic #820 rule 8)
— these numbers price variant children on one machine and NEVER enter a
cross-library table. Denominator: ×-of-INPUT, matching the CLI lane's read
row. A/B/A: a full base pass brackets the variant passes in one sitting; the
A-legs' spread prices the session's drift. Every child's --out CSV is
sha256-checked identical to the baseline's per rung — a variant that changes
the bytes measured a different program.

Usage (quiet machine; needs the ladder manifest + the SPIKE release lat):
    uv run --no-project python tools/perf_probe_m8.py [--reps 3]
    # LAT_BIN overrides the binary; output: tools/perf-results/m8-diagnosis.json
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
MANIFEST = REPO / "output" / "perf-ladder" / "manifest.json"
OUT_PATH = REPO / "tools" / "perf-results" / "m8-diagnosis.json"

VARIANTS = ["base", "slab", "proj", "both"]
GROUP = "GEOL"  # the forge scaffold's bulk group on every rung (#825's pick)


def load_bench() -> Any:
    spec = importlib.util.spec_from_file_location(
        "bench_vs_python_ags4", REPO / "tools" / "bench-vs-python-ags4.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["bench_vs_python_ags4"] = mod
    spec.loader.exec_module(mod)
    return mod


bench = load_bench()


def resolve_lat() -> Path:
    if pinned := os.environ.get("LAT_BIN"):
        p = Path(pinned)
        if not p.exists():
            raise SystemExit(f"error: LAT_BIN points at {p}, which does not exist")
        return p
    built = REPO / "rust-packages" / "target" / "release" / "lat"
    if not built.exists():
        raise SystemExit("error: no release lat; build the spike branch first")
    return built


def child_env(variant: str) -> dict[str, str]:
    env = dict(os.environ)
    env.pop("LATERITE_M8_SPIKE", None)
    if variant != "base":
        env["LATERITE_M8_SPIKE"] = variant
    return env


def run_child(argv: list[str], env: dict[str, str], stderr_path: Path) -> tuple[int, int, float]:
    """(returncode, peak_rss_bytes, wall_ms) of one fresh child, wait4-reaped."""
    t = time.perf_counter()
    with stderr_path.open("wb") as errf:
        proc = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=errf, env=env)
        _, status, ru = os.wait4(proc.pid, 0)
        proc.returncode = os.waitstatus_to_exitcode(status)
    wall_ms = (time.perf_counter() - t) * 1000.0
    sysname = "darwin" if sys.platform == "darwin" else "linux"
    return proc.returncode, bench.maxrss_to_bytes(ru.ru_maxrss, sysname), wall_ms


def measure_cell(
    lat: Path, rung: dict[str, Any], variant: str, reps: int, scratch: Path
) -> dict[str, Any]:
    """One (variant, rung) cell: `reps` fresh children, median peak; the swap
    watch and refusal vocabulary are the shared contract's."""
    input_bytes = rung["bytes"]
    if not bench.mem_rung_allowed(input_bytes):
        return bench.refusal_cell(
            "beyond-mem-cap",
            f"{input_bytes}-byte rung is past the {bench.MEM_CAP_BYTES}-byte cap",
        )
    out_csv = scratch / f"read-{variant}.csv"
    argv = [
        str(lat), "read", rung["path"], GROUP,
        "--csv", "--out", str(out_csv), "--quiet",
    ]
    env = child_env(variant)
    stderr_path = scratch / "child.stderr"
    peaks: list[int] = []
    walls: list[float] = []
    swap_before = bench.swap_used_bytes()
    for _ in range(reps):
        rc, peak, wall_ms = run_child(argv, env, stderr_path)
        if rc != 0:
            tail = stderr_path.read_text(errors="replace").strip().splitlines()[-3:]
            return bench.refusal_cell("failed", " | ".join(tail) or f"exit {rc}")
        peaks.append(peak)
        walls.append(wall_ms)
    swap_after = bench.swap_used_bytes()
    if (
        swap_before is not None
        and swap_after is not None
        and swap_after - swap_before > bench.SWAP_REFUSAL_BYTES
    ):
        grew = (swap_after - swap_before) / 1e6
        return bench.refusal_cell("swapped", f"swap grew {grew:.1f} MB during the cell")
    peaks.sort()
    walls.sort()
    cell = bench.mem_cell(peaks[len(peaks) // 2], max(input_bytes, 1))
    cell["peaks_bytes"] = peaks
    cell["wall_ms_median"] = round(walls[len(walls) // 2], 1)
    cell["out_sha256"] = hashlib.sha256(out_csv.read_bytes()).hexdigest()
    return cell


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--out", type=Path, default=OUT_PATH)
    args = ap.parse_args()

    lat = resolve_lat()
    manifest = json.loads(MANIFEST.read_text())
    rungs = [r for r in manifest["rungs"] if bench.mem_rung_allowed(r["bytes"])]

    git_sha = subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()

    loadavg_start = list(os.getloadavg())
    passes: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix="m8-probe-") as td:
        scratch = Path(td)
        # A/B/A: base brackets the variants in one sitting.
        for leg, variant in [("A1", "base"), *[("B", v) for v in VARIANTS[1:]], ("A2", "base")]:
            cells = {}
            for rung in rungs:
                cells[rung["label"]] = measure_cell(lat, rung, variant, args.reps, scratch)
                print(
                    f"{leg:>2} {variant:<5} {rung['label']:>6}: "
                    f"{cells[rung['label']].get('x_output', 'REFUSAL')}×-of-input",
                    flush=True,
                )
            passes.append({"leg": leg, "variant": variant, "cells": cells})

    # Byte-identity: every variant's CSV sha must equal the A1 base leg's.
    identity: dict[str, bool] = {}
    a1 = passes[0]["cells"]
    for p in passes[1:]:
        for label, cell in p["cells"].items():
            same = cell.get("out_sha256") == a1[label].get("out_sha256")
            identity[f"{p['leg']}-{p['variant']}/{label}"] = same

    out = {
        "schema": "m8-diagnosis/1",
        "issue": 893,
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "git_sha": git_sha,
        "lat_bin": str(lat),
        "instrument": "fresh-child peak RSS (ru_maxrss via os.wait4) — diagnosis family",
        "denominator": "x-of-input (the CLI lane's read-row denominator)",
        "group": GROUP,
        "reps": args.reps,
        "loadavg_start": loadavg_start,
        "passes": passes,
        "output_byte_identity": identity,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(out, indent=2) + "\n")
    print(f"written: {args.out.relative_to(REPO)}")
    if not all(identity.values()):
        print("BYTE-IDENTITY FAILURE — a variant changed the output", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
