"""`laterite.compat` mirrors python-ags4's MODULE LAYOUT, not just its names.

The claim this file defends is the announcement's headline: every import shape a
real python-ags4 user writes ports by changing one token, `python_ags4` →
`laterite.compat`. Names alone are not enough — `from python_ags4.AGS4 import
AGS4Error` is third-party production code, and a flat module cannot serve it.

**Why the parity oracle cannot do this job.** `tools/run_python_ags4_tests.sh`
generates a shim that registers `sys.modules["python_ags4.AGS4"] =
sys.modules["python_ags4.check"] = sys.modules["python_ags4.utils"] =
sys.modules["python_ags4.ags4_cli"] = laterite.compat` — four distinct upstream
modules aliased to ONE object. Its 121/131 contract is therefore a statement
about flattened function behaviour and is structurally incapable of seeing a
module boundary. Green there never was evidence of drop-in-ness, and this file
exists precisely because it cannot be.

Three assertions, in increasing strength: the import shapes users actually
write, per-module placement, and module identity.
"""

from __future__ import annotations

import ast
import importlib
import inspect
import json
from pathlib import Path, PurePath

import pytest

REPO = Path(__file__).resolve().parents[3]
GAPS = REPO / "compat-surface-gaps.json"

# The upstream modules laterite mirrors. `ags4_cli` is excluded wholesale —
# laterite ships `lat` instead (compat-surface-gaps.json `excluded_modules`).
MIRRORED_MODULES = ("AGS4", "check", "utils", "data")


# Every entry is a literal import line observed in real code, with the source it
# came from, `python_ags4` swapped for `laterite.compat`. The citation is the
# point: this table is EVIDENCE, not invention, and each row can be checked
# against a real consumer. `†` marks upstream's own material (its README, docs,
# notebook or tests); the rest is third-party production code.
IMPORT_SHAPES: list[tuple[str, str]] = [
    (
        "from laterite.compat import AGS4",
        "† README.md:47, docs/usage.md:39, notebook cell 2, tests/test_ags4.py:5; "
        "third-party: BritishGeologicalSurvey/pyagsapi app/checkers.py:12, "
        "bedrock-engineer/bedrock-ge src/bedrock_ge/gi/ags4.py, ucgmsim/nzgd",
    ),
    (
        "from laterite.compat import AGS4, check, __version__",
        "† tests/test_check.py:3",
    ),
    (
        "from laterite.compat import AGS4, __version__",
        "† tests/test_ags4.py:5",
    ),
    (
        "from laterite.compat import check",
        "† python_ags4/AGS4.py (deferred, inside check_file); "
        "simon969/ge_lib src/ge_lib/ags/AGS4_fs.py",
    ),
    (
        "from laterite.compat.AGS4 import AGS4Error",
        "ucgmsim/nzgd nzgd/extract/bh/ags_parser.py — submodule-direct, "
        "third-party production code a flat module cannot serve",
    ),
    (
        "from laterite.compat.AGS4 import check_file as check_file2, AGS4_to_dataframe",
        "simon969/ge_lib src/ge_lib/ags/AGSWorkingGroup.py",
    ),
    (
        "from laterite.compat.AGS4 import AGS4_to_dataframe",
        "simon969/ge_lib src/ge_lib/ags/AGSData.py, src/ge_lib/ags/AGSCharts.py",
    ),
    (
        "from laterite.compat.AGS4 import AGS4_to_dict",
        "† docs/usage.md — the dict entry point taught beside AGS4_to_dataframe",
    ),
    (
        "from laterite.compat.check import get_TRAN_AGS",
        "† python_ags4/check.py — the one name COMPAT.md maps as user-facing",
    ),
    (
        "from laterite.compat.utils import get_DICT_table_from_json_file",
        "† tests/test_utils.py:2 (`from python_ags4 import AGS4, utils`)",
    ),
    (
        "from laterite.compat.data import TEST_DATA",
        "† tests/test_ags4.py:6, tests/test_check.py:4",
    ),
    (
        "from laterite.compat.data import load_test_data",
        "† README.md:57 and :126, docs/usage.md:49 and :120; "
        "third-party: robinhilderman/Ground-Hazard-Tool streamlit_app.py",
    ),
    (
        "import laterite.compat as python_ags4\nassert python_ags4.__version__",
        "BritishGeologicalSurvey/pyagsapi app/checkers.py:11+:62 — "
        'checker=f"python_ags4 v{python_ags4.__version__}"',
    ),
]


