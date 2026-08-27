# laterite-ags4-trust

One door for the question *"can I trust this AGS4 file's certificate enough to
skip re-validating it?"*

```rust
use laterite_ags4_trust::{check, Request};
use laterite_ags4_validator::{CheckOptions, WorldScope};

const AGS4: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\"\r\n",
    "\"TYPE\",\"ID\"\r\n",
    "\"DATA\",\"P1\"\r\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request {
        bytes: AGS4.as_bytes(),
        opts: &CheckOptions::default(),
        // No certificate offered. An `.ags.idx` beside the file is not consent.
        cert: None,
        // Content-only: the bytes are the whole of the evidence.
        world: WorldScope::None,
        compat: None,
    };

    let outcome = check(request)?;
    println!("judged against {:?}", outcome.dict_version);
    Ok(())
}
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-trust` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-trust
```

Currently v0.11.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Why this is a crate

The question was previously answered in five places — a CLI, the Rust and
Python halves of one binding, another binding's TypeScript, and a wasm
surface — each with its own hand-written conjunction of freshness, engine
identity and profile checks. They did not agree, and four of the five would
report a file clean when it was not:

- a certificate minted with file-tree checking, then the tree deleted, stayed
  trusted — because the certified bytes themselves had not moved;
- a certificate whose warning count was never actually measured satisfied a
  request that asked to see warnings.

Consolidating the decision is the point. A trust check that is subtly different
in each caller is worse than no trust check, because it produces confident
answers instead of absent ones.

## Engine identity

A certificate records which engine judged the file, as a fingerprint over
everything that can change a verdict — the rules and the crates they run
through, not just a version string. An engine whose parser changed produces a
different identity, so certificates minted by the older one stop being honoured
automatically rather than being trusted on a version number that did not move.

`mint` issues a certificate for a file that passes; `check` decides whether an
existing one may be relied on.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
