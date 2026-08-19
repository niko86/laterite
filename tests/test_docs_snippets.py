"""Runnable-guarantee gate for the docs' LITERAL Python, which nothing else runs.

`tests/test_docs_examples.py` and `gen_doc_outputs.py --check` cover the snippets a
page `--8<--`-INCLUDES from `web/docs-site/examples/python/`. Both key off that
include, so a hand-written fence is invisible to them — `--check-pages` inspects
only the `text` OUTPUT blocks, and a literal code fence with no output block is not
even counted. Twenty-two such fences sit across ~20 pages, plus the code on
`README.md`, the PyPI landing page and `COMPAT.md`, and none of it was executed by
anything.

What that cost, found the day this file was written: `COMPAT.md` documented
`check_file(..., dictionary=...)` twice. No such parameter has ever existed in
either library — both spell it `standard_AGS4_dictionary` — so the two blocks
demonstrating laterite's error-handling divergence raised `TypeError` on both
sides, showing the reader nothing. The surrounding prose was correct. Only the
code was uncopyable, and prose is not executable, so nothing noticed.

HOW A FRAGMENT IS CHECKED. Most literal fences are fragments: no imports, names an
earlier fence bound (`ags`, `fixed`, `delta`), paths to files that do not exist.
A page is therefore treated as ONE program in document order, with each `--8<--`
include contributing its example's source, because that is how a reader reads it —
`cookbook/fix-a-dirty-file.md`'s second fence uses the `fixed` its first created.

Each statement is then EXECUTED. That is the strongest evidence a snippet is
honest, and it reaches most of them; what genuinely cannot run (a fragment whose
setup is prose) falls back to resolving its `laterite.*` attribute chains against
the installed wheel, so a renamed symbol still fails here. `EXEC_FLOOR` keeps that
split from rotting — see `test_execution_coverage_has_not_regressed`.

Runs from a `tmp_path` with the fixture copied under `examples/`, the same way the
CLI example gate does, and for the same reason: these snippets WRITE. Executing
them from the repo root mints `checked.ags` (`surfaces/python.md`),
`delivery.xlsx` + `round-trip.ags` (`cookbook/excel.md`, the PyPI README) into the
working tree on every run.
"""

from __future__ import annotations

import ast
import contextlib
import doctest
import inspect
import io
import re
import shutil
import textwrap
from pathlib import Path

import pytest

# Needs the built surfaces: this module imports `laterite` to resolve the attribute chains its snippets claim. The buildless
# `repo-gates` job deselects it; the `python` job runs it after the wheel
# and the CLI exist.
pytestmark = pytest.mark.needs_env


REPO = Path(__file__).resolve().parents[1]
DOCS = REPO / "web" / "docs-site" / "docs"
EXAMPLES = REPO / "web" / "docs-site" / "examples"
FIXTURE = REPO / "examples" / "sample_site.ags"

#: Pages whose code is a claim about laterite. `docs/parity-coverage-map.md` and
#: `CHANGELOG.md` are excluded for the reason `test_self_named_gates.py` excludes
#: them: they tabulate and narrate, they do not instruct.
PAGES = [
    *sorted(DOCS.rglob("*.md")),
    REPO / "README.md",
    REPO / "packages" / "laterite" / "README.md",
    REPO / "COMPAT.md",
]

FENCE = re.compile(r"^(?P<i>[ \t]*)```python\n(?P<body>(?:.*?\n)*?)(?P=i)```\n", re.M)
INCLUDE = re.compile(r'--8<-- "([^"]+)"')

#: Same shape as `gen_doc_outputs.py`'s `<!-- doc-output: skip — … -->`, down to
#: insisting on a reason: an escape hatch nobody has to justify stops being one.
#: One verb, deliberately. A second (`raises X`, for a fence demonstrating an
#: error) was written and removed the same hour — the one fence that looked like
#: it needed it CATCHES its exception, so nothing propagates and the marker would
#: have been machinery with no caller.
MARKER = re.compile(
    r"<!-- doc-snippet: (?P<verb>skip)\s*[—-]\s*(?P<reason>[^\n]*?)\s*-->"
)

