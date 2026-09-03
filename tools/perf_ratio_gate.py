#!/usr/bin/env python3
"""The Linux memory-ratio gate (#894, campaign 2's T2) — seed and check.

WHY this exists: the #826 close-out declined a ratio gate because a fresh
environment could not mint the corpus (#873) and no runner discipline
existed. Both halves fell — corpus-v2 made the fixtures mintable anywhere,
and the amended revisit condition (owner call, 2026-09-03, on
`ags-wiki/concepts/perf-campaign.md`) replaced the dedicated-runner demand
with what the reasoning actually requires: the same hardware class run to
run, a baseline pair seeded fresh, and a dirty-run refusal guard. This
script is those three things, over the existing instrument.

The instrument is `bench-vs-python-ags4.py --skip-time` — the
fresh-subprocess peak-RSS pass, both libraries' doors, corpus-v2 pinned
fixtures — run as a child so every measured cell is that harness's, not a
re-implementation (#865's drift class). This gate only wraps it: guard
readings around each pass, the pair on seed, the comparison on check.

The claim family (ledger rule 8): these are LINUX numbers on the pool's
hardware class. They never "hold" the darwin ledger's cells — the committed
baseline founds this family's own history, and a check compares within the
family only (a hardware-class mismatch refuses the comparison by name).

Modes
-----
--mode seed   Two guarded back-to-back passes (the pair). The pair's spread
              is the family's founding drift record. A pressured leg FAILS
              the seed loudly (exit 2) — a family is not founded on a dirty
              pair. Output: the baseline doc (commit it as
              `tools/perf-results/linux-mem-baseline.json`).
--mode check  One guarded pass, compared per (axis, rung, ours-door)
              against the committed baseline: the measured `x_output` FAILS
              the gate (exit 1) when it worsens more than --band (default
              10% relative — the campaign's tranche floor) past the pair's
              worse leg. Upstream (`python_ags4`) cells are recorded for
              the family's history, never gated — ×-of-output is the gated
              quantity (#894). A pressured run withholds the verdict and
              reports itself as a SKIP (exit 0, the skip recorded in the
              artifact): contention degrades to a recorded skip, never a
              flake.

The dirty-run refusal guard (both modes): load average and swap usage are
recorded at each pass's start and end; the pass is dirty when swap grew
past the bench's own refusal threshold. The bench already refuses per-cell
on swap — this pass-level guard catches growth between cells. Load is
recorded, not gated: the ledger's own runner-choice note (perf-probe.yml)
records that peak memory, unlike timing, is robust to CPU contention.

The die-guard (the M1 probe's lesson): a check where every ours-door cell
was refused measured nothing and says so with exit 2 — a green run must
mean verdicts existed.

Runs under the probe venv's python (the measured wheel + pandas<3 +
python-ags4 importable); the bench child inherits the interpreter.
"""

from __future__ import annotations

import argparse
import datetime
import importlib.util
import json
import os
import platform
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
BENCH = REPO / "tools" / "bench-vs-python-ags4.py"
BASELINE_DEFAULT = REPO / "tools" / "perf-results" / "linux-mem-baseline.json"
OUT_DIR_DEFAULT = REPO / "output" / "perf-ratio"
SCHEMA = "laterite-linux-mem-ratio/1"
UPSTREAM_DOOR = "python_ags4"


def load_bench_module() -> Any:
    """The bench's helpers (swap reading, thresholds) — imported, not copied,
    so the two can't drift. Its module level is stdlib-only by design."""
    spec = importlib.util.spec_from_file_location("bench_vs_python_ags4", BENCH)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["bench_vs_python_ags4"] = mod
    spec.loader.exec_module(mod)
    return mod


bench = load_bench_module()


def hardware_class() -> dict[str, Any]:
    """What must match run-to-run for a comparison to mean anything. Exact
    equality of this dict is the class test — coarse on purpose (a kernel
    bump does not change the class; a different machine does)."""
    model = ""
    if sys.platform == "linux":
        try:
            for line in Path("/proc/cpuinfo").read_text().splitlines():
                if line.lower().startswith("model name"):
                    model = line.split(":", 1)[1].strip()
                    break
        except OSError:
            pass
    mem = bench.mem_total_bytes()
    return {
        "platform": sys.platform,
        "machine": platform.machine(),
        "cpu_model": model,
        "cpu_count": os.cpu_count(),
        "mem_total_gb": round(mem / 1e9) if mem else None,
    }


