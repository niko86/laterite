#!/usr/bin/env python3
"""The CLI leg of the cross-surface performance matrix (#825).

Drives the shipped Rust `lat` binary over the forge ladder
(`tools/perf-ladder.py` → `output/perf-ladder/manifest.json`) and writes the
matrix's uniform per-surface schema (schema 2: `{surface, results:[{op, rung,
bytes, median_ms, throughput_mb_s, door, mem?}], skipped:[{rung, reason}]}`)
into `output/perf-results/cli.json` for `tools/perf-matrix.py` to merge.

Which `lat`
-----------
Three programs answer to `lat` (the Rust binary, the wheel's console script,
the Node launcher), so this harness NEVER resolves from `PATH`: `--lat-bin`,
else `$LAT_BIN`, else this checkout's release build — and it dies loudly
rather than fall through, prints what it resolved, and records the path and
`lat --version` in the artifact (`lat_bin`), because a measurement whose
subject depends on the caller's environment must at least say what it
measured.

The doors
---------
Every cell is one FRESH `lat` subprocess, spawn included — that is what a
CLI caller pays, and it is the named skew against the in-process lanes'
timed loops. Each row carries a `door` string naming the invocation.

  validate   `lat validate <rung> --quiet` — the shared axis (the same rule
             engine end-to-end), so it joins the other surfaces' validate
             block. Two skews, named: process spawn + report rendering ride
             inside the time, and the CLI's surface default keeps the
             warnings tier ON where the rust bin's `CheckOptions::default()`
             has it off (the tier the campaign's T5 tranche priced — the
             wasm lane's skew, same direction; the price lives on the
             ledger, not here).
  read       `lat read <rung> <G> --csv --out … --quiet` — the read door as
             exposed: tolerant-parse the whole file, render ONE group as
             CSV, write it atomically. `<G>` is the rung's bulk group,
             picked by streaming the file once (recorded in the door
             string, never assumed from the scaffold). Deliberately NOT
             named `parse-to-typed`: no typed materialisation happens, so
             the op gets its own matrix block rather than a false pairing.
  merge      `lat merge <rung> <rung> --out … --quiet` — the write door as
             exposed. The CLI has no build-from-held-input verb, and `fix`
             on a clean file returns the source verbatim (a byte copy, not
             a write) — `merge` is the one verb that drives the emit engine
             (`emit_ags4`). Self-merge: parse the rung twice, KEY-dedup
             (last wins), emit the union. Its own matrix block, for the
             same honesty reason; the emitted size is recorded beside the
             memory cell (`out_bytes`) so a reader can see what the door
             produced.

Memory
------
`mem` is the campaign's peak-RSS instrument (epic #820 decision 1): the CLI
is a subprocess already, so the measured child IS `lat` — one invocation per
(op, rung), its own `ru_maxrss` read per-child via `os.wait4` (never
`RUSAGE_CHILDREN`, whose high-water mark is over ALL reaped children). The
shared contract — the 265 MB cap, the `beyond-mem-cap`/`swapped`/`failed`
refusal vocabulary, the swap watch, the cell shapes — is IMPORTED from the
python lane's harness (`tools/bench-vs-python-ags4.py`) rather than copied:
#865 records that the per-lane copies agree by discipline only, so this lane
adds a consumer, not a fifth copy. The memory pass runs FIRST (the node
lane's ordering) — this parent holds nothing either way, but the siblings'
shape is kept so the lanes stay comparable by construction.

x_output denominators follow the lanes: input bytes for validate/read, the
emitted size for the write door (`out_bytes or input` — a genuinely-zero
out_bytes falls back rather than reaching the division).

Usage (needs the ladder manifest; scratch output is cleaned at exit):
    uv run python tools/perf-cli.py
    uv run python tools/perf-cli.py --lat-bin <p> [--manifest <p>] [--out <p>]
                                    [--iters N] [--skip-mem]
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = REPO / "output" / "perf-ladder" / "manifest.json"
DEFAULT_OUT = REPO / "output" / "perf-results" / "cli.json"
SCRATCH = REPO / "output" / "perf-cli-scratch"


def _load_bench() -> Any:
    """Import `tools/bench-vs-python-ags4.py` — the shared memory contract's
    single python copy (cap, refusal cells, swap watch, `ru_maxrss` units).
    The hyphenated name is not importable, and `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "bench_vs_python_ags4", REPO / "tools" / "bench-vs-python-ags4.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["bench_vs_python_ags4"] = mod
    spec.loader.exec_module(mod)
    return mod


bench = _load_bench()


def die(msg: str) -> None:
    """Exit with the message ON the exception, so a caller (or test) sees why."""
    raise SystemExit(f"error: {msg}")


# --- pure seams (pinned by tests/test_perf_cli_lane.py) --------------------


def resolve_lat(env: dict[str, str], repo: Path) -> Path:
    """`LAT_BIN` if set (dying if it points nowhere — silently measuring a
    fallback would publish numbers for the wrong program), else this
    checkout's release build, else a loud death. NEVER `PATH`."""
    if pinned := env.get("LAT_BIN"):
        p = Path(pinned)
        if not p.exists():
            die(f"LAT_BIN points at {p}, which does not exist")
        return p
    built = repo / "rust-packages" / "target" / "release" / "lat"
    if built.exists():
        return built
    die(
        "no lat binary: set LAT_BIN, or build this checkout's with "
        "`cargo build --release --manifest-path rust-packages/Cargo.toml "
        "-p laterite-cli`"
    )
    raise AssertionError("unreachable")