#: Fragment filenames -> the copied fixture. A snippet that only lacked a file can
#: then really run. Word-boundary-guarded, or `site.ags` rewrites the
#: `sample_site.ags` a page already names correctly (`learn/read.md` does).
PATHS = {
    "delivery.ags": "examples/sample_site.ags",
    "phase1.ags": "examples/sample_site.ags",
    "phase2.ags": "examples/sample_site.ags",
    "site.ags": "examples/sample_site.ags",
}
PATH_RE = re.compile(
    r"(?<![\w/])(?:"
    + "|".join(re.escape(k) for k in sorted(PATHS, key=len, reverse=True))
    + ")"
)

#: Roots whose attribute chains are resolvable no matter what context is missing.
MODULE_ROOTS = {"laterite", "L", "AGS4"}

#: The floor for `executed / (executed + unrunnable)`, as a percentage. Measured at
#: 85% when this landed. It is a RATCHET against silent decay: a refactor that
#: turned every fence into an unrunnable fragment would otherwise leave a green
#: gate asserting nothing, which is the failure mode `test_examples_are_discovered`
#: guards against next door ("Zero is a bad witness"). Raise it when it climbs;
#: lowering it is a decision that needs a sentence in the commit message.
EXEC_FLOOR = 80


def _seed() -> dict[str, object]:
    import laterite
    from laterite import compat

    return {"laterite": laterite, "L": laterite, "AGS4": compat, "__name__": "__docs__"}


def _fences(md: Path) -> list[tuple[str, str, re.Match | None]]:
    """(kind, source, marker) per python fence, in order. Includes are resolved."""
    text = md.read_text(encoding="utf-8")
    out: list[tuple[str, str, re.Match | None]] = []
    for m in FENCE.finditer(text):
        body = textwrap.dedent(m.group("body"))
        # The marker sits in the 300 chars before the fence — close enough to be
        # unambiguously about it, far enough to survive a wrapped sentence.
        marker = None
        for cand in MARKER.finditer(text, max(0, m.start() - 300), m.start()):
            marker = cand
        inc = INCLUDE.search(body)
        if inc:
            f = EXAMPLES / inc.group(1).split(":")[0]
            if f.exists():
                out.append(("include", f.read_text(encoding="utf-8"), None))
        else:
            out.append(("literal", body, marker))
    return out


def _chain(node: ast.AST) -> list[str] | None:
    parts, cur = [], node
    while isinstance(cur, ast.Attribute):
        parts.append(cur.attr)
        cur = cur.value
    return [cur.id, *reversed(parts)] if isinstance(cur, ast.Name) else None


def _lookup(path: list[str], env: dict) -> tuple[object | None, str | None]:
    """Walk a dotted path from a module root. Returns (object, first-missing-prefix)."""
    if not path or path[0] not in MODULE_ROOTS or path[0] not in env:
        return None, None
    obj = env[path[0]]
    for i, attr in enumerate(path[1:], 1):
        if not hasattr(obj, attr):
            return None, ".".join(path[: i + 1])
        obj = getattr(obj, attr)
    return obj, None


def _resolve(stmt: ast.AST, env: dict) -> list[str]:
    """Module-rooted names and their call keywords — what holds without context.

    Kwargs are checked here and not only by execution because a fence can be
    unrunnable (a fragment) or deliberately unrun (`doc-snippet: skip`) and still
    name a parameter that does not exist. `COMPAT.md`'s two `check_file` blocks
    were exactly that: one runs, one documents upstream and is skipped, and BOTH
    carried `dictionary=` for nine releases.
    """
    bad = []
    for node in ast.walk(stmt):
        if isinstance(node, ast.Attribute):
            _, missing = _lookup(_chain(node) or [], env)
            if missing:
                bad.append(f"[MISSING SYMBOL] {missing}")
        elif isinstance(node, ast.Call) and node.keywords:
            path = _chain(node.func) if isinstance(node.func, ast.Attribute) else None
            if not path:
                continue
            fn, _ = _lookup(path, env)
            # `callable`, not `is not None`: a resolved name can be a module or a
            # plain data attribute (`laterite.__version__` is a str), and handing
            # one of those to `signature` is meaningless. The previous guard let
            # them through and relied on the `except TypeError` below to absorb
            # the mistake, which is not the same as not making it.
            if not callable(fn):
                continue
            try:
                sig = inspect.signature(fn)
            except (TypeError, ValueError):
                continue  # a PyO3 callable with no introspectable signature
            if any(
                p.kind is inspect.Parameter.VAR_KEYWORD for p in sig.parameters.values()
            ):
                continue
            bad += [
                f"[BAD KWARG] {'.'.join(path)}(… {k.arg}=…) — not a parameter"
                for k in node.keywords
                if k.arg and k.arg not in sig.parameters
            ]
    return bad


