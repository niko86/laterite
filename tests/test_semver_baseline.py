"""The semver gate's partition of the publish set (#782).

`check_semver.py` compares each publishable crate against the version crates.io
serves, and a crate can be in one of three states:

    ENFORCING  tree == published. Every lint is live, and a break fails.
    ahead      tree != published. The version pair already permits the change,
               so `cargo semver-checks` skips every lint — correctly, because the
               bump was already declared.
    ABSENT     never published. There is no prior API to break.

The `ahead` state is the one worth a test. It is a legitimate green run in which
the gate enforces NOTHING, and it is the state the tree sits in for most of a
release cycle. A gate that reports green while looking at nothing has to say so,
which is only true if every crate is accounted for — hence the partition property
below rather than a spot check on one bucket.

No network: `classify` takes the registry answer as an argument precisely so the
partition can be asserted without one. Whether the registry lookup itself works is
the run's own business, and it fails loudly there.
"""

from __future__ import annotations

from _tools import load_tool

_GATE = load_tool("check_semver")
classify = _GATE.classify
tree_version = _GATE.tree_version
version_in = _GATE.version_in
PUBLISH_SET = _GATE.PUBLISH_SET


def test_every_crate_lands_in_exactly_one_bucket() -> None:
    """The property that makes "it reported what it skipped" true.

    A crate falling through would be silently unchecked AND unnamed, which is the
    failure this gate's whole reporting contract exists to prevent.

    The input deliberately spans ALL THREE states. A first draft of this fed
    `{c: tree_version(c)}`, so no crate was ever `None` and the absent branch was
    never reached — deleting that branch outright left this test green. Feeding
    one of each is what makes the partition claim falsifiable.
    """
    published: dict[str, str | None] = {}
    for i, crate in enumerate(PUBLISH_SET):
        published[crate] = (
            None if i % 3 == 0 else tree_version(crate) if i % 3 == 1 else "0.0.1"
        )
    enforcing, ahead, absent = classify(published)

    assert enforcing and ahead and absent, (
        "the input must reach all three branches or this asserts nothing"
    )
    assert sorted(enforcing + ahead + absent) == sorted(PUBLISH_SET)
    assert len(enforcing) + len(ahead) + len(absent) == len(PUBLISH_SET), (
        "a crate appears in more than one bucket"
    )


def test_the_three_states_are_told_apart() -> None:
    core_tree = tree_version("laterite-ags4-core")
    enforcing, ahead, absent = classify(
        {
            # level with the registry — every lint live
            "laterite-ags4-core": core_tree,
            # tree ahead — semver already permits the change
            "laterite-ags4-parse": "0.0.1",
            # never published
            "laterite-transport": None,
        }
    )
    assert enforcing == ["laterite-ags4-core"]
    assert ahead == ["laterite-ags4-parse"]
    assert absent == ["laterite-transport"]


def test_tree_version_resolves_workspace_inheritance() -> None:
    """The facade carries its own number; the engine crates inherit one.

    Asserting they DIFFER is the point — a `tree_version` that only ever read
    `[workspace.package]` would pass every check on the ten engine crates and be
    wrong about the eleventh, which is the one currently level with the registry
    and therefore the only one this gate is enforcing on at all.
    """
    facade = tree_version("laterite")
    engine = tree_version("laterite-ags4-core")
    assert facade and engine
    assert facade != engine, (
        "the facade and the engine tier are separately versioned by design; if "
        "these ever match it is a coincidence, and this test should be re-aimed "
        "at the manifests rather than deleted"
    )


def test_the_version_is_read_through_cargos_colour_codes() -> None:
    """CI sets `CARGO_TERM_COLOR: always`, and this is what that does to the field.

    Captured from a real `cargo info` run under forced colour, not hand-written.
    A plain `startswith("version:")` misses it entirely — which is how the first
    cut of this gate reported all eleven published crates as never published, in
    CI only, while passing locally.
    """
    coloured = (
        "laterite-ags4-core #ags #ags4\n"
        "DuckDB-free pure-string core modules for the AGS4 toolchain\n"
        "\x1b[1m\x1b[92mversion:\x1b[0m 0.9.0\n"
        "\x1b[1m\x1b[92mrust-version:\x1b[0m 1.85\n"
    )
    assert version_in(coloured) == "0.9.0"
    # rust-version must not be mistaken for it, coloured or not.
    assert version_in("rust-version: 1.85\n") is None


def test_output_with_no_version_line_is_not_read_as_a_version() -> None:
    """`None` here routes into a `die`, never into the ABSENT bucket.

    Conflating the two is the defect this pair of tests exists to hold shut: a
    crate reported ABSENT is a claim that there is no prior API to break, and an
    unparseable answer is not evidence for that claim.
    """
    assert version_in("") is None
    assert version_in("error: something else entirely\n") is None
