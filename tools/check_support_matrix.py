#!/usr/bin/env python3
"""Assert the support floors we DECLARE are the ones CI actually TESTS.

`engines.node` said `">=18"` for fifteen months. Every Node job in this repo ran
Node 22; nothing had ever built or tested on 18, and 18 had been EOL since
2025-04-30. It was not a policy anyone chose — it was an unverified guess, and
nothing in the repo could see it. #316 fixed the value. This fixes the blind spot.

`tools/check_msrv.py` already does this for Rust: it builds each crate on its own
declared `rust-version`, so the MSRV cannot drift into being aspirational. Python
and Node had no equivalent, and their floors are the ones a consumer plans
against — a wheel that will not install is a harder failure than an API that moved.

The check is deliberately BIDIRECTIONAL, the same shape `gen_changelog.py` uses
for its breaking flag:

  * a version we CLAIM and never test is a false promise;
  * a version we TEST and never claim is a missing classifier — which is how
    "add 3.15" gets half-done (the matrix row lands, the classifier doesn't, and
    the wheel silently supports something its metadata denies).

Both are drift; neither is visible without this.

Usage:
    uv run --no-sync python tools/check_support_matrix.py

Exit 0 when the declared and tested sets agree, 1 when they do not.

Deliberately NOT a JSON SSOT with a rendered view. That idiom (`changelog.json`,
`observations.json`, `modality.json`) earns its ceremony over 174 groups or 50
observations. Over four rows it is pure overhead, and the thing that actually
goes false silently here is not a rendering but declared-vs-tested, which is one
assertion.
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PYPROJECT = ROOT / "packages" / "laterite" / "pyproject.toml"
NODE_PKG = ROOT / "rust-packages" / "laterite-node" / "package.json"
WORKFLOWS = ROOT / ".github" / "workflows"

# `uv python install 3.12 3.13 3.14` and `for v in 3.12 3.13 3.14; do`. Both
# forms name the interpreters a job will actually run something on; a version
# that appears in neither is not tested by anything.
_UV_INSTALL = re.compile(r"uv python install\s+((?:3\.\d+\s*)+)")
_FOR_LOOP = re.compile(r"for\s+v\s+in\s+((?:3\.\d+\s*)+);")
# `node-version: 22`. Quoted or bare, ignoring commented-out lines.
_NODE_VERSION = re.compile(r"^\s*node-version:\s*['\"]?(\d+)['\"]?\s*$", re.M)
_CLASSIFIER = re.compile(r"Programming Language :: Python :: (3\.\d+)")


def _fail(msg: str) -> None:
    print(f"[support-matrix] FAIL: {msg}", file=sys.stderr)


def declared() -> tuple[set[str], str, int]:
    """(python versions claimed, requires-python floor, node floor)."""
    data = tomllib.loads(PYPROJECT.read_text())
    project = data["project"]

    py = set(_CLASSIFIER.findall("\n".join(project.get("classifiers", []))))

    requires = project["requires-python"]
    m = re.search(r">=\s*(3\.\d+)", requires)
    if not m:
        raise SystemExit(
            f"[support-matrix] cannot read a floor from requires-python {requires!r}"
        )
    py_floor = m.group(1)

    engines = json.loads(NODE_PKG.read_text())["engines"]["node"]
    n = re.search(r">=\s*(\d+)", engines)
    if not n:
        raise SystemExit(
            f"[support-matrix] cannot read a floor from engines.node {engines!r}"
        )
    return py, py_floor, int(n.group(1))


def tested() -> tuple[set[str], set[int]]:
    """(python versions CI installs, node versions CI installs)."""
    py: set[str] = set()
    node: set[int] = set()
    for wf in sorted(WORKFLOWS.glob("*.yml")):
        text = wf.read_text()
        for group in _UV_INSTALL.findall(text) + _FOR_LOOP.findall(text):
            py.update(group.split())
        node.update(int(v) for v in _NODE_VERSION.findall(text))
    return py, node


def main() -> int:
    if not WORKFLOWS.is_dir():
        _fail(f"no workflows directory at {WORKFLOWS}")
        return 1

    claimed_py, py_floor, node_floor = declared()
    ci_py, ci_node = tested()

    # An empty scrape reads as "everything agrees" and would make this gate
    # silently useless — the exact failure `check_parity.py` documents for
    # "0 passed". A moved workflow or a reworded step must be loud.
    if not ci_py or not ci_node:
        _fail(
            f"scraped {len(ci_py)} python and {len(ci_node)} node versions from "
            f"{WORKFLOWS} — the patterns no longer match anything, so this gate "
            "is asserting nothing. Fix the patterns, not this check."
        )
        return 1

    ok = True

    # --- python ---------------------------------------------------------
    if claimed_py and py_floor != min(claimed_py, key=lambda v: int(v.split(".")[1])):
        _fail(
            f"requires-python floor is {py_floor} but the lowest classifier is "
            f"{min(claimed_py, key=lambda v: int(v.split('.')[1]))} — the metadata "
            "disagrees with itself"
        )
        ok = False

    if unproven := claimed_py - ci_py:
        _fail(
            f"classifiers claim Python {', '.join(sorted(unproven))} and no workflow "
            "installs it — the claim is untested"
        )
        ok = False

    if unclaimed := ci_py - claimed_py:
        _fail(
            f"CI installs Python {', '.join(sorted(unclaimed))} with no matching "
            "classifier — add the classifier, or stop testing it"
        )
        ok = False

    # --- node -----------------------------------------------------------
    if node_floor not in ci_node:
        _fail(
            f"engines.node declares >={node_floor} and no workflow runs Node "
            f"{node_floor} — the floor is a guess (CI runs "
            f"{', '.join(str(v) for v in sorted(ci_node))})"
        )
        ok = False

    if below := {v for v in ci_node if v < node_floor}:
        _fail(
            f"CI runs Node {', '.join(str(v) for v in sorted(below))}, below the "
            f"declared floor of {node_floor} — one of the two is wrong"
        )
        ok = False

    if ok:
        print(
            f"[support-matrix] OK: Python {', '.join(sorted(claimed_py))} "
            f"(floor {py_floor}) and Node >={node_floor} are each declared and "
            f"tested. CI runs Node {', '.join(str(v) for v in sorted(ci_node))}."
        )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
