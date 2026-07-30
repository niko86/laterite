"""#294 Backbone — **cross-surface behavioural-knob parity** (Python ↔ Node ↔ wasm).

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

**wasm joined this gate when its exports took options objects.** It used to be
excluded, and the reason was real: a positional signature has no field names to
compare, so `validate(bytes, undefined, true, false, …)` could not be lined up
against a Python keyword list at all. Now that every migrated export takes one
named-options struct, it can — and the comparison reads the `WasmOptions::KEYS`
consts rather than the hand-written TypeScript interfaces, because KEYS is what
the runtime guard actually **accepts**. The TS interface is the declared
contract; KEYS is the enforced one, and a gate should compare the enforced one.
(`option_keys_match_the_structs`, in the wasm crate, holds KEYS to the struct's
own serde fields, so the chain from here to the shipped behaviour is closed.)

Scope: the headless *library* surfaces plus the browser. The CLI is covered by the
free↔chained gate + `_cli`'s own tests; DuckDB is a read-query-validate surface in
a separate repo (its by-design gaps — no fix/emit — are catalogued in #294, not
here). Python free↔chained parity is the sibling gate `test_free_chained_parity`.
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
    # merge joined when wasm did: it was never compared on ANY axis before, so
    # `dict_version` being reachable from Python and the CLI but not the browser
    # went unnoticed for as long as the browser had a merge door.
    "merge": (L.merge, set(), "MergeOptions", {}),
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


def _ts_interface_fields(
    src: str, name: str, _seen: set[str] | None = None
) -> set[str]:
    """The field names of a TS `interface`, following one `extends` chain (so
    `ValidateOptions extends ReadOptions` inherits `encoding`/`text`/…). Fields are
    the identifiers before `?:` / `:` at the top of the interface body."""
    _seen = _seen or set()
    if name in _seen:
        return set()
    _seen.add(name)
    m = re.search(
        rf"export interface {name}(?: extends (\w+))?\s*\{{(.*?)\n\}}", src, re.DOTALL
    )
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
            assert knob in py, (
                f"{op}: stale allowlist entry {knob!r} (not a Python knob)"
            )
            assert _snake_to_camel(knob) not in node, (
                f"{op}: {knob!r} is now on Node — remove it from the allowlist"
            )


#: Repo path to the wasm surface, whose `WasmOptions::KEYS` consts are the
#: enforced option vocabulary (see the module docstring for why KEYS and not the
#: TypeScript interfaces).
_WASM_LIB = (
    Path(__file__).parents[3]
    / "rust-packages"
    / "laterite-ags4-wasm"
    / "src"
    / "lib.rs"
)

#: op -> (wasm options struct, allowlist of by-design gaps with the REASON).
#:
#: An entry here is a claim that the surfaces *should* differ. `_MATRIX`'s Node
#: allowlists have the same contract, and `test_wasm_allowlist_is_live` gives
#: these the same anti-rot treatment.
_WASM: dict[str, tuple[str, dict[str, str]]] = {
    "validate": (
        "ValidateOptions",
        {
            "check_files": (
                "Python-only: walks FILE/ on disk. The wasm sandbox has no "
                "filesystem, so there is nothing for the browser to check."
            ),
            "max_per_rule": (
                "wasm-only: clips how many findings per rule CROSS the "
                "wasm→JS boundary, so an interactive view of a pathologically "
                "dirty file moves thousands of rows instead of millions. Every "
                "rule still runs over every line and the reported totals stay "
                "uncapped. A library caller has no boundary to protect and no "
                "counterpart should be invented."
            ),
        },
    ),
    "build": (
        "BuildOptions",
        {
            "units": (
                "Different door, not a missing knob: wasm carries per-heading "
                "UNIT/TYPE overrides INSIDE each group of `groups_json` "
                "(`GroupInputJson.units`), where Python takes a separate "
                "`{code: {heading: value}}` map. Same capability, expressed in "
                "the shape each surface's input already had."
            ),
            "types": "See `units` — the same per-group input shape.",
        },
    ),
    "merge": ("MergeOptions", {}),
}


def _wasm_keys(struct: str) -> set[str]:
    """The `KEYS` const on `impl WasmOptions for <struct>`, in camelCase.

    Parsed from source rather than imported because this is a Python test
    reading a Rust surface — the same reason `_ts_interface_fields` parses
    TypeScript. The wasm crate's own `option_keys_match_the_structs` is what
    keeps KEYS honest against the struct; this only has to read it correctly.
    """
    src = _WASM_LIB.read_text(encoding="utf-8")
    m = re.search(
        rf"impl WasmOptions for {struct}\s*\{{\s*const KEYS[^=]*=\s*&\[(.*?)\];",
        src,
        re.DOTALL,
    )
    if m is None:
        raise AssertionError(f"no `impl WasmOptions for {struct}` in {_WASM_LIB.name}")
    keys = set(re.findall(r'"(\w+)"', m.group(1)))
    assert keys, f"{struct}: KEYS parsed as empty — the regex has drifted"
    return keys


@pytest.mark.parametrize("op", list(_WASM))
def test_wasm_behavioural_knobs_match_python(op):
    """Every knob reachable from Python is reachable from the browser, and back.

    This is what the options-object migration bought. While the wasm exports
    were positional there was nothing to compare, so a knob could land on one
    surface and never on the other with no gate noticing — which is exactly how
    `merge` came to resolve its edition from `TRAN_AGS` with no override, while
    Python and the CLI both took `dict_version`.
    """
    free_fn, py_drop, _iface, _node_missing = _MATRIX[op]
    struct, allow = _WASM[op]

    py = _py_knobs(free_fn, py_drop)
    wasm = _wasm_keys(struct)

    py_camel = {_snake_to_camel(n) for n in py if n not in allow}
    wasm_cmp = {k for k in wasm if k not in {_snake_to_camel(a) for a in allow}}

    missing_on_wasm = py_camel - wasm_cmp
    extra_on_wasm = wasm_cmp - py_camel

    assert not missing_on_wasm, (
        f"{op}: knob(s) on Python but missing from wasm {struct}: "
        f"{sorted(missing_on_wasm)} — add them to the struct AND its KEYS, or "
        f"to the _WASM allowlist with the reason if the gap is by design"
    )
    assert not extra_on_wasm, (
        f"{op}: knob(s) on wasm {struct} but missing from Python: "
        f"{sorted(extra_on_wasm)} — add them to Python or allowlist with a reason"
    )


def test_wasm_allowlist_is_live():
    """Same anti-rot contract as `test_allowlist_is_live`, for the wasm side.

    A stale entry is worse than none: it reads as "known and tracked" while the
    gap has either been closed (so the allowlist now hides that the gate passes
    on its own) or renamed (so it silently protects nothing).
    """
    for op, (struct, allow) in _WASM.items():
        free_fn, py_drop, _iface, _node_missing = _MATRIX[op]
        py = _py_knobs(free_fn, py_drop)
        wasm = _wasm_keys(struct)
        for knob, reason in allow.items():
            assert reason.strip(), f"{op}: allowlisted {knob!r} with no reason"
            camel = _snake_to_camel(knob)
            on_py, on_wasm = knob in py, camel in wasm
            assert on_py or on_wasm, (
                f"{op}: allowlisted {knob!r} exists on NEITHER surface — a "
                f"rename left a stale entry that now masks nothing"
            )
            assert not (on_py and on_wasm), (
                f"{op}: allowlisted {knob!r} is present on BOTH Python and wasm "
                f"— the gap closed; delete the entry so the gate guards it"
            )
