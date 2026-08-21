---
type: insight
title: "Compat decisions held — next-steps register for the python-ags4 parity arc"
status: confirmed
tags: [insight, register, compat]
gap_kind: design-choice
severity: low
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: false
feeds_strategy: []
discovered_phase: I
related: [O-37, oracle-drift-pin, parity-model]
sources: []
---
# Compat decisions held — next-steps register

> The python-ags4 parity arc (Stages 1–6) closed 99/131 parity tests.
> This page parks the *deliberate non-closures* — items we have a
> position on but haven't actioned, so future sessions don't
> re-derive the analysis.

## Held items

### H-1 — Wrap engine exceptions as "Validator Process Error"
**What**: python-ags4's `check_file` swallows its own exceptions and
returns them as a `"Validator Process Error"` entry in the result
dict. `laterite.compat.check_file` lets `BadDictError` /
`UnsupportedEditionError` propagate.

**Closes**: ~2 tests (`test_checking_without_dictionary_raises_error`,
the `test_check.py::test_duplicate_groups_raises_error` "wants Process
Error key" arm).

**Why held**: changes the return shape of `check_file` — callers
that currently catch `BadDictError` would silently miss the dict entry.
Pythonic raise-on-error is more useful than return-and-hope-they-check;
this divergence is design-coherent on both sides.

**To revisit when**: a porting user reports the divergence as
breaking; or we add a `compat_python_ags4_errors=True` flag (default
off) that opts in to the wrap.

### H-2 — Checker identity claim relaxation
**What**: `Metadata.Checker` currently reads
`"laterite (clean-room laterite_ags4_validator engine)"`. python-ags4 tests
assert `"python_ags4 v1.2.0"`.

**Closes**: ~6 tests (`test_rule_2`, `test_rule_2b_1`,
`test_rule_LBSGCheck`, `test_rule_STNDandPREMCheck`, `test_version`,
+ per-edition Metadata.Dictionary arms).

**Why held**: lying about which validator produced a report would be
strictly worse for downstream consumers — they should be able to tell
laterite from python-ags4 from the report itself. Closing these tests
is a no-op except in the test-suite-pass-count axis.

**To revisit when**: never, unless a `compat_python_ags4_identity=True`
flag is added (and even then, opt-in, never default).

### H-3 — 4 JSON-helper functions (still deferred)
**What**: `utils.get_DICT_table_from_json_file`,
`get_ABBR_table_from_json_file`, `get_TYPE_table_from_json_file`,
`get_UNIT_table_from_json_file`. Each reads python-ags4's bundled
per-edition `.json` dictionary helpers and returns a DataFrame.

**Status**: H-4's dependency now unblocked (Stage 6d landed
`convert_to_text(dictionary=...)`). The 4 JSON helpers can now be
implemented as a clean follow-up — each is ~30–60 LoC of pandas
transformation, and they'll close the corresponding 4 parity tests.

**Why still held**: not blocking; the work is straightforward but
unrelated to the validator engine, and lower-value than other
directions. Move when there's appetite for compat surface
completeness.

**To revisit when**: a session is dedicated to compat surface
completeness, or a porting user reports needing one of these.

### ~~H-4 — O-28: external dict path for `convert_to_text` and `excel_to_AGS4`~~ — **CLOSED Stage 6d**
**What was held**: `convert_to_text(df, dictionary='4.1')` and
`excel_to_AGS4(input, output, dictionary='tests/DICT.ags')` raised
`BadDictError`.

**Closed by**: Stage 6d added per-edition UNIT/TYPE lookup via the
new PyO3 fn `_native.dict_group_unit_type(edition, group)` (built on
the existing `Dictionary::heading(group, name)` bundled-dict
infrastructure). `convert_to_text` accepts version strings, bundled
dict basenames, and external AGS4 dict files (via the same parser
used for regular AGS4 input — bundled dicts ship as AGS4 too).
`excel_to_AGS4(dictionary=...)` post-processes the Rust-emitted
AGS4 file by piping it through `convert_to_text`.

**Closed**: 6 parity tests (`test_convert_to_text` + 5
`test_convert_to_text_specifying_dictionary_version[X]` arms +
`test_excel_to_AGS4`).

### H-5 — `??FIELD??` placeholder markers in Rule 10b Empty REQUIRED
**What**: python-ags4 emits
`Empty REQUIRED fields: DATA|SAMP_TYPE|??ABBR_CODE??|...` — inlining
the failed DATA row with `??NAME??` placeholders where REQUIRED cells
were empty. laterite reports
`Empty REQUIRED field(s): ABBR_CODE`.

**Closes**: 1 test (`test_rule_10_9`).

**Why held**: cosmetic. Our wording is informationally equivalent and
arguably clearer. Producing the python-ags4 form needs Rust changes
to reconstruct the full DATA row with markers in the validator.

**To revisit when**: never, unless a downstream tool depends on the
exact placeholder format.

### H-6 — Rule 6 / BOM handling fine-grained wording
**What**: `test_rule_6_1` exercises a malformed file laterite refuses
as `NotAgs4Error` before reaching Rule 6 (Rule 6 is a no-op in
python-ags4 per O-2). `test_file_with_BOM` asserts a specific BOM-
inclusive Rule 1 wording laterite doesn't emit verbatim.

**Closes**: 2 tests.

**Why held**: Rule 6 case is genuine behavioural divergence (we
correctly refuse non-CSV input; python-ags4 silently mis-validates —
the O-34 dynamic). BOM wording is a Rule-1 message gap the translator
hasn't covered because the underlying laterite emission collapses two
cases.

**To revisit when**: comprehensive Rule 1 / BOM probe shows
laterite's wording is wrong (not just different).

## Why this page exists

The user wants future sessions to not re-derive the parity-residual
analysis. Each H-N here:

1. Names what python-ags4 does that we don't.
2. Names what *would* close it.
3. Names *why* it's held (not "we forgot" — we have a position).
4. Names the trigger for revisiting.

When an H-N is actioned, move it to OBSERVATIONS.md (if it produces
behavioural change) or just delete it from this page.

## Related
[[O-37]] · [[O-28]] · [[oracle-drift-pin]] · [[parity-model]]
