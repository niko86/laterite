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

WHY THE BLOCK ITSELF IS STILL DECLARED, NOT DERIVED (#313). The obvious repair
for a hand-maintained list that needs its own test suite is to derive it —
compute each job's required outputs from the paths it reads. It was considered
and rejected, and this note is here so it is not re-proposed as if it were new.
Deriving reproduces the hand-maintained declaration ONE LAYER UP: something
still has to state which paths a job reads, and that statement needs its own
test to police it. The complaint was never that the list is declared. It was
that the declaration was only checkable HALFWAY — the patterns were guarded and
the wiring under them was not. Closing it end-to-end is what derivation was
really being asked to buy, and it is what the wiring tests below do.

TWO SHAPES OF BLIND GATE, because they need different questions asked (#295):

  NARROWED INPUT    — the gate runs, and silently sees less than its name
                      claims. `check_dropin_surface` skipping `ast.Assign`;
                      `check_doc_refs` requiring a `/` in a backticked token.
                      The fix is for a gate that drops input to SAY what it
                      dropped, so the scope is visible the first time it runs.

  MISMATCHED TRIGGER — the gate is correct and never fires on a change it
                      should judge. #207; and the two wasm release ceilings
                      riding in `ts-lint` on a TS filter (#455). Reporting
                      dropped input cannot surface this: on a run where the
                      gate never fired there is no report, and on one where it
                      did it dropped nothing. The fix is to state what a gate
                      READS and check the trigger covers it — which is this
                      file's whole job for the one gate it can see.

`_matches` reimplements the picomatch subset these filters use. It was verified
against the real `picomatch` (via `web/node_modules`) over every tracked file
in the repo — exact agreement, including the negation semantics that produced
#230. Re-run whenever a PATTERN is added — not whenever a filter is, which is
what let this go stale — and say what it covered, because the number is the
only thing that shows the claim is not stale:

  #230 (four filters)   1385 paths x every pattern, exact agreement
  #313 (five filters)   1453 paths x 40 patterns = 58120 pairs, exact agreement
                        — `prose` joined for the cadence gate
  #313 (now gated)      1653 paths x 42 patterns = 69426 pairs, exact agreement
                        — two patterns had landed unre-run. That run held under
                          picomatch's `dot: true`, which is what
                          dorny/paths-filter matches with; at the default the
                          two parted company on 32 dotfile pairs, every one of
                          the `rust-packages/** vs .../.gitignore` shape.
  #494 (tools narrowed) 1684 paths x 48 patterns = 80832 pairs, exact agreement
                        — `tools/**` split into the nine scripts a gated job
                          runs, so the count went UP while the filter narrowed.
                          `dot: true`, as above.

Only the PATTERN COUNT in the newest entry is gated, by
`test_cross_check_series_is_current`. The path count and the pair total are
log, not assertion — the path count moves with every file added to the repo,
and gating it would demand a re-run, needing `web/node_modules`, on commits
that cannot have changed the answer. So the entries above are history and the
count is the one thing kept honest about the present.
"""

from __future__ import annotations

import re
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


# --- the cross-check's own staleness ------------------------------------------
#
# The series in this module's docstring is the only evidence `_matches` still
# agrees with picomatch. It went stale exactly as you would expect a manual
# protocol to: its trigger was "whenever a FILTER is added" while what
# invalidates it is a new PATTERN, so two patterns landed unre-run and the
# newest entry was being read as a statement about the present (#313).
#
# The cross-check stays manual — it needs `web/node_modules` for the real
# picomatch, which no Python job has. What is gated is the CLAIM: if the block
# has moved since the last run, this fails and names the number to re-run at.

SERIES_ENTRY = re.compile(r"#\d+\s*\([^)]*\)\s+(\d+) paths x (\d+) patterns")


def test_cross_check_series_is_current(filters: dict[str, list[str]]) -> None:
    """The newest series entry must describe the block as it is now."""
    entries = SERIES_ENTRY.findall(__doc__ or "")
    assert entries, (
        "the picomatch cross-check series has gone from this module's "
        "docstring — without it nothing shows `_matches` was ever verified"
    )
    claimed = int(entries[-1][1])
    live = sum(len(patterns) for patterns in filters.values())
    assert claimed == live, (
        f"the cross-check's newest entry covers {claimed} patterns; the filters "
        f"block now has {live}. Re-run it against the real picomatch and add a "
        "series entry — the old number is not a statement about the present."
    )


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


# --- it still narrows: paths that must NOT buy the heavy python job ----------


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
        # Tooling a heavy job executes or opens. The other four names that were
        # here — gen_changelog, gen_observations, check_doc_refs, and a
        # `tools/xcheck/run.py` the repo has never had — asserted that `code`
        # covered them, which `tools/**` did, for gates that all run in
        # `repo-gates`. They are in BUILDLESS_TOOLS below now, asserting the
        # opposite, which is the claim that was true all along.
        "tools/gen_census.py",
        "tools/gen_doc_outputs.py",
        "tools/generate_pyi.py",
        "tools/release/public-api/laterite.txt",
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
    wheel build this split removed.
    """
    assert not _filter_matches(filters["code"], path), (
        f"{path} fires the heavy jobs again, but its gate runs in `repo-gates`, "
        "which already runs unconditionally — the wheel build buys nothing"
    )


# --- the wiring, one layer below the patterns ---------------------------------
#
# Everything above guards the PATTERNS. Nothing guarded what carries their
# result to the jobs, and every hop on that path fails the same silent way —
# an unresolved GitHub expression is the empty string, the condition is false,
# and the job simply never runs. #207 with green tests.
#
#   filters block   `code:` …            the pattern lists above
#         |         changes.outputs.code: ${{ steps.filter.outputs.code }}
#         v         `if: needs.changes.outputs.code == 'true'`   + needs: changes
#   the job
#
# The middle hop is why these tests read the mapping rather than the filter
# names directly: an `if:` that names a declared OUTPUT proves nothing if that
# output is wired to a filter that does not exist. Checking the ends without
# the middle would report green on exactly the typo it exists to catch. The
# `needs:` edge is a fourth way to reach the same skip, off the value path
# rather than on it, and is checked last below.
#
# Deliberately NOT a job-to-paths model — that would reproduce the declaration
# one layer up and need its own test to police it. See the derivation note in
# this module's docstring.

OUTPUT_REF = re.compile(r"needs\.changes\.outputs\.([A-Za-z0-9_-]+)")
FILTER_REF = re.compile(r"steps\.filter\.outputs\.([A-Za-z0-9_-]+)")


@pytest.fixture(scope="module")
def workflow() -> dict:
    return yaml.safe_load(CI.read_text(encoding="utf-8"))


@pytest.fixture(scope="module")
def declared(workflow: dict) -> dict[str, str]:
    outputs = workflow["jobs"]["changes"].get("outputs")
    assert outputs, "the `changes` job publishes no outputs; nothing can gate"
    return outputs


@pytest.fixture(scope="module")
def mentioned() -> set[str]:
    """Outputs named anywhere in the raw file, comments included.

    The permissive end. Used only where a SUPERSET is the safe direction: for
    "is this reference declared?", reading prose too can raise a false alarm
    but cannot miss a real one.
    """
    return set(OUTPUT_REF.findall(CI.read_text(encoding="utf-8")))


@pytest.fixture(scope="module")
def consumed(workflow: dict) -> set[str]:
    """Outputs read by a real condition — job `if:` or step `if:`, nothing else.

    The strict end, and it has to be strict: "is this output read by anybody?"
    asserts an ABSENCE, and a scan that counts a mention in a YAML comment or a
    commented-out job would let a dead output pass the one check that exists to
    find it. Same trap as any gate whose input is wider than its claim (#295).
    """
    out: set[str] = set()
    for job in workflow["jobs"].values():
        conditions = [job.get("if", "")]
        conditions += [step.get("if", "") for step in job.get("steps") or []]
        for condition in conditions:
            out.update(OUTPUT_REF.findall(str(condition)))
    return out


def test_consumed_outputs_are_declared(
    declared: dict[str, str], mentioned: set[str]
) -> None:
    """Every `needs.changes.outputs.X` names an output `changes` publishes."""
    assert mentioned, "nothing references the filter outputs — the scan found none"
    undeclared = sorted(mentioned - set(declared))
    assert not undeclared, (
        f"these are read but never published by `changes`: {undeclared}. Each "
        "resolves to the empty string, so its condition is false and the job "
        "never runs, with every other test here still green (#207)"
    )


def test_declared_outputs_are_consumed(
    declared: dict[str, str], consumed: set[str]
) -> None:
    """...and nothing is published that no job reads.

    The other direction, and the cheaper failure: a dead output is a filter
    someone believed was gating something. It costs nothing at runtime, which
    is exactly why it survives.
    """
    unread = sorted(set(declared) - consumed)
    assert not unread, (
        f"these outputs are published and read by no job's `if:`: {unread}. "
        "Either a job lost its condition, or the filter is no longer earning "
        "its place"
    )


def test_consumers_declare_the_dependency(workflow: dict) -> None:
    """A job reading the outputs must also `needs: changes`.

    The fourth hop, and the one that looks least like a bug: the `if:` is
    spelled correctly, the output exists and is wired to a real filter — but
    without the dependency GitHub has nothing to substitute, so the expression
    is the empty string and the job never runs. #207 again, reached by a route
    none of the three checks above can see.
    """
    missing = []
    for name, job in workflow["jobs"].items():
        reads = OUTPUT_REF.findall(str(job.get("if", "")))
        reads += [
            ref
            for step in job.get("steps") or []
            for ref in OUTPUT_REF.findall(str(step.get("if", "")))
        ]
        needs = job.get("needs") or []
        needs = [needs] if isinstance(needs, str) else needs
        if reads and "changes" not in needs:
            missing.append(name)
    assert not missing, (
        f"these jobs read `needs.changes.outputs.*` without `needs: changes`: "
        f"{missing}. The reference resolves to nothing and the job is skipped"
    )


def test_declared_outputs_are_wired_to_real_filters(
    declared: dict[str, str], filters: dict[str, list[str]]
) -> None:
    """The middle hop: each output must resolve to a filter that exists.

    `code: ${{ steps.filter.outputs.cdoe }}` publishes an output whose name is
    right and whose value is always empty. Both tests above pass on it.
    """
    for name, expression in declared.items():
        referenced = FILTER_REF.findall(str(expression))
        assert referenced, (
            f"`changes.outputs.{name}` does not read `steps.filter.outputs.*` "
            f"at all: {expression!r}"
        )
        missing = sorted(set(referenced) - set(filters))
        assert not missing, (
            f"`changes.outputs.{name}` reads {missing}, which the filters block "
            f"does not define — it can only ever be empty. Declared: "
            f"{sorted(filters)}"
        )


def test_every_filter_reaches_a_job(
    declared: dict[str, str], filters: dict[str, list[str]]
) -> None:
    """And the far end: a filter nobody publishes is a list nothing reads.

    `dorny/paths-filter` computes every filter whether or not it is exposed,
    so this cannot fail loudly at runtime — the pattern list just sits there
    looking load-bearing.
    """
    exposed = {f for e in declared.values() for f in FILTER_REF.findall(str(e))}
    stranded = sorted(set(filters) - exposed)
    assert not stranded, (
        f"these filters are computed and never published: {stranded}. Nothing "
        "can gate on them, so their patterns are maintained for no reader"
    )


# --- the audit, made permanent ------------------------------------------------


TOOL_INVOCATION = re.compile(r"\btools/[A-Za-z0-9_/.-]+\.(?:py|sh|mjs)\b")


def _gated_jobs(workflow: dict) -> dict[str, set[str]]:
    """Every job with a path gate, mapped to the filters it ORs over.

    The `if:` expressions are all of one shape — `!cancelled() && (not a PR ||
    changes did not run || <filter> == 'true' || …)`. OR, so a job runs when ANY
    of its filters is true, and a tool it invokes needs a rule in any ONE of
    them. Reading the names out of the expression rather than restating them
    keeps this honest when a job's gate is widened.
    """
    out: dict[str, set[str]] = {}
    for name, job in workflow["jobs"].items():
        names = set(OUTPUT_REF.findall(str(job.get("if", ""))))
        if names:
            out[name] = names
    return out


def test_every_executed_tool_fires_its_job(
    workflow: dict, filters: dict[str, list[str]]
) -> None:
    """A gated job must be triggered by every `tools/` script it runs.

    This replaces a check derived from `ruff --show-files`, which asked whether
    a linted file fires `code`. That question stopped being the right one when
    ruff moved to the unconditional `repo-gates` job — but the answer it
    happened to give was load-bearing, because `code` listed `tools/**` and the
    lint check was the only thing holding the glob in place. Removing the glob
    without replacing the check would have left every executing gate trusting a
    hand-written list, which is #207 waiting to happen.

    So the question is asked properly instead: for each gated job, take the
    `tools/` scripts its own `run:` lines name and require the job's filters to
    reach them. Nothing is hand-listed — a step added tomorrow is covered the
    day it lands, and a filter narrowed past one of its gates fails here rather
    than in six months on the PR that needed the gate.

    It found a live one on the commit that introduced it: the `node` job runs
    `gen_doc_outputs.py --check --surface node` and no rule in `node` named the
    generator, so editing it re-ran the python surface's drift gate and not the
    node one.
    """
    missed: list[str] = []
    for job, names in _gated_jobs(workflow).items():
        steps = workflow["jobs"][job].get("steps") or []
        scripts = {
            s
            for step in steps
            for s in TOOL_INVOCATION.findall(str(step.get("run", "")))
        }
        missed.extend(
            f"{job} runs {script}, gated on {sorted(names)}"
            for script in sorted(scripts)
            if not any(_filter_matches(filters[n], script) for n in sorted(names))
        )

    assert not missed, (
        "a gated job invokes a tool that no rule in its own filters reaches, so "
        "editing that tool will not re-run the gate that executes it (#207):\n  "
        + "\n  ".join(missed)
    )


# Tools whose only gate is an UNFILTERED job — `repo-gates` here, or a workflow
# with no path filter at all (`wiki-lint.yml`, `nightly.yml`). Same bargain as
# BUILDLESS_SSOTS above and guarded for the same reason: the natural repair for
# "did my gate run?" is to add the path back to `code`, which buys a cargo
# build, the cdylib, pytest and a wheel smoke for a script no heavy job touches.
#
# Judgement, not derivation, and that is deliberate: a tool can also reach a
# heavy job by being READ rather than run (`generate_pyi.py` is imported by a
# test in the python job), and no scan of `run:` lines can see that. Adding a
# name here is a claim that nothing in a filtered job opens it.
BUILDLESS_TOOLS = [
    "tools/check_doc_refs.py",  # repo-gates
    "tools/check_issue_refs.py",  # repo-gates
    "tools/gen_changelog.py",  # repo-gates
    "tools/gen_crate_graph.py",  # repo-gates
    "tools/gen_install_channels.py",  # repo-gates
    "tools/gen_modality.py",  # repo-gates
    "tools/gen_observations.py",  # repo-gates
    "tools/release/trusted_publishing.py",  # no CI job at all — run by hand, once
    # The first tools added after `code` stopped listing `tools/**` (#271's
    # measurement harnesses). Under the old glob a bench script nobody runs in
    # CI bought a cargo build, the cdylib, pytest and a wheel smoke; these fire
    # only the cheap stdlib `cadence` job, which is the narrowing working.
    "tools/bench-cert-parse-share.py",
    "tools/bench-cert-python-routes.py",
    "tools/xcheck/emit_py.py",  # nightly.yml, which has no path filter
    "ags-wiki/.bootstrap/lint.py",  # wiki-lint.yml
    "ags-wiki/.bootstrap/reindex.py",  # wiki-lint.yml
    "examples/laterite_tour.py",  # a marimo notebook; no job runs it
]


@pytest.mark.parametrize("path", BUILDLESS_TOOLS)
def test_buildless_tools_need_no_heavy_job(
    filters: dict[str, list[str]], path: str
) -> None:
    assert not _filter_matches(filters["code"], path), (
        f"{path} fires the heavy jobs, and no filtered job runs or reads it — "
        "the cargo build, the cdylib, pytest and the wheel smoke all buy nothing"
    )


@pytest.mark.parametrize("path", BUILDLESS_TOOLS)
def test_buildless_tools_exist(path: str) -> None:
    """...and each one names a real file.

    A path that no longer exists asserts nothing while passing — which is how
    `tools/xcheck/run.py` sat in the gate-inputs list next to this one, naming a
    file the repo has never had.
    """
    assert (REPO / path).is_file(), (
        f"{path} does not exist, so the claim above is vacuous — delete the "
        "entry or fix the path"
    )


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
