---
type: decision
title: "A docs example is run twice: against this tree, and in the environment its own header declares"
status: accepted
tags: [design, decision]
decided: 2026-08-22
supersedes: []
from_gap: []
related: [design/_README, concepts/docs-site, dec-doc-code-fences]
sources: []
---

# A docs example is run twice: against this tree, and in the environment its own header declares

## Context

Every `ex*.py` under `repo:web/docs-site/examples/python` carries a PEP 723
`# /// script` header pinning `requires-python` and an exact
`laterite[extras]==<product>`, and a docstring telling the reader to run it with
`uv run`. `repo:tests/test_docs_examples.py` executes all of them — under the
interpreter running pytest, which is the dev venv, where the root
`pyproject.toml` pins pandas and pyarrow.

So **the header was never the environment any gate ran in**, and a header could
be missing a dependency without anything being able to fail.

Two were. `ex06_sql_join.py` and `ex21_synthetic_keys.py` declared bare
`laterite` and finished with `rel.pl()`. Under the gate they exited 0; under
their own headers they died with `ModuleNotFoundError: No module named
'pyarrow'` — the same bytes, green for us and broken for a reader, which is the
whole shape of the defect.

**It is narrower than it looks, and the narrowness is the lesson.** laterite's
own `.frame()` / `.to_polars()` / `.to_pandas()` are genuinely pyarrow-free,
exactly as the docs' `concepts/dependency-shape.md` claims. `.sql()` leaves that
surface: it hands back a **`DuckDBPyRelation`**, and the materialiser is then
DuckDB's, under DuckDB's rules. Measured against a `[compat]` install with no
pyarrow: `.df()` works, `.pl()` and `.arrow()` raise. `.pl()` is the one that
catches people, because polars is in the base install and the line looks like it
cannot need an extra.

## Options considered

1. **Make the existing gate honour the header** — run everything with `uv run`.
2. **Fix the two headers and leave the gate alone.**
3. **Add a second run** that honours the header, keeping the first as it is.

## Decision

**Option 3**, plus the header fixes and the documentation the second fix implies:

- `repo:tests/test_docs_examples.py` is unchanged and keeps asking *do these
  examples work against this working tree*.
- `repo:tests/test_docs_example_headers.py` asks *do these examples work in the
  environment they publish*, running each one with `uv run --exact --script`.
- It is **opt-in** (`LATERITE_DOCS_HEADER_ENV=1`) and runs in its own nightly
  job, `docs-example-headers` (`repo:.github/workflows/nightly.yml`).
- A failure is **classified, in the test**, by re-running the example with the
  same pin widened to `laterite[all]`. Decided by the extras → fatal. Not decided
  by the extras → skipped with the resolver's own output as the reason.
- Both `.sql()` examples now pin `laterite[pyarrow]`, and the DuckDB terminals'
  dependencies are stated on the pages that document them.

## Why

**Option 1 trades a guarantee for a guarantee rather than adding one.** The
header pins a released version, so honouring it would stop testing the tree
entirely — and a breaking change in the tree no longer turning an example red is
precisely the regression `test_docs_examples.py` was created for. Its own module
docstring records that the Python half of the guarantee went unenforced once
already and "it bit exactly as you'd expect". Two questions, two runs.

**Option 2 fixes today's two and leaves the hole.** The defect class is "a header
nobody has executed", and the next one arrives with the next example.

**Its own job, because the calibration is different.** The `docs-vs-released-*`
legs amnesty every step below their `under-test` determination while the tree is
ahead of the released tag, and `repo:tests/test_nightly_wiring.py` enforces
that in both directions. That amnesty is right for an example using an API the
release does not have yet — `ex18_severity_tiers.py` failed on the first run for
`Report.warnings`, which exists in this tree and not in the release, a fact about
the calendar and not about the header. It is wrong for a **missing extra**, which
no release fixes and which would then be excused on nearly every night, since the
tree is ahead of the tag nearly always. Putting the leg inside that job would
have meant choosing one calibration for two classes of failure.

