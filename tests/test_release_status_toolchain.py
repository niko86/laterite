"""The cut's API delta must not read a snapshot re-render as API movement (#998).

#988 regenerated every snapshot because a new rustdoc nightly renders derives
as `Self`, and the next cut read each rewritten line as one removal plus one
addition: twelve crates took a minor with no API change behind any of them.
#997 pinned the nightly, but moving that pin on purpose re-renders every
snapshot again. So `api_delta` checks which nightly each side was rendered on.
If both sides used the same nightly, the cheap text diff stands. If not, it
renders both sides on HEAD's nightly. If that render cannot run, the crate goes
to a human, never to a bump and never to a silent zero.

These tests build a small git history of their own, so the real `git diff` and
`git show` reads are what is under test. Only the render is stubbed, because it
needs a nightly, `cargo-public-api` and crates.io.
"""

from __future__ import annotations

import json
import subprocess
from dataclasses import asdict, replace
from typing import TYPE_CHECKING

import pytest
from _tools import default_crate, load_tool, report_of

if TYPE_CHECKING:
    from pathlib import Path

rs = load_tool("release_status")
cpa = load_tool("check_public_api")

OLD_PIN = "nightly-2026-09-29"
NEW_PIN = "nightly-2026-10-20"

#: The #988 shape: one derive, rendered two ways by two nightlies.
BEFORE = "pub fn demo::Foo::clone(&self) -> demo::Foo"
AFTER = "pub fn demo::Foo::clone(&self) -> Self"
REAL_ADDITION = "pub fn demo::Foo::grow(&mut self)"


class _Repo:
    """A two-commit history: the published baseline, then HEAD."""

    def __init__(self, root: Path):
        self.root = root
        self._git("init", "-q", "-b", "main")

    def _git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-c", "user.name=t", "-c", "user.email=t@t", *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()

    def commit(self, *, pin: str | None, snapshots: dict[str, list[str]]) -> str:
        tool = self.root / "tools" / "check_public_api.py"
        tool.parent.mkdir(parents=True, exist_ok=True)
        # `pin=None` is a commit from before #997, whose tool named no nightly.
        tool.write_text(f'NIGHTLY = "{pin}"\n' if pin else "# no pin yet\n")
        for name, lines in snapshots.items():
            snap = self.root / "tools" / "release" / "public-api" / name
            snap.parent.mkdir(parents=True, exist_ok=True)
            snap.write_text("\n".join(lines) + "\n")
        self._git("add", "-A")
        self._git("commit", "-q", "-m", "x")
        return self._git("rev-parse", "HEAD")


@pytest.fixture
def repo(tmp_path, monkeypatch):
    r = _Repo(tmp_path)
    monkeypatch.setattr(rs, "ROOT", tmp_path)
    monkeypatch.setattr(rs, "SNAPSHOTS", tmp_path / "tools" / "release" / "public-api")
    return r


class _Render:
    """Stands in for `cargo public-api diff <version>` on HEAD's nightly."""

    def __init__(self, out: list[str] | None):
        self.out = out
        self.calls: list[tuple[str, str, str, bool]] = []

    def __call__(
        self, crate: str, version: str, toolchain: str, *, all_features: bool
    ) -> list[str] | None:
        self.calls.append((crate, version, toolchain, all_features))
        return self.out


def _history(repo: _Repo, *, pin_then, pin_now, now: list[str]) -> str:
    since = repo.commit(pin=pin_then, snapshots={"demo.txt": [BEFORE]})
    repo.commit(pin=pin_now, snapshots={"demo.txt": now})
    return since


# --- the acceptance cases ---


