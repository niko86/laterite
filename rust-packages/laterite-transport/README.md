# laterite-transport

A small, content-agnostic file envelope: **zstd** compression and **age**
passphrase encryption, as four operations.

```rust
laterite_transport::pack(&src, &dest, 19)?;      // compress
laterite_transport::unpack(&src, &dest)?;        // decompress
laterite_transport::lock(&src, &dest, pw, 19)?;  // compress + encrypt
laterite_transport::unlock(&src, &dest, pw)?;    // decrypt + decompress
```

Byte-oriented variants (`pack_bytes`, `lock_bytes`, …) are available for callers
that never touch the filesystem.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
## Install it

```bash
cargo add laterite-transport
```

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Content-agnostic on purpose

Nothing here knows what an AGS file is. The operations run over raw bytes, so
they work on any file at all. This crate carries no format prefix in its name
for exactly that reason — it is the one piece of the
[laterite](https://github.com/niko86/laterite) toolchain that would carry over
unchanged to a different data format.

The `age` envelope is the standard one, interoperable with other age
implementations — including Python's `pyrage`, which links the same underlying
Rust crate. A file locked here can be unlocked there and vice versa.

## Errors

One `TransportError` enum, with the failing operation named. Consumers map it to
their own error type rather than having a `Box<dyn Error>` pushed on them.

## Licence

MIT.
