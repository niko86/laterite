"""Render the `lat` verb table into the wiki's CLI tool page from the shipped
CLI guide — so the page can't drift from the tool (wiki-reliability plan, C4).

The wiki page `ags-wiki/tools/laterite-cli.md` used to hand-list the CLI
surface and it rotted the way API docs always do — by *omission and phantom*,
which the dead-`repo:`-ref and retired-term checks structurally cannot catch: it
still described the pre-#430 flat-flag interface (`--fix` / `--diff` /
`--emit-index`) long after those became the `fix` / `diff` / `certify`
subcommands, and never mentioned `read` / `pack` / `lock` / `excel` at all.

The single source of truth is `rust-packages/laterite-cli/README-cli.md`
— it *is* what `lat --readme` prints (embedded verbatim via `include_str!`), the
same guide the docs-site renders (`web/docs-site/scripts/gen_cli.py`). We parse
its `## Commands` block and render a table into a marked block on the wiki page.
Because the SSOT is a committed markdown file, this needs no Rust build, so the
faithfulness gate rides the cheap stdlib-only `wiki-lint` job (`--check`).

`--readme` is hand-maintained (include_str!, not derived from the clap tree), so
a verb could in principle be added to `Commands` without reaching the README.
The paired test (`tests/test_wiki_cli_faithful.py`) closes that by cross-checking
the parsed verbs against `cli.rs`'s `SUBCOMMANDS` const — the runtime verb list.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_README = _REPO / "rust-packages" / "laterite-cli" / "README-cli.md"
_CLI_RS = _REPO / "rust-packages" / "laterite-cli" / "src" / "cli.rs"
_PAGE = _REPO / "ags-wiki" / "tools" / "laterite-cli.md"

_BEGIN = (
    "<!-- generated:cli-verbs — DO NOT EDIT; source "
    "repo:rust-packages/laterite-cli/README-cli.md (== `lat --readme`); "
    "regenerate: uv run --no-sync python tools/gen_wiki_cli.py -->"
)
_END = "<!-- /generated:cli-verbs -->"
_BLOCK_RE = re.compile(
    r"<!-- generated:cli-verbs.*?<!-- /generated:cli-verbs -->", re.DOTALL
)


def parse_commands(readme: str) -> list[tuple[str, str, str]]:
    """(verb, args-signature, description) from README-cli.md's `## Commands`.

    A command line is `verb <placeholders…>  description`; the signature is the
    leading verb plus every `<…>` / `[…]` placeholder token, the description is
    the remaining prose — no reliance on the column alignment of the guide."""
    out: list[tuple[str, str, str]] = []
    in_block = False
    for ln in readme.splitlines():
        if ln.strip() == "## Commands":
            in_block = True
            continue
        if in_block and ln.startswith("## "):
            break
        if not in_block:
            continue
        toks = ln.split()
        if not toks:
            continue
        verb, rest = toks[0], toks[1:]
        i = 0
        while i < len(rest) and (rest[i].startswith("<") or rest[i].startswith("[")):
            i += 1
        out.append((verb, " ".join(rest[:i]), " ".join(rest[i:])))
    return out


def parse_subcommands(cli_rs: str) -> list[str]:
    """The `SUBCOMMANDS: &[&str]` const from cli.rs — the runtime verb list."""
    m = re.search(r"SUBCOMMANDS:\s*&\[&str\]\s*=\s*&\[(.*?)\];", cli_rs, re.DOTALL)
    if not m:
        raise SystemExit("cli.rs: SUBCOMMANDS const not found")
    return re.findall(r'"([^"]+)"', m.group(1))


def render_block(readme: str) -> str:
    rows = ["| Verb | Arguments | What it does |", "|---|---|---|"]
    for verb, args, desc in parse_commands(readme):
        sig = f"`{args}`" if args else "—"
        rows.append(f"| `{verb}` | {sig} | {desc.replace('|', chr(92) + '|')} |")
    return _BEGIN + "\n" + "\n".join(rows) + "\n" + _END


def splice(page: str, block: str) -> str:
    if not _BLOCK_RE.search(page):
        raise SystemExit(
            f"{_PAGE.name}: no <!-- generated:cli-verbs --> block to update"
        )
    return _BLOCK_RE.sub(lambda _m: block, page, count=1)


def extract_block(page: str) -> str:
    m = _BLOCK_RE.search(page)
    if not m:
        raise SystemExit(f"{_PAGE.name}: no generated:cli-verbs block found")
    return m.group(0)


def main(argv: list[str]) -> int:
    block = render_block(_README.read_text())
    page = _PAGE.read_text()
    if "--check" in argv:
        if extract_block(page) != block:
            print(
                "DRIFT: ags-wiki/tools/laterite-cli.md verb table is stale "
                "— run `uv run --no-sync python tools/gen_wiki_cli.py`",
                file=sys.stderr,
            )
            return 1
        print("gen_wiki_cli: CLI verb table is faithful to README-cli.md")
        return 0
    _PAGE.write_text(splice(page, block))
    print(f"wrote {_PAGE.relative_to(_REPO)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
