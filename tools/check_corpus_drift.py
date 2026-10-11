"""The `corpus-drift` CI verdict: corpus-qa `baseline --check` over the vendored corpus.

The committed baseline (`rust-packages/laterite-ags4-corpus-qa/baselines/
pyags4-vendor.json`) freezes the exact findings the validator produces over the
vendored python-ags4 fixtures. Until #1034 nothing in this repo ran the check,
so a change that moved findings merged green and the baseline went stale until
someone ran it by hand. The job runs `crawl` + `baseline --check` and hands the
two JSON outputs here, which decides and explains:

  * EVERY run prints one summary line: how many files were checked, how many
    the walk skipped, how many files under the fixture root the crawl did not
    select, and how many contents are only in the baseline or only in the
    crawl. A gate that drops input says what it dropped (CLAUDE.md), and a
    green "no drift" over a corpus that silently shrank would be exactly that.
  * On drift it names each moved file by its fixture name (the corpus is
    vendored and public, so names are safe here even though the baseline is
    sha-keyed), lists the removed / added findings, and prints the re-freeze
    command.
  * The binary's exit code and the JSON must agree. If they do not, neither is
    believed and the run fails, rather than one silently winning.

Stdlib only: the job installs no project environment for it.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

#: Exit codes. 1 matches corpus-qa's own "drift" exit; 2 is this script's
#: refusal to pick between a JSON and an exit code that disagree.
CLEAN, DRIFT, DISAGREE = 0, 1, 2


def _norm(p: str) -> str:
    return Path(p).as_posix()


def _finding(f: dict) -> str:
    loc = ""
    if f.get("line") is not None:
        loc = f" L{f['line']}"
        if f.get("field_index") is not None:
            loc += f" f{f['field_index']}"
    sev = f.get("severity", "error")
    tail = "" if sev == "error" else f" [{sev}]"
    return f"{f['rule']} {f.get('group', '')}{loc}{tail}"


def _names(sha: str, by_sha: dict[str, list[str]]) -> str:
    names = by_sha.get(sha)
    short = sha[:12]
    return (
        f"{', '.join(names)} ({short})" if names else f"{short} (no file in this crawl)"
    )


def report(
    *,
    crawl: dict,
    drift: dict | None,
    check_exit: int,
    root_files: list[str],
    root: str,
    baseline: str,
) -> tuple[list[str], int]:
    """The lines to print and the exit code. Pure, so every branch is testable."""
    files = crawl.get("files", [])
    by_sha: dict[str, list[str]] = {}
    for e in files:
        by_sha.setdefault(e["sha256"], []).append(Path(e["source"]).name)
    selected = {_norm(e["source"]) for e in files}
    not_selected = sum(1 for p in root_files if _norm(p) not in selected)
    copy_errors = sum(1 for e in files if e.get("copy_error"))

    def count(key: str) -> str:
        return "?" if drift is None else str(len(drift.get(key, [])))

    summary = (
        f"corpus-drift: {len(files)} files checked ({len(by_sha)} by content), "
        f"walk_skipped {crawl.get('walk_skipped', '?')}, "
        f"not selected {not_selected}, copy errors {copy_errors}, "
        f"only_in_committed {count('only_in_committed')}, "
        f"only_in_current {count('only_in_current')}, "
        f"changed {count('changed')}"
    )

    if drift is None or check_exit not in (CLEAN, DRIFT):
        return [
            f"baseline --check exited {check_exit}: an error, not a drift verdict "
            "(its own message is above).",
            summary,
        ], (check_exit or DISAGREE)

    lines: list[str] = []
    drifted = bool(
        drift.get("only_in_committed")
        or drift.get("only_in_current")
        or drift.get("changed")
    )
    if drifted != (check_exit == DRIFT):
        lines.append(
            f"baseline --check exited {check_exit} but its JSON reports "
            f"{'drift' if drifted else 'no drift'}: the two disagree, so neither "
            "is trusted."
        )
        lines.append(summary)
        return lines, DISAGREE

    if not drifted:
        lines.append(
            f"no drift: {drift.get('current_files')} contents match the committed "
            "baseline."
        )
        lines.append(summary)
        return lines, CLEAN

    lines.append("Finding drift against the committed baseline:")
    for fd in drift.get("changed", []):
        lines.append("")
        lines.append(
            f"  {_names(fd['sha256'], by_sha)}: "
            f"{fd['committed_outcome']} -> {fd['current_outcome']}"
        )
        if fd.get("dict_change"):
            lines.append(f"    dict: {fd['dict_change']}")
        lines.extend(f"    - {_finding(f)}" for f in fd.get("removed", []))
        lines.extend(f"    + {_finding(f)}" for f in fd.get("added", []))
    lines.extend(
        f"\n  only in the baseline: {_names(sha, by_sha)}"
        for sha in drift.get("only_in_committed", [])
    )
    lines.extend(
        f"\n  only in this crawl: {_names(sha, by_sha)}"
        for sha in drift.get("only_in_current", [])
    )
    lines += [
        "",
        "If this change is intended, re-freeze the baseline in THIS PR (from the "
        "repo root), so the diff shows exactly which findings moved:",
        "",
        "  QA=output/qa-drift",
        "  cargo run --manifest-path rust-packages/Cargo.toml "
        f"-p laterite-ags4-corpus-qa -- crawl --root {root} --all "
        '--corpus-dir "$QA" --no-input -q',
        "  cargo run --manifest-path rust-packages/Cargo.toml "
        f"-p laterite-ags4-corpus-qa -- baseline --out {baseline} "
        '--show-warnings --show-fyi --no-check-files --corpus-dir "$QA" --no-input',
        "",
        "An unintended drift is a regression; see the README beside the baseline.",
        summary,
    ]
    return lines, DRIFT


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--crawl", required=True, type=Path, help="crawl --json output")
    ap.add_argument(
        "--drift", required=True, type=Path, help="baseline --check --json output"
    )
    ap.add_argument(
        "--check-exit", required=True, type=int, help="baseline --check exit code"
    )
    ap.add_argument("--root", required=True, help="the crawled fixture root")
    ap.add_argument("--baseline", required=True, help="the committed baseline path")
    args = ap.parse_args(argv)

    crawl = json.loads(args.crawl.read_text(encoding="utf-8"))
    drift = None
    if args.check_exit in (CLEAN, DRIFT):
        drift = json.loads(args.drift.read_text(encoding="utf-8"))
    root_files = [p.as_posix() for p in Path(args.root).rglob("*") if p.is_file()]

    lines, code = report(
        crawl=crawl,
        drift=drift,
        check_exit=args.check_exit,
        root_files=root_files,
        root=args.root,
        baseline=args.baseline,
    )
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    sys.exit(main())
