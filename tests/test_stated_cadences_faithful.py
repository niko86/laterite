"""Prose that states a workflow's cadence must agree with that workflow's cron.

Sibling to `test_vendored_authority_faithful.py`, and the same failure shape one
subject over: **the cron is the fact; every sentence naming a cadence is a claim
about it.** Nothing compared the two, and the claims drifted twice —

  #311  `cross-surface-parity.md` + `surfaces/index.md` called the 6-surface
        matrix report **weekly**. Weekly is `parity.yml`, a different workflow in
        a different repo that runs no matrix at all.
  #312  the same conflation, restated in four Rust doc comments.

Neither page named a workflow file, and neither lived under `ags-wiki/`, so the
wiki lint could not have caught either one.

TWO CHECKS, and the second is the one that survives a new page being written.

  Annotated blocks   a block carrying `cadence: <id>` must contain exactly the
                     cadence words its markers derive — no more, no fewer. Both
                     directions matter: a missing word means the prose dropped or
                     contradicts the claim, an extra one means a second, unstated
                     claim rode along in the same sentence.
  The tripwire       a block that names a known workflow (by filename OR by a
                     declared alias) and states a cadence, but carries no marker,
                     is an unannotated claim. This is what catches the sentence
                     nobody thought to annotate.

WHERE THE AUTHORITY LIVES decides which half resolves it. `parity.yml` is in this
tree, so `_IN_REPO` points at the file and the cron is read from it directly.
`compliance-report.yml` and `compliance.yml` are in the dev satellite, invisible
to this repo's CI, so `external-authorities.json` mirrors them — and that mirror
is itself reconciled against the real files by `tools/check_external_authorities.py`
running in THAT repo. Without the far-side job this test would be #549's Shape 1:
enforcing a proxy for a promise, with nothing comparing the proxy back.

The derived word is computed from the cron rather than written down beside it.
A hand-typed "monthly" in the mirror would just be the same drift one file over.
"""

from __future__ import annotations

import importlib.util
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from types import ModuleType

_REPO = Path(__file__).resolve().parent.parent
_MIRROR = _REPO / "external-authorities.json"


def _load_checker() -> ModuleType:
    """The far-side reconciler, for its workflow parser.

    Imported rather than reimplemented: this test reads `on:` blocks in THIS
    tree and that script reads them in the satellite's, and two parsers
    disagreeing about what a workflow says is the exact failure both exist to
    prevent. Loaded by path, the idiom `test_check_changelog.py` established.
    """
    spec = importlib.util.spec_from_file_location(
        "check_external_authorities", _REPO / "tools" / "check_external_authorities.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_external_authorities"] = mod
    spec.loader.exec_module(mod)
    return mod


_cx = _load_checker()


@dataclass(frozen=True)
class _InRepoSpec:
    """A workflow in THIS tree whose cadence prose asserts."""

    path: str
    form: str
    aliases: tuple[str, ...]
    self_identifies: bool


