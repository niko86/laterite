---
type: decision
title: "AGS4 merge semantics: union, argument-order authority, KEY-based identity"
status: accepted
tags: [design, decision, architecture, merge]
decided: "2026-07-12"
supersedes: []
from_gap: []
related: [crate-map, laterite-ags4-reference, laterite-ags4-types, rule-08-typed-values, rule-17-type-group, dec-duckdb-extension]
sources: []
---

# AGS4 merge semantics: union, argument-order authority, KEY-based identity

## Context

Real geotechnical delivery is incremental: a site investigation issues one
AGS4 file per round (a batch of boreholes, a lab-results supplement, a
re-survey), each carrying only what that round captured. Reconciling N such
deliveries into one file — cross-referencing rows, resolving conflicting
values, tracking what changed — was previously a manual (spreadsheet) task.
`laterite-ags4-merge` (new crate, `repo:rust-packages/laterite-ags4-merge`)
is a wasm-safe leaf that automates it: `merge_parsed(files: &[ParsedFile],
opts: &MergeOpts) -> Result<MergeResult, MergeError>`
(`repo:rust-packages/laterite-ags4-merge/src/lib.rs`).

It reuses the shipped primitives rather than reinventing them: the shared
parse leaf (`laterite-ags4-parse`) for tokenising, the reference leaf
(`laterite-ags4-reference`) for the dictionary + the row-identity keychain,
`laterite-ags4-types::parse_value` for type-aware cell comparison, and
`laterite-ags4-emit` for byte-faithful output. Landed alongside a **row-
identity consolidation**: `keychain` (with its `key_heading_names` — the one
definition of "what KEY headings identify a row") moved from
`laterite-ags4-core` into `laterite-ags4-reference`
(`repo:rust-packages/laterite-ags4-reference/src/keychain.rs`, re-exported
unchanged at `laterite-ags4-core::keychain` for existing consumers), so merge
and `laterite-ags4-diff` — the two things that need to match rows across
files — share one derivation instead of each re-deriving it.

Several semantic questions have no single obviously-correct answer for AGS4
specifically (no per-row timestamp, `KEY` headings are the only declared row
identity, groups nest hierarchically); this decision records the answers
chosen and why.

## Options considered

1. **Intersection semantics** (only rows present in every file survive).
   Rejected: silently drops legitimate single-file data — a producer who
   sent one supplementary group in only one delivery would lose it.
2. **Union semantics, later-file-wins on conflict** (chosen).
3. **Timestamp-based recency** (trust an explicit revision date per row).
   Rejected: AGS4 carries no per-row timestamp; the closest proxy,
   `TRAN_DATE`, is file-level only — a per-row precedence rule would be
   fabricating precision the format doesn't have.
4. **A delete/supersede primitive** (a row can explicitly retract an earlier
   one). Deferred: no such primitive exists in AGS4 groups today, and adding
   one would be a own schema extension, not a projection of the spec.

## Decision

**Union, never intersection.** A row or group absent from a later file reads
as *silence*, not deletion — a producer simply expressed no opinion on it
that round. Merge only ever *adds* to the accumulated state.

**Argument order is authority; `TRAN_DATE` only cross-checks.** Callers
supply files in delivery order; when two files carry the same KEY with
different content, the later argument wins. If a later file's `TRAN_DATE`
predates an earlier one's, that's *warned* (`recency_contradiction`), never
blocking — `TRAN_ISNO`/`TRAN_STAT` are free text (AGS type `X`), only
`TRAN_DATE` (`DT`) is machine-orderable, and even that is file-level, blind
to a per-row regression inside an overall-newer file.

**Rows are identified by dictionary KEY headings — the ONE shared
definition.** `keychain::key_heading_names(&GroupDescriptor)`, filtered to
the KEY headings actually present in a group's union schema (a child missing
an ancestor KEY, e.g. `LOCA` without `PROJ_ID`, keys on what it has). An
unregistered/keyless group (custom or passthrough) falls back to
whole-union-tuple identity — it dedups only rows identical across every
column, so it never merges distinct rows and never loses data, but a
same-content re-send collapses.

