#!/usr/bin/env python3
"""Compile each crate README's example against the crate ON crates.io.

`tests/test_crate_readme_doctests.py` wired every publishable crate's README into
`cargo test --workspace` (#278), which answers "does this example compile against
this tree". That is not the reader's question. Their path is

    cargo add laterite-ags4-core

and then the example off the crates.io page — the RELEASED crate plus its
RELEASED dependency graph. The tree wiring cannot see the difference, because
`#[cfg(doctest)] #[doc = include_str!("../README.md")]` compiles inside the
workspace, where every `laterite-*` dependency resolves through a `path =` entry
to the source next door. A re-export or a feature gate that exists only in the
tree compiles there and fails for the reader, and nothing would say so.

So this builds a scratch consumer per crate — a crate with no relationship to
this workspace at all — `cargo add`s the crate FROM THE REGISTRY, drops the
tree's README in beside it with the same three-line doctest wiring, and runs
`cargo test --doc`. **No `path =` anywhere**: that absence is the whole
instrument, and `tests/test_released_crate_readmes.py` asserts it rather than
trusting this file to keep it.

Unpinned, deliberately. `cargo add` with no version is what a reader types, so
the subject is whatever the registry serves today — which makes the released
version a MEASUREMENT, printed beside this tree's for every crate, for the same
reason `gen_doc_outputs.py:_lat()` prints the binary it picked.

## What a red run means

The same thing it means on the wheel and npm legs, and it is not always a defect:
the tree legitimately runs ahead of the last release, so a README documenting
unreleased API fails here until that release is cut. That is why the nightly leg
this drives is out of `notify`'s `needs` — it emails rather than filing an issue
about a state the owner chose. The printed released-vs-tree pair is what tells
the two apart.

## The extern set is derived, not written down

An example that says `use laterite_ags4_parse::parse_str;` needs that crate as a
dependency of the scratch consumer too — and from the registry, like everything
else. A hand-kept map of crate → extra dependencies would be a second statement
of what the examples import, wrong the first time one is edited. So the `use`
roots are read out of the rust fences: a root naming a directory in
`rust-packages/` is one of ours (`laterite_ags4_parse` → `laterite-ags4-parse`),
anything else is taken as written (`serde_json` is the crates.io name).

Only `use` roots — NOT every `ident::` in the example. `use laterite::ags4;`
followed by `ags4::read(…)` would otherwise send `cargo add ags4` at the
registry, and adding a stranger's crate because an example imported a module is
a worse failure than the one this prevents.

Usage:
    python tools/check_released_crate_readmes.py                 # every discovered crate
    python tools/check_released_crate_readmes.py --list          # what it would test, no network
    python tools/check_released_crate_readmes.py --crate laterite-ags4-parse
    python tools/check_released_crate_readmes.py --work-dir out/released-readmes
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST = REPO / "rust-packages"
WORKSPACE = RUST / "Cargo.toml"

#: A ```rust fence in a README. `no_run` and friends ride in the tag and are
#: preserved by copying the README verbatim — rustdoc reads them, not this file.
RUST_FENCE = re.compile(r"^```rust[^\n]*\n(?P<body>.*?)^```$", re.M | re.S)

#: The root of a `use` path — the crate the example needs on its dependency line.
USE_ROOT = re.compile(r"^\s*use\s+([a-z][a-z0-9_]*)\s*(?:::|;)", re.M)

#: Any path root, `use` line or not — `laterite_ags4_parse::ParseError` in a
#: return type needs the dependency just as much as an import does. Admitted only
#: when it names one of OURS (see `example_deps`), which is what keeps it from
#: fetching a stranger.
PATH_ROOT = re.compile(r"(?<![\w:.])([a-z][a-z0-9_]*)::")

#: Roots that name no crate. `crate`/`self`/`super` are relative paths, and the
#: three implicit ones are already there.
NOT_A_DEPENDENCY = frozenset({"std", "core", "alloc", "crate", "self", "super"})

#: Fallback edition for the scratch consumer. Only reached if a manifest omits
#: `edition`; the real value is read per-crate, because rustdoc compiles a README
#: doctest under the CONSUMER's edition and a leg compiling under different rules
#: from the crate's own doctest would report phantom breakage.
DEFAULT_EDITION = "2024"

#: `cargo add` saying "this crate is not on crates.io", which is a state of
#: release prep and NOT a broken README. `publish_crates.py` holds crates back
#: deliberately (its `DEFERRED` set), and a crate prepared for the registry but
#: not yet uploaded would otherwise fail here identically to one whose published
#: example is broken. Same distinction `check_package_contents.NOT_YET_PUBLISHABLE`
#: draws, read off the same cargo message rather than off a second list.
NOT_ON_REGISTRY = "no matching package named"


def die(msg: str) -> None:
    print(f"check_released_crate_readmes: {msg}", file=sys.stderr)
    raise SystemExit(2)


@dataclass(frozen=True)
class Subject:
    """One crate README, and what a scratch consumer of it needs."""

    crate: str
    readme: Path
    #: This tree's version — the other half of the released-vs-tree pair.
    tree_version: str
    #: Registry names to `cargo add`, the crate itself first.
    deps: tuple[str, ...]
    #: The crate's own edition, which rustdoc will compile its doctest under.
    edition: str


def _manifest(crate: str) -> dict:
    return tomllib.loads((RUST / crate / "Cargo.toml").read_text(encoding="utf-8"))


def _tree_version(crate: str) -> str:
    """`crate`'s version, resolving workspace inheritance.

    Per-crate rather than one number for the tree: the engine crates inherit the
    workspace version in lockstep, but `laterite` — the facade — carries its own
    0.1.x, and printing the workspace number beside it would misreport the pair
    this whole tool exists to state.
    """
    version = _manifest(crate)["package"].get("version")
    if isinstance(version, str):
        return version
    if isinstance(version, dict) and version.get("workspace") is True:
        ws = tomllib.loads(WORKSPACE.read_text(encoding="utf-8"))
        return str(ws["workspace"]["package"]["version"])
    die(f"{crate}: cannot determine its version from the manifest")
    raise AssertionError("unreachable")  # die() exits


def workspace_crate_names() -> set[str]:
    """Every crate directory in `rust-packages/` — the "is this one of ours" test.

    Directories, not the workspace `members` list: a crate excluded from the
    workspace is still ours, and would still be the wrong thing to fetch off the
    registry under its underscored lib name.
    """
    return {p.parent.name for p in RUST.glob("*/Cargo.toml")}


def registry_name(root: str, ours: set[str]) -> str:
    """The crates.io name for a `use` root.

    Rust identifiers cannot carry a hyphen, so OUR crates arrive underscored
    (`laterite_ags4_parse`) and must be un-underscored to be fetched. Third-party
    names must NOT be: `serde_json` is spelled with the underscore on crates.io,
    and rewriting it asks for a crate that does not exist.
    """
    hyphenated = root.replace("_", "-")
    return hyphenated if hyphenated in ours else root


def example_deps(readme_text: str, crate: str, ours: set[str]) -> tuple[str, ...]:
    """`crate` first, then every other crate its README's rust fences need.

    TWO PASSES, and the asymmetry between them is the whole safety property.

    A `use` root is taken as written, whatever it names: an example that imports
    a third-party crate needs it, and `serde_json` can only be discovered this
    way. Any OTHER path root — `laterite_ags4_parse::ParseError` in a return
    type, with no matching import — is admitted **only if it resolves to one of
    ours**. Bare-root matching cannot be trusted in general, because
    `use laterite::ags4;` followed by `ags4::read(…)` would otherwise send
    `cargo add ags4` at the registry; requiring the second pass to hit a crate
    directory in `rust-packages/` closes the gap for our own crates without ever
    reaching for a stranger's.
    """
    deps = [crate]

    def admit(name: str) -> None:
        if name not in deps:
            deps.append(name)

    for fence in RUST_FENCE.finditer(readme_text):
        body = fence.group("body")
        for root in USE_ROOT.findall(body):
            if root not in NOT_A_DEPENDENCY:
                admit(registry_name(root, ours))
        for root in PATH_ROOT.findall(body):
            if root not in NOT_A_DEPENDENCY and root.replace("_", "-") in ours:
                admit(registry_name(root, ours))
    return tuple(deps)


def subjects() -> list[Subject]:
    """The crates a reader can `cargo add` and then copy an example from.

    Discovered, so a new crate joins by having a README fence and a
    publishable manifest — not by being added to a list here. `publish = false`
    is the exclusion that matters: `laterite-ags4-wasm` carries examples too, but
    it reaches its readers through npm, and this asks the crates.io question.
    """
    out: list[Subject] = []
    ours = workspace_crate_names()
    for readme in sorted(RUST.glob("*/README.md")):
        crate = readme.parent.name
        # A rust-fenced README in a directory with no manifest is not a crate at
        # all. The two scans disagree about what a subdirectory is, and the
        # disagreement must not come out as a bare traceback from `_manifest`.
        if crate not in ours:
            continue
        text = readme.read_text(encoding="utf-8")
        if not RUST_FENCE.search(text):
            continue
        pkg = _manifest(crate)["package"]
        if pkg.get("publish") is False:
            continue
        out.append(
            Subject(
                crate=crate,
                readme=readme,
                tree_version=_tree_version(crate),
                deps=example_deps(text, crate, ours),
                edition=str(pkg.get("edition", DEFAULT_EDITION)),
            )
        )
    return out


def scaffold(subject: Subject, into: Path) -> Path:
    """Write the scratch consumer for `subject` and return its directory.

    `[workspace]` is not decoration: an empty table declares this a workspace
    root, so the crate is its own world even if the directory it lands in sits
    under one. Without it a run inside a checkout is silently absorbed by the
    surrounding workspace — which is where the path dependencies live, and would
    quietly answer the question this tool exists to stop answering.
    """
    root = into / subject.crate
    (root / "src").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(subject.readme, root / "README.md")
    (root / "Cargo.toml").write_text(
        "# Generated by tools/check_released_crate_readmes.py — a consumer of the\n"
        "# PUBLISHED crate. Dependencies are added by `cargo add`, from the registry.\n"
        "[workspace]\n\n"
        "[package]\n"
        f'name = "readme-{subject.crate}"\n'
        'version = "0.0.0"\n'
        # The SUBJECT crate's edition, not a constant: rustdoc compiles a README
        # doctest under the CONSUMER's edition, so a fixed one here would compile
        # the example under different rules from the crate's own in-tree doctest
        # the moment a crate moves edition — and report the difference as the
        # released crate being broken.
        f'edition = "{subject.edition}"\n'
        "publish = false\n",
        encoding="utf-8",
    )
    # The tree's own wiring, verbatim — the README is the single copy of the
    # example on both sides of the registry, so the only difference between this
    # doctest and the in-workspace one is which crate it links against.
    (root / "src" / "lib.rs").write_text(
        "#[cfg(doctest)]\n"
        '#[doc = include_str!("../README.md")]\n'
        "mod readme_doctests {}\n",
        encoding="utf-8",
    )
    return root


def _run(
    argv: list[str], cwd: Path, env_target: Path, capture: bool = False
) -> subprocess.CompletedProcess:
    """One cargo invocation, with the shared target dir.

    Shared across all ten consumers on purpose: they overlap almost entirely in
    their dependency graphs, and ten cold builds of the same graph is the
    difference between a nightly leg that fits in its timeout and one that does
    not.
    """
    proc = subprocess.run(
        argv,
        cwd=cwd,
        env={**os.environ, "CARGO_TARGET_DIR": str(env_target)},
        text=True,
        capture_output=capture,
    )
    if capture:
        print(proc.stdout, end="")
        print(proc.stderr, end="", file=sys.stderr)
    return proc


def add_argv(subject: Subject) -> list[str]:
    """The `cargo add` this consumer runs — every dependency FROM THE REGISTRY.

    Its own function so a test can assert what it does not contain. `--path` here
    is the one edit that would silently turn this leg back into a second copy of
    the workspace's doctest run, and no compiler would ever object to it.
    """
    return ["cargo", "add", *subject.deps]


def resolved_version(root: Path, crate: str) -> str:
    """What the registry actually served, read out of the scratch lockfile."""
    lock = root / "Cargo.lock"
    if not lock.is_file():
        return "?"
    for pkg in tomllib.loads(lock.read_text(encoding="utf-8")).get("package", []):
        if pkg.get("name") == crate:
            return str(pkg.get("version", "?"))
    return "?"


def drift(released: str, tree: str) -> str:
    """How the two versions relate, stated only as far as they support.

    This line is, by the nightly leg's own design, the only thing separating a
    real defect from ordinary tree-ahead drift — so it must not assert a
    DIRECTION it has not established. An unread version says so; a tree that is
    BEHIND the registry (a release cut from elsewhere) is its own sentence, not
    the same one.
    """
    if "?" in (released, tree):
        return "  VERSION UNREAD — cannot say which side is ahead"
    if released == tree:
        return "  same version — any failure below is a real defect"

    def key(v: str) -> tuple:
        return tuple(int(p) if p.isdigit() else p for p in re.split(r"[.\-+]", v))

    try:
        ahead = key(tree) > key(released)
    except TypeError:  # a prerelease against a plain triple; don't guess
        return "  VERSIONS DIFFER — compare them by hand"
    if ahead:
        return "  TREE IS AHEAD — unreleased API in the README fails until a release"
    return "  TREE IS BEHIND the registry — this checkout predates the release"


def check(subject: Subject, work: Path, target: Path) -> bool | None:
    """Compile (and run) `subject`'s README example against the released crate.

    True passed, False failed, None not on the registry yet (see `NOT_ON_REGISTRY`).
    """
    print(f"\n=== {subject.crate}", flush=True)
    root = scaffold(subject, work)

    argv = add_argv(subject)
    added = _run(argv, root, target, capture=True)
    if added.returncode != 0:
        if NOT_ON_REGISTRY in added.stderr:
            print("  SKIP — not on crates.io yet, so there is no release to ask")
            return None
        print(f"  FAIL {' '.join(argv)} — see the output above")
        return False

    released = resolved_version(root, subject.crate)
    print(f"  released crate : {released}")
    print(f"  this tree      : {subject.tree_version}")
    print(drift(released, subject.tree_version))
    print(f"  dependencies   : {' '.join(subject.deps)}", flush=True)

    return _run(["cargo", "test", "--doc"], root, target).returncode == 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--crate",
        action="append",
        metavar="NAME",
        help="limit to this crate (repeatable); default is every discovered one",
    )
    ap.add_argument(
        "--list",
        action="store_true",
        help="print what would be tested (crate, tree version, dependencies) and stop",
    )
    ap.add_argument(
        "--work-dir",
        type=Path,
        metavar="PATH",
        help="where the scratch consumers go (default: a temp dir, removed after)",
    )
    args = ap.parse_args()

    found = subjects()
    if not found:
        die("no publishable crate README carries a `rust` fence — the layout moved")
    if args.crate:
        wanted = set(args.crate)
        if unknown := sorted(wanted - {s.crate for s in found}):
            die(f"not a crate with a publishable README example: {unknown}")
        found = [s for s in found if s.crate in wanted]

    if args.list:
        for s in found:
            print(f"{s.crate:<26} tree {s.tree_version:<8} needs {' '.join(s.deps)}")
        return 0

    if shutil.which("cargo") is None:
        die("`cargo` is not on PATH — this compiles against the registry")

    tmp = None
    if args.work_dir:
        work = args.work_dir
        work.mkdir(parents=True, exist_ok=True)
    else:
        tmp = tempfile.TemporaryDirectory(prefix="released-readmes-")
        work = Path(tmp.name)
    target = work / ".target"

    failed: list[str] = []
    skipped: list[str] = []
    try:
        for s in found:
            verdict = check(s, work, target)
            if verdict is None:
                skipped.append(s.crate)
            elif not verdict:
                failed.append(s.crate)
    finally:
        if tmp is not None:
            tmp.cleanup()

    asked = len(found) - len(skipped)
    print(
        f"\n{asked - len(failed)}/{asked} README example(s) compile "
        "against the crate on crates.io"
    )
    # Never silently: a leg that covered less than it appears to is the failure
    # this repo has been bitten by before (#207).
    if skipped:
        print("not on the registry yet, so unasked: " + ", ".join(skipped))
    if failed:
        print("FAILED: " + ", ".join(failed), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
