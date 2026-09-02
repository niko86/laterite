"""The Python-lane bench's measurement plumbing, tested without measuring.

`tools/bench-vs-python-ags4.py` grew a peak-RSS harness and a committed
results file for #821. The benchmark itself cannot run in CI (it needs the
release wheel, python-ags4 and ~1 GB of fixtures), but the plumbing that
turns raw readings into the committed record CAN go wrong silently — and a
wrong results file outlives the run that wrote it. So the pure seams are
pinned here:

- `ru_maxrss` units differ per platform (bytes on Darwin, KiB on Linux); a
  unit slip inflates or deflates every memory cell by 1024×.
- the 265 MB memory cap (epic #820 decision 7) must REFUSE, not skip — a
  refusal is a recorded cell, a skip is a silent blind spot.
- a run that pushed the machine into swap measured the pager, not the
  library; the swap-delta refusal is what keeps that number out of the file.
- the results document must carry the schema id, the rung pins and every
  cell it was handed — it is the machine-readable half of the campaign
  ledger.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _load():
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


bench = _load()


# --- ru_maxrss units -------------------------------------------------------


def test_maxrss_is_bytes_on_darwin() -> None:
    assert bench.maxrss_to_bytes(1_048_576, "darwin") == 1_048_576


def test_maxrss_is_kib_on_linux() -> None:
    assert bench.maxrss_to_bytes(1_024, "linux") == 1_048_576


# --- the memory cap (epic #820 decision 7) ---------------------------------


def test_cap_admits_the_265mb_rung() -> None:
    # The pinned 265MB rung's byte size, from the committed manifest — the
    # cap exists to admit exactly this rung and refuse the next one.
    manifest = json.loads((REPO / "tools" / "readme-bench-fixtures.json").read_text())
    assert bench.mem_rung_allowed(manifest["265MB"]["bytes"])


def test_cap_refuses_the_524mb_rung() -> None:
    manifest = json.loads((REPO / "tools" / "readme-bench-fixtures.json").read_text())
    assert not bench.mem_rung_allowed(manifest["524MB"]["bytes"])


# --- cells: measurements and refusals --------------------------------------


def test_mem_cell_carries_peak_and_x_output() -> None:
    cell = bench.mem_cell(peak_bytes=250_000_000, denom_bytes=25_000_000)
    assert cell["peak_rss_bytes"] == 250_000_000
    assert cell["x_output"] == 10.0


def test_refusal_cell_is_a_recorded_verdict_not_a_skip() -> None:
    cell = bench.refusal_cell("swapped", "swap grew 210.0 MB during the run")
    assert cell["refusal"] == "swapped"
    assert "210.0 MB" in cell["detail"]
    # A refusal must be distinguishable from a measurement by shape.
    assert "peak_rss_bytes" not in cell


# --- swap parsing (the refusal's instrument) -------------------------------


def test_parse_darwin_swapusage() -> None:
    text = "total = 2048.00M  used = 512.50M  free = 1535.50M  (encrypted)"
    assert bench.parse_swap_used(text) == int(512.5 * 1024 * 1024)


def test_parse_darwin_swapusage_zero() -> None:
    text = "total = 0.00M  used = 0.00M  free = 0.00M  (encrypted)"
    assert bench.parse_swap_used(text) == 0


# --- the results document --------------------------------------------------


def test_results_document_shape() -> None:
    doc = bench.build_results(
        rungs={"5MB": {"bytes": 4_866_777, "sha256": "ab" * 32}},
        time_cells={"validate": {"5MB": {"python_ags4": {"seconds": 1.5, "runs": 5}}}},
        mem_cells={
            "validate": {
                "5MB": {"laterite": bench.mem_cell(100_000_000, 4_866_777)},
                "265MB": {"python_ags4": bench.refusal_cell("swapped", "swap grew")},
            }
        },
        baselines={"python_ags4": {"peak_rss_bytes": 50_000_000}},
        versions={"laterite": "0.12.0"},
        notes=[],
    )
    assert doc["schema"] == bench.RESULTS_SCHEMA
    assert doc["rungs"]["5MB"]["sha256"] == "ab" * 32
    assert doc["time"]["validate"]["5MB"]["python_ags4"]["seconds"] == 1.5
    assert doc["memory"]["validate"]["265MB"]["python_ags4"]["refusal"] == "swapped"
    assert doc["import_baselines"]["python_ags4"]["peak_rss_bytes"] == 50_000_000
    # The instrument split (peak-RSS vs dhat/tracemalloc never share a table)
    # is a protocol statement the file must self-describe.
    assert "peak RSS" in doc["protocol"]["memory"]
    assert "generated" in doc and "machine" in doc and "commit" in doc


# --- merging partial runs --------------------------------------------------
#
# The harness checkpoints after every rung and merges into an existing results
# file, so a killed run keeps its finished cells and a re-run can cover just
# the rungs it names. Fresh cells win; disjoint cells union; the runs history
# records every contributing invocation.


def _doc(rungs, time_cells, mem_cells, **kw):
    return bench.build_results(
        rungs=rungs,
        time_cells=time_cells,
        mem_cells=mem_cells,
        baselines=kw.get("baselines", {}),
        versions=kw.get("versions", {}),
        notes=kw.get("notes", []),
        invocation=kw.get("invocation", {}),
    )


def test_merge_unions_disjoint_rungs() -> None:
    a = _doc(
        {"5MB": {"bytes": 1, "sha256": "a"}},
        {"validate": {"5MB": {"laterite": {"seconds": 1.0, "runs": 5}}}},
        {},
    )
    b = _doc(
        {"25MB": {"bytes": 2, "sha256": "b"}},
        {"validate": {"25MB": {"laterite": {"seconds": 2.0, "runs": 5}}}},
        {},
    )
    m = bench.merge_results(a, b)
    assert set(m["rungs"]) == {"5MB", "25MB"}
    assert set(m["time"]["validate"]) == {"5MB", "25MB"}


def test_merge_fresh_cell_wins() -> None:
    a = _doc({}, {"validate": {"5MB": {"laterite": {"seconds": 1.0, "runs": 5}}}}, {})
    b = _doc({}, {"validate": {"5MB": {"laterite": {"seconds": 9.0, "runs": 5}}}}, {})
    m = bench.merge_results(a, b)
    assert m["time"]["validate"]["5MB"]["laterite"]["seconds"] == 9.0


def test_merge_keeps_runs_history() -> None:
    a = _doc({}, {}, {}, invocation={"rungs": ["5MB"]})
    b = _doc({}, {}, {}, invocation={"rungs": ["25MB"]})
    m = bench.merge_results(a, b)
    assert [r["invocation"]["rungs"] for r in m["runs"]] == [["5MB"], ["25MB"]]
    # And merging a third run appends rather than nests.
    c = _doc({}, {}, {}, invocation={"rungs": ["100MB"]})
    m2 = bench.merge_results(m, c)
    assert len(m2["runs"]) == 3


# --- README table rendering (the #826 promotion's paste source) -------------


def _mc(peak: int) -> dict:
    return {"peak_rss_bytes": peak, "x_output": 1.0}


_REFUSED = {"refusal": "beyond-mem-cap", "detail": "time-only rung"}


def test_mem_ratio_is_baseline_over_ours() -> None:
    assert bench.mem_ratio(_mc(200), _mc(100)) == 2.0


def test_mem_ratio_refusal_is_none_not_a_number() -> None:
    assert bench.mem_ratio(_REFUSED, _mc(100)) is None
    assert bench.mem_ratio(_mc(200), _REFUSED) is None


_MEM_CELLS = {
    "validate": {
        "5MB": {"python_ags4": _mc(200_000_000), "laterite": _mc(100_000_000)},
        "524MB": {"python_ags4": _REFUSED, "laterite": _REFUSED},
    },
    "read_typed": {
        "5MB": {"python_ags4": _mc(150_000_000), "laterite": _mc(100_000_000)},
        "524MB": {"python_ags4": _REFUSED, "laterite": _REFUSED},
    },
    "read_strings": {
        "5MB": {
            "python_ags4": _mc(100_000_000),
            "laterite_compat": _mc(125_000_000),
        },
        "524MB": {"python_ags4": _REFUSED, "laterite_compat": _REFUSED},
    },
}
_LABELS = {"5MB": "4.9 MB", "524MB": "549.7 MB"}


def test_memory_tables_band_value_and_mem_marker() -> None:
    lines = bench.memory_readme_tables(_MEM_CELLS, _LABELS, "pyarrow accelerator")
    text = "\n".join(lines)
    # Every header row carries the marker check_speed_claims splits tables on.
    headers = [ln for ln in lines if ln.startswith("| File |")]
    assert headers and all("peak RSS" in h for h in headers)
    # The bolded ratio is baseline/ours to 2dp — including below 1 for the
    # axis where laterite holds more (no flattering rounding, no omission).
    assert "**2.00×**" in text and "**1.50×**" in text and "**0.80×**" in text
    # A refused rung renders as no row, never as a number.
    assert "549.7 MB" not in text
    # The hop the compat cells measured is named in the caption.
    assert "pyarrow accelerator" in text


def test_memory_tables_empty_cells_render_nothing() -> None:
    assert bench.memory_readme_tables({}, {}, "x") == []


def test_condensed_time_table_pairs_rungs_across_axes() -> None:
    rows = {
        "validate": [("4.9 MB", 1.5, 0.05)],
        "read_typed": [("4.9 MB", 0.187, 0.026)],
        "read_strings": [("4.9 MB", 0.144, 0.049)],
    }
    lines = bench.condensed_time_table(rows)
    assert lines[0].startswith("| File (123 groups) |")
    assert "50 ms · **30.0×**" in lines[2]
    assert "26 ms · **7.2×**" in lines[2]
    assert "49 ms · **2.9×**" in lines[2]


def test_condensed_memory_table_drops_partial_rungs() -> None:
    lines = bench.condensed_memory_table(_MEM_CELLS, _LABELS)
    text = "\n".join(lines)
    assert "100 MB · **2.00×**" in text and "125 MB · **0.80×**" in text
    assert "549.7 MB" not in text
