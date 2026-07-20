#!/usr/bin/env python3
"""Surface census — every launcher's own parser, diffed against ONE authority.

Phase 1 of the cross-surface output-value gate (`output/output-value-gate-plan.md`).

WHY THIS EXISTS
---------------
Every cross-surface gate we owned compared *knob names against another hand-list*:
`test_cross_surface_parity` (signatures), `test_modality_parity` (which I/O forms
exist), `test_wiki_cli_faithful` (the README's verbs vs a const in `cli.rs`). All of
them can pass while two surfaces do entirely different things — and one of them can
pass while a surface *does not implement the verb at all*.

That is not hypothetical. `lat merge` shipped in the native binary (#494) and never
reached the uvx or npx launchers. Three launchers of "one tool", and one of them
simply had no `merge`. Nothing failed, because the gates compared a hand-list to a
hand-list and both were equally wrong.

**A value-comparison gate can never catch this**: there is no output to compare when
the door does not exist. So the census comes first.

HOW IT WORKS
------------
Each launcher REFLECTS its own parser and dumps it as JSON — it never describes
itself from a list:

  * native  — `lat census`  → clap's `get_subcommands` / `get_arguments` /
              `get_possible_values`, on the same `Cli` struct that parses argv.
  * uvx     — `laterite._cli.census()` → argparse's `_actions`, on the same parser
              `main()` builds.
  * npx     — `lat census` (bin.mjs) → the keys of `HANDLERS`, which *is* the
              dispatch table (it used to be a Set beside a `switch` — two lists that
              could disagree, so a census reading either could lie about the other).

The native binary is the AUTHORITY. `surface-census.json` records what each launcher
answered, and any difference must be fixed or declared there with a reason and a
verdict.

HOW IT IS GATED WITHOUT NEW CI MINUTES
--------------------------------------
No single CI job builds all three launchers (python builds the wheel + `lat`; node
builds `dist/`). Rather than add a job, the **committed SSOT is the shared contract**
and each job pins the surfaces it already has:

  * python job — `tests/test_census_faithful.py` runs `--check`, which probes the
    launchers present and asserts each still answers exactly what the SSOT records,
    then that every divergence is declared.
  * node job   — `test/census.test.ts` asserts npx's live `census()` equals the
    SSOT's `cli-npx` entry.

So changing any launcher without regenerating the SSOT fails in that launcher's own
job, and the SSOT can never quietly drift from the authority.

USAGE
    python tools/gen_census.py            # probe every launcher, refresh SSOT + render
    python tools/gen_census.py --check    # CI: fail on undeclared drift
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SSOT = REPO / "surface-census.json"
RENDER = REPO / "ags-wiki" / "concepts" / "surface-census.md"

AUTHORITY = "cli-native"

_TARGET = REPO / "rust-packages" / "target"
_NPX = REPO / "rust-packages" / "laterite-node" / "bin.mjs"

#: `lat` ships from release (and CI's python job builds release), but a dev box
#: usually has debug too. Try release first, then debug — and crucially, take the
#: first that ANSWERS, not merely the first that exists: an old release binary
#: predating `census` treats the word as a filename and "validates" it, which would
#: silently leave the authority unavailable and disarm the whole gate.
_LAT_CANDIDATES = [_TARGET / "release" / "lat", _TARGET / "debug" / "lat"]


#: The census schema this generator understands. Bumped whenever a TABLE is added
#: (1: verbs · 2: + editions/fallback_edition · 3: + encodings · 4: + per-verb flags),
#: in lockstep with `CENSUS_VERSION` in
#: `commands/census.rs`, `_cli.py::census`, and `cli.ts::census`.
#:
#: A dump older than this is REFUSED, not read. A stale-but-answering launcher would
#: otherwise report a table it has never heard of as EMPTY — which is indistinguishable
#: from "no drift", i.e. a gate that quietly disarms itself. That is not hypothetical:
#: the release `lat` from one commit earlier answered `census` perfectly well and
#: reported no editions at all.
CENSUS_VERSION = 5


class StaleLauncher(Exception):
    """A launcher answered, but with an older census schema than we can trust."""

    def __init__(self, cmd: list[str], got: int) -> None:
        super().__init__(
            f"{cmd[0]} answers census_version {got}, this generator needs "
            f"{CENSUS_VERSION} — it is built from older sources. Rebuild it "
            f"(cargo build -p laterite-ags4-check; maturin develop; npm run build)."
        )


def probe(cmd: list[str]) -> dict | None:
    """Ask one launcher to describe itself.

    `None` if it cannot answer at all (not built). Raises [`StaleLauncher`] if it
    answers with an OLD schema — silence about a table is not the same as agreement
    about it, and conflating the two is how a gate goes green while blind.
    """
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=120, cwd=REPO)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0 or not out.stdout.strip():
        return None
    try:
        c = json.loads(out.stdout)
    except json.JSONDecodeError:
        return None
    if not isinstance(c, dict) or "verbs" not in c:
        return None
    got = c.get("census_version", 0)
    if got != CENSUS_VERSION:
        raise StaleLauncher(cmd, got)
    return c


def shape(census: dict) -> dict:
    """One launcher's answer, reduced to the closed tables the census compares.

    Every table here is an enumeration the toolchain used to hand-copy per surface.
    Adding one is how a whole class of drift gets closed at once.
    """
    return {
        # The doors. `census` itself is a hidden machine door, dropped by the diff.
        "verbs": sorted(v["verb"] for v in census.get("verbs", [])),
        # verb -> {flag: takes_value}. The flags each verb accepts, and — the
        # load-bearing half — whether each one eats the next token.
        #
        # A verb table alone cannot see this. `validate` is present on all three
        # launchers, so every earlier gate called it agreement while npx accepted
        # `--encoding`, `--dict` and `--index` on that very verb and honoured none of
        # them: it had ONE global valued-flag set and no per-verb notion of "accepted"
        # at all. A flag a launcher advertises and drops is worse than one it lacks —
        # the user gets a confident answer to a question they did not ask.
        "args": {
            v["verb"]: {a["name"]: bool(a["takes_value"]) for a in v.get("args", [])}
            for v in census.get("verbs", [])
        },
        # Accepted on every verb. Reported separately because the launchers declare
        # them in opposite directions — clap hangs them off the root, argparse copies
        # them into each subparser — and comparing the per-verb tables without
        # subtracting these first would report phantom drift on every verb.
        "global_args": {
            a["name"]: bool(a["takes_value"]) for a in census.get("global_args", [])
        },
        # verb flag -> the CLOSED VALUE SET the flag accepts (sorted). This is the
        # table #555 added: the args table above compares a flag's arity, not the
        # modes it accepts, so a launcher offering a DIFFERENT `--on-type-clash` set
        # was invisible — the same shape as the swallowed `--encoding`, one level in.
        #
        # Normalised, because the three parser frameworks report per-arg values
        # inconsistently and a raw compare would be pure false drift:
        #   * only `takes_value` flags — clap reports `['true','false']` for a bool
        #     switch (its possible-values), argparse reports nothing; a switch has no
        #     user-facing value enum, so both collapse to "no entry" here.
        #   * `--dict-version` excluded — its set is the `editions` table, compared
        #     there and ORDERED; clap doesn't enforce it as a closed arg-value set at
        #     all (it validates in the engine), so a per-arg copy is double-reported.
        # What survives today is `merge --on-type-clash`; a future value-flag joins
        # automatically. Sorted: a value SET has no meaningful order (unlike editions).
        "arg_values": {
            f"{v['verb']} {a['name']}": sorted(a["values"])
            for v in census.get("verbs", [])
            for a in v.get("args", [])
            if a.get("takes_value")
            and a.get("values")
            and a["name"] != "--dict-version"
        },
        # The dictionary editions this launcher accepts for --dict-version. ORDERED:
        # it is a sequence (oldest → newest), and the order is meaningful.
        "editions": list(census.get("editions", [])),
        # Which edition `auto` resolves to when TRAN_AGS is missing/unrecognised.
        "fallback_edition": census.get("fallback_edition", ""),
        # label -> the encoding this launcher resolves it to, or None if it refuses.
        # Each launcher answers through its OWN resolver, not the shared leaf — the
        # leaf was always right, and the bug lived in the wrappers above it.
        "encodings": dict(census.get("encodings", {})),
    }


def take_census() -> dict[str, dict]:
    """Every launcher that is built AND can answer, mapped to its tables."""
    live: dict[str, dict] = {}

    for lat in _LAT_CANDIDATES:
        if not lat.is_file():
            continue
        c = probe([str(lat), "census"])
        if c is not None:
            live[AUTHORITY] = shape(c)
            break

    c = probe([sys.executable, "-m", "laterite._cli", "census"])
    if c is not None:
        live["cli-uvx"] = shape(c)

    # bin.mjs imports dist/cli.mjs — only present once the node package is built.
    if (_NPX.parent / "dist" / "cli.mjs").is_file():
        c = probe(["node", str(_NPX), "census"])
        if c is not None:
            live["cli-npx"] = shape(c)

    return live


def divergences(surfaces: dict[str, dict]) -> list[dict]:
    """Every way a launcher's tables differ from the authority's."""
    auth = surfaces[AUTHORITY]
    found: list[dict] = []

    for name, s in surfaces.items():
        if name == AUTHORITY:
            continue

        # --- CLI verbs: a set. A missing door is the headline bug this caught.
        a_verbs = set(auth["verbs"]) - {"census"}
        s_verbs = set(s["verbs"]) - {"census"}
        found.extend(
            {
                "table": "cli-verbs",
                "surface": name,
                "key": key,
                "detail": f"`lat {key}` exists in {AUTHORITY} and is absent from {name}",
            }
            for key in sorted(a_verbs - s_verbs)
        )
        found.extend(
            {
                "table": "cli-verbs",
                "surface": name,
                "key": key,
                "detail": f"`lat {key}` exists in {name} and is absent from {AUTHORITY}",
            }
            for key in sorted(s_verbs - a_verbs)
        )

        # --- Per-verb flags. Only for verbs BOTH launchers have: a missing verb is
        # already reported above, and re-reporting each of its flags would bury the
        # one finding that matters under a dozen that follow from it.
        for verb in sorted(a_verbs & s_verbs):
            a_flags = auth["args"].get(verb, {})
            s_flags = s["args"].get(verb, {})
            for flag in sorted(set(a_flags) | set(s_flags)):
                if flag not in s_flags:
                    detail = f"`lat {verb} {flag}` exists in {AUTHORITY}, not in {name}"
                elif flag not in a_flags:
                    detail = f"`lat {verb} {flag}` exists in {name}, not in {AUTHORITY}"
                elif a_flags[flag] != s_flags[flag]:
                    # One reads `--flag value`, the other reads `--flag` and treats
                    # the value as a positional. Same spelling, different command line.
                    took = {True: "takes a value", False: "is a bare switch"}
                    detail = (
                        f"`lat {verb} {flag}`: it {took[s_flags[flag]]} on {name}, "
                        f"but {took[a_flags[flag]]} on {AUTHORITY}"
                    )
                else:
                    continue
                found.append(
                    {
                        "table": "args",
                        "surface": name,
                        "key": f"{verb} {flag}",
                        "detail": detail,
                    }
                )

        # --- The globals, compared once rather than per verb.
        for flag in sorted(set(auth["global_args"]) | set(s["global_args"])):
            if flag not in s["global_args"]:
                detail = f"global `{flag}` exists in {AUTHORITY}, not in {name}"
            elif flag not in auth["global_args"]:
                detail = f"global `{flag}` exists in {name}, not in {AUTHORITY}"
            else:
                continue
            found.append(
                {
                    "table": "args",
                    "surface": name,
                    "key": f"(global) {flag}",
                    "detail": detail,
                }
            )

        # --- Editions: an ordered sequence, compared whole. A launcher that accepts
        # a DIFFERENT SET is broken; one that accepts the same set in a different
        # ORDER is reporting from a hand-written list rather than DictVersion::ALL.
        if s["editions"] != auth["editions"]:
            found.append(
                {
                    "table": "editions",
                    "surface": name,
                    "key": "--dict-version",
                    "detail": (
                        f"{name} accepts {s['editions']}, "
                        f"{AUTHORITY} accepts {auth['editions']}"
                    ),
                }
            )
        if s["fallback_edition"] != auth["fallback_edition"]:
            found.append(
                {
                    "table": "editions",
                    "surface": name,
                    "key": "fallback_edition",
                    "detail": (
                        f"{name} resolves `auto` to {s['fallback_edition']!r}, "
                        f"{AUTHORITY} to {auth['fallback_edition']!r}"
                    ),
                }
            )

        # --- Per-flag value sets (#555). Compared as SETS: a launcher that accepts a
        # different set of `--on-type-clash` modes is broken the same way one that
        # swallows `--encoding` is — it answers a question the user did not ask. A
        # flag present on only one side is skipped here: that's already an `args`
        # finding (the flag itself diverged), and re-reporting its values would bury
        # the root under a follow-on.
        a_vals = auth.get("arg_values", {})
        s_vals = s.get("arg_values", {})
        found.extend(
            {
                "table": "arg-values",
                "surface": name,
                "key": key,
                "detail": (
                    f"`lat {key}` accepts {s_vals[key]} on {name}, "
                    f"{a_vals[key]} on {AUTHORITY}"
                ),
            }
            for key in sorted(set(a_vals) & set(s_vals))
            if a_vals[key] != s_vals[key]
        )

        # --- Encoding labels: per-label resolution. A launcher that answers UTF-8
        # where the authority answers None has re-added a silent fallback — the bug
        # that let a typo'd label hand back the wrong text with no error.
        for label in sorted(set(auth["encodings"]) | set(s["encodings"])):
            a_res = auth["encodings"].get(label, "<not probed>")
            s_res = s["encodings"].get(label, "<not probed>")
            if a_res != s_res:
                found.append(
                    {
                        "table": "encodings",
                        "surface": name,
                        "key": label,
                        "detail": (
                            f"--encoding {label!r}: {name} resolves it to {s_res!r}, "
                            f"{AUTHORITY} to {a_res!r}"
                        ),
                    }
                )

    return found


def render(ssot: dict) -> str:
    """The committed human view. Generated — never hand-edit."""
    surfaces = ssot["surfaces"]
    names = list(surfaces)
    auth_verbs = sorted(set(surfaces[AUTHORITY]["verbs"]) - {"census"})

    lines = [
        "---",
        "type: concept",
        "title: surface census",
        "status: drafted",
        "tags: [concept, architecture, api-parity]",
        "ags_editions: []",
        "repo_refs:",
        '  census: "repo:surface-census.json"',
        '  generator: "repo:tools/gen_census.py"',
        '  gate_python: "repo:tests/test_census_faithful.py"',
        '  gate_node: "repo:rust-packages/laterite-node/test/census.test.ts"',
        '  authority: "repo:rust-packages/laterite-ags4-check/src/commands/census.rs"',
        "related: [modality-register, crate-map, agent-first-cli-contract, parity-model, start-here, ags4-output-value-gate]",
        "sources: []",
        "---",
        "",
        "# surface census",
        "",
        "> **Generated** by `tools/gen_census.py` — do not hand-edit.",
        "> Gated by `tests/test_census_faithful.py` (native + uvx) and",
        "> `rust-packages/laterite-node/test/census.test.ts` (npx).",
        "",
        "## Definition",
        "",
        "`lat` is one tool behind three launchers — the native Rust binary, `uvx --from",
        "laterite lat`, and `npx laterite` — and they are contractually the same tool",
        "(#430). The census is what makes that claim checkable. Each launcher **reflects",
        "its own parser** (clap's `get_subcommands` for the binary, argparse's `_actions`",
        "for uvx, the `HANDLERS` dispatch table for npx) and dumps it as JSON; the native",
        "binary is the authority, and every other launcher is diffed against it.",
        "",
        "The point is what a *value*-comparison gate structurally cannot reach. Feed one",
        "file through every surface and compare the outputs, and you still learn nothing",
        "about a verb one launcher never implemented: **a door that does not exist has no",
        "output to compare.** `lat merge` shipped in the binary (#494) and reached neither",
        "other launcher; every cross-surface gate stayed green, because each compared one",
        "hand-list against another hand-list and both were equally wrong. The census is the",
        "first thing that could see it — and it did, on its first run.",
        "",
        "Reflection is the load-bearing part. A census that read a hand-written verb list",
        "would just be a fourth list to drift. Every name here comes from the structure",
        "that actually parses the command line.",
        "",
        "## CLI verbs",
        "",
        "| verb | " + " | ".join(names) + " |",
        "|---|" + "---|" * len(names),
    ]
    for v in auth_verbs:
        cells = ["✅" if v in surfaces[n]["verbs"] else "❌" for n in names]
        lines.append(f"| `{v}` | " + " | ".join(cells) + " |")

    lines += [
        "",
        "`census` itself is a hidden machine door on the binary — it is how the authority",
        "is read, not a verb the launchers must mirror, so it is excluded from the diff.",
        "",
        "## Dictionary editions",
        "",
        "Which editions each launcher accepts for `--dict-version`, and which one `auto`",
        "falls back to. The authority is **`DictVersion::ALL`**, generated by the reference",
        "leaf's `build.rs` from `ags_dictionary.json` — so bundling a new edition should",
        "reach every launcher at once.",
        "",
        "| surface | editions | `auto` → |",
        "|---|---|---|",
    ]
    for n in names:
        eds = ", ".join(f"`{e}`" for e in surfaces[n]["editions"]) or "—"
        lines.append(f"| {n} | {eds} | `{surfaces[n]['fallback_edition']}` |")
    lines += [
        "",
        "This set was hand-copied roughly nine times across the tree. The binary's own copy",
        "was the sharpest trap: its rejection **message** was generated from this list while",
        "its **match arms** were not, so bundling `4.3` would have shipped a `lat` that",
        "rejects `4.3` with a message advertising `4.3`. Every launcher now asks the",
        "generated `from_edition` / `DictVersion::ALL`; the npx launcher keeps no table at",
        "all and passes the string to the engine, which is the ideal end state — nothing to",
        "drift. (The web app still hand-lists them in four places; it is not a `lat`",
        "launcher, so the census cannot probe it — that convergence is its own change.)",
        "",
        "## Per-verb arguments",
        "",
        "Every flag and positional each verb accepts, and — the load-bearing half — whether",
        "each one **eats the next token**. A ✅ means the launcher's argument table for that",
        "verb is *identical* to the authority's, names and `takes_value` alike.",
        "",
        "| verb | " + " | ".join(names) + " |",
        "|---|" + "---|" * len(names),
    ]
    for v in auth_verbs:
        cells = []
        for n in names:
            a = surfaces[AUTHORITY]["args"].get(v, {})
            s = surfaces[n]["args"].get(v, {})
            cells.append(
                "—" if v not in surfaces[n]["verbs"] else ("✅" if s == a else "❌")
            )
        lines.append(f"| `{v}` | {' | '.join(cells)} |")

    glob = ", ".join(f"`{f}`" for f in sorted(surfaces[AUTHORITY]["global_args"]))
    lines += [
        "",
        f"Globals, accepted on every verb: {glob}. They are compared once rather than per",
        "verb, because the launchers declare them in opposite directions — clap hangs them",
        "off the root command, argparse can only reach a subcommand by *copying* them into",
        "each subparser — and a naive per-verb diff would report phantom drift on all of",
        "them.",
        "",
        "This is the table that hurt. A verb-name gate cannot see any of it: `validate` is",
        "present on all three launchers, so every gate we had called that agreement — while",
        "npx accepted `--encoding`, `--dict` and `--index` on that very verb and honoured",
        "**none** of them. It had one *global* valued-flag set and no per-verb notion of",
        "'accepted' at all, where clap declares flags per verb. So it silently swallowed any",
        "flag on any verb:",
        "",
        "* `--dict custom.ags` — accepted, ignored, validated against the *bundled*",
        "  dictionary, and reported **clean**. The binary and uvx both refuse it (exit 5).",
        "* `--index cert.ags.idx` — accepted and dropped entirely. The free `validate()`",
        "  takes `index` only because `ValidateOptions extends ReadOptions` and never reads",
        "  it, so the compiler was satisfied while the certificate went nowhere; a cert",
        "  minted for a *completely different file* changed nothing.",
        "* a **typo'd** flag — swallowed in silence, so `--no-warnigs` left the user",
        "  believing warnings were suppressed.",
        "",
        "A flag a launcher advertises and drops is worse than one it lacks: the user gets a",
        "confident answer to a question they did not ask. uvx, meanwhile, simply had no",
        "`--index` at all — so a script portable across the three launchers was not.",
        "",
        "`takes_value` earns its own column because getting it wrong **mis-parses the",
        "command line** rather than merely mis-reporting it: one launcher reads",
        "`--flag value`, another reads `--flag` and treats `value` as a positional. The",
        "authority's first implementation asked clap for `get_num_args()`, which is `None`",
        "unless an explicit arity was set — so every valued flag in the tool reported",
        "`takes_value: false`. The census's own key column was wrong before it ever ran a",
        "diff; it now asks the **action**, and a test pins it.",
        "",
        "## Encoding labels",
        "",
        "What each launcher makes of an `--encoding` label. The accepted set is not",
        "enumerable (every WHATWG label goes through `Encoding::for_label`), so the census",
        "compares **resolutions of a fixed probe list** instead.",
        "",
        "| label | " + " | ".join(names) + " |",
        "|---|" + "---|" * len(names),
    ]
    for label in sorted(surfaces[AUTHORITY]["encodings"]):
        cells = [f"`{surfaces[n]['encodings'].get(label) or '-'}`" for n in names]
        lines.append(f"| `{label}` | " + " | ".join(cells) + " |")
    lines += [
        "",
        "A `-` means the label is **refused**, and that is the point of the `cp1252x` row:",
        "an unknown label must resolve to nothing on every surface. Node used to answer",
        "`UTF-8` there — a silent fallback, which reads like leniency and behaves like",
        "corruption, because `C3 A9` decodes cleanly as `e-acute` in UTF-8 and as two",
        "characters in cp1252. A caller who fat-fingered a label got the wrong text back",
        "and a clean bill of health.",
        "",
        "Each launcher answers through its **own** resolver, never the shared parse leaf.",
        "The leaf was always right; the bug lived in the thin wrapper above it. A census",
        "that asked the leaf would have agreed with itself and seen nothing.",
        "",
        "`latin9` / `latin-9` are the two labels WHATWG does not know and the leaf now",
        "does. They used to be accepted only by a private table inside the `lat` binary, so",
        "`--encoding latin-9` worked on the CLI and was rejected by the Python library.",
        "",
        "## Declared divergences",
        "",
    ]
    div = ssot.get("divergences", [])
    if not div:
        lines.append("None — every launcher matches the authority on every table.")
    else:
        lines.append("| table | surface | key | verdict | reason |")
        lines.append("|---|---|---|---|---|")
        for d in div:
            lines.append(
                f"| {d['table']} | {d['surface']} | `{d['key']}` | "
                f"**{d['verdict']}** | {d['reason']} |"
            )
    lines += [
        "",
        "A divergence must be **fixed or declared**, and a declaration that stops being",
        "true fails the gate too — so the allowlist cannot quietly rot into a list of",
        "things we have stopped noticing. `known-bug` is a promise to fix and must cite",
        "an issue; `by-design` is a deliberate, reasoned difference.",
        "",
        "## Scope",
        "",
        "Closed so far: **CLI verbs**, **dictionary editions**, **per-verb arguments**,",
        "**encoding labels**. Still open: the severity / exit-code map.",
        "",
        "### What the census cannot see",
        "",
        "Agreement here is a **floor, not a ceiling** — the census compares what each",
        "launcher *declares*, and a launcher can declare a door truthfully and still do the",
        "wrong thing behind it. Three findings from this phase were invisible to every",
        "table above, and each one names a class:",
        "",
        "* **A verb that crashes.** `lat rules` was on all three launchers while *crashing*",
        "  on npx — it parsed `{schema_version, rules: [...]}` and iterated the object;",
        "  `JSON.parse` returns `any`, so the `Array<...>` annotation was an unchecked",
        "  assertion, and only `--json` had ever been run. The node gate therefore also",
        "  **runs** every verb it advertises.",
        "* **A flag that is honoured differently.** `--index` is declared identically on all",
        "  three launchers and skips the rule engine on different conditions: the binary",
        "  skips with warnings *on* (reporting the cert's recorded counts), while uvx and",
        "  npx skip only for an errors-only request. Same verdict, different work — a",
        "  performance difference, not a correctness one, but nothing above would show it.",
        "* **A result on the wrong stream.** `lat certify` printed its path to **stdout** on",
        "  the binary and uvx and to **stderr** on npx, so `CERT=$(lat certify f.ags)` worked",
        "  on two launchers and captured an empty string on the third. No gate we own",
        "  compares streams.",
        "",
        "Closing that class means comparing what the surfaces *produce*, not what they",
        "advertise. That gate has since landed: [[ags4-output-value-gate]] (#519-525) pushes a",
        "committed case manifest through every surface and compares the observed *values*, with",
        "the in-process Rust leaf as an authority column. See [[modality-register]] for the",
        "I/O-form axis the value gate does not cover.",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="CI: fail on undeclared drift")
    args = ap.parse_args()

    try:
        live = take_census()
    except StaleLauncher as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    if not args.check:
        # Writing the SSOT is a DEV action and needs the full picture — a partial
        # census would silently drop a launcher's row and look like agreement.
        missing = {AUTHORITY, "cli-uvx", "cli-npx"} - set(live)
        if missing:
            print(
                f"error: cannot write the census without {sorted(missing)} — build them "
                f"first (cargo build -p laterite-ags4-check; npm run build in "
                f"rust-packages/laterite-node)",
                file=sys.stderr,
            )
            return 2

        prior = json.loads(SSOT.read_text()) if SSOT.exists() else {}
        ssot = {
            "_comment": (
                "SSOT for the surface census. `surfaces` is GENERATED by "
                "tools/gen_census.py; `divergences` is HAND-MAINTAINED — every entry "
                "needs a verdict ('known-bug' = a promise to fix, with an issue; "
                "'by-design' = a documented, deliberate difference) and a reason."
            ),
            "authority": AUTHORITY,
            "surfaces": dict(sorted(live.items())),
            "divergences": prior.get("divergences", []),
        }
        SSOT.write_text(json.dumps(ssot, indent=2) + "\n")
        RENDER.parent.mkdir(parents=True, exist_ok=True)
        RENDER.write_text(render(ssot))
        print(f"wrote {SSOT.relative_to(REPO)} + {RENDER.relative_to(REPO)}")

        found = divergences(ssot["surfaces"])
        declared = {(d["table"], d["surface"], d["key"]) for d in ssot["divergences"]}
        if found:
            print(f"\n{len(found)} divergence(s):")
            for f in found:
                state = (
                    "declared"
                    if (f["table"], f["surface"], f["key"]) in declared
                    else "UNDECLARED"
                )
                print(f"  [{state}] {f['detail']}")
        return 0

    # --- --check ------------------------------------------------------------
    # Each job has only the launchers it builds. Pin the ones present against the
    # committed SSOT; the SSOT's other rows are pinned by the job that owns them.
    if not SSOT.exists():
        print(
            f"error: {SSOT.name} is missing — run tools/gen_census.py", file=sys.stderr
        )
        return 1
    ssot = json.loads(SSOT.read_text())
    recorded: dict[str, dict] = ssot["surfaces"]
    ok = True

    if AUTHORITY not in live:
        print(
            f"error: the authority ({AUTHORITY}) is not built — the census cannot be "
            f"checked against anything. Run `cargo build -p laterite-ags4-check`.",
            file=sys.stderr,
        )
        return 2

    for name, tables in live.items():
        if name not in recorded:
            ok = False
            print(
                f"UNRECORDED surface {name} — regenerate the census.", file=sys.stderr
            )
            continue
        if recorded[name] == tables:
            continue
        ok = False
        print(f"STALE census for {name}:", file=sys.stderr)
        was, now = set(recorded[name].get("verbs", [])), set(tables["verbs"])
        for v in sorted(now - was):
            print(
                f"  verbs: + {v} (the launcher has it; the census does not know)",
                file=sys.stderr,
            )
        for v in sorted(was - now):
            print(
                f"  verbs: - {v} (the census claims it; the launcher lacks it)",
                file=sys.stderr,
            )
        for key in ("editions", "fallback_edition", "encodings"):
            if recorded[name].get(key) != tables[key]:
                print(
                    f"  {key}: census says {recorded[name].get(key)!r}, "
                    f"the launcher says {tables[key]!r}",
                    file=sys.stderr,
                )
        print("  → run `uv run --no-sync python tools/gen_census.py`", file=sys.stderr)

    declared = {
        (d["table"], d["surface"], d["key"]) for d in ssot.get("divergences", [])
    }
    found = divergences(recorded)
    keys = {(f["table"], f["surface"], f["key"]) for f in found}

    undeclared = [
        f for f in found if (f["table"], f["surface"], f["key"]) not in declared
    ]
    if undeclared:
        ok = False
        print("CENSUS DRIFT — undeclared divergence(s):", file=sys.stderr)
        for f in undeclared:
            print(f"  [{f['table']}] {f['detail']}", file=sys.stderr)
        print(
            "\nFix the launcher, or declare it in surface-census.json with a verdict "
            "+ reason.",
            file=sys.stderr,
        )

    stale = sorted(declared - keys)
    if stale:
        ok = False
        print("STALE divergence(s) — declared but no longer real:", file=sys.stderr)
        for t, s, k in stale:
            print(
                f"  [{t}] {s}: {k} — remove it from surface-census.json",
                file=sys.stderr,
            )

    if RENDER.read_text() != render(ssot):
        ok = False
        print(
            f"{RENDER.relative_to(REPO)} is stale — run "
            f"`uv run --no-sync python tools/gen_census.py`",
            file=sys.stderr,
        )

    if ok:
        print(f"CENSUS CLEAN ({', '.join(sorted(live))})")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
