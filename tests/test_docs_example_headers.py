"""Do the docs examples run in the environment their own header declares?

`tests/test_docs_examples.py` runs every example under the interpreter running
pytest — the dev venv, where the root `pyproject.toml` pins pandas and pyarrow.
Each example also carries a PEP 723 `# /// script` header naming its own
dependencies, and that header is never the environment that gate runs in. So a
header could be missing a dependency and nothing could say (#514).

Proven with a positive control before this file existed:
`ex06_sql_join.py` declared `laterite==<product>` and its own docstring says to
run it with `uv run`. Under the dev venv it exited 0; under its own header it
died with `ModuleNotFoundError: No module named 'pyarrow'` — the same script,
green for the gate and broken for the reader, which is the whole shape of the
defect. `.sql()` returns a **DuckDB** relation and DuckDB's `.pl()` imports
pyarrow; laterite's own materialisers are pyarrow-free exactly as
`concepts/dependency-shape.md` claims.

TWO GATES, TWO GUARANTEES, AND NEITHER SUBSUMES THE OTHER. The sibling asserts
*these examples work against this working tree* — the locally built wheel, so a
breaking change in the tree turns an example red, which is the regression it was
created for. This one asserts *these examples work in the environment they
publish*, which is a claim about the RELEASE, because the header pins one. Making
the sibling honour the header would have traded the first guarantee for the
second rather than adding it.

## Why a failure here is not automatically a defect

Because the header pins the released wheel, this module runs the docs against the
RELEASE while the tree usually sits ahead of it. `ex18_severity_tiers.py` was red
on the first run for `Report.warnings`, which exists in this tree and not in the
release — a fact about the calendar, not about the header.

So a failure is re-run with the same pin widened to `laterite[all]`, and the two
outcomes mean different things:

  passes with [all]  → the header is missing an extra. Permanent, reader-facing,
                       and no release fixes it. FATAL.
  still fails        → the script needs something the pinned release does not
                       have, or the pin does not resolve at all (the window
                       between a version bump and its publish). REPORTED, skipped.

The classifier only answers when the extras alone decide it: a script that both
misses an extra AND uses unreleased API lands in the second bucket and its header
defect waits for the release. Stated rather than discovered later.

## Running it

Costs an isolated environment per example, so it is opt-in and the nightly's
`docs-example-headers` job is what sets the switch::

    LATERITE_DOCS_HEADER_ENV=1 uv run pytest tests/test_docs_example_headers.py -q -rs

Unset, the runs skip and per-PR CI keeps answering the tree question only. The
two file-reading cases — the glob guard and the widening check — are NOT gated:
they cost nothing, and a malformed header should fail on the PR that adds it
rather than wait for a nightly.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
EXAMPLE_DIR = REPO_ROOT / "web" / "docs-site" / "examples" / "python"

EXAMPLES = sorted(EXAMPLE_DIR.glob("ex*.py"))

#: Room for a cold resolve plus the slowest example — `ex17_lock.py` runs an age
#: passphrase round trip whose KDF is deliberately slow — and deliberately far
#: BELOW the job's own `timeout-minutes`. A per-script ceiling near the job's
#: lets one hung example take the runner down with it, and a cancelled job emits
#: neither the `-rs` skip reasons nor the census, which are this leg's whole
#: report. The sibling gate uses the same order of magnitude.
_TIMEOUT = 300

#: A PEP 723 header block, so the widening below edits the HEADER and not the
#: first `"laterite…"` string anywhere in the file. `ex16_diff.py` has one on
#: line 36 — `"laterite demo site (…)"` — which a whole-file substitution would
#: happily rewrite the moment an example's header stopped naming laterite,
#: producing a corrupt copy whose failure reads as "not decided by the extras".
#: A misclassification in exactly the direction that hides a real defect.
#: `tests/test_version_faithful.py` scopes its own scan the same way.
_PEP723 = re.compile(r"^# /// script\n(?P<body>(?:^#.*\n)*?)^# ///$", re.M)

#: The laterite requirement inside that block, the specifier captured so the
#: widened re-run can replace the extras and keep the pin.
_LATERITE_DEP = re.compile(r'"laterite(?:\[[\w,]+\])?(?P<spec>[^"]*)"')

_ENABLED = os.environ.get("LATERITE_DOCS_HEADER_ENV", "") not in ("", "0")

#: What each example turned out to be, tallied as the cases run so the reporter
#: at the bottom can say what the run actually measured. A gate that classifies
#: has to publish its classification, or "green" and "measured nothing" look the
#: same from outside.
_CENSUS: dict[str, list[str]] = {"ran": [], "missing an extra": [], "undecided": []}

#: Examples that failed once and passed on the confirming re-run. Counted as
#: `ran`, and named anyway: a retry that nobody can see is a flake this gate has
#: agreed to stop reporting, and the day one becomes a real intermittent defect
#: the only trace of it is this line.
_RETRIED: list[str] = []

# Deliberately NOT `needs_env`: this module imports nothing built and re-enters
# nothing through this interpreter. It shells out to `uv`, which builds the
# environment the header asks for — that is the entire point, and inheriting this
# process's environment would be the bug. `tests/test_build_marker_faithful.py`
# enforces the inverse direction too, so a marker added here would fail. It also
# decides by TEXT SEARCH, which is why the prose above says "the interpreter
# running pytest" rather than naming the attribute: spelling it out marks this
# module as needing the built wheel, which it does not.
#
# The switch is per-TEST rather than on the module, because two of the cases here
# only read files: the glob guard and the widening check cost nothing, catch a
# malformed header the moment it lands, and would otherwise sit unrun until the
# next nightly.
_needs_uv = pytest.mark.skipif(
    not _ENABLED,
    reason="set LATERITE_DOCS_HEADER_ENV=1 — each example resolves its own environment",
)


def _uv() -> str:
    """`uv` on PATH, or a hard failure.

    Skipping here would turn a broken job wiring into a green run over nothing,
    which is the failure mode this repo keeps meeting. The switch above is the
    opt-in; once it is on, a missing `uv` is a defect in whoever set it.
    """
    found = shutil.which("uv")
    assert found, (
        "LATERITE_DOCS_HEADER_ENV is set but `uv` is not on PATH — this module "
        "cannot build a header's environment without it"
    )
    return found


def _run(script: Path) -> subprocess.CompletedProcess[str]:
    """Run one PEP 723 script in the environment its own header declares.

    `cwd` is the repo root for the same reason the sibling gate uses it: the
    examples read `examples/sample_site.ags` by a repo-relative path, and with the
    fixture present each example's network arm stays cold, so this leg pays for
    package installs and nothing else.

    `--exact` IS THE GATE. uv caches a script's environment against the script's
    PATH and, by default, only adds to it — a header that loses a dependency keeps
    the package a previous run installed. That made this very file report green
    on a deliberately broken `ex06` header while the same bytes at a fresh path
    failed, which is the defect being gated wearing the gate's own clothes. With
    `--exact` uv uninstalls the extraneous package first, so the environment is
    the header's and not the history's.
    """
    return subprocess.run(
        [_uv(), "run", "--exact", "--script", str(script)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=_TIMEOUT,
    )


def _widened(example: Path, into: Path) -> Path:
    """A copy of `example` whose laterite pin asks for every extra.

    The version is carried over untouched — widening the EXTRAS is the question,
    and rewriting the pin as well would answer a different one.
    """
    target = into / example.name
    target.write_text(_widen_text(example), encoding="utf-8")
    return target


def _widen_text(example: Path) -> str:
    """The widened source, split out so the check below can read it without a run."""
    text = example.read_text(encoding="utf-8")
    header = _PEP723.search(text)
    assert header, f"{example.name} has no PEP 723 header to widen"
    body, count = _LATERITE_DEP.subn(
        r'"laterite[all]\g<spec>"', header.group("body"), count=1
    )
    assert count == 1, f"{example.name}'s PEP 723 header names no laterite to widen"
    return text[: header.start("body")] + body + text[header.end("body") :]


def test_examples_are_discovered() -> None:
    """Zero is a bad witness: an empty glob would make every case below vacuous."""
    assert EXAMPLES, f"no docs examples found under {EXAMPLE_DIR}"


@pytest.mark.parametrize("example", EXAMPLES, ids=lambda p: p.name)
def test_widening_edits_the_header_and_only_the_header(example: Path) -> None:
    """The classifier's second run must differ from the first in the pin alone.

    A substitution that reached past the header would produce a corrupt copy, its
    run would fail for a reason having nothing to do with the extras, and a real
    header defect would be downgraded from FATAL to a skip — the misclassification
    this module exists to prevent, wearing its own verdict. It is not hypothetical
    at one remove: `ex16_diff.py` carries `"laterite demo site (…)"` in its body,
    which a whole-file scan rewrites the moment a header stops naming laterite.

    Read-only and unskipped, so a malformed header fails on the PR that adds it
    rather than in the next nightly.
    """
    original = example.read_text(encoding="utf-8")
    widened = _widen_text(example)

    assert widened != original or "[all]" in original, (
        f"{example.name}: widening changed nothing, so the classifier's two runs "
        "would be the same run and every failure would read as a header defect"
    )
    before, after = _PEP723.search(original), _PEP723.search(widened)
    assert before and after, f"{example.name} has no PEP 723 header"
    # Compared at each text's OWN header boundaries, not at a shared offset:
    # `[all]` is a different length from the extras it replaces, so everything
    # after it shifts, and an offset-shared comparison fails on every example
    # whatever the substitution did. (It did, on the first run of this check.)
    assert original[: before.start()] == widened[: after.start()], (
        f"{example.name}: widening edited bytes ABOVE the PEP 723 header"
    )
    assert original[before.end() :] == widened[after.end() :], (
        f"{example.name}: widening edited bytes BELOW the PEP 723 header, so the "
        "re-run would execute a script the docs do not publish"
    )
    assert '"laterite[all]' in after.group("body"), (
        f"{example.name}: the widened header does not ask for every extra, so the "
        "classifier's second run answers a different question from the one asked"
    )


def test_widening_refuses_a_header_that_names_no_laterite(tmp_path: Path) -> None:
    """The case the check above cannot reach, and the one that motivated scoping.

    While every header names laterite, a whole-file scan and a header-scoped one
    agree — the header is on line 3 and matches first either way, so the
    invariant above holds for both and proves nothing about which is in use. The
    disagreement needs a header WITHOUT a laterite requirement, and then a
    whole-file scan silently rewrites the first `"laterite…"` in the body
    instead. `ex16_diff.py` is the live example: `"laterite demo site (…)"` is
    fixture text it edits, and a copy with that string mangled fails for a reason
    unrelated to the extras — which the classifier reads as "not decided by the
    extras" and skips. A defect downgraded to a skip by the machinery meant to
    catch it.

    Refusing loudly is the correct behaviour: an example whose header lost its
    laterite requirement is a defect in its own right, and `_widen_text` is not
    the place to decide what it meant.
    """
    source = (EXAMPLE_DIR / "ex16_diff.py").read_text(encoding="utf-8")
    decoy = '"laterite demo site (synthetic starter - replace me)"'
    assert decoy in source, (
        "ex16_diff.py no longer carries the body string this case is built on — "
        'pick another example with a `"laterite…"` outside its header, or drop '
        "this test and say why in the module docstring"
    )
    stripped = tmp_path / "ex16_diff.py"
    stripped.write_text(
        source.replace(
            '# dependencies = ["laterite==', '# dependencies = ["polars==', 1
        ),
        encoding="utf-8",
    )

    with pytest.raises(AssertionError, match="names no laterite to widen"):
        _widen_text(stripped)


@_needs_uv
def test_the_runner_uses_the_header_environment() -> None:
    """The positive control, without which every green below means nothing.

    If `uv run --script` leaked the ambient interpreter's packages, this module
    would be a second, slower copy of `test_docs_examples.py` reporting the same
    green for the same reason — and the defect it exists for would still be
    invisible. A header declaring NO dependencies must therefore find none of the
    packages this process can import.

    **`pytest` is the canary, and it is here because the first cut probed only for
    laterite / pyarrow / pandas and therefore SKIPPED in the one job that runs
    this file.** The nightly starts from `uv run --no-project --with pytest`, so
    the ambient environment holds pytest and nothing else; the control was green
    by abstention exactly where it was needed, and passed locally only because a
    developer's environment has the other three. Whatever else is around, pytest
    is importable here by definition — this module is running under it.
    """
    import importlib.util

    ambient = [
        name
        for name in ("pytest", "laterite", "pyarrow", "pandas")
        if importlib.util.find_spec(name) is not None
    ]
    assert "pytest" in ambient, (
        "pytest is not importable in the process running pytest, so the control "
        "has no guaranteed canary and cannot tell isolation from an empty "
        "environment"
    )

    with tempfile.TemporaryDirectory() as tmp:
        probe = Path(tmp) / "control.py"
        probe.write_text(
            "# /// script\n"
            '# requires-python = ">=3.12"\n'
            "# dependencies = []\n"
            "# ///\n"
            "import importlib.util\n"
            "names = " + repr(ambient) + "\n"
            "leaked = [n for n in names if importlib.util.find_spec(n)]\n"
            'assert not leaked, f"leaked from the ambient environment: {leaked}"\n',
            encoding="utf-8",
        )
        proc = _run(probe)

    assert proc.returncode == 0, (
        "a script declaring no dependencies could still import "
        f"{ambient} — the runner is not honouring the header, so every other "
        f"result in this module is measuring the wrong environment.\n"
        f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
    )


@_needs_uv
@pytest.mark.parametrize("example", EXAMPLES, ids=lambda p: p.name)
def test_docs_example_runs_under_its_header(example: Path) -> None:
    proc = _run(example)
    if proc.returncode == 0:
        _CENSUS["ran"].append(example.name)
        return

    # CONFIRM THE FAILURE BEFORE CLASSIFYING IT. Both arms below are verdicts on
    # the header, and each is reached through a comparison of two runs — so a
    # single transient (a PyPI hiccup mid-resolve) that clears before the widened
    # run reads as "passes with [all]", which is the FATAL arm, naming a defect
    # that does not exist. The whole point of classifying is to be right about
    # which class a failure is in; a second attempt is cheap and only failures
    # pay for it.
    proc = _run(example)
    if proc.returncode == 0:
        _CENSUS["ran"].append(example.name)
        _RETRIED.append(example.name)
        return

    with tempfile.TemporaryDirectory() as tmp:
        widened = _run(_widened(example, Path(tmp)))

    if widened.returncode == 0:
        _CENSUS["missing an extra"].append(example.name)
        pytest.fail(
            f"{example.name} does not run in the environment its own PEP 723 "
            f"header declares, and DOES run once that pin is widened to "
            f"laterite[all] — so the header is missing an extra. A reader "
            f"following the docstring's `uv run` gets this:\n"
            f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
        )

    _CENSUS["undecided"].append(example.name)
    pytest.skip(
        f"{example.name} fails under its header AND under laterite[all], so the "
        f"extras are not what decides it: the pinned release is behind this tree "
        f"or does not resolve. Reported, not fatal — the tree question is "
        f"tests/test_docs_examples.py's.\n"
        f"--- stderr (header run) ---\n{proc.stderr.strip()[-800:]}"
    )


@_needs_uv
def test_the_run_says_what_it_measured(capsys: pytest.CaptureFixture[str]) -> None:
    """Last by position, because it reports on the cases above.

    The undecided bucket is the one that can hollow this gate out: while the tree
    sits ahead of the release — which is most of the time — an example can fail
    for a reason the extras do not decide, and a run where EVERY example lands
    there has measured nothing about any header while reporting green. That is
    the shape this repo keeps re-learning, so it is asserted rather than left to
    whoever reads the skips.

    It fires for real in the window between a version bump and its publish, when
    the pin resolves to nothing: the docs then pin a laterite that PyPI does not
    have, and a nightly saying so out loud is the point rather than the cost.
    """
    measured = sum(len(v) for v in _CENSUS.values())
    with capsys.disabled():
        print(
            f"\n[header environments] {len(_CENSUS['ran'])} ran, "
            f"{len(_CENSUS['missing an extra'])} missing an extra, "
            f"{len(_CENSUS['undecided'])} not decided by the extras "
            f"({', '.join(_CENSUS['undecided']) or 'none'})"
        )
        if _RETRIED:
            print(
                f"[header environments] passed only on the confirming re-run: "
                f"{', '.join(_RETRIED)} — transient, or an intermittent defect "
                f"this gate has just absorbed"
            )
        if measured != len(EXAMPLES):
            print(
                f"[header environments] PARTIAL RUN — {measured} of "
                f"{len(EXAMPLES)} examples reached; the check below is skipped"
            )
    if measured != len(EXAMPLES):
        return
    assert _CENSUS["ran"] or _CENSUS["missing an extra"], (
        "every example failed for a reason the extras do not decide, so this run "
        "measured no header at all. The usual cause is a pin that resolves to "
        "nothing — the docs ask for a laterite version PyPI does not have, which "
        "is the bump-to-publish window if a release is in flight and a real "
        "defect otherwise. The per-example skip reasons carry the resolver output."
    )
