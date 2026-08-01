# laterite-ags4-core

The DuckDB-free core of the [laterite](https://github.com/niko86/laterite)
**AGS4** toolchain: the read codec, the dictionary registry, and the `.ags.idx`
certificate and byte-offset index.

```rust
let parsed = laterite_ags4_core::ags4_codec::read_ags4_bytes(&bytes, opts)?;
```

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
