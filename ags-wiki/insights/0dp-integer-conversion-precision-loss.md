---
type: insight
title: "0DP integer CONVERSION range-guards an out-of-i64 value to Null (#611) — was a fabricated i64::MAX; python-ags4's conversion keeps full precision"
status: ratified
tags: [insight]
gap_kind: rust-vs-python
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-08-typed-values]
proposes_observation: true
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: D
related: [rule-08-typed-values, nDP, O-49, O-50, python-ags4]
sources: [spec-4.2]
---
# The 0DP conversion step, not the check, was the divergence

## Claim
> [!variance] A `0DP` (Integer) cell whose text cannot be a clean in-range
> `i64` — a huge `"1E30"`, a tiny `"1E-30"`, a fractional `"5.7"` — takes a
> different string→number CONVERSION path on each engine, even though **both**
> validators already FLAG the cell via the identical strict Rule 8 regex
> (`^-?\d+\.?$`, laterite's `is_ndp(s, 0)`) — so this is *not* a validation
> divergence, only a conversion one. Pre-#611, laterite's saturating `f as
> i64` cast silently **fabricated** `i64::MAX` for an out-of-range value that
> was never in the file; python-ags4's arbitrary-precision `int(float(s))`
> preserves the value exactly. Same formatter family as [[O-49]] (the count-DoS
> sibling), opposite direction: O-49 is unbounded OUTPUT, this is unbounded
> INPUT hitting a bounded (`i64`) store.

## Evidence
- **python-ags4** `ext:ags-python-library:python_ags4/AGS4.py::convert_to_numeric`
  (`int(float(s))`): arbitrary-precision Python int — `"1E30"` → the exact
  `1000000000000000019884624838656`, `"5.7"` → `5`, `"1E-30"` → `0`. Never
  fabricates; conversion preserves. (Its own Rule 8 validation separately
  flags the same cells — the two operations are independent, and it is the
  validation half, not the conversion half, that already agrees with us.)
- **laterite, pre-#611**: the Integer arm of `parse_value` only checked
  `f.is_finite()` then cast `(f as i64)` unconditionally — a saturating cast,
  so `"1E30"` silently became `9223372036854775807`, a number that was never
  in the file. The identical shape was copied three ways: the leaf's own
  `parse_value`/`ags4_str`, and laterite-py's PyO3 `parse_value` wrapper
  (`repo:rust-packages/laterite-py/src/ags_types_fns.rs`) — the #531
  single-sourcing dedup covered the date/time/bool parsers but left this
  Integer arm un-converged (the #503 record-link lesson: a typed-read object
  and the hash canonicalisation can silently drift from each other when they
  don't share one function).
- **#611**: `parse_ags_integer` (`repo:rust-packages/laterite-ags4-types/src/lib.rs::parse_ags_integer`)
  range-guards via `f64_fits_i64`
  (`repo:rust-packages/laterite-ags4-types/src/lib.rs::f64_fits_i64`) before the
  cast — out of `i64` range → `None` (a Null typed value / Python `None`);
  in-range is untouched (`"5.0"`→5, `"5.7"`→5, `"1E-30"`→0, so
  `_content_hash` is byte-identical for every real value). Regression tests
  `repo:rust-packages/laterite-ags4-types/src/lib.rs::parse_ags_integer_guards_the_i64_range`
  and
  `repo:rust-packages/laterite-ags4-types/src/lib.rs::parse_value_0dp_overflow_is_null_not_fabricated`
  pin both the guard and that unchanged behaviour. `laterite-py`'s Integer/
  Decimal `parse_value` arms
  (`repo:rust-packages/laterite-py/src/ags_types_fns.rs`) now route through
  the same `parse_ags_integer`/`parse_ags_decimal` — the #531 dedup finished
  for this pair, so the typed-read object and `_content_hash` cannot drift.

## Why it matters
Fabricating `i64::MAX` for an out-of-range value is worse than refusing it —
a caller sees a concrete, plausible-looking integer that was never in the
source file, with no signal that it is wrong. The exposure is narrow by
construction: real geotech `0DP` columns run single-digit to low-thousands,
and the one integer known to grow (a cyclic-triaxial cycle count) sits around
`1e4`, nowhere near `i64`'s ~9.2e18 ceiling — so the guard only ever fires on
a ≥19-digit value, which in a whole-number column is already an Excel/export
error, and Rule 8 already flags that cell independently. Reject-to-Null
surfaces the error instead of manufacturing a wrong number, and leaves every
genuine value's hash unchanged. Full-precision preservation (matching
python's arbitrary-precision path) was considered and deliberately
**deferred** — it needs an arbitrary-precision store threaded through
`_content_hash` and every typed-read surface, for a case the validator
already flags; `parse_ags_integer`'s own doc-comment records exactly what to
widen if that ever becomes necessary.

## OBSERVATIONS entry — **ratified as [[O-50]]**
> [!note] Written into `repo:OBSERVATIONS.md#o-50` (5-field house style).
> **Our decision** (#611): range-guard `parse_ags_integer` so an out-of-`i64`
> `0DP` value converts to Null, not a saturated integer — the "reject
> overflow" option, single-sourced across the leaf's `parse_value`/`ags4_str`
> and laterite-py's PyO3 wrapper so the typed-read object and the hash
> canonicalisation cannot drift. **Upstream-reportable: [NO]** — python-ags4's
> conversion is not defective here; it preserves precision and WE were the
> lossy side. Recorded for our own decision trail, and as the sibling of
> [[O-49]] (the Class B count-DoS, where python-ags4 is the reportable side).

## Related
[[rule-08-typed-values]] · [[nDP]] · [[O-49]] · [[O-50]] · [[python-ags4]]
