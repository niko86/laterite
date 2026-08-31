#!/usr/bin/env python3
"""Aggregate the per-surface perf-matrix results into one document.

Each surface's harness — `laterite-ags4-perf` (rust) today, the Node/wasm/CLI
lanes to follow (#823-#825) — writes the matrix's uniform per-surface schema
(`{surface, results: [{op, rung, bytes, median_ms, throughput_mb_s, mem?}]}`)
into `output/perf-results/`. This script is the dumb merger those files were
shaped for: it folds them into one matrix document and prints the table. It
parses no per-tool formats and computes no statistics — a surface's numbers
are its harness's business.

What it does NOT merge: a results file without the uniform shape (for
example a copy of the python lane's `laterite-python-lane-bench/1` record,
whose schema is its own and whose ledger lives in the wiki). Skipped files
are NAMED on every run, pass or fail — a filter nobody can see is a blind
spot with a green tick on it.

Matrix schema `laterite-perf-matrix/1`:

    schema      the string above; bump on shape changes
    generated   UTC timestamp of the merge
    surfaces    {surface: {tool, iters, schema}} — each contributor's stamp
    cells       {op: {rung: {surface: {bytes, median_ms, throughput_mb_s, mem?}}}}

A `mem` cell is either `{peak_rss_bytes, x_output}` or a recorded refusal
`{refusal, detail}` — distinguishable by shape, and rendered as such.

Usage:
    uv run python tools/perf-matrix.py                # output/perf-results/
    uv run python tools/perf-matrix.py --results-dir <d> --out <p>
"""

from __future__ import annotations

import argparse
import datetime
import json
import sys
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
DEFAULT_DIR = REPO / "output" / "perf-results"
MATRIX_SCHEMA = "laterite-perf-matrix/1"

# Render order for the ops every surface shares; anything else follows,
# alphabetically — the merger must not refuse an op it has never heard of.
OP_ORDER = ["validate", "parse-to-typed", "write"]


def classify(name: str, doc: Any) -> str | None:
    """Why a JSON file is not a per-surface matrix file, or None if it is."""
    if not isinstance(doc, dict):
        return "not a JSON object"
    if "surface" not in doc or "results" not in doc:
        schema = doc.get("schema")
        if isinstance(schema, str):
            return f"schema {schema!r} — not a per-surface matrix file"
        return "no surface/results keys — not a per-surface matrix file"
    return None


def merge(docs: dict[str, dict[str, Any]]) -> dict[str, Any]:
    """Fold per-surface documents (keyed by filename) into the matrix.

    Two files claiming the same surface is an error, not a pick — silently
    preferring one would publish whichever file happened to sort later.
    """
    surfaces: dict[str, Any] = {}
    owners: dict[str, str] = {}
    cells: dict[str, dict[str, dict[str, Any]]] = {}
    for name in sorted(docs):
        doc = docs[name]
        surface = doc["surface"]
        if surface in owners:
            raise SystemExit(
                f"error: surface {surface!r} appears in both {owners[surface]} "
                f"and {name} — remove or rename one"
            )
        owners[surface] = name
        surfaces[surface] = {
            "tool": doc.get("tool"),
            "iters": doc.get("iters"),
            "schema": doc.get("schema"),
        }
        for row in doc["results"]:
            cell = {k: v for k, v in row.items() if k not in ("op", "rung")}
            cells.setdefault(row["op"], {}).setdefault(row["rung"], {})[surface] = cell
    return {
        "schema": MATRIX_SCHEMA,
        "generated": datetime.datetime.now(datetime.UTC).isoformat(timespec="seconds"),
        "surfaces": surfaces,
        "cells": cells,
    }


def fmt_mem(mem: dict[str, Any] | None) -> str:
    if mem is None:
        return "—"
    if "refusal" in mem:
        return f"refused: {mem['refusal']}"
    return f"{mem['peak_rss_bytes'] / 1e6:,.0f} MB ({mem['x_output']}×)"


def render_table(matrix: dict[str, Any]) -> str:
    """The matrix as text: one block per op, rungs by size, surfaces within."""
    lines: list[str] = []
    ops = [op for op in OP_ORDER if op in matrix["cells"]]
    ops += sorted(op for op in matrix["cells"] if op not in OP_ORDER)
    header = f"{'rung':>8}  {'surface':<10} {'median':>10}  {'MB/s':>8}  peak RSS"
    for op in ops:
        lines.append(f"\n== {op} ==")
        lines.append(header)
        per_rung = matrix["cells"][op]
        by_size = sorted(
            per_rung, key=lambda r: min(c.get("bytes", 0) for c in per_rung[r].values())
        )
        for rung in by_size:
            for surface in sorted(per_rung[rung]):
                cell = per_rung[rung][surface]
                lines.append(
                    f"{rung:>8}  {surface:<10} "
                    f"{cell['median_ms']:>8.1f}ms  "
                    f"{cell['throughput_mb_s']:>8.1f}  "
                    f"{fmt_mem(cell.get('mem'))}"
                )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--results-dir",
        type=Path,
        default=DEFAULT_DIR,
        help="directory of per-surface result files (default: %(default)s)",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="matrix file to write (default: <results-dir>/matrix.json)",
    )
    args = ap.parse_args()
    out = args.out or args.results_dir / "matrix.json"

    if not args.results_dir.is_dir():
        print(
            f"error: {args.results_dir} does not exist — run a surface harness "
            f"first (rust: `cargo run --release --manifest-path "
            f"rust-packages/Cargo.toml -p laterite-ags4-perf`)",
            file=sys.stderr,
        )
        return 1

    docs: dict[str, dict[str, Any]] = {}
    skipped: list[str] = []
    for path in sorted(args.results_dir.glob("*.json")):
        if path == out:
            continue  # never merge our own output back in
        try:
            doc = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            skipped.append(f"{path.name} (unreadable JSON: {e})")
            continue
        reason = classify(path.name, doc)
        if reason:
            skipped.append(f"{path.name} ({reason})")
        else:
            docs[path.name] = doc

    # The filter report, on every run: what was merged and what was not.
    print(f"merged {len(docs)} surface file(s), skipped {len(skipped)}")
    for line in skipped:
        print(f"  skipped: {line}")
    if not docs:
        print(
            f"error: no per-surface result files in {args.results_dir}", file=sys.stderr
        )
        return 1

    matrix = merge(docs)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(matrix, indent=2) + "\n")
    print(render_table(matrix))
    print(f"\nmatrix → {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
