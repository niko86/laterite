---
type: decision
title: "Forge is a description service with a calibration side-job — a bounded second audience"
status: accepted
tags: [design, decision]
decided: "2026-08-25"
supersedes: []
from_gap: []
related: [dec-ags4-forge-evolutionary-dogfood, dec-forge-type-axis-instrument, laterite-ags4-forge, bs5930-soil-descriptions, parity-model, evolutionary-dogfooding, agent-first-cli-contract]
sources: []
---
# Forge is a description service with a calibration side-job — a bounded second audience

## Context
[[dec-ags4-forge-evolutionary-dogfood]] commissioned forge to manufacture
*proven* parity divergences, and says nothing about who else might use it.
Nothing else in this workspace does — but `laterite-showcase`, building a
demonstration deliverable, became forge's first consumer from outside the
parity mission and filed two issues (#697, #698) asking for output that is
**convincing**, not merely valid.

Both asks sit furthest from the commissioned mission, and the capability
survey that preceded this decision framed the question as binary: does forge
serve one audience or two? Answering it needed the premise tested first,
because two of its supporting claims turned out to be false.

`scale` is **not** evidence of a second audience — `tools/gen-bench-fixtures.sh`,
`tools/bench-vs-python-ags4.py` and `tools/bench-cert-parse-share.py` all
drive it, so its sized fixtures serve the first audience. And the realism ask
is **not** one thing: the half of #698 concerning result *values* is a parity
defect on its own terms, split out as #707 and decided in
[[dec-forge-type-axis-instrument]]. What remained after that split is the
genuine second-audience question, and it is much smaller than it looked.

## Options considered
1. **Commit fully** — build the realism lane: stratigraphy shaping, depths
   derived from a profile, values keyed to the strata the description engine
   already invents, plus the organic/peat lane.
2. **Decline** — forge serves one audience; close #697 and #698 as `wontfix`
   and let consumers build their own layer.
3. **A bounded second audience** — ratify the role the consumer already
   derived from use: forge owns the descriptions and the byte-target
   calibration, and nothing else.

## Decision
Option 3. **Forge is a description service with a calibration side-job.** It
owns the [[bs5930-soil-descriptions]] engine that writes every `GEOL_DESC`,
and the borehole-count calibration `scale` performs against a byte target. It
does **not** own the geological model those descriptions sit in.

Consequently #697 (the organic/peat lane) is **in scope**, rescoped to include
the `--principal`/`--lane` selector it offers as optional; and the realism
remainder of #698 is **declined**.

## Why
The boundary was not invented to settle this question — it was derived
independently, from use. After its own ticket falsified the assumption that
forge would supply the realistic base, `laterite-showcase` recorded exactly
this split in its ADR 0003 (*"Forge is a description service, not the
generator"*), and the division of labour it measured on its own deliverable
is the one ratified here: forge authored the descriptions, its director
authored the model.

That division is the right one on the merits. Stratigraphy — layer ordering,
depth containment, cross-hole correlation — is a property of the **narrative a
consumer is telling**, not of a fault injector, and two consumers would want
different ones. It would also buy the parity mission nothing: once the type
axis is carved out ([[dec-forge-type-axis-instrument]]), the values realism
would make *plausible* are already type-**valid**, and per the settled finding
that arbitrary text is inert outside KEY, typed, PA and non-ASCII fields, a
better-sounding `GEOL_DESC` cannot change a verdict — `GEOL_DESC` is `X`-typed.

Descriptions are the opposite case. They are already forge's, already
self-contained, and the v1 scope note in
`repo:rust-packages/laterite-ags4-forge/src/synth/bs5930.rs` staged the
remaining lanes deliberately rather than omitting them.

## Consequences
Commits us to:

- **A consumer-facing contract on an unpublished internal crate**, in a public
  repo, with no gate on that interface. Every other cross-surface promise here
  has one — [[surface-census]] for the `lat` launchers, a gate-held CLI guide,
  `tools/check_doc_refs.py` for citations. This one does not, and the consumer
  builds from source at a pinned commit, so it would break on its own schedule
  rather than at a release boundary. Accepted knowingly; it is the longest tail
  on this decision and the first thing to revisit if it bites.
- **The organic/peat lane must be opt-in behind a flag.** `describe()` opens
  with a coarse/fine draw; adding a third lane shifts it and every subsequent
  draw, so every seed produces a different file. Nothing fails when that
  happens — the determinism contract asserts only that a seed reproduces
  itself, and the safety net asserts the base validates *clean*, not
  identically — while the bench fixtures drawn from this engine quietly stop
  being comparable across the change. #698 asked for this discipline for its
  own request; #697 inherits it. Turning the lane on by default is a deliberate
  re-roll with the fixtures regenerated in the same change.

Rules out:

- **Stratigraphy shaping in forge**, and with it profile-derived depths and
  strata-keyed values. Recorded here rather than filed, so it is not
  rediscovered as an open gap: it is a decision, not a backlog item.

Leaves untouched: everything [[dec-ags4-forge-evolutionary-dogfood]] actually
decided — manufactured divergences over found corpora, no LLM in the binary,
the declarative strategy, extract-over-duplicate. It needs no correction and
remains `accepted`; the third axis is named on the tool page, which is where
the count was stated.

## Related
[[dec-ags4-forge-evolutionary-dogfood]] · [[dec-forge-type-axis-instrument]] ·
[[laterite-ags4-forge]] · [[bs5930-soil-descriptions]] · [[parity-model]] ·
[[evolutionary-dogfooding]] · [[agent-first-cli-contract]] ·
`repo:rust-packages/laterite-ags4-forge/src/synth/bs5930.rs`
