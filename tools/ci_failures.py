#!/usr/bin/env python3
"""Print why each failed check on a PR failed: the job, its failed step, the error.

    uv run --no-project python tools/ci_failures.py 1189
    uv run --no-project python tools/ci_failures.py 1189 --context 40
    uv run --no-project python tools/ci_failures.py --job 128580293094

Reading a red PR used to take a pass per check: `gh pr checks`, then
`gh run view --log-failed` for each failed job, then a grep through its
checkout and cleanup noise for the line that mattered, then another grep when
that one missed. This does that pass once. For every failed check it prints the
job's failed step and the log lines leading up to each `##[error]`, with
timestamps, ANSI colour and runner boilerplate stripped.

WHAT THIS DROPS: checks that are not GitHub Actions jobs (codecov and the like)
are listed by name and link only, since they have no job log to read. Each run
prints how many of those it skipped. Boilerplate lines (`shell:`, `env:`,
`##[group]`) are dropped from the error context.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys

JOB_LINK = re.compile(r"/actions/runs/(\d+)/job/(\d+)")
TIMESTAMP = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z ?")
ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]|\^\[\[[0-9;]*m")
#: Runner lines that say how a step ran, never why it failed.
NOISE = re.compile(
    r"^(\s*(shell|env):|##\[(group|endgroup)\]|\s+[A-Z_]+: |Post job cleanup|"
    r"\[command\]/usr/bin/git )"
)
FAILED = {"FAILURE", "ERROR", "TIMED_OUT", "STARTUP_FAILURE", "ACTION_REQUIRED"}


def job_id(link: str) -> str | None:
    """The Actions job id in a check's link, or None for a non-Actions check."""
    m = JOB_LINK.search(link or "")
    return m.group(2) if m else None


def clean(line: str) -> str:
    return ANSI.sub("", TIMESTAMP.sub("", line)).rstrip()


def error_context(log: str, context: int) -> list[list[str]]:
    """For each `##[error]` line, it and up to `context` meaningful lines before it."""
    lines = [clean(raw) for raw in log.splitlines()]
    blocks: list[list[str]] = []
    for i, line in enumerate(lines):
        if not line.startswith("##[error]"):
            continue
        before = [
            ln
            for ln in lines[max(0, i - context * 3) : i]
            if ln and not NOISE.match(ln)
        ]
        blocks.append([*before[-context:], line])
    return blocks


def gh(*args: str) -> str:
    return subprocess.run(
        ["gh", *args], capture_output=True, text=True, check=False
    ).stdout


def report_job(repo: str, jid: str, context: int) -> None:
    """Print one job's failed step and the context of each `##[error]` in its log."""
    steps = json.loads(gh("api", f"repos/{repo}/actions/jobs/{jid}") or "{}")
    bad = [s["name"] for s in steps.get("steps", []) if s["conclusion"] == "failure"]
    print(f"    failed step: {', '.join(bad) or '(none recorded)'}")
    log = gh("api", "--allow-escape-sequences", f"repos/{repo}/actions/jobs/{jid}/logs")
    blocks = error_context(log, context)
    if not blocks:
        print("    no ##[error] line in the log")
    for block in blocks:
        print()
        print("\n".join(f"    {ln}" for ln in block))
    print()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("pr", nargs="?", help="PR number")
    ap.add_argument("--job", help="one Actions job id instead of a PR's checks")
    ap.add_argument("--repo", default="niko86/laterite")
    ap.add_argument("--context", type=int, default=25, help="lines before each error")
    args = ap.parse_args()
    if args.job:
        report_job(args.repo, args.job, args.context)
        return 0
    if not args.pr:
        ap.error("give a PR number or --job")

    raw = gh("pr", "checks", args.pr, "--repo", args.repo, "--json", "name,state,link")
    try:
        checks = json.loads(raw)
    except json.JSONDecodeError:
        print(f"ci_failures: could not read the checks on #{args.pr}", file=sys.stderr)
        return 2
    failed = [c for c in checks if c["state"] in FAILED]
    not_actions = 0
    for c in failed:
        jid = job_id(c["link"])
        print(f"=== {c['name']}  ({c['link']})")
        if jid is None:
            not_actions += 1
            print("    not a GitHub Actions job: no log to read")
            continue
        report_job(args.repo, jid, args.context)
    print(
        f"ci_failures: {len(failed)} of {len(checks)} check(s) failed on #{args.pr}; "
        f"{not_actions} of them not an Actions job (no log read)."
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
