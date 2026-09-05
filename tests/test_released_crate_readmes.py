"""The registry leg must not be able to answer the workspace's question.

`tools/check_released_crate_readmes.py` exists because the in-tree doctests
(`tests/test_crate_readme_doctests.py`) compile every crate README against the
source next door, where each `laterite-*` dependency resolves through a `path =`
entry. The reader gets the released crate and its released dependency graph
instead, and the two can disagree.

The whole instrument is the ABSENCE of a path dependency in the scratch consumer
it generates. That is one line of a generated manifest and no compiler would ever
complain about its return — so it is asserted here, along with the discovery and
name-mapping rules that decide what gets fetched, because getting those wrong
sends `cargo add` after a stranger's crate.

Network-free by construction: everything below runs on the generator's pure
functions and its scaffold output. Compiling against crates.io is the nightly
leg's job (`docs-vs-released-crates` in `.github/workflows/nightly.yml`).
"""

from __future__ import annotations

import tomllib
from typing import TYPE_CHECKING

from _tools import load_tool

if TYPE_CHECKING:
    from pathlib import Path

crr = load_tool("check_released_crate_readmes")


def test_discovery_finds_the_publishable_readme_examples() -> None:
    """Zero is a bad witness, and so is a set that quietly shrank."""
    found = crr.subjects()
    assert len(found) >= 10, (
        f"only {len(found)} publishable crate README(s) with a `rust` fence — "
        "the layout moved, or a crate stopped being published"
    )
    assert len({s.crate for s in found}) == len(found), "a crate discovered twice"


def test_discovery_excludes_what_a_reader_cannot_cargo_add() -> None:
    """`publish = false` is the line between the two questions.

    `laterite-ags4-wasm` reaches its readers through npm and says `publish =
    false` for crates.io. Fetching it here would fail at `cargo add` with a
    confusing error about a crate that was never meant to be there.
    """
    for s in crr.subjects():
        manifest = tomllib.loads(
            (crr.RUST / s.crate / "Cargo.toml").read_text(encoding="utf-8")
        )
        assert manifest["package"].get("publish") is not False, (
            f"{s.crate} says `publish = false` but was picked up as a crates.io subject"
        )


def test_every_subject_states_a_version_to_print_against() -> None:
    """The released-vs-tree pair is the leg's only way to tell defect from drift."""
    for s in crr.subjects():
        assert s.tree_version and s.tree_version[0].isdigit(), (
            f"{s.crate}: tree version {s.tree_version!r} is not a version"
        )


def test_our_crates_are_un_underscored_and_strangers_are_not() -> None:
    """`serde_json` is spelled with the underscore on crates.io; ours are not.

    Falsify by hyphenating unconditionally — `cargo add serde-json` then asks the
    registry for a crate that does not exist.
    """
    ours = crr.workspace_crate_names()
    assert "laterite-ags4-parse" in ours, "the workspace scan found nothing"
    assert crr.registry_name("laterite_ags4_parse", ours) == "laterite-ags4-parse"
    assert crr.registry_name("serde_json", ours) == "serde_json"


def test_only_use_roots_become_dependencies() -> None:
    """A module imported from a crate must not be fetched AS a crate.

    The facade README does `use laterite::ags4;` and then calls `ags4::read(…)`.
    A rule that took every `ident::` in the example would send `cargo add ags4`
    at the registry — adding a stranger's crate because an example named a
    module.
    """
    ours = crr.workspace_crate_names()
    readme = """
# x

```rust
use laterite::ags4;
use std::path::Path;

fn main() {
    let _ = ags4::read("x.ags");
    let _ = other::thing();
}
```

```bash
use not_a_dependency::at_all;
```
"""
    deps = crr.example_deps(readme, "laterite", ours)
    assert deps == ("laterite",), deps


def test_dependencies_are_derived_from_the_fences() -> None:
    """The extra crates an example imports ride along, once each, crate first."""
    ours = crr.workspace_crate_names()
    readme = """
```rust
use laterite_ags4_merge::{merge_parsed, MergeOpts};
use laterite_ags4_parse::parse_str;
use laterite_ags4_parse::ParseError;
use serde_json::Value;
```
"""
    deps = crr.example_deps(readme, "laterite-ags4-merge", ours)
    assert deps == ("laterite-ags4-merge", "laterite-ags4-parse", "serde_json"), deps