#: Authorities in THIS tree. Deliberately not in external-authorities.json: that
#: file is for values CI cannot read, and mirroring a file sitting right here
#: would create a proxy to drift from a promise that was already readable.
_IN_REPO: dict[str, _InRepoSpec] = {
    "parity": _InRepoSpec(
        path=".github/workflows/parity.yml",
        form="cron",
        # Phrases that DENOTE the workflow, not the domain word. Bare "parity"
        # was tried and measured: it flags `strat-forge-…md` ("the per-PR
        # divergence-lock gate") and `laterite-ags4-corpus-qa.md`
        # ("parity-cross-check vs python-ags4"), neither of which says anything
        # about a schedule. An alias that matches the subject matter rather than
        # the workflow trains people to exempt, which is how a tripwire dies.
        aliases=("parity.yml", "parity oracle", "parity job", "oracle run"),
        self_identifies=True,
    ),
    # One workflow, two cadences: the cron above AND a `pull_request` trigger. Its
    # own header states both, one line apart ("Weekly rather than monthly…" then
    # "Per-PR regression gate (#190)"), so an authority is a (workflow, trigger)
    # pair rather than a workflow — otherwise the second claim reads as an
    # unaccounted word in the first one's block.
    "parity-pr": _InRepoSpec(
        path=".github/workflows/parity.yml",
        form="trigger",
        aliases=("parity.yml",),
        self_identifies=True,
    ),
    # Resolvable but NEVER matched, and that asymmetry is the point: an authority
    # serves two roles — resolving a marker, and identifying a claim for the
    # tripwire — and this one earns its place on the first alone.
    # `concepts/docs-site.md` says "per-PR the `.sql` files are include-checked
    # only, that tree's monthly compliance-report.yml runs them live": one
    # sentence, two workflows, and the per-PR half is this repo's own gate.
    #
    # Empty aliases, measured rather than assumed: `"ci.yml"` as one pulls in 10
    # further blocks across ci/nightly/e2e/release, all describing THIS repo's
    # own gates. They are real claims, but this gate's subject is the
    # parity/compliance conflation, and a tripwire that arrives demanding 10
    # annotations unrelated to the drift it was built for is one people learn to
    # exempt. Give it aliases the day a ci.yml cadence claim actually rots.
    "ci": _InRepoSpec(
        path=".github/workflows/ci.yml",
        form="trigger",
        aliases=(),
        # Off for the same reason the aliases are empty — otherwise every cadence
        # word in ci.yml's own 76 KB of comments becomes a claim to annotate.
        self_identifies=False,
    ),
}

_SUFFIXES = {".md", ".rs", ".py", ".yml", ".yaml"}

#: Files whose cadence words are not claims about a workflow's schedule.
#:
#: The first four are the gate's own parts: they carry the grammar's examples,
#: and a scanner that parsed its own documentation would report on prose that is
#: *about* markers rather than making claims. lint.py skips `.bootstrap/` and
#: `templates/` for the same reason — a template is documentation of a grammar,
#: and cannot be read by it.
#:
#: The wiki page costs something real and is listed anyway: its terminology table
#: states both cadences ("weekly (Sun 03:00)", "monthly (1st, 04:00)"), and that
#: is exactly the kind of row that rots. Nothing here holds it. Carrying the
#: page would mean exempting its grammar section line by line, and an exemption
#: block that large is indistinguishable from switching the gate off.
#:
#: The last three are append-only HISTORY. `check_ext_drift.py` already carved
#: `log.md` out on exactly this argument — an entry narrating a past drift spells
#: out the very claim it reports dead, so scanning it re-flags that drift forever.
#: A changelog entry describing #311 is a record of the correction, not a
#: restatement of the error.
_NOT_SCANNED = frozenset(
    {
        "tests/test_stated_cadences_faithful.py",
        "tools/check_external_authorities.py",
        "external-authorities.json",
        "ags-wiki/tools/stated-cadences-faithful.md",
        "CHANGELOG.md",
        "changelog.json",
        "ags-wiki/log.md",
    }
)

#: Seed list, extendable — same posture as lint.py's A2 retired terms. "nightly"
#: is deliberately absent: it is both a cadence and the name of a workflow in
#: this repo, so it cannot be scanned as one without meaning the other.
_CADENCE_WORDS = ("daily", "weekly", "monthly", "per-PR")
_WORD_RES = {
    w: re.compile(rf"(?<![\w-]){re.escape(w)}(?![\w-])", re.I) for w in _CADENCE_WORDS
}

#: `cadence: <id>` inside whatever comment syntax the file already uses — an HTML
#: comment in Markdown (invisible when rendered, and docs-site pages are product
#: pages), `#` in Python/YAML, `//!` in Rust, `%%` inside a mermaid fence.
_MARKER = re.compile(r"cadence:\s*([A-Za-z0-9_.-]+)")

