"""Generate `_laterite_native.pyi` from the AGS5 dictionary.

Reads `rust-packages/laterite-ags4-core/data/ags_dictionary.json` and emits the
type-stub file that sits next to the compiled `_laterite_native.so`,
giving IDE autocomplete and mypy/pyright type-checking on the standard
AGS4 typed-graph classes (the union of the official 4.0.3-4.2 dictionary).

Run after every dictionary edit::

    uv run python tools/generate_pyi.py

The companion test `tests/test_pyi_stubs_match_generator.py` re-runs
the generator and asserts byte-equality with the committed file —
catches dictionary-edit drift in CI.

F2b-6a. The compile-time codegen in `rust-packages/laterite-py/build.rs`
mirrors this type mapping; the two MUST stay in sync.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DICT_JSON = REPO_ROOT / "rust-packages" / "laterite-ags4-core" / "data" / "ags_dictionary.json"
OUT_PYI = (
    REPO_ROOT
    / "packages"
    / "laterite"
    / "python"
    / "laterite"
    / "_laterite_native.pyi"
)


# AGS type → Python type literal in the stub. Mirrors
# `rust_type` in build.rs and `canonical_type` in laterite_ags4_core::ags_types.
_STRING_TYPES = {"ID", "X", "PA", "PT", "PU", "T", "U", "DMS", "MC", "XN"}


def _py_type(ags_type: str) -> str:
    """Map an AGS spec type code to a Python type literal."""
    t = (ags_type or "").strip().upper()
    if t in _STRING_TYPES:
        return "str"
    if t == "0DP":
        return "int"
    if t == "DT":
        return "_dt.datetime"
    if t == "YN":
        return "bool"
    if t == "RL":
        return "float"
    for suffix in ("DP", "SF", "SCI"):
        prefix = t.removesuffix(suffix)
        if prefix and prefix != t and prefix.isdigit():
            return "float"
    return "str"


def _children_of(groups: list[dict]) -> dict[str, list[str]]:
    """parent code → list of direct child codes (sorted alphabetically
    for stable output)."""
    by_parent: dict[str, list[str]] = {}
    for g in groups:
        if g.get("parent"):
            by_parent.setdefault(g["parent"], []).append(g["code"])
    for parent in by_parent:
        by_parent[parent].sort()
    return by_parent


def _emit_class(g: dict, children: list[str]) -> str:
    """Emit one stub class block for an AGS group."""
    code = g["code"]
    parts: list[str] = []
    parts.append(f"class {code}:")
    # Scalar attributes.
    for h in g["headings"]:
        py_field = h["name"].lower()
        py_type = _py_type(h["type"])
        parts.append(f"    {py_field}: {py_type} | None")
    # Child list fields.
    for child_code in children:
        field = f"{child_code.lower()}s"
        parts.append(f"    {field}: list[{child_code}]")
    # Constructor: keyword-only, every field with a default.
    parts.append("    def __init__(")
    parts.append("        self,")
    parts.append("        *,")
    for h in g["headings"]:
        py_field = h["name"].lower()
        py_type = _py_type(h["type"])
        parts.append(f"        {py_field}: {py_type} | None = ...,")
    for child_code in children:
        field = f"{child_code.lower()}s"
        parts.append(f"        {field}: list[{child_code}] | None = ...,")
    parts.append("    ) -> None: ...")
    # walk + __repr__ (uniform across every class).
    parts.append("    def walk(self, code: str) -> list[Any]: ...")
    parts.append("    def __repr__(self) -> str: ...")
    parts.append("")
    return "\n".join(parts)


def generate() -> str:
    """Build the full `.pyi` content from the dictionary."""
    data = json.loads(DICT_JSON.read_text(encoding="utf-8"))
    # Heading-local schema: `groups` is a {CODE: group} map. Inject the code and
    # take the flat headings (the UNION at each heading's latest-edition def;
    # the `by_ed`/`eds` per-edition variation is ignored). Mirrors build.rs.
    groups: list[dict] = [{"code": code, **g} for code, g in data["groups"].items()]
    children = _children_of(groups)
    # Sort groups alphabetically for stable, easy-to-diff output;
    # `from __future__ import annotations` makes forward refs work
    # so we don't need topo order.
    groups_sorted = sorted(groups, key=lambda g: g["code"])

    header = (
        "# AUTO-GENERATED from rust-packages/laterite-ags4-core/data/ags_dictionary.json\n"
        "# DO NOT EDIT BY HAND. Regenerate via:\n"
        "#   uv run python tools/generate_pyi.py\n"
        "#\n"
        "# Type-stub file for the compiled `laterite._laterite_native`\n"
        "# extension. IDEs and type-checkers consult this to type-check\n"
        "# code that imports the standard AGS4 typed-graph classes\n"
        "# (`from laterite import PROJ, LOCA, ...`). The module's internal\n"
        "# functions (run_check / fix_file / list_rules / parse_* / the\n"
        "# excel + transport helpers / Sidecar) are reached through the\n"
        "# typed Python wrappers in `laterite/__init__.py`, which carry the\n"
        "# annotations, so they are not stubbed here.\n"
        "#\n"
        "# Custom / passthrough groups built at runtime via\n"
        "# `laterite.dynamic.get_or_register` are NOT typed in this stub —\n"
        "# they show as `Any` to type checkers (acceptable; their schema\n"
        "# isn't known until a file is read).\n"
        "\n"
        "from __future__ import annotations\n"
        "\n"
        "import datetime as _dt\n"
        "from typing import Any\n"
        "\n"
    )

    body_blocks = [_emit_class(g, children.get(g["code"], [])) for g in groups_sorted]

    # Only the typed-graph classes are stubbed. The native module's internal
    # functions (run_check / fix_file / list_rules / parse_* / excel /
    # transport / Sidecar) are reached through the typed Python wrappers in
    # `laterite/__init__.py`, which carry the annotations — so they are
    # deliberately not stubbed here. (The old `ags5db_*` stubs were removed:
    # the AGS5 surface was decoupled in #177, so they typed symbols the
    # module no longer registers.)
    return header + "\n".join(body_blocks) + "\n"


def main() -> int:
    content = generate()
    if "--check" in sys.argv:
        existing = OUT_PYI.read_text(encoding="utf-8") if OUT_PYI.exists() else ""
        if existing != content:
            print(
                f"{OUT_PYI} is out of date — regenerate with "
                "`uv run python tools/generate_pyi.py`",
                file=sys.stderr,
            )
            return 1
        print(f"{OUT_PYI.name}: in sync")
        return 0
    OUT_PYI.write_text(content, encoding="utf-8")
    print(f"wrote {OUT_PYI} ({len(content):,} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