def test_a_rerender_across_a_pin_move_is_not_a_delta(repo):
    """The #988 shape. The text diff reads +1 -1; rendered on one nightly the
    two sides agree, so there is nothing to bump."""
    since = _history(repo, pin_then=OLD_PIN, pin_now=NEW_PIN, now=[AFTER])
    render = _Render([])
    d = rs.api_delta(since, "demo", "0.1.0", render=render)
    assert (d.added, d.removed, d.removed_names) == (0, 0, [])
    assert d.source == "render"
    assert render.calls == [("demo", "0.1.0", NEW_PIN, True)]
    assert rs.required_part(d.added, d.removed, False, False) == "none"


def test_the_same_text_change_under_one_pin_still_counts(repo):
    """Same nightly on both sides: the text diff is the measure and no render runs."""
    since = _history(repo, pin_then=OLD_PIN, pin_now=OLD_PIN, now=[AFTER])
    render = _Render([])
    d = rs.api_delta(since, "demo", "0.1.0", render=render)
    assert (d.added, d.removed, d.removed_names) == (1, 1, [BEFORE])
    assert d.source == "snapshot"
    assert render.calls == []


def test_a_real_addition_across_a_pin_move_still_cuts_a_minor(repo):
    since = _history(
        repo, pin_then=OLD_PIN, pin_now=NEW_PIN, now=[AFTER, REAL_ADDITION]
    )
    d = rs.api_delta(since, "demo", "0.1.0", render=_Render([f"+{REAL_ADDITION}"]))
    assert (d.added, d.removed) == (1, 0)
    assert d.source == "render"
    assert rs.required_part(d.added, d.removed, False, False) == "minor"


def test_a_render_that_cannot_run_goes_to_a_human(repo):
    """Not a bump, and not a silent zero either."""
    since = _history(repo, pin_then=OLD_PIN, pin_now=NEW_PIN, now=[AFTER])
    d = rs.api_delta(since, "demo", "0.1.0", render=_Render(None))
    assert d.source == "unavailable"
    assert OLD_PIN in d.note and NEW_PIN in d.note
    action, why = rs.cut_action(
        state="ok",
        tier="engine",
        live="0.1.0",
        stamped="0.1.0",
        part=rs.required_part(d.added, d.removed, False, False),
        baseline_kind="publish",
        deps_stale=False,
        delta_unavailable=d.note,
    )
    assert action == "human"
    assert NEW_PIN in why


def test_an_unavailable_delta_needs_a_human_even_when_code_moved(repo):
    """A code change alone would cut a patch, but the API may have moved too,
    and a patch would undersell that."""
    action, _ = rs.cut_action(
        state="ok",
        tier="engine",
        live="0.1.0",
        stamped="0.1.0",
        part="patch",
        baseline_kind="publish",
        deps_stale=False,
        delta_unavailable="toolchain mismatch",
    )
    assert action == "human"


def test_a_baseline_from_before_the_pin_is_an_unknown_toolchain(repo):
    """Every crate published before #997 has no recorded nightly. That is
    unknown, not a match."""
    since = _history(repo, pin_then=None, pin_now=NEW_PIN, now=[AFTER])
    render = _Render([])
    d = rs.api_delta(since, "demo", "0.1.0", render=render)
    assert d.source == "render"
    assert (d.added, d.removed) == (0, 0)
    assert len(render.calls) == 1


def test_unmoved_text_needs_no_render_even_across_a_pin_move(repo):
    """The text did not change, so no re-render happened to cancel out. This
    is what keeps the common nightly free of a toolchain install."""
    since = _history(repo, pin_then=None, pin_now=NEW_PIN, now=[BEFORE])
    render = _Render(None)
    d = rs.api_delta(since, "demo", "0.1.0", render=render)
    assert (d.added, d.removed, d.source) == (0, 0, "snapshot")
    assert render.calls == []


def test_no_published_version_leaves_nothing_to_render_against(repo):
    since = _history(repo, pin_then=OLD_PIN, pin_now=NEW_PIN, now=[AFTER])
    d = rs.api_delta(since, "demo", "", render=_Render([]))
    assert d.source == "unavailable"
    assert "no published version" in d.note