def _workdir(tmp: Path) -> Path:
    """A temp cwd where the docs' own relative paths resolve.

    The fixture is copied to `examples/sample_site.ags` rather than the snippets
    being rewritten to an absolute path, so the text under test stays the text on
    the page — the CLI example gate's trick, for the same reason.
    """
    (tmp / "examples").mkdir(parents=True, exist_ok=True)
    shutil.copy(FIXTURE, tmp / "examples" / "sample_site.ags")
    return tmp


def _audit(md: Path, tmp: Path) -> tuple[list[str], int, int]:
    env = _seed()
    findings: list[str] = []
    ran = stuck = 0
    for kind, src, marker in _fences(md):
        src = PATH_RE.sub(lambda m: PATHS[m.group(0)], src)
        if src.lstrip().startswith(">>>"):
            continue  # doctest-shaped; test_doctests_hold covers these
        try:
            tree = ast.parse(src)
        except SyntaxError as e:
            findings.append(f"[UNPARSEABLE] {e.msg}: {src.splitlines()[0][:60]}")
            continue
        if marker:
            # Not executed — but still resolved. `skip` means "running this would
            # assert the wrong thing", never "nobody checks this".
            for stmt in tree.body:
                findings += _resolve(stmt, env)
            continue
        for stmt in tree.body:
            mod = ast.Module(body=[stmt], type_ignores=[])
            try:
                with contextlib.chdir(tmp), contextlib.redirect_stdout(io.StringIO()):
                    exec(compile(mod, str(md), "exec"), env)
                ran += kind == "literal"
            except (AttributeError, ImportError) as e:
                findings.append(f"[API DEFECT] {type(e).__name__}: {e}")
            except TypeError as e:
                # A signature that moved under the docs. Other TypeErrors are the
                # ordinary consequence of a fragment running without its setup.
                if "unexpected keyword" in str(e) or "no attribute" in str(e):
                    findings.append(f"[API DEFECT] TypeError: {e}")
            # Classified, not swallowed: anything else is a fragment running
            # without its setup, so fall back to resolving what is context-free.
            except Exception:
                if kind == "literal":
                    stuck += 1
                    findings += _resolve(stmt, env)
    return findings, ran, stuck


def _compat_knobs() -> dict[str, object]:
    """Every PROCESS-WIDE compat default, read out of the module that owns them.

    Discovered by prefix rather than listed, so a knob added later is covered
    without anyone remembering to come back here. That matters because the
    documented setters are *supposed* to be process-wide — `set_string_dtype`
    says so on the page — and this file executes documented code for a living.
    """
    from laterite import _frames

    return {k: v for k, v in vars(_frames).items() if k.startswith("_DEFAULT_")}


