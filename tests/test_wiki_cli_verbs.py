"""`gen_wiki_cli.verb_drift()` has to actually catch a verb going missing.

The chain that keeps the wiki's CLI table honest is clap → `cli::SUBCOMMANDS` →
`README-cli.md` → the generated table. `census.rs` pins the first link and the
`--check` splice pins the last. The middle link — `--readme` is hand-maintained
via `include_str!`, not derived from clap — was credited in `gen_wiki_cli.py`'s
docstring to a paired pytest that has never existed, while `parse_subcommands()`
sat in the module unreferenced. It is wired into `main()` now.

A comparison written and never falsified is how the middle link came to be
missing in the first place, so both directions are exercised here rather than
asserting the current tree happens to agree.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


gwc = _load("gen_wiki_cli", REPO / "tools" / "gen_wiki_cli.py")

README = gwc._README.read_text(encoding="utf-8")
CLI_RS = gwc._CLI_RS.read_text(encoding="utf-8")


def test_the_shipped_guide_and_the_runtime_verb_list_agree() -> None:
    assert gwc.verb_drift(README, CLI_RS) is None


def test_a_verb_the_guide_omits_is_caught() -> None:
    """A verb ships and `lat --readme` never mentions it — the live risk."""
    verb = gwc.parse_subcommands(CLI_RS)[-1]
    stripped = "\n".join(
        ln for ln in README.splitlines() if not ln.strip().startswith(f"{verb} ")
    )
    drift = gwc.verb_drift(stripped, CLI_RS)
    assert drift and verb in drift, (
        f"dropping `{verb}` from README-cli.md's ## Commands went unnoticed"
    )


def test_a_verb_the_guide_invents_is_caught() -> None:
    """The other direction: the table would advertise what `lat` won't dispatch."""
    verb = gwc.parse_subcommands(CLI_RS)[-1]
    without = CLI_RS.replace(f'"{verb}",', "", 1)
    assert verb not in gwc.parse_subcommands(without), "mutation did not take"
    drift = gwc.verb_drift(README, without)
    assert drift and verb in drift, (
        f"README-cli.md documenting `{verb}` against a SUBCOMMANDS list without "
        "it went unnoticed"
    )
