"""A Rust example on a crates.io page must be compiled by something.

Ten crate READMEs carried a `rust` fence each. Every one shipped to crates.io,
where a published version's README is FROZEN — `tools/check_doc_refs.py` already
treats those pages as a strict special case for exactly that reason, checking
their links with no repo-root fallback because "the person deciding whether to
`cargo add` your crate" cannot see this repo. Their CODE was checked by nothing,
and three of the ten did not compile:

  * `laterite-ags4-parse` iterated `parsed.groups` — a `BTreeMap` — as though it
    were a sequence, so `group.code` was a field access on a `(&String, &_)` tuple;
  * `laterite-ags4-core` called `read_ags4_bytes(&bytes, opts)`, which takes ONE
    argument (the two-argument form is `read_ags4_bytes_with`);
  * `laterite-transport` called `lock(src, dest, pw, level)`, which takes five —
    the scrypt `log_n` was missing.

The fix needed no new CI. `cargo test --workspace` already runs doctests, so a
three-line module per crate points rustdoc at the README and the existing gate
compiles it:

    #[cfg(doctest)]
    #[doc = include_str!("../README.md")]
    mod readme_doctests {}

`cfg(doctest)` is what makes this free of side effects: the module exists only
while rustdoc collects doctests, so it is absent from a normal build AND from the
rendered docs.rs page. The crate's own `//!` docs — 82 lines on the facade — are
untouched, and the README stays the single copy of its example.

WHAT THIS GATE ADDS. The doctests catch drift in the ten crates that have the
module today. They cannot notice an ELEVENTH crate that arrives with a README
fence and no module — that crate's example would be unchecked while looking, from
the outside, exactly like its nine gated siblings. So this asserts the wiring
itself, by discovery rather than by a list.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
RUST = REPO / "rust-packages"

#: The wiring, matched loosely enough to survive reformatting but strictly enough
#: that a plain `#[doc = include_str!]` (which WOULD duplicate the crate docs on
#: docs.rs) does not satisfy it.
_WIRED = re.compile(
    r"#\[cfg\(doctest\)\]\s*#\[doc\s*=\s*include_str!\(\s*\"\.\./README\.md\"\s*\)\]",
    re.S,
)
_RUST_FENCE = re.compile(r"^```rust([^\n]*)$", re.M)

#: `cargo test --workspace` in ci.yml excludes these four. A fence in one of them
#: would be gated by nothing while looking gated, which is worse than an ungated
#: fence nobody believes in — so they are a hard error, not a skip.
_UNTESTED = {
    "laterite-ags4-wasm",
    "laterite-ags4-tokenizer-wasm",
    "laterite-py",
    "laterite-node",
}


def _crates_with_rust_fences() -> list[tuple[str, Path, list[str]]]:
    out = []
    for readme in sorted(RUST.glob("*/README.md")):
        tags = _RUST_FENCE.findall(readme.read_text(encoding="utf-8"))
        if tags:
            out.append((readme.parent.name, readme.parent, tags))
    return out


def test_the_scan_finds_the_crates() -> None:
    """Zero is a bad witness: an empty scan makes every case below vacuous."""
    found = _crates_with_rust_fences()
    assert len(found) >= 10, (
        f"only {len(found)} crate README(s) with a `rust` fence — "
        "the layout moved, or the fences were removed"
    )


def test_every_readme_rust_fence_is_wired_as_a_doctest() -> None:
    """Falsify by adding a `rust` fence to a crate README with no module.

    Or by deleting the module from any crate that has one.
    """
    missing = [
        name
        for name, crate, _ in _crates_with_rust_fences()
        if not _WIRED.search((crate / "src" / "lib.rs").read_text(encoding="utf-8"))
    ]
    assert not missing, (
        "these crate READMEs carry a `rust` fence that nothing compiles:\n  "
        + "\n  ".join(sorted(missing))
        + "\n\nAdd to the crate's src/lib.rs:\n\n"
        '    #[cfg(doctest)]\n    #[doc = include_str!("../README.md")]\n'
        "    mod readme_doctests {}\n"
    )


def test_no_fence_opts_out_of_compilation() -> None:
    """`ignore` would leave the example on crates.io and check nothing.

    `no_run` is fine and two crates use it — the example touches the filesystem,
    so it is compiled but not executed, and compiling is what catches the drift
    this exists for. `ignore` compiles nothing at all.
    """
    opted_out = [
        f"{name}: ```rust{tag}"
        for name, _, tags in _crates_with_rust_fences()
        for tag in tags
        if "ignore" in tag or "compile_fail" in tag
    ]
    assert not opted_out, (
        "a crates.io example opts out of being compiled:\n  " + "\n  ".join(opted_out)
    )


def test_every_wired_crate_is_inside_the_ci_doctest_run() -> None:
    """A gated-looking fence in an excluded crate would be checked by nothing.

    `cargo test --workspace` (ci.yml) excludes four crates. None of them has a
    README fence today; this is what says so out loud, so a fence added to one is
    a failure here rather than a silent hole.
    """
    inside = [name for name, _, _ in _crates_with_rust_fences()]
    stranded = sorted(set(inside) & _UNTESTED)
    assert not stranded, (
        "these crates have a README `rust` fence but are EXCLUDED from "
        "`cargo test --workspace`, so nothing compiles it:\n  "
        + "\n  ".join(stranded)
        + "\n\nEither drop the fence or bring the crate into the CI doctest run."
    )


def test_wired_crates_are_real_workspace_members() -> None:
    """Guards the path assumption: `../README.md` must resolve from `src/lib.rs`."""
    for name, crate, _ in _crates_with_rust_fences():
        manifest = crate / "Cargo.toml"
        assert manifest.is_file(), f"{name} has a README but no Cargo.toml"
        pkg = tomllib.loads(manifest.read_text(encoding="utf-8")).get("package", {})
        assert pkg.get("name") == name, (
            f"{crate.name}/Cargo.toml declares package "
            f"{pkg.get('name')!r} — the include_str! path assumes they match"
        )
