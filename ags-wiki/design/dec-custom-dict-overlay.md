---
type: decision
title: "Custom-dictionary overlay: base-as-a-property-of-the-dict, honour + warn, cert as record"
status: accepted
tags: [design, decision, architecture, dictionary]
decided: "2026-07-18"
supersedes: []
from_gap: []
related: [crate-map, laterite-ags4-reference, cert-trust-v2, dec-dictionary-single-source, edition-resolution, laterite-ags4-validator, O-28]
sources: []
---

# Custom-dictionary overlay: base-as-a-property-of-the-dict, honour + warn, cert as record

## Context

The V8 roadmap listed a runtime `--dict <path>` override, then deferred it
([[O-28]]): `Dictionary` was `'static` phf-backed (zero-startup, compiled-in
from the five official editions — see [[dec-dictionary-single-source]]), so a
runtime-parsed dictionary looked like a broad, regression-prone lifetime
refactor threaded through every rule module for a feature validation didn't
strictly need. For a period the flag was plumbed but refused with a `BadDict`
error (exit 5). **laterite-dev#568** revisited that call as a focused seven-phase arc
(designed by ultracode workflows, owner-steered) and shipped it. This page
records the decisions and the why; [[O-28]] is the observation, this page is
the design.

## Options considered

1. **A second, fully-owned `Dictionary` type for the custom case**, with call
   sites branching on which one they hold. Rejected: doubles every lookup
   surface (`heading`, `group_headings`, `abbr_desc`, …) and reintroduces the
   exact drift risk [[dec-dictionary-single-source]] closed for the bundled
   editions.
2. **Parse the custom dict into a full 174-group `Dictionary`**, discarding
   the bundled tables entirely. Rejected: throws away the zero-cost `&'static`
   phf lookup for the overwhelming majority of a custom dict's content (a
   client adding one group still needs the other 173), and duplicates the
   whole bundled schema into owned heap data on every validation.
3. **A sparse overlay `delta` on top of a bundled `base`** (chosen) — own only
   the *difference*.
