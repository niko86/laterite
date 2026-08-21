#!/usr/bin/env python3
"""Reconcile `external-authorities.json` against the repo it makes claims about.

The far half of the cadence gate, and the half that stops it being decorative.

`external-authorities.json` mirrors workflows that live in the dev satellite,
which this repo's CI cannot read. The root-suite gate
`tests/test_stated_cadences_faithful.py` therefore checks this repo's prose
against that mirror — and a mirror with nothing comparing it back to its subject
is #549's Shape 1, the gate enforcing a proxy for the promise (the same hole
`test_vendored_authority_faithful.py` closed at the dictionary's root). This
script is the comparison, and it necessarily runs in the OTHER repo:

    # in niko86/laterite-dev, having checked out public laterite/ alongside
    python laterite/tools/check_external_authorities.py \\
        --mirror laterite/external-authorities.json \\
        --tree . --repo niko86/laterite-dev

Direction matters and was chosen, not defaulted. The satellite can read this
public repo for free — it already clones it — while the reverse needs a PAT and
would print a private repo's CI structure into a world-readable Actions log.

It lives here rather than being copied there so the two cannot drift; the
satellite carries its own engine and its own copies of these tools, and a second
copy of this one would be the same class of bug it exists to catch.

WHY IT CAN BE A PER-PR GATE, unlike `ruleset-drift.yml` — the precedent it
otherwise follows. That one is a cron because its subject is GitHub's API, and a
per-PR gate on a flaky remote fails for reasons the PR didn't cause (#561). This
one's subject is a file on disk in the checkout. There is no network, so there is
no "could not read" that isn't a genuine fault, and a mismatch is always
actionable by whoever is editing the workflow.

A MISSING record, path, or mirror is a FAILURE, never a skip. `ruleset-drift.yml`
already wrote the argument: a gate that cannot read its own subject is
decorative, and a silent permanent skip is how it stays that way.

This DELIBERATELY overrides the "unreachable exits 0 — no opinion" posture
`check_upstream_pin.py` and `check_ext_drift.py` both take, and which was the
stated intent for this check too. Their escape hatch exists because their subject
is a NETWORK — PyPI, the GitHub API — where a timeout is not evidence of
anything. Pushing the check into the satellite removed the network: the subject
here is a file in a checkout, so every way of failing to read it is a real fault
(a bad path, a lost record, a mirror that was deleted), and "no opinion" would
just be the permanent skip the paragraph above warns about. If a future version
reaches over a network again, that clause comes back with it.

Exit 0 all reconciled · 1 drift, or anything unreadable.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


class AuthorityError(Exception):
    """The subject could not be read — never silently downgraded to a pass."""


def on_block(text: str) -> str:
    """A workflow's top-level `on:` mapping, as text.

    Scoped rather than whole-file because both patterns callers look for have
    decoys elsewhere in a real workflow: `github.event_name != 'pull_request'`
    in a job condition, or a `cron:` inside a comment quoting another workflow.
    """
    kept: list[str] = []
    inside = False
    for line in text.splitlines():
        if re.match(r"^on:", line):
            inside = True
            continue
        if inside:
            if line and not line[0].isspace():
                break
            kept.append(line)
    return "\n".join(kept)


def authority_value(text: str, form: str) -> str:
    """The workflow's own text for `form` — the FACT a mirror record claims.

    Returned verbatim (the cron string as written), because the whole point of
    mirroring the raw value is that nobody re-types a derived one.
    """
    block = on_block(text)
    if form == "cron":
        crons = re.findall(r'^\s*-\s*cron:\s*"([^"]+)"', block, re.M)
        if len(crons) != 1:
            raise AuthorityError(f"expected exactly one cron, found {len(crons)}")
        return crons[0]
    if form == "trigger":
        if not re.search(r"^\s{2}pull_request:", block, re.M):
            raise AuthorityError("no top-level `pull_request:` trigger")
        return "pull_request"
    # `manual` is the ABSENCE of a schedule, so it is the one form whose fact is
    # a negative — and a negative is exactly what a mirror record cannot assert
    # by quoting a line. Both halves are checked: the trigger must be there AND
    # no cron may be, or "on-demand" in the prose would survive somebody adding
    # a schedule back.
    if form == "manual":
        if not re.search(r"^\s{2}workflow_dispatch:", block, re.M):
            raise AuthorityError("no top-level `workflow_dispatch:` trigger")
        crons = re.findall(r'^\s*-\s*cron:\s*"([^"]+)"', block, re.M)
        if crons:
            raise AuthorityError(f"claims manual but carries cron(s) {crons}")
        return "workflow_dispatch"
    raise AuthorityError(f"unknown authority form {form!r}")


def reconcile(mirror: Path, tree: Path, repo: str) -> tuple[list[str], int]:
    """(problems, number reconciled) for every record claiming to be about `repo`."""
    if not mirror.is_file():
        raise AuthorityError(f"mirror not found: {mirror}")
    records = json.loads(mirror.read_text("utf-8"))["authorities"]

    problems: list[str] = []
    checked = 0
    for rec in records:
        if rec["repo"] != repo:
            continue
        checked += 1
        path = tree / rec["path"]
        if not path.is_file():
            problems.append(f"{rec['id']}: {rec['path']} does not exist in {tree}")
            continue
        try:
            real = authority_value(path.read_text("utf-8"), rec["form"])
        except AuthorityError as exc:
            problems.append(f"{rec['id']}: {rec['path']}: {exc}")
            continue
        if real != rec["value"]:
            problems.append(
                f"{rec['id']}: {rec['path']} says {real!r}, "
                f"the mirror claims {rec['value']!r}"
            )
    return problems, checked


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--mirror", type=Path, required=True)
    ap.add_argument("--tree", type=Path, required=True)
    ap.add_argument("--repo", required=True, help="which records to check, by `repo`")
    args = ap.parse_args()

    try:
        problems, checked = reconcile(args.mirror, args.tree, args.repo)
    except (AuthorityError, KeyError, json.JSONDecodeError) as exc:
        print(f"cannot read the mirror: {exc}", file=sys.stderr)
        return 1

    if not checked:
        # Not a pass. Either the mirror lost its records for this repo or the
        # --repo spelling is wrong, and both look identical to "all clear" if
        # this returns 0 — the empty-comparison trap check_changelog.py records
        # eating a CI job.
        print(
            f"no records claim to be about {args.repo} — nothing was checked",
            file=sys.stderr,
        )
        return 1

    for p in problems:
        print(f"DRIFT {p}", file=sys.stderr)
    print(f"{checked - len(problems)}/{checked} authorities reconciled for {args.repo}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
