"""The nightly cut derives; it never guesses on an append-only registry.

#809 is the specimen these tests are built around: `reference` published a
breaking 0.12.0 and the crates re-exporting its types — none of which changed a
line themselves — stayed pinned to `^0.11.0` on the registry. Every crate-local
signal (API snapshot, code diff, workspace build) was green, because in-tree
the deps are `path` deps and always unify. The fault existed only in the shape
of the PUBLISHED manifests, so the derivation has to read the registry's own
requirement ranges — and these tests pin that it does, that it maps the answer
to the 0.x parts #806 fixed (never a major), and that on partial knowledge it
says "unconcluded" rather than either of the two convenient lies.

The other hazard is spending versions: a stamp is not a publish (emit 0.13.0's
API delta was measured from its stamp and showed +7 -4 that was already ON the
registry), so a delta whose baseline cannot be placed in history must not turn
into a bump. crates.io cannot take a number back.

Everything here is pure over a `collect()`-shaped dict or over injected index
rows; no test touches the network or git history it does not own.
"""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import pytest
from _tools import default_crate, load_tool, report_of

REPO = Path(__file__).resolve().parents[1]

# Order matters: `engine_cut` imports `release_status` by bare name, and the
# registration `load_tool` does is what binds it to THIS file's instance.
rs = load_tool("release_status")
ec = load_tool("engine_cut")

#: The one all-quiet row (tests/_tools.py) — a `CrateStatus`, so a typo'd
#: field below is an error at the call, not a silently ignored dict key.
DEFAULT = default_crate(rs)


def _crate(**kw):
    return replace(DEFAULT, **kw)


def _status(*crates):
    return report_of(rs, *crates)


# --- caret admission: the 0.x rule the whole fault class hangs on ---


@pytest.mark.parametrize(
    ("req", "version", "admitted"),
    [
        ("^0.11.0", "0.11.0", True),
        ("^0.11.0", "0.11.9", True),
        ("^0.11.0", "0.12.0", False),  # the #809 shape
        ("^0.11.0", "0.10.0", False),
        ("^1.2.0", "1.9.0", True),
        ("^1.2.0", "2.0.0", False),
        ("^0.0.3", "0.0.4", False),  # 0.0.x locks the patch
        ("=0.11.0", "0.11.0", True),
        ("=0.11.0", "0.11.1", False),
        (">=0.11.0", "0.12.0", True),  # an operator this repo never publishes:
        # undecided reads as admitting, because the false alarm here is a bump
        # demanded that is not owed
    ],
)
def test_caret_admission(req, version, admitted):
    assert rs.caret_admits(req, version) is admitted


# --- the stale-pin signal reads the index rows nothing else can see ---


def _row(vers: str, deps: list[tuple[str, str]], *, yanked: bool = False) -> dict:
    return {
        "vers": vers,
        "yanked": yanked,
        "deps": [{"name": n, "req": r} for n, r in deps],
    }


def test_a_floor_past_a_published_pin_is_named():
    rows = [_row("0.11.0", [("laterite-ags4-reference", "^0.11.0")])]
    out = rs.deps_behind(rows, "0.11.0", {"laterite-ags4-reference": "0.12.0"})
    assert out == ["laterite-ags4-reference ^0.11.0 left behind by floor 0.12.0"]


def test_a_pin_the_floor_still_satisfies_is_silent():
    rows = [_row("0.11.0", [("laterite-ags4-reference", "^0.11.0")])]
    assert rs.deps_behind(rows, "0.11.0", {"laterite-ags4-reference": "0.11.2"}) == []


def test_third_party_deps_are_not_ours_to_judge():
    rows = [_row("0.11.0", [("serde", "^1")])]
    assert rs.deps_behind(rows, "0.11.0", {"laterite-ags4-reference": "0.12.0"}) == []


def test_the_baseline_skips_yanked_versions():
    """A yanked version is a fact, not a baseline — nobody resolves against it."""
    rows = [_row("0.12.0", [], yanked=True), _row("0.11.0", [])]
    assert rs.highest_live(rows) == "0.11.0"


# --- the 0.x part mapping (#806): never a major ---


