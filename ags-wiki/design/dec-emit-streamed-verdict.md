---
type: decision
title: "build-and-judge without the double parse hold — the streamed join + the writer-built verdict (queue M2's shape)"
status: accepted
tags: [design, decision]
decided: 2026-09-01
supersedes: []
from_gap: []
related: [laterite-ags4-emit, perf-campaign, dec-emit-cell-representation, dec-parse-structure-layout, dec-parse-cell-representation]
sources: []
---

# build-and-judge without the double parse hold — the streamed join + the writer-built verdict

> [!note] **Precondition status (2026-09-01):** [[dec-parse-structure-layout]]'s
> spike missed its own floor, and the owner then landed the layout anyway at
> the measured prize (#850 — a dated waiver of the floor for that row). So
> the post-layout lane this page's re-price waits on **exists once M6's land
> PR merges**; the M2 ticket is still minted only after that re-price runs,
> exactly as written below.

The write-side successor of [[dec-emit-cell-representation]], and
**conditional on [[dec-parse-structure-layout]]'s spike gate**: this page
records the settled shape of queue M2's fix, whose ticket is minted only
after a re-price on the post-layout lane. The prices live on the M2 row of
[[perf-campaign]]'s memory queue and on #848 (attribution probe, spike
A/B/A, and the grilled decisions this page transcribes).

## Context

