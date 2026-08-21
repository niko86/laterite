"""The nightly's own wiring: can every leg run, can every leg be reported, and
does the one amnesty each leg computes reach every step that can be fatal?

Three defects in one night (#493), none of them in what the nightly measures —
all three in how it is wired:

  * `docs-vs-released-duckdb` never ran a test. Its pytest inherited
    `--benchmark-disable` from the root `pyproject.toml` into an ephemeral env
    with no pytest-benchmark, so the invocation exited 4 during argument
    parsing. A leg that cannot start looks, from the outside, exactly like a leg
    that found something.
  * `docs-vs-released-wheel` printed "failures below are reported, not fatal",
    having correctly determined that the checkout was ahead of the released tag,
    and then went red on the one step that never consumed the determination. The
    comment above that step said it was "split the same way as the two steps
    above". It was not, and no reader of the YAML would catch a missing key.
  * the tracking issue named one of the three failing legs. The other two were
    outside `notify`'s `needs` and therefore unreportable BY CONSTRUCTION —
    #295's shape in the reporter itself.

Each is a property of the workflow file rather than of any code, which is why
they survived: nothing reads a workflow except GitHub, and GitHub has no opinion
about whether a gate can report what it finds. This file reads it.

WHAT THIS FILE CANNOT SEE, stated because a gate that drops input has to say so:
it reads `nightly.yml` and nothing else. Another workflow could grow a pytest
invocation with the same trap (ci.yml explains it in place, twice, and carries
the neutraliser in both jobs that need it) and this file would stay green. The
count of what was scanned is printed on every run, pass or fail.
"""

from __future__ import annotations

import importlib.util
import re
from pathlib import Path
from typing import Any

import pytest
import yaml

REPO = Path(__file__).resolve().parents[1]
NIGHTLY = REPO / ".github" / "workflows" / "nightly.yml"


@pytest.fixture(scope="module")
def workflow() -> dict[str, Any]:
    return yaml.safe_load(NIGHTLY.read_text(encoding="utf-8"))