#: Line-scoped exemption. It borrows A11's WORD and its line-scoping, and its
#: generic/specific SHAPE — but the specific spelling is new: A11's is
#: `<!-- retired: TERM -->` naming a retired term, and there is no A11 form that
#: names a word to exempt. Do not read this as "the same two forms".
#:
#: Line- rather than block-scoped because the real cases make a live claim and
#: narrate a superseded one in the same breath.
#:
#: The specific form is what does the work, which building this proved rather
#: than predicted. All THREE exemptions this tree wants put the live word and the
#: dead one on ONE line — `parity.yml`'s "Weekly rather than monthly since the
#: dropin-surface job joined", `wiki-ext-drift.yml`'s note about parity's old
#: slot, and `oracle-drift-pin.md`'s "weekly since 2026-07-24, monthly when this
#: was written". The generic form would silence the live claim along with the
#: dead one in every case, so it is never the right one here.
#:
#:   cadence: historical           every cadence word on this line
#:   cadence: historical=monthly   only `monthly`, and it must be a real cadence
#:                                 word — an exemption naming something else is a
#:                                 typo or dead weight, and is reported (A11's
#:                                 rule for an untracked TERM).
_HISTORICAL = "historical"
_EXEMPT = re.compile(rf"cadence:\s*{_HISTORICAL}(?:=([A-Za-z-]+))?")


class CadenceError(ValueError):
    """A cron this gate refuses to classify, or a marker it cannot resolve."""


def cadence_of(form: str, value: str) -> str:
    """The human cadence word an authority's own text implies.

    Refuses rather than guesses. A cron this cannot classify (`0 4 1,15 * *`,
    a step, a range) is a cadence no single word describes, and inventing one
    would put a wrong claim in the tree with a gate's name on it.
    """
    if form == "trigger":
        if value == "pull_request":
            return "per-PR"
        raise CadenceError(f"no cadence word for trigger {value!r}")
    if form != "cron":
        raise CadenceError(f"unknown authority form {form!r}")

    fields = value.split()
    if len(fields) != 5:
        raise CadenceError(f"not a 5-field cron: {value!r}")
    _minute, _hour, dom, month, dow = fields
    if month != "*":
        raise CadenceError(f"month-scoped cron, no single-word cadence: {value!r}")
    if dom == "*" and dow == "*":
        return "daily"
    if dom == "*" and dow.isdigit():
        return "weekly"
    if dom.isdigit() and dow == "*":
        return "monthly"
    raise CadenceError(f"cron has no single-word cadence: {value!r}")


@dataclass(frozen=True)
class _Authority:
    """A resolved (workflow, trigger-form) pair — the two roles kept separate.

    `cadence` resolves a marker; `aliases`/`self_path` decide whether the tripwire
    even looks at a block. A record can carry the first and none of the second
    (see `ci`), which is why they are distinct fields rather than one notion of
    "known workflow".
    """

    cadence: str
    aliases: tuple[str, ...]
    self_path: str | None


def _authorities() -> dict[str, _Authority]:
    """id -> {cadence, aliases}, over both halves.

    The in-repo half reads the cron out of the workflow file; the invisible half
    reads the mirrored value. Same shape out, so callers cannot accidentally
    treat one as more trustworthy than the other.
    """
    out: dict[str, _Authority] = {}

    for wid, spec in _IN_REPO.items():
        rel = spec.path
        form = spec.form
        try:
            value = _cx.authority_value((_REPO / rel).read_text("utf-8"), form)
        except _cx.AuthorityError as exc:
            raise CadenceError(f"{rel}: {exc}") from exc
        out[wid] = _Authority(
            cadence=cadence_of(form, value),
            aliases=spec.aliases,
            # A workflow's own comments describe itself, so inside this file every
            # cadence word is a claim about it and needs no alias to be spotted.
            # `parity.yml:120` is exactly that: "On the weekly cron, a red run IS
            # the notice" names no workflow because it doesn't have to.
            self_path=rel if spec.self_identifies else None,
        )

    mirror = json.loads(_MIRROR.read_text("utf-8"))
    for rec in mirror["authorities"]:
        if rec["kind"] != "cadence":
            continue
        out[rec["id"]] = _Authority(
            cadence=cadence_of(rec["form"], rec["value"]),
            aliases=tuple(rec["aliases"]),
            # No self-path: the file is in another repo, so nothing here can be it.
            self_path=None,
        )
    return out


