"""The `--on-type-clash` mode set has ONE source, and the copies are pinned to it.

`TypeClashMode::ALL` in `laterite-ags4-merge` is the authority — `error`, `widen`,
`promote`, in that order (default first). It was hand-copied across the tree with
nothing comparing the copies back: `_cli.py`'s `_CLASH_CHOICES` (the argparse choice
list, load-bearing — it decides what the CLI accepts) sat one line below
`_DICT_CHOICES`, which already asks the registry. A fourth mode added to the Rust
enum would have reached the copy through no path. That is laterite-dev#549's Shape 1 (a hand-list
believed complete, nothing comparing it to the promise) applied to a closed value set
— the same shape as laterite-dev#550's fingerprint list and laterite-dev#557's rename.

`registry_type_clash_modes()` now exposes `TypeClashMode::ALL` to Python, and
`_CLASH_CHOICES` derives from it (laterite-dev#555 part 3). This file pins the *remaining*
copies that structurally cannot self-derive — the `Literal[...]` type alias needs
literal arguments, so it can't consume a runtime function — to that one source. They
pass trivially today; they are written for the day a mode is added, when the derived
list moves past a hand-list and the hand-list goes red.

The TypeScript copies (`laterite-node/ts/index.ts`, `web/src/lib/validator.ts`) are
the same shape one surface over; they are checked by the census value-set parity gate
(laterite-dev#555 part 3b) rather than here, because this suite cannot reach them.
"""

from __future__ import annotations

import argparse
import typing

import pytest
from laterite import _cli
from laterite import _laterite_native as _native


def _authority() -> list[str]:
    """The one source: TypeClashMode::ALL, via the PyO3 bridge."""
    modes = list(_native.registry_type_clash_modes())
    assert modes, "registry_type_clash_modes() returned nothing — the bridge is broken"
    return modes


def test_the_native_list_is_the_merge_enum() -> None:
    """The bridge itself. Ordered (default `error` first), as `ALL` is declared."""
    assert _authority() == ["error", "widen", "promote"], (
        "if this changed deliberately (a mode added/removed/reordered in "
        "laterite-ags4-merge's TypeClashMode::ALL), update this expectation — it is the "
        "one place the concrete set is asserted, so it should be a visible edit"
    )


def test_cli_choices_are_derived_not_hand_listed() -> None:
    """`_CLASH_CHOICES` is the argparse choice list — what the CLI actually accepts."""
    assert tuple(_cli._CLASH_CHOICES) == tuple(_authority()), (
        "_CLASH_CHOICES has drifted from TypeClashMode::ALL. It must derive from "
        "registry_type_clash_modes(), not be re-hard-coded."
    )


def test_the_merge_subparsers_on_type_clash_offers_exactly_those_modes() -> None:
    """Reach into the built parser: the `--on-type-clash` action's `choices`.

    `_CLASH_CHOICES` being right is necessary but not sufficient — the argument has to
    actually use it. This walks the real parser argparse builds, so a future edit that
    passes a different list to `add_argument` is caught.
    """
    parser = _cli._build_parser()
    found: list[str] = [
        sorted(action.choices or [])
        for action in _walk_actions(parser)
        if "--on-type-clash" in getattr(action, "option_strings", [])
    ]
    assert found, "no --on-type-clash argument found in the parser"
    for choices in found:
        assert choices == sorted(_authority()), (
            f"--on-type-clash offers {choices}, authority is {sorted(_authority())}"
        )


def test_the_public_literal_type_alias_matches_the_authority() -> None:
    """`laterite.TypeClashMode` is a `Literal[...]` — hand-typed, can't self-derive.

    A `Literal` needs literal arguments, so it cannot be `Literal[*runtime_list]`. It is
    therefore a genuine hand-copy that this gate exists to pin: if a mode is added to the
    Rust enum, `registry_type_clash_modes()` moves and this alias goes red until a human
    updates it — which is the point, because the alias is what type-checkers show callers.
    """
    import laterite

    alias = laterite.TypeClashMode
    literal_args = list(typing.get_args(alias))
    assert set(literal_args) == set(_authority()), (
        f"laterite.TypeClashMode = Literal{literal_args} has drifted from "
        f"TypeClashMode::ALL {_authority()}. Update the Literal — it cannot derive itself, "
        "so it must be kept in step by hand, and this test is what forces that."
    )


def test_merge_accepts_every_mode_and_rejects_a_bogus_one() -> None:
    """The set is not just declared — each value is a mode `merge` actually honours.

    Guards the failure where a copy lists a mode the engine dropped, or omits one it
    gained: every authority value must be accepted by the real API, and a value outside
    the set must be refused *for being an unknown mode*.

    Two DISTINCT valid deliveries of one project — `merge` takes positional varargs and
    rejects fewer than two sources before it ever looks at the mode, so a single file
    (or a list) would make this pass for the wrong reason. That trap is why the check
    below asserts on the error's *text*, not merely that something raised.
    """
    import laterite

    def _proj(pid: str) -> str:
        return "\r\n".join(
            [
                '"GROUP","PROJ"',
                '"HEADING","PROJ_ID"',
                '"UNIT",""',
                '"TYPE","ID"',
                f'"DATA","{pid}"',
                "",
            ]
        )

    a, b = (
        _proj("P1"),
        _proj("P1"),
    )  # same project id, no clashing column -> a clean merge
    for mode in _authority():
        try:
            laterite.merge(a, b, on_type_clash=mode)
        except Exception as e:
            # A clean two-source merge must not fail *because of the mode*. Any
            # "unknown mode" wording here means an authority value the engine rejects.
            # Not a pytest.raises() case: the no-exception path is the expected
            # outcome for every authority mode, this branch only guards a failure.
            assert (  # noqa: PT017
                "mode" not in str(e).lower() and "type-clash" not in str(e).lower()
            ), f"merge() rejected authority mode {mode!r}: {e}"

    with pytest.raises(Exception) as excinfo:  # noqa: PT011 - message asserted below
        laterite.merge(a, b, on_type_clash="definitely-not-a-mode")
    assert (
        "definitely-not-a-mode" in str(excinfo.value)
        or "mode" in str(excinfo.value).lower()
    ), f"a bogus mode must be refused for being unknown, got: {excinfo.value}"


def _walk_actions(parser: argparse.ArgumentParser) -> typing.Iterator[argparse.Action]:
    yield from parser._actions
    for action in parser._actions:
        if isinstance(action, argparse._SubParsersAction):
            for sub in action.choices.values():
                yield from _walk_actions(sub)