@pytest.mark.parametrize(
    ("added", "removed", "code", "stale", "part"),
    [
        (0, 0, False, False, "none"),
        (3, 0, False, False, "minor"),  # additive takes the minor on 0.x
        (0, 2, False, False, "minor"),  # breaking takes the minor too — never major
        (0, 0, True, False, "patch"),
        (0, 0, False, True, "minor"),  # the cascade: a stale pin is breaking
    ],
)
def test_the_part_mapping_never_says_major(added, removed, code, stale, part):
    assert rs.required_part(added, removed, code, stale) == part


@pytest.mark.parametrize(
    ("published", "stamped", "part", "covered"),
    [
        ("0.11.0", "0.12.0", "minor", True),
        ("0.11.0", "0.11.1", "minor", False),  # a patch stamp does not cover a minor
        ("0.11.0", "0.11.1", "patch", True),
        ("0.11.0", "0.11.0", "minor", False),
        ("0.11.0", "0.11.0", "none", True),
    ],
)
def test_a_stamp_covers_the_part_or_it_does_not(published, stamped, part, covered):
    assert rs.covers(published, stamped, part) is covered


# --- the action table: every branch acts on confirmed knowledge or declines ---


def act(**kw) -> tuple[str, str]:
    args = {
        "state": "ok",
        "tier": "engine",
        "live": "0.11.0",
        "stamped": "0.11.0",
        "part": "none",
        "baseline_kind": "publish",
        "deps_stale": False,
    }
    args.update(kw)
    return rs.cut_action(**args)


def test_level_crate_gets_nothing():
    assert act() == ("none", "level with the registry")


def test_a_stale_pin_cuts_a_bump():
    assert act(part="minor", deps_stale=True)[0] == "bump"


def test_a_covered_stamp_cuts_a_publish_not_a_second_bump():
    action, _ = act(state="owed", stamped="0.12.0", part="minor")
    assert action == "publish"


def test_an_insufficient_stamp_is_bumped_again():
    """Stamped a patch when a minor is owed: publishing it would not pay the debt."""
    action, _ = act(state="owed", stamped="0.11.1", part="minor")
    assert action == "bump"


def test_the_product_tier_is_not_the_engine_cuts_to_touch():
    assert act(tier="product", part="minor", deps_stale=True)[0] == "none"


def test_an_unlabelled_tier_is_refused_not_defaulted():
    assert act(tier="?", part="minor", deps_stale=True)[0] == "none"


def test_a_stamp_only_baseline_never_spends_a_version():
    """The emit 0.13.0 lesson: the delta may already be ON the registry."""
    action, why = act(baseline_kind="stamp", part="minor")
    assert action == "unconcluded"
    assert "already be on the registry" in why


def test_the_stale_pin_signal_survives_a_stamp_only_baseline():
    """deps_behind is registry-derived — the baseline doubt does not touch it."""
    assert act(baseline_kind="stamp", part="minor", deps_stale=True)[0] == "bump"


def test_a_registry_ahead_of_the_tree_needs_a_human():
    assert act(live="0.13.0", stamped="0.12.0")[0] == "human"


def test_a_yanked_stamp_needs_a_human():
    assert act(state="yanked")[0] == "human"


def test_no_registry_answer_concludes_nothing():
    assert act(state="unknown")[0] == "unconcluded"


# --- engine_cut: what each nightly step reads off the one snapshot ---


def test_tokens_are_space_free_or_the_tracker_marker_corrupts():
    """The tracker's state marker regex stops at whitespace; a token with a
    space would silently truncate the stored set and every later night would
    read as a change."""
    status = _status(
        _crate(cut_action="bump", part_required="minor"),
        _crate(crate="laterite-ags4-emit", cut_action="publish", version="0.13.0"),
        _crate(crate="laterite-ags4-trust", cut_action="human"),
    )
    toks = ec.tokens(status)
    assert toks == [
        "laterite-ags4-core:bump-minor",
        "laterite-ags4-emit:publish-0.13.0",
        "laterite-ags4-trust:human",
    ]
    assert all(" " not in t for t in toks)