def guard_readings() -> dict[str, Any]:
    load = os.getloadavg() if hasattr(os, "getloadavg") else (None, None, None)
    return {
        "load_avg": list(load),
        "swap_used_bytes": bench.swap_used_bytes(),
    }


def pass_is_dirty(start: dict[str, Any], end: dict[str, Any]) -> str | None:
    """The refusal condition: swap growth past the bench's own threshold.
    Returns the detail string, or None for a clean pass."""
    s0, s1 = start.get("swap_used_bytes"), end.get("swap_used_bytes")
    if s0 is not None and s1 is not None and s1 - s0 > bench.SWAP_REFUSAL_BYTES:
        return f"swap grew {(s1 - s0) / 1e6:.1f} MB during the pass"
    return None


def run_pass(rungs: str, scratch: Path, tag: str) -> dict[str, Any]:
    """One guarded bench memory pass. Returns {guard_start, guard_end,
    dirty, result} where result is the bench's own JSON document."""
    out = scratch / f"{tag}.json"
    argv = [
        sys.executable,
        str(BENCH),
        "--skip-time",
        "--rungs",
        rungs,
        "--mem-rungs",
        rungs,
        "--out",
        str(out),
    ]
    start = guard_readings()
    proc = subprocess.run(argv, cwd=REPO, check=False)
    end = guard_readings()
    if proc.returncode != 0:
        raise SystemExit(
            f"error: the bench pass '{tag}' failed (exit {proc.returncode})"
        )
    return {
        "guard_start": start,
        "guard_end": end,
        "dirty": pass_is_dirty(start, end),
        "result": json.loads(out.read_text()),
    }


def mem_cells(leg: dict[str, Any]) -> dict[str, dict[str, dict[str, Any]]]:
    return leg["result"].get("memory", {})


def ours_doors(
    cells: dict[str, dict[str, dict[str, Any]]],
) -> list[tuple[str, str, str]]:
    """Every (axis, rung, door) that is ours — discovered from the data, so a
    new door (e.g. write's unchecked) joins the gate without an edit here."""
    return sorted(
        (axis, rung, door)
        for axis, rungs in cells.items()
        for rung, doors in rungs.items()
        for door in doors
        if door != UPSTREAM_DOOR
    )


def compare(
    baseline: dict[str, Any], fresh_leg: dict[str, Any], band: float
) -> dict[str, Any]:
    """The gate's verdict over one fresh pass vs the committed pair. Only
    measured-vs-measured cells gate; refusals are carried as notes (a fresh
    `swapped`/`failed` marks the cell pressured, a `beyond-mem-cap` is
    structural and expected to match)."""
    legs = [mem_cells(leg) for leg in baseline["legs"]]
    fresh = mem_cells(fresh_leg)
    regressions: list[str] = []
    notes: list[str] = []
    verdict_cells = 0
    pressured_cells = 0
    for axis, rung, door in ours_doors(fresh):
        cell = fresh[axis][rung][door]
        base = [leg.get(axis, {}).get(rung, {}).get(door) for leg in legs]
        base = [c for c in base if c is not None]
        if "refusal" in cell:
            if cell["refusal"] in ("swapped", "failed"):
                pressured_cells += 1
                notes.append(f"{axis}/{rung}/{door}: fresh refusal ({cell['refusal']})")
            continue
        base_measured = [c for c in base if c and "refusal" not in c]
        if not base_measured:
            notes.append(f"{axis}/{rung}/{door}: no measured baseline cell — not gated")
            continue
        worse = max(c["x_output"] for c in base_measured)
        verdict_cells += 1
        if cell["x_output"] > worse * (1.0 + band):
            regressions.append(
                f"{axis}/{rung}/{door}: x_output {cell['x_output']} vs baseline "
                f"worse-leg {worse} (band {band:+.0%})"
            )
    return {
        "verdict_cells": verdict_cells,
        "pressured_cells": pressured_cells,
        "regressions": regressions,
        "notes": notes,
    }


