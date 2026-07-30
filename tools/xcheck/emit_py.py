#!/usr/bin/env python3
"""The PYTHON legs of the cross-surface OUTPUT-VALUE gate (plan
`output/output-value-gate-plan.md` §2).

In-process `import laterite`. Python exposes MORE THAN ONE door, so this script
writes MORE THAN ONE observation file — one per leg:

  * ``python``         — the library (``laterite.read(path).text``, …).
  * ``python-compat``  — the python-ags4 drop-in shim
                         (``laterite.compat.dataframe_to_AGS4``).

For each leg it drives the single public door the case's ``op`` names — no
adapter logic — and records the three-variant envelope the comparator reads
(``{"ok": …}`` / ``{"err": <sentinel>}`` / ``{"absent": <reason>}``). The
comparator does zero normalisation, so any host-idiom transform (mapping a
native exception to the canonical error sentinel) belongs HERE, visible.

    python tools/xcheck/emit_py.py --out <dir> [--cases <dir>] [--repo-root <dir>]
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path

import laterite


def load_cases(cases_dir: Path) -> list[dict]:
    cases: list[dict] = []
    for path in sorted(cases_dir.glob("*.json")):
        cases.extend(json.loads(path.read_text())["cases"])
    return cases


# --- the `python` leg (the library) ----------------------------------


def reemit_canonical(fixture: Path) -> dict:
    """`laterite.read(path).text` — the canonical re-emit door (drift #1a).
    Post-#518 this routes through the shared `write_ags4_matrix` writer in the
    canonical shape, so it must equal the rust-leaf authority byte-for-byte."""
    return {"ok": laterite.read(fixture).text}


def excel_roundtrip(fixture: Path) -> dict:
    """`to_excel` → `from_excel` → the recovered AGS4 bytes. The Excel formatter
    (`NumericFormat::format`) is a SECOND numeric formatter that renders 3SF-of-
    zero `"0"` where the AGS4 canonical `ags4_str` renders `"0.00"` — a
    cross-PATH divergence N-way surface equality is structurally blind to (every
    surface calls each formatter identically). Read bytes, decode without
    universal-newline translation, so the CRLF is preserved faithfully."""
    with tempfile.TemporaryDirectory() as tmp:
        xlsx = Path(tmp) / "x.xlsx"
        back = Path(tmp) / "back.ags"
        laterite.to_excel(fixture, xlsx)
        laterite.from_excel(xlsx, back)
        return {"ok": back.read_bytes().decode("utf-8")}


def observe_python(case: dict, repo_root: Path) -> dict | None:
    op = case["op"]
    if op == "reemit_canonical":
        fixture = case["input"].get("fixture")
        if fixture is None:
            return None
        try:
            return reemit_canonical(repo_root / fixture)
        except Exception as e:
            return {"err": type(e).__name__}
    if op == "excel_roundtrip":
        fixture = case["input"].get("fixture")
        if fixture is None:
            return None
        try:
            return excel_roundtrip(repo_root / fixture)
        except Exception as e:
            return {"err": type(e).__name__}
    if op == "build_typed":
        build = case["input"].get("build")
        if build is None:
            return None
        try:
            return build_typed(build, case["input"].get("build_opts"))
        except Exception as e:
            return {"err": type(e).__name__}
    return None


def build_typed(groups: list[dict], opts: dict | None = None) -> dict:
    """`laterite.build_ags4` — the data→AGS4 door. Construct a polars frame per
    group from the typed inline rows (columns = headings, cells = the JSON
    values), and let build_ags4's shared orchestrator + dictionary fill the
    rest. Routes through the same emitter as the rust-leaf authority, so it must
    reproduce it byte-for-byte.

    `build_opts` carries the two knobs the build legs share — synthesis, and the
    transmission stamp. A case that sets them checks that the surfaces agree on
    the SYNTHESIS path, where a divergence changes which GROUPS a file has, not
    merely its bytes."""
    import polars as pl

    tables = {
        g["code"]: pl.DataFrame(g["rows"], schema=g["headings"], orient="row")
        for g in groups
    }
    opts = opts or {}
    tran = opts.get("tran")
    return {
        "ok": laterite.build_ags4(
            tables,
            synthesise_metadata=bool(opts.get("synthesise_metadata", False)),
            tran=laterite.TranStamp(**tran) if tran else None,
        ).text
    }


# --- the `python-compat` leg (the python-ags4 shim) ------------------


def emit_typed_verbatim(groups: list[dict]) -> dict:
    """`compat.dataframe_to_AGS4` — the verbatim emit door (drift #1b). Each
    inline group's `rows[0]` is the HEADING line (the frame's column names);
    the rest are the tagged UNIT/TYPE/DATA rows. Post-#518 a cell carrying an
    embedded CR/LF is REFUSED by the shared writer (the Rule-6 guard) instead
    of being torn into an illegal file — so this must refuse when the rust-leaf
    authority refuses, and write identical bytes when it doesn't."""
    import pandas as pd
    from laterite import compat as AGS4

    tables: dict[str, object] = {}
    headings: dict[str, list[str]] = {}
    for g in groups:
        cols = g["rows"][0]
        data = g["rows"][1:]
        tables[g["code"]] = pd.DataFrame(data, columns=cols)
        headings[g["code"]] = cols

    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "out.ags"
        try:
            AGS4.dataframe_to_AGS4(tables, headings, out)
        except Exception as e:
            # Edge map: the shared writer's Rule-6 refusal surfaces here as a
            # ValueError whose message names the embedded CR/LF. Map it to the
            # canonical sentinel the rust-leaf authority emits.
            if "embedded CR/LF" in str(e):
                return {"err": "EmbeddedNewline"}
            return {"err": type(e).__name__}
        # Read BYTES, not text: `read_text()` does universal-newline translation
        # and would silently fold the AGS4 CRLF to LF — the exact kind of quiet
        # byte edit this gate exists to catch. Decode without translation.
        return {"ok": out.read_bytes().decode("utf-8")}


def observe_compat(case: dict, repo_root: Path) -> dict | None:
    if case["op"] == "emit_typed_verbatim":
        groups = case["input"].get("groups")
        if groups is None:
            return None
        return emit_typed_verbatim(groups)
    return None


PY_LEGS = {
    "python": observe_python,
    "python-compat": observe_compat,
}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="output/xcheck")
    ap.add_argument("--cases", default="rust-packages/laterite-ags4-xcheck/cases")
    ap.add_argument("--repo-root", default=".")
    args = ap.parse_args()

    repo_root = Path(args.repo_root)
    cases = load_cases(Path(args.cases))
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    for leg, observe in PY_LEGS.items():
        observations: dict[str, dict] = {}
        for case in cases:
            if leg not in case["legs"]:
                continue
            obs = observe(case, repo_root)
            if obs is not None:
                observations[case["id"]] = obs
        path = out_dir / f"{leg}.json"
        path.write_text(
            json.dumps({"schema": 1, "leg": leg, "cases": observations}, indent=2)
        )
        print(f"{leg}: {len(observations)} cases -> {path}", flush=True)


if __name__ == "__main__":
    main()
