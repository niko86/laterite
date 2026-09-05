# laterite-ags4-hostopts

One copy of the caller-facing option normalisation every **laterite** AGS4
surface shares — edition labels, write modes, custom dictionaries, staged
atomic writes.

```rust
use laterite_ags4_hostopts::{edition, edition_or_fallback, write_mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // "auto" stays deferrable for doors that can resolve it later…
    assert!(edition(Some("auto"))?.is_none());

    // …and collapses to the dictionary's own generated fallback for the
    // doors with nothing to defer to.
    let ed = edition_or_fallback(Some("auto"))?;
    println!("auto resolves to {ed:?}");

    // A bad label is refused with every accepted spelling in the message.
    assert!(write_mode(Some("nope")).is_err());
    Ok(())
}
```

The Python, Node, wasm and CLI bindings each take the same knobs from their
callers; this crate is the single parser behind all of them, so an edition
added to the dictionary reaches every surface's accepted set — and every
surface's error message — in the same commit. What stays per-surface is data,
not logic: the flag spellings that surface's user actually typed, so a
refusal names the knob as the caller spelled it.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-hostopts` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-hostopts
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->