The #848 probe corrected M2's queued mechanism: post-M4, on the lane's
rungs, the write peak is **not** per-cell-bound. The formatted
`OwnedGroup` slab is dropped before the validating parse-back runs (the
#790 discipline held), and what the peak actually carries twice is the
**parse hold** — the caller's retained `ParsedFile` and the emit door's
own re-parse of the bytes it just wrote, live together at t-gmax. The
per-cell costing survives only as the dense-cell worst case (the TREL
diagnosis row), not as the lane rungs' mechanism.

The #848 spike then measured the fix space and found the peak standing on
two **co-peak shoulders that back each other up**: short-circuiting the
validate tail alone moved the peak almost nothing (the slab moment still
stood), streaming the slab alone landed well under the invasive floor
(the parse-back moment still stood), and only the pair cleared rule 10's
floor at the gate rungs — with the entire margin above the floor being
what a real validation mechanism may cost, since the spike's tail-skip
deleted the verdict outright. The A/B/A record, byte-identity checks and
per-leg numbers are on #848.

So the design problem is exactly: **keep build-and-judge** — the same
call that emits the bytes judges them, `BuildResult` never hands over
unchecked output — **while never holding a second whole-file
`ParsedFile` behind a streamed writer.**

## Options considered

1. **An opt-in unchecked mode / lazy findings.** Rejected: it moves the
   contract instead of the mechanism — the default path keeps the cost,
   and only `Report` could even be lazy (Strict must raise, AutoFix
   rewrites bytes).
2. **Streamed writer + constructed verdict on today's parse layout.**
   Rejected by arithmetic: the buffer can be shared (below), but today's
   per-row/per-line structure alone re-adds enough hold that the pair
   misses the invasive floor at every rung (priced on #848).
3. **Streamed join + writer-built verdict on the post-layout structure.**
   Chosen; the shape below, conditional on
   [[dec-parse-structure-layout]].
4. **Incremental per-group validation** (retire the whole-file verdict
   object). Rejected: the relational rule families are cross-group by
   nature; this is a rules-engine redesign nothing in the evidence
   motivates.

## Decision

**The streamed join.** `emit_owned_groups`' pipeline becomes per-group:
format one group's `OwnedGroup`, write its section, drop it — for **both**
doors (the Arrow door and the cell-rows door), which continue to meet at
the same join point, now group-at-a-time. Byte output is identical by
construction (the writer's own separator discipline), and the public
crate API (`emit_ags4`, `emit_ags4_owned`, `emit_ags4_from_arrow`,
`EmitResult`) is unchanged.

**The writer-built verdict.** The writer authors every byte it emits —
each terminator, separator line and quote — so a counting writer records
the verdict's lines and spans **as it writes**, and the emitted bytes are
adopted as the verdict's buffer (`Arc<String>`, taken back zero-copy
after the check): no `parse_bytes` runs on the happy path, and no second
buffer exists. The constructed object populates exactly the fields the
validator and `compute_fixes` read (the #848 inventory: `groups`,
`group_order`, `raw_lines`, `has_bom`, line text via spans, per-group
descriptor rows and lines, per-row line numbers and value spans);
source-byte fields follow [[dec-parse-structure-layout]]'s profile rules
(absent under the validating profile).

**The equivalence contract.** One differential test holds the constructed
verdict honest, with **both** definitions: findings-equality (running
`check_parsed` over the constructed object and over a real
`parse_bytes` of the same bytes yields identical findings) and
field-equality on the read-inventory (which localises a failure when the
first fires). It runs over the forge rungs and the parity corpus. The
constructed side is fed by the writer's own emission machine, never a
re-implementation of it — M4's tokenizer lesson
([[dec-parse-cell-representation]], why `scan_line` could not serve)
applies verbatim.

**Synthesis under streaming.** `synthesise_metadata` derives its
catalogues from accumulators filled as the groups stream (unit/type sets,
PA-code detection), and the synthesised groups are formatted and written
last — the byte order today, preserved. No permanent non-streamed fork
exists; the accumulator path is differential-tested against the batch
result.

**The fix branch.** When AutoFix finds actual safe fixes, `apply_fixes`
re-parses as today — the rare path pays a bounded, documented re-parse
rather than complicating the mechanism for a branch no measurement has
flagged.

**The to-disk rider.** `build_ags4(..., out=path)` — the `fix(out=)`
idiom — streams the judged bytes to disk and returns a result carrying
`path` and no `bytes`; the `.pyi` overloads keep the two result shapes
honest to type-checkers. Node mirrors it on `buildAgs4`; wasm is excluded
(no filesystem — a modality fact, recorded as such); the CLI has no build
door, so there is nothing to mirror there. Writes go to a temp file in
the destination directory with any autofix rewrite applied there, and
`os.replace` moves it into place only once the verdict allows — the
destination path never holds unjudged bytes, so the build-and-judge
contract survives translation to disk. The rider's motivation is caller
steady-state (a result without a whole-file hold, for long-lived
processes); it is **explicitly not part of the floor arithmetic** — the
verdict needs the whole text resident regardless, so the op peak does not
move with it.

**Gate.** The land ticket (minted only after the post-layout re-price)
gates on rule 10's invasive floor at the **100 MB and 265 MB rungs** of
the write peak, A/B/A-bracketed; the 25 MB cell is measured and recorded
as a structural shortfall (the import floor's share of that rung's
denominator — even #848's no-verdict ceiling barely cleared there), never
skipped silently. Under the bar: an M1-style decline, and this page flips
to `rejected` with the record cited.

## Why

- **The pair, not either half** — #848's spike is the whole argument: the
  co-peaks back each other up, so shares are not contributions and a
  half-fix moves the instrument by its co-peak gap, not its slice.
- **Writer-built over re-parse** because the writer already knows
  everything the parse-back re-derives, and the buffer share makes the
  emitted bytes and the verdict's text one allocation instead of two.
- **Dependency on the layout row** because the verdict's residual cost is
  pure structure; on today's layout the pair misses the floor, and
  pretending otherwise would mint a ticket the re-price would kill.
- **Temp + rename** because it is the only disk semantics where the
  contract survives: strict failures leave nothing behind, autofix
  rewrites happen before the path exists, and same-directory `os.replace`
  keeps the move atomic on one filesystem (and correct on Windows).
- **Caller-side levers stay out of the queue**: dropping the handle
  before building and feeding raw capsules are real, measured savings
  (#848), but they are the caller's allocations, not the door's — they
  ship as a docs recipe and a ledger note, not as a fix row.

## Consequences

- Sequenced behind [[dec-parse-structure-layout]]: a layout decline
  leaves option 2's arithmetic in force and this row heads to a written
  decline of its own; a layout GO triggers the re-price that decides the
  mint.
- Commits the emit crate to the constructed-verdict equivalence test as a
  permanent gate — the differential test is the contract's enforcement,
  not scaffolding.
- `BuildResult` grows the `out=` variant: two result shapes under one
  door, held apart by typed overloads on both surfaces.
- Rules out: an unchecked mode; a permanent synthesis fork; treating the
  to-disk rider as a peak argument; incremental validation as this row's
  mechanism.
- The caller-facing recipe (drop the handle once frames exist; pass
  capsule-bearing tables) lands on the write door's docs page with the
  ledger note pointing at #848's variant table.

## Related

[[perf-campaign]] · [[dec-emit-cell-representation]] ·
[[dec-parse-structure-layout]] · [[dec-parse-cell-representation]] ·
[[laterite-ags4-emit]] ·
repo: rust-packages/laterite-ags4-emit/src/emit.rs ·
repo: rust-packages/laterite-ags4-emit/src/arrow_in.rs ·
repo: rust-packages/laterite-ags4-emit/src/writer.rs ·
repo: packages/laterite/python/laterite/__init__.py
