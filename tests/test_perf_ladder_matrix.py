"""The cross-surface matrix's plumbing, tested without measuring.

`tools/perf-ladder.py` writes the ladder manifest every surface harness
reads, and `tools/perf-matrix.py` merges the per-surface results those
harnesses write (#822). Neither can run its real job in CI (fixtures are
hundreds of MB, the harnesses need release builds), but the shapes they
produce are contracts — the rust bin deserialises the manifest, the later
Node/wasm/CLI lanes (#823-#825) will write to the merger's expectations —
so the pure seams are pinned here:

- the manifest carries the schema id, size-ordered rungs, and the pins it
  inherited — a consumer trusting an unpinned path would measure drift.
- the merger refuses two files claiming one surface: silently preferring
  one would publish whichever file happened to sort later.
- a non-matrix file (e.g. the python lane's own results record) is skipped
  WITH a reason, never merged and never silently dropped.
- a memory refusal cell survives the merge and renders as a refusal — a
  reader must not mistake a vetoed run for a small number.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

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


ladder = _load("perf-ladder")
matrix = _load("perf-matrix")


# --- the ladder manifest ---------------------------------------------------


def test_manifest_is_pinned_and_size_ordered(tmp_path: Path) -> None:
    paths = {
        "25MB": tmp_path / "readme-25MB.ags",
        "5MB": tmp_path / "readme-5MB.ags",
    }
    pins = {
        "5MB": {"bytes": 5_000_000, "sha256": "aa"},
        "25MB": {"bytes": 25_000_000, "sha256": "bb"},
    }
    doc = ladder.build_manifest(paths, pins, "wide", 0)
    assert doc["schema"] == ladder.LADDER_SCHEMA
    assert doc["forge"] == {"scaffold": "wide", "seed": 0}
    # Size-ordered regardless of input order, pins carried through, absolute paths.
    assert [r["label"] for r in doc["rungs"]] == ["5MB", "25MB"]
    assert doc["rungs"][0]["sha256"] == "aa"
    assert all(Path(r["path"]).is_absolute() for r in doc["rungs"])


def test_manifest_speaks_the_bins_dialect(tmp_path: Path) -> None:
    # The rust bin deserialises {rungs: [{label, path}]} and ignores the
    # rest; both keys must be present on every rung.
    doc = ladder.build_manifest(
        {"5MB": tmp_path / "a.ags"}, {"5MB": {"bytes": 1, "sha256": "aa"}}, "wide", 0
    )
    assert all("label" in r and "path" in r for r in doc["rungs"])


# --- the merger ------------------------------------------------------------


def surface_doc(surface: str, **cell) -> dict:
    row = {
        "op": "validate",
        "rung": "5MB",
        "bytes": 5_000_000,
        "median_ms": 10.0,
        "throughput_mb_s": 500.0,
        **cell,
    }
    return {
        "schema": 2,
        "surface": surface,
        "tool": f"{surface}-harness",
        "iters": 10,
        "results": [row],
    }


def test_merge_nests_op_rung_surface() -> None:
    doc = matrix.merge(
        {"rust.json": surface_doc("rust"), "node.json": surface_doc("node")}
    )
    assert doc["schema"] == matrix.MATRIX_SCHEMA
    assert set(doc["surfaces"]) == {"rust", "node"}
    cell = doc["cells"]["validate"]["5MB"]["rust"]
    assert cell["median_ms"] == 10.0
    # op/rung are the nesting keys, not duplicated inside the cell.
    assert "op" not in cell and "rung" not in cell


def test_merge_refuses_a_duplicated_surface() -> None:
    with pytest.raises(SystemExit, match="surface 'rust' appears in both"):
        matrix.merge({"a.json": surface_doc("rust"), "b.json": surface_doc("rust")})


def test_classify_names_the_foreign_schema() -> None:
    # The python lane's committed record must be skipped with its schema named.
    reason = matrix.classify(
        "python-lane.json", {"schema": "laterite-python-lane-bench/1", "time": {}}
    )
    assert reason is not None and "laterite-python-lane-bench/1" in reason
    assert matrix.classify("rust.json", surface_doc("rust")) is None


def test_refusal_cells_survive_and_render_as_refusals() -> None:
    refused = surface_doc("rust", mem={"refusal": "beyond-mem-cap", "detail": "x"})
    doc = matrix.merge({"rust.json": refused})
    mem = doc["cells"]["validate"]["5MB"]["rust"]["mem"]
    assert mem["refusal"] == "beyond-mem-cap"
    assert "refused: beyond-mem-cap" in matrix.fmt_mem(mem)
    # A measured cell renders its headline factor; an absent column renders as such.
    assert "1.5×" in matrix.fmt_mem({"peak_rss_bytes": 7_500_000, "x_output": 1.5})
    assert matrix.fmt_mem(None) == "—"


def test_render_table_covers_every_surface_and_op() -> None:
    doc = matrix.merge(
        {
            "rust.json": surface_doc(
                "rust", mem={"peak_rss_bytes": 1_000_000, "x_output": 0.2}
            ),
            "node.json": surface_doc("node"),
        }
    )
    text = matrix.render_table(doc)
    assert "== validate ==" in text
    assert "rust" in text and "node" in text