def _load_issue_tracker():
    """Import `tools/issue_tracker.py` — `tools/` is not a package. Same shape as
    tests/test_issue_tracker.py's loader, so there is one way to do this."""
    spec = importlib.util.spec_from_file_location(
        "issue_tracker", REPO / "tools" / "issue_tracker.py"
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _steps(job: dict[str, Any]) -> list[dict[str, Any]]:
    return [s for s in job.get("steps", []) if isinstance(s, dict)]


def _run_steps(workflow: dict[str, Any]) -> list[tuple[str, dict[str, Any]]]:
    """Every `run:` step in the file, tagged with its job name."""
    return [
        (job_name, step)
        for job_name, job in workflow["jobs"].items()
        for step in _steps(job)
        if "run" in step
    ]


# --- 1. a leg that cannot start ----------------------------------------------

#: What an invocation must carry to be immune to the repo-level `addopts`. The
#: root `pyproject.toml` sets `--benchmark-disable`, and pytest reads the inifile
#: from the ROOTDIR — which is this checkout — no matter which interpreter runs
#: it. Every pytest in this workflow runs from a venv built ad hoc from a
#: published artifact, so the plugin is never there to answer for the option.
#:
#: Both quotings and the split form are accepted because the shell accepts them:
#: a gate that demands one spelling of a working command teaches people to fight
#: it. What is NOT optional is the `-o` — `addopts=""` on its own is a shell
#: assignment pytest never sees.
NEUTRALISER = re.compile(r"""-o[= ]\s*addopts=(""|''|"" |'' )""")
NEUTRALISER_HUMAN = '-o addopts=""'

#: Invocations exempt from the rule, each with the reason it is exempt. Empty,
#: and declared rather than derived on purpose: an exemption is a decision
#: somebody made, and the next person needs to read it rather than infer it from
#: a pattern that happens not to match.
ADDOPTS_EXEMPT: dict[str, str] = {}

#: Where one shell command ends and the next begins, closely enough for a
#: workflow's `run:` block. Deliberately not a shell parser: a mention of pytest
#: is not an invocation of it (`uv pip install … pytest` is how the wheel leg's
#: env gets the runner) and telling the two apart is the whole job here.
_COMMAND_BREAK = re.compile(r"\n|&&|\|\||;|(?<![|>])\|(?!\|)")

#: `uv run` flags that swallow the next token, so the wrapper can be skipped to
#: find the program actually being run.
_UV_VALUE_FLAGS = frozenset(
    {"--with", "--python", "--project", "--directory", "--group", "--extra"}
)


def _program(tokens: list[str]) -> str | None:
    """The program a command runs, seeing through a `uv run` wrapper.

    `uv run --no-project --with pytest pytest …` runs pytest; the `--with pytest`
    before it does not. Without the skip, the flag's VALUE would answer for the
    program and every such command would look like an invocation.
    """
    if not tokens:
        return None
    if tokens[0] != "uv" or len(tokens) < 2 or tokens[1] != "run":
        return tokens[0].rsplit("/", 1)[-1].strip("\"'")
    i = 2
    while i < len(tokens) and tokens[i].startswith("-"):
        i += 2 if tokens[i] in _UV_VALUE_FLAGS else 1
    return tokens[i].rsplit("/", 1)[-1].strip("\"'") if i < len(tokens) else None


def pytest_invocations(run: str) -> list[str]:
    """The commands in a `run:` block that actually start pytest."""
    found = []
    for command in _COMMAND_BREAK.split(run.replace("\\\n", " ")):
        tokens = command.split()
        interpreted = any(
            tokens[i] == "-m" and tokens[i + 1] == "pytest"
            for i in range(len(tokens) - 1)
        )
        if interpreted or _program(tokens) == "pytest":
            found.append(command)
    return found


@pytest.mark.parametrize(
    ("command", "is_invocation"),
    [
        ('"$WS/.released/bin/python" -m pytest tests/x.py -q', True),
        ("pytest tests/x.py -q", True),
        ("uv run --no-project --with pytest pytest -o addopts='' tests", True),
        (
            'uv pip install --python "$WS/.released/bin/python" "laterite[all]" pytest',
            False,
        ),
        ("uv run --no-project python tools/gen_observations.py --check", False),
        ("echo 'pytest exits 4 on an unrecognised argument'", False),
    ],
)
def test_a_mention_of_pytest_is_not_an_invocation_of_it(
    command: str, *, is_invocation: bool
) -> None:
    """The gate above is only as good as this distinction. Its first draft matched
    the word, and the wheel leg's `uv pip install … pytest` came back as an
    invocation with no `-o addopts=""` on it — a gate failing on the one step
    that could not possibly carry the flag."""
    assert bool(pytest_invocations(command)) is is_invocation


def test_every_pytest_invocation_neutralises_the_repo_addopts(
    workflow: dict[str, Any], capsys: pytest.CaptureFixture[str]
) -> None:
    run_steps = _run_steps(workflow)
    invocations = [
        (job, step, command)
        for job, step in run_steps
        for command in pytest_invocations(step["run"])
    ]

    # The scope report — what this gate looked at, and what it did not. Printed
    # whether or not it passes, because a filter nobody can see is a blind spot
    # with a green tick on it.
    with capsys.disabled():
        print(
            f"\n[nightly addopts] {len(invocations)} pytest invocation(s) across "
            f"{len(run_steps)} run-steps in nightly.yml; "
            f"{len(ADDOPTS_EXEMPT)} declared exemption(s); "
            f"other workflows not scanned"
        )

    assert invocations, (
        "no pytest invocation found in nightly.yml — either the legs moved or "
        "this gate stopped recognising its subject; both need a reader"
    )

    for job, step, command in invocations:
        name = step.get("name", "<unnamed>")
        if name in ADDOPTS_EXEMPT:
            continue
        assert NEUTRALISER.search(command), (
            f"{job} / {name!r} invokes pytest without `{NEUTRALISER_HUMAN}`. The root "
            f'pyproject sets `addopts = "--benchmark-disable"`; an ephemeral env '
            f"without pytest-benchmark exits 4 on it before collecting anything "
            f"(#493). Reproduce locally with `-p no:benchmark`."
        )


def test_the_wheel_leg_does_not_neutralise_addopts_by_installing_a_plugin(
    workflow: dict[str, Any],
) -> None:
    """`pytest-benchmark` in the install step would satisfy `--benchmark-disable`
    without saying it was doing so — a neutraliser nobody can see, and the reason
    the DuckDB leg was built without one at all. No test in this workflow uses
    the plugin; the flag is the whole mechanism."""
    installs = [
        step["run"]
        for _, step in _run_steps(workflow)
        if "pytest-benchmark" in step["run"]
    ]
    assert not installs, (
        "a nightly step installs pytest-benchmark; if a benchmark really is "
        "needed here, say so at the invocation rather than letting the install "
        "double as the addopts neutraliser"
    )


# --- 2. one determination, every fatal step ----------------------------------

#: The `continue-on-error` expression a leg's fatal steps use to read its
#: tree-ahead determination. Matched exactly: an expression this file cannot
#: evaluate must fail loudly rather than pass by not matching anything.
_TREE_AHEAD = re.compile(
    r"^\$\{\{\s*steps\.under-test\.outputs\.tree_ahead\s*==\s*'true'\s*\}\}$"
)


def continue_on_error(value: object, *, tree_ahead: bool) -> bool:
    """Evaluate a step's `continue-on-error` for one state of the determination.

    True means "this step cannot fail the job". Only the two forms the workflow
    actually uses are understood; anything else raises, so a new expression gets
    a reader rather than a silent pass.
    """
    if value is None:
        return False
    if isinstance(value, bool):
        return value
    if isinstance(value, str) and _TREE_AHEAD.match(value.strip()):
        return tree_ahead
    raise AssertionError(f"unrecognised continue-on-error expression: {value!r}")


def _legs_with_a_determination(workflow: dict[str, Any]) -> list[str]:
    """Jobs whose `under-test` step publishes `tree_ahead`."""
    found = []
    for job_name, job in workflow["jobs"].items():
        for step in _steps(job):
            if step.get("id") == "under-test" and "tree_ahead=" in step.get("run", ""):
                found.append(job_name)
                break
    return found


def _steps_after_the_determination(job: dict[str, Any]) -> list[dict[str, Any]]:
    """EVERY step below the determination, `run` and `uses` alike.

    Filtering to `run` steps would be the same blind spot one level down: a
    `uses:` step added after `under-test` can fail the job just as hard, and a
    gate meant to prove nothing downstream is fatal must not decide for itself
    which downstream steps count.
    """
    steps = _steps(job)
    at = next(i for i, s in enumerate(steps) if s.get("id") == "under-test")
    return steps[at + 1 :]


#: The legs that determine whether this checkout is the released tag. Declared
#: once and pinned by the test below, so a third one arriving is a decision
#: somebody records rather than a parametrize list somebody forgets.
DETERMINING_LEGS = ("docs-vs-released-wheel", "docs-vs-released-npm")


def test_the_determining_legs_are_the_ones_declared(workflow: dict[str, Any]) -> None:
    """Set equality, not sequence: job ORDER in the workflow is not governance,
    and a gate that reds on a reordering is a gate people learn to ignore."""
    assert set(_legs_with_a_determination(workflow)) == set(DETERMINING_LEGS)


@pytest.mark.parametrize("leg", DETERMINING_LEGS)
def test_tree_ahead_amnesties_every_step_that_could_be_fatal(
    workflow: dict[str, Any], leg: str
) -> None:
    """AHEAD of the released tag: nothing downstream of the determination may
    fail the job. This is the direction that was broken — the wheel leg's CLI
    write-mode step had no `continue-on-error` at all, so an unreleased CLI
    change was fatal on a run whose own banner said it would not be.

    Nothing is filtered out here, which is the point: this is the half that has
    to be exhaustive."""
    for step in _steps_after_the_determination(workflow["jobs"][leg]):
        assert continue_on_error(step.get("continue-on-error"), tree_ahead=True), (
            f"{leg} / {step.get('name')!r} stays fatal when the checkout is ahead "
            f"of the released tag, contradicting what the `under-test` step prints"
        )


#: The steps that may stay non-fatal at the released tag, named in full per leg.
#: A committed `.out` that no longer byte-matches is expected drift, never a
#: defect — but that is a decision about two specific steps, so it is written
#: down as one. Deriving it from `(informational)` in the step NAME, which is how
#: this started, makes the opt-out reachable by rename: appending the word to a
#: step would quietly excuse it from the only rule holding the other direction of
#: the amnesty, and nothing would say so. Same argument as `ADDOPTS_EXEMPT` and
#: `TRACKER_EXCLUDED`, and it applies harder here because this table is not empty.
INFORMATIONAL_STEPS: dict[str, frozenset[str]] = {
    "docs-vs-released-wheel": frozenset(
        {
            "Committed .out still byte-matches (informational)",
            "CLI examples still byte-match the wheel's own `lat` (informational)",
        }
    ),
    "docs-vs-released-npm": frozenset(
        {"Committed .out still byte-matches (informational)"}
    ),
}


@pytest.mark.parametrize("leg", DETERMINING_LEGS)
def test_at_the_released_tag_the_actionable_steps_are_still_fatal(
    workflow: dict[str, Any], leg: str, capsys: pytest.CaptureFixture[str]
) -> None:
    """AT the released tag: an example that fails to run is a real defect and must
    take the job down. The other direction, demonstrated rather than assumed —
    "always non-fatal" would pass the test above and destroy the leg."""
    steps = _steps_after_the_determination(workflow["jobs"][leg])
    declared = INFORMATIONAL_STEPS[leg]
    names = {s.get("name", "") for s in steps}
    assert declared <= names, (
        f"{leg} declares informational step(s) it no longer has: "
        f"{sorted(declared - names)}. A renamed step is not an excused one — "
        f"update INFORMATIONAL_STEPS deliberately or let the rule apply."
    )
    actionable = [s for s in steps if s.get("name", "") not in declared]
    with capsys.disabled():
        print(
            f"\n[nightly amnesty] {leg}: {len(actionable)} of {len(steps)} steps "
            f"below the determination are actionable; "
            f"{len(declared)} declared informational"
        )
    assert actionable, f"{leg} has no fatal step left — the determination is decorative"
    for step in actionable:
        assert not continue_on_error(step.get("continue-on-error"), tree_ahead=False), (
            f"{leg} / {step.get('name')!r} cannot fail even when this checkout IS "
            f"the released tag; the amnesty is for the tree-ahead window only"
        )


# --- 3. what the tracker can see ---------------------------------------------

#: Jobs deliberately kept out of the tracking issue, each with the reason. Empty:
#: every leg reports. An entry here is a decision that a class of failure will
#: reach nobody but an email, and it has to be written down as one — that it was
#: previously an omission rather than a statement is the defect #493 fixed.
TRACKER_EXCLUDED: dict[str, str] = {}


def test_the_tracker_sees_every_nightly_job(
    workflow: dict[str, Any], capsys: pytest.CaptureFixture[str]
) -> None:
    """`notify` drives the issue from `toJSON(needs)`, so its dependency set is
    exactly what the tracker can report. A job outside it can fail every night
    forever without appearing anywhere."""
    jobs = set(workflow["jobs"]) - {"notify"}
    needs = set(workflow["jobs"]["notify"]["needs"])

    with capsys.disabled():
        print(
            f"\n[nightly tracker] {len(needs)} of {len(jobs)} jobs report to the "
            f"tracking issue; excluded: "
            f"{', '.join(sorted(TRACKER_EXCLUDED)) or '(none)'}"
        )

    assert jobs - needs == set(TRACKER_EXCLUDED), (
        "a nightly job cannot reach the tracking issue. Add it to `notify`'s "
        "`needs`, or declare it in TRACKER_EXCLUDED with the reason it reports "
        "to nobody."
    )
    assert needs - jobs == set(), "`notify` depends on a job that no longer exists"


def test_a_docs_leg_failing_alone_opens_an_issue_that_names_it(
    workflow: dict[str, Any],
) -> None:
    """The acceptance criterion end to end, over the REAL dependency set.

    `plan()` was never wrong — it reports whatever it is handed, and it was handed
    seven of twelve jobs. Exercising it against a hand-written `needs` dict would
    prove nothing about that, which is the whole defect; so the input here is
    built from the workflow, exactly as GitHub builds it. Drop the leg from
    `notify`'s `needs` and this goes red because the failure never appears in the
    context at all — a green tracker over a red run."""
    tracker = _load_issue_tracker()
    leg = "docs-vs-released-duckdb"
    # What the `NEEDS` env carries: one entry per dependency, and nothing else.
    context = {
        job: {"result": "failure" if job == leg else "success"}
        for job in workflow["jobs"]["notify"]["needs"]
    }

    action = tracker.plan(context, None, "https://example.invalid/run/1")

    assert action["action"] == "create", (
        f"a nightly in which only {leg} failed opens no tracking issue — the leg "
        f"cannot reach `notify`"
    )
    assert f"**Failing:** {leg}" in action["body"]
