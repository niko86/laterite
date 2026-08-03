"""The release advisor recommends the bump this project's policy documents.

`gen_changelog.py --advise` is what RELEASING.md sends you to before stamping a
version, so a wrong answer here becomes a wrong number on PyPI, npm and — for
the engine tier — crates.io, which is append-only and cannot re-cut a version.

Two defects motivated these tests, and both had shipped:

  * The pre-1.0 rule was applied in one place and not the other. `_bump()`
    refused MAJOR at `0.x`, but the advisor's *other* branch escalated any
    non-empty `added` to MINOR regardless of version — contradicting the policy
    paragraph directly above it (RELEASING.md: "while at `0.x` … features and
    fixes bump PATCH"). A docs page in `added` would have cut a minor, which
    pre-1.0 *means* "something broke".

  * "Breaking" was detected by searching the entry prose for `\\bbreaking\\b`.
    That cannot tell a marker from a sentence denying one: "a non-breaking
    change" matched (the hyphen is a word boundary), as did "this is not a
    breaking change". Every historical hit was a true positive only because the
    house style happened to be disciplined.

So compatibility is now DECLARED (`"breaking": true`) and the prose marker is a
cross-check. The tests below pin both halves — the version arithmetic and the
disagreement gate — and the last one pins the advisor to the policy text it
implements, so the two cannot drift apart silently.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load_gen_changelog():
    """Import `tools/gen_changelog.py` as a module — `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "gen_changelog", REPO / "tools" / "gen_changelog.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["gen_changelog"] = mod
    spec.loader.exec_module(mod)
    return mod


gc = _load_gen_changelog()


def _data(current: str, **block) -> dict:
    """A minimal changelog shaped like the real one: one shipped release (which
    is what fixes the era) plus an [Unreleased] block built from kwargs."""
    unreleased = {c: block.get(c, []) for c in gc.CATEGORIES}
    return {
        "repo": "niko86/laterite",
        "unreleased": unreleased,
        "releases": [{"version": current, "date": "2026-01-01"}],
    }


def _e(text: str, breaking: bool = False) -> dict:
    entry: dict = {"text": text}
    if breaking:
        entry["breaking"] = True
    return entry


# --- the era rule -----------------------------------------------------------


@pytest.mark.parametrize(
    ("block", "expected"),
    [
        ({"added": [_e("A new CLI flag.")]}, ("patch", "0.10.1")),
        ({"fixed": [_e("A bug fix.")]}, ("patch", "0.10.1")),
        ({"changed": [_e("A visible but compatible tweak.")]}, ("patch", "0.10.1")),
        (
            {"changed": [_e("**Breaking:** callers must pass a flag.", True)]},
            ("minor", "0.11.0"),
        ),
    ],
)
def test_pre_1_0_only_a_breaking_change_leaves_patch(block, expected):
    """Pre-1.0, MINOR is reserved to *mean* "something broke" — so a feature, a
    fix and a compatible change all land on PATCH, and only a declared break
    escalates. This is the defect that would have cut 0.11.0 for a docs page."""
    part, version, _ = gc.advise(_data("0.10.0", **block))
    assert (part, version) == expected


@pytest.mark.parametrize(
    ("block", "expected"),
    [
        ({"fixed": [_e("A bug fix.")]}, ("patch", "1.2.4")),
        ({"added": [_e("A new CLI flag.")]}, ("minor", "1.3.0")),
        (
            {"changed": [_e("**Breaking:** the default changed.", True)]},
            ("major", "2.0.0"),
        ),
    ],
)
def test_post_1_0_follows_standard_semver(block, expected):
    """Past 1.0 the pre-1.0 convention stops applying and additive goes back to
    MINOR, breaking to MAJOR. Pinned so the era switch is real rather than the
    advisor having simply been hard-wired to the project's current era."""
    part, version, _ = gc.advise(_data("1.2.3", **block))
    assert (part, version) == expected


# --- the signal itself ------------------------------------------------------


@pytest.mark.parametrize(
    "text",
    [
        "A non-breaking change: existing callers are unaffected.",
        "This is not a breaking change.",
        "Avoids breaking downstream consumers.",
        "The rename is source-compatible.",
    ],
)
def test_prose_about_compatibility_does_not_escalate_the_bump(text):
    """The regression that motivated the flag. Every string here was counted as
    a breaking change by the old `\\bbreaking\\b` search — including the two that
    say the opposite — which at 0.x is the difference between a patch and a
    minor."""
    part, _, _ = gc.advise(_data("0.10.0", changed=[_e(text)]))
    assert part == "patch"


def test_a_declared_break_is_counted_regardless_of_wording():
    """The flag is the signal, so an entry whose marker sits mid-sentence (the
    0.6.0 house style) counts exactly like one that opens with it."""
    block = {"changed": [_e("A clean break — **breaking**, but pre-1.0.", True)]}
    assert gc.advise(_data("0.10.0", **block))[0] == "minor"


# --- the cross-check --------------------------------------------------------


def test_flag_without_marker_is_a_disagreement():
    """A break the reader is never told about."""
    data = _data("0.10.0", changed=[_e("The default changed.", breaking=True)])
    hits = gc._breaking_check(data)
    assert len(hits) == 1
    assert "no **Breaking:** marker" in hits[0]


def test_marker_without_flag_is_a_disagreement():
    """The reverse: the entry says so, the advisor would still say patch."""
    data = _data("0.10.0", changed=[_e("**Breaking:** the default changed.")])
    hits = gc._breaking_check(data)
    assert len(hits) == 1
    assert '"breaking": true is missing' in hits[0]


def test_agreeing_entries_pass_the_cross_check():
    data = _data(
        "0.10.0",
        changed=[_e("**Breaking:** the default changed.", breaking=True)],
        fixed=[_e("Something about a non-breaking fix.")],
    )
    assert gc._breaking_check(data) == []


def test_the_repo_changelog_agrees_with_itself():
    """The real file, every release included — this is what the CI gate runs."""
    data = gc._load()
    assert gc._breaking_check(data) == []


# --- policy and implementation held together --------------------------------


def test_releasing_md_still_documents_the_convention_the_advisor_implements():
    """The advisor encodes RELEASING.md's pre-1.0 convention. If that policy is
    ever revised, this fails and points at the code that has to move with it —
    the alternative is a tool that quietly contradicts the document sending
    people to it, which is exactly what happened before."""
    policy = (REPO / "RELEASING.md").read_text(encoding="utf-8")
    assert "Pre-1.0 convention" in policy, "the policy section was renamed or removed"
    para = policy.split("Pre-1.0 convention", 1)[1].split("\n\n", 1)[0]
    assert "breaking" in para and "MINOR" in para, "pre-1.0: breaking → MINOR"
    assert "PATCH" in para, "pre-1.0: features and fixes → PATCH"
