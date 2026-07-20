"""Generate `_laterite_native.pyi` from the AGS4 dictionary.

Reads `rust-packages/laterite-ags4-reference/data/ags_dictionary.json` and emits the
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
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DICT_JSON = (
    REPO_ROOT
    / "rust-packages"
    / "laterite-ags4-reference"
    / "data"
    / "ags_dictionary.json"
)
OUT_PYI = (
    REPO_ROOT / "packages" / "laterite" / "python" / "laterite" / "_laterite_native.pyi"
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


def _stub_params(text_sig: str) -> str:
    """A PyO3 ``__text_signature__`` like ``($self, data, level=9)`` → a stub
    parameter list ``self, data: Any, level: Any = ...``. Types are ``Any`` (PyO3
    exposes names + defaults, not types) and defaults collapse to ``...`` — enough
    to resolve the call and check arity without inventing types. Defaults here are
    scalars (None / ints / bools), so a plain ``, `` split is safe."""
    inner = text_sig.strip()[1:-1].strip()
    if not inner:
        return ""
    out: list[str] = []
    for raw in inner.split(", "):
        p = raw.strip()
        if p == "$self":
            out.append("self")
        elif p in ("*", "/"):  # positional-only / keyword-only markers
            out.append(p)
        elif p.startswith("*"):  # *args / **kwargs
            out.append(f"{p}: Any")
        elif "=" in p:
            out.append(f"{p.split('=', 1)[0]}: Any = ...")
        else:
            out.append(f"{p}: Any")
    return ", ".join(out)


def _emit_native_function(name: str, obj: object) -> str:
    ts = getattr(obj, "__text_signature__", None)
    params = _stub_params(ts) if ts else "*args: Any, **kwargs: Any"
    return f"def {name}({params}) -> Any: ..."


def _emit_native_class(name: str, cls: type) -> str:
    """Stub a native (non-group) class — e.g. the ``Sidecar`` certificate. getset
    descriptors become ``Any`` attributes; a method whose signature has no leading
    ``self`` (a PyO3 ``#[staticmethod]``, called on the type) gets ``@staticmethod``."""
    parts = [f"class {name}:"]
    members = [m for m in sorted(dir(cls)) if not m.startswith("__")]
    if not members:
        parts.append("    ...")
    for mname in members:
        mobj = getattr(cls, mname)
        ts = getattr(mobj, "__text_signature__", None)
        if ts is None and not callable(mobj):  # getset_descriptor (a property)
            parts.append(f"    {mname}: Any")
            continue
        params = _stub_params(ts) if ts else "self, *args: Any, **kwargs: Any"
        if params.split(",", 1)[0].strip() == "self":
            parts.append(f"    def {mname}({params}) -> Any: ...")
        else:  # no self → static/class method invoked on the type
            parts.append("    @staticmethod")
            parts.append(f"    def {mname}({params}) -> Any: ...")
    return "\n".join(parts)


def _emit_native_members(skip: set[str]) -> str:
    """Stub the native module's non-group public members — the internal functions
    (``run_check`` / ``parse_*`` / ``fix_file`` / emit + excel + transport helpers)
    and the ``Sidecar`` class — by introspecting the COMPILED module, so the
    names/signatures are single-sourced from the Rust ``#[pyfunction]`` /
    ``#[pymethods]`` and can't drift from a hand-written table."""
    import laterite._laterite_native as native  # the compiled module is the SoT

    funcs: list[str] = []
    classes: list[str] = []
    for name in sorted(dir(native)):
        if name.startswith("_") or name in skip:
            continue
        obj = getattr(native, name)
        if isinstance(obj, type):
            classes.append(name)
        elif callable(obj):
            funcs.append(name)
    blocks = [_emit_native_function(n, getattr(native, n)) for n in funcs]
    blocks += [_emit_native_class(n, getattr(native, n)) for n in classes]
    return "\n".join(blocks)


def _ruff_format(content: str) -> str:
    """Run the generated stub through the project's `ruff format` so the
    committed `.pyi` is real formatted code, not a file the formatter is told
    to skip. Config is discovered from `REPO_ROOT` (via ``cwd``); the `.pyi`
    stdin-filename makes ruff apply its stub-file rules. This couples the stub
    to ruff's version — a ruff bump that changes wrapping means regenerating —
    which `test_pyi_stubs_match_generator` catches loudly."""
    ruff = shutil.which("ruff") or str(Path(sys.executable).parent / "ruff")
    result = subprocess.run(
        [ruff, "format", "--stdin-filename", str(OUT_PYI), "-"],
        input=content,
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
        check=True,
    )
    return result.stdout


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
        "# AUTO-GENERATED from rust-packages/laterite-ags4-reference/data/ags_dictionary.json\n"
        "# DO NOT EDIT BY HAND. Regenerate via:\n"
        "#   uv run python tools/generate_pyi.py\n"
        "#\n"
        "# Type-stub file for the compiled `laterite._laterite_native`\n"
        "# extension. IDEs and type-checkers consult this to type-check\n"
        "# the standard AGS4 typed-graph classes (`from laterite.groups import\n"
        "# PROJ, LOCA, ...`) AND the internal `_native.<fn>` calls inside\n"
        "# `laterite/__init__.py`. The module's functions (run_check / fix_file /\n"
        "# parse_* / the excel + transport helpers) and the `Sidecar` certificate\n"
        "# class are stubbed by INTROSPECTING the compiled module (param names +\n"
        "# defaults; types are `Any` — PyO3 doesn't expose them), so the calls\n"
        "# resolve without a hand-maintained, drift-prone signature table.\n"
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

    # `__all__` lists every typed-graph class. `laterite/__init__.py` does
    # `if TYPE_CHECKING: from ._laterite_native import *`, which re-exports
    # exactly these names to the package root so `from laterite import PROJ,
    # LOCA, ...` resolves + autocompletes (the runtime globals() alias loop is
    # invisible to static analysers). Restricting the star to the classes keeps
    # the module's internal functions out of the package namespace.
    all_names = [g["code"] for g in groups_sorted]
    all_block = (
        "\n# Re-exported by `laterite.groups` (its TYPE_CHECKING star import) for\n"
        "# `from laterite.groups import PROJ, ...`. DO NOT EDIT BY HAND.\n"
        "__all__ = [\n" + "".join(f"    {name!r},\n" for name in all_names) + "]\n"
    )

    # The native module's internal functions + the Sidecar certificate class,
    # introspected from the compiled module so they resolve for type-checkers
    # without a hand-maintained (drift-prone) table. NOT in `__all__` — they stay
    # internal, reached only as `_native.<name>` from laterite/__init__.py.
    native_header = (
        "\n"
        "# --- Internal native surface (functions + Sidecar) -----------------------\n"
        "# Introspected from the compiled module; single-sourced from the Rust\n"
        "# #[pyfunction] / #[pymethods]. Types are `Any` (PyO3 exposes names +\n"
        "# defaults, not types). Reached as `_native.<name>`; kept out of __all__.\n"
        "\n"
    )
    native_block = _emit_native_members(set(all_names))

    return _ruff_format(
        header
        + "\n".join(body_blocks)
        + native_header
        + native_block
        + "\n"
        + all_block
    )


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
