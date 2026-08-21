#!/usr/bin/env python3
"""Notice when the pinned python-ags4 goes stale (laterite-dev#558).

`parity.yml`'s header states why the cron exists:

    # Manual `workflow_dispatch` + scheduled weekly so an upstream
    # silent drift surfaces before it bites a user.

Thirty lines below, `PYTHON_AGS4_VERSION: "1.2.0"`. The pin makes the stated
purpose structurally impossible: the cron re-runs the same frozen oracle forever
and can never see upstream move. It ran on that schedule, going green, and
proving something other than what its header claims — the single sentence and the
single constant were 30 lines apart, and three separate audit lenses read that
workflow's shape without noticing.

Unpinning is the wrong fix — reproducibility is why the pin exists, and a floating
oracle would thrash `parity-known-failures.json` on upstream's schedule rather than
ours. So keep the pin and add the missing half: **notice**. A human then decides
whether to follow.

This is deliberately NOT a PR gate. A PR must not go red because upstream released
something while it was open — that is not the PR's fault, and a check that fails
for reasons the author cannot act on is a check people learn to ignore. It runs on
the weekly cron, where a red run is the notification.

It compares PyPI's latest against the version *actually installed*, not against a
constant scraped from a file. The installed distribution is the fact; the
hand-written "1.2.0"s scattered across the tree are claims about it.

Exit codes:
    0  the pin is current, or we could not reach PyPI (no opinion, not a verdict)
    1  upstream has moved -- go look
    2  usage error
"""

from __future__ import annotations

import importlib.metadata
import json
import sys
import urllib.error
import urllib.request

_DIST = "python-ags4"
_PYPI = f"https://pypi.org/pypi/{_DIST}/json"
_TIMEOUT = 20


class InfraError(RuntimeError):
    """PyPI was unreachable or unintelligible. Not a verdict about the pin."""


def _latest_on_pypi() -> str:
    try:
        with urllib.request.urlopen(_PYPI, timeout=_TIMEOUT) as r:
            body = r.read()
    except (urllib.error.URLError, TimeoutError, OSError) as e:
        raise InfraError(f"could not reach PyPI: {e}") from e
    try:
        data = json.loads(body)
        version = data["info"]["version"]
    except (json.JSONDecodeError, KeyError, TypeError) as e:
        raise InfraError(f"PyPI returned something we could not read: {e}") from e
    if not isinstance(version, str) or not version:
        raise InfraError(f"PyPI reported an implausible version: {version!r}")
    return version


def main() -> int:
    if len(sys.argv) > 1:
        print(f"usage: {sys.argv[0]}  (no arguments)", file=sys.stderr)
        return 2

    try:
        pinned = importlib.metadata.version(_DIST)
    except importlib.metadata.PackageNotFoundError:
        # The oracle isn't installed. That is a broken environment, and this tool
        # has nothing to say about it — but it must not be mistaken for "the pin
        # is fine", so it is loud and non-zero.
        print(
            f"BROKEN: {_DIST} is not installed, so there is no pin to check.\n"
            "  It is a declared dev dependency (pyproject.toml). Run `uv sync`.",
            file=sys.stderr,
        )
        return 1

    try:
        latest = _latest_on_pypi()
    except InfraError as e:
        # Deliberately exit 0. A network blip is not evidence the pin is current,
        # and it is not evidence it is stale either. Reporting red here would
        # train the reader to ignore this job — which is exactly how a check that
        # exists stops being a check that works.
        print(f"NO OPINION: {e}", file=sys.stderr)
        print("  Not failing — an unreachable index says nothing about the pin.")
        return 0

    if pinned == latest:
        print(f"{_DIST} pin is current: {pinned} is the latest on PyPI.")
        return 0

    print(
        f"UPSTREAM HAS MOVED: we pin {_DIST} {pinned}; PyPI's latest is {latest}.\n"
        "\n"
        "This is a notice, not a regression. The pin is deliberate — it is what makes\n"
        "parity-known-failures.json a stable contract. But parity.yml's cron exists so\n"
        "'an upstream silent behavioural drift surfaces before it bites a user', and a\n"
        "frozen oracle cannot surface drift. Someone has to look.\n"
        "\n"
        "To follow upstream, these move together (a test enforces it):\n"
        f"  pyproject.toml                 python-ags4=={latest}\n"
        f'  .github/workflows/parity.yml   PYTHON_AGS4_VERSION: "{latest}"\n'
        "  parity-known-failures.json     python_ags4_version + re-vendor the failing SET\n"
        "                                 (tools/check_parity.py parity.log --write)\n"
        "  rust-packages/laterite-ags4-validator/data/PROVENANCE.md\n"
        "                                 re-sync the dictionaries if upstream changed them\n"
        "                                 (see PROVENANCE.md) and bump the date\n"
        "\n"
        "To stay put, say so: this will report again next month.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