**TYPE conflict handling — a three-way lattice, not a bool.**
replaced the two-state `lenient: bool` with a typed `on_type_clash` choice —
`TypeClashMode::{Error, Widen, Promote}`
(`repo:rust-packages/laterite-ags4-merge/src/lib.rs`), single-sourced via
`TypeClashMode::ALL` + its `FromStr`: the CLI's clap `PossibleValuesParser` is
*projected* from `ALL`, and Python/Node/wasm all parse the caller's string
through the same `FromStr`, so no surface can drift on the token names or the
rejection message. Because merge shipped in #494 on 2026-07-13 but 0.7.0 (the
last PyPI/npm publish) went out 2026-07-08, `merge()`/`lenient` had never been
released — #500 is a clean replacement, not a breaking change, and carries no
deprecated alias.

- `error` (default) — refuse. Reconciling two independent producers' declared
  types is high-stakes and less reversible than a single-file fixup, so any
  automatic resolution must be opted into.
- `widen` — fall back to `X` (text, the top of the AGS type lattice — lossless
  at the byte level, every raw value kept). This is exactly the old `lenient`.
  Typed-vs-`X` widens silently (`X` trivially absorbs anything); two
  *different* non-`X` types warn (`type_widened`) even under `widen` — that's
  a genuine disagreement, not just a widen.
- `promote` (new) — keep the column *numeric* where that's possible without
  losing a digit: when **every** clashing code is in the `nDP` family, join to
  **max(n)** and zero-pad the coarser files' cells (`2DP` + `5DP` → `5DP`;
  `10.00` → `10.00000`, warned as `type_promoted`). Anything else — `nSF`,
  `nSCI`, `DT`, `X`, any cross-family clash — falls through to `widen`,
  deliberately: `canonical_type` maps `2DP`/`3SF`/`2SCI` all to the same
  `Decimal` bucket, too coarse to drive this, so the lattice keys on the AGS
  **code family** via the new `laterite_ags4_types::decimal_places(code) ->
  Option<usize>` (`nDP` only — narrower than `canonical_type` on purpose).
  Padding *decimal places* is a formatting change; padding *significant
  figures* would overstate measurement precision (`3SF` → `5SF` asserts two
  digits the instrument never resolved) — that's why `nSF`/`nSCI` are excluded
  even though they're also parametric-numeric.

**Promote, never demote.** `max(n)` is the only lossless direction — the lower
precision would round (`10.00123` → `10.00`) and destroy data — which also
makes the outcome **independent of argument order**, deliberately unlike the
KEY-conflict rule (later argument wins).

**`promote` is the only mode in which merge rewrites a cell.** The new
`laterite_ags4_types::pad_decimals(raw, n) -> Option<String>` does it —
**string-only, never via `f64`**: the validator's existing `format_ndp` is an
f64 round-and-render (right for a Rule 8 *fix*, where rounding is the intent;
wrong for a *widen*, and an f64 round-trip silently perturbs a value past
2^53). `pad_decimals` returns `None` — "keep the producer's bytes verbatim" —
for anything it cannot pad losslessly (more places than the target, or not a
number); merge then emits that cell byte-for-byte and raises a
`promote_value_kept_verbatim` warning rather than rounding it.

**Two rule interactions, verified against the shipped validator (`promote.rs`
emits every fixture under `EmitMode::Strict`, which refuses to write a file
breaking any error-severity rule — so bytes coming back at all *is* the
validity proof):**
- **Rule 8** ([[rule-08-typed-values]]) — promoting the TYPE without rewriting
  the values yields an invalid file (`5DP` declared, `10.00` present → Rule 8
  error). That's *why* the pad is mandatory, not optional.
