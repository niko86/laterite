---
type: strategy
title: "Probe: duplicate HEADING → python rule_7_2 IndexError (O-8) — REFUTED"
status: confirmed
tags: [strategy]
targets: [rule-07-heading-order, rule-09-unknown-headings]
divergence_hypothesis: "see body"
probe_files: [ags-wiki/.bootstrap/probes/probe-o8-dup-heading.ags]
expected_rust: "Rule 7 (duplicate field names), completes (bounds-guarded)"
expected_python: "(hypothesised) IndexError crash → PythonError"
evidence: "probe-run — ags-wiki/.bootstrap/probes/RESULTS.md"
related: [rule-07-heading-order, rule-09-unknown-headings, O-08, o8-unreachable-rename-dup-headers]
sources: []
---
# Probe: duplicate HEADING → python rule_7_2 IndexError (O-8) — REFUTED

## Hypothesis
> [!divergence] [[O-08]] [BUG]: python `rule_7_2` indexes `temp[i]`
> unguarded → a trailing duplicate in-order HEADING (e.g.
> `PROJ_ID,PROJ_NAME,PROJ_NAME`) `IndexError`s and aborts the whole
> python run → opaque `PythonError`, while Rust completes. Proposed: a
> classify arm mapping the crash → `KnownDivergence{O-8}`.

## Probe design
- Fixture: `ags-wiki/.bootstrap/probes/probe-o8-dup-heading.ags` (PROJ with `PROJ_ID,PROJ_NAME,PROJ_NAME`, 3-col UNIT/TYPE/DATA).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 7 | IndexError crash |
| observed | **Rule 7** (completes, bounds-guarded) | **Rule 7 + Rule 9 + Rule 18 — NO crash** |

## Verdict
> [!note] **HYPOTHESIS REFUTED (probe-run).** python did **not**
> crash. python-ags4's *default* `AGS4.check_file(...,
> rename_duplicate_headers=True)` (exactly what `ags4 check` does)
> renames the 2nd `PROJ_NAME` → `PROJ_NAME_1` *before* `rule_7_2`, so
> `set(headings).issubset(reference)` fails first and the unguarded
> `temp[i]` is never reached. O-8's IndexError is reachable only with
> `rename_duplicate_headers=False` (non-default). **So O-8 [BUG] is
> effectively unreachable via a HEADING-row duplicate under default
> python-ags4 1.2.0** — Rust's bounds-guard defends against a bug
> python's own default param shields. A speculative O-8 *crash*
> classify arm would be over-claiming and is **not** added (the
> generic `PythonError` short-circuit is adequate). See the refined
> finding + proposed prose tightening in
> [[o8-unreachable-rename-dup-headers]].

## Related
[[rule-07-heading-order]] · [[rule-09-unknown-headings]] · [[O-08]] · [[o8-unreachable-rename-dup-headers]]
