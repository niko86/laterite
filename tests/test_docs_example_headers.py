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

Unset, the module skips — per-PR CI keeps answering the tree question only.
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

#: Long enough for a cold resolve plus the slowest example: `ex17_lock.py` runs
#: an age passphrase round trip whose KDF is deliberately slow, so a generous
#: ceiling here is not a hedge against a hang.
_TIMEOUT = 900

#: The laterite requirement inside a PEP 723 dependency list, extras captured
#: separately from the specifier so the widened re-run can replace the extras and
#: keep the pin. Same shape as `tests/test_version_faithful.py:_LATERITE_DEP`,
#: which holds that pin to the shipped version.
_LATERITE_DEP = re.compile(r'"laterite(?:\[[\w,]+\])?(?P<spec>[^"]*)"')

_ENABLED = os.environ.get("LATERITE_DOCS_HEADER_ENV", "") not in ("", "0")

#: What each example turned out to be, tallied as the cases run so the reporter
#: at the bottom can say what the run actually measured. A gate that classifies
#: has to publish its classification, or "green" and "measured nothing" look the
#: same from outside.
_CENSUS: dict[str, list[str]] = {"ran": [], "missing an extra": [], "undecided": []}

# Deliberately NOT `needs_env`: this module imports nothing built and re-enters
# nothing through this interpreter. It shells out to `uv`, which builds the
# environment the header asks for — that is the entire point, and inheriting this
# process's environment would be the bug. `tests/test_build_marker_faithful.py`
# enforces the inverse direction too, so a marker added here would fail. It also
# decides by TEXT SEARCH, which is why the prose above says "the interpreter
# running pytest" rather than naming the attribute: spelling it out marks this
# module as needing the built wheel, which it does not.
pytestmark = pytest.mark.skipif(
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
    text = example.read_text(encoding="utf-8")
    widened, count = _LATERITE_DEP.subn(r'"laterite[all]\g<spec>"', text, count=1)
    assert count == 1, f"{example.name} has no laterite requirement to widen"
    target = into / example.name
    target.write_text(widened, encoding="utf-8")
    return target


def test_examples_are_discovered() -> None:
    """Zero is a bad witness: an empty glob would make every case below vacuous."""
    assert EXAMPLES, f"no docs examples found under {EXAMPLE_DIR}"


def test_the_runner_uses_the_header_environment() -> None:
    """The positive control, without which every green below means nothing.

    If `uv run --script` leaked the ambient interpreter's packages, this module
    would be a second, slower copy of `test_docs_examples.py` reporting the same
    green for the same reason — and the defect it exists for would still be
    invisible. A header declaring NO dependencies must therefore find none of the
    packages this process can import.
    """
    import importlib.util

    ambient = [
        name
        for name in ("laterite", "pyarrow", "pandas")
        if importlib.util.find_spec(name) is not None
    ]
    if not ambient:
        pytest.skip(
            "nothing importable here for the runner to leak — the control cannot "
            "distinguish isolation from an empty ambient environment"
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


@pytest.mark.parametrize("example", EXAMPLES, ids=lambda p: p.name)
def test_docs_example_runs_under_its_header(example: Path) -> None:
    proc = _run(example)
    if proc.returncode == 0:
        _CENSUS["ran"].append(example.name)
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