def _scanned_files() -> list[Path]:
    """Tracked files only — which is a sharper edge than it looks.

    A page is invisible to this gate until it is `git add`ed, so a new document
    full of cadence claims runs green right up to the commit that tracks it. That
    is the right scope (an untracked scratch file is not a claim this repo makes)
    but it means the gate's verdict on a work-in-progress tree is provisional.
    """
    listing = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=_REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [
        p
        for rel in listing.split("\0")
        if rel and rel not in _NOT_SCANNED and Path(rel).suffix in _SUFFIXES
        # Tracked but absent: a staged deletion, or a checkout mid-rebase. Not a
        # cadence finding, and not worth crashing the gate over.
        if (p := _REPO / rel).is_file()
    ]


_LIST_ITEM = re.compile(r"^\s*(?:[-*+]|\d+\.)\s")
_FENCE = re.compile(r"^\s*(?:```|~~~)")


def _blocks(text: str, *, markdown: bool) -> list[tuple[int, list[str]]]:
    """(1-based start line, lines) for each run of consecutive non-blank lines.

    One rule for four languages, which is why it is this and not a Markdown
    paragraph parser: in prose a run is a paragraph, in code it is a comment run
    plus whatever it sits against. A fenced mermaid diagram is one run, so a
    `%%` marker inside the fence governs the nodes it is drawn with.

    Markdown gets ONE extra break, at a list item, because a tight bullet list is
    a single run of non-blank lines and `concepts/docs-site.md`'s is 36 of them —
    which would put one bullet's `monthly` in the same neighbourhood as five
    unrelated bullets and force a marker for each. Fences suppress the break: a
    quoted YAML snippet is full of lines starting `- ` that are not list items.
    """
    out: list[tuple[int, list[str]]] = []
    cur: list[str] = []
    start = 0
    fenced = False

    def flush() -> None:
        nonlocal cur
        if cur:
            out.append((start, cur))
            cur = []

    for n, line in enumerate(text.splitlines(), start=1):
        if markdown and _FENCE.match(line):
            fenced = not fenced
        if not line.strip():
            flush()
            continue
        if markdown and not fenced and cur and _LIST_ITEM.match(line):
            flush()
        if not cur:
            start = n
        cur.append(line)
    flush()
    return out


def _words_in(lines: list[str]) -> tuple[set[str], list[str], set[str]]:
    """(cadence words claimed, malformed exemptions, words an exemption hid).

    The third return value exists because an exemption is the one construct here
    that carries a cadence word and REMOVES it from the check — i.e. the only
    off-switch in the grammar. `_findings` compares what it hid against what the
    block's workflows currently run, so `historical` cannot be used to silence a
    live claim.
    """
    found: set[str] = set()
    malformed: list[str] = []
    hidden: set[str] = set()
    for line in lines:
        exempt: set[str] = set()
        for m in _EXEMPT.finditer(line):
            named = m.group(1)
            if named is None:
                exempt = set(_CADENCE_WORDS)
                break
            if named.lower() not in {w.lower() for w in _CADENCE_WORDS}:
                malformed.append(named)
                continue
            exempt.update(w for w in _CADENCE_WORDS if w.lower() == named.lower())
        on_line = {w for w, rx in _WORD_RES.items() if rx.search(line)}
        hidden |= on_line & exempt
        found |= on_line - exempt
    return found, malformed, hidden


def _markers_in(lines: list[str]) -> set[str]:
    return {
        m.group(1)
        for line in lines
        for m in _MARKER.finditer(line)
        if m.group(1) != _HISTORICAL
    }


def _identities_in(
    lines: list[str], rel: str, authorities: dict[str, _Authority]
) -> set[str]:
    blob = "\n".join(lines).lower()
    named = {
        wid
        for wid, spec in authorities.items()
        for alias in spec.aliases
        if alias.lower() in blob
    }
    return named | {wid for wid, spec in authorities.items() if spec.self_path == rel}


