"""`needs_env` must name every root test the buildless job cannot run.

The root `tests/` suite is two things wearing one directory. Most of it is repo
hygiene — it opens files and compares them, and needs nothing but a Python
interpreter. A minority needs the synced project environment, and *that
minority* is why the whole suite used to run behind an artifact download and
`cargo build --release`: a doc-only PR paid for a wheel so that a handful of
tests could run.

`repo-gates` splits them by marker rather than by a list in the workflow,
because a list in a workflow is a second place to remember. This file is what
stops the marker rotting, and it guards TWO axes, because there are two ways to
be unrunnable in a job that installs nothing:

  1. the module imports the compiled package, or drives the `lat` binary;
  2. the module imports a third party the job does not install.

The second is the nastier one, and it is not hypothetical — it is why this file
has a second axis at all. `test_vendored_authority_faithful` imports
`python_ags4`, a pinned dev-only dependency. It collected perfectly on a
developer's machine, where that package is present, and died at COLLECTION on a
clean runner — which fails the entire suite rather than one test, with a
ModuleNotFoundError that names the package but not the reason.

Detection is deliberately syntactic: an AST walk for imports, plus a text search
for the out-of-process helpers. It cannot be fooled by a lazy import inside a
function, which is the point — a lazy import still needs the package installed.

What the marker does NOT do is divide the work. The `python` job runs this suite
in full, wheel and all; `repo-gates` runs the buildless part early. A module
that needs the environment for only SOME of its tests (test_landing_demo_delivery
`importorskip`s the wheel per test) therefore needs no marker at all — it
degrades to skips in the fast job and runs for real in the slow one.
"""

from __future__ import annotations

import ast
import importlib.util
import re
import sys
from functools import cache
from pathlib import Path

import pytest

TESTS = Path(__file__).resolve().parent
REPO = TESTS.parent
CI = REPO / ".github/workflows/ci.yml"

#: Names that only exist once something has been compiled.
BUILT_PACKAGES = {"laterite", "laterite_native", "_laterite_native"}

#: Repo-local names an import walk will see but which pip never installs.
#: `_tools` is this suite's shared script loader (tests/_tools.py, #925).
LOCAL_MODULES = {"tests", "tools", "conftest", "_tools"}

#: Text that betrays a module running project code OUT OF PROCESS, where an
#: import walk cannot follow. `native` is this suite's fixture for the resolved
#: `lat` path; `two-lat-programs` in the wiki is why resolving it from PATH is
#: banned. `sys.executable` is here because re-entering Python to run a script
#: is the same dependency at one remove — test_docs_examples spawns the docs
#: examples, and it is the examples that import `laterite`.
OUT_OF_PROCESS = ("native_cli", "lat_binary", "str(native)", "sys.executable")


def _buildless_provides() -> set[str]:
    """Packages `repo-gates` installs, read out of its own `--with` flags.

    Derived rather than restated, so adding a `--with` and adding an import
    cannot drift apart.
    """
    text = CI.read_text(encoding="utf-8")
    start = text.index("\n  repo-gates:")
    # The next job header: a line at exactly two spaces of indent. Anchored so
    # a folded `run: >-` block's continuation lines — also indented — cannot end
    # the slice early, which is a bug this function shipped with once.
    match = re.search(r"\n {2}[a-z][a-z0-9-]*:\n", text[start + 1 :])
    end = start + 1 + (match.start() if match else len(text))
    flags = re.findall(r"--with ([A-Za-z0-9_.-]+)", text[start:end])
    assert flags, "found no `--with` flags in repo-gates — has the job moved?"
    # uv takes distribution names; imports use module names.
    aliases = {"pyyaml": "yaml", "python-ags4": "python_ags4"}
    return {aliases.get(f.lower(), f.lower().replace("-", "_")) for f in flags}


