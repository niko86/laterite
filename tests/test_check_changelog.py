"""Which manifest table moved, not which file changed.

`check_changelog.py` decides whether a PR touched a shipped surface. It used to
decide by PATH alone, which fired on dev-tooling bumps that change nothing a
consumer receives — `globals` in laterite-node (#233), `ruff`/`ty`/`marimo` in the
root pyproject (#236). Both were waved through with `no-changelog`, and that
label only works while applying it stays a deliberate act.

The risk in fixing it is the opposite one: a classifier that is too generous
silently stops asking for entries. So the cases below are weighted toward the
directions that must NOT relax — an unknown section, an unparseable manifest, a
one-sided file — because those are the ones that fail quietly if wrong.

The gate's own history is the argument for testing it: it exists because
#178-#182 merged with an empty `[Unreleased]` and nothing looked.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load():
    spec = importlib.util.spec_from_file_location(
        "check_changelog", REPO / "tools" / "check_changelog.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_changelog"] = mod
    spec.loader.exec_module(mod)
    return mod


cc = _load()


# --- routing ------------------------------------------------------------------


@pytest.mark.parametrize(
    ("path", "kind"),
    [
        ("pyproject.toml", "pyproject.toml"),
        ("packages/laterite/pyproject.toml", "pyproject.toml"),
        ("rust-packages/Cargo.toml", "Cargo.toml"),
        ("rust-packages/laterite-ags4-core/Cargo.toml", "Cargo.toml"),
        ("rust-packages/laterite-node/package.json", "package.json"),
        ("rust-packages/laterite-ags4-core/src/lib.rs", None),
        ("packages/laterite/python/laterite/__init__.py", None),
    ],
)
def test_manifest_kind(path: str, kind: str | None) -> None:
    assert cc.manifest_kind(path) == kind


@pytest.mark.parametrize(
    ("path", "follows"),
    [
        ("rust-packages/laterite-node/package-lock.json", True),
        ("uv.lock", True),
        ("web/pnpm-lock.yaml", True),
        # NOT a follower — see below.
        ("rust-packages/Cargo.lock", False),
        ("rust-packages/Cargo.toml", False),
        ("rust-packages/laterite-node/package.json", False),
    ],
)
def test_lock_follows_manifest(path: str, *, follows: bool) -> None:
    assert cc.lock_follows_manifest(path) is follows


def test_cargo_lock_ships_on_its_own() -> None:
    """#237 changed `rust-packages/Cargo.lock` and NOTHING else — the manifest
    range already admitted the new versions — while bumping napi 3.11 -> 3.12,
    which is compiled into the published addon.

    Treating every lock as a follower (as the issue originally proposed) would
    have passed that PR with no entry and no label. `Cargo.lock` decides what is
    COMPILED into the artefacts this repo ships; npm and uv consumers resolve
    their own tree from the published ranges, so those locks are ours alone.
    """
    assert cc.is_shipped_lockfile("rust-packages/Cargo.lock") is True
    assert cc.lock_follows_manifest("rust-packages/Cargo.lock") is False
    assert cc.shipped("rust-packages/Cargo.lock") is True

    for ours in ("rust-packages/laterite-node/package-lock.json", "uv.lock"):
        assert cc.is_shipped_lockfile(ours) is False


# --- `tool.*` must not share a verdict ----------------------------------------


def test_tool_subtables_are_distinguished() -> None:
    """`[tool.ruff]` and `[tool.maturin]` are both under `tool`; collapsing them
    would let a wheel-build change ride in on a linter bump."""
    before = {"tool": {"ruff": {"line-length": 88}, "maturin": {"features": ["a"]}}}
    after = {"tool": {"ruff": {"line-length": 100}, "maturin": {"features": ["a"]}}}
    assert cc.changed_sections(before, after) == {"tool.ruff"}

    after2 = {"tool": {"ruff": {"line-length": 88}, "maturin": {"features": ["b"]}}}
    assert cc.changed_sections(before, after2) == {"tool.maturin"}


# --- pyproject ----------------------------------------------------------------

_PY_BASE = """\
[project]
name = "laterite"
dependencies = ["polars>=1", "duckdb>=1"]

[dependency-groups]
dev = ["ruff==0.1.0", "ty==0.0.65"]

[tool.ruff]
line-length = 88
"""


def test_dependency_group_bump_is_dev_only() -> None:
    """#236: marimo/hypothesis/ruff/ty all live in `[dependency-groups]`."""
    after = _PY_BASE.replace("ruff==0.1.0", "ruff==0.2.0")
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, after) is True


def test_project_dependencies_change_is_shipped() -> None:
    """The dep-shape split is a user-visible contract (CLAUDE.md)."""
    after = _PY_BASE.replace('"polars>=1"', '"polars>=2"')
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, after) is False


def test_requires_python_change_is_shipped() -> None:
    after = _PY_BASE.replace(
        "[dependency-groups]", 'requires-python = ">=3.13"\n\n[dependency-groups]'
    )
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, after) is False


def test_a_mixed_change_is_shipped() -> None:
    """One shipped section among dev ones must not be masked by the majority."""
    after = _PY_BASE.replace("ruff==0.1.0", "ruff==0.2.0").replace(
        '"polars>=1"', '"polars>=2"'
    )
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, after) is False


