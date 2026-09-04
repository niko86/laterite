"""The CLI perf lane's pure seams, tested without measuring.

`tools/perf-cli.py` (#825) is the CLI surface's matrix harness: it drives a
NAMED `lat` binary — three programs answer to `lat`, so resolution is a
seam, never a `PATH` lookup — and writes the same schema-2 per-surface file
as the rust/node/wasm lanes for `tools/perf-matrix.py` to merge. The real
job (release binary, hundreds-of-MB fixtures) cannot run in CI, so what is
pinned here is the contract:

- binary resolution honours `LAT_BIN`, falls back to this checkout's release
  build, and DIES rather than consult `PATH` — a lane that resolves from
  `PATH` measures whichever of the three programs the environment put there.
- the shared memory contract (cap, refusal vocabulary, cell shapes, swap
  parser, `ru_maxrss` units) is IMPORTED from the python lane's harness
  rather than copied — #865's drift class — and pinned here, which also
  gives the python copy the tests it never had.
- a `failed` refusal reads the child's signal and the stderr tail's Error
  line, and a genuinely-zero `out_bytes` falls back to the input denominator
  (both #824-review lessons, applied rather than re-learned).
- every result row names its door: the CLI's read and write doors are not
  the other surfaces' ops, so the artifact must say what was measured.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

# The suite split is file-granular (the conftest collection hook ignores
# whole modules): one test here spawns a child interpreter to prove the
# timed pass raises rather than dies, and `sys.executable` puts the whole
# file behind the built python job. The seam tests ride with it — the
# contract they pin can only drift when code changes, which runs that job.
pytestmark = pytest.mark.needs_env

REPO = Path(__file__).resolve().parents[1]


def _load(stem: str):
    """Import a hyphen-named tools/ script as a module."""
    name = stem.replace("-", "_")
    spec = importlib.util.spec_from_file_location(name, REPO / "tools" / f"{stem}.py")
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


cli = _load("perf-cli")


# --- the artifact ----------------------------------------------------------


def test_output_shape_and_lat_bin_recorded() -> None:
    # Schema 2, the merger's dialect; `skipped` serialises even when empty (a
    # positive statement that nothing was dropped); and the artifact names the
    # binary it measured — the resolved path outlives the run.
    doc = cli.build_output(10, [], [], {"path": "/x/lat", "version": "lat 9.9"})
    assert doc["schema"] == 2
    assert doc["surface"] == "cli"
    assert doc["tool"] == "tools/perf-cli.py"
    assert doc["skipped"] == []
    assert doc["lat_bin"] == {"path": "/x/lat", "version": "lat 9.9"}


def test_rows_carry_their_door() -> None:
    # The CLI's read/write doors are NOT the other surfaces' ops (`read`
    # renders strings, the write door is `merge`), so every row says exactly
    # what was invoked — the matrix cell carries it through.
    row = cli.measurement("read", "5MB", 5_000_000, 10.0, "read LOCA --csv --out …")
    assert row["op"] == "read"
    assert row["door"] == "read LOCA --csv --out …"
    assert row["median_ms"] == 10.0
    assert row["throughput_mb_s"] == 500.0


# --- timing statistics (the siblings' definitions, kept in step) -----------


def test_median_picks_the_upper_middle_sample() -> None:
    # The len/2 pick every lane uses (rust `samples[len/2]`, node/wasm
    # `s[s.length >> 1]`) — an even count takes the upper middle.
    assert cli.median([4.0, 1.0, 3.0, 2.0]) == 3.0
    assert cli.median([3.0, 1.0, 2.0]) == 2.0


def test_throughput_is_decimal_mb_per_s() -> None:
    # 5 MB in 10 ms → 500 MB/s (decimal MB, matching forge's parse_size);
    # degenerate timings never divide by zero.
    assert cli.throughput_mb_s(5_000_000, 10.0) == 500.0
    assert cli.throughput_mb_s(1_000, 0.0) == 0.0


# --- the shared memory contract (imported, not copied — see #865) ----------


def test_shared_mem_contract_is_the_python_lanes() -> None:
    # The cap admits the pinned 265MB rung and refuses 524MB (epic #820
    # decision 7), the refusal threshold is the trio's 64 MiB, and the cell
    # shapes share no keys — all read THROUGH this lane's import, so these
    # are also the python copy's first tests.
    assert cli.bench.MEM_CAP_BYTES == 300_000_000
    assert cli.bench.mem_rung_allowed(276_462_834)
    assert not cli.bench.mem_rung_allowed(551_560_078)
    assert cli.bench.SWAP_REFUSAL_BYTES == 64 * 1024 * 1024

    measured = cli.bench.mem_cell(1_500_000, 1_000_000)
    assert measured == {"peak_rss_bytes": 1_500_000, "x_output": 1.5}
    refused = cli.bench.refusal_cell("beyond-mem-cap", "too big")
    assert refused == {"refusal": "beyond-mem-cap", "detail": "too big"}
    assert not set(measured) & set(refused)


def test_darwin_swap_parse_and_maxrss_units() -> None:
    # The `used = 512.50M` field of vm.swapusage, and getrusage(2)'s lineage
    # split: `ru_maxrss` is bytes on Darwin, kibibytes on Linux — a unit slip
    # moves every cell 1024×.
    text = "total = 2048.00M  used = 512.50M  free = 1535.50M  (encrypted)"
    assert cli.bench.parse_swap_used(text) == 537_395_200
    assert cli.bench.maxrss_to_bytes(1_048_576, "darwin") == 1_048_576
    assert cli.bench.maxrss_to_bytes(1_024, "linux") == 1_048_576


def test_denominator_prefers_real_out_bytes_only() -> None:
    # `out_bytes or input` — a genuinely-zero out_bytes must fall back to the
    # input denominator, never reach the division (#824's `||`-not-`??`
    # lesson), and the floor is 1 so an empty rung cannot divide by zero.
    assert cli.denominator(50, 100) == 50
    assert cli.denominator(0, 100) == 100
    assert cli.denominator(None, 100) == 100
    assert cli.denominator(None, 0) == 1


# --- binary resolution (never PATH) ----------------------------------------


def test_resolve_lat_prefers_lat_bin_env(tmp_path: Path) -> None:
    exe = tmp_path / "lat"
    exe.write_bytes(b"")
    assert cli.resolve_lat({"LAT_BIN": str(exe)}, tmp_path / "repo") == exe


def test_resolve_lat_dies_on_a_missing_lat_bin(tmp_path: Path) -> None:
    # A pinned-but-absent LAT_BIN must die, not fall through: silently
    # measuring the fallback would publish numbers for the wrong program.
    with pytest.raises(SystemExit, match="LAT_BIN"):
        cli.resolve_lat({"LAT_BIN": str(tmp_path / "gone")}, tmp_path)


def test_resolve_lat_falls_back_to_this_checkouts_release_build(
    tmp_path: Path,
) -> None:
    built = tmp_path / "rust-packages" / "target" / "release" / "lat"
    built.parent.mkdir(parents=True)
    built.write_bytes(b"")
    assert cli.resolve_lat({}, tmp_path) == built


def test_resolve_lat_never_consults_path(tmp_path: Path) -> None:
    # No release build, no LAT_BIN → a loud death naming the build command.
    # Three programs answer to `lat`; a PATH lookup would time whichever one
    # the environment put there.
    with pytest.raises(SystemExit, match="cargo build"):
        cli.resolve_lat({}, tmp_path)


# --- child verdicts ---------------------------------------------------------


def test_failure_detail_names_the_signal_and_the_stderr_tail() -> None:
    # An OOM-SIGKILLed child has returncode -9 and often an empty stderr —
    # the refusal must say "signal 9", not "exit -9" or nothing (#824's
    # review lesson, applied here rather than mirrored later).
    assert "signal 9" in cli.failure_detail(-9, "")
    detail = cli.failure_detail(1, "a\nb\nc\nd\ne")
    assert detail == "c | d | e"
    assert cli.failure_detail(3, "") == "exit 3"


def test_a_failing_timed_door_raises_rather_than_dying() -> None:
    # The wasm lane's lesson (#824), inherited rather than re-learned: one
    # bad (op, rung) in the timed pass must become a recorded skip while the
    # artifact still gets written — so `timed_ms` raises `DoorFailed` for
    # the caller to record, never a process-killing SystemExit that would
    # lose every measurement already taken. (The child is a bare
    # interpreter exiting 1, not project code — but it is why this file
    # carries `needs_env`.)
    door = cli.Door(
        op="read",
        argv=[sys.executable, "-c", "import sys; sys.exit(1)"],
        out=None,
        label="failing door",
    )
    with pytest.raises(cli.DoorFailed, match="exited 1"):
        cli.timed_ms(door, 1)


def test_acceptable_exit_codes_per_door() -> None:
    # `lat validate` exits 1 when the file has findings — still a completed
    # validation. The read/write doors succeed only at 0.
    assert cli.acceptable_exit("validate", 0)
    assert cli.acceptable_exit("validate", 1)
    assert not cli.acceptable_exit("validate", 4)
    assert cli.acceptable_exit("read", 0)
    assert not cli.acceptable_exit("read", 1)
    assert cli.acceptable_exit("merge", 0)
    assert not cli.acceptable_exit("merge", 1)


# --- the read door's group choice -------------------------------------------


def test_bulk_group_picks_the_most_data_rows(tmp_path: Path) -> None:
    # The read door dumps ONE group, so the harness picks the rung's bulk
    # group by streaming the file once — deterministic, recorded in the row's
    # door string, never an assumption about the scaffold.
    ags = tmp_path / "x.ags"
    ags.write_text(
        '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"DATA","P1"\n'
        "\n"
        '"GROUP","LOCA"\n"HEADING","LOCA_ID"\n'
        '"DATA","L1"\n"DATA","L2"\n"DATA","L3"\n'
    )
    assert cli.bulk_group(ags) == ("LOCA", 3)


def test_bulk_group_dies_on_a_file_with_no_data(tmp_path: Path) -> None:
    ags = tmp_path / "x.ags"
    ags.write_text('"GROUP","PROJ"\n"HEADING","PROJ_ID"\n')
    with pytest.raises(SystemExit, match="no DATA rows"):
        cli.bulk_group(ags)
