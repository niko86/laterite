# laterite-ags4-reference

**AGS4** reference data as a dependency-light leaf: the multi-edition group
dictionary, and the rules catalogue, both generated from one JSON source.

```rust
use laterite_ags4_reference::dict::{Dictionary, FALLBACK};

let dict = Dictionary::bundled(FALLBACK);
let group = dict.group("LOCA").expect("a standard group");
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-reference` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-reference
```

Currently v0.11.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## One source of truth

`ags_dictionary.json` holds **174 AGS groups** — the union across editions
4.0.3 to 4.2 — each with its 4-letter code, its parent, and an ordered tuple of
headings carrying status (KEY / REQUIRED / OTHER), AGS type, unit and
description. Everything else is generated from it at build time.

The per-edition projection is compiled into static `phf` tables by `build.rs`,
so a lookup costs no startup work and no allocation: the dictionary is in the
binary's read-only data, not parsed on first use. That matters for the wasm
build, where startup cost is visible to a user, and for CLI runs short enough
that parsing a megabyte of JSON would dominate.

`Dictionary` covers both the bundled editions and a caller-supplied overlay, so
a project dictionary that adds groups or headings to a standard edition is a
first-class case rather than something callers reimplement.

## Why it is separate

Extracted so consumers that need *only* the dictionary — a read-only database
extension, a diff tool — can depend on this instead of pulling in a validator
engine or a whole codec. It is wasm-safe and has no I/O.

## Reference data provenance

The bundled standard dictionaries are ©AGS reference data, redistributed as a
documented decision — see
[`PROVENANCE.md`](https://github.com/niko86/laterite/blob/main/rust-packages/laterite-ags4-validator/data/PROVENANCE.md),
which lives beside the source `.ags` dictionaries in `laterite-ags4-validator`.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