def test_the_facade_renders_both_of_its_surfaces(repo):
    """The facade keeps a default snapshot and an all-features one, so the
    render covers both, and a line in either one counts once."""
    since = repo.commit(
        pin=OLD_PIN,
        snapshots={"laterite.txt": [BEFORE], "laterite.all-features.txt": [BEFORE]},
    )
    repo.commit(
        pin=NEW_PIN,
        snapshots={"laterite.txt": [AFTER], "laterite.all-features.txt": [AFTER]},
    )
    render = _Render([f"+{REAL_ADDITION}"])
    d = rs.api_delta(since, "laterite", "0.1.0", render=render)
    assert [c[3] for c in render.calls] == [False, True]
    assert (d.added, d.removed, d.source) == (1, 0, "render")


def test_the_render_parser_nets_a_changed_item_like_the_text_diff():
    """`cargo public-api diff` lists a changed item as a -/+ pair. The same
    netting as the text diff applies, and the headings are not items."""
    out = [
        "Removed items from the public API",
        "=================================",
        "-pub fn demo::gone()",
        "",
        "Changed items in the public API",
        "===============================",
        "-pub fn demo::f(u8)",
        "+pub fn demo::f(u16)",
        "",
        "Added items to the public API",
        "=============================",
        "+pub fn demo::new()",
        "+impl core::marker::Send for demo::Foo",
    ]
    added, removed, names = rs.net_delta(out)
    # The `+impl` counts too (#1023): an added auto-trait impl is API.
    assert (added, removed) == (3, 2)
    assert names == ["pub fn demo::f(u8)", "pub fn demo::gone()"]


# --- impl lines are API: a method-less impl has no `pub fn` to count (#1023) ---


def test_an_added_method_less_impl_is_an_addition_and_cuts_a_minor():
    """`Copy` brings no method line, so counting only `pub` lines cut it as a
    patch while the 0.x mapping requires a minor for any addition."""
    added, removed, _ = rs.net_delta(["+impl core::marker::Copy for demo::X"])
    assert (added, removed) == (1, 0)
    assert rs.required_part(added, removed, True, False) == "minor"


def test_an_impl_removed_and_re_added_verbatim_nets_to_zero():
    line = "impl core::marker::Copy for demo::X"
    assert rs.net_delta([f"-{line}", f"+{line}"]) == (0, 0, [])


def test_a_removed_impl_is_a_removal_and_is_named():
    added, removed, names = rs.net_delta(["-impl core::cmp::Eq for demo::X"])
    assert (added, removed) == (0, 1)
    assert names == ["impl core::cmp::Eq for demo::X"]


# --- the source is reported on every run ---


def _status(**kw):
    return report_of(rs, replace(default_crate(rs), **kw))


def test_every_crate_line_names_its_measure():
    text = rs.render(_status(api_delta_source="render"))
    assert "[render]" in text
    assert "api delta: 0 from snapshot text, 1 from a same-toolchain render" in text


def test_an_unmeasured_crate_is_shouted_and_counted():
    s = _status(api_delta_source="unavailable", cut_action="human", cut_why="x")
    text = rs.render(s)
    assert "[UNMEASURED]" in text
    assert "1 unmeasured" in text
    assert "1 unmeasured" in rs.render_cut(s)


def test_the_measure_rides_the_json_wire():
    wire = json.loads(json.dumps(asdict(_status(api_delta_source="render"))))
    assert wire["engine_crates"][0]["api_delta_source"] == "render"
    assert rs.Report.from_json(wire) == _status(api_delta_source="render")


# --- one reader of the pin ---


def test_the_pin_parser_reads_the_live_tool():
    """`--print-toolchain` prints through this parser, so CI exercises it."""
    assert cpa.pinned_nightly(cpa.Path(cpa.__file__).read_text()) == cpa.NIGHTLY


def test_a_tool_without_a_pin_reads_as_unknown():
    assert cpa.pinned_nightly("# no pin yet\n") == ""