@pytest.fixture(scope="module")
def audited(tmp_path_factory: pytest.TempPathFactory) -> dict:
    tmp = _workdir(tmp_path_factory.mktemp("docs-snippets"))
    out: dict = {"findings": {}, "ran": 0, "stuck": 0}
    # Executing the docs means executing what the docs correctly document:
    # `AGS4.set_string_dtype("string")` on `concepts/dependency-shape.md` sets it
    # for the rest of the interpreter, exactly as written. Nothing put it back,
    # so every later `compat` test in the same process got `string` where the
    # DROP-IN CONTRACT asserts `object` — and the suite was green only because
    # ci.yml happens to pass `packages/laterite/tests` before `tests`. Reverse
    # the two arguments and it went red for a reason unrelated to the change in
    # flight (#328).
    #
    # Restored around the whole audit rather than per snippet: nothing runs
    # between the pages, and a page is one program by design. The limit worth
    # naming — this covers the compat knobs, not any process-wide state a future
    # snippet might document (an env var, a pandas option). Those would need the
    # exec moved into a subprocess, which costs a fork per page and buys nothing
    # until such a snippet exists.
    before = _compat_knobs()
    assert before, "no process-wide compat defaults found — has laterite._frames moved?"
    out["knobs_before"] = before
    try:
        for md in PAGES:
            f, ran, stuck = _audit(md, tmp)
            if f:
                out["findings"][md.relative_to(REPO).as_posix()] = list(
                    dict.fromkeys(f)
                )
            out["ran"] += ran
            out["stuck"] += stuck
    finally:
        from laterite import _frames

        for name, value in before.items():
            setattr(_frames, name, value)
    return out


def test_pages_are_discovered() -> None:
    """Zero is a bad witness: an empty page list makes every case below vacuous."""
    assert len(PAGES) > 40, f"only {len(PAGES)} pages found — has the docs tree moved?"
    assert FIXTURE.exists(), f"the shared fixture is gone: {FIXTURE}"


def test_running_the_docs_leaves_no_process_wide_state(audited: dict) -> None:
    """Falsify by deleting the restore in `audited`: `_DEFAULT_STRING_DTYPE` comes
    back "string", because `concepts/dependency-shape.md` documents the setter and
    this module runs what the docs say.

    Worth a test of its own rather than trusting the `finally`: the failure it
    guards against does not appear here at all. It appears in
    `packages/laterite/tests`, on a test asserting the drop-in's object dtype, and
    only when that directory is collected AFTER this one — so it reads as an
    unrelated flake in whatever change happens to be in flight (#328).
    """
    assert _compat_knobs() == audited["knobs_before"]


def test_documented_python_resolves_against_the_wheel(audited: dict) -> None:
    """Falsify by renaming any `laterite.*` name a doc names, or breaking a kwarg.

    This is the check that would have caught `check_file(..., dictionary=...)` the
    day it was written instead of nine releases later.
    """
    bad = audited["findings"]
    assert not bad, (
        "documented Python does not match the installed laterite:\n"
        + "\n".join(
            f"  {page}\n" + "\n".join(f"    {x}" for x in items)
            for page, items in sorted(bad.items())
        )
    )


def test_execution_coverage_has_not_regressed(audited: dict) -> None:
    """The anti-vacuity half: most snippets must really RUN, not merely resolve.

    Static resolution proves a name exists; it cannot prove a call works. If a
    change pushed every fence into the fallback path, the test above would still
    pass while asserting far less — so the split itself is asserted.
    """
    ran, stuck = audited["ran"], audited["stuck"]
    pct = round(100 * ran / max(ran + stuck, 1))
    assert pct >= EXEC_FLOOR, (
        f"only {pct}% of literal statements executed ({ran} ran, {stuck} could not), "
        f"below the {EXEC_FLOOR}% floor — the gate is drifting toward name-checking. "
        "Give the fences their setup, or lower EXEC_FLOOR and say why."
    )


def test_doctests_hold() -> None:
    """`>>> laterite.compat.__version__` must print what the wheel prints.

    Fences are stripped first: pointing `doctest` at raw Markdown swallows the
    closing ``` into the expected output and reports a failure on a passing
    example.
    """
    import laterite
    import laterite.compat

    md = REPO / "COMPAT.md"
    src = "\n".join(
        line
        for line in md.read_text(encoding="utf-8").splitlines()
        if not line.startswith("```")
    )
    test = doctest.DocTestParser().get_doctest(
        src, {"laterite": laterite}, md.name, str(md), 0
    )
    assert test.examples, (
        "no doctests found in COMPAT.md — has the fence shape changed?"
    )

    out = io.StringIO()
    runner = doctest.DocTestRunner(optionflags=doctest.ELLIPSIS)
    runner.run(test, out=out.write)
    assert runner.failures == 0, f"COMPAT.md doctests are stale:\n{out.getvalue()}"
