#!/usr/bin/env python3
"""Diff the rendered public API of each publish-set crate against a checked-in snapshot.

`cargo package --list` (tools/check_package_contents.py) answers "which FILES
ship". This answers the harder question: "which API ships". On crates.io that
question is irreversible — a published version's surface is frozen, and the only
remedy for shipping the wrong one is a version bump that every consumer must
follow.

## Why a checked-in snapshot rather than a tool run in isolation

The snapshot puts the whole public surface INTO THE DIFF. Nothing else in this
repo does. Two concrete failures here argue for it:

**A self-consistent revert is invisible to every other gate.** laterite#178 was
branched before #175 and squash-merged after it, so its diff restored two public
functions (`GroupDescriptor::table()`/`view()`) that #175 had deliberately
deleted. The tree compiled, every test passed, the result was internally
consistent, and it sat unnoticed for nine days. A snapshot would have shown
`+pub fn ...::table(...)` in the PR's own diff, on the reviewer's screen.

**Auto traits are public API nobody writes down.** `Send`/`Sync`/`Unpin` are
inferred from private fields. Putting an `Rc` in a private field of a public
struct is a one-line, entirely local-looking change that removes `Send` from a
consumer's type — a major-version break with no signature anywhere to review.
`cargo public-api` renders those impls as lines, so the snapshot carries them
and [`check_auto_traits`] below turns them into a hard failure rather than
something a reviewer has to notice.

## Scope

`--all-features`, not the default feature set. A feature-gated item is still
public API — a consumer can turn the feature on, and breaking it is still a
breaking change. Snapshotting the default set would leave `laterite-ags4-types`'
whole `arrow` surface (`arrow_cols`, `ipc`) unwatched.

`--omit blanket-impls` drops the blanket impls our dependencies project onto our
types (`zerocopy::pointer::invariant::CastableFrom` and friends). Those are
neither ours nor stable across a `cargo update`, and they were two thirds of the
raw output. Auto-trait impls are deliberately KEPT — they are the point.

## What this does NOT check

`impl Trait` in return position. rustdoc records the return as
`-> impl Iterator<Item = &Heading>`; whether that opaque type is `Send` is
invisible here, and it leaks to consumers all the same. That gap is why
[`check_impl_trait_is_asserted`] exists: every `-> impl` in a snapshot must be
named in the owning crate's `tests/auto_traits.rs`, which proves the bound at
compile time. The snapshot finds them; the test asserts them.

This also requires a NIGHTLY toolchain: rustdoc's JSON output is unstable, and
`cargo public-api` reads it. That is a real cost — it is the reason this is a
separate CI job rather than a step in the main `rust` one.

Usage:
    python tools/check_public_api.py           # compare against the snapshots
    python tools/check_public_api.py --write   # regenerate after an intended change
"""

from __future__ import annotations

import argparse
import difflib
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = REPO / "rust-packages"
SNAPSHOTS = REPO / "tools" / "release" / "public-api"

# The publish set is defined ONCE, next to the packaging gate. Restating the ten
# names here would let the two gates cover different sets of crates, and the one
# that silently stopped covering a crate is the one nobody would notice.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_package_contents import PUBLISH_SET  # noqa: E402


def die(msg: str) -> None:
    print(f"check_public_api: {msg}", file=sys.stderr)
    raise SystemExit(2)


def snapshot_path(crate: str) -> Path:
    return SNAPSHOTS / f"{crate}.txt"


