"""#294 Backbone — **cross-surface behavioural-knob parity** (Python ↔ Node).

`laterite` is one engine behind several doors. A behavioural knob (`encoding`,
`dict_version`, `warnings`/`fyi`/`check_files`, `risky`, `mode`, `units`/`types`,
…) that lands on one library surface but not the other is the exact
capability-reachability drift #294 is about — it slips through the green suite
because nothing asserts the doors *agree*.

This gate compares the two in-repo **library** surfaces that are statically
introspectable: **Python** (`laterite.<verb>`) and **Node** (the TS option
interfaces in `rust-packages/laterite-node/ts/index.ts`). For each shared
operation it extracts the behavioural knobs from both — Python via `inspect`,
Node by parsing the interface fields — maps `snake_case ↔ camelCase`, and asserts
they match after removing each side's declared input/IO params and an explicit
**by-design allowlist**. A future knob added to one side and forgotten on the
other fails CI; the allowlist is the *spec* of the intentional gaps, so it can't
rot silently (`test_allowlist_is_live`).

Scope: the two headless *library* surfaces. The CLI is covered by the free↔chained
gate + `_cli`'s own tests; the browser (wasm) exposes a different JSON-shaped API;
DuckDB is a read-query-validate surface in a separate repo (its by-design gaps —
no fix/emit — are catalogued in #294, not here). Python free↔chained parity is the
sibling gate `test_free_chained_parity`.
"""

from __future__ import annotations

import inspect
import re
from pathlib import Path

import laterite as L
import pytest

_INDEX_TS = (
    Path(__file__).parents[3] / "rust-packages" / "laterite-node" / "ts" / "index.ts"
)

# operation -> (python free fn, python input/IO params to drop, node interface,
#               node-missing allowlist: Python behavioural knobs deliberately NOT
#               on Node — each with the reason it's a by-design gap, not drift).
_MATRIX: dict[str, tuple] = {
    "validate": (L.validate, {"source", "text"}, "ValidateOptions", {}),
    "fix": (
        L.fix,
        {"source", "path", "text", "data", "in_place", "out"},
        "FixOptions",
        # only/exclude landed on Node in #394 — the allowlist shrank to empty
        # (the whole point: closing the gap removes its by-design entry).
        {},
    ),
    "diff": (L.diff, {"a", "b"}, "DiffOptions", {}),
    "build": (L.build_ags4, {"groups"}, "EmitOptions", {}),
}


def _snake_to_camel(name: str) -> str:
    head, *tail = name.split("_")
    return head + "".join(w[:1].upper() + w[1:] for w in tail)


def _py_knobs(fn, drop: set[str]) -> set[str]:
    return {
        n
        for n, p in inspect.signature(fn).parameters.items()
        if n not in drop
        and n != "self"
        and p.kind not in (p.VAR_POSITIONAL, p.VAR_KEYWORD)
    }


def _ts_interface_fields(src: str, name: str, _seen: set[str] | None = None) -> set[str]:
    """The field names of a TS `interface`, following one `extends` chain (so
    `ValidateOptions extends ReadOptions` inherits `encoding`/`text`/…). Fields are
    the identifiers before `?:` / `:` at the top of the interface body."""
    _seen = _seen or set()
    if name in _seen:
        return set()
    _seen.add(name)
    m = re.search(rf"export interface {name}(?: extends (\w+))?\s*\{{(.*?)\n\}}", src, re.DOTALL)
    if m is None:
        raise AssertionError(f"interface {name} not found in index.ts")
    parent, body = m.group(1), m.group(2)
    fields = set(re.findall(r"^\s*(\w+)\??\s*:", body, re.MULTILINE))
    if parent:
        fields |= _ts_interface_fields(src, parent, _seen)
    return fields


# Node option fields that are input/IO selectors, not behavioural knobs (the
# analog of the Python `drop` sets) — `text` is an input door, `index` is a cert
# path, and `inPlace`/`out` are fix write-back destinations (the camelCase twins
# of Python's dropped `in_place`/`out`). None is a behavioural knob to compare.
_NODE_IO = {"text", "index", "inPlace", "out"}


@pytest.mark.parametrize("op", list(_MATRIX))
def test_python_node_behavioural_knobs_match(op):
    free_fn, py_drop, iface, node_missing = _MATRIX[op]
    src = _INDEX_TS.read_text(encoding="utf-8")

    py = _py_knobs(free_fn, py_drop)
    node = _ts_interface_fields(src, iface) - _NODE_IO

    # Compare in Node's camelCase vocabulary; drop the allowlisted by-design gaps.
    py_camel = {_snake_to_camel(n) for n in py if n not in node_missing}
    missing_on_node = py_camel - node
    extra_on_node = node - py_camel

    assert not missing_on_node, (
        f"{op}: knob(s) on Python but missing from Node {iface}: "
        f"{sorted(missing_on_node)} — add them to Node, or to the _MATRIX "
        f"node-missing allowlist with a reason if intentional"
    )
    assert not extra_on_node, (
        f"{op}: knob(s) on Node {iface} but missing from Python: "
        f"{sorted(extra_on_node)} — add them to Python or reconcile the matrix"
    )


def test_allowlist_is_live():
    """Hygiene on the allowlist: every declared Node-missing knob must really be a
    Python knob for that op (else a rename left a stale entry masking a real gap),
    AND must really be absent from the Node interface (else the gap was closed and
    the allowlist should shrink)."""
    src = _INDEX_TS.read_text(encoding="utf-8")
    for op, (free_fn, py_drop, iface, node_missing) in _MATRIX.items():
        py = _py_knobs(free_fn, py_drop)
        node = _ts_interface_fields(src, iface)
        for knob in node_missing:
            assert knob in py, f"{op}: stale allowlist entry {knob!r} (not a Python knob)"
            assert _snake_to_camel(knob) not in node, (
                f"{op}: {knob!r} is now on Node — remove it from the allowlist"
            )
