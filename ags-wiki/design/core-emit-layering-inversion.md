---
type: decision
title: "core→emit layering inversion: cut (#441)"
status: accepted
tags: [design, decision, architecture, crate-map]
decided: "2026-07-11"
supersedes: []
from_gap: []
related: [crate-map, laterite-ags4-core, laterite-ags4-reference, laterite-py, dec-duckdb-extension, dec-laterite-ags4-types-leaf]
sources: []
---

# core→emit layering inversion: cut (#441)

**Resolved 2026-07-11**: the `core → emit` edge is gone.
This page records **why** the inversion existed, **how it was cut**, and **why an
obvious alternative "fix" (extracting the dictionary) does not work** — so none is
re-derived.

## The inversion (before the cut)

`laterite-ags4-core` **unconditionally** depended on `laterite-ags4-emit` (the AGS4
*writer*), which **unconditionally** depends on `laterite-ags4-validator` (rule
engine + dictionary). So **every consumer of `core` transitively compiled `emit` +
the whole `validator`**, and the foundational data crate depended on the writer —
backwards.

The edge existed for **one** reason: `core::error::CliError` carried
`impl From<laterite_ags4_emit::EmitError>` (`repo:rust-packages/laterite-ags4-core/src/error.rs`),
so callers of `write_ags4` got `?`-ergonomics. Nothing else in `core` ever touched
`emit`, and `write_ags4`'s only caller was `laterite-ags4-excel`.

## Why the "extract the dictionary" fix is a dead end

Surfaced while scoping a lean read-only DuckDB extension ([[dec-duckdb-extension]],
`niko86/laterite-duckdb#17`): the plan was to drop `laterite-ags4-validator` from the
extension to slim it / help wasm. It does not work, for two independent reasons:

1. **`core → emit → validator` is unconditional**, so the extension pulls the
   validator transitively through `core` regardless of its direct dependency —
   removing the direct dep slims nothing. Moving the `Dictionary` into `core` (or a
   new leaf) to "free" the extension additionally hits a **package cycle**
   (`validator → core → emit → validator`) and, even without the cycle, changes no
   real weight.
2. **The validator is already wasm-safe** — `laterite-ags4-wasm` depends on it
   directly ("its pure, filesystem-free library API"), so it was never the wasm
   blocker. The wasm gap is the DuckDB / `getrandom` / emscripten build plumbing
   (`niko86/laterite-duckdb#16`), unrelated to these crates.

So the dict-extraction was abandoned. The inversion is the real underlying item.

## What was done (2026-07-11)

The cut was simpler than the options first framed. #473 had already emptied `CliError`
of its CLI/ags5 baggage — it is now just `FileNotFound` + `Schema`, a near-native data
error — so relocating it (the old **Option B**, a new `laterite-cli-error` crate)
became unnecessary. Since `write_ags4`'s sole caller is `laterite-ags4-excel`, the whole
edge collapses to **moving one conversion to its one consumer**:

1. Deleted `impl From<EmitError> for CliError` from `core/error.rs`.
2. Moved that mapping verbatim into `laterite-ags4-excel` as a private `emit_err(e)` helper
   (excel already deps `emit` directly), and changed its one `write_ags4(…)?`
   call-site to `.map_err(emit_err)?`.
3. Dropped `laterite-ags4-emit` from `core`'s `Cargo.toml`.

`CliError` **stays in `core`**. `core` no longer depends on `emit` (`cargo tree -i
laterite-ags4-emit` no longer lists `core`), so the `core → emit → validator`
transitive chain is broken and the crate-map edge is gone. The rejected **Option C**
(flip the impl into `emit`) still stands rejected — `emit` deliberately avoids `core`
to stay a wasm-lean leaf ([[dec-laterite-ags4-types-leaf]]).

**Caveat — payoff is latent.** No shipped consumer avoids `emit`/`validator` *today*,
so this shows no binary-size win yet; its value is structural — the layering is now
correct, and it's a prerequisite for a genuinely lean `core` consumer. The one that
wants it — the read-only DuckDB extension ([[dec-duckdb-extension]]) — still directly
deps `validator` for the dictionary, so it needs the **dictionary-leaf extraction**
too before it slims (see the dead-end section above: dict-extraction is the part that
hits a package cycle). This cut was the half that was clean; the dictionary-leaf
extraction (#475) is the half that followed: **PR1** (#488) moved the union registry
out of `core`, and **PR2** (#492) moved the rest — the per-edition `phf` projection and
the rules catalogue out of the validator, plus the bundled JSON data itself — into
[[laterite-ags4-reference]], which now carries everything the DuckDB extension needs.
The **repoint** itself — pointing the extension and `laterite-ags4-diff` at the leaf
instead of the whole validator — is half done: #475's in-tree follow-up (#493)
repointed `laterite-ags4-diff` (it only ever used `Dictionary`/`DictVersion`, never
the rule engine) and, while there, also repointed [[laterite-py]]'s `build.rs` onto
the leaf's `union_groups()`, retiring a third independent reader of the union JSON.
The DuckDB extension's repoint is the **remaining** half — still-open, separate,
owner/mirror-gated (its own repo, `niko86/laterite-duckdb`).

## Related

[[crate-map]] · [[laterite-ags4-core]] · [[laterite-ags4-reference]] · [[laterite-py]] · [[dec-duckdb-extension|laterite-duckdb: the lean read-only reader]] · [[dec-laterite-ags4-types-leaf|the wasm-lean leaf precedent]]
