---
type: decision
title: "Dictionary single-source: one converter → the union JSON → every consumer"
status: accepted
tags: [design, decision, architecture, dictionary]
decided: "2026-06-21"
supersedes: []
from_gap: []
related: [crate-map, repo-layout, effective-dictionary, laterite-ags4-validator, laterite-ags4-reference, laterite-ags4-core, laterite-ags4-wasm, laterite-py, laterite, data-single-source-audit, cert-trust-v2, vendored-authority-faithful, dec-custom-dict-overlay]
sources: []
---

# Dictionary single-source: one converter → the union JSON → every consumer

## Context

The AGS4 standard dictionary has one **origin**: the five official
`Standard_dictionary_v4_*.ags` files (4.0.3 → 4.2), vendored at
`repo:rust-packages/laterite-ags4-validator/data` (provenance: python-ags4
1.2.0; see that crate's `PROVENANCE.md`; that claim went unchecked until
**laterite-dev#558** — see [[vendored-authority-faithful]]). (Earlier, pre-registry dictionary
population used ad hoc scaffolder scripts (dev satellite) to infer/merge
entries from sample files instead of one canonical converter — not
production code, but the predecessor this decision's single-converter
approach supersedes.)

dec-registry-driven-generation (#173) generated the consolidated, faithful
multi-edition **union** `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json`
from those `.ags` via `tools/gen_dictionary.py`, with a CI faithfulness gate
(`tests/test_dictionary_faithful.py`). But the union didn't become the *only*
representation: the same official data was still derived **twice** —

1. `gen_dictionary.py` → `ags_dictionary.json` (consumed by the [[laterite-py]]
   registry → typed-graph + DuckDB DDL, the `.pyi` + node codegen), and
2. the validator's `build.rs` → its own per-edition `phf` tables (consumed by
   validation + the [[laterite-ags4-wasm]] `dictionary()` export → the web Tools tab).

Two independent readers of one spec, plus a third representation in the web (a
stale hand-named JSON copy). The owner directed (2026-06-21): **one converter
reads the `.ags`; the union JSON is the single hub; every consumer refs it
(even if it then projects its own internal form).**

## Decision

`gen_dictionary.py` is the **sole** `.ags` reader; `ags_dictionary.json` is the
one machine-readable dictionary; everything else reads *it*. Delivered in three
phases:

- **Web (#202).** All web dict consumers read the union: `lib/dict.ts` is the one
  loader/projector (union → `DictMap`; `projectEdition(ed)` for the Tools UIs via
  `eds`/`by_ed`); Explore, Analyse, and the Dictionary/Template browsers all use
  it. `sync-dict.mjs` copies the union into `web/public/` (a blessed build-time
  copy); the old `ags5_dictionary.json` web asset is gone.
- **core-1 (#203).** The union gains everything the validator needs beyond the
  group/heading schema: the **ABBR pick-list** values
  (`(ABBR_HDNG, ABBR_CODE) → ABBR_DESC`, Rule 16; top-level `abbreviations`,
  union'd latest + `by_ed` + `eds`) and the per-edition **`TRAN_AGS`** (Rule 14;
  top-level `tran_ags`). The faithfulness gate reconstructs both per edition.
- **core-2.** The validator's `build.rs` reads `ags_dictionary.json` (via a
  workspace-relative path to the sibling core crate) and projects each edition
  into the **same** `phf` tables — the [[laterite-ags4-wasm]] `dictionary()`
  export follows for free. The five `.ags` stay put as `gen_dictionary.py`'s
  input; `build.rs` no longer reads them. (Since relocated: laterite-dev#475 PR2 moved
  this `build.rs` + projection, and the JSON itself, into the dedicated
  [[laterite-ags4-reference]] leaf — the validator re-exports the result and,
  at the time, no longer had a `build.rs` of its own. It regained one in the
  unrelated `cert-trust-v2` arc's PR 2 (2026-07-14) for a build-time
  `ENGINE_FINGERPRINT` hash over the rule sources — at the time nothing to do
  with the dictionary projection described here, though **laterite-dev#550** (2026-07-16)
  later widened that hash to also cover the reference leaf's own `build.rs` —
  i.e. the code that *generates* the phf tables this decision describes, not
  just the JSON it reads — because a change to that projection code changes a
  verdict without changing the fingerprint otherwise. See [[cert-trust-v2]].)
- **laterite-dev#475 follow-ups (laterite-dev#493).** The reference leaf's extraction (PR1/PR2) closed
  two of the workspace's three independent Rust readers of `ags_dictionary.json`
  (the validator's `phf` projection and [[laterite-ags4-core]]'s registry both
  now resolve through it). The third — [[laterite-py]]'s `build.rs`, which
  hand-parsed the JSON itself to emit the `#[pyclass]` typed-graph codegen — is
  now **also** collapsed: it consumes `laterite_ags4_reference::union::union_groups()`
  instead of re-reading the file. Verified no drift: the regenerated `.pyi` is
  byte-identical to the committed one, so the retired reconstruction hadn't
  silently diverged. The same follow-up slimmed `laterite-ags4-diff` to depend
  on the reference leaf directly rather than the whole validator — diff only
  ever used `Dictionary`/`DictVersion`, never the rule engine, so this drops a
  transitive dependency without changing behaviour. **Net: the union/dictionary
  registry is now single-sourced from the reference leaf across the whole
  workspace** — every one of the three independent reconstructions this decision
  originally flagged (core registry, validator phf, laterite-py build.rs) now
  goes through the one leaf. The drift vector for the union projection is
  **closed**. The only remaining `.ags`→JSON generator is `gen_dictionary.py`
  itself (orthogonal — it's the one converter this decision names, not a
  reconstruction of its output).

UNIT/TYPE pick-list **values** are deliberately not carried — the validator
doesn't use them (it checks each heading's own declared unit/type).

