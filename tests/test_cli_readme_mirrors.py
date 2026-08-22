"""Every launcher prints the SAME `lat --readme` (#509).

`tests/test_cli_readme_flags.py` holds `README-cli.md` to `cli.rs`, and it reads
`rust-packages/laterite-cli/README-cli.md` — the copy the binary embeds. The wheel
ships its own, `ts/cli.ts` now reads a third, and a gate that can only see one of
them guards a file rather than the claim `surfaces/cli.md` makes.

WHAT THE BLIND SPOT COST, measured when this file was written. The wheel's copy
had last been touched by a bulk tree sync in July while the authority moved on, so
`pip install laterite` shipped a guide that documented `--check-files` under
`## certify` — the exact line `test_cli_readme_flags.py` was written to prevent,
still reaching readers because that gate reads the authority and the wheel ships
the copy. It also predated #468 entirely: no `--warnings-as-errors`, and exit-code
prose the engine had stopped using.

So this is the gate that makes the OTHER gate's reach real: hold the mirrors
byte-identical, and everything `test_cli_readme_flags.py` proves about the
authority is true of every launcher rather than of one file.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _tool():
    """Import `tools/gen_cli_readme.py` — `tools/` is not a package.

    Same shape as tests/test_issue_tracker.py's loader, so there is one way to do
    this, and the mirror list is READ from the tool rather than restated here: a
    second copy of the list is the defect this module exists to catch, one level
    up.
    """
    path = REPO / "tools" / "gen_cli_readme.py"
    spec = importlib.util.spec_from_file_location("gen_cli_readme", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules["gen_cli_readme"] = module
    spec.loader.exec_module(module)
    return module


TOOL = _tool()


def test_there_are_mirrors_to_check() -> None:
    """Zero is a bad witness: an empty list would make every case below vacuous,
    and emptying it is exactly how a launcher stops being covered."""
    assert TOOL.MIRRORS, "no README mirrors declared — has a launcher lost its copy?"
    assert TOOL.AUTHORITY.exists(), f"no authority at {TOOL.AUTHORITY}"


@pytest.mark.parametrize(
    "mirror", list(TOOL.MIRRORS), ids=lambda p: str(p.relative_to(REPO))
)
def test_mirror_matches_the_authority(mirror: Path) -> None:
    assert mirror.exists(), (
        f"{mirror.relative_to(REPO)} is missing — {TOOL.MIRRORS[mirror]}. "
        "Run `uv run --no-project python tools/gen_cli_readme.py`."
    )
    assert mirror.read_text(encoding="utf-8") == TOOL.AUTHORITY.read_text(
        encoding="utf-8"
    ), (
        f"{mirror.relative_to(REPO)} has drifted from "
        f"{TOOL.AUTHORITY.relative_to(REPO)} — {TOOL.MIRRORS[mirror]}.\n"
        "Run `uv run --no-project python tools/gen_cli_readme.py`; edit the "
        "authority, never a mirror."
    )


def test_the_tool_agrees_with_the_comparison_above(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """`gen_cli_readme.py --check` is the command the failure message names, so it
    has to be a command that works. Asserted here rather than given its own CI
    step: a second workflow step in the same job would be a second reader of one
    truth, and this way the tool's own code path cannot rot unexercised.
    """
    assert TOOL.main(["--check"]) == 0, capsys.readouterr().out


def test_every_launcher_that_ships_a_guide_has_a_mirror() -> None:
    """The list is a thing to forget, so it is derived-checked rather than trusted.

    A `README-cli.md` under a launcher's own package directory is a shipped guide
    by construction. If one exists that the tool does not know about, it is a
    fourth copy nobody compares — which is the state this whole module exists to
    leave behind.
    """
    shipped = {
        REPO / "packages" / "laterite" / "python" / "laterite" / "README-cli.md",
        REPO / "rust-packages" / "laterite-node" / "README-cli.md",
    }
    unmirrored = shipped - set(TOOL.MIRRORS) - {TOOL.AUTHORITY}
    assert not unmirrored, (
        f"shipped guide(s) with no mirror entry: {sorted(map(str, unmirrored))} — "
        "add them to gen_cli_readme.MIRRORS or they drift unwatched"
    )
