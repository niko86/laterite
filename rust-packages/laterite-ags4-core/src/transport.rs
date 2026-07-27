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

#[cfg(test)]
mod tests {
    use super::*;

    // A payload with structure zstd can actually shrink — a canned `vec![]` /
    // `vec![0]` stand-in for any of these wrappers survives a "does it return
    // *something*" test, so every assertion below checks the FULL round-trip
    // recovers the exact plaintext (which no fixed byte string can fake).
    const PAYLOAD: &[u8] = b"\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\"\n\"DATA\",\"BH01\"\n";

    #[test]
    fn pack_unpack_bytes_round_trip_recovers_the_plaintext() {
        let packed = pack_bytes(PAYLOAD, 9).unwrap();
        assert_ne!(packed, PAYLOAD, "packing changed the bytes");
        assert_eq!(unpack_bytes(&packed).unwrap(), PAYLOAD);
    }

    #[test]
    fn lock_unlock_bytes_round_trip_recovers_the_plaintext() {
        // log_n kept low for test speed; unlock pins max_work_factor high enough
        // to open anything lock produces, so any factor round-trips.
        let sealed = lock_bytes(PAYLOAD, "hunter2", 9, 10).unwrap();
        assert_ne!(sealed, PAYLOAD, "sealing changed the bytes");
        assert_eq!(unlock_bytes(&sealed, "hunter2").unwrap(), PAYLOAD);
    }

    #[test]
    fn pack_unpack_files_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("in.ags");
        let packed = dir.path().join("in.ags.zst");
        let out = dir.path().join("out.ags");
        std::fs::write(&src, PAYLOAD).unwrap();

        pack(&src, &packed, 9).unwrap();
        unpack(&packed, &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), PAYLOAD);
    }

    #[test]
    fn a_missing_source_maps_to_the_file_not_found_variant() {
        // The `From<TransportError>` face must keep FileNotFound's dedicated
        // variant (exit code 3) rather than folding it into Schema (6).
        let err = unpack(Path::new("/no/such/file.zst"), Path::new("/tmp/x")).unwrap_err();
        assert!(matches!(err, CliError::FileNotFound(_)), "got {err:?}");
        assert_eq!(err.exit_code(), 3);
    }
}