def _workspace_dependency_names() -> set[str]:
    """Every crate this workspace already depends on, by its registry name.

    The evidence that a derived name is fetchable: something in this repo
    resolves it from crates.io today.
    """
    names: set[str] = set()
    tables = ("dependencies", "dev-dependencies", "build-dependencies")
    # The workspace ROOT too, one level above the per-crate glob: a third-party
    # crate declared only in `[workspace.dependencies]` is invisible otherwise,
    # and this set is what decides whether a derived name is legitimate.
    for manifest in [*crr.RUST.glob("*/Cargo.toml"), crr.WORKSPACE]:
        parsed = tomllib.loads(manifest.read_text(encoding="utf-8"))
        for table in tables:
            names |= set(parsed.get(table, {}))
            names |= set(parsed.get("workspace", {}).get(table, {}))
    return names


def test_the_real_readmes_name_only_fetchable_crates() -> None:
    """Every derived dependency is one this repo already resolves.

    A root that is neither ours nor a crate the workspace depends on is a module
    mistaken for a crate, or a typo — and would surface only as a `cargo add`
    failure on the nightly leg, hours later and attributed to the wrong thing.
    Falsify by taking every `ident::` instead of `use` roots: the facade's
    `ags4::read(…)` then arrives here as a dependency.
    """
    fetchable = crr.workspace_crate_names() | _workspace_dependency_names()
    assert "serde_json" in fetchable, "the workspace dependency scan found nothing"
    for s in crr.subjects():
        assert s.deps[0] == s.crate, f"{s.crate}: the crate itself must come first"
        for dep in s.deps:
            assert dep in fetchable, (
                f"{s.crate}: {dep!r} is neither one of ours nor a crate this "
                "workspace depends on — is it a module, not a crate?"
            )


def test_every_dependency_comes_from_the_registry(tmp_path: Path) -> None:
    """THE assertion, and it has to be made where a path could actually enter.

    Not in the scaffolded manifest — that has no dependency table at all, so
    "no path here" is true by construction and would stay true through the edit
    that matters. Dependencies enter through `cargo add`, so the argv is the
    subject: `_run(["cargo", "add", "--path", …])` is the one change that turns
    this leg back into a second copy of the workspace's doctest run with every
    other gate still green, and no compiler would ever object to it.

    Falsify by adding `--path` (or a `path =` line to the manifest): both halves
    below go red.
    """
    subject = crr.subjects()[0]
    argv = crr.add_argv(subject)
    assert argv[:2] == ["cargo", "add"], argv
    assert not [a for a in argv if "path" in a], (
        f"the scratch consumer for {subject.crate} would add a path dependency: {argv}"
    )
    assert list(argv[2:]) == list(subject.deps), (
        "every dependency must be a bare registry name — anything else is a "
        f"channel this leg is not supposed to have: {argv}"
    )

    parsed = tomllib.loads((crr.scaffold(subject, tmp_path) / "Cargo.toml").read_text())
    for table in ("dependencies", "dev-dependencies", "build-dependencies"):
        for name, spec in parsed.get(table, {}).items():
            assert not (isinstance(spec, dict) and "path" in spec), (
                f"{name} in [{table}] resolves through a path, not the registry"
            )
    assert parsed["package"]["publish"] is False
    assert "workspace" in parsed, (
        "without an empty `[workspace]` table the consumer is absorbed by any "
        "workspace above it — which is where the path dependencies are"
    )


def test_the_scratch_consumer_carries_the_readme_and_the_wiring(
    tmp_path: Path,
) -> None:
    """The README is copied verbatim, so `no_run` and friends survive."""
    picked = [s for s in crr.subjects() if s.crate == "laterite-transport"]
    assert picked, (
        "laterite-transport is no longer a subject — it carries the `no_run` "
        "fence this case is about, so check whether it lost its README fence or "
        "stopped being published before repointing this test"
    )
    subject = picked[0]
    root = crr.scaffold(subject, tmp_path)
    assert (root / "README.md").read_text(encoding="utf-8") == subject.readme.read_text(
        encoding="utf-8"
    )
    lib = (root / "src" / "lib.rs").read_text(encoding="utf-8")
    assert "#[cfg(doctest)]" in lib
    assert 'include_str!("../README.md")' in lib


