//! Transport — zstd + age passphrase file envelope, the Node/napi face of the
//! shared `laterite_transport` leaf (#327). The pack/unpack/lock/unlock logic
//! lives in the leaf (ONE copy, shared with laterite-ags4-core) instead of the
//! byte-identical reimplementation that used to live here — so a transport dep
//! bump (e.g. age 0.10→0.11) is one migration, not two. This module only maps
//! the leaf to napi: `String` paths in, `BigInt` sizes out, `TransportError` →
//! `napi::Error`, camelCased (`transport_pack` → `transportPack`, `elapsed_s` →
//! `elapsedS`). The age envelope stays interoperable with the Python side
//! (`pyrage`) — same `age` crate, same on-disk format.

use std::path::Path;

use laterite_transport::TransportError;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Result of `transportPack` / `transportLock`: output size, ratio vs source,
/// elapsed seconds.
#[napi(object)]
pub struct PackStats {
    pub bytes: BigInt,
    pub ratio: f64,
    pub elapsed_s: f64,
}

/// Result of `transportUnpack` / `transportUnlock`: output size, elapsed seconds.
#[napi(object)]
pub struct UnpackStats {
    pub bytes: BigInt,
    pub elapsed_s: f64,
}

/// The leaf renders every failure (missing file, wrong password, corrupt
/// envelope) as a human message; surface it verbatim as a JS error.
fn napi_err(e: TransportError) -> Error {
    Error::from_reason(e.to_string())
}

/// zstd-compress `src` → `dest`. `level` is 1–22 (default 9, the AGS sweet spot).
#[napi]
pub fn transport_pack(src: String, dest: String, level: Option<i32>) -> Result<PackStats> {
    let s = laterite_transport::pack(Path::new(&src), Path::new(&dest), level.unwrap_or(9))
        .map_err(napi_err)?;
    Ok(PackStats {
        bytes: BigInt::from(s.bytes),
        ratio: s.ratio,
        elapsed_s: s.elapsed_s,
    })
}

/// zstd-decompress `src` → `dest`.
#[napi]
pub fn transport_unpack(src: String, dest: String) -> Result<UnpackStats> {
    let s = laterite_transport::unpack(Path::new(&src), Path::new(&dest)).map_err(napi_err)?;
    Ok(UnpackStats {
        bytes: BigInt::from(s.bytes),
        elapsed_s: s.elapsed_s,
    })
}

/// zstd-compress, then age-encrypt with `password` → `dest`. Compress-then-
/// encrypt is load-bearing: zstd needs low-entropy input; ciphertext is random.
#[napi]
pub fn transport_lock(
    src: String,
    dest: String,
    password: String,
    level: Option<i32>,
) -> Result<PackStats> {
    let s = laterite_transport::lock(
        Path::new(&src),
        Path::new(&dest),
        &password,
        level.unwrap_or(9),
    )
    .map_err(napi_err)?;
    Ok(PackStats {
        bytes: BigInt::from(s.bytes),
        ratio: s.ratio,
        elapsed_s: s.elapsed_s,
    })
}

/// age-decrypt with `password`, then zstd-decompress → `dest`. Wrong passphrase
/// / non-passphrase envelopes surface as an error.
#[napi]
pub fn transport_unlock(src: String, dest: String, password: String) -> Result<UnpackStats> {
    let s = laterite_transport::unlock(Path::new(&src), Path::new(&dest), &password)
        .map_err(napi_err)?;
    Ok(UnpackStats {
        bytes: BigInt::from(s.bytes),
        elapsed_s: s.elapsed_s,
    })
}

// No `#[cfg(test)]` here: napi 3's `Error::drop` references N-API symbols the
// Node host provides only at load, so `cargo test -p laterite-node` can't link
// a standalone harness (this crate is excluded from the coverage job for the
// same reason). The envelope round-trip is pinned in the leaf's own Rust tests
// (`laterite-transport`), and this napi boundary — pack/unpack/lock/unlock +
// the BigInt stats + wrong-password rejection — is covered end-to-end through
// the real `.node` addon by `test/p3-transport.test.ts` (vitest).