def _findings() -> tuple[list[str], list[str]]:
    """(disagreements, unannotated) — collected over the whole tree in one walk."""
    authorities = _authorities()
    disagreements: list[str] = []
    unannotated: list[str] = []

    for path in _scanned_files():
        rel = path.relative_to(_REPO).as_posix()
        try:
            text = path.read_text("utf-8")
        except UnicodeDecodeError:
            continue
        for start, lines in _blocks(text, markdown=path.suffix == ".md"):
            markers = _markers_in(lines)
            stated, malformed, hidden = _words_in(lines)
            ids = _identities_in(lines, rel, authorities)
            where = f"{rel}:{start}"

            if malformed:
                disagreements.append(
                    f"{where}: exemption names no cadence word: {', '.join(malformed)}"
                )

            # An exemption is the grammar's only off-switch, so it needs a lock.
            # Without this, `cadence: historical=weekly` on a block claiming
            # `parity.yml` runs weekly is GREEN — the word is stripped before
            # either check sees it, and a marker that silences the live claim it
            # annotates is worse than no marker at all. `historical` means a
            # schedule that no longer holds; if the word it hides is what that
            # workflow runs on TODAY, it is being used as a mute button.
            live = {authorities[w].cadence for w in markers | ids if w in authorities}
            if bogus := hidden & live:
                disagreements.append(
                    f"{where}: `historical` hides {sorted(bogus)}, which is the "
                    f"CURRENT cadence of {sorted(markers | ids)} — exempt a dead "
                    f"schedule, not a live one"
                )

            if markers:
                unknown = [m for m in markers if m not in authorities]
                if unknown:
                    disagreements.append(
                        f"{where}: marker names no known workflow: {', '.join(unknown)}"
                    )
                    continue
                declared = {authorities[m].cadence for m in markers}
                if declared != stated:
                    # The hint is not decoration: a `historical` exemption that
                    # missed by one line is the likeliest way to land here, and
                    # the symptom (an unaccounted word) looks nothing like the
                    # cause (the marker sits under the sentence, not in it).
                    hint = (
                        "  [an exemption is LINE-scoped — it must sit on the same "
                        "line as the word it exempts]"
                        if any(_EXEMPT.search(x) for x in lines)
                        else ""
                    )
                    disagreements.append(
                        f"{where}: markers {sorted(markers)} derive "
                        f"{sorted(declared)}, prose states {sorted(stated) or '[]'}"
                        f"{hint}"
                    )
            elif stated and (hits := _identities_in(lines, rel, authorities)):
                unannotated.append(
                    f"{where}: states {sorted(stated)} about {sorted(hits)}, "
                    f"no `cadence:` marker"
                )

    return disagreements, unannotated


def test_every_stated_cadence_matches_its_workflows_cron() -> None:
    """Annotated prose agrees with the authority the marker names."""
    disagreements, _ = _findings()
    assert not disagreements, (
        "cadence claims disagreeing with their authority:\n"
        + "\n".join(f"  {d}" for d in disagreements)
    )


def test_no_cadence_claim_goes_unannotated() -> None:
    """A block naming a workflow and a cadence must say which it is claiming.

    The tripwire, and the half that catches tomorrow's page. Annotate the block,
    or mark the line `cadence: historical` if it narrates a superseded schedule
    rather than asserting a live one.
    """
    _, unannotated = _findings()
    assert not unannotated, "unannotated cadence claims:\n" + "\n".join(
        f"  {u}" for u in unannotated
    )


@pytest.mark.parametrize(
    ("form", "value", "expected"),
    [
        ("cron", "0 3 * * 0", "weekly"),
        ("cron", "0 4 * * 1", "weekly"),
        ("cron", "0 4 1 * *", "monthly"),
        ("cron", "0 4 * * *", "daily"),
        ("cron", "17 6 * * *", "daily"),
        ("trigger", "pull_request", "per-PR"),
    ],
)
def test_cadence_of_classifies_every_cron_in_use(
    form: str, value: str, expected: str
) -> None:
    """The seven crons across both repos, plus the one trigger form mirrored."""
    assert cadence_of(form, value) == expected


