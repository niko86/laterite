"""A flag documented under a `lat` verb must exist on that verb.

`README-cli.md` listed `--check-files` under `## certify`. `CertifyArgs` has no
such field, and `commands/certify.rs` says so deliberately: a stale on-disk `FILE/`
tree could still read as valid against an unchanged `.ags.idx`, so certify refuses
the flag rather than recording a check it cannot stand behind. `--check-files`
lives on `ValidateArgs` alone.

This is the most-travelled prose in the repo. The file is `include_str!`d into the
binary and printed verbatim by `lat --readme`, `tools/gen_wiki_cli.py` renders its
`## Commands` block into the wiki, and `web/docs-site/scripts/gen_cli.py` renders
it into the docs site. One wrong line reaches three surfaces.

`gen_wiki_cli.py` already pins the *verbs* both ways (README ↔ `cli::SUBCOMMANDS`,
which `census.rs` pins to clap). Flags were the layer under that, unchecked — a
verb cannot go missing, but everything written about it could.

**What "exists on that verb" means.** clap's derive is the authority: a field with
`#[arg(long)]` becomes `--kebab-case`, `#[arg(long = "x")]` becomes `--x`, and
`#[command(flatten)]` pulls in a shared struct's flags (that is how `--dict-version`
reaches nine verbs from one `DictArgs`). Flags on `Cli` itself — `--quiet` — are
global and allowed anywhere.

**Only lines that START with the flag count as documenting it.** The `## excel`
section mentions `--no-default-features` mid-sentence — a cargo flag, not a `lat`
one. Matching flags anywhere in the section made that the sole false positive; the
README's own layout puts a documented flag first on its line.
"""

from __future__ import annotations

import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_RS = REPO / "rust-packages" / "laterite-cli" / "src" / "cli.rs"
README = REPO / "rust-packages" / "laterite-cli" / "README-cli.md"

_STRUCT = re.compile(r"pub struct (\w+)\s*\{(.*?)\n\}", re.S)
_FIELD = re.compile(r"((?:#\[[^\]]*\]\s*)*)pub (\w+)\s*:\s*([^,]+),", re.S)
_LONG_NAMED = re.compile(r'long\s*=\s*"([\w-]+)"')
_COMMANDS = re.compile(r"pub enum Commands\s*\{(.*?)\n\}", re.S)
_VARIANT = re.compile(r"^\s+(\w+)\((\w+Args)\),", re.M)
_SECTION = re.compile(r"^## (\w+)[^\n]*\n(.*?)(?=^## |\Z)", re.S | re.M)
#: A documented flag opens its line, optionally after a short alias.
_DOC_FLAG = re.compile(r"^\s+(?:-\w,\s*)?--([\w-]+)", re.M)

#: clap supplies these; they are never fields.
_IMPLICIT = frozenset({"help", "version"})


def _parse() -> tuple[dict[str, set[str]], dict[str, list[str]]]:
    """(struct -> its own long flags, struct -> the structs it flattens in)."""
    src = CLI_RS.read_text(encoding="utf-8")
    own: dict[str, set[str]] = {}
    flattened: dict[str, list[str]] = {}
    for m in _STRUCT.finditer(src):
        name, body = m.group(1), m.group(2)
        longs: set[str] = set()
        flats: list[str] = []
        for attrs, field, ty in _FIELD.findall(body):
            if "command(flatten)" in attrs:
                flats.append(ty.strip().removeprefix("Option<").rstrip(">"))
                continue
            named = _LONG_NAMED.search(attrs)
            if named:
                longs.add(named.group(1))
            elif re.search(r"\blong\b", attrs):
                longs.add(field.replace("_", "-"))
        own[name], flattened[name] = longs, flats
    return own, flattened


def _flags_of(struct: str, own, flattened, seen=None) -> set[str]:
    seen = seen or set()
    if struct in seen or struct not in own:
        return set()
    seen.add(struct)
    out = set(own[struct])
    for child in flattened.get(struct, []):
        out |= _flags_of(child, own, flattened, seen)
    return out


def test_every_documented_flag_exists_on_its_verb() -> None:
    """Falsify by documenting a flag under the wrong verb, or by removing one.

    Putting `--check-files` back under `## certify` must fail; so must deleting
    `--dict-replace` from `DictArgs` while the README still lists it.
    """
    own, flattened = _parse()
    variants = _COMMANDS.search(CLI_RS.read_text(encoding="utf-8"))
    assert variants, "no `pub enum Commands` in cli.rs — the CLI was restructured"
    verbs = {v.lower(): args for v, args in _VARIANT.findall(variants.group(1))}
    assert "certify" in verbs, f"verb parse produced {sorted(verbs)}"

    global_flags = _flags_of("Cli", own, flattened) | _IMPLICIT

    checked = 0
    bad: list[str] = []
    for section in _SECTION.finditer(README.read_text(encoding="utf-8")):
        verb, body = section.group(1).lower(), section.group(2)
        if verb not in verbs:
            continue  # `## Dictionary auto-selection` and friends
        allowed = _flags_of(verbs[verb], own, flattened) | global_flags
        for flag in sorted(set(_DOC_FLAG.findall(body))):
            checked += 1
            if flag not in allowed:
                bad.append(f"lat {verb} --{flag}  (not on {verbs[verb]})")

    assert checked > 20, (
        f"only {checked} documented flags found — the README layout changed and "
        "this gate has stopped reading it"
    )
    assert not bad, (
        "README-cli.md documents flags that the verb does not accept — and this "
        "file is what `lat --readme` prints:\n  " + "\n  ".join(bad)
    )