**So the classifier lives in the test, not in the YAML.** Widening the pin to
`laterite[all]` and re-running answers exactly the question the amnesty cannot:
if widening the extras fixes it, the header is the defect, and no release ever
changes what a header asks for. It answers only when the extras
DECIDE it — a script that both misses an extra and uses unreleased API lands in
the second bucket and waits for the release — and that limit is written into the
module rather than left to be discovered.

**Nightly, because an isolated environment per example is not free on a cold
cache** — the count is whatever the glob finds, and it only grows — and
because the pin names the release, so the leg is asking a released-artefact
question and belongs with the other ones. There is no released venv or toolchain
to build: `uv` resolves each header itself.

## What building it found about `uv`

**`--exact` is the gate, not a flourish.** uv caches a script's environment
against the script's **path** and by default only adds to it: a header that
*loses* a dependency keeps the package an earlier run installed. The first
falsification of this module was therefore green — `ex06`'s header reverted to
the broken one still passed, while the same bytes at a fresh path failed. A gate
that inherits its own history is not a gate. `--exact` uninstalls the extraneous
package first.

That mattered here and would have mattered more later: CI runners are cold, so
the fault would have been invisible in the one place that runs this nightly and
live on every developer machine that ran it twice.

**The positive control is in the module** — a synthetic header declaring no
dependencies must find none of the packages the calling interpreter can import.
Without it, a runner quietly leaking the ambient environment would make this file
a slower copy of its sibling, reporting the same green for the same reason.

**And the first cut of that control skipped in the only job that runs it.** It
probed for laterite / pyarrow / pandas and abstained when none was importable —
which is exactly the nightly's environment, since the leg starts from `uv run
--no-project --with pytest`. It passed locally, where a developer has all three,
and would have skipped every night. `pytest` is now the canary: whatever else is
around, it is importable in a process running under pytest. A control that can
decline to run is a control with a hole the shape of its own subject.

**A single transient would have been reported as a header defect.** Both verdicts
are comparisons of two runs, so one flaky resolve during the header run followed
by a clean widened run reads as "passes with `[all]`" — the FATAL arm, naming a
defect that does not exist. The failure is confirmed by a second header run
before it is classified, and an example that needed that second run is named in
the census rather than quietly absorbed.

**The widening scans the header block, not the file.** While every header names
laterite the two are indistinguishable — the header is on line 3 and matches
first either way — so the invariant is held by an adversarial case rather than by
the happy path: `ex16_diff.py` carries `"laterite demo site (…)"` as fixture text
it edits, and a whole-file scan rewrites *that* the moment a header stops naming
laterite, producing a corrupt copy whose failure reads as "not decided by the
extras". A real defect downgraded to a skip by the machinery meant to catch it.

## Consequences

- A new example is covered by both runs the moment it is added — same glob, same
  argument as `test_docs_examples.py` makes for its own discovery.
- The header is now load-bearing in two directions: `test_version_faithful.py`
  holds the pin to the shipped version, and this holds the *extras* to what the
  script actually needs.
- **The blind spot, stated:** an example that is both missing an extra and using
  unreleased API reports as the second class, so its header defect is invisible
  until the release catches up. The census the module prints on every run names
  the undecided examples, so the set is visible rather than inferred.
- The run fails if **every** example lands in the undecided bucket, because that
  is a run that measured no header at all. It fires for real between a version
  bump and its publish, when the pin resolves to nothing — which is a nightly
  saying the docs ask for a laterite that PyPI does not have, and worth hearing.
- `.sql()`'s dependency story is now documented where a reader meets it —
  `concepts/dependency-shape.md`, `concepts/fluent-model.md`, `learn/query.md`
  and `cookbook/sql-across-groups.md` — which was wanted regardless of the gate,
  since it is the thing a reader gets wrong on their own code rather than ours.

## Related
[[concepts/docs-site]] · [[dec-doc-code-fences]] ·
repo:tests/test_docs_example_headers.py ·
repo:tests/test_docs_examples.py ·
repo:web/docs-site/examples/python ·
repo:.github/workflows/nightly.yml
