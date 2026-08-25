---
type: decision
title: "The type axis is closed by edit-layer projection, not by teaching the synthesizer"
status: accepted
tags: [design, decision]
decided: "2026-08-25"
supersedes: []
from_gap: [parity-triage-sampling-bias]
related: [dec-ags4-forge-evolutionary-dogfood, dec-forge-audience-boundary, laterite-ags4-forge, laterite-ags4-parse, strat-parity-matrix, parity-model, dec-dictionary-single-source]
sources: []
---
# The type axis is closed by edit-layer projection, not by teaching the synthesizer

## Context
[[laterite-ags4-forge]] states the search has **two** axes: *rule*, small and
enumerable, and *placement*, large and only samplable. There is a third, and
nothing recorded it, so nothing noticed it bounding the search: the **AGS
TYPE** a value is declared as. ([[dec-ags4-forge-evolutionary-dogfood]] does
not count the axes at all — the claim lives on the tool page, and in the
capability survey that preceded this decision.)

Type behaves like *rule*, not like *placement* — it is small and enumerable —
but it was collapsed into placement, where "sampled" reads as adequate
coverage. The consequence is concrete. `catalog` gives the Rule 8 injector as
*"write a non-date into a DT-typed cell"*, and the scaffolds put only a
handful of the dictionary's types in front of it. Meanwhile
[[strat-parity-matrix]] carries Rule 8 among its asserted-clean rows, the ones
that regression-guard the clean-room claim. That verdict was earned on one
type and generalised to the rest.

This is [[parity-triage-sampling-bias]] ([[O-36]]) arriving on the axis nobody
named — the same defect forge was commissioned to fix, inside forge. It
surfaced from the outside, in #698, as a complaint about realism.

## Options considered
1. **Teach the synthesizer the missing types** — populate the result headings
   across the SAMP-child result groups so the richer numeric types appear by
   construction.
2. **Project at the edit layer** — give `edit` the operations to retype a
   column and reshape it, so any generated file can be cast into any type.
3. **Leave it**, and keep the type axis unnamed.

## Decision
Option 2. `edit` gains `--add-column`, insert-row-at-position, `--set-unit`
and `--set-type`, with **type re-formatting** on a retype. The reach is held by
a gate: `scaffolds_reach_a_pinned_set_of_ags_types` in
`repo:rust-packages/laterite-ags4-forge/src/synth/mod.rs` pins the set of types
each scaffold reaches, as an **exact set** rather than a floor, so widening it
is a reviewed edit rather than a drift.

`edit` therefore **stays in this crate** — see Consequences.

## Why
Option 1 spreads the work across the result groups below SAMP and buys one
thing: those specific types, in those specific places. Option 2 is a single
general projection in a module that already exists, and it composes — a
retype-and-reformat pass reaches types no scaffold would ever emit naturally.

It also unlocks two things option 1 does not:

- The Rule 8 injector generalises from *"non-date into a DT cell"* to
  *type-invalid value into **any** typed cell*.
- **Rule 15** (undefined `UNIT`) becomes injectable. `catalog` currently
  records it in `not_single_injectable` as *"the UNIT-group twin of Rule
  16/17 (candidate future injector)"* — editable UNITs are the missing piece.

And it leaves the synthesizer's determinism contract untouched, which option 1
would not: changing what the generator emits re-rolls every seed, and the
bench fixtures are drawn from it (the same hazard [[dec-forge-audience-boundary]]
records for the description lanes).

The gate is pinned exactly, not as a floor, because the reach is a property of
a scaffold's heading set rather than of the values drawn into it — so it is
seed-stable, an exact pin costs no flakiness, and any movement in either
direction is worth a reviewer's attention. It is falsifiable by construction:
dropping a type from the pin fails the test naming that type.

## Consequences
Commits us to:

- **`edit` stays in forge.** The case for extracting it rested on it sharing
  nothing with the synthesize→validate→evolve spine. That is false:
  `minimize` — which *is* the spine, since it produces the reproducers — calls
  `edit::rebuild` for its field pass, and that quote-aware rebuild is what
  stopped the minimizer emitting reproducers that were not smaller versions of
  their input. Growing `edit` into the type instrument deepens the coupling
  deliberately.
- **A live parity experiment, not a text edit.** Widening Rule 8 across the
  remaining types runs the comparison for the first time. It may restore the
  AGREE verdict as *earned*, or it may manufacture new `O-N` records with the
  process cost those carry. That is forge working as commissioned; it means the
  work cannot be scoped in advance.
- **[[strat-parity-matrix]] rescopes its Rule 8 row** to the type it was
  actually earned on, until the widening says otherwise.

Rules out:

- **Physical unit conversion.** Editing a `UNIT` will not rescale the values
  under it. The dictionary carries units as bare strings with no dimension or
  factor data, so conversion needs a second reference-data source — which
  [[dec-dictionary-single-source]] exists to prevent — and AGS unit strings are
  not a controlled vocabulary, so dimension would have to be *inferred* from
  free text before anything could be converted. It buys the parity mission
  nothing either: Rule 15 cares whether a `UNIT` is present, absent or
  undefined, never whether the number was scaled to match. Recorded here rather
  than filed, so the argument is not rebuilt from scratch.

## Related
[[dec-ags4-forge-evolutionary-dogfood]] · [[dec-forge-audience-boundary]] ·
[[strat-parity-matrix]] · [[parity-triage-sampling-bias]] · [[parity-model]] ·
[[dec-dictionary-single-source]] · [[laterite-ags4-parse]] ·
`repo:rust-packages/laterite-ags4-forge/src/synth/mod.rs` ·
`repo:rust-packages/laterite-ags4-forge/src/edit.rs`