def test_bumps_lists_only_the_bump_actions():
    status = _status(
        _crate(cut_action="bump", part_required="minor"),
        _crate(crate="laterite-ags4-emit", cut_action="publish"),
    )
    assert ec.bumps(status) == [("laterite-ags4-core", "minor")]


def test_publish_owed_lists_only_the_publishes():
    status = _status(
        _crate(cut_action="bump", part_required="minor"),
        _crate(crate="laterite-ags4-emit", cut_action="publish"),
    )
    assert ec.publish_owed(status) == ["laterite-ags4-emit"]


def test_nothing_owed_with_a_dark_crate_is_not_an_all_clear():
    """An empty item list closes the tracker, so it may only be emitted on full
    knowledge — partial knowledge must leave the tracker standing."""
    status = _status(_crate(cut_action="unconcluded"))
    assert ec.tokens(status) == []
    assert ec.unconcluded(status) == ["laterite-ags4-core"]


# --- the tier register: every published crate says which train cuts it ---


def test_every_published_crate_declares_its_release_tier():
    """#806: an unlabelled crate must fail loudly here, never default into a
    tier — the cut refuses '?' at runtime, and this is what makes the omission
    a red test instead of a silently skipped crate."""
    for crate in rs.engine_crates():
        tier = rs.release_tier(crate)
        assert tier in rs.TIERS, (
            f"{crate}: add `[package.metadata.laterite] release_tier` to its "
            "Cargo.toml — 'engine' rides the nightly cut, 'product' the product trains"
        )


def test_the_facade_is_product_tier_until_parity():
    """dec-facade-parity: excluded from the engine cut until it reaches parity
    with the Python/Node surfaces, then it rides the product release type."""
    assert rs.release_tier("laterite") == "product"


# --- the coherence gate (#809's PR-time form) ---


def _fetch_for(rows_by_crate: dict):
    return lambda crate: rows_by_crate.get(crate)


def test_introduced_debt_fails_and_names_the_fix(capsys, monkeypatch):
    """A PR moving reference's floor past diff's published pin fails, and the
    message names the crate to bump — the #809 review, run before the merge."""
    monkeypatch.setattr(rs, "engine_crates", lambda: ["laterite-ags4-diff"])
    monkeypatch.setattr(rs, "release_tier", lambda c: "engine")
    monkeypatch.setattr(rs, "version_of", lambda *a: "0.11.0")
    monkeypatch.setattr(
        rs,
        "workspace_floors",
        lambda text=None: (
            {"laterite-ags4-reference": "0.11.0"}
            if text == "BASE"
            else {"laterite-ags4-reference": "0.12.0"}
        ),
    )
    fetch = _fetch_for(
        {
            "laterite-ags4-diff": [
                _row("0.11.0", [("laterite-ags4-reference", "^0.11.0")])
            ]
        }
    )
    assert rs.check_coherence(fetch, "BASE") == 1
    out = capsys.readouterr().out
    assert "laterite-ags4-diff: laterite-ags4-reference ^0.11.0" in out


def test_standing_debt_is_the_nightlys_not_this_prs(capsys, monkeypatch):
    """Debt already on the base must not redden an innocent PR — a gate that
    fails every PR over someone else's debt is a gate that gets skipped."""
    monkeypatch.setattr(rs, "engine_crates", lambda: ["laterite-ags4-diff"])
    monkeypatch.setattr(rs, "release_tier", lambda c: "engine")
    monkeypatch.setattr(rs, "version_of", lambda *a: "0.11.0")
    monkeypatch.setattr(
        rs,
        "workspace_floors",
        lambda text=None: {"laterite-ags4-reference": "0.12.0"},
    )
    fetch = _fetch_for(
        {
            "laterite-ags4-diff": [
                _row("0.11.0", [("laterite-ags4-reference", "^0.11.0")])
            ]
        }
    )
    assert rs.check_coherence(fetch, "BASE") == 0
    assert "standing stale pin(s) left to the nightly cut" in capsys.readouterr().out


