//! Transport helpers — zstd compression + age passphrase encryption.
//!
//! The pack/unpack/lock/unlock logic now lives in the shared
//! [`laterite_transport`] leaf (#327), so core and laterite-node no longer
//! carry two byte-identical copies of the age/zstd envelope. This module is a
//! thin `CliError`-returning face over that leaf: `laterite-py` binds
//! `core::transport::{pack,unpack,lock,unlock}` unchanged (same signatures,
//! same `PackStats`/`UnpackStats` fields, same `CliError`), and the
//! `From<TransportError>` below keeps the exit-code/message mapping identical to
//! when the code lived here.

use std::path::Path;

use crate::error::CliError;
// Re-export the stat shapes so `core::transport::PackStats` still resolves for
// existing callers (laterite-py reads the fields directly).
pub use laterite_transport::{PackStats, SCRYPT_LOG_N, UnpackStats};

/// `FileNotFound` keeps its dedicated variant (exit code 3); every other
/// transport failure is a schema-level error whose Display is already the
/// human message the CLI/py surface expects (`schema error: read: …`, etc.).
impl From<laterite_transport::TransportError> for CliError {
    fn from(e: laterite_transport::TransportError) -> Self {
        match e {
            laterite_transport::TransportError::FileNotFound(p) => CliError::FileNotFound(p),
            other => CliError::Schema(other.to_string()),
        }
    }
}

/// zstd-compress `src` → `dest`. `level` is 1-22 (9 is the empirical
/// sweet spot on AGS data — see `commands/pack.rs` docstring).
pub fn pack(src: &Path, dest: &Path, level: i32) -> Result<PackStats, CliError> {
    Ok(laterite_transport::pack(src, dest, level)?)
}

/// zstd-decompress `src` → `dest`.
pub fn unpack(src: &Path, dest: &Path) -> Result<UnpackStats, CliError> {
    Ok(laterite_transport::unpack(src, dest)?)
}

/// zstd-compress, then age-encrypt with `password`. Output goes to `dest`
/// (suffix `.zst.age` conventional). The compress-then-encrypt order is
/// load-bearing: zstd needs low-entropy input; encrypted bytes are random.
pub fn lock(
    src: &Path,
    dest: &Path,
    password: &str,
    level: i32,
    log_n: u8,
) -> Result<PackStats, CliError> {
    Ok(laterite_transport::lock(src, dest, password, level, log_n)?)
}

/// age-decrypt with `password`, then zstd-decompress.
pub fn unlock(src: &Path, dest: &Path, password: &str) -> Result<UnpackStats, CliError> {
    Ok(laterite_transport::unlock(src, dest, password)?)
}

/// zstd-compress bytes → bytes in memory (the filesystem-free form of [`pack`]).
pub fn pack_bytes(data: &[u8], level: i32) -> Result<Vec<u8>, CliError> {
    Ok(laterite_transport::pack_bytes(data, level)?)
}

/// zstd-decompress bytes → bytes in memory (the filesystem-free form of [`unpack`]).
pub fn unpack_bytes(data: &[u8]) -> Result<Vec<u8>, CliError> {
    Ok(laterite_transport::unpack_bytes(data)?)
}

/// zstd-compress + age-encrypt bytes → bytes in memory (the filesystem-free
/// form of [`lock`] — no plaintext ever hits disk).
pub fn lock_bytes(data: &[u8], password: &str, level: i32, log_n: u8) -> Result<Vec<u8>, CliError> {
    Ok(laterite_transport::lock_bytes(
        data, password, level, log_n,
    )?)
}

/// age-decrypt + zstd-decompress bytes → bytes in memory (the filesystem-free
/// form of [`unlock`]).
pub fn unlock_bytes(data: &[u8], password: &str) -> Result<Vec<u8>, CliError> {
    Ok(laterite_transport::unlock_bytes(data, password)?)
}
