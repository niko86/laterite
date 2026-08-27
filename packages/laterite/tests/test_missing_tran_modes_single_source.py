"""The `--on-missing-tran` mode set has ONE source, and the copies are pinned to it.

The sibling of `test_type_clash_modes_single_source.py`, deliberately the same
shape. That file exists because `TypeClashMode`'s set was hand-copied across the
tree and nothing compared the copies back — a fourth mode would have reached some
of them through no path at all. This option starts with the authority in place
rather than acquiring one after a drift, and this file is what keeps it there.

`MissingTranMode::ALL` in `laterite-ags4-merge` is the authority — `reconcile`,
`error`, in that order (default first). `registry_missing_tran_modes()` exposes it
through the PyO3 bridge and `_cli.py`'s `_MISSING_TRAN_CHOICES` derives from it.
What is pinned here is the copy that structurally CANNOT self-derive: the
`Literal[...]` type alias, whose arguments must be literals, so it cannot consume
a runtime function.

These pass trivially today, with two values. They are written for the day a third
is added — the ticket names `omit` as the candidate — when the derived list moves
and the hand-list goes red.

The TypeScript copies (`laterite-node/ts/index.ts`, the wasm boundary's
`MergeOptions`) are the same shape one surface over; the census value-set parity
gate covers them, because this suite cannot reach them.
"""

from __future__ import annotations

import argparse
import typing

import pytest
from laterite import _cli
from laterite import _laterite_native as _native


def _authority() -> list[str]:
    """The one source: MissingTranMode::ALL, via the PyO3 bridge."""
    modes = list(_native.registry_missing_tran_modes())
    assert modes, (
        "registry_missing_tran_modes() returned nothing — the bridge is broken"
    )
    return modes


def test_the_native_list_is_the_merge_enum() -> None:
    """The bridge itself. Ordered (default `reconcile` first), as `ALL` is declared."""
    assert _authority() == ["reconcile", "error"], (
        "if this changed deliberately (a mode added/removed/reordered in "
        "laterite-ags4-merge's MissingTranMode::ALL), update this expectation — it is "
        "the one place the concrete set is asserted, so it should be a visible edit"
    )


def test_cli_choices_are_derived_not_hand_listed() -> None:
    """`_MISSING_TRAN_CHOICES` is the argparse choice list — what the CLI accepts."""
    assert tuple(_cli._MISSING_TRAN_CHOICES) == tuple(_authority()), (
        "_MISSING_TRAN_CHOICES has drifted from MissingTranMode::ALL. It must derive "
        "from registry_missing_tran_modes(), not be re-hard-coded."
    )


def test_the_merge_subparsers_on_missing_tran_offers_exactly_those_modes() -> None:
    """Reach into the built parser: the `--on-missing-tran` action's `choices`.

    `_MISSING_TRAN_CHOICES` being right is necessary but not sufficient — the
    argument has to actually use it. This walks the real parser argparse builds, so
    an edit that passes a different list to `add_argument` is caught.
    """
    parser = _cli._build_parser()
    found: list[list[str]] = [
        sorted(action.choices or [])
        for action in _walk_actions(parser)
        if "--on-missing-tran" in getattr(action, "option_strings", [])
    ]
    assert found, "no --on-missing-tran argument found in the parser"
    for choices in found:
        assert choices == sorted(_authority()), (
            f"--on-missing-tran offers {choices}, authority is {sorted(_authority())}"
        )


def test_the_public_literal_type_alias_matches_the_authority() -> None:
    """`laterite.MissingTranMode` is a `Literal[...]` — it cannot self-derive.

    The one genuine hand-copy on this surface, and the one this gate exists for: it
    is what a type-checker shows callers, so it going stale is a wrong answer given
    to every editor in the ecosystem while every runtime path stays correct.
    """
    import laterite

    literal_args = list(typing.get_args(laterite.MissingTranMode))
    assert set(literal_args) == set(_authority()), (
        f"laterite.MissingTranMode = Literal{literal_args} has drifted from "
        f"MissingTranMode::ALL {_authority()}. Update the Literal — it cannot derive "
        "itself, so it must be kept in step by hand, and this test forces that."
    )


def test_merge_accepts_every_mode_and_rejects_a_bogus_one() -> None:
    """The set is declared AND honoured: each value is a mode `merge` really takes.

    Two DISTINCT valid deliveries with no TRAN, so `reconcile` and `error` both
    succeed — the option only bites when the inputs carry transmissions, and this
    test is about the VOCABULARY, not the behaviour (that is the Rust facade's
    `on_missing_tran_*` tests). A bogus value must be refused *for being an unknown
    mode*, which is why the message is asserted rather than merely that it raised.
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

    a, b = _proj("P1"), _proj("P1")
    for mode in _authority():
        try:
            laterite.merge(a, b, on_missing_tran=mode)
        except Exception as e:
            # Not a pytest.raises() case: the no-exception path is the expected
            # outcome for every authority mode, so this branch only guards a
            # failure — an authority value the engine turns round and rejects.
            assert "unknown on_missing_tran" not in str(e), (  # noqa: PT017
                f"merge() rejected authority mode {mode!r}: {e}"
            )

    with pytest.raises(Exception) as excinfo:  # noqa: PT011 - message asserted below
        laterite.merge(a, b, on_missing_tran="definitely-not-a-mode")
    assert "definitely-not-a-mode" in str(excinfo.value), (
        f"a bogus mode must be refused for being unknown, got: {excinfo.value}"
    )


def _walk_actions(parser: argparse.ArgumentParser) -> typing.Iterator[argparse.Action]:
    yield from parser._actions
    for action in parser._actions:
        if isinstance(action, argparse._SubParsersAction):
            for sub in action.choices.values():
                yield from _walk_actions(sub)