def test_a_crate_republishing_anyway_is_not_flagged(monkeypatch):
    """Its stamped version is ahead of the registry, so its fresh floors ride
    along with the publish — demanding a second bump would be noise."""
    monkeypatch.setattr(rs, "engine_crates", lambda: ["laterite-ags4-diff"])
    monkeypatch.setattr(rs, "release_tier", lambda c: "engine")
    monkeypatch.setattr(rs, "version_of", lambda *a: "0.12.0")  # bumped in this PR
    monkeypatch.setattr(
        rs, "workspace_floors", lambda text=None: {"laterite-ags4-reference": "0.12.0"}
    )
    fetch = _fetch_for(
        {
            "laterite-ags4-diff": [
                _row("0.11.0", [("laterite-ags4-reference", "^0.11.0")])
            ]
        }
    )
    assert rs.check_coherence(fetch, None) == 0


def test_an_unreachable_registry_concludes_nothing_out_loud(capsys, monkeypatch):
    monkeypatch.setattr(rs, "engine_crates", lambda: ["laterite-ags4-diff"])
    monkeypatch.setattr(rs, "release_tier", lambda c: "engine")
    monkeypatch.setattr(
        rs, "workspace_floors", lambda text=None: {"laterite-ags4-reference": "0.12.0"}
    )
    assert rs.check_coherence(lambda crate: None, None) == 0
    out = capsys.readouterr().out
    assert "1 unreachable" in out
    assert "concluded NOTHING" in out


# --- the published-commit baseline holds against this repo's real history ---


def test_stamp_of_version_finds_the_commit_that_set_the_line():
    """Property, not a pinned sha: the commit found must actually carry the
    version it is claimed to stamp, and be the LAST one that did."""
    manifest = REPO / "rust-packages" / "laterite-ags4-emit" / "Cargo.toml"
    sha = rs.stamp_of_version(manifest, "0.11.0")
    assert sha, "emit 0.11.0 was stamped in this history"
    at = rs.sh("git", "show", f"{sha}:rust-packages/laterite-ags4-emit/Cargo.toml")
    assert 'version = "0.11.0"' in at


def test_a_version_never_stamped_resolves_to_nothing():
    manifest = REPO / "rust-packages" / "laterite-ags4-emit" / "Cargo.toml"
    assert rs.stamp_of_version(manifest, "9.9.9") == ""


# --- the coherence cascade: the cut applies it, round by round (#1043) ---

REF, CORE, DIFF = "laterite-ags4-reference", "laterite-ags4-core", "laterite-ags4-diff"
EMIT, MERGE, VAL = (
    "laterite-ags4-emit",
    "laterite-ags4-merge",
    "laterite-ags4-validator",
)


class _World:
    """A registry whose stranded set answers to what has been bumped so far.

    `rounds` maps "every crate bumped so far" to the debt the check names
    next — the shape #1031 had to walk by hand, replayed without a network.
    """

    def __init__(self, rounds: list[tuple[set[str], list[tuple[str, str]]]]):
        self.rounds = rounds
        self.bumped: list[str] = []

    def stranded(self) -> list[tuple[str, str]]:
        done = set(self.bumped)
        for after, debt in self.rounds:
            if after == done:
                return debt
        return []

    def bump(self, crate: str, part: str = "patch") -> None:
        self.bumped.append(crate)


def _1031() -> _World:
    return _World(
        [
            (
                set(),
                [
                    (CORE, f"{REF} ^0.15.0 left behind by floor 0.16.0"),
                    (DIFF, f"{REF} ^0.15.0 left behind by floor 0.16.0"),
                ],
            ),
            (
                {CORE, DIFF},
                [
                    (EMIT, f"{CORE} ^0.16.0 left behind by floor 0.17.1"),
                    (MERGE, f"{DIFF} ^0.13.0 left behind by floor 0.14.1"),
                    (VAL, f"{CORE} ^0.16.0 left behind by floor 0.17.1"),
                ],
            ),
        ]
    )


