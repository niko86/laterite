#!/usr/bin/env python3
"""Drop-in surface drift check: does `laterite.compat` still cover python-ags4's
public API?

The parity oracle (`tools/run_python_ags4_tests.sh` + `tools/check_parity.py`)
answers a different question — it runs python-ags4's own tests and enforces the
failing SET by identity, which proves BEHAVIOUR on the surface upstream happens
to test. It is blind to the failure mode this script exists for:

    upstream adds a public function -> nothing in their old test suite calls it
    -> parity stays green -> a user porting `from python_ags4 import AGS4` hits
       AttributeError on a name we never knew existed.

Nothing else notices that. `upstream-pin` (parity.yml) notices a VERSION move
but says nothing about what moved; parity notices behaviour on tested paths. So
this closes the surface half: enumerate upstream's public callables, compare
against `laterite.compat`, and fail on a gap that isn't already accounted for.

The contract is `compat-surface-gaps.json` — the deliberate non-mirrors, by
identity, each with the reason it is deliberate. Same shape and same reasoning
as `parity-known-failures.json`: a COUNT would go green on a swap (one gap
closed while another opened) and a bare list with no reasons decays into
folklore. A fixture entry that is no longer a gap is ALSO a failure — a stale
allowlist is how a contract quietly stops describing reality.

Deliberately AST-based, both sides: no import, so no built wheel, no pandas, no
`_laterite_native`. That is what lets this run in a cheap scheduled job next to
`upstream-pin` instead of behind a maturin build.

NOTE ON WHAT THIS DOES NOT CHECK: name coverage is not drop-in-ness. laterite
ships ONE flat module where upstream ships a package (`AGS4.py` / `check.py` /
`utils.py` / `ags4_cli.py` / `data/`), and there is no `python_ags4` import path
in the wheel at all. That is a permanent, documented structural difference (see
COMPAT.md and the README), not drift, so it is stated here and not gated.

Run:
    uv run --no-sync python tools/check_dropin_surface.py
    uv run --no-sync python tools/check_dropin_surface.py --update   # rewrite the fixture
"""

from __future__ import annotations

import argparse
import ast
import importlib.util
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
COMPAT = REPO / "packages" / "laterite" / "python" / "laterite" / "compat.py"
FIXTURE = REPO / "compat-surface-gaps.json"

# The upstream submodules whose public callables a drop-in is expected to cover.
# `ags4_cli` is excluded WHOLESALE (not gap-by-gap): it is a Click command group,
# and laterite ships `lat` — a different CLI with different output shapes — rather
# than mirroring it. Listing its commands as individual "gaps" would imply we
# intend to close them.
UPSTREAM_MODULES = ("AGS4", "check", "utils")
EXCLUDED_MODULES = {
    "ags4_cli": (
        "python-ags4's Click CLI. laterite ships `lat` instead — a standalone "
        "binary with its own JSON/NDJSON output shapes, not a command-level "
        "mirror. Not a gap to close; a deliberate divergence."
    ),
}


# Seed reasons for `--update`. Written per CATEGORY rather than per name because
# the categories are the real argument — 40-odd individually-worded entries would
# be 40 chances to say the same thing slightly differently and then drift.
def _seed_reason(module: str, name: str) -> str:
    if module == "check" and name.startswith(("rule_", "fyi_")):
        return (
            "python-ags4's implementation of one numbered AGS4 rule. laterite "
            "implements the rules in Rust (laterite-ags4-validator) and exposes "
            "the verdict through AGS4.check_file, which IS mirrored — a drop-in "
            "user never calls these directly. Listed by identity rather than "
            "excluded wholesale so that a NEW rule_* appearing upstream shows up "
            "here: that means a rule we may need to implement."
        )
    if module == "check":
        return (
            "Internal helper of python-ags4's checker. COMPAT.md maps only "
            "check.get_TRAN_AGS from this module as user-facing; the rest is "
            "machinery behind check_file, which laterite implements in Rust."
        )
    return "TODO: why is this deliberate?"