def doc_header(args: argparse.Namespace) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "mode": args.mode,
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "measured_sha": args.measured_sha or None,
        "instrument_sha": args.instrument_sha or None,
        "hardware_class": hardware_class(),
        "band": args.band,
        "rungs": args.rungs,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--mode", choices=["seed", "check"], required=True)
    ap.add_argument("--rungs", default="5MB,25MB,100MB,265MB")
    ap.add_argument("--band", type=float, default=0.10)
    ap.add_argument("--baseline", type=Path, default=BASELINE_DEFAULT)
    ap.add_argument("--out", type=Path, default=None)
    ap.add_argument("--measured-sha", default="")
    ap.add_argument("--instrument-sha", default="")
    args = ap.parse_args()

    if sys.platform != "linux" and not os.environ.get("LATERITE_RATIO_GATE_ANY_OS"):
        raise SystemExit(
            "error: this gate is the LINUX claim family (#894). Set "
            "LATERITE_RATIO_GATE_ANY_OS=1 only to smoke the plumbing — such a "
            "run founds and holds nothing."
        )

    out_path = args.out or (OUT_DIR_DEFAULT / f"{args.mode}.json")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    doc = doc_header(args)

    with tempfile.TemporaryDirectory() as td:
        scratch = Path(td)
        if args.mode == "seed":
            legs = [run_pass(args.rungs, scratch, f"leg{i + 1}") for i in range(2)]
            doc["legs"] = legs
            dirty = [leg["dirty"] for leg in legs if leg["dirty"]]
            # A dirty or refusal-carrying leg fails the SEED loudly: the pair
            # founds the family's history and must be clean by construction.
            refused = [
                f"{a}/{r}/{d}"
                for leg in legs
                for (a, r, d) in ours_doors(mem_cells(leg))
                if mem_cells(leg)[a][r][d].get("refusal") in ("swapped", "failed")
            ]
            out_path.write_text(json.dumps(doc, indent=2) + "\n")
            if dirty or refused:
                print(f"seed: DIRTY — not a baseline. dirty={dirty} refused={refused}")
                return 2
            print(f"seed: clean pair written to {out_path} — commit it as the baseline")
            return 0

        if not args.baseline.exists():
            raise SystemExit(f"error: no baseline at {args.baseline} — seed first")
        baseline = json.loads(args.baseline.read_text())
        if baseline.get("hardware_class") != doc["hardware_class"]:
            doc["skip"] = {
                "reason": "hardware-class-mismatch",
                "detail": {
                    "baseline": baseline.get("hardware_class"),
                    "this_run": doc["hardware_class"],
                },
            }
            out_path.write_text(json.dumps(doc, indent=2) + "\n")
            print(
                "check: SKIP — hardware class differs from the baseline's; a "
                "cross-class comparison holds nothing (re-seed on this class)."
            )
            return 0
        leg = run_pass(args.rungs, scratch, "check")
        doc["leg"] = leg
        if leg["dirty"]:
            doc["skip"] = {"reason": "pressured", "detail": leg["dirty"]}
            out_path.write_text(json.dumps(doc, indent=2) + "\n")
            print(
                f"check: SKIP — {leg['dirty']}; verdict withheld (recorded, not flaked)"
            )
            return 0
        verdict = compare(baseline, leg, args.band)
        doc["verdict"] = verdict
        out_path.write_text(json.dumps(doc, indent=2) + "\n")
        for note in verdict["notes"]:
            print(f"check: note — {note}")
        if verdict["verdict_cells"] == 0:
            print(
                "error: no verdicts — every ours-door cell was refused (the "
                "M1 probe's die-guard: a green run must mean something ran)"
            )
            return 2
        if verdict["regressions"]:
            print("check: FAIL — memory regressions past the band:")
            for r in verdict["regressions"]:
                print(f"  {r}")
            return 1
        print(
            f"check: PASS — {verdict['verdict_cells']} cell(s) inside the band "
            f"({verdict['pressured_cells']} pressured cell(s) recorded, not judged)"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
