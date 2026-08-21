#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""How much of a validate run is PARSE, and how much is the rule engine? (#271)

#271 asks whether the free `validate()` on Python and Node should gain an
`index=` parameter, the way `read()` already has one. The code path it would
unlock is real and already written — a vouched certificate with no world check
returns WITHOUT PARSING, hashing the bytes and reading the stamp — but the size
of the win is unmeasured, and a public parameter on two published surfaces is
not worth adding for a saving nobody has seen.

WHAT IS ALREADY POSSIBLE decides the question, not what is fastest overall.
Python can reach the certificate today through the handle route:

    read(path, index=cert).validate()      # parses, then skips the rule engine

so the INCREMENTAL win of the proposal is not "rules skipped" — that is already
available — it is "parse skipped as well". The number that decides #271 is
therefore the parse share of a validate run, which is what this measures.

MEASURED THROUGH THE CLI, ON PURPOSE. #271 requires the comparison be
end-to-end through a public API rather than a native entry point, and `lat` is
the one shipped surface that ALREADY has the door the proposal would add:
`lat validate --index` is a free validate that consumes a certificate. It runs
the same engine the Python parameter would, so it stands in for the proposed
door without anyone having to build it first.

The cost of that choice is process startup, which the Python door would not pay.
It is measured rather than assumed — `FLOOR` below runs the binary on a job that
does no AGS4 work at all, and every figure is reported both raw and floor-
subtracted. A startup cost folded silently into a "parse is X%" claim would
make parsing look cheaper than it is, biasing the answer toward NOT building —
so it is the error worth being loud about.

THE INSTRUMENT'S RESOLUTION IS PART OF THE RESULT. Benchmarks on a laptop drift
between runs, so this reports min / median / spread over N repeats and runs the
baseline TWICE, at the start and the end (A/B/A). If the two baselines disagree
by more than the effect being measured, the machine moved under the experiment
and the run says so instead of reporting a number.

Fixtures come from `tools/gen-bench-fixtures.sh` (`forge scale`) — synthetic,
reproducible from a seed, and carrying no real delivery data.

Usage:
    uv run --no-project python tools/bench-cert-parse-share.py
    uv run --no-project python tools/bench-cert-parse-share.py --repeats 15
"""

from __future__ import annotations

import argparse
import json
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
LAT = REPO / "rust-packages" / "target" / "release" / "lat"
FIXTURES = REPO / "output" / "bench-fixtures"


def timed(cmd: list[str], repeats: int) -> list[float]:
    """Wall-clock seconds per run. Discards the first — it pays for the page
    cache and the dynamic loader, and every later run does not."""
    out = []
    for _ in range(repeats + 1):
        t0 = time.perf_counter()
        proc = subprocess.run(cmd, capture_output=True)
        out.append(time.perf_counter() - t0)
        if proc.returncode not in (0, 1):  # 1 = findings, a legitimate verdict
            sys.exit(
                f"{' '.join(cmd)} exited {proc.returncode}\n"
                f"{proc.stderr.decode(errors='replace')[:800]}"
            )
    return out[1:]


def summarise(samples: list[float]) -> dict[str, float]:
    return {
        "min": min(samples),
        "median": statistics.median(samples),
        "max": max(samples),
        "spread_pct": (max(samples) - min(samples)) / min(samples) * 100,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repeats", type=int, default=10)
    ap.add_argument("--fixture", default="large.ags")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    args = ap.parse_args()

    if not LAT.is_file():
        sys.exit(
            "lat is not built. `cargo build --release -p laterite-cli` in "
            "rust-packages/ — and note this must be a RELEASE build; a debug "
            "binary would measure the wrong engine entirely."
        )
    ags = FIXTURES / args.fixture
    cert = FIXTURES / f"{args.fixture}.idx"
    if not ags.is_file():
        sys.exit(f"{ags} missing — run tools/gen-bench-fixtures.sh first")
    if not cert.is_file():
        sys.exit(f"{cert} missing — run `lat certify {ags}` first")

    size_mb = ags.stat().st_size / 1_000_000

    # The floor: the binary doing no AGS4 work. Everything below is also paying
    # this, and the Python door the issue proposes would not.
    floor = timed([str(LAT), "--version"], args.repeats)

    # A: today's answer to "is this file valid?" — parse + rule engine.
    full_a = timed([str(LAT), "validate", str(ags)], args.repeats)

    # B: the proposed door, already shipped on this surface — vouched cert, no
    # parse, no rules. Hash the bytes, read the stamp, answer.
    vouched = timed(
        [str(LAT), "validate", str(ags), "--index", str(cert)], args.repeats
    )

    # A again. If the machine moved, this is where it shows.
    full_b = timed([str(LAT), "validate", str(ags)], args.repeats)

    f, a, v, a2 = (summarise(s) for s in (floor, full_a, vouched, full_b))

    drift = abs(a2["median"] - a["median"]) / a["median"] * 100
    work_a = a["median"] - f["median"]
    work_v = v["median"] - f["median"]
    saving = work_a - work_v

    result = {
        "fixture": args.fixture,
        "size_mb": round(size_mb, 1),
        "repeats": args.repeats,
        "floor_s": f,
        "validate_full_s": a,
        "validate_vouched_s": v,
        "validate_full_repeat_s": a2,
        "baseline_drift_pct": drift,
        "work_full_s": work_a,
        "work_vouched_s": work_v,
        "saving_s": saving,
        "saving_pct_of_work": saving / work_a * 100 if work_a else 0.0,
    }

    if args.json:
        print(json.dumps(result, indent=2))
        return 0

    w = shutil.get_terminal_size().columns
    print(
        f"\n{args.fixture} — {size_mb:.1f} MB, {args.repeats} repeats\n"
        + "─" * min(w, 72)
    )
    for label, s in (
        ("floor (lat --version)", f),
        ("validate (parse + rules)", a),
        ("validate --index (vouched)", v),
        ("validate again (A/B/A)", a2),
    ):
        print(
            f"  {label:<28} min {s['min'] * 1000:8.1f} ms   "
            f"median {s['median'] * 1000:8.1f} ms   spread {s['spread_pct']:5.1f}%"
        )

    print("─" * min(w, 72))
    print(f"  baseline drift between the two A runs: {drift:.1f}%")
    if drift > 10:
        print(
            "\n  THE MACHINE MOVED UNDER THE EXPERIMENT. The two identical\n"
            "  baselines disagree by more than a tenth, so the split below is\n"
            "  not resolvable here. Re-run on a quiet machine before quoting it."
        )
        return 2

    print(f"\n  AGS4 work, full validate:    {work_a * 1000:8.1f} ms")
    print(f"  AGS4 work, vouched cert:     {work_v * 1000:8.1f} ms")
    print(
        f"  saving the proposal buys:    {saving * 1000:8.1f} ms "
        f"({result['saving_pct_of_work']:.1f}% of the work)"
    )
    print(
        "\n  Read this against what Python can ALREADY do: read(index=).validate()\n"
        "  skips the rules but still parses. The proposal's incremental win is\n"
        "  the parse — so compare the vouched figure with a parse, not with zero."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
