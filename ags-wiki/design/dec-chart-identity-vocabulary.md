---
type: decision
title: "Chart identity is a separate vocabulary from brand identity (#434)"
status: accepted
tags: [design, decision, web, tokens, accessibility, dataviz]
decided: "2026-08-19"
supersedes: []
from_gap: []
related: [dec-landing-build-shared-tokens, validator-site]
sources: []
---

# Chart identity is a separate vocabulary from brand identity

## Context

The brand ramp is a **sequential** scale by construction: one hue family, from
topsoil to bedrock, monotone in lightness. `repo:web/src/shared/styles/colors.css`
says so in as many words — it is also the seven-band rail, and band colour encodes
group identity by *position*.

A chart series legend does a different job. It encodes **identity**, which needs
hues that differ rather than steps that differ. #410 asked for "a categorical
sequence that is legible and distinguishable in both themes" and the ramp could
not supply one: every candidate that separated on the light surface collapsed on
the dark one, where the admissible lightness band is narrower and adjacent steps
converge. No ordering fixes that, because the problem is the scale's shape.

Two further things were found while answering it, and they belong here because
they share one cause — nothing in this repo had ever measured the distance
*between* two colours.

## Options considered

1. **Extend the vocabulary** with chart-categorical hues chosen for both surfaces.
2. **Sanction lightness-led identity** — keep the ramp, make every chart carry
   mandatory direct labels or texture as a second channel.
3. **Constrain charts to one series** per view, facetting instead of colour-splitting.

## Decision

**Option 1, fenced.** Charts get `--chart-1..5` in their own
`repo:web/src/shared/styles/charts.css`, imported by the shared token layer.

The fence is one-directional and deliberate: nothing in the chart file is aliased
into the semantic layer, and nothing in it reads *from* the semantic layer. Slot 1
is the rust value written out rather than `var(--cta)` — the two are meant to be
equal and the gate asserts it, so the coupling is checked instead of inherited.

The hues are **Paul Tol's `muted` qualitative scheme**, minus its sand and olive
(the brand already owns warm, and those two crowd `--warn`), with lightness
snapped to each theme's band and the hues kept. Rust anchors slot 1 because it is
the one value identical either side of the theme flip.

**Slots 1–3 are the scatter-safe head.** They are validated on the all-pairs
pairlist, where any two marks can be neighbours; all five are validated
adjacent-only, which is what bar and line need. Past five, colour stops being the
answer — fold to "Other" or facet.

Green is admitted here and nowhere else. `colors.css` says there is no green in
this brand, and that rule holds for chrome; the reason behind it is severity
semantics, and a chart series carrying a group code makes no severity claim. The
fence is what makes admitting it safe.

## Why

**The measure is ΔE under simulation, not hue angle.** This is the load-bearing
correction. An earlier pass used hue separation and cleared a pair sitting 29°
apart that was ΔE 1.1 under protanopia — indistinguishable — while flagging a
tighter-angled pair that was fine. Hue angle has no opinion about lightness,
chroma, or what a colour-blind reader sees. The rule is now: minimum ΔE in OKLab
to every status token, evaluated under normal, protan and deutan vision, in both
themes.

**A hard floor is not reachable and never was.** Rust sits 7.2 from the light
warning, because the brand's action colour and the status hues are cut from the
same strata. The dataviz method's own two tiers fit — target 8, floor 6, where the
floor band is legal only with secondary encoding — and a status chip's mandatory
icon and label *is* that encoding.

**Status hues are semantically pinned; the accent is not.** Asked to maximise
separation, a search turns warnings and errors purple, which is the tell that the
question was posed wrongly. Red must read as error and amber as warning, so
crowding cannot be optimised away by moving them. What can move is lightness —
the channel a cone deficiency leaves intact — which is why dark `--warn` steps up
to a pale gold rather than sideways out of amber.

**`--info` was the one genuine departure.** It was steel *grey*, and two
low-chroma colours have nothing left to separate them once hue is removed; it sat
ΔE 1.3 from `--ok` under deuteranopia. Every design system that publishes its
reasoning puts the informative slot in **blue**, because blue is the hue that
survives protan and deutan. Steel is blue-grey before it is grey, so the auger
keeps its referent.

## Consequences

- **New chart hues land on all three surfaces at once**, because the token layer
  is shared ([[dec-landing-build-shared-tokens]]). Only the app draws charts
  today; the docs and apex carry five unread custom properties, which is cheaper
  than the first exception to "one vocabulary, three surfaces".
- **The gate is the record, not this page.** `repo:web/src/shared/styles/separation.test.ts`
  holds every threshold; re-run it after any edit to either file, because the two
  sets are measured against each other and a status retune can invalidate a chart
  hue without either file looking wrong alone.
- **Two pairs are accepted rather than fixed**, pinned in the gate as ratchets so
  they may improve and not decay. Light `warn`/`err` cannot separate: both must
  clear AA as *text* on near-white, which forces them into the same dark warm
  region, and the best colour-only alternative costs a near-black warning and a
  muddy error. Dark `err`/`accent` would need a brand change to the accent, which
  is a larger call than this decision should force.
- **The series cap is enforced by the builder, and the fold has its own token**
  (#445). The caps were a comment here and in the token reader until the chart
  builder was made to spend against them: `repo:web/src/shared/styles/chartSlots.ts`
  is now the single definition of both — the slot count and the all-pairs head —
  read by the gate and by `repo:web/src/lib/chartSeries.ts`, the pure module that
  turns rows into series. Everything past a form's cap folds into one series in
  `--chart-other`, which is held to the slots' separation floors and to the
  *inverse* of their chroma rule, because a fold carries no identity to encode.
  **Colour follows the value's rank, not its series position**, and the rank is a
  SQL cardinality probe composed from the plot query's own FROM and WHERE — so
  nothing about the plotted *slice* can repaint a survivor. Composing it from the
  plot's population is what costs the other half: adding a join that fans base
  rows out changes the counts, and so can reorder the ranking. That is the trade
  taken deliberately, because it is what makes "Other" a claim about the delivery
  the chart draws. The ranking cannot be read off the plotted rows: the scatter
  query is a bare row `LIMIT` with no `ORDER BY`, so its values are an arbitrary
  slice and a legend saying "Other" over them would claim more than the sample
  supports.
- **It rules out** reintroducing hue angle as a proximity measure, and reusing the
  brand ramp for any categorical channel.

## Related

[[dec-landing-build-shared-tokens]] · repo:web/src/shared/styles/charts.css ·
repo:web/src/shared/styles/colors.css ·
repo:web/src/shared/styles/separation.test.ts ·
repo:web/src/shared/styles/contrast.test.ts ·
repo:web/src/shared/styles/chartSlots.ts · repo:web/src/lib/chartTheme.ts ·
repo:web/src/lib/chartSeries.ts · repo:web/src/components/explore/ChartBuilder.tsx ·
repo:web/docs-site/docs/stylesheets/laterite.css
