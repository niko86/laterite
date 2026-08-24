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
from typing import Literal, NotRequired, TypedDict

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web"
MAP = ROOT / "web" / "landing" / "demo" / "state-map.json"
NOTES = ROOT / "web" / "landing" / "demo" / "divergence-notes.json"
COUNTS = ROOT / "web" / "landing" / "demo" / "python-counts.json"
FORGE = "laterite-ags4-forge"

#: The prefix a rule key wears when the finding is FYI-tier. It is the tier
#: marker in this key space — `Warning (Related to Rule 10c)` and
#: `FYI (Related to Rule 16)` are how forge names the two non-error tiers — and
#: it is the one thing separating what this map measures from what the demo
#: shows. See `visible_signature`.
FYI_PREFIX = "FYI"


class Reader(TypedDict):
    """The reader-facing half of a triage entry: which direction the difference
    goes, and what the demo says about it. Typed rather than left as a loose
    dict because `side` is what the page switches its whole rendering on."""

    side: Literal["ours", "theirs", "tier"]
    text: str


class Triage(TypedDict):
    """One difference shape's explanation. `why` is for whoever maintains this
    sweep; `reader` is for whoever is looking at the demo. They are separate
    fields because they are written for different people and should read
    differently — collapsing them produces prose that serves neither."""

    triage: str
    why: str
    reader: NotRequired[Reader]