def test_the_cascade_replays_1031s_two_rounds_in_one_run():
    """Reference's minor strands core + diff; their patches strand emit, merge
    and validator. #1031 needed a human for each round; one run does both."""
    world = _1031()
    applied = ec.cascade(world.stranded, world.bump, limit=12)
    assert world.bumped == [CORE, DIFF, EMIT, MERGE, VAL]
    assert [(b.crate, b.round) for b in applied] == [
        (CORE, 1),
        (DIFF, 1),
        (EMIT, 2),
        (MERGE, 2),
        (VAL, 2),
    ]
    # Each bump says whose moved floor made it necessary.
    assert applied[2].because == (f"{CORE} ^0.16.0 left behind by floor 0.17.1",)


def test_nothing_stranded_bumps_nothing():
    """A cut with no stranded pin behaves exactly as before the cascade."""
    world = _World([])
    assert ec.cascade(world.stranded, world.bump, limit=12) == []
    assert world.bumped == []


def test_a_round_that_adds_no_new_crate_is_stuck_and_names_the_debt():
    """Bumping a crate that is demanded again did not clear it — another lap
    would loop forever, so the cut fails loudly instead of opening a PR."""
    world = _World(
        [
            (set(), [(CORE, f"{REF} ^0.15.0 left behind by floor 0.16.0")]),
            ({CORE}, [(CORE, f"{REF} ^0.15.0 left behind by floor 0.16.0")]),
        ]
    )
    with pytest.raises(ec.CascadeStuck) as stuck:
        ec.cascade(world.stranded, world.bump, limit=12)
    assert stuck.value.demanded == [CORE]
    assert "no new crate" in str(stuck.value)
    assert world.bumped == [CORE]


def test_the_round_limit_fails_with_the_debt_still_owed():
    world = _1031()
    with pytest.raises(ec.CascadeStuck) as stuck:
        ec.cascade(world.stranded, world.bump, limit=1)
    assert stuck.value.demanded == [EMIT, MERGE, VAL]
    assert "limit" in str(stuck.value)


def test_the_limit_is_the_published_engine_crate_count(monkeypatch):
    monkeypatch.setattr(rs, "engine_crates", lambda: [CORE, DIFF, "laterite"])
    monkeypatch.setattr(
        rs, "release_tier", lambda c: "product" if c == "laterite" else "engine"
    )
    assert ec.cascade_limit() == 2


def test_the_cascade_section_names_round_and_cause():
    world = _1031()
    text = ec.render_cascade(ec.cascade(world.stranded, world.bump, limit=12))
    assert "coherence cascade" in text
    assert f"{EMIT} patch" in text and "round 2" in text
    assert f"{CORE} ^0.16.0 left behind by floor 0.17.1" in text
    assert ec.render_cascade([]) == ""


def test_coherence_reading_returns_the_debt_it_prints(monkeypatch):
    """The cascade reads the gate's own answer — the check stays the one
    authority on what is stranded, never a second derivation."""
    monkeypatch.setattr(rs, "engine_crates", lambda: [DIFF])
    monkeypatch.setattr(rs, "release_tier", lambda c: "engine")
    monkeypatch.setattr(rs, "version_of", lambda *a: "0.11.0")
    monkeypatch.setattr(rs, "workspace_floors", lambda text=None: {REF: "0.12.0"})
    fetch = _fetch_for({DIFF: [_row("0.11.0", [(REF, "^0.11.0")])]})
    reading = rs.coherence_reading(fetch, None)
    assert reading.introduced == [(DIFF, f"{REF} ^0.11.0 left behind by floor 0.12.0")]
    assert (reading.asked, reading.unreachable, reading.standing) == (1, 0, 0)


def test_a_stuck_cascade_exits_4_with_no_pr_body(capsys, monkeypatch):
    """The job's guarantee, not just the loop's: a stuck cascade prints no PR
    section on stdout and exits non-zero, so the PR step never runs."""
    debt = [(CORE, f"{REF} ^0.15.0 left behind by floor 0.16.0")]
    reading = rs.Coherence(introduced=debt, standing=0, unreachable=0, asked=1)
    monkeypatch.setattr(rs, "coherence_reading", lambda fetch, base: reading)
    monkeypatch.setattr(ec, "_bump", lambda crate, part: None)
    monkeypatch.setattr(ec, "cascade_limit", lambda: 12)
    monkeypatch.setattr(ec.sys, "argv", ["engine_cut.py", "--cascade"])
    assert ec.main() == ec.EXIT_CASCADE_STUCK
    out = capsys.readouterr()
    assert out.out == ""
    assert (
        f"cascade stuck: round 2 added no new crate; still demanded: {CORE}" in out.err
    )