def public_api(path: Path) -> set[str]:
    """Public top-level callables (functions + classes) declared in a module.

    AST, not import: importing upstream drags pandas, and importing
    `laterite.compat` needs the compiled `_laterite_native`. Neither is worth a
    build for a name-level comparison.
    """
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    names = {
        node.name
        for node in tree.body
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef | ast.ClassDef)
        and not node.name.startswith("_")
    }
    # An explicit `__all__` is a stronger statement of intent than what happens
    # to be declared here, so honour it additively (re-exports count as covered).
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if not any(isinstance(t, ast.Name) and t.id == "__all__" for t in node.targets):
            continue
        if isinstance(node.value, ast.List | ast.Tuple):
            names |= {
                el.value
                for el in node.value.elts
                if isinstance(el, ast.Constant) and isinstance(el.value, str)
            }
    return names


def find_upstream(explicit: str | None) -> Path:
    if explicit:
        p = Path(explicit).resolve()
        if not p.is_dir():
            sys.exit(f"--upstream {p} is not a directory")
        return p
    # find_spec resolves a top-level package WITHOUT executing its __init__, so
    # this stays import-free even though it goes through the import machinery.
    spec = importlib.util.find_spec("python_ags4")
    if spec is None or not spec.submodule_search_locations:
        sys.exit(
            "python_ags4 not importable. It is a declared dev dependency — run "
            "`uv sync --group dev`, or pass --upstream <path to python_ags4/>."
        )
    return Path(next(iter(spec.submodule_search_locations)))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--upstream", help="path to the python_ags4 package directory")
    ap.add_argument(
        "--update",
        action="store_true",
        help="rewrite the fixture from the current diff (reasons must then be filled in)",
    )
    args = ap.parse_args()

    up_dir = find_upstream(args.upstream)
    ours = public_api(COMPAT)

    upstream: dict[str, set[str]] = {}
    for mod in UPSTREAM_MODULES:
        f = up_dir / f"{mod}.py"
        if not f.exists():
            sys.exit(
                f"upstream module {mod}.py not found in {up_dir} — python-ags4's "
                f"package layout changed, which is itself a drop-in concern. "
                f"Reconcile UPSTREAM_MODULES before trusting this check."
            )
        upstream[mod] = public_api(f)

    fixture = (
        json.loads(FIXTURE.read_text(encoding="utf-8")) if FIXTURE.exists() else {}
    )
    known = {
        (g["module"], g["name"]): g["reason"] for g in fixture.get("known_gaps", [])
    }

    gaps = [
        (mod, name)
        for mod, names in upstream.items()
        for name in sorted(names)
        if name not in ours
    ]

    print(f"upstream: {up_dir}")
    print(f"laterite.compat: {len(ours)} public callables")
    for mod, names in upstream.items():
        covered = len(names & ours)
        print(f"  python_ags4.{mod}: {covered}/{len(names)} covered")
    for mod, why in EXCLUDED_MODULES.items():
        print(f"  python_ags4.{mod}: EXCLUDED — {why.splitlines()[0]}")

    if args.update:
        FIXTURE.write_text(
            json.dumps(
                {
                    "_comment": (
                        "Deliberate non-mirrors of python-ags4's public API, by "
                        "identity. Enforced by tools/check_dropin_surface.py. A "
                        "new gap fails; so does an entry that is no longer a gap."
                    ),
                    "excluded_modules": EXCLUDED_MODULES,
                    "known_gaps": [
                        {
                            "module": m,
                            "name": n,
                            "reason": known.get((m, n)) or _seed_reason(m, n),
                        }
                        for m, n in gaps
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        print(f"\nwrote {FIXTURE.relative_to(REPO)} ({len(gaps)} gap(s))")
        return 0

    new_gaps = [(m, n) for m, n in gaps if (m, n) not in known]
    stale = [k for k in known if k not in set(gaps)]

    print(f"\ngaps: {len(gaps)} total, {len(known)} accounted for in {FIXTURE.name}")
    if new_gaps:
        print(
            f"\nNEW gaps ({len(new_gaps)}) — upstream has these, laterite.compat does not:"
        )
        for m, n in new_gaps:
            print(f"  - python_ags4.{m}.{n}")
        print(
            "\nEither mirror them in laterite.compat, or record each in "
            f"{FIXTURE.name} with the reason it is deliberate."
        )
    if stale:
        print(
            f"\nSTALE fixture entries ({len(stale)}) — recorded as gaps but no longer are:"
        )
        for m, n in stale:
            print(f"  - python_ags4.{m}.{n}")
        print(
            f"\nRemove them from {FIXTURE.name} — a stale allowlist stops describing reality."
        )

    if new_gaps or stale:
        return 1
    print("\ndrop-in surface OK: every unmirrored upstream name is accounted for.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
