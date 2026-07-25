#!/usr/bin/env python3
"""Reproduce the GIL-release throughput numbers (#8).

The single-threaded criterion/pytest benchmarks are blind to this by construction:
the win is *concurrency*, not per-call latency. The CPU-bound native read/validate
entry points release the GIL for their pure-Rust compute (`Python::detach`), so
running the same total work across N OS threads (a real `ThreadPoolExecutor`,
shared interpreter) now scales with cores instead of serialising behind the lock.

  GIL HELD    -> N threads ≈ 1 thread     (speedup ~1.0x)   [pre-#8]
  GIL RELEASED-> N threads ≈ N× faster     (speedup ~cores)  [post-#8]

Self-contained: a synthetic AGS4 file is generated in-memory (no gitignored
fixture), so this runs on any checkout. `--rows` sizes it; `--path FILE` benches a
real file's bytes instead. The regression *guard* (that the GIL is released at
all) is `packages/laterite/tests/test_gil_released.py`, run every CI; this script
is the on-demand scaling reproducer, like the criterion benches.

Run: `uv run --no-sync python tools/bench-gil-throughput.py`
"""

from __future__ import annotations

import argparse
import os
import time
from concurrent.futures import ThreadPoolExecutor

from laterite import _laterite_native as N


def synthetic_ags(n_rows: int) -> bytes:
    header_lines = [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","BENCH","gil throughput"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE","LOCA_FDEP"',
        '"UNIT","","","m","m"',
        '"TYPE","ID","PA","2DP","2DP"',
    ]
    head = "\r\n".join(header_lines)
    rows = "".join(
        f'\r\n"DATA","BH{i:05d}","CP","{523000 + i}.20","{(i % 50) + 0.25:.2f}"'
        for i in range(n_rows)
    )
    return (head + rows + "\r\n").encode("utf-8")


def wall(op, n_tasks: int, n_workers: int) -> float:
    with ThreadPoolExecutor(max_workers=n_workers) as ex:
        t0 = time.perf_counter()
        list(ex.map(lambda _: op(), range(n_tasks)))
        return time.perf_counter() - t0


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40_000, help="synthetic DATA rows")
    ap.add_argument(
        "--path", type=str, default=None, help="bench a real file's bytes instead"
    )
    ap.add_argument("--workers", type=int, default=os.cpu_count() or 4)
    args = ap.parse_args()

    if args.path:
        from pathlib import Path

        data = Path(args.path).read_bytes()
        label = Path(args.path).name
    else:
        data = synthetic_ags(args.rows)
        label = f"synthetic {args.rows} rows"

    ops = {
        "validate (run_check)": lambda: N.run_check(data=data),
        "read (parse_arrow)": lambda: N.parse_arrow(data=data),
        "compat read (parse_compat_arrow)": lambda: N.parse_compat_arrow(data=data),
    }
    # warm up (first call pays one-time init)
    for op in ops.values():
        op()

    cores = args.workers
    tasks = 2 * cores
    print(f"cores={cores}  tasks={tasks}  fixture={label} ({len(data) / 1e6:.1f} MB)\n")
    for name, op in ops.items():
        t1 = wall(op, tasks, 1)
        tn = wall(op, tasks, cores)
        print(f"{name}")
        print(f"  1-thread    : {t1 * 1e3:7.0f} ms  ({tasks / t1:6.1f} tasks/s)")
        print(f"  {cores}-thread   : {tn * 1e3:7.0f} ms  ({tasks / tn:6.1f} tasks/s)")
        print(
            f"  => speedup {t1 / tn:.2f}x  (efficiency {100 * (t1 / tn) / cores:.0f}%)\n"
        )


if __name__ == "__main__":
    main()