def render(crate: str) -> list[str]:
    """The public API of `crate` as `cargo public-api` renders it.

    Run per-crate with `--manifest-path` rather than `-p` from the workspace
    root: the root is a virtual manifest and `cargo public-api` refuses to
    render one.

    No toolchain argument: it selects a nightly itself and offers no flag to
    point it elsewhere. The only requirement is that one is installed.
    """
    proc = subprocess.run(
        [
            "cargo",
            "public-api",
            "--manifest-path",
            str(CRATES / crate / "Cargo.toml"),
            "--all-features",
            "--omit",
            "blanket-impls",
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        die(f"`cargo public-api` failed for {crate}:\n{proc.stderr.strip()}")
    lines = [ln.rstrip() for ln in proc.stdout.splitlines() if ln.strip()]
    # Zero is a bad witness: an empty rendering would make every comparison
    # below pass vacuously, and a crate whose surface really did vanish is a
    # far bigger finding than a diff.
    if not lines:
        die(
            f"{crate}: rendered an EMPTY public API — refusing to treat that as a result"
        )
    return lines


#: `pub struct Foo`, `pub enum Foo<'a>`, `pub struct Foo(u8)`, `pub struct Foo { .. }`.
DECL = re.compile(r"^pub (?:struct|enum|union) (\S+?)(?:\(|\s|$)")
#: `impl<'a> core::marker::Send for path::Foo<'a>` — the trailing path is what matters.
AUTO_IMPL = re.compile(r"^impl.* core::marker::(Send|Sync) for (\S+)")
#: A function returning an opaque type. rustdoc renders the trait path in full,
#: so `-> impl ` is unambiguous. The NAME needs [`_fn_name`], not a regex group:
#: the path carries generics (`dict::Dictionary<'a>::group_codes`) whose own
#: `::` and `(` defeat any pattern simple enough to read.
IMPL_TRAIT = re.compile(r"^pub fn (.+?) -> impl ")


def _fn_name(sig: str) -> str:
    """`a::b::Dictionary<'a>::group_codes(&self)` -> `group_codes`.

    Generic parameters are stripped by bracket depth before anything is split
    on `::`, because they contain both separators: a lifetime turns a path into
    `Type<'a>::method`, and a bound turns it into `f<W: core::io::Write>`. A
    naive split gets `Dictionary` for the first and `Write>` for the second —
    and the first is the silent one, since it collapses every method of a
    generic type into a single plausible-looking name.
    """
    out, depth = [], 0
    for ch in sig:
        if ch == "<":
            depth += 1
        elif ch == ">":
            depth -= 1
        elif depth == 0:
            if ch == "(":
                break
            out.append(ch)
    return "".join(out).rsplit("::", 1)[-1]


def _terminal(path: str) -> str:
    """`a::b::Foo<'x>` -> `Foo`. Re-exports declare a type at a second path.

    `pub use findings::Finding` renders BOTH `pub struct ...::findings::Finding`
    (carrying the impls) and `pub struct ...::Finding` (carrying none). Keying
    coverage on the bare name treats those as the one type they are.
    """
    return path.split("<")[0].rsplit("::", 1)[-1]


def check_auto_traits(crate: str, lines: list[str]) -> list[str]:
    """Every public type must be `Send` and `Sync`.

    Not a style preference: a type that is neither cannot cross a thread in a
    consumer's code, and every surface this repo ships — PyO3, napi-rs, a wasm
    worker — is threaded on the far side. Losing an auto trait is a major
    version break; making it a build failure means it can only happen on
    purpose.
    """
    declared = {_terminal(m.group(1)) for ln in lines if (m := DECL.match(ln))}
    have = {trait: set() for trait in ("Send", "Sync")}
    for ln in lines:
        if m := AUTO_IMPL.match(ln):
            have[m.group(1)].add(_terminal(m.group(2)))
    return [
        f"  {crate}: `{name}` is public but not {trait}"
        for trait in ("Send", "Sync")
        for name in sorted(declared - have[trait])
    ]


#: The crate whose whole purpose is to have no third-party type in its API.
FACADE = "laterite"

#: Path roots a facade signature may mention. Everything else is somebody's
#: crate, and a crate in a signature is a crate whose major version can force
#: ours.
ALLOWED_ROOTS = frozenset({"laterite", "core", "alloc", "std"})

#: A leading path segment: `core::fmt::…`, `serde_json::Value`. Segments AFTER
#: the first are inner modules of whatever the first names, so only roots — the
#: text at a line start or after a delimiter — are checked.
ROOT = re.compile(r"(?:^|[ (<&\[,])([a-z_][a-z0-9_]*)::")


def check_no_third_party(crate: str, lines: list[str]) -> list[str]:
    """The facade's public API must name no crate but its own and the standard library.

    This is the highest-leverage rule in `dec-rust-api-crates-io.md`, and the
    one most easily lost by accident: returning a `serde_json::Value` or taking
    an `encoding_rs::Encoding` is a one-line convenience that permanently binds
    this crate's major version to that dependency's. Every such slip is visible
    in the rendered API, so it can simply be forbidden rather than reviewed for.

    Only the facade is held to this. The engine crates traffic in `arrow`,
    `encoding_rs` and `serde` on purpose — being the layer that does, so this one
    does not, is the entire point of the split.
    """
    if crate != FACADE:
        return []
    bad: dict[str, str] = {}
    for ln in lines:
        for root in ROOT.findall(ln):
            if root not in ALLOWED_ROOTS:
                bad.setdefault(root, ln)
    return [
        f"  {crate}: `{root}` appears in the public API — a third-party type in a "
        f"signature binds this crate's semver to that dependency's:\n      {ln}"
        for root, ln in sorted(bad.items())
    ]


def check_impl_trait_is_asserted(crate: str, lines: list[str]) -> list[str]:
    """Every `-> impl Trait` must be named in the crate's `tests/auto_traits.rs`.

    The snapshot cannot see whether an opaque return type is `Send` — rustdoc
    renders the declared bounds and stops. So the snapshot's job here is only to
    FIND them; a compile-time assertion on a real returned value does the
    proving. This keeps the two in step, so a new `-> impl` cannot be added
    without either an assertion or a deliberate argument against one.
    """
    fns = sorted({_fn_name(m.group(1)) for ln in lines if (m := IMPL_TRAIT.match(ln))})
    if not fns:
        return []
    test = CRATES / crate / "tests" / "auto_traits.rs"
    if not test.exists():
        return [
            f"  {crate}: returns opaque types from {fns} but has no tests/auto_traits.rs"
        ]
    text = test.read_text(encoding="utf-8")
    return [
        f"  {crate}: `{fn}` returns an opaque type that tests/auto_traits.rs never asserts"
        for fn in fns
        if fn not in text
    ]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="regenerate the snapshots")
    args = ap.parse_args()

    SNAPSHOTS.mkdir(parents=True, exist_ok=True)
    problems: list[str] = []
    total = 0

    for crate in PUBLISH_SET:
        lines = render(crate)
        total += len(lines)
        path = snapshot_path(crate)

        if args.write:
            path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        elif not path.exists():
            die(f"{path.relative_to(REPO)} is missing — run with --write to create it")
        else:
            want = path.read_text(encoding="utf-8").splitlines()
            if want != lines:
                diff = difflib.unified_diff(
                    want,
                    lines,
                    fromfile=f"{crate} (snapshot)",
                    tofile=f"{crate} (now)",
                    lineterm="",
                )
                problems.append("\n".join(diff))

        problems += check_auto_traits(crate, lines)
        problems += check_impl_trait_is_asserted(crate, lines)
        problems += check_no_third_party(crate, lines)

    if args.write and not problems:
        print(
            f"wrote {len(PUBLISH_SET)} snapshots to {SNAPSHOTS.relative_to(REPO)} — {total} lines"
        )
        return 0

    if problems:
        # In --write mode the snapshots have already been rewritten, so a
        # problem here is never a diff — only an auto-trait or opaque-return
        # failure, which regenerating does not and must not silence.
        print("check_public_api: the public API changed.\n")
        print("\n".join(problems))
        if not args.write:
            print(
                "\nIf this is intended, run `python tools/check_public_api.py --write` and"
                "\ncommit the snapshot with the change that caused it. Read the diff first:"
                "\ncrates.io freezes a published version's surface, and a removal is a major"
                "\nbump every consumer has to follow."
            )
        return 1

    print(f"check_public_api: OK — {len(PUBLISH_SET)} crates, {total} lines, no drift")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
