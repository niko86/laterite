#!/usr/bin/env python3
"""Sweep every state the landing demo can reach through BOTH engines (#659).

The demo tells a reader that laterite and python-ags4 answer differently in
places. That claim was being made from a handful of hand-checked states, and a
handful is not evidence about a few hundred — which is the correction that
produced this ticket. This walks the reachable states, dual-validates each, and
checks in the answer, so the demo can explain a difference without ever running
python-ags4 in a browser and so a validator change that moves a demo state
cannot land silently.

## The two halves, which are different in kind

`web/scripts/sweep-demo-states.mjs` enumerates the states and emits each one
**through the demo's own emitter**, loaded through Vite rather than
reimplemented — so every file here is provably reachable rather than merely
plausible. This script dual-validates them with `laterite-ags4-forge check`,
triages each difference, and renders the map.

## What "every state" honestly means

NOT the product of the levers. Group-present x row-present alone is exponential
in the seeded delivery's group and row counts, and every one of those states
costs a python-ags4 subprocess. (The counts themselves belong in the map, which
recomputes them; a figure written here would have been wrong within the hour
the reachable set was narrowed — as one was.) The sweep is exhaustive to
**depth 1**: every state one lever from the seed. Deeper states are covered
only by a small named set of sequences, the ones the demo's own teach loops
walk a reader through.

That bound is printed on every run and carried in the map. It is not a
footnote: a map that quietly stopped at depth 1 would read as "the demo has
been swept" when it means something much narrower — and this repo has been bitten
by a gate whose scope nobody could see (CLAUDE.md, Conventions).

## Why a difference must be triaged, not just recorded

A map that lists differences without saying what they ARE is a list of things
to re-investigate every time someone reads it. Every difference shape is
matched against a known, documented one; an unknown shape FAILS the run rather
than being written down, because an unrecognised divergence is either a new
O-N to write or a defect in one of the two engines, and both need a person.

Usage:
    uv run --no-sync python tools/gen_demo_state_map.py           # regenerate
    uv run --no-sync python tools/gen_demo_state_map.py --check   # drift gate
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web"
MAP = ROOT / "web" / "landing" / "demo" / "state-map.json"
FORGE = "laterite-ags4-forge"

#: The difference shapes this sweep knows how to explain. A shape is the pair
#: (rules only laterite raised, rules only python-ags4 raised) — the key sets,
#: not the counts, because a count difference on a shared key is a separate
#: and louder thing (see `count_differences` below, which must stay empty).
#:
#: Keyed by the frozen pair so a shape cannot be matched loosely. Anything not
#: here stops the run.
KNOWN: dict[tuple[tuple[str, ...], tuple[str, ...]], dict[str, str]] = {
    ((), ()): {
        "triage": "no difference",
        "why": "the two engines agree on which rules the state breaks",
    },
    (("Warning (Related to Rule 10c)",), ()): {
        "triage": "O-52",
        "why": (
            "a child row whose parent KEY cells are all empty claims no "
            "parent, so laterite DECLINES the parentage check and says so at "
            "the warning tier. python-ags4 reports nothing, for the reason "
            "O-52 documents: the all-empty key matches the parent's UNIT "
            "pseudo-row through its merge"
        ),
    },
    (("Warning (Related to Rule 14)",), ("FYI",)): {
        "triage": "O-45",
        "why": (
            "one condition, two tiers: a TRAN_AGS that is present but not a "
            "recognised edition is a laterite warning shown by default, and "
            "the same finding in python-ags4's opt-in FYI tier. Both engines "
            "report it, so this shape is a tier difference and not a silence"
        ),
    },
    ((), ("FYI",)): {
        "triage": "O-53",
        "why": (
            "a BLANK TRAN_AGS: python-ags4 stacks its unrecognised-edition FYI "
            "on the cell, on top of the Rule 10b error the blank already "
            "earns. laterite reports the cell once and states the schema it "
            "fell back to on the report envelope instead"
        ),
    },
}


def run(
    cmd: list[str], cwd: Path, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, capture_output=True, text=True, cwd=cwd, env=env)


def python_ags4_version() -> str:
    """The map is a claim about a SPECIFIC python-ags4, so the version is part
    of the map rather than something a reader has to infer from the date."""
    proc = run(
        [
            "uv",
            "run",
            "--no-sync",
            "python",
            "-c",
            "import python_ags4; print(python_ags4.__version__)",
        ],
        ROOT,
    )
    return proc.stdout.strip() or "unknown"


def emit_states(out_dir: Path) -> dict:
    # Vite is what resolves the demo's `?raw` seed import, which is what lets
    # the enumerator load the REAL model instead of a copy. Say so plainly
    # rather than letting node fail with a bare module-not-found.
    if not (WEB / "node_modules").is_dir():
        raise SystemExit(
            "gen_demo_state_map: web/node_modules is missing. The enumerator "
            "loads the demo's own model through Vite, so it needs them:\n"
            "    (cd web && npm ci)"
        )
    proc = run(["node", "scripts/sweep-demo-states.mjs", str(out_dir)], cwd=WEB)
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        raise SystemExit("gen_demo_state_map: the state enumerator failed")
    print(proc.stderr.strip(), file=sys.stderr)
    return json.loads((out_dir / "manifest.json").read_text(encoding="utf-8"))


def dual_validate(out_dir: Path) -> dict:
    binary = shutil.which(FORGE) or str(
        ROOT / "rust-packages" / "target" / "release" / FORGE
    )
    if not Path(binary).exists():
        binary = str(ROOT / "rust-packages" / "target" / "debug" / FORGE)
    if not Path(binary).exists():
        raise SystemExit(
            f"gen_demo_state_map: no {FORGE} binary. It is what runs BOTH "
            f"engines, so there is nothing to compare without it:\n"
            f"    (cd rust-packages && cargo build --release -p {FORGE})"
        )
    # Say which one. Three programs answer to a forge-shaped name here — a
    # PATH install, a release build and a debug build — and a stale one
    # produces a map that looks exactly like a fresh one.
    print(f"gen_demo_state_map: dual-validating with {binary}", file=sys.stderr)
    # BOTH sides unfiltered, which is the whole basis of a difference meaning
    # anything. The python bridge defaults to error-tier rule keys only —
    # correct for the parity gate, whose contract is exactly that (O-45) — and
    # this sweep compares laterite's warnings and FYI too. Filtering one side
    # while leaving the other whole does not under-report, it INVENTS findings:
    # python-ags4 raises `FYI (Related to Rule 16)` for a drifted abbreviation
    # description with the same message laterite does, and the first version of
    # this map recorded 145 states as a laterite-only divergence on the strength
    # of the filter dropping python's copy.
    env = {**os.environ, "LAT_PY_AGS4_ALL_KEYS": "1"}
    proc = run([binary, "check", str(out_dir), "--json"], ROOT, env)
    # A non-zero exit means "divergences found", which is the normal case here.
    if not proc.stdout.strip():
        print(proc.stderr, file=sys.stderr)
        raise SystemExit("gen_demo_state_map: forge produced no report")
    return json.loads(proc.stdout)


def build_map(manifest: dict, report: dict, version: str) -> tuple[dict, list[str]]:
    by_id = {s["id"]: s for s in manifest["states"]}
    states, shapes, count_diffs, unknown = [], {}, [], []
    validated: set[str] = set()

    for f in report.get("files", []):
        state_id = Path(f["file"]).stem
        meta = by_id.get(state_id)
        if meta is None:
            unknown.append(
                f"forge validated {state_id!r}, which the manifest does not list"
            )
            continue
        validated.add(state_id)
        rust = f.get("rust_rules", [])
        py = f.get("python_rules", [])
        shape = (
            tuple(sorted(set(rust) - set(py))),
            tuple(sorted(set(py) - set(rust))),
        )
        known = KNOWN.get(shape)
        if known is None:
            unknown.append(
                f"{state_id}: rust-only {list(shape[0])}, python-only {list(shape[1])}"
            )
        shapes.setdefault(shape, []).append(state_id)

        rc, pc = f.get("rust_rule_counts", {}), f.get("python_rule_counts", {})
        # The same rule key with a DIFFERENT count on each side is a louder
        # signal than a tier difference: the engines agree on which rule the
        # state breaks but not on how often, which no O-N currently explains.
        count_diffs.extend(
            {"state": state_id, "rule": key, "rust": rc[key], "python": pc[key]}
            for key in sorted(set(rc) & set(pc))
            if rc[key] != pc[key]
        )

        states.append(
            {
                "id": state_id,
                "lever": meta["lever"],
                "reached_by": {
                    k: v for k, v in meta.items() if k not in ("id", "lever")
                },
                "dictionary": f.get("dict_used"),
                "rust_rules": rust,
                "python_rules": py,
                "rust_rule_counts": rc,
                "python_rule_counts": pc,
                "difference": {
                    "rust_only": list(shape[0]),
                    "python_only": list(shape[1]),
                    "triage": (known or {}).get("triage", "UNTRIAGED"),
                },
            }
        )

    # The other direction, which is the one that goes quiet: a state the
    # enumerator emitted and forge never validated would simply be absent from
    # the map, and a shorter map reads exactly like a smaller state space.
    missing = sorted(set(by_id) - validated)
    if missing:
        unknown.extend(
            f"{state_id}: emitted by the enumerator but never validated — the "
            f"map would silently be missing it"
            for state_id in missing
        )

    states.sort(key=lambda s: s["id"])
    count_diffs.sort(key=lambda c: (c["state"], c["rule"]))
    doc = {
        "schema": 1,
        "generator": "tools/gen_demo_state_map.py",
        "python_ags4_version": version,
        "engines": {
            "laterite": "the in-process Rust validator, warnings and FYI ON",
            "note": (
                "forge validates with both tiers enabled so the Rust side is "
                "tier-comparable to python-ags4, which reports several of them "
                "at error tier. That is why laterite-only tier findings appear "
                "here at all, and why they are triaged rather than counted as "
                "disagreements about the file."
            ),
        },
        "scope": manifest["depth"],
        "value_classes": manifest["classes"],
        "counts": {
            "states": len(states),
            "by_lever": manifest["counts"]["by_lever"],
            "difference_shapes": len(shapes),
            "count_differences": len(count_diffs),
        },
        "not_covered": {
            "levers_that_were_no_ops": manifest["skipped"],
            "beyond_depth_1": manifest["depth"]["beyond"],
            # The one that matters most, because it is a lever a reader really
            # pulls rather than a combinatorial corner: applying the engine's
            # own fixes. Named, with why, rather than approximated — see the
            # enumerator.
            "levers_not_enumerated": manifest["depth"].get("levers_not_enumerated", []),
            "cell_values": (
                "setCell takes free text, so values are unbounded and are "
                "covered by equivalence class, not enumerated; a value outside "
                "every class below is outside this map"
            ),
        },
        "difference_shapes": [
            {
                "rust_only": list(ro),
                "python_only": list(po),
                "states": len(ids),
                "triage": KNOWN.get((ro, po), {}).get("triage", "UNTRIAGED"),
                "why": KNOWN.get((ro, po), {}).get("why", ""),
                "example": sorted(ids)[0],
            }
            for (ro, po), ids in sorted(shapes.items(), key=lambda kv: -len(kv[1]))
        ],
        "count_differences": count_diffs,
        "states": states,
    }
    return doc, unknown


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="fail on drift, write nothing")
    args = ap.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        out_dir = Path(tmp) / "states"
        manifest = emit_states(out_dir)
        report = dual_validate(out_dir)
        doc, unknown = build_map(manifest, report, python_ags4_version())

    # The scope statement, printed pass or fail. A sweep that says only "199
    # states, no surprises" reads as "the demo has been swept".
    print(
        f"gen_demo_state_map: {doc['counts']['states']} state(s), exhaustive to "
        f"depth {manifest['depth']['exhaustive_to']} "
        f"({', '.join(f'{k}={v}' for k, v in doc['counts']['by_lever'].items())})"
    )
    not_enumerated = manifest["depth"].get("levers_not_enumerated", [])
    print(
        f"gen_demo_state_map: NOT covered — the full product of the levers "
        f"(exponential in the group and row counts); multi-lever states beyond "
        f"{len(manifest['depth']['sequences'])} named sequence(s); cell values "
        f"outside the {len(doc['value_classes'])} equivalence classes; and "
        f"{len(not_enumerated)} lever(s) not enumerated at all "
        f"({', '.join(n['lever'] for n in not_enumerated)}). "
        f"{len(manifest['skipped'])} lever(s) were no-ops and are listed in the map."
    )
    print(
        f"gen_demo_state_map: {doc['counts']['difference_shapes']} distinct "
        f"difference shape(s) against python-ags4 "
        f"{doc['python_ags4_version']}; {doc['counts']['count_differences']} "
        f"per-rule count difference(s)"
    )

    if unknown:
        print(
            "\ngen_demo_state_map: UNTRIAGED difference(s) — each is either a new "
            "O-N to write or a defect in one of the two engines, and neither is "
            "something this generator may decide:\n",
            file=sys.stderr,
        )
        for line in unknown[:20]:
            print(f"  {line}", file=sys.stderr)
        if len(unknown) > 20:
            print(f"  … and {len(unknown) - 20} more", file=sys.stderr)
        return 1

    rendered = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    if args.check:
        current = MAP.read_text(encoding="utf-8") if MAP.exists() else ""
        if current != rendered:
            print(
                f"\ngen_demo_state_map: {MAP.relative_to(ROOT)} has DRIFTED. "
                "Re-run without --check and read the diff — it says which of "
                "the three this is: a state whose answer moved (the states "
                "block changes), the OTHER engine moving under us "
                "(`python_ags4_version` changes), or the reachable set itself "
                "changing (`counts` and `scope` change).",
                file=sys.stderr,
            )
            return 1
        print(f"gen_demo_state_map: {MAP.relative_to(ROOT)} is up to date")
        return 0

    MAP.write_text(rendered, encoding="utf-8")
    print(f"gen_demo_state_map: wrote {MAP.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
