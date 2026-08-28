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

import importlib.util
import sys
from pathlib import Path

_TOOLS = Path(__file__).resolve().parents[1] / "tools"


def _load():
    # The house pattern for testing a tools/ script (test_check_doc_types.py):
    # load by path, not by import name. The `sys.path` insert is still needed —
    # the gate imports its sibling `check_package_contents` for the publish set.
    sys.path.insert(0, str(_TOOLS))
    spec = importlib.util.spec_from_file_location(
        "check_semver", _TOOLS / "check_semver.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_semver"] = mod
    spec.loader.exec_module(mod)
    return mod


_GATE = _load()
classify = _GATE.classify
tree_version = _GATE.tree_version
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