#: The difference shapes this sweep knows how to explain. A shape is the pair
#: (rules only laterite raised, rules only python-ags4 raised) — the key sets,
#: not the counts, because a count difference on a shared key is a separate
#: and louder thing (see `count_differences` below, which must stay empty).
#:
#: Keyed by the frozen pair so a shape cannot be matched loosely. Anything not
#: here stops the run.
KNOWN: dict[tuple[tuple[str, ...], tuple[str, ...]], Triage] = {
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
        "reader": {
            "side": "ours",
            "text": (
                "python-ags4 stays silent here. Not because it decided the "
                "row was fine: its parent lookup happens to match an empty "
                "key against the parent group's units row, so it never asks "
                "the question."
            ),
        },
    },
    (("Warning (Related to Rule 14)",), ("FYI",)): {
        "triage": "O-45",
        "why": (
            "one condition, two tiers: a TRAN_AGS that is present but not a "
            "recognised edition is a laterite warning shown by default, and "
            "the same finding in python-ags4's opt-in FYI tier. Both engines "
            "report it, so this shape is a tier difference and not a silence"
        ),
        "reader": {
            "side": "tier",
            "text": (
                "python-ags4 reports this too, as an FYI you have to ask for. "
                "We show it by default: the edition could not be matched, so "
                "the file was checked against a dictionary its author never "
                "named."
            ),
        },
    },
    ((), ("FYI",)): {
        "triage": "O-53",
        "why": (
            "a BLANK TRAN_AGS: python-ags4 stacks its unrecognised-edition FYI "
            "on the cell, on top of the Rule 10b error the blank already "
            "earns. laterite reports the cell once and states the schema it "
            "fell back to on the report envelope instead"
        ),
        "reader": {
            "side": "theirs",
            "text": (
                "python-ags4 adds a second note on this cell, saying the "
                "blank is not a recognised AGS4 version. We report the empty "
                "required field once, and say which dictionary we fell back "
                "to on the result itself rather than on the cell."
            ),
        },
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
            # The `arbitrary-text` class covers the four surfaces where free
            # text can change an answer; these are the columns where it cannot,
            # named rather than left as an unexplained gap in the sweep.
            "inert_columns": manifest.get("inert_columns", []),
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


def build_notes(doc: dict) -> tuple[dict, list[str]]:
    """The reader-facing half of the map: one note per difference shape the
    demo can actually meet, small enough to ship to a browser.

    The map is the evidence and is ~150x the size of this; the landing page
    does not need 150 states to explain the one it is in. What it needs is the
    note keyed by something it can match against its own state WITHOUT
    reimplementing a rule:

    * `ours` / `tier` — matched on the RULE KEY. The finding is already on the
      page carrying that key, so the note attaches to it and needs nothing
      else. It generalises for free: any state that raises the key gets the
      explanation, including states this sweep never enumerated.
    * `theirs` — there is no finding to attach to, which is the whole point of
      the case. Matched instead on the CELL the state was reached by, carried
      as the literal value the enumerator wrote. Comparing one cell to one
      string is something a browser can do exactly; deciding "is this value
      non-ASCII for its declared type" is not, and a demo that re-derived it
      would be a second validator disagreeing with the first.

    A shape with no `reader` entry stops the run, for the same reason an
    untriaged shape does: silence here renders as "this state has nothing to
    explain", which is indistinguishable from the truth and is not it.
    """
    by_state = {st["id"]: st for st in doc["states"]}
    notes: list[dict] = []
    missing: list[str] = []

    for shape in doc["difference_shapes"]:
        if not shape["rust_only"] and not shape["python_only"]:
            continue
        key = (tuple(shape["rust_only"]), tuple(shape["python_only"]))
        reader = KNOWN.get(key, {}).get("reader")
        if not reader:
            missing.append(
                f"{shape['triage']}: shape {key} has no `reader` note, so the "
                f"demo would show its {shape['states']} state(s) with nothing "
                f"said about them"
            )
            continue

        note = {
            "observation": shape["triage"],
            "side": reader["side"],
            "text": reader["text"],
            "states": shape["states"],
        }
        if reader["side"] == "theirs":
            # Every cell edit that reaches this shape, so a reader who finds it
            # by another route is not met with silence.
            cells = []
            for state_id in sorted(
                st["id"]
                for st in doc["states"]
                if tuple(st["difference"]["python_only"]) == key[1]
                and tuple(st["difference"]["rust_only"]) == key[0]
            ):
                by = by_state[state_id]["reached_by"]
                if "value" in by:
                    cells.append(
                        {
                            "group": by["group"],
                            "row": by["row"],
                            "heading": by["heading"],
                            "value": by["value"],
                        }
                    )
            if not cells:
                missing.append(
                    f"{shape['triage']}: reached by no single cell edit, so "
                    f"the demo has nothing exact to match on — this shape "
                    f"needs a trigger the browser can evaluate"
                )
                continue
            note["when_cell_is"] = cells
        else:
            note["rules"] = list(shape["rust_only"])
        notes.append(note)

    # The third case the demo was asked to explain — the same rule key raised by
    # both engines at DIFFERENT counts — has no note here, and deliberately no
    # render path either. It is not a difference SHAPE (the shapes compare key
    # sets; this is `count_differences`, a separate list), it has no O-N because
    # none has ever occurred, and shipping a generic path for it would put code
    # on the page that has never rendered and cannot be tested against anything.
    #
    # So the guard goes here instead, which is the stronger half of "not dropped
    # silently": if one ever appears, this STOPS rather than writing a map the
    # demo would be quiet about. Nobody has to remember to look.
    missing.extend(
        f"{diff['state']}: the two engines raise {diff['rule']!r} a different "
        f"number of times, which no O-N explains and the demo has no way to "
        f"say — the louder of the two signals this sweep watches for"
        for diff in doc["count_differences"]
    )

    notes.sort(key=lambda n: n["observation"])
    return {
        "schema": 1,
        "generator": "tools/gen_demo_state_map.py",
        "source": str(MAP.relative_to(ROOT)),
        "python_ags4_version": doc["python_ags4_version"],
        "sides": {
            "ours": "laterite reports it, python-ags4 does not",
            "theirs": "python-ags4 reports it, laterite does not",
            "tier": "both report it, at different tiers",
        },
        "notes": notes,
    }, missing


def visible_signature(rust_rule_counts: dict[str, int]) -> tuple[str, list[str]]:
    """A state's laterite finding signature AS THE DEMO SEES IT, and what that
    cost.

    The two engines are compared here with every tier on, which is what makes a
    difference mean anything (see `dual_validate`). The demo's `validate` call
    takes the wasm defaults: warnings on, **FYI off**. So the signature the
    browser can compute for itself is over the visible tiers only, and building
    it from the same subset here is what makes the lookup a function of
    something the page actually holds rather than of something forge measured.

    The dropped keys come back rather than vanishing, because they are not a
    detail: the map's laterite total and the demo's displayed total are the
    same number only while this list is empty.
    """
    dropped = sorted(k for k in rust_rule_counts if k.startswith(FYI_PREFIX))
    visible = {k: v for k, v in rust_rule_counts.items() if k not in dropped}
    return "|".join(f"{k}={visible[k]}" for k in sorted(visible)), dropped


def _cell(state: dict) -> dict | None:
    """The single cell edit a state was reached by, or None if it was reached
    some other way. This is the only trigger a browser can evaluate exactly —
    one cell against one string — which is why a collision resolvable any other
    way is not resolved at all."""
    by = state["reached_by"]
    if state["lever"] != "setCell" or "value" not in by:
        return None
    return {
        "group": by["group"],
        "row": by["row"],
        "heading": by["heading"],
        "value": by["value"],
    }


def build_python_counts(doc: dict) -> tuple[dict, list[str]]:
    """python-ags4's finding TOTAL for each laterite signature the demo can
    reach, small enough to ship to a browser (#673).

    python-ags4 is a dev-only dependency and is not in the page, so the number
    beside laterite's is read, never computed. The key is laterite's own
    finding signature because that is what the demo already has from its own
    run — and it is a legitimate key only because the sweep measured it to be
    one: the swept states collapse to far fewer signatures than states, and
    python's answer is constant across all but one of them.

    The exception is handled rather than hidden. A signature with two python
    answers is not a function, and a page cannot show a number it has two of,
    so this resolves it from the states themselves: if every state on the
    minority answer was reached by a single cell edit that no state on the
    majority answer shares, those cells become overrides the browser can test
    exactly. Anything else FAILS, which is the point — the collision that
    exists today (only a blank `TRAN_AGS` earns python's extra FYI, O-53)
    resolves that way, and a second one landing on the page unnoticed is the
    failure this whole sweep exists to prevent.
    """
    problems: list[str] = []
    by_signature: dict[str, dict[int, list[dict]]] = {}
    fyi_states: list[str] = []

    for state in doc["states"]:
        signature, dropped = visible_signature(state["rust_rule_counts"])
        if dropped:
            fyi_states.append(f"{state['id']} ({', '.join(dropped)})")
        total = sum(state["python_rule_counts"].values())
        by_signature.setdefault(signature, {}).setdefault(total, []).append(state)

    # The trap this gate exists for, which had no gate at all: the map measures
    # laterite with FYI ON and the demo displays it OFF. They are the same
    # number only while nothing raises one — true today, and true by accident
    # rather than by design.
    if fyi_states:
        problems.append(
            f"{len(fyi_states)} state(s) raise a laterite FYI the demo does not "
            f"display ({', '.join(fyi_states[:5])}"
            f"{' …' if len(fyi_states) > 5 else ''}), so the page would put "
            "python-ags4's total beside a laterite total it is not showing. "
            "Either turn the demo's `fyi` option on, or stop counting the tier "
            "here — not a call this generator may make on its own."
        )

    entries: list[dict] = []
    for signature, by_total in sorted(by_signature.items()):
        states = [st for group in by_total.values() for st in group]
        if len(by_total) == 1:
            entries.append(
                {
                    "signature": signature,
                    "python": next(iter(by_total)),
                    "states": len(states),
                }
            )
            continue

        majority = max(by_total, key=lambda total: len(by_total[total]))
        majority_cells = {
            tuple(cell.values())
            for st in by_total[majority]
            if (cell := _cell(st)) is not None
        }
        overrides, unresolved = [], []
        for total, group in sorted(by_total.items()):
            if total == majority:
                continue
            for st in group:
                cell = _cell(st)
                if cell is None or tuple(cell.values()) in majority_cells:
                    unresolved.append(st["id"])
                else:
                    overrides.append({**cell, "python": total})
        if unresolved:
            problems.append(
                f"laterite signature {signature!r} has python-ags4 answers "
                f"{sorted(by_total)} and {len(unresolved)} of them "
                f"({', '.join(sorted(unresolved)[:5])}) cannot be told apart by "
                "a single cell the browser can test. The demo would show one of "
                "the two numbers and be wrong about the other."
            )
            continue
        entries.append(
            {
                "signature": signature,
                "python": majority,
                "states": len(states),
                "when_cell_is": overrides,
            }
        )

    return {
        "schema": 1,
        "generator": "tools/gen_demo_state_map.py",
        "source": str(MAP.relative_to(ROOT)),
        "python_ags4_version": doc["python_ags4_version"],
        "measured": {
            "states": len(doc["states"]),
            "signatures": len(entries),
            "laterite_tiers": (
                "errors and warnings, which are the tiers the demo's own "
                "validate call surfaces. The map itself measures FYI too, and "
                "a state that raised one would fail this generator rather "
                "than reach here"
            ),
            "python_total": (
                "every finding python-ags4 reports at any tier, which is what "
                "its own report shows a reader"
            ),
        },
        "not_measured": (
            "a signature absent from this table is a state the sweep never "
            "validated, and the page must say so rather than go quiet. "
            "Silence is indistinguishable from the two engines agreeing, "
            "which is the confusion this whole line of work exists to remove"
        ),
        "signatures": entries,
    }, problems


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="fail on drift, write nothing")
    args = ap.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        out_dir = Path(tmp) / "states"
        manifest = emit_states(out_dir)
        report = dual_validate(out_dir)
        doc, unknown = build_map(manifest, report, python_ags4_version())
    notes, note_gaps = build_notes(doc)
    unknown.extend(note_gaps)
    counts, count_gaps = build_python_counts(doc)
    unknown.extend(count_gaps)

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

    print(
        f"gen_demo_state_map: {len(notes['notes'])} reader note(s) for the "
        f"demo ({', '.join(n['observation'] for n in notes['notes']) or 'none'})"
    )
    resolved = sum(1 for e in counts["signatures"] if "when_cell_is" in e)
    inert = doc["not_covered"]["inert_columns"]
    print(
        f"gen_demo_state_map: {len(counts['signatures'])} laterite signature(s) "
        f"carry a python-ags4 total, {resolved} of them resolving a collision "
        f"by cell; {len(inert)} column(s) where arbitrary text is inert and was "
        "not enumerated"
    )

    rendered = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    rendered_notes = json.dumps(notes, indent=2, ensure_ascii=False) + "\n"
    rendered_counts = json.dumps(counts, indent=2, ensure_ascii=False) + "\n"
    if args.check:
        stale = [
            path
            for path, want in (
                (MAP, rendered),
                (NOTES, rendered_notes),
                (COUNTS, rendered_counts),
            )
            if (path.read_text(encoding="utf-8") if path.exists() else "") != want
        ]
        if stale:
            print(
                "\ngen_demo_state_map: "
                + ", ".join(str(x.relative_to(ROOT)) for x in stale)
                + " has DRIFTED. Re-run without --check and read the diff — "
                "it says which of the four this is: a state whose answer moved "
                "(the states block changes), the OTHER engine moving under us "
                "(`python_ags4_version` changes), the reachable set itself "
                "changing (`counts` and `scope` change), a difference "
                "gaining/losing the note the demo shows for it (only "
                "divergence-notes.json changes), or python-ags4's total moving "
                "for a signature the page already shows (only "
                "python-counts.json changes).",
                file=sys.stderr,
            )
            return 1
        print(
            "gen_demo_state_map: "
            + ", ".join(str(x.relative_to(ROOT)) for x in (MAP, NOTES, COUNTS))
            + " are up to date"
        )
        return 0

    MAP.write_text(rendered, encoding="utf-8")
    NOTES.write_text(rendered_notes, encoding="utf-8")
    COUNTS.write_text(rendered_counts, encoding="utf-8")
    print(
        "gen_demo_state_map: wrote "
        + ", ".join(str(x.relative_to(ROOT)) for x in (MAP, NOTES, COUNTS))
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