4. **Always require an explicit `--dict-version`/base.** Rejected: makes the
   common case (hand a dict that's obviously "4.2 plus one group") ceremony,
   and — more importantly — couples the cert-trust CONTENT/WORLD split
   (below) to a caller-supplied fact rather than a derivable one.

## Decision

**(a) `Dictionary<'a>` is a lifetime-parametric enum, still `Copy`.**
`Bundled(BundledDict)` is the existing zero-cost `&'static` handle every
current call site returns (`Dictionary::bundled(version)` still yields
`Dictionary<'static>`, so `resolve_dict_version`, `fixes.rs`, and every
py/node/wasm caller are unchanged). `Layered { base: BundledDict, delta: &'a
OwnedDelta }` borrows a stack-local `OwnedDelta` for one validation. Both arms
are refs/statics, so the enum stays `Copy` (`repo:rust-packages/laterite-ags4-reference/src/dict.rs`)
— no call site that takes `dict: Dictionary` by value had to change. `OwnedDelta`
carries only the groups/headings a client actually adds or overrides, keyed
the same way the bundled `phf` tables are (`repo:rust-packages/laterite-ags4-reference/src/overlay.rs`)
— a client adding one group to a 4.2-shaped dictionary yields a delta with one
group, not 174.

**(b) The base is a property of the dictionary itself, detected once, before
any delivery byte is read.** `parse_dict` (the one entry point every surface
funnels bytes through) resolves the base via `BaseSpec`: `Auto` runs
`detect_base`, which scores every bundled edition by how many of the custom
dict's heading `(name, type, status)` tuples agree with that edition's own
definition, and takes the highest-scoring edition (ties broken toward the
newer one); `Force(v)` honours an explicit `--dict-version`; `Replace` drops
the base entirely (`--dict-replace`). This is the load-bearing move: because
the base is fixed **structurally**, from the dictionary's own content, before
any file is validated against it, the custom dict's identity — `{base_version,
hash}` — is computable at the surface boundary with zero delivery bytes in
hand. That is exactly what [[cert-trust-v2]]'s CONTENT/WORLD partition needs:
a certificate can only stand in for a computation that is a pure function of
`(bytes, inputs, engine)`, and "which dictionary judges this file" had to
become one of those *inputs* rather than a per-file side effect for `--dict`
to fit the existing trust model at all, instead of requiring a second one.

**(c) Input is `.ags` or JSON, auto-sniffed — no second schema.** The JSON path
(`DictFormat::Json`) deserialises the *same* `DictionaryFile` shape
`ags_dictionary.json`'s union serde already uses — not a bespoke custom-dict
JSON schema. The `.ags` path (`dict_read.rs`, the FIRST runtime AGS4 DICT-group
reader in the workspace — the bundled dictionaries are all compiled in at
build time) reuses the shared `laterite-ags4-parse` tokenizer and reconstructs
the identical shape, so both formats converge on one internal representation
before `detect_base`/`build_delta` ever see them
(`repo:rust-packages/laterite-ags4-reference/src/dict_read.rs`). One converter
principle, extended: [[dec-dictionary-single-source]] closed the *bundled*
dictionary's drift vector; this closes it for the custom path too, rather than
opening a parallel one.

**(d) Re-parenting or overriding a STANDARD heading is honour + warn, never a
silent shadow.** The overlay takes effect — a client CAN re-point a standard
group's parent or redefine a standard heading's type/status — but every such
override against the base is surfaced as a laterite-originated finding
(`emit_override_findings`, `repo:rust-packages/laterite-ags4-validator/src/lib.rs`).
The loudness has been tiered since [[O-51]]: re-parenting and a KEY→non-KEY
status demotion (the cases that change row identity) stay WARNING findings,
while a plain type/status override reports as an FYI
(`"FYI (Related to DICT)"`), gated on the tier flags like every other FYI. A full
replacement (`--dict-replace`) is exempt from this — it declares a wholesale
new schema, so there is no "standard" to have silently diverged from.

**(e) `--dict` and `--dict-version` coexist; `--dict-replace` is the only
one-way door.** `--dict-version` selects which bundled edition the overlay
sits on (`BaseSpec::Force`); the two compose. `--dict-replace` cannot combine
with `--dict-version` (validated at the surface boundary, all four
bindings) — replacement has no base for a version to select.

**(f) The certificate records, it does not contract.** The `.ags.idx` stamp
carries the effective dictionary's `{name, hash}` (`CustomDictRef`,
`repo:rust-packages/laterite-ags4-core/src/index.rs`) — an advisory label
(a declared name or the dict's filename basename, never a path) plus the
SHA-256 over `(normalised delta ⊕ base edition ⊕ mode)`. `Sidecar::decide`
compares it field-for-field against the request's own custom dict; any
difference is `RevalidateReason::DictionaryChanged` — the engine reruns, it
never silently vouches for a verdict reached under a different dictionary.
This is the same "record, not contract" posture [[O-48]] established for
`encoding` — see [[cert-trust-v2]] §2a. `custom_dict` is classified CONTENT
in `split_options`'s exhaustive destructure (not `Unsupported`, as an earlier
draft of that design anticipated it might have to stay): its identity is a
pure function of the parsed dictionary, computed once, independent of the
delivery file, so a certificate CAN and must speak for it.

**(g) All four surfaces, wasm bytes-only.** CLI `--dict <path>` +
`--dict-replace`; Python/Node `dict_path` / `dict_bytes` / `dict_replace`
(a path OR raw bytes, never both); wasm `dict_bytes` only (no filesystem in
the browser). One fast Rust core (`parse_dict` + `Dictionary::layered`), thin
per-surface bindings that each build a `CustomDict` once and hand it to
`CheckOptions::custom_dict` — no per-surface reimplementation of base
detection or delta construction.

**(h) v1 cuts, confirmed safe.** Custom ABBR picklists and a custom
`TRAN_AGS` edition-selection table are **not** overridable in v1 — a layered
dictionary's `abbr_desc` always answers from the base's picklist
(`OwnedDelta.abbrs` exists, reserved empty, so the on-disk/owned shape is
stable if a v2 populates it, but nothing constructs it yet). Confirmed safe
because Rule 16 (ABBR) and Rule 14 (`TRAN_AGS`) validity checks are about
whether a *value* used in the file matches the base's own vocabulary, not
about the custom dict's schema — a client adding a bespoke group/heading has
no need to also redefine what edition strings or abbreviation codes mean.

## Why

- **Base-as-a-property, not base-as-an-argument, is what makes the cert
  fingerprintable before parse.** Any design that resolved the base from
  delivery-file content (the way `TRAN_AGS` resolves a *bundled* edition,
  [[edition-resolution]]) would make "which dictionary judged this file" a
  WORLD-shaped question — dependent on the file, not just the dict — which
  would have forced `custom_dict` into the same never-cacheable bucket as
  Rule 20's on-disk check. Detecting the base from the dictionary's own
  content instead keeps the whole feature CONTENT-shaped by construction.
- **A sparse delta, not a full re-parse, matches the shape of the actual use
  case.** Every real request seen in practice is "the standard dictionary,
  plus/minus a handful of groups" — the AGS4 vocabulary rarely needs wholesale
  replacement, so paying for only the difference is both the cheap path and
  the one that composes with `Copy`.
- **Honour + warn over reject-or-silently-shadow.** A hard refusal of any
  override would make the overlay useless for its actual purpose (extending
  the standard schema with project-specific groups often *does* need to
  adjust a borderline standard heading); silently accepting an override with
  no signal would let a bespoke dictionary quietly reshape validation
  semantics underneath a user who expected the standard schema. Loud +
  honoured is the position that neither blocks the legitimate case nor hides
  the risky one.
- **One converter, one schema, two readers** (the `.ags` reader and JSON
  deserialisation both target `DictionaryFile`) extends
  [[dec-dictionary-single-source]]'s anti-drift principle to the runtime path
  instead of exempting it.

## Consequences

- **The bundled-only path is byte-for-byte unchanged.** `Dictionary::bundled`
  still returns `Dictionary<'static>`; no existing caller's behaviour moved.
- **`custom_dict` joined the CONTENT side of [[cert-trust-v2]]'s exhaustive
  `split_options` destructure**, so a future knob added to `CheckOptions`
  still can't compile without being classified — the destructure did not need
  loosening to accommodate this feature.
- **Companion authoring ergonomics are a tracked fast-follow, not part of this
  arc**: `lat dict export` / `convert` / `generate` (round-tripping a bundled
  edition to a starting-point `.ags`/JSON a client can then edit) are not
  built yet.
- **PROVENANCE's fallback claim is now true and gated.** `rust-packages/laterite-ags4-validator/data/PROVENANCE.md`
  states a consumer who cannot rely on the embedded ©AGS dictionary can supply
  their own at validation time — that claim was false before laterite-dev#568 (the flag
  refused). `tests/test_provenance_dict_fallback.py` pins both halves: the
  document still makes the claim, and `--dict` no longer exits 5 for a valid
  custom dictionary.

## Related

[[crate-map]] · [[laterite-ags4-reference]] · [[cert-trust-v2]] · [[dec-dictionary-single-source]] · [[edition-resolution]] · [[laterite-ags4-validator]] · [[O-28]]