def test_the_cascade_reports_its_scope_every_round(capsys, monkeypatch):
    reading = rs.Coherence(introduced=[], standing=0, unreachable=3, asked=9)
    monkeypatch.setattr(rs, "coherence_reading", lambda fetch, base: reading)
    monkeypatch.setattr(ec.sys, "argv", ["engine_cut.py", "--cascade"])
    assert ec.main() == 0
    err = capsys.readouterr().err
    assert "9 published crate(s) asked, 3 unreachable" in err
    assert "concluding nothing" in err


# --- #1073: the facade is asked, never cascaded; public deps take a minor ---

PARSE, FACADE = "laterite-ags4-parse", "laterite"


def _facade_world(monkeypatch, *, stamped: str = "0.14.1"):
    """The 0.19.2 situation: parse's floor moved 0.13 -> 0.14 while the
    published facade still pins `^0.13.0`."""
    monkeypatch.setattr(rs, "engine_crates", lambda: [FACADE])
    monkeypatch.setattr(
        rs, "release_tier", lambda c: "product" if c == FACADE else "engine"
    )
    monkeypatch.setattr(rs, "version_of", lambda *a: stamped)
    monkeypatch.setattr(
        rs,
        "workspace_floors",
        lambda text=None: {PARSE: "0.13.0"} if text == "BASE" else {PARSE: "0.14.0"},
    )
    return _fetch_for({FACADE: [_row("0.14.1", [(PARSE, "^0.13.0")])]})


def test_a_stranded_facade_fails_coherence_and_owes_a_product_release(
    capsys, monkeypatch
):
    """Before #1073 the reading skipped every non-engine crate, so this passed."""
    fetch = _facade_world(monkeypatch)
    assert rs.check_coherence(fetch, "BASE") == 1
    out = capsys.readouterr().out
    assert f"{FACADE}: {PARSE} ^0.13.0 left behind by floor 0.14.0" in out
    assert "product release owed" in out


def test_the_facade_is_asked_and_counted_in_the_scope_line(capsys, monkeypatch):
    fetch = _facade_world(monkeypatch)
    reading = rs.coherence_reading(fetch, None)
    assert reading.introduced == []  # never the cascade's to bump
    assert reading.product == [(FACADE, f"{PARSE} ^0.13.0 left behind by floor 0.14.0")]
    assert reading.asked == 1
    rs.check_coherence(fetch, None)
    assert "1 published crate(s) asked" in capsys.readouterr().out


def test_a_facade_stamped_ahead_is_not_stranded(monkeypatch):
    """A product cut in the same PR re-stamps it, and its fresh floors ride along."""
    fetch = _facade_world(monkeypatch, stamped="0.15.0")
    assert rs.check_coherence(fetch, "BASE") == 0


def test_the_cascade_never_bumps_the_facade_and_reports_it(capsys, monkeypatch):
    debt = [(FACADE, f"{PARSE} ^0.13.0 left behind by floor 0.14.0")]
    reading = rs.Coherence(
        introduced=[], standing=0, unreachable=0, asked=1, product=debt
    )
    bumped: list[str] = []
    monkeypatch.setattr(rs, "coherence_reading", lambda fetch, base: reading)
    monkeypatch.setattr(ec, "_bump", lambda crate, part: bumped.append(crate))
    monkeypatch.setattr(ec, "cascade_limit", lambda: 12)
    monkeypatch.setattr(ec.sys, "argv", ["engine_cut.py", "--cascade"])
    assert ec.main() == 0
    assert bumped == []
    out = capsys.readouterr().out
    assert "product release owed" in out
    assert f"{FACADE}: {PARSE} ^0.13.0 left behind by floor 0.14.0" in out