def test_the_mirrors_own_claims_are_reconcilable_in_shape() -> None:
    """Every record carries what the far side needs to check it.

    Not a substitute for that check — only the satellite can compare a value to
    its cron. This catches the record that could never be reconciled at all: a
    form nothing can read, or a repo with no reconciling job, which would sit
    here looking checked.
    """
    records = json.loads(_MIRROR.read_text("utf-8"))["authorities"]
    assert records, "an empty mirror is not a pass"
    for rec in records:
        assert rec["form"] in {"cron", "trigger"}, rec["id"]
        assert rec["repo"] == "niko86/laterite-dev", (
            f"{rec['id']}: records only reconcile where a far-side job runs; "
            f"adding a repo means adding that job first"
        )
        cadence_of(rec["form"], rec["value"])  # raises if unclassifiable


def test_the_reconciler_refuses_every_way_of_seeing_nothing(tmp_path: Path) -> None:
    """Unreadable, absent and not-mine must all fail, never quietly pass.

    Three separate ways to check zero things, and each one looks exactly like
    "all clear" to a caller that only reads the exit code.
    """
    mirror = tmp_path / "m.json"
    mirror.write_text(
        json.dumps(
            {
                "authorities": [
                    {
                        "id": "x",
                        "kind": "cadence",
                        "repo": "owner/repo",
                        "path": ".github/workflows/x.yml",
                        "form": "cron",
                        "value": "0 4 1 * *",
                        "aliases": [],
                    }
                ]
            }
        )
    )

    with pytest.raises(_cx.AuthorityError):
        _cx.reconcile(tmp_path / "absent.json", tmp_path, "owner/repo")

    # A record for another repo is not this run's business, but a run that
    # matches NOTHING has verified nothing and must say so via main()'s exit.
    problems, checked = _cx.reconcile(mirror, tmp_path, "someone/else")
    assert (problems, checked) == ([], 0)

    # Matched, but the workflow it names isn't there.
    problems, checked = _cx.reconcile(mirror, tmp_path, "owner/repo")
    assert checked == 1
    assert problems and "does not exist" in problems[0]


def test_the_reconciler_reads_the_cron_not_a_lookalike(tmp_path: Path) -> None:
    """`on:`-scoped, so a cron quoted in a comment elsewhere can't be mistaken
    for the schedule — which is how a gate reports agreement with a sentence."""
    wf = """\
# A comment mentioning another workflow's schedule:
#     - cron: "0 9 * * 5"
on:
  workflow_dispatch: {}
  schedule:
    - cron: "0 4 1 * *"
  pull_request:

jobs:
  build:
    if: github.event_name != 'pull_request'
"""
    assert _cx.authority_value(wf, "cron") == "0 4 1 * *"
    assert _cx.authority_value(wf, "trigger") == "pull_request"

    no_pr = 'on:\n  schedule:\n    - cron: "0 4 1 * *"\n\njobs: {}\n'
    with pytest.raises(_cx.AuthorityError):
        _cx.authority_value(no_pr, "trigger")


@pytest.mark.parametrize(
    "value",
    [
        "0 4 1,15 * *",  # twice a month — no single word describes it
        "0 4 */2 * *",  # every other day
        "0 4 1 6 *",  # once a year
        "0 4 1 * 0",  # day-of-month AND weekday
        "0 4 1 *",  # malformed
    ],
)
def test_cadence_of_refuses_rather_than_guesses(value: str) -> None:
    """An unclassifiable cron must raise, not round to the nearest word.

    A guess here is worse than no gate: it would write a wrong cadence into the
    tree with a passing test's name attached to it.
    """
    with pytest.raises(CadenceError):
        cadence_of("cron", value)