@pytest.mark.parametrize(
    ("statement", "citation"),
    IMPORT_SHAPES,
    ids=[s.splitlines()[0][:60] for s, _ in IMPORT_SHAPES],
)
def test_import_shape_ports_by_one_token(statement: str, citation: str):
    """A real-world import line, with `python_ags4` → `laterite.compat`, works.

    Executed in a fresh namespace so a name that moved fails as ImportError at
    the exact line rather than being masked by an earlier import.
    """
    try:
        exec(compile(statement, "<import-shape>", "exec"), {})
    except Exception as exc:  # pragma: no cover - the failure IS the message
        pytest.fail(
            f"import shape broke: {statement!r}\n"
            f"observed in: {citation}\n"
            f"{type(exc).__name__}: {exc}"
        )


# Plain-data types a module-level assignment can hold and still be API rather
# than machinery. `logger = logging.getLogger(...)` is also a module-level
# assignment, which is why this is a type allowlist and not "everything assigned".
_DATA_TYPES = (str, int, float, bool, dict, list, tuple, set, frozenset, PurePath)


def _own_assignments(mod: object) -> set[str]:
    """Names assigned at module level in upstream's own source."""
    src = inspect.getsourcefile(mod)  # type: ignore[arg-type]
    tree = ast.parse(Path(src).read_text(encoding="utf-8"), filename=str(src))
    return {
        t.id
        for node in tree.body
        if isinstance(node, ast.Assign)
        for t in node.targets
        if isinstance(t, ast.Name) and not t.id.startswith("_")
    }


def _upstream_public(module: str) -> set[str]:
    """The public API of an installed upstream module — its own names, not its
    plumbing.

    python-ags4 is a declared dev dependency, so this reads the real installed
    package: no clone, no network, and it tracks the pinned version rather than
    a hand-list that rots.

    Two exclusions, both deliberate:

    * **Imported modules.** `python_ags4.check.pd` is pandas, not API. Mirroring
      it would mean re-exporting someone else's library under our name.
    * **Objects owned by another distribution.** `Path`, `DataFrame`, `concat`,
      `logger` are all reachable via `dir()` because upstream imported or built
      them; none is a name a port has to carry.

    Two inclusions that a naive "defined here" rule would miss:

    * **Incidental re-exports of upstream's OWN names.** `AGS4Error` is
      importable from `python_ags4.check` because that module imports it, so
      `from laterite.compat.check import AGS4Error` must work too.
    * **Plain-data constants**, which carry no `__module__` of their own —
      `STANDARD_DICT_FILES`, `LATEST_DICT_VERSION`, `TEST_DATA`, `DATA_DIR`.
    """
    mod = pytest.importorskip(f"python_ags4.{module}")
    own = _own_assignments(mod)
    names: set[str] = set()
    for n in dir(mod):
        if n.startswith("_"):
            continue
        obj = getattr(mod, n)
        if inspect.ismodule(obj):
            continue
        home = getattr(obj, "__module__", None) or getattr(type(obj), "__module__", "")
        if home.startswith("python_ags4") or (
            n in own and isinstance(obj, _DATA_TYPES)
        ):
            names.add(n)
    return names


def _known_gaps() -> set[tuple[str, str]]:
    data = json.loads(GAPS.read_text(encoding="utf-8"))
    return {(g["module"], g["name"]) for g in data["known_gaps"]}


@pytest.mark.parametrize("module", MIRRORED_MODULES)
def test_names_live_in_the_same_module_as_upstream(module: str):
    """Placement, not mere presence — the "fails when a name moves" half.

    Driven from the INSTALLED python_ags4 minus `compat-surface-gaps.json`, so
    the expectation tracks upstream rather than a hand-list that rots. A name
    that drifts from `AGS4` to `check` fails here even though the flat
    `laterite.compat` namespace would still resolve it.
    """
    gaps = _known_gaps()
    expected = {n for n in _upstream_public(module) if (module, n) not in gaps}
    ours = importlib.import_module(f"laterite.compat.{module}")

    missing = sorted(n for n in expected if not hasattr(ours, n))
    assert not missing, (
        f"laterite.compat.{module} is missing {missing}.\n"
        f"Either mirror them there, or record each as a deliberate non-mirror "
        f"in compat-surface-gaps.json with a reason."
    )


def test_mirrored_modules_are_distinct_objects():
    """The invariant the parity shim violates.

    The generated oracle shim aliases every upstream submodule to one object.
    Asserting distinctness here is what stops that convenience leaking into the
    product: if `laterite.compat.AGS4 is laterite.compat.check`, then module
    boundaries are decorative and `from laterite.compat.check import <anything
    at all>` would silently succeed.
    """
    mods = [importlib.import_module(f"laterite.compat.{m}") for m in MIRRORED_MODULES]

    assert len({id(m) for m in mods}) == len(MIRRORED_MODULES), (
        f"compat submodules are not distinct objects: {[m.__name__ for m in mods]}"
    )
    for name, mod in zip(MIRRORED_MODULES, mods, strict=True):
        assert mod.__name__ == f"laterite.compat.{name}", mod.__name__