def _third_party_imports(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    found: set[str] = set()
    for node in ast.walk(tree):
        names: list[str] = []
        if isinstance(node, ast.Import):
            names = [a.name.split(".")[0] for a in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module and node.level == 0:
            names = [node.module.split(".")[0]]
        for name in names:
            if name not in sys.stdlib_module_names and name not in LOCAL_MODULES:
                found.add(name)
    return found


def _needs_env(path: Path) -> str | None:
    """Why the buildless job cannot run this module, or None."""
    imports = _third_party_imports(path)
    built = imports & BUILT_PACKAGES
    if built:
        return f"imports {sorted(built)[0]}"
    source = path.read_text(encoding="utf-8")
    for marker in OUT_OF_PROCESS:
        if marker in source:
            return f"runs project code out of process ({marker})"
    missing = imports - BUILT_PACKAGES - _buildless_provides()
    if missing:
        return f"imports {sorted(missing)}, which the job does not install"
    return None


def _is_marked(path: Path) -> bool:
    return "pytest.mark.needs_env" in path.read_text(encoding="utf-8")


ROOT_TESTS = sorted(p for p in TESTS.glob("test_*.py") if p.name != Path(__file__).name)


def test_the_suite_is_actually_split() -> None:
    """A guard that never fires is a guard nobody has read.

    If the marked set grew to most of the suite, the buildless job would be
    running almost nothing and the split would be a fiction with a green tick
    on it. Half is the line: past that, the fast job has stopped being the
    point and the marker is being reached for too easily.
    """
    marked = [p.name for p in ROOT_TESTS if _is_marked(p)]
    assert marked, "no root test carries needs_env — has the marker been renamed?"
    assert len(marked) < len(ROOT_TESTS) / 2, (
        f"{len(marked)} of {len(ROOT_TESTS)} root tests are marked needs_env, so "
        "the buildless job runs less than half the suite: {marked}"
    )


@pytest.mark.parametrize("path", ROOT_TESTS, ids=lambda p: p.name)
def test_env_dependent_tests_are_marked(path: Path) -> None:
    reason = _needs_env(path)
    if reason is None:
        return
    assert _is_marked(path), (
        f"{path.name} {reason}, so it cannot run in the buildless `repo-gates` "
        "job — add `pytestmark = pytest.mark.needs_env` beneath its imports, or "
        "add the package to the job's `--with` flags. Left unmarked it fails at "
        "COLLECTION, which takes the whole suite down, far from the cause."
    )


@pytest.mark.parametrize("path", ROOT_TESTS, ids=lambda p: p.name)
def test_unmarked_tests_stay_buildless(path: Path) -> None:
    """The inverse: a marker nobody needs sends a cheap test back to the slow job.

    This is the direction that decays quietly — marking is the safe-looking
    move, and each unnecessary one pulls a file-reading test back behind a
    release build where nothing will ever notice.
    """
    if not _is_marked(path):
        return
    assert _needs_env(path) is not None, (
        f"{path.name} is marked needs_env but imports nothing the buildless job "
        "lacks and runs nothing out of process. Drop the marker."
    )


@cache
def _root_conftest():
    """Import THIS suite's conftest.py by path, never by the bare name.

    `from conftest import …` resolves to whichever conftest.py claimed that
    name in `sys.modules` first — and with both suites in one serial session,
    root-suite-first, the wheel suite's conftest owns it by the time this
    test runs (#828). Loading by file path is order-proof; the module is
    deliberately NOT registered in `sys.modules`, so the bare name stays
    whatever pytest made it.
    """
    spec = importlib.util.spec_from_file_location(
        "_marker_gate_root_conftest", TESTS / "conftest.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


@pytest.mark.parametrize("path", ROOT_TESTS, ids=lambda p: p.name)
def test_the_ignore_hook_reads_the_same_marker(path: Path) -> None:
    """The marker and the hook that acts on it must see the same set.

    `-m "not needs_env"` cannot help a module that fails on IMPORT, so
    tests/conftest.py re-reads the marker before collection, off the source
    text. Two readers of one marker is two chances to disagree — a
    `pytestmark = [pytest.mark.needs_env, pytest.mark.slow]` list, say, that one
    of them handles and the other does not.
    """
    _declares_marker = _root_conftest()._declares_marker

    assert _declares_marker(path) == _is_marked(path), (
        f"{path.name}: the collection-time hook and the marker text disagree. "
        "Whichever is wrong, the buildless job either imports a module it "
        "cannot import or silently drops one it could have run."
    )
