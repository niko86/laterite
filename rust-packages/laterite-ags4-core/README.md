# laterite-ags4-core

The DuckDB-free core of the [laterite](https://github.com/niko86/laterite)
**AGS4** toolchain: the read codec, the dictionary registry, and the `.ags.idx`
certificate and byte-offset index.

```rust
use laterite_ags4_core::ags4_codec::{read_ags4_bytes_with, ReadOptions};

const AGS4: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\"\r\n",
    "\"TYPE\",\"ID\"\r\n",
    "\"DATA\",\"P1\"\r\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = read_ags4_bytes_with(AGS4.as_bytes(), ReadOptions::default())?;
    println!("{} group(s)", parsed.order().len());
    Ok(())
}
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-core` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-core
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->

## What is in here

- **`ags4_codec`** — the AGS4 reader, producing groups of pure strings. No type
  coercion happens at this layer; a file round-trips through it unchanged.
- **`registry`** — the multi-edition group dictionary and the group tree.
- **`index`** — the `.ags.idx` sidecar: per-group byte ranges plus a validation
  stamp recording *which engine* judged the file and *under what options*. The
  byte ranges let a consumer seek directly to a group without re-reading the
  file; the stamp is what makes it safe to skip re-validation.
- **`keychain`** — content-addressed row keys.
- **`transport`** *(default feature)* — re-exports the zstd + age envelope from
  [`laterite-transport`](https://crates.io/crates/laterite-transport).

## Pure strings, deliberately

Nothing here converts a value to a number. AGS4 is a text interchange format
and its files are frequently the authoritative artefact in a contractual
handover, so the read path preserves bytes exactly as delivered. Typed access is
a separate, opt-in layer
([`laterite-ags4-types`](https://crates.io/crates/laterite-ags4-types)).

Turn the `transport` feature off for a build with no cryptography dependencies —
the wasm and certificate-only consumers do.

## Licence

MIT.