def test_ags4_cli_is_not_importable():
    """`ags4_cli` is a deliberate non-mirror, and the package shape makes that
    honest: it now raises ImportError instead of resolving silently off a flat
    module. laterite ships `lat` instead."""
    with pytest.raises(ImportError):
        importlib.import_module("laterite.compat.ags4_cli")


def test_no_top_level_python_ags4_is_shipped_by_laterite():
    """Shipping our own `python_ags4` is a permanent NON-GOAL, not a gap.

    Two installed distributions cannot both own `site-packages/python_ags4/`.
    If `python_ags4` is importable in this environment it must be the real
    upstream dev dependency — never a module laterite installed.
    """
    upstream = pytest.importorskip("python_ags4")
    origin = Path(upstream.__file__ or "").resolve()
    laterite_pkg = Path(importlib.import_module("laterite").__file__ or "").parent

    assert laterite_pkg not in origin.parents, (
        f"python_ags4 resolved inside the laterite package ({origin}) — "
        "laterite must never ship a top-level python_ags4 import name."
    )


def test_ags3_stub_is_gone():
    """`AGS4_to_dataframe_AGS3` never existed upstream in any version.

    It was a laterite invention that COMPAT.md then listed as mirroring an
    upstream function, i.e. a divergence claimed against nothing. AGS3 is out of
    scope; the real O-30 refusal happens on the ordinary entry points and is
    covered by test_laterite.test_compat_ags3_is_refused.
    """
    compat = importlib.import_module("laterite.compat")
    assert not hasattr(compat, "AGS4_to_dataframe_AGS3")
    assert "AGS4_to_dataframe_AGS3" not in (compat.__all__ or ())


def test_check_constants_mirror_upstream_values():
    """`STANDARD_DICT_FILES` / `LATEST_DICT_VERSION` are mirrored VERBATIM.

    Not "close enough": upstream's fallback is 4.1.1 even though it ships a 4.2
    dictionary, and a file with no TRAN_AGS validates against 4.1.1 there. Ours
    must agree, or the drop-in silently validates against a different edition.
    """
    up_check = pytest.importorskip("python_ags4.check")
    from laterite.compat import check as our_check

    assert our_check.LATEST_DICT_VERSION == up_check.LATEST_DICT_VERSION
    assert our_check.STANDARD_DICT_FILES == up_check.STANDARD_DICT_FILES


def test_flat_namespace_is_unchanged_by_the_reshape():
    """The reshape must be additive: every documented flat entry point survives.

    `from laterite import compat as AGS4` is the form the README, the cookbook
    and `laterite/__init__.py` all teach, and the parity shim imports
    `laterite.compat` as a module object.
    """
    from laterite import compat as AGS4
    from laterite.compat import _impl

    # Everything the engine declares public must reach the package surface. This
    # is the general form; the hand-list below is the part a rename of `__all__`
    # itself could not silently satisfy.
    missing = sorted(n for n in _impl.__all__ if not hasattr(AGS4, n))
    assert not missing, f"flat namespace lost {missing} from _impl.__all__"

    for name in (
        "AGS4_to_dataframe",
        "AGS4_to_dict",
        "check_file",
        "dataframe_to_AGS4",
        "get_TRAN_AGS",
        "get_DICT_table_from_json_file",
        "set_backend",
        "get_backend",
        "set_string_dtype",
        "get_string_dtype",
        "PYTHON_AGS4_COMPAT",
        "__version__",
        # Not in `__all__` and not from `_impl`: these were reachable only
        # because the pre-package module imported them, and `BadDictError` is
        # documented in COMPAT.md's error-handling section. The docs-snippet
        # gate caught their loss during this reshape; asserting them here means
        # the compat suite catches it first next time.
        "Ags4Error",
        "BadDictError",
    ):
        assert hasattr(AGS4, name), f"flat namespace lost {name}"


def test_submodule_docstrings_do_not_reimplement():
    """The shims are re-exports, not a second implementation.

    A submodule that grows a `def` is a fork of the engine waiting to diverge
    from the flat namespace — the one failure mode this layout could introduce.
    """
    pkg = Path(importlib.import_module("laterite.compat").__file__ or "").parent
    for path in sorted(pkg.rglob("*.py")):
        if path.name in {"_impl.py", "__init__.py"}:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        defs = [
            n.name
            for n in tree.body
            if isinstance(n, ast.FunctionDef | ast.AsyncFunctionDef | ast.ClassDef)
        ]
        assert not defs, (
            f"{path.name} declares {defs} — compat submodules must re-export "
            "from _impl, never implement."
        )