- **Rule 17** ([[rule-17-type-group]]) — a promoted code (e.g. `5DP`) is
  always one an input already declared, so its `TYPE`-group row rides in free
  with merge's group union; a declared-but-now-unused leftover code (e.g. the
  old `2DP` row) validates clean too — Rule 17 only requires every code *used*
  to be *declared*, not the reverse, and there is no rule requiring a
  heading's TYPE to equal the *dictionary's* TYPE (promoting `LOCA_GL` from
  `2DP` to `5DP` is spec-legal even though the dictionary says `2DP`).

**The composition payoff with `_content_hash` (#448/#499).** `content_hash`
canonicalises a cell *through its declared TYPE*
(`repo:rust-packages/laterite-ags4-reference/src/keychain.rs`), so `10.00`
hashes as a **number** under `2DP` but as a **string** under `X`. A *widened*
merge therefore no longer value-dedups against its own typed inputs, while a
*promoted* one does — pinned by a passing test in both
`packages/laterite/tests/test_merge.py` and
`rust-packages/laterite-ags4-merge/tests/promote.rs`. This composition
argument didn't exist when merge v1 shipped `Strict`/`Lenient` — `promote` is
the response to it, not a re-litigation.

**#448 rollout now complete, every read surface.** `_content_hash` ships on
Python (#499/#536), Node (#537), wasm (#538) and — as of laterite-duckdb#28
— the DuckDB extension's `read_ags`/`read_ags_text`, where it is
trailing and always-on (the library surfaces keep it opt-in). `SELECT
DISTINCT ON (_content_hash)` is the SQL form of the value-dedup this section
describes; see [[dec-duckdb-extension]] for the extension-side detail.

**UNIT conflict handling — fatal in EVERY `on_type_clash` mode, `promote`
included.** This is
the one place merge is deliberately *less* forgiving than it is about types, and
the asymmetry is the whole reason it exists: **`TYPE` has a universal absorber
(`X`); `UNIT` has none.** There is no supertype of metres and millimetres, and
merge must never *convert* — AGS units are free text, not a unit system. So the
only choices on a genuine unit clash are (a) pick one and silently mislabel the
other file's values, or (b) refuse. Merge originally did (a) — "first non-empty
`UNIT` wins" — which is **undetectable data corruption**: given `LOCA_GL` in `m`
(`10.00`) and in `mm` (`10500.00`), *both survive as valid `2DP` numbers under
the surviving `m` label*, so no downstream check can ever catch it and the
borehole's ground level silently becomes 10,500 metres. Contrast a `DT`
format clash (`yyyy-mm-dd` vs `dd/mm/yyyy`, whose format lives in the `UNIT`
row): that at least trips Rule 8 on the merged file. The numeric case trips
nothing, ever. Hence (b), unconditionally —
`MergeError::UnitConflict` / kind `unit_conflict` / exit 6, and the message
deliberately offers **no** `--on-type-clash` hint — unlike `TypeConflict`,
where the CLI prints both the `promote` and `widen` escape hatches, no mode
can absorb a UNIT clash, so hinting one would just send the caller in a
circle. **Blank is not a
disagreement**: an empty `UNIT` means "unspecified", so blank-vs-`m` resolves to
`m` and all-blank stays empty for emit to fill from the dictionary — only two
*different non-empty* units conflict. (found while speccing the
TYPE lattice in #500, and strictly more urgent than it.)

**A KEY-value correction is architecturally inseparable from a new row —
pinned, not a bug.** If a later file corrects `"BH1"` → `"BH01"`, merge has
no way to know that's a rename rather than a new borehole: both rows
persist. This is an inherent limit of KEY-based identity with no
delete/supersede primitive (see Options #4) — documented, not silently
"fixed" by guessing intent.

**The merged file IS a new transmission.** Callers may supply a
`TranStamp` (`tran_issue` + `tran_date`, both required together) to write a
freshly synthesised `TRAN` row recording every input's `ISNO`/`DATE` in
`TRAN_REM` for provenance. Without a stamp, `TRAN` is reconciled like any
other group (newest wins) and a warning (`tran_not_stamped`) notes no
merge-transmission stamp was supplied.

**A per-row TYPED revision report.** `RevisionNote` records the group, KEY
tuple, changed headings, and winning file index — but only when the *typed*
comparison (`parse_value`) disagrees. A formatting-only change (`"1.0"` →
`"1.00"`) is not a revision; a TYPE widen over identical raw bytes (e.g.
`2DP`→`X`) is not a revision either — the type change would otherwise make
equal bytes compare unequal across the boundary.

**Cardinality differs by surface, deliberately.** `lat merge <files...>
--out`, `laterite.merge(*sources, …)`, and Node's `merge(sources[], …)` are
all N-ary (2+ files) — the same reconciliation a delivery of any batch size
needs. The browser **Tools → Merge** tab is 2-file only
(`repo:rust-packages/laterite-ags4-wasm/src/lib.rs::merge`,
`repo:web/src/components/tools/MergeTool.tsx`) — a deliberately narrower UI
for the common "merge one incoming delivery into what I'm working on" case,
not a capability gap in the underlying leaf (which is N-ary).

## Why

- **Silence ≠ deletion is the only safe default for a *reconciliation* tool**
  — merge cannot know whether an omission is deliberate or incidental, so it
  must never destroy data a caller didn't explicitly ask to drop.
- **Argument order as authority is the simplest, most auditable precedence
  rule** available given AGS4 has no per-row timestamp; anything smarter
  would be inventing precision the format doesn't carry.
- **Sharing `key_heading_names` with diff closes a drift vector** the same
  way the #475 dictionary-leaf extraction did: two independent derivations of
  "what identifies a row" could silently diverge; one shared function cannot.
- **TYPE-conflict `error`-by-default matches the "required CLI args, not
  hard-coded defaults" convention's spirit** — a lossy, hard-to-reverse
  decision (widening or promoting a column's declared type) needs an
  explicit opt-in, not a silent guess.
- **`promote` keys on the AGS code family, not `canonical_type`** — the
  taxonomy that drives every other type-aware decision in this codebase is
  deliberately too coarse for this one call: it can't tell `2DP` from `3SF`,
  but promote must, because padding one is a formatting change and padding
  the other overstates measurement precision.

## Consequences

- **Behaviour-neutral for existing surfaces.** The keychain relocation is a
  pure move + re-export shim; the content-addressed `_id`/`_parent_id` golden
  UUIDs (#303) are unchanged, and `laterite-ags4-diff`'s row-matching
  behaviour is unchanged (it now calls the same function from a new home).
- **No delete/supersede primitive exists**, so a KEY-value correction always
  reads as a new row. A future supersede primitive is the only way to change
  this — tracked as an open idea, not built here.
- **The `.ags5db` writer's stateless cross-delivery merge remains a separate,
  still-open follow-up** ([[dec-duckdb-extension]] Consequences) — dedup
  *inside a persisted store* via the same keychain, a different problem from
  this leaf's file-to-file reconciliation with no store at all.
- **Test coverage**: 46 crate tests (`poc.rs` 15 + `hardening.rs` 10 +
  `unit_conflict.rs` 7 + `promote.rs` 12 (#500's lattice, `EmitMode::Strict`
  as the Rule 8/17 validity oracle) + `properties.rs` 2 proptest invariants —
  order-independence of the merged KEY set, and a reformatted re-send
  collapsing with zero revisions), 16 Python
  (`packages/laterite/tests/test_merge.py`), 14 Node
  (`rust-packages/laterite-node/test/p3-merge.test.ts`), CLI integration
  tests (`rust-packages/laterite-ags4-check/tests/cli_merge.rs`), and a
  browser e2e (`web/e2e/merge.spec.ts`).

## Related

[[crate-map]] · [[laterite-ags4-reference]] · [[laterite-ags4-types]] · [[rule-08-typed-values]] · [[rule-17-type-group]] · [[dec-duckdb-extension]] · dec-registry-driven-generation · AGS5 experiment: dual dedup (raw-string _content_hash contrast)
