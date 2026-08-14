# laterite-ags4-emit

Produce byte-faithful **AGS4** plaintext from typed or string data.

```rust
use laterite_ags4_emit::{emit_ags4, EmitOpts, GroupInput};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let groups = vec![GroupInput {
        code: "PROJ".into(),
        headings: vec!["PROJ_ID".into()],
        units: None,
        types: Some(vec!["ID".into()]),
        rows: vec![vec![Value::String("P1".into())]],
    }];

    let out = emit_ags4(&groups, &EmitOpts::default())?;
    println!("{} bytes", out.bytes.len());
    Ok(())
}
```

One host-agnostic orchestrator sits under thin native and browser frontends, so
the Python binding, the Node binding and the wasm build all emit identical
bytes rather than each having its own writer that drifts.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-emit` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-emit
```

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## What it takes care of

- **Quoting and formatting** to the spec's rules, including the
  significant-figure and decimal-place conventions, so a value read from a file
  and written back is byte-identical.
- **Metadata synthesis.** AGS4 requires `UNIT` and `TYPE` catalogue groups
  covering everything used elsewhere in the file. `emit_ags4` can derive them
  from the data rather than making the caller assemble them by hand.
- **`TRAN` is stamped by the caller, never invented.** Without a supplied
  `TranStamp` no `TRAN` group is emitted at all, and the validator reports its
  absence. A synthesised placeholder would be a claim about who transferred what
  and when — the one thing a writer has no business guessing.

The optional `arrow` feature adds construction from Apache Arrow record batches,
which is how the dataframe-shaped bindings get their data in.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