## Why

- **No silent drift.** Two independent projections of one spec can diverge; one
  source + gates cannot. This is the same anti-drift principle as
  dec-registry-driven-generation, finished.
- **The owner's rule.** "One mechanism to JSON; everyone refs the union — generated
  build-time files from it are fine, two independent sources of the same data are
  not."

## Consequences

- **Parity preserved by construction.** core-2's regenerated `dict_data.rs` is
  **byte-identical** to the old direct-`.ags` parse (verified by diff), so
  validation behaviour + the 122/131 python-ags4 parity are unchanged. The
  validator's `build.rs` projection mirrors `gen_dictionary.reconstruct` exactly
  (flat = latest edition; `by_ed` overlay; `eds` membership; `order_by_ed`).
- **`build.rs` reads a sibling crate's data** (`../laterite-ags4-core/data/...`
  via `CARGO_MANIFEST_DIR`). Safe: the validator is `publish = false` (workspace-
  only), and the maturin sdist vendors path-dep crates as siblings — verified by
  building + installing the sdist in isolation. (This cross-crate `include_str!`
  reach was itself later removed: laterite-dev#475 PR2 relocated the JSON into the reference
  leaf's own `data/`, so `build.rs` now reads its own crate's data.)
- The validator gains a `serde_json` build-dependency (replacing `csv`).
- **Effective dictionary unchanged** — [[effective-dictionary]] (standard ∪ the
  file's DICT group) is about validation-time merge, independent of how the
  standard half is sourced.
- **The origin's own faithfulness was unguarded until laterite-dev#558.**
  `test_dictionary_faithful.py` proves the union is a faithful *render* of
  the five `.ags` files; it never proved — and by construction cannot
  prove — that the five files themselves are faithful to the source
  `PROVENANCE.md` names. Measured, not argued: appending a fabricated group
  to `Standard_dictionary_v4_2.ags` and regenerating moved the union
  174 → 175 groups with that gate still green. `tests/test_vendored_authority_faithful.py`
  closes it: byte-for-byte against the `python-ags4` copies installed as the
  dev-dependency oracle, the file *set*, `fallback_edition` against
  upstream's `LATEST_DICT_VERSION`, and the four hand-written `1.2.0`
  version claims scattered across the tree. It does **not** prove
  `python-ags4`'s dictionaries match the AGS standard itself — parity is
  structurally blind to a divergence both sides share. See
  [[vendored-authority-faithful]].

## Related

[[crate-map]] · [[repo-layout]] · dec-registry-driven-generation · [[effective-dictionary]] · [[laterite-ags4-validator]] · [[laterite-ags4-reference]] · [[laterite-ags4-core]] · [[laterite-ags4-wasm]] · [[laterite-py]] · [[laterite]] · [[data-single-source-audit]] · [[cert-trust-v2]] · [[vendored-authority-faithful]] · [[dec-custom-dict-overlay]]