def test_an_unrecognised_tool_table_is_shipped() -> None:
    """Unknown must never buy silence — `[tool.maturin]` decides how the wheel
    is built, and nothing in the allowlist covers it."""
    after = _PY_BASE + '\n[tool.maturin]\nfeatures = ["pyo3/abi3"]\n'
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, after) is False


def test_no_change_at_all_is_not_dev_only() -> None:
    """An identical manifest has moved no dev section; it must not report as one
    (the caller would otherwise list an untouched file as 'set aside')."""
    assert cc.dev_only_change("pyproject.toml", _PY_BASE, _PY_BASE) is False


# --- Cargo.toml ---------------------------------------------------------------

_CARGO = """\
[package]
name = "laterite-ags4-core"
version = "0.9.0"

[dependencies]
memchr = "2.7"

[dev-dependencies]
proptest = "1.5"
"""


def test_cargo_dev_dependencies_are_dev_only() -> None:
    after = _CARGO.replace('proptest = "1.5"', 'proptest = "1.6"')
    assert cc.dev_only_change("Cargo.toml", _CARGO, after) is True


def test_cargo_dependencies_are_shipped() -> None:
    after = _CARGO.replace('memchr = "2.7"', 'memchr = "2.8"')
    assert cc.dev_only_change("Cargo.toml", _CARGO, after) is False


def test_workspace_dependencies_are_shipped() -> None:
    """`[workspace.dependencies]` is where the real versions live, and it sits
    under `workspace` — so a bump there stays shipped while a crate's own
    `[dev-dependencies]` does not. (#237 moved only the lock, not this table;
    `test_cargo_lock_ships_on_its_own` is the one that covers that case.)"""
    before = '[workspace.dependencies]\nnapi = "3.11"\n'
    after = '[workspace.dependencies]\nnapi = "3.12"\n'
    assert cc.dev_only_change("Cargo.toml", before, after) is False


# --- package.json -------------------------------------------------------------

_PKG = """\
{
  "name": "laterite",
  "version": "0.10.1",
  "dependencies": { "apache-arrow": "^21" },
  "devDependencies": { "globals": "14.0.0", "eslint": "^10" }
}
"""


def test_dev_dependencies_bump_is_dev_only() -> None:
    """#233: `globals` 14 -> 17 is an eslint devDependency; the package's only
    runtime dependency is apache-arrow."""
    after = _PKG.replace('"globals": "14.0.0"', '"globals": "17.8.0"')
    assert cc.dev_only_change("package.json", _PKG, after) is True


def test_runtime_dependencies_bump_is_shipped() -> None:
    after = _PKG.replace('"apache-arrow": "^21"', '"apache-arrow": "^22"')
    assert cc.dev_only_change("package.json", _PKG, after) is False


@pytest.mark.parametrize("key", ["overrides", "optionalDependencies", "scripts"])
def test_other_top_level_keys_are_shipped(key: str) -> None:
    """`overrides` and `optionalDependencies` both change the installed tree."""
    after = _PKG.replace(
        '"name": "laterite",', f'"name": "laterite",\n  "{key}": {{}},'
    )
    assert cc.dev_only_change("package.json", _PKG, after) is False


def test_version_bump_is_shipped() -> None:
    after = _PKG.replace('"0.10.1"', '"0.10.2"')
    assert cc.dev_only_change("package.json", _PKG, after) is False


# --- failing closed -----------------------------------------------------------


@pytest.mark.parametrize(
    ("before", "after"),
    [
        (None, _PY_BASE),  # added
        (_PY_BASE, None),  # deleted
        (None, None),
    ],
)
def test_a_one_sided_manifest_is_shipped(before: str | None, after: str | None) -> None:
    """The gate cannot compare what isn't there, so it must not claim to have."""
    assert cc.dev_only_change("pyproject.toml", before, after) is False


@pytest.mark.parametrize(
    ("kind", "before", "after"),
    [
        ("pyproject.toml", _PY_BASE, "[project\nbroken = "),
        ("pyproject.toml", "not : valid = toml [[", _PY_BASE),
        ("package.json", _PKG, "{ not json"),
        ("Cargo.toml", _CARGO, "[[[["),
    ],
)
def test_an_unparseable_manifest_is_shipped(kind: str, before: str, after: str) -> None:
    """A manifest the gate cannot read is a manifest it has not checked — the
    same principle it already applies to an unreachable base ref."""
    assert cc.dev_only_change(kind, before, after) is False


# --- the shipped() path layer still holds -------------------------------------


@pytest.mark.parametrize(
    "path",
    [
        "rust-packages/laterite-ags4-core/src/lib.rs",
        "packages/laterite/python/laterite/__init__.py",
        "web/src/App.tsx",
        "pyproject.toml",
    ],
)
def test_shipped_paths(path: str) -> None:
    assert cc.shipped(path) is True


@pytest.mark.parametrize(
    "path",
    [
        "tools/check_changelog.py",
        ".github/workflows/ci.yml",
        "ags-wiki/start-here.md",
        "rust-packages/laterite-ags4-core/tests/roundtrip.rs",
        "rust-packages/laterite-node/README.md",
        "web/docs-site/index.md",
    ],
)
def test_unshipped_paths(path: str) -> None:
    assert cc.shipped(path) is False
