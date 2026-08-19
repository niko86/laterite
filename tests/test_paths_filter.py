"""The `code` path filter must narrow — and must not silently switch a gate off.

`ci.yml`'s `changes` job decides which PRs pay for the heavy jobs (`python`,
`rust`, `test`). Two failure modes sit either side of it, and this file guards
both because fixing one is how you cause the other:

  TOO WIDE  — #230. `dorny/paths-filter` treats each list entry as an
              independent pattern and OR's the results, and picomatch's `!x`
              matches *everything that is not x*. Two `!` rules meant to narrow
              `code` therefore made it true for 1385 of 1385 tracked files, so
              the heaviest jobs ran on every PR including docs-only ones.

  TOO NARROW — #207. A gate whose inputs are not declared as a POSITIVE rule
              never fires. Dropping the `!` rules without first declaring what
              the gates actually read would have silently stopped linting 26
              files under `tools/`, `tools/xcheck/`, `ags-wiki/.bootstrap/` and
              `examples/` — a gate that exists and nothing reaches.

So: a ban on the pattern class that caused the first, and a live check against
`ruff`'s own file list for the second. The second is deliberately DERIVED rather
than a hand-written list, so a new `tools/*.py` cannot fall out of coverage
without this test noticing.

`_matches` reimplements the picomatch subset these filters use. It was verified
against the real `picomatch` (via `web/node_modules`) over every tracked file
in the repo — exact agreement, including the negation semantics that produced
#230. Re-run whenever a filter is added, and say what it covered, because the
number is the only thing that shows the claim is not stale:

  #230 (four filters)   1385 paths x every pattern, exact agreement
  #313 (five filters)   1453 paths x 40 patterns = 58120 pairs, exact agreement
                        — `prose` joined for the cadence gate
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

REPO = Path(__file__).resolve().parents[1]
CI = REPO / ".github" / "workflows" / "ci.yml"


def _regex(pattern: str) -> re.Pattern[str]:
    """Translate one glob to a regex, picomatch-style.

    `**/` spans zero or more directories (so `a/**/b` matches `a/b`); a lone `*`
    never crosses a `/`. Nothing else in these filters uses extglob or braces —
    if that changes, re-run the picomatch cross-check in this module's docstring.
    """
    i, out = 0, []
    while i < len(pattern):
        if pattern.startswith("**/", i):
            out.append("(?:.*/)?")
            i += 3
        elif pattern.startswith("**", i):
            out.append(".*")
            i += 2
        elif pattern[i] == "*":
            out.append("[^/]*")
            i += 1
        elif pattern[i] == "?":
            out.append("[^/]")
            i += 1
        else:
            out.append(re.escape(pattern[i]))
            i += 1
    return re.compile("^" + "".join(out) + "$")


def _matches(pattern: str, path: str) -> bool:
    """One pattern against one path. `!x` means "anything that is not x" —
    the semantics that made #230, kept here so the ban below is honest about
    what it is banning."""
    if pattern.startswith("!"):
        return not _regex(pattern[1:]).match(path)
    return bool(_regex(pattern).match(path))


def _filter_matches(patterns: list[str], path: str) -> bool:
    """dorny/paths-filter: a path is in the filter if ANY pattern matches."""
    return any(_matches(p, path) for p in patterns)


@pytest.fixture(scope="module")
def filters() -> dict[str, list[str]]:
    workflow = yaml.safe_load(CI.read_text(encoding="utf-8"))
    step = next(
        s for s in workflow["jobs"]["changes"]["steps"] if s.get("id") == "filter"
    )
    return yaml.safe_load(step["with"]["filters"])


# --- the matcher's own semantics, so a rewrite can't quietly change meaning ---


@pytest.mark.parametrize(
    ("pattern", "path", "expected"),
    [
        ("rust-packages/**", "rust-packages/a/src/lib.rs", True),
        ("rust-packages/**", "packages/x.py", False),
        ("tools/**", "tools/xcheck/run.py", True),
        ("tools/**/*.py", "tools/xcheck/run.py", True),
        ("tools/**/*.py", "tools/gen_census.py", True),  # `**/` spans zero dirs
        ("tools/**/*.py", "tools/build-rust.sh", False),
        ("*.toml", "a/b.toml", False),  # a lone `*` never crosses `/`
        ("!x/**/*.ts", "CHANGELOG.md", True),  # <- the #230 mechanism
    ],
)
def test_matcher_semantics(pattern: str, path: str, *, expected: bool) -> None:
    assert _matches(pattern, path) is expected


# --- #230: the pattern class that made `code` true for the whole repo ---------


def test_no_negated_patterns(filters: dict[str, list[str]]) -> None:
    """`!x` cannot narrow a dorny filter — it widens it to everything-but-x.

    There is no correct use of one here: because entries are OR'd, any `!` rule
    makes its filter true for nearly every path in the repo. Express an
    exclusion by not listing it, or by listing the narrower positive paths.
    """
    offenders = [
        f"{name}: {pattern}"
        for name, patterns in filters.items()
        for pattern in patterns
        if pattern.startswith("!")
    ]
    assert not offenders, (
        "negated patterns widen a dorny/paths-filter instead of narrowing it "
        f"(#230): {offenders}"
    )


# --- it still narrows: paths that must NOT buy a 12-minute python job --------


@pytest.mark.parametrize(
    "path",
    [
        # The three files PR #218 changed — the case that proved #230 in CI.
        "codecov.yml",
        "ags-wiki/concepts/coverage-campaign.md",
        ".github/workflows/nightly.yml",
        # Repo furniture with no gate behind it.
        ".github/dependabot.yml",
        ".github/ISSUE_TEMPLATE/bug.md",
        "ags-wiki/start-here.md",
        "LICENSE",
    ],
)
def test_code_is_false_for_irrelevant_paths(
    filters: dict[str, list[str]], path: str
) -> None:
    assert not _filter_matches(filters["code"], path), (
        f"{path} fires the heavy jobs but no gate in python/rust/test reads it"
    )


# --- and it still fires: inputs a gate genuinely reads ------------------------


@pytest.mark.parametrize(
    "path",
    [
        "rust-packages/laterite-ags4-core/src/lib.rs",
        "rust-packages/laterite-ags4-reference/data/ags_dictionary.json",
        "packages/laterite/python/laterite/__init__.py",
        "packages/laterite/tests/test_certificate.py",
        "tests/test_paths_filter.py",
        "pyproject.toml",
        "uv.lock",
        ".github/workflows/ci.yml",
        # SSOTs whose gates still run in a heavy job: modality.json is read by
        # test_modality_parity in packages/laterite/tests, and the census gate
        # needs the built launchers. `changelog.json` and `observations.json`
        # are NOT here any more — see test_buildless_ssots_need_no_heavy_job.
        "modality.json",
        "surface-census.json",
        # Tooling those gates execute.
        "tools/gen_changelog.py",
        "tools/gen_observations.py",
        "tools/gen_census.py",
        "tools/check_doc_refs.py",
        "tools/xcheck/run.py",
    ],
)
def test_code_is_true_for_gate_inputs(filters: dict[str, list[str]], path: str) -> None:
    assert _filter_matches(filters["code"], path), (
        f"a gate in the python/rust/test jobs reads {path}, but no positive rule "
        "in `code` declares it — the gate would never fire (#207)"
    )


# --- gates that moved to the unconditional job -------------------------------
#
# #207's lesson was that a gate must fire by DECLARATION, not by accident. An
# unconditional job satisfies that more strongly than any path list can: there
# is no condition to get wrong. These two tests are what make that claim
# checkable rather than a comment — the first proves the job really has no
# filter, the second proves the narrowing it paid for actually happened.

BUILDLESS_SSOTS = [
    "changelog.json",
    "CHANGELOG.md",
    "observations.json",
    "OBSERVATIONS.md",
    "web/docs-site/docs/reference/divergences.md",
    "web/docs-site/docs/stylesheets/laterite.css",
    "RELEASING.md",
    "CONTRIBUTING.md",
]


def test_repo_gates_is_unconditional() -> None:
    """The buildless job must have no `if:` and no `needs:`.

    Everything below rests on this. The moment `repo-gates` acquires a path
    filter, every SSOT this commit removed from `code` becomes an undeclared
    input to a gate that can be skipped — which is #207 exactly, reintroduced
    by a change that would look like a tidy-up.
    """
    workflow = yaml.safe_load(CI.read_text(encoding="utf-8"))
    job = workflow["jobs"].get("repo-gates")
    assert job is not None, "the `repo-gates` job is gone; the SSOT gates with it"
    assert "if" not in job, (
        "`repo-gates` has grown an `if:`. Either drop it, or put the SSOT paths "
        "back in `code` — they cannot be undeclared AND skippable."
    )
    assert "needs" not in job, (
        "`repo-gates` now depends on another job, so a failure upstream skips "
        "the SSOT gates entirely. Same bargain as the `if:` above."
    )


@pytest.mark.parametrize("path", BUILDLESS_SSOTS)
def test_buildless_ssots_need_no_heavy_job(
    filters: dict[str, list[str]], path: str
) -> None:
    """...and the point of it: these no longer buy a wheel build.

    Guarded rather than merely done, because the natural repair for "did my
    gate run?" is to add the path back here, which silently restores the
    two-minute wait this split removed.
    """
    assert not _filter_matches(filters["code"], path), (
        f"{path} fires the heavy jobs again, but its gate runs in `repo-gates`, "
        "which already runs unconditionally — the wheel build buys nothing"
    )


# --- the audit, made permanent ------------------------------------------------


def test_every_linted_file_fires_code(filters: dict[str, list[str]]) -> None:
    """Every linted file must still fire a job that can catch a lint failure.

    `ruff check .` and `ruff format --check .` moved to `repo-gates`, which runs
    unconditionally — so strictly this can no longer fail for the reason it was
    written. It is kept, retargeted, because `tools/**` remains in `code` for a
    different and still-live reason: heavy gates EXECUTE those scripts
    (gen_census, wheel_smoke, xcheck, check_msrv, check_public_api). If a future
    edit narrows `tools/**`, this is what notices that the executing gates lost
    their trigger.

    Derived from ruff's own file list rather than a hand-written one: a new
    script under `tools/` must either be covered by an existing rule or fail
    here, which is the only way this stays true as the repo grows.
    """
    ruff = shutil.which("ruff")
    assert ruff, (
        "ruff is a dev dependency and a CI gate; without it this check cannot "
        "run, and a check that cannot run must not pass silently"
    )
    listed = subprocess.run(
        [ruff, "check", ".", "--show-files"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    linted = [
        str(Path(line).resolve().relative_to(REPO)) for line in listed if line.strip()
    ]
    assert linted, "ruff reported no files to lint — the check did not run"

    missed = sorted(f for f in linted if not _filter_matches(filters["code"], f))
    assert not missed, (
        f"{len(missed)} linted file(s) fire no heavy job. Lint itself is safe "
        f"(`repo-gates` is unconditional), but a script the python/rust jobs "
        f"EXECUTE would now change without re-running them: {missed[:10]}"
    )


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
