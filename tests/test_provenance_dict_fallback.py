"""#568 Phase 7 — the PROVENANCE `--dict` fallback claim, enforced BOTH ways.

`rust-packages/laterite-ags4-validator/data/PROVENANCE.md` states the licence
retreat from the bundled ©AGS dictionary: a consumer who cannot rely on the
embedded copy can supply their own at validation time via the runtime `--dict`
custom-dictionary override. That claim was once FALSE — the flag parsed the file
and then *refused* (`external --dict override is not implemented`, O-28) — while a
sibling document asserted the capability was available, and nothing compared the
two. #568 made it real across all four surfaces.

This gate is the missing comparison, a two-way pin so the claim cannot silently
un-become true again:

1. the `<!-- dict-fallback-claim -->` sentinel + the affirming paragraph are
   present in PROVENANCE.md (remove the claim → this fails);
2. `lat validate --dict <custom-dict> <delivery>` does NOT exit 5 — i.e. the flag
   actually validates against the supplied dictionary instead of refusing it
   (regress the capability → this fails). A control proves exit 5 is still the
   live refusal code, so the `!= 5` assertion is never vacuous.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_PROVENANCE = _REPO / "rust-packages/laterite-ags4-validator/data/PROVENANCE.md"
_FIXTURES = _REPO / "rust-packages/laterite-ags4-validator/tests/fixtures/custom_dict"
_DICT = _FIXTURES / "xtra.dict.ags"
_DELIVERY = _FIXTURES / "delivery_with_xtra.ags"

_BAD_DICT_EXIT = (
    5  # the CLI's BadDict refusal code — what --dict USED to always return.
)


def test_provenance_states_the_dict_fallback_exists() -> None:
    """Half one of the pin: the document still makes the claim."""
    text = _PROVENANCE.read_text(encoding="utf-8")
    assert "<!-- dict-fallback-claim -->" in text, (
        "the `<!-- dict-fallback-claim -->` sentinel is missing from PROVENANCE.md — "
        "the two-way pin lost the half that states the --dict fallback exists"
    )
    assert "--dict" in text and "now exists" in text, (
        "PROVENANCE.md must state the runtime --dict custom-dictionary fallback now "
        "exists (#568); the licence risk position rests on it"
    )


def _lat_validate(*args: str) -> int:
    """Run `lat validate <args...>` and return its exit code.

    Prefers the native `lat` binary the PROVENANCE note names; falls back to the
    in-process uvx launcher (`laterite._cli.main`, the same engine + the same
    `--dict` codepath) so the claim is pinned even where the Rust binary is not
    built. It deliberately never *skips*: a claim with no test that runs is exactly
    the gap #568 closed.
    """
    for prof in ("release", "debug"):
        native = _REPO / "rust-packages" / "target" / prof / "lat"
        if native.is_file():
            return subprocess.run(
                [str(native), "validate", *args],
                capture_output=True,
            ).returncode
    from laterite import _cli

    return _cli.main(["validate", *args])


def test_lat_validate_dict_does_not_refuse() -> None:
    """Half two: `lat validate --dict <dict> <delivery>` no longer refuses.

    The delivery's bespoke `XTRA` group is defined ONLY by the custom dictionary, so
    a working overlay validates it (the file still carries incidental non-XTRA
    findings, so the code is `1`, not `0` — the point is it is NOT the `5` the flag
    used to always return).
    """
    rc = _lat_validate("--dict", str(_DICT), str(_DELIVERY))
    assert rc != _BAD_DICT_EXIT, (
        f"`lat validate --dict` exited {_BAD_DICT_EXIT} (BadDict) — the custom-"
        f"dictionary fallback PROVENANCE.md promises has regressed (O-28 / #568)"
    )


def test_bad_dict_still_exits_5_so_the_pin_is_not_vacuous() -> None:
    """The control: exit 5 remains the live refusal signal. A forced base
    (`--dict-version`) contradicts a full replacement (`--dict-replace`), which every
    surface rejects with BadDict — so `!= 5` above genuinely means "did not refuse",
    not "5 is unreachable"."""
    rc = _lat_validate(
        "--dict", str(_DICT), "--dict-replace", "--dict-version", "4.2", str(_DELIVERY)
    )
    assert rc == _BAD_DICT_EXIT, (
        f"the --dict-replace + --dict-version contradiction should exit "
        f"{_BAD_DICT_EXIT} (BadDict); got {rc} — exit 5 is no longer the refusal "
        f"code, so the fallback pin above is vacuous"
    )