def test_scaffolding_twice_is_idempotent(tmp_path: Path) -> None:
    """A re-run with `--work-dir` must not fail on the directory it left behind."""
    subject = crr.subjects()[0]
    first = crr.scaffold(subject, tmp_path)
    second = crr.scaffold(subject, tmp_path)
    assert first == second
    assert (second / "src" / "lib.rs").is_file()


def test_a_fully_qualified_sibling_is_still_a_dependency() -> None:
    """An example can name our crate without importing it, and still need it.

    `laterite-ags4-diff`'s fence writes `Result<(), laterite_ags4_parse::ParseError>`
    and `laterite-ags4-parse`'s calls `laterite_ags4_parse::parse_str` outright.
    Both are safe today for incidental reasons; a `use`-roots-only rule would
    miss the first edit that removed the matching import, and the scratch
    consumer would fail with "use of undeclared crate or module" attributed to
    the released crate.

    The second pass is narrow ON PURPOSE — it admits a bare root only if it names
    a crate directory in `rust-packages/`, so `ags4::read(…)` below stays out.
    """
    ours = crr.workspace_crate_names()
    readme = """
```rust
use laterite::ags4;

fn main() -> Result<(), laterite_ags4_parse::ParseError> {
    let _ = ags4::read("x.ags");
    let _ = other_crate::thing();
    Ok(())
}
```
"""
    deps = crr.example_deps(readme, "laterite", ours)
    assert deps == ("laterite", "laterite-ags4-parse"), deps


def test_the_consumer_takes_the_crate_s_own_edition() -> None:
    """rustdoc compiles a README doctest under the CONSUMER's edition.

    A constant here would compile the example under different rules from the
    crate's in-tree doctest the moment a crate moves edition, and report the
    difference as the released crate being broken. Falsify by hardcoding one:
    this goes red as soon as any subject declares another.
    """
    for s in crr.subjects():
        declared = tomllib.loads(
            (crr.RUST / s.crate / "Cargo.toml").read_text(encoding="utf-8")
        )["package"].get("edition", crr.DEFAULT_EDITION)
        assert s.edition == str(declared), (
            f"{s.crate}: consumer edition {s.edition} != the crate's {declared}"
        )


def test_drift_never_asserts_a_direction_it_has_not_established() -> None:
    """This line is the leg's only defect-vs-drift signal, so it must not guess.

    An unread version is not evidence of anything, and a tree BEHIND the registry
    is a different situation from one ahead of it — reporting either as "TREE IS
    AHEAD" would send a reader looking for unreleased API that isn't there.
    """
    assert "real defect" in crr.drift("0.9.0", "0.9.0")
    assert "AHEAD" in crr.drift("0.9.0", "0.10.0")
    assert "BEHIND" in crr.drift("0.10.0", "0.9.0")
    for unread in (crr.drift("?", "0.9.0"), crr.drift("0.9.0", "?")):
        assert "UNREAD" in unread
        assert "AHEAD" not in unread


def test_tree_versions_agree_with_the_publisher() -> None:
    """Two tools now resolve a crate's version; they must not drift apart.

    `publish_crates.py:crate_version` decides what gets uploaded and this decides
    what gets printed beside the release. The subtle part is shared — workspace
    inheritance in lockstep, except the facade on its own 0.1.x clock — and per-
    crate versioning is a stated goal, so the layout WILL move. This is what
    fails when only one of them is updated.
    """
    pc = load_tool("publish_crates")
    for s in crr.subjects():
        assert s.tree_version == pc.crate_version(s.crate), (
            f"{s.crate}: this tool says {s.tree_version}, publish_crates says "
            f"{pc.crate_version(s.crate)}"
        )


def test_resolved_version_reads_the_scratch_lockfile(tmp_path: Path) -> None:
    """The released half of the printed pair comes from what cargo resolved."""
    (tmp_path / "Cargo.lock").write_text(
        '[[package]]\nname = "laterite-ags4-parse"\nversion = "0.9.0"\n',
        encoding="utf-8",
    )
    assert crr.resolved_version(tmp_path, "laterite-ags4-parse") == "0.9.0"
    assert crr.resolved_version(tmp_path, "absent") == "?"
    assert crr.resolved_version(tmp_path / "nowhere", "laterite-ags4-parse") == "?"
