"""Free-function ↔ fluent-method **argument parity**.

A behavioural knob added to a free function (``validate`` / ``fix`` / ``diff`` /
``to_excel``) must also exist on its chained ``Ags4File`` counterpart — and with
the same default and annotation — so the two can't silently drift. This is the
``free`` > ``chained`` root-cause class behind several #294 gaps (e.g. ``encoding``
once lived on free ``validate`` only); it currently holds *by hand*, with nothing
to stop a future knob landing on one side and being forgotten on the other.

This is a **drift-gate**, the same family as ``test_typed_choices`` (a ``Literal``
vs its runtime set) and ``test_pyi_stubs_match_generator`` (generated vs committed):
it compares two representations of one contract and fails when they diverge. Each
pair's ``free_only`` / ``chained_only`` sets are the *spec* of the intentional
differences — the input selectors each side resolves differently (``source`` /
``text`` / ``a`` / ``b`` vs the handle / ``other``) and the deliberately free-only
I/O knobs (``in_place`` / ``out``). They get their own hygiene check so the
allowlist can't quietly rot.
"""

from __future__ import annotations

import inspect

import laterite as L
import pytest

# free fn, chained method, free-only params, chained-only params.
# A "free-only" / "chained-only" param is an input/output selector or a
# deliberately one-sided I/O knob; EVERYTHING ELSE is a behavioural knob that must
# be identical on both sides. (`to_excel` carries `path` on both with different
# roles — input selector on free, output destination on chained — so it is listed
# on both; that's fine, both are I/O, not behavioural.)
_PAIRS = {
    "validate": (L.validate, L.Ags4File.validate, {"source", "text"}, set()),
    "fix": (
        L.fix,
        L.Ags4File.fix,
        {"source", "path", "text", "data", "in_place", "out"},
        set(),
    ),
    "diff": (L.diff, L.Ags4File.diff, {"a", "b"}, {"other"}),
    "to_excel": (
        L.to_excel,
        L.Ags4File.to_excel,
        {"source", "output", "path", "text", "data"},
        {"path"},
    ),
}


def _params(fn) -> dict[str, inspect.Parameter]:
    """Named, non-variadic parameters of ``fn`` (drops ``self`` and ``*args`` / ``**kwargs``)."""
    return {
        n: p
        for n, p in inspect.signature(fn).parameters.items()
        if n != "self" and p.kind not in (p.VAR_POSITIONAL, p.VAR_KEYWORD)
    }


def _behavioural(fn, drop: set[str]) -> dict[str, inspect.Parameter]:
    return {n: p for n, p in _params(fn).items() if n not in drop}


@pytest.mark.parametrize("name", _PAIRS)
def test_behavioural_knob_names_match(name):
    """The behavioural-knob set is identical free ↔ chained, after removing each
    side's declared input/IO params."""
    free_fn, chained_fn, free_only, chained_only = _PAIRS[name]
    free = set(_behavioural(free_fn, free_only))
    chained = set(_behavioural(chained_fn, chained_only))
    assert free == chained, (
        f"{name}: behavioural-knob drift — free-only {sorted(free - chained)}, "
        f"chained-only {sorted(chained - free)} "
        f"(put it on BOTH the free fn and Ags4File.{name}, or add it to the "
        f"_PAIRS allowlist if the difference is intentional)"
    )


@pytest.mark.parametrize("name", _PAIRS)
def test_shared_knobs_have_matching_defaults_and_annotations(name):
    """A shared knob carries the SAME default and annotation on both sides — so
    ``encoding=None`` on free can't drift to ``encoding="utf-8"`` on chained."""
    free_fn, chained_fn, free_only, chained_only = _PAIRS[name]
    free = _behavioural(free_fn, free_only)
    chained = _behavioural(chained_fn, chained_only)
    for knob in free.keys() & chained.keys():
        assert free[knob].default == chained[knob].default, (
            f"{name}.{knob}: default differs — free={free[knob].default!r} "
            f"chained={chained[knob].default!r}"
        )
        assert free[knob].annotation == chained[knob].annotation, (
            f"{name}.{knob}: annotation differs — free={free[knob].annotation!r} "
            f"chained={chained[knob].annotation!r}"
        )


def test_allowlist_entries_exist():
    """Hygiene on ``_PAIRS`` itself: every declared one-sided param must exist on
    its own side, so a renamed/removed param can't leave a stale entry silently
    masking a real knob."""
    for name, (free_fn, chained_fn, free_only, chained_only) in _PAIRS.items():
        free, chained = set(_params(free_fn)), set(_params(chained_fn))
        assert free_only <= free, (
            f"{name}: stale free-only entry {sorted(free_only - free)}"
        )
        assert chained_only <= chained, (
            f"{name}: stale chained-only entry {sorted(chained_only - chained)}"
        )
