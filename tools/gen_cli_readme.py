#!/usr/bin/env python3
"""One `lat --readme`, however you launched it (#509).

`surfaces/cli.md` opens with "It is the same tool however you launch it". The
guide each launcher prints is part of that claim, and it was not true.

THE AUTHORITY is `rust-packages/laterite-cli/README-cli.md` — `include_str!`d into
the binary, rendered into the wiki by `gen_wiki_cli.py` and into the docs site by
`web/docs-site/scripts/gen_cli.py`, and held to `cli.rs` by
`tests/test_cli_readme_flags.py`. Everything above reads that one file.

THE WHEEL SHIPS A SECOND COPY, and nothing was comparing them. `_cli.py`'s
`_print_readme()` reads `README-cli.md` from inside the installed package, so
`pip install laterite` had its own document — last updated by a bulk tree sync in
July, while the authority moved on. Measured when this file was written, the
wheel's copy:

  - documented `--check-files` under `## certify`, which `CertifyArgs` has never
    had. That is the EXACT line `tests/test_cli_readme_flags.py` exists to
    prevent, still shipping, because that gate reads the authority and the wheel
    ships the copy;
  - did not mention `--warnings-as-errors` at all (#468's verdict split);
  - described exit codes the engine had stopped using.

A gate that can only see one of two copies is not guarding the claim, it is
guarding a file. So the copies are generated here and compared here.

    gen_cli_readme.py           refresh every mirror from the authority
    gen_cli_readme.py --check   CI gate: compare and diff, write nothing

WHY MIRRORS AT ALL, rather than one file read at runtime. Each launcher ships as
its own artifact — a wheel, an npm package — and neither can reach into a Rust
crate directory at install time. The copy is a packaging fact; what was missing
is a generator that owns it and a gate that fails when it drifts.
"""

from __future__ import annotations

import argparse
import difflib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: The one a human edits. Everything else in this file is downstream of it.
AUTHORITY = ROOT / "rust-packages" / "laterite-cli" / "README-cli.md"

#: Where a launcher's own artifact needs its own copy, and why it cannot share.
MIRRORS: dict[Path, str] = {
    ROOT
    / "packages"
    / "laterite"
    / "python"
    / "laterite"
    / "README-cli.md": "the wheel — `_cli.py:_print_readme()` reads it from the installed package",
    ROOT
    / "rust-packages"
    / "laterite-node"
    / "README-cli.md": "the npm package — `ts/cli.ts` reads it beside `dist/`",
}


def _diff(authority: str, mirror: str, path: Path) -> str:
    return "".join(
        difflib.unified_diff(
            authority.splitlines(keepends=True),
            mirror.splitlines(keepends=True),
            fromfile=str(AUTHORITY.relative_to(ROOT)),
            tofile=str(path.relative_to(ROOT)),
        )
    )


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="compare only; exit 1 on drift and print the diff",
    )
    args = ap.parse_args(argv)

    if not AUTHORITY.exists():
        print(f"gen_cli_readme: no authority at {AUTHORITY}", file=sys.stderr)
        return 1
    text = AUTHORITY.read_text(encoding="utf-8")

    drifted: list[Path] = []
    for path, why in MIRRORS.items():
        rel = path.relative_to(ROOT)
        current = path.read_text(encoding="utf-8") if path.exists() else None
        if current == text:
            continue
        drifted.append(path)
        if args.check:
            print(f"gen_cli_readme: {rel} has DRIFTED from the authority ({why})")
            print(_diff(text, current or "", path) or "  (file is missing)")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
            print(f"gen_cli_readme: rewrote {rel}")

    # Printed on every run, pass or fail: a mirror list is a thing to forget, and
    # the count is what shows a launcher was added without one.
    print(
        f"gen_cli_readme: {len(MIRRORS)} mirror(s) of "
        f"{AUTHORITY.relative_to(ROOT)}; {len(drifted)} drifted"
    )
    if args.check and drifted:
        print(
            "gen_cli_readme: run `uv run --no-project python tools/gen_cli_readme.py` "
            "— never hand-edit a mirror, the authority is the file to change",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