def test_a_cascade_bump_whose_api_names_the_moved_dep_is_a_minor():
    """The cascade forms a verdict too: patching a crate whose public API names
    the moved dep would leave next night's cut owing a minor on top of it."""
    world = _World([(set(), [(VAL, f"{PARSE} ^0.13.0 left behind by floor 0.14.0")])])
    parts: dict[str, str] = {}

    def bump(crate: str, part: str) -> None:
        parts[crate] = part
        world.bump(crate)

    snap = "pub fn x(&laterite_ags4_parse::ParsedFile)"
    applied = ec.cascade(
        world.stranded,
        bump,
        limit=12,
        part=lambda crate, because: ec.cascade_part(snap, because),
    )
    assert parts == {VAL: "minor"}
    assert applied[0].part == "minor"
    assert ec.cascade_part("pub fn y()", applied[0].because) == "patch"


# Gap 2: an incompatible move of a dependency the crate's API names.


def test_a_public_dep_moved_incompatibly_is_named():
    snap = "impl From<laterite_ags4_parse::ParseError> for laterite_ags4_validator::E"
    moves = rs.public_dep_moves(snap, {PARSE: "^0.13.0"}, {PARSE: "^0.14.0"})
    assert len(moves) == 1
    assert PARSE in moves[0]
    assert rs.required_part(0, 0, False, False, bool(moves)) == "minor"
    # The same move, code changed, would have been cut as a patch before.
    assert rs.required_part(0, 0, True, False, bool(moves)) == "minor"


def test_a_private_dep_move_keeps_todays_verdict():
    snap = "pub fn laterite_ags4_validator::validate(&str)"
    moves = rs.public_dep_moves(snap, {PARSE: "^0.13.0"}, {PARSE: "^0.14.0"})
    assert moves == []
    assert rs.required_part(0, 0, True, False, bool(moves)) == "patch"


@pytest.mark.parametrize(
    ("base", "head", "moved"),
    [
        ("0.13.0", "0.14.0", True),
        ("^0.13.0", "^0.14.0", True),
        ("0.13.0", "0.13.4", False),  # compatible under caret
        ("1.2.0", "1.9.0", False),
        ("1.2.0", "2.0.0", True),
    ],
)
def test_an_incompatible_move_is_a_caret_break(base, head, moved):
    snap = "laterite_ags4_parse::X"
    assert bool(rs.public_dep_moves(snap, {PARSE: base}, {PARSE: head})) is moved


def test_dep_requirements_resolve_workspace_floors():
    crate = (
        "[dependencies]\n"
        "laterite-ags4-parse = { workspace = true }\n"
        'laterite-transport = { version = "0.12.0" }\n'
        'serde = "1"\n'
        "[dev-dependencies]\n"
        "laterite-ags4-emit = { workspace = true }\n"
    )
    ws = (
        "[workspace.dependencies]\n"
        'laterite-ags4-parse = { path = "p", version = "0.14.0" }\n'
        'laterite-ags4-emit = { path = "e", version = "0.20.0" }\n'
    )
    assert rs.dep_requirements(crate, ws) == {
        PARSE: "0.14.0",
        "laterite-transport": "0.12.0",
    }


# Gap 3: a facade stamped ahead of crates.io is publish-owed.


def test_a_facade_stamped_ahead_of_the_registry_is_publish_owed():
    action, _ = act(tier="product", state="owed", stamped="0.14.2", live="0.14.1")
    assert action == "publish"
    status = _status(
        _crate(crate=FACADE, tier="product", version="0.14.2", cut_action="publish")
    )
    assert ec.publish_owed(status) == [FACADE]
    assert ec.tokens(status) == [f"{FACADE}:publish-0.14.2"]


def test_a_facade_level_with_the_registry_is_not_publish_owed():
    assert act(tier="product", stamped="0.14.1", live="0.14.1")[0] == "none"


def test_a_stranded_facade_is_a_tracker_item_not_a_bump():
    status = _status(
        _crate(
            crate=FACADE,
            tier="product",
            deps_behind=[f"{PARSE} ^0.13.0 left behind by floor 0.14.0"],
        )
    )
    assert ec.tokens(status) == [f"{FACADE}:product-release-owed"]
    assert ec.bumps(status) == []
