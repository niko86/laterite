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

Memory — peak RSS, fresh subprocess (#821)
------------------------------------------
Time is measured warm and in-process; memory is not, because an in-process
number inherits every allocation the harness made before the operation ran.
Each (library, axis, rung) memory cell is one FRESH subprocess running one
end-to-end operation through that library's public API, and the cell is the
child's own `ru_maxrss` at exit — the same instrument on both sides, which is
the whole fairness argument. Cross-library memory numbers come ONLY from this
harness; `dhat`/`tracemalloc` are diagnosis instruments on our own code, and
the two kinds of number never share a table (the campaign's rule — see
`ags-wiki/concepts/perf-campaign.md`).

Two verdicts a memory cell can hold instead of a number, both RECORDED rather
than skipped — a skip is a blind spot, a refusal is a result:

- `swapped` — system swap grew while the child ran. A run that pushes the
  machine into swap measures the pager, not the library.
- `failed` — the child died (OOM kill, MemoryError, crash).

Memory rungs stop at the 265MB rung (epic #820 decision 7); the 524MB rung is
time-only. `mem_rung_allowed` enforces the cap; a rung past it is recorded as
a `beyond-mem-cap` refusal.

The results file
----------------
Every run writes a machine-readable record (default
`tools/perf-results/python-lane.json`, committed — the campaign ledger's
machine-readable half). Schema `laterite-python-lane-bench/1`:

    schema            the string above; bump on shape changes
    generated         UTC timestamp of the run
    commit            `git rev-parse HEAD` of the measured tree
    invocation        the rungs/runs arguments that produced this record
    machine           platform, arch, cpu_count, mem_total_bytes, load_avg_start
    versions          python, python-ags4, laterite, pandas, polars
    protocol          prose: what each instrument is, so the file self-describes
    rungs             {name: {bytes, sha256}} — the pinned fixtures measured
    time              {axis: {rung: {door: {seconds, runs}}}} — mean of warm runs
    memory            {axis: {rung: {door: {peak_rss_bytes, x_output}
                                     | {refusal, detail}}}}
    import_baselines  {door: {peak_rss_bytes}} — an import-only child per
                      library: the interpreter+import floor under every cell
    notes             free-form caveats recorded by the run

Axes: `validate`, `read_strings`, `read_typed`, `write`. Doors: `python_ags4`
(the baseline measure), `laterite` (native), `laterite_compat`. The write
axis has three doors — baseline `dataframe_to_AGS4`, compat
`dataframe_to_AGS4` (the streaming door), native `build_ags4().save()` — and
a write cell's peak includes materialising the input through that library's
own read door, because you cannot write what you do not hold; attribute a
write number by comparing it against the same door's read cell.

Usage
-----
    uv run python tools/bench-vs-python-ags4.py                 # default rungs
    uv run python tools/bench-vs-python-ags4.py --rungs 5MB,25MB
    uv run python tools/bench-vs-python-ags4.py --runs 10
    uv run python tools/bench-vs-python-ags4.py --skip-mem      # time only
    uv run python tools/bench-vs-python-ags4.py --update-manifest

Needs `python-ags4` importable (the comparison target) and the release `lat`
toolchain built; both are checked before any timing starts.
"""

from __future__ import annotations

import argparse
import datetime
import gc
import hashlib
import json
import os
import pathlib
import platform
import re
import statistics
import subprocess
import sys
import tempfile
import time
from functools import partial
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable
    from types import ModuleType

REPO = Path(__file__).resolve().parent.parent
# Both rebindable via --fixtures-dir / --manifest (#878): a runner lane pins
# against the current-forge corpus (`tools/perf-probe-fixtures.json`) because
# the README corpus is unmintable off the ledger machine (#873), and the two
# corpora must never share a cache path — same rung names, different bytes.
OUT_DIR = REPO / "output" / "readme-bench"
MANIFEST = REPO / "tools" / "readme-bench-fixtures.json"
FORGE = (
    REPO
    / "rust-packages"
    / "target"
    / "release"
    / ("laterite-ags4-forge.exe" if sys.platform == "win32" else "laterite-ags4-forge")
)

# The README's rungs. Seed 0 and the `wide` scaffold everywhere — the point is a
# FIXED file, not a varied one.
DEFAULT_RUNGS = ["5MB", "25MB", "100MB"]
SEED = 0
SCAFFOLD = "wide"

RESULTS_SCHEMA = "laterite-python-lane-bench/1"
DEFAULT_OUT = REPO / "tools" / "perf-results" / "python-lane.json"
DEFAULT_MEM_RUNGS = ["5MB", "25MB", "100MB", "265MB"]

# Epic #820 decision 7: memory columns stop at the 265MB rung — a bigger run
# pushes the measuring machine into swap, and a swapping run measures the
# pager, not the library (each run's actual RAM is recorded in the results
# file's machine block). The threshold admits the pinned 265MB rung and
# refuses 524MB; tests/test_bench_python_lane.py holds it against the
# committed manifest so it cannot drift apart from the rungs it judges.
MEM_CAP_BYTES = 300_000_000

# Swap growth past this during a child's run marks the cell `swapped`. Small
# enough to catch a real spill, large enough that unrelated background paging
# does not veto a clean run.
SWAP_REFUSAL_BYTES = 64 * 1024 * 1024

# Door identifiers — the results file's vocabulary, shared by every axis.
UPSTREAM_DOOR = "python_ags4"
NATIVE_DOOR = "laterite"
COMPAT_DOOR = "laterite_compat"
# The write axis's fourth door (#881): build_ags4_unchecked — the judged
# build's assembly with the verdict declined. python-ags4's write checks
# nothing either, so this is the closest like-for-like write cell.
UNCHECKED_DOOR = "laterite_unchecked"


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


def check_manifest(paths: dict[str, Path], update: bool) -> dict[str, Any]:
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
        return actual

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
    return actual


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


def fmt_mb(n_bytes: int) -> str:
    return f"{n_bytes / 1e6:.0f} MB"


# --- memory plumbing (pure, pinned by tests/test_bench_python_lane.py) -----


def maxrss_to_bytes(raw: int, sysname: str) -> int:
    """`ru_maxrss` is bytes on Darwin, kibibytes on Linux — getrusage(2)
    differs by lineage, and a unit slip here moves every cell by 1024×."""
    return raw if sysname == "darwin" else raw * 1024


def mem_rung_allowed(rung_bytes: int) -> bool:
    """The epic-#820 cap: memory measurement stops at the 265MB rung."""
    return rung_bytes <= MEM_CAP_BYTES


def mem_cell(peak_bytes: int, denom_bytes: int) -> dict[str, Any]:
    """One measured memory cell. `x_output` is the campaign's headline unit —
    peak as a multiple of the operation's file size, which stays comparable
    across rungs where raw MB does not."""
    return {
        "peak_rss_bytes": peak_bytes,
        "x_output": round(peak_bytes / denom_bytes, 2),
    }


def refusal_cell(reason: str, detail: str) -> dict[str, Any]:
    """A recorded refusal — distinguishable from a measurement by shape, so a
    reader (human or script) cannot mistake a vetoed run for a small number."""
    return {"refusal": reason, "detail": detail}


# --- README table rendering (pure, pinned by tests/test_bench_python_lane.py)
#
# The READMEs may hold only what this instrument prints (the house rule against
# measured values in prose: the tables stay legal because this tool regenerates
# them and `tools/check_speed_claims.py` reads them). So every README-format
# table — the wheel's per-axis tables, the root's condensed pair, and the
# memory tables the #826 close-out promoted — is rendered here, by the run
# that measured it, and pasted verbatim.

# (title, axis key in mem_cells, ours-door key, left header, right header)
MEM_README_PLAN: list[tuple[str, str, str, str, str]] = [
    (
        "Validation — peak RSS",
        "validate",
        NATIVE_DOOR,
        "`python-ags4 check_file`",
        "`laterite.validate`",
    ),
    (
        "Read → typed — peak RSS",
        "read_typed",
        NATIVE_DOOR,
        "`python-ags4` + `convert_to_numeric`",
        "`laterite.read`",
    ),
    (
        "Read → strings — peak RSS",
        "read_strings",
        COMPAT_DOOR,
        "`python-ags4 AGS4_to_dataframe`",
        "`laterite.compat`",
    ),
]


def mem_ratio(pa_cell: dict[str, Any], ours_cell: dict[str, Any]) -> float | None:
    """python-ags4's peak over ours — above 1 means laterite holds less.
    None when either side is a recorded refusal (a vetoed rung renders as no
    row, never as a number)."""
    if "refusal" in pa_cell or "refusal" in ours_cell:
        return None
    return pa_cell["peak_rss_bytes"] / ours_cell["peak_rss_bytes"]


def memory_readme_tables(
    mem_cells: dict[str, dict[str, dict[str, Any]]],
    labels: dict[str, str],
    compat_hop: str,
) -> list[str]:
    """The wheel README's per-axis peak-RSS tables. The bolded ratio is the
    value `check_speed_claims.py` bands, and 'peak RSS' in the header row is
    how it tells a memory table from the time table sharing the API name."""
    lines: list[str] = []
    for title, axis, ours_door, left, right in MEM_README_PLAN:
        rows: list[str] = []
        for size, cell in mem_cells.get(axis, {}).items():
            ratio = mem_ratio(cell[UPSTREAM_DOOR], cell[ours_door])
            if ratio is None:
                continue
            pa, ours = cell[UPSTREAM_DOOR], cell[ours_door]
            rows.append(
                f"| {labels.get(size, size)} "
                f"| {fmt_mb(pa['peak_rss_bytes'])} "
                f"| {fmt_mb(ours['peak_rss_bytes'])} "
                f"| **{ratio:.2f}×** |"
            )
        if not rows:
            continue
        lines += [f"\n**{title}**\n"]
        lines += [f"| File | {left} peak RSS | {right} peak RSS | ratio |"]
        lines += ["|---:|---:|---:|:---:|"]
        lines += rows
    if lines:
        lines += [
            "",
            "Peak RSS of one fresh process per cell; the ratio is python-ags4's",
            "peak over laterite's, so above 1 laterite holds less. The largest",
            "rung is time-only (epic #820 decision 7). Read → strings measured",
            f"on the {compat_hop} hop.",
        ]
    return lines


def condensed_time_table(rows: dict[str, list]) -> list[str]:
    """The root README's one-table summary, from the same run as the wheel's
    per-axis tables so the two can never quote different measurements."""
    lines = [
        "| File (123 groups) | `laterite.validate` | `laterite.read` (typed) "
        "| `laterite.compat` (strings) |",
        "|---:|---:|---:|---:|",
    ]
    for (label, v_up, v_ours), (_, t_up, t_ours), (_, s_up, s_ours) in zip(
        rows["validate"], rows["read_typed"], rows["read_strings"], strict=True
    ):
        lines.append(
            f"| {label} "
            f"| {fmt(v_ours)} · **{v_up / v_ours:.1f}×** "
            f"| {fmt(t_ours)} · **{t_up / t_ours:.1f}×** "
            f"| {fmt(s_ours)} · **{s_up / s_ours:.1f}×** |"
        )
    return lines


def condensed_memory_table(
    mem_cells: dict[str, dict[str, dict[str, Any]]], labels: dict[str, str]
) -> list[str]:
    """The root README's memory summary — same axes, cells `ours · ratio`,
    rendered only for rungs where every door measured (a refusal anywhere
    drops the rung rather than printing a partial row)."""
    plan = [(axis, door) for _, axis, door, _, _ in MEM_README_PLAN]
    sizes = [
        size
        for size in mem_cells.get("validate", {})
        if all(
            mem_ratio(mem_cells[axis][size][UPSTREAM_DOOR], mem_cells[axis][size][door])
            is not None
            for axis, door in plan
        )
    ]
    if not sizes:
        return []
    lines = [
        "| File | `laterite.validate` | `laterite.read` (typed) "
        "| `laterite.compat` (strings) |",
        "|---:|---:|---:|---:|",
    ]
    for size in sizes:
        cells = []
        for axis, door in plan:
            ours = mem_cells[axis][size][door]
            ratio = mem_ratio(mem_cells[axis][size][UPSTREAM_DOOR], ours)
            cells.append(f"{fmt_mb(ours['peak_rss_bytes'])} · **{ratio:.2f}×**")
        lines.append(f"| {labels.get(size, size)} | " + " | ".join(cells) + " |")
    return lines


def parse_swap_used(text: str) -> int:
    """The `used = 512.50M` field of Darwin's `vm.swapusage` sysctl, in bytes."""
    m = re.search(r"used\s*=\s*([0-9.]+)([KMG])", text)
    if not m:
        raise ValueError(f"unrecognised vm.swapusage output: {text!r}")
    scale = {"K": 1024, "M": 1024**2, "G": 1024**3}[m.group(2)]
    return int(float(m.group(1)) * scale)


def swap_used_bytes() -> int | None:
    """Current swap in use, or None where no instrument exists. Read before
    and after each child: growth means the child's number includes the pager."""
    if sys.platform == "darwin":
        out = subprocess.run(
            ["sysctl", "-n", "vm.swapusage"],
            capture_output=True,
            text=True,
            check=False,
        )
        return parse_swap_used(out.stdout) if out.returncode == 0 else None
    meminfo = Path("/proc/meminfo")
    if meminfo.exists():
        fields = {
            line.split(":")[0]: line.split()[1]
            for line in meminfo.read_text().splitlines()
            if ":" in line
        }
        try:
            return (int(fields["SwapTotal"]) - int(fields["SwapFree"])) * 1024
        except KeyError:
            return None
    return None


def mem_total_bytes() -> int | None:
    if sys.platform == "darwin":
        out = subprocess.run(
            ["sysctl", "-n", "hw.memsize"], capture_output=True, text=True, check=False
        )
        return int(out.stdout.strip()) if out.returncode == 0 else None
    # Windows has no os.sysconf — absent context, not a crash (#878; the
    # first windows lane leg died right here building its results block).
    if not hasattr(os, "sysconf"):
        return None
    try:
        return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")
    except (ValueError, OSError):
        return None


def build_results(
    *,
    rungs: dict[str, Any],
    time_cells: dict[str, Any],
    mem_cells: dict[str, Any],
    baselines: dict[str, Any],
    versions: dict[str, str],
    notes: list[str],
    invocation: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Assemble the committed results document (schema in the module docstring)."""
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
        cwd=REPO,
    )
    return {
        "schema": RESULTS_SCHEMA,
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "commit": commit.stdout.strip() if commit.returncode == 0 else "unknown",
        "invocation": invocation or {},
        "machine": {
            "platform": platform.platform(),
            "arch": platform.machine(),
            "cpu_count": os.cpu_count(),
            "mem_total_bytes": mem_total_bytes(),
            # No load average on Windows — absent, not zero (#878).
            "load_avg_start": (
                list(os.getloadavg()) if hasattr(os, "getloadavg") else None
            ),
        },
        "versions": versions,
        "protocol": {
            "time": "mean of N warm in-process runs, first run discarded",
            "memory": (
                "peak RSS of a fresh subprocess running one end-to-end "
                "operation through the library's public API — ru_maxrss on "
                "POSIX, PeakWorkingSetSize on Windows, the same instrument "
                "both sides of every cell; dhat/tracemalloc numbers are a "
                "different claim and never share a table with these"
            ),
            "refusals": (
                "a cell whose run swapped, died, or sits past the memory cap "
                "is recorded as {refusal, detail}, never skipped"
            ),
        },
        "rungs": rungs,
        "time": time_cells,
        "memory": mem_cells,
        "import_baselines": baselines,
        "notes": notes,
    }


def _run_header(doc: dict[str, Any]) -> dict[str, Any]:
    """The per-run provenance a merged document keeps for each contributor."""
    return {
        "generated": doc.get("generated"),
        "commit": doc.get("commit"),
        "invocation": doc.get("invocation", {}),
    }


def merge_results(existing: dict[str, Any], fresh: dict[str, Any]) -> dict[str, Any]:
    """Fold a (possibly partial) fresh run into an existing results document.

    Cells the fresh run measured win; cells it did not touch survive from the
    existing document; the `runs` list keeps every contributing invocation's
    provenance. This is what lets a killed run keep its finished rungs and a
    re-run name only the rungs it needs to redo.
    """
    merged = dict(fresh)
    merged["rungs"] = {**existing.get("rungs", {}), **fresh.get("rungs", {})}
    for section in ("time", "memory"):
        out: dict[str, Any] = {k: dict(v) for k, v in existing.get(section, {}).items()}
        for axis, per_rung in fresh.get(section, {}).items():
            out.setdefault(axis, {}).update(per_rung)
        merged[section] = out
    merged["import_baselines"] = {
        **existing.get("import_baselines", {}),
        **fresh.get("import_baselines", {}),
    }
    merged["notes"] = list(
        dict.fromkeys(existing.get("notes", []) + fresh.get("notes", []))
    )
    merged["runs"] = [
        *existing.get("runs", [_run_header(existing)]),
        _run_header(fresh),
    ]
    return merged


# --- the worker: one operation, one fresh process --------------------------
#
# Each op imports only its own library so the child's footprint is that
# library's, not the union of both sides. The parent talks to the child via a
# JSON spec (argv) and a JSON result (a temp file — stdout is not used, so a
# library that prints cannot corrupt the channel).


def _op_baseline_upstream(path: str, out: str | None) -> None:
    import python_ags4  # noqa: F401


def _op_baseline_native(path: str, out: str | None) -> None:
    import laterite  # noqa: F401


def _op_validate_upstream(path: str, out: str | None) -> None:
    from python_ags4 import AGS4

    AGS4.check_file(path)


def _op_validate_native(path: str, out: str | None) -> None:
    import laterite

    laterite.validate(path)


def _op_read_strings_upstream(path: str, out: str | None) -> None:
    from python_ags4 import AGS4

    AGS4.AGS4_to_dataframe(path)


def _op_read_strings_compat(path: str, out: str | None) -> None:
    from laterite import compat

    compat.AGS4_to_dataframe(path)


def _op_read_typed_upstream(path: str, out: str | None) -> None:
    from python_ags4 import AGS4

    tables, _ = AGS4.AGS4_to_dataframe(path)
    for key in tables:
        AGS4.convert_to_numeric(tables[key])


def _op_read_typed_native(path: str, out: str | None) -> None:
    import laterite

    laterite.read(path)


def _op_write_upstream(path: str, out: str | None) -> None:
    from python_ags4 import AGS4

    tables, headings = AGS4.AGS4_to_dataframe(path)
    AGS4.dataframe_to_AGS4(tables, headings, out)


def _op_write_compat(path: str, out: str | None) -> None:
    from laterite import compat

    assert out is not None  # the write plan always supplies a destination
    tables, headings = compat.AGS4_to_dataframe(path)
    compat.dataframe_to_AGS4(tables, headings, out)


def _op_write_native(path: str, out: str | None) -> None:
    import laterite

    assert out is not None  # the write plan always supplies a destination
    handle = laterite.read(path)
    frames = {code: handle[code] for code in handle.groups}
    laterite.build_ags4(frames).save(out)


def _op_write_native_unchecked(path: str, out: str | None) -> None:
    import laterite

    assert out is not None  # the write plan always supplies a destination
    handle = laterite.read(path)
    frames = {code: handle[code] for code in handle.groups}
    laterite.build_ags4_unchecked(frames, out=out)


WORKER_OPS: dict[str, Callable[[str, str | None], None]] = {
    "baseline_upstream": _op_baseline_upstream,
    "baseline_native": _op_baseline_native,
    "validate_upstream": _op_validate_upstream,
    "validate_native": _op_validate_native,
    "read_strings_upstream": _op_read_strings_upstream,
    "read_strings_compat": _op_read_strings_compat,
    "read_typed_upstream": _op_read_typed_upstream,
    "read_typed_native": _op_read_typed_native,
    "write_upstream": _op_write_upstream,
    "write_compat": _op_write_compat,
    "write_native": _op_write_native,
    "write_native_unchecked": _op_write_native_unchecked,
}


def parse_vm_hwm_bytes(status_text: str) -> int | None:
    """`VmHWM:  123456 kB` from a /proc/<pid>/status dump, in bytes."""
    m = re.search(r"^VmHWM:\s*(\d+)\s*kB", status_text, re.MULTILINE)
    return int(m.group(1)) * 1024 if m else None


def reset_peak_accounting() -> bool:
    """Linux only: reset this process's VmHWM high-water (`echo 5 >
    /proc/self/clear_refs`), so the peak read at exit is the peak of THIS
    process's own work.

    Why it exists (#878, lane run 33610286305): Linux `ru_maxrss` is
    inherited across fork() and never resets on exec, so a child spawned by
    a fat parent — this tool, right after timing the biggest rung
    in-process — starts with the parent's high-water already stamped, and
    every memory cell reads the parent's RSS as one constant. darwin's
    spawn semantics reset accounting (the committed lane never saw this)
    and the M1 probe's parent is thin (nor did it); the Linux lane's parent
    is exactly the fat case. VmHWM is the resettable twin of ru_maxrss.
    Returns False where the mechanism doesn't exist (not Linux, or /proc
    withheld) — the reader below falls back to ru_maxrss there."""
    if not sys.platform.startswith("linux"):
        return False
    try:
        Path("/proc/self/clear_refs").write_text("5")
    except OSError:
        return False
    return True


def peak_rss_self_bytes() -> int:
    """This process's peak RSS in bytes — the one instrument, per platform.

    Linux prefers `VmHWM` (the resettable twin of `ru_maxrss` — see
    `reset_peak_accounting` for why resettable matters) with `ru_maxrss` as
    the fallback; darwin reads `ru_maxrss` (bytes there, KiB on Linux —
    `maxrss_to_bytes` owns that split). Windows has no `resource` module;
    the equivalent high-water is `PeakWorkingSetSize` via ctypes, with
    declared signatures — without them ctypes marshals the 64-bit
    pseudo-handle through `c_int` and every call fails ERROR_INVALID_HANDLE
    (perf_probe_m1.py learned this on run 33585274632; this is the same
    instrument, kept in step)."""
    if sys.platform.startswith("linux"):
        try:
            hwm = parse_vm_hwm_bytes(Path("/proc/self/status").read_text())
        except OSError:
            hwm = None
        if hwm is not None:
            return hwm
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
        return int(counters.PeakWorkingSetSize)
    import resource

    raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return maxrss_to_bytes(raw, sys.platform)


def worker_main(spec_json: str) -> int:
    spec = json.loads(spec_json)
    # Before the op, so the peak read at exit belongs to this child's own
    # work, not to the accounting the fork stamped on it (see
    # reset_peak_accounting — this line is the #878 fix).
    reset_peak_accounting()
    WORKER_OPS[spec["op"]](spec.get("path", ""), spec.get("out"))
    out = spec.get("out")
    result = {
        "ok": True,
        "maxrss_bytes": peak_rss_self_bytes(),
        "out_bytes": Path(out).stat().st_size if out and Path(out).exists() else None,
    }
    Path(spec["result_path"]).write_text(json.dumps(result))
    return 0


def measure_mem(op: str, path: Path | None, out: Path | None) -> dict[str, Any]:
    """One (op, rung) memory cell: fresh child, swap watched across the run."""
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False, dir=OUT_DIR) as tf:
        result_path = Path(tf.name)
    spec = {
        "op": op,
        "path": str(path) if path else "",
        "out": str(out) if out else None,
        "result_path": str(result_path),
    }
    swap_before = swap_used_bytes()
    proc = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--worker", json.dumps(spec)],
        capture_output=True,
        text=True,
        check=False,
    )
    swap_after = swap_used_bytes()
    try:
        if proc.returncode != 0:
            tail = (proc.stderr or "").strip().splitlines()[-3:]
            return refusal_cell("failed", " | ".join(tail) or f"exit {proc.returncode}")
        if (
            swap_before is not None
            and swap_after is not None
            and swap_after - swap_before > SWAP_REFUSAL_BYTES
        ):
            grew = (swap_after - swap_before) / 1e6
            return refusal_cell("swapped", f"swap grew {grew:.1f} MB during the run")
        payload = json.loads(result_path.read_text())
        denom = payload["out_bytes"] if payload.get("out_bytes") else None
        if denom is None and path is not None:
            denom = path.stat().st_size
        if denom:
            return mem_cell(payload["maxrss_bytes"], denom)
        return {"peak_rss_bytes": payload["maxrss_bytes"]}
    finally:
        result_path.unlink(missing_ok=True)
        if out is not None:
            Path(out).unlink(missing_ok=True)


def main() -> int:
    # Rebound from --fixtures-dir / --manifest below; declared up top because
    # the argparse defaults read the same names.
    global OUT_DIR, MANIFEST

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
    ap.add_argument(
        "--mem-rungs",
        default=",".join(DEFAULT_MEM_RUNGS),
        help="rungs for the peak-RSS harness (default: %(default)s; capped "
        "at the 265MB rung — a bigger rung is recorded as a refusal)",
    )
    ap.add_argument(
        "--skip-mem", action="store_true", help="time only, no subprocess harness"
    )
    ap.add_argument(
        "--skip-time", action="store_true", help="memory only, no warm timing"
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="results file to write (default: %(default)s)",
    )
    ap.add_argument(
        "--manifest",
        type=Path,
        default=MANIFEST,
        help="fixture pin manifest (default: %(default)s). A fresh runner "
        "cannot mint the README corpus (#873), so the cross-OS lane (#878) "
        "passes tools/perf-probe-fixtures.json — the current-forge pins whose "
        "per-OS drift checks double as the cross-OS byte-identity proof",
    )
    ap.add_argument(
        "--fixtures-dir",
        type=Path,
        default=OUT_DIR,
        help="fixture cache directory (default: %(default)s). Pass a fresh "
        "directory alongside --manifest: the two corpora share rung names "
        "and must never share a cache path",
    )
    ap.add_argument("--worker", help=argparse.SUPPRESS)
    args = ap.parse_args()

    if args.worker:
        return worker_main(args.worker)

    if sys.platform == "win32":
        # The tables carry '→' and '×', which a cp1252 console cannot encode
        # — and dying while PRINTING already-measured results is the worst
        # possible exit (run 33612539765 measured everything, then crashed
        # on this glyph). Reconfigure the streams rather than strip the
        # glyphs: the printed tables are the README paste source and must be
        # byte-identical across platforms.
        for stream in (sys.stdout, sys.stderr):
            if hasattr(stream, "reconfigure"):
                stream.reconfigure(encoding="utf-8", errors="replace")

    # Rebind the module's fixture config before anything touches a rung —
    # fixture(), check_manifest() and the mem harness all read these.
    OUT_DIR = args.fixtures_dir
    MANIFEST = args.manifest

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

    time_sizes = (
        []
        if args.skip_time
        else [s.strip() for s in args.rungs.split(",") if s.strip()]
    )
    mem_sizes = (
        []
        if args.skip_mem
        else [s.strip() for s in args.mem_rungs.split(",") if s.strip()]
    )
    # One fixture pass over the union, so a rung shared by both harnesses is
    # unpacked and hash-checked once.
    all_sizes = list(dict.fromkeys(time_sizes + mem_sizes))
    paths = {s: fixture(s) for s in all_sizes}
    rung_pins = check_manifest(paths, args.update_manifest)

    versions = {
        "python": platform.python_version(),
        "python-ags4": dist_version("python-ags4"),
        "laterite": dist_version("laterite"),
        "pandas": dist_version("pandas"),
        "polars": dist_version("polars"),
        # pyarrow decides WHICH pandas hop the compat cells measure (the
        # accelerated pyarrow hop when importable, the shipped pyarrow-free
        # DuckDB hop otherwise) — #831 found the two differ by whole
        # ×-of-output on the memory axis, so a results file that doesn't name
        # the hop misdescribes what it measured. "?" means absent.
        "pyarrow": dist_version("pyarrow"),
    }
    hop = (
        "pyarrow" if dist_version("pyarrow") != "?" else "duckdb (the shipped default)"
    )
    run_notes = [
        f"compat pandas cells measured the {hop} hop this run; the shipped "
        f"default [compat] install is pyarrow-free (DuckDB hop) — see the "
        f"memory queue's M5 row (#834)"
    ]
    if swap_used_bytes() is None:
        # A watch that silently is not watching is a blind spot with a green
        # tick on it — say so in the record (Windows has no swap instrument).
        run_notes.append(
            "no swap instrument on this platform: memory cells carry no "
            "swapped-refusal protection this run"
        )
    # Report both versions: a speedup is meaningless without knowing what it
    # was measured against, and neither package exposes __version__ reliably.
    print(
        f"\npython-ags4 {versions['python-ags4']} vs "
        f"laterite {versions['laterite']} — mean of {args.runs} warm runs\n"
    )

    rows: dict[str, list[tuple]] = {
        "validate": [],
        "read_strings": [],
        "read_typed": [],
        "write": [],
    }
    time_cells: dict[str, dict[str, dict[str, Any]]] = {k: {} for k in rows}
    labels: dict[str, str] = {}

    def upstream_typed(target: str) -> None:
        """python-ags4's route to typed columns: read, then convert per group."""
        tables, _ = UPSTREAM.AGS4_to_dataframe(target)
        for key in tables:
            UPSTREAM.convert_to_numeric(tables[key])

    write_out = OUT_DIR / "write-bench.ags"

    def timed_write_upstream(target: str) -> float:
        tables, headings = UPSTREAM.AGS4_to_dataframe(target)
        return best_of(
            partial(UPSTREAM.dataframe_to_AGS4, tables, headings, str(write_out)),
            args.runs,
        )

    def timed_write_compat(target: str) -> float:
        tables, headings = COMPAT.AGS4_to_dataframe(target)
        return best_of(
            partial(COMPAT.dataframe_to_AGS4, tables, headings, str(write_out)),
            args.runs,
        )

    def timed_write_native(target: str) -> float:
        handle = laterite.read(target)
        frames = {code: handle[code] for code in handle.groups}
        return best_of(
            lambda: laterite.build_ags4(frames).save(str(write_out)), args.runs
        )

    def timed_write_unchecked(target: str) -> float:
        handle = laterite.read(target)
        frames = {code: handle[code] for code in handle.groups}
        return best_of(
            lambda: laterite.build_ags4_unchecked(frames, out=str(write_out)),
            args.runs,
        )

    def record_time(axis: str, size: str, door: str, seconds: float) -> None:
        time_cells[axis].setdefault(size, {})[door] = {
            "seconds": round(seconds, 4),
            "runs": args.runs,
        }

    for size in all_sizes:
        labels[size] = f"{paths[size].stat().st_size / 1e6:.1f} MB"

    mem_cells: dict[str, dict[str, dict[str, Any]]] = {}
    baselines: dict[str, Any] = {}

    # A killed run must keep its finished rungs: checkpoint the results file
    # after every rung, merging into whatever document is already there. The
    # existing file is read ONCE — merging against a moving target would
    # append this run's provenance to the history at every flush.
    base_doc: dict[str, Any] | None = None
    if args.out.exists():
        prior = json.loads(args.out.read_text())
        if prior.get("schema") == RESULTS_SCHEMA:
            base_doc = prior
        else:
            print(f"note: {args.out} has schema {prior.get('schema')!r}; replacing")

    def flush() -> None:
        doc = build_results(
            rungs={s: rung_pins[s] for s in all_sizes if s in rung_pins},
            time_cells=time_cells,
            mem_cells=mem_cells,
            baselines=baselines,
            versions=versions,
            notes=run_notes,
            invocation={
                "rungs": time_sizes,
                "mem_rungs": mem_sizes,
                "runs": args.runs,
            },
        )
        if base_doc is not None:
            doc = merge_results(base_doc, doc)
        args.out.parent.mkdir(parents=True, exist_ok=True)
        tmp = args.out.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(doc, indent=1) + "\n")
        tmp.replace(args.out)

    # `partial` rather than a lambda: a closure over the loop variable would
    # bind late, so every rung would end up timing the last file.
    for size in time_sizes:
        path = paths[size]
        label = labels[size]
        p = str(path)
        print(f"[{label}] timing ...", flush=True)

        cells = [
            (
                "validate",
                UPSTREAM_DOOR,
                best_of(partial(UPSTREAM.check_file, p), args.runs),
            ),
            (
                "validate",
                NATIVE_DOOR,
                best_of(partial(laterite.validate, p), args.runs),
            ),
            (
                "read_strings",
                UPSTREAM_DOOR,
                best_of(partial(UPSTREAM.AGS4_to_dataframe, p), args.runs),
            ),
            (
                "read_strings",
                COMPAT_DOOR,
                best_of(partial(COMPAT.AGS4_to_dataframe, p), args.runs),
            ),
            (
                "read_typed",
                UPSTREAM_DOOR,
                best_of(partial(upstream_typed, p), args.runs),
            ),
            ("read_typed", NATIVE_DOOR, best_of(partial(laterite.read, p), args.runs)),
        ]
        for axis, door, seconds in cells:
            record_time(axis, size, door, seconds)
        rows["validate"].append((label, cells[0][2], cells[1][2]))
        rows["read_strings"].append((label, cells[2][2], cells[3][2]))
        rows["read_typed"].append((label, cells[4][2], cells[5][2]))

        # The write doors hold a full read's tables while timing, so run them
        # one door at a time and collect between doors — three inputs at once
        # would stack three whole-file materialisations in one process.
        w_up = timed_write_upstream(p)
        gc.collect()
        w_compat = timed_write_compat(p)
        gc.collect()
        w_native = timed_write_native(p)
        gc.collect()
        w_unchecked = timed_write_unchecked(p)
        gc.collect()
        write_out.unlink(missing_ok=True)
        for door, seconds in (
            (UPSTREAM_DOOR, w_up),
            (COMPAT_DOOR, w_compat),
            (NATIVE_DOOR, w_native),
            (UNCHECKED_DOOR, w_unchecked),
        ):
            record_time("write", size, door, seconds)
        rows["write"].append((label, w_up, w_compat, w_native, w_unchecked))
        flush()
        print(
            f"  [{label}] done — validate {fmt(cells[0][2])}/{fmt(cells[1][2])}, "
            f"read {fmt(cells[2][2])}/{fmt(cells[3][2])}, "
            f"typed {fmt(cells[4][2])}/{fmt(cells[5][2])}, "
            f"write {fmt(w_up)}/{fmt(w_compat)}/{fmt(w_native)}/{fmt(w_unchecked)} "
            f"(upstream/ours) — checkpointed",
            flush=True,
        )

    # --- the peak-RSS harness (#821) — fresh subprocess per cell -----------

    MEM_PLAN: list[tuple[str, list[tuple[str, str, bool]]]] = [
        (
            "validate",
            [
                (UPSTREAM_DOOR, "validate_upstream", False),
                (NATIVE_DOOR, "validate_native", False),
            ],
        ),
        (
            "read_strings",
            [
                (UPSTREAM_DOOR, "read_strings_upstream", False),
                (COMPAT_DOOR, "read_strings_compat", False),
            ],
        ),
        (
            "read_typed",
            [
                (UPSTREAM_DOOR, "read_typed_upstream", False),
                (NATIVE_DOOR, "read_typed_native", False),
            ],
        ),
        (
            "write",
            [
                (UPSTREAM_DOOR, "write_upstream", True),
                (COMPAT_DOOR, "write_compat", True),
                (NATIVE_DOOR, "write_native", True),
                (UNCHECKED_DOOR, "write_native_unchecked", True),
            ],
        ),
    ]
    if mem_sizes:
        print("\nmeasuring peak RSS (one fresh subprocess per cell) ...", flush=True)
        for door, op in (
            (UPSTREAM_DOOR, "baseline_upstream"),
            (NATIVE_DOOR, "baseline_native"),
        ):
            baselines[door] = measure_mem(op, None, None)
        for size in mem_sizes:
            path = paths[size]
            capped = not mem_rung_allowed(path.stat().st_size)
            for axis, doors in MEM_PLAN:
                for door, op, needs_out in doors:
                    if capped:
                        cell = refusal_cell(
                            "beyond-mem-cap",
                            "memory columns stop at the 265MB rung "
                            "(epic #820 decision 7); this rung is time-only",
                        )
                    else:
                        out = OUT_DIR / f"mem-write-{door}.ags" if needs_out else None
                        cell = measure_mem(op, path, out)
                    mem_cells.setdefault(axis, {}).setdefault(size, {})[door] = cell
                    shown = (
                        f"REFUSED ({cell['refusal']})"
                        if "refusal" in cell
                        else f"{fmt_mb(cell['peak_rss_bytes'])} · {cell.get('x_output', '?')}×"
                    )
                    print(
                        f"  [{labels[size]}] {axis:12s} {door:16s} {shown}", flush=True
                    )
            flush()

        # A broken peak instrument does not scatter, it repeats: when fork
        # accounting (or an emulated kernel) hands every child the same
        # inherited high-water, every cell reads one constant and the
        # numbers LOOK precise (#878 — lane run 33610286305 published
        # fifteen byte-identical cells before this guard existed). The
        # checkpointed results file keeps the raw cells for diagnosis; the
        # run itself must not conclude success over them.
        measured = {
            cell["peak_rss_bytes"]
            for by_size in mem_cells.values()
            for by_door in by_size.values()
            for cell in by_door.values()
            if "peak_rss_bytes" in cell
        }
        if (
            len(measured) == 1
            and sum(
                1
                for by_size in mem_cells.values()
                for by_door in by_size.values()
                for cell in by_door.values()
                if "peak_rss_bytes" in cell
            )
            >= 4
        ):
            die(
                "every memory cell measured the same peak "
                f"({next(iter(measured))} bytes) — that is an instrument "
                "reading one inherited constant, not a set of measurements. "
                "Per-process peak accounting is not working in this "
                "environment; the checkpointed results file carries the raw "
                "cells for diagnosis."
            )

    # --- output -------------------------------------------------------------

    def table(title: str, left: str, right: str, key: str) -> None:
        print(f"\n**{title}**\n")
        print(f"| File | `{left}` | `{right}` | speedup |")
        print("|---:|---:|---:|:---:|")
        for label, a, b in rows[key]:
            print(f"| {label} | {fmt(a)} | {fmt(b)} | **{a / b:.1f}×** |")

    if time_sizes:
        print("\n" + "=" * 62)
        print("README-format tables — paste into the Performance section")
        print("=" * 62)
        table("Validation", "python-ags4 check_file", "laterite.validate", "validate")
        # `laterite.compat`, not `compat`: the claims gate keys the axis off
        # this header token, and the README paste must carry it verbatim.
        table(
            "Read, strings",
            "python-ags4 AGS4_to_dataframe",
            "laterite.compat",
            "read_strings",
        )
        table(
            "Read, typed",
            "python-ags4 + convert_to_numeric",
            "laterite.read",
            "read_typed",
        )
        print("\n**Write**\n")
        print(
            "| File | `python-ags4 dataframe_to_AGS4` | `compat` | speedup "
            "| `build_ags4` | speedup | `build_ags4_unchecked` | speedup |"
        )
        print("|---:|---:|---:|:---:|---:|:---:|---:|:---:|")
        for label, a, b, c, d in rows["write"]:
            print(
                f"| {label} | {fmt(a)} | {fmt(b)} | **{a / b:.1f}×** "
                f"| {fmt(c)} | **{a / c:.1f}×** "
                f"| {fmt(d)} | **{a / d:.1f}×** |"
            )

    if mem_cells:
        from laterite import _frames

        compat_hop = (
            "pyarrow accelerator"
            if _frames._pyarrow_available()
            else "shipped pyarrow-free DuckDB"
        )
        print("\n" + "=" * 62)
        print("README-format memory tables — the #826 promotion paste source")
        print("=" * 62)
        for line in memory_readme_tables(mem_cells, labels, compat_hop):
            print(line)

    if time_sizes:
        print("\n" + "=" * 62)
        print("root-README condensed tables — paste into its Performance section")
        print("=" * 62)
        print()
        for line in condensed_time_table(rows):
            print(line)
        if mem_cells:
            mem_lines = condensed_memory_table(mem_cells, labels)
            if mem_lines:
                print("\nPeak memory (same run, one fresh process per cell):\n")
                for line in mem_lines:
                    print(line)

    flush()
    print(f"\nresults written: {args.out}")

    if not args.keep_plain:
        repack(paths)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
