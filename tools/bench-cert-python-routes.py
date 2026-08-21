#!/usr/bin/env python3
"""What the three Python routes to "is this file valid?" cost today (#271).

The CLI half of this measurement (`tools/bench-cert-parse-share.py`) shows what
a vouched certificate is worth in total. It cannot answer #271's actual
question, because #271 is not "is the certificate fast" — it is whether the free
`validate()` should gain `index=` GIVEN that Python can already reach the
certificate through the handle route. So the comparison that decides it is
between two things Python can do, plus the one it cannot:

    validate(path)                      parse + rule engine        (today, plain)
    read(path, index=cert).validate()   parse + cert, rules skipped (today, best)
    validate(path, index=cert)          neither                     (the proposal)

The third does not exist — that is the issue — so the CLI stands in for it, and
the two figures are compared across surfaces with the CLI's process floor
subtracted. Cross-surface subtraction is a weaker instrument than one process
timing itself, which is why the CLI script measures its floor explicitly and
this one reports the in-process routes separately rather than folding everything
into a single ratio.

WHAT THE HANDLE ROUTE ALSO PAYS, and why it is reported apart: `read()` does not
merely parse — it loads the file into DuckDB, which the proposed free validate
would not do. Counting that as "parse" would inflate the apparent win and bias
the answer toward building. So `read(path)` is timed on its own, and the parse
share is bounded rather than asserted.

Everything runs in-process through the public `laterite` import surface — no
native entry point, per #271.

Usage:
    uv run --no-sync python tools/bench-cert-python-routes.py
"""

from __future__ import annotations

import argparse
import gc
import statistics
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "output" / "bench-fixtures"


def timed(fn: Callable[[], object], repeats: int) -> list[float]:
    """Seconds per call, first discarded — it pays for the page cache, and the
    engine's one-off lazy initialisation, which no later call repeats."""
    out = []
    for _ in range(repeats + 1):
        gc.collect()
        t0 = time.perf_counter()
        fn()
        out.append(time.perf_counter() - t0)
    return out[1:]


def summarise(samples: list[float]) -> dict[str, float]:
    return {
        "min": min(samples),
        "median": statistics.median(samples),
        "spread_pct": (max(samples) - min(samples)) / min(samples) * 100,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repeats", type=int, default=10)
    ap.add_argument("--fixture", default="large.ags")
    args = ap.parse_args()

    import laterite

    ags = FIXTURES / args.fixture
    cert = FIXTURES / f"{args.fixture}.idx"
    for p in (ags, cert):
        if not p.is_file():
            sys.exit(f"{p} missing — see tools/gen-bench-fixtures.sh and `lat certify`")

    size_mb = ags.stat().st_size / 1_000_000
    path = str(ags)
    idx = str(cert)

    # Prove the cert is actually being consumed before timing anything with it.
    # A certificate that is stale, or for another engine, falls back to a full
    # run in silence — which would look like "the certificate saves nothing"
    # and answer #271 backwards.
    # `Ags4File.validate()` is fluent — it returns the handle, and the verdict
    # arrives on `.report`. Worth the extra hop here rather than a bare call:
    # `.certified` is the only evidence the certificate was consumed at all.
    probe = laterite.read(path, index=idx).validate().report
    print(f"cert accepted by the handle route: certified={probe.certified}")
    if not probe.certified:
        sys.exit(
            "the certificate was NOT honoured, so every figure below would be "
            "two full validate runs wearing different names. Re-mint it with "
            "the same engine build: `lat certify` from this checkout."
        )

    plain = timed(lambda: laterite.validate(path), args.repeats)
    handle = timed(lambda: laterite.read(path, index=idx).validate(), args.repeats)
    read_only = timed(lambda: laterite.read(path), args.repeats)

    p, h, r = (summarise(s) for s in (plain, handle, read_only))
    plain_again = summarise(timed(lambda: laterite.validate(path), args.repeats))
    drift = abs(plain_again["median"] - p["median"]) / p["median"] * 100

    print(f"\n{args.fixture} — {size_mb:.1f} MB, {args.repeats} repeats, in-process")
    print("─" * 72)
    for label, s in (
        ("validate(path)", p),
        ("read(path, index=).validate()", h),
        ("read(path)  [parse + DuckDB]", r),
        ("validate(path) again (A/B/A)", plain_again),
    ):
        print(
            f"  {label:<32} min {s['min'] * 1000:8.1f} ms   "
            f"median {s['median'] * 1000:8.1f} ms   spread {s['spread_pct']:5.1f}%"
        )
    print("─" * 72)
    print(f"  baseline drift between the two identical runs: {drift:.1f}%")
    if drift > 10:
        print(
            "\n  THE MACHINE MOVED. The two identical baselines disagree by more\n"
            "  than a tenth; nothing below is resolvable on this run."
        )
        return 2

    rules_saved = p["median"] - h["median"]
    print(
        f"\n  what the handle route already saves: {rules_saved * 1000:8.1f} ms "
        f"({rules_saved / p['median'] * 100:.1f}% of validate(path))"
    )
    print(
        f"  what it still pays (parse + DuckDB): {h['median'] * 1000:8.1f} ms\n"
        f"  of which read(path) alone accounts for {r['median'] * 1000:.1f} ms — "
        "the DuckDB load\n  the proposed door would NOT pay, so treat it as an "
        "upper bound on parse."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