def median(samples: list[float]) -> float:
    """Upper-middle sample of the sorted values — the len/2 pick every lane
    uses, kept identical so the matrix's medians mean one thing."""
    s = sorted(samples)
    return s[len(s) // 2]


def throughput_mb_s(nbytes: int, median_ms: float) -> float:
    """Decimal MB/s (MB = 1e6 bytes, matching forge's parse_size): the
    cross-surface throughput headline. Guards the degenerate timing."""
    if median_ms <= 0:
        return 0.0
    return nbytes / (median_ms * 1000.0)


def denominator(out_bytes: int | None, input_bytes: int) -> int:
    """`out_bytes or input` — truthiness on purpose: a genuinely-zero
    out_bytes must fall back, never reach the division (#824's `||`-not-`??`
    lesson). Floor 1 so an empty rung cannot divide by zero."""
    return max(out_bytes or input_bytes, 1)


def failure_detail(returncode: int | None, stderr_text: str) -> str:
    """Why a child failed: the signal when it was killed (an OOM-SIGKILLed
    child has a negative returncode and often an EMPTY stderr — "exit -9"
    hides the mechanism), else the stderr tail, else the exit code."""
    tail = " | ".join(stderr_text.strip().splitlines()[-3:])
    if returncode is not None and returncode < 0:
        return f"signal {-returncode}" + (f": {tail}" if tail else "")
    return tail or f"exit {returncode}"


def acceptable_exit(op: str, returncode: int | None) -> bool:
    """`lat validate` exits 1 on findings — a completed validation, not a
    failure. The read/write doors succeed only at 0."""
    if op == "validate":
        return returncode in (0, 1)
    return returncode == 0


def bulk_group(path: Path) -> tuple[str, int]:
    """The group with the most DATA rows, by streaming the file once — the
    read door dumps ONE group, so the choice must be deterministic and
    recorded, never an assumption about the forge scaffold. Returns
    (code, data_row_count)."""
    counts: dict[str, int] = {}
    current: str | None = None
    group_prefix = '"GROUP","'
    with path.open(encoding="utf-8", errors="replace") as f:
        for line in f:
            if line.startswith('"DATA",'):
                if current is not None:
                    counts[current] += 1
            elif line.startswith(group_prefix):
                current = line[len(group_prefix) :].split('"', 1)[0]
                counts.setdefault(current, 0)
    if not any(counts.values()):
        die(f"{path}: no DATA rows — cannot pick the read door's group")
    return max(counts.items(), key=lambda kv: kv[1])


def measurement(
    op: str, rung: str, nbytes: int, ms: float, door: str
) -> dict[str, Any]:
    """One timed row. `door` names the exact invocation shape: the CLI's
    read/write doors are not the other surfaces' ops, and the artifact
    outlives the run, so every row says what was measured."""
    return {
        "op": op,
        "rung": rung,
        "bytes": nbytes,
        "median_ms": ms,
        "throughput_mb_s": throughput_mb_s(nbytes, ms),
        "door": door,
    }


def build_output(
    iters: int,
    results: list[dict[str, Any]],
    skipped: list[dict[str, str]],
    lat_bin: dict[str, str],
) -> dict[str, Any]:
    """The matrix's uniform per-surface document, plus the measured binary's
    identity. `skipped` serialises even when empty — a positive statement
    that nothing was dropped."""
    return {
        "schema": 2,
        "surface": "cli",
        "tool": "tools/perf-cli.py",
        "iters": iters,
        "lat_bin": lat_bin,
        "results": results,
        "skipped": skipped,
    }


# --- the doors --------------------------------------------------------------


@dataclass
class Door:
    """One invocation shape, built once per rung so the timed loop and the
    memory child run the same argv by construction. `out` is the path whose
    size is the op's `out_bytes` (the write door only — the read door's CSV
    is one group's slice, not the operation's product); `label` is the
    path-free description the artifact's `door` field carries."""

    op: str
    argv: list[str]
    out: Path | None
    label: str


class DoorFailed(Exception):
    """A door failed during the timed pass. Raised, not died on: one bad
    (op, rung) must land in the artifact's `skipped` list, never destroy
    the measurements already taken — the wasm lane's lesson (#824)."""


def doors(lat: Path, rung_path: Path, group: str, scratch: Path) -> list[Door]:
    """The three doors for one rung."""
    read_out = scratch / "read.csv"
    merge_out = scratch / "merged.ags"
    return [
        Door(
            op="validate",
            argv=[str(lat), "validate", str(rung_path), "--quiet"],
            out=None,
            label="lat validate <rung> --quiet",
        ),
        Door(
            op="read",
            argv=[
                str(lat),
                "read",
                str(rung_path),
                group,
                "--csv",
                "--out",
                str(read_out),
                "--quiet",
            ],
            out=None,
            label=f"lat read <rung> {group} --csv --out <scratch> --quiet",
        ),
        Door(
            op="merge",
            argv=[
                str(lat),
                "merge",
                str(rung_path),
                str(rung_path),
                "--out",
                str(merge_out),
                "--quiet",
            ],
            out=merge_out,
            label="lat merge <rung> <rung> --out <scratch> --quiet",
        ),
    ]


def run_child(door: Door, stderr_path: Path) -> tuple[int, int]:
    """One fresh `lat` child, reaped with `os.wait4` so the peak RSS is THIS
    child's own — `RUSAGE_CHILDREN` high-waters over every reaped child, so
    a small child after a big one would inherit the big one's number.
    Returns (returncode, peak_rss_bytes)."""
    with stderr_path.open("wb") as errf:
        proc = subprocess.Popen(door.argv, stdout=subprocess.DEVNULL, stderr=errf)
        _, status, ru = os.wait4(proc.pid, 0)
        # Tell Popen the child is reaped; a second waitpid would raise.
        proc.returncode = os.waitstatus_to_exitcode(status)
    sysname = "darwin" if sys.platform == "darwin" else "linux"
    return proc.returncode, bench.maxrss_to_bytes(ru.ru_maxrss, sysname)


def measure_mem(door: Door, input_bytes: int) -> dict[str, Any]:
    """One (op, rung) memory cell: a fresh child, swap watched across the
    run. Every veto is a recorded refusal, never a silent skip — the shared
    semantics, through the shared code."""
    if not bench.mem_rung_allowed(input_bytes):
        return bench.refusal_cell(
            "beyond-mem-cap",
            f"{input_bytes}-byte rung is past the {bench.MEM_CAP_BYTES}-byte "
            "cap (epic #820 decision 7: a swapping run measures the pager)",
        )
    stderr_path = SCRATCH / "mem-child.stderr"
    swap_before = bench.swap_used_bytes()
    try:
        returncode, peak = run_child(door, stderr_path)
    except OSError as e:
        # Covers the spawn and the wait4 reap alike — either way there is no
        # child number to report, only the OS's reason.
        return bench.refusal_cell("failed", f"spawn/reap: {e}")
    swap_after = bench.swap_used_bytes()
    if not acceptable_exit(door.op, returncode):
        stderr_text = stderr_path.read_text(errors="replace")
        return bench.refusal_cell("failed", failure_detail(returncode, stderr_text))
    if (
        swap_before is not None
        and swap_after is not None
        and swap_after - swap_before > bench.SWAP_REFUSAL_BYTES
    ):
        grew = (swap_after - swap_before) / 1e6
        return bench.refusal_cell("swapped", f"swap grew {grew:.1f} MB during the run")
    out_bytes = None
    if door.out is not None and door.out.exists():
        out_bytes = door.out.stat().st_size
    cell = bench.mem_cell(peak, denominator(out_bytes, input_bytes))
    if out_bytes is not None:
        cell["out_bytes"] = out_bytes
    return cell


def timed_ms(door: Door, iters: int) -> float:
    """Warm up one untimed run, then the median wall time over `iters` fresh
    children — spawn included, because a CLI caller pays it. Output sinks to
    /dev/null (the cheapest honest stand-in for a terminal). A door that
    stops completing raises `DoorFailed` — the caller records the skip and
    the artifact still gets written."""
    samples = []
    for i in range(iters + 1):
        t = time.perf_counter()
        proc = subprocess.run(
            door.argv,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        elapsed = (time.perf_counter() - t) * 1000.0
        if not acceptable_exit(door.op, proc.returncode):
            raise DoorFailed(
                f"{door.label} exited {proc.returncode} during the timed pass"
            )
        if i > 0:
            samples.append(elapsed)
    return median(samples)


@dataclass
class Rung:
    """One on-disk ladder rung, plus the read door's chosen bulk group."""

    label: str
    path: Path
    bytes: int
    group: str = ""


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--iters", type=int, default=10)
    ap.add_argument("--skip-mem", action="store_true")
    ap.add_argument(
        "--lat-bin",
        type=Path,
        default=None,
        help="the lat binary to measure (else $LAT_BIN, else this checkout's "
        "release build; never PATH)",
    )
    args = ap.parse_args()
    if args.iters < 1:
        die("--iters needs a positive integer")

    if args.lat_bin is not None:
        if not args.lat_bin.exists():
            die(f"--lat-bin points at {args.lat_bin}, which does not exist")
        lat = args.lat_bin
    else:
        lat = resolve_lat(dict(os.environ), REPO)
    version = subprocess.run(
        [str(lat), "--version"], capture_output=True, text=True, check=False
    ).stdout.strip()
    print(f"perf-cli.py: measuring {lat} ({version or 'no --version output'})")

    try:
        manifest = json.loads(args.manifest.read_text())
    except OSError as e:
        die(
            f"read ladder manifest {args.manifest}: {e} — run `uv run python tools/perf-ladder.py` first"
        )

    rungs: list[Rung] = []
    skipped: list[dict[str, str]] = []
    for entry in manifest["rungs"]:
        path = Path(entry["path"])
        if not path.exists():
            print(
                f"perf-cli.py: rung {entry['label']} missing ({path}) — "
                "skipping (re-run `uv run python tools/perf-ladder.py`)",
                file=sys.stderr,
            )
            skipped.append(
                {"rung": entry["label"], "reason": f"missing on disk: {path}"}
            )
            continue
        rungs.append(Rung(entry["label"], path, path.stat().st_size))

    SCRATCH.mkdir(parents=True, exist_ok=True)

    # The read door's group, per rung — chosen from the data, outside any
    # timed or measured window, and recorded in the door string.
    for rung in rungs:
        code, nrows = bulk_group(rung.path)
        rung.group = code
        print(
            f"perf-cli.py: {rung.label} read door dumps {code} ({nrows} DATA rows)",
            file=sys.stderr,
        )

    # Memory pass FIRST (the node lane's ordering, kept for comparability by
    # construction — this parent never holds rung data either way).
    mem_cells: dict[str, dict[str, Any]] = {}
    if not args.skip_mem:
        for rung in rungs:
            print(f"perf-cli.py: {rung.label} memory children", file=sys.stderr)
            cells: dict[str, Any] = {}
            for door in doors(lat, rung.path, rung.group, SCRATCH):
                cells[door.op] = measure_mem(door, rung.bytes)
            mem_cells[rung.label] = cells

    results = []
    for rung in rungs:
        print(
            f"perf-cli.py: {rung.label} ({rung.bytes} bytes) × {args.iters} iters",
            file=sys.stderr,
        )
        for door in doors(lat, rung.path, rung.group, SCRATCH):
            # A door failing mid-ladder is recorded, and every measurement
            # already taken still reaches the artifact (the wasm lesson —
            # one bad cell must not lose the run).
            try:
                ms = timed_ms(door, args.iters)
            except DoorFailed as e:
                print(f"perf-cli.py: {rung.label} {e} — skipping", file=sys.stderr)
                skipped.append({"rung": rung.label, "reason": f"{door.op}: {e}"})
                continue
            row = measurement(door.op, rung.label, rung.bytes, ms, door.label)
            cells_for_rung = mem_cells.get(rung.label)
            if cells_for_rung:
                row["mem"] = cells_for_rung[door.op]
            results.append(row)

    output = build_output(
        args.iters, results, skipped, {"path": str(lat), "version": version}
    )
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(output, indent=2) + "\n")
    print(
        f"perf-cli.py: wrote {len(results)} measurements "
        f"({len(skipped)} rung(s) skipped) → {args.out}",
        file=sys.stderr,
    )
    shutil.rmtree(SCRATCH, ignore_errors=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
