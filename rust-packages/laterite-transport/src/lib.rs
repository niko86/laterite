//! Transport helpers — zstd compression + age passphrase encryption.
//!
//! The pack/unpack/lock/unlock envelope, extracted (#327) from
//! `laterite-ags4-core::transport` so the age + zstd logic lives in ONE crate
//! instead of the two byte-identical copies (core + laterite-node) the age
//! 0.10→0.11 migration had to touch. Consumers map [`TransportError`] to their
//! own error type: core provides `From<TransportError> for CliError` and
//! re-exports these behind its `transport` feature; laterite-node maps to
//! `napi::Error`.
//!
//! The operations are **content-agnostic** — zstd/age over raw file bytes, so
//! they work on any file (`.ags`, `.ags5db`, anything). The `age` envelope is
//! interoperable with the python-side `pyrage` library — same on-disk format,
//! both link the same Rust `age` crate under the hood.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::Instant;

use age::secrecy::SecretString;

/// Every way a transport operation can fail. Deliberately message-carrying
/// (not a rich typed tree): the consumers render these as human-facing strings
/// (`CliError::Schema` / `napi::Error::from_reason`), so the boundary the value
/// crosses is text either way.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("read: {0}")]
    Read(String),
    #[error("write: {0}")]
    Write(String),
    #[error("zstd {op}: {detail}")]
    Zstd { op: &'static str, detail: String },
    #[error("age {op}: {detail}")]
    Age { op: &'static str, detail: String },
    /// A key-recipient (non-passphrase) envelope handed to a passphrase unlock.
    #[error("file is encrypted to a key recipient, not a passphrase")]
    NotPassphrase,
}

/// Result of [`pack`] / [`lock`]: output file size, ratio vs source,
/// elapsed seconds.
#[derive(Debug, Clone)]
pub struct PackStats {
    pub bytes: u64,
    pub ratio: f64,
    pub elapsed_s: f64,
}

/// Result of [`unpack`] / [`unlock`]: output file size, elapsed seconds.
#[derive(Debug, Clone)]
pub struct UnpackStats {
    pub bytes: u64,
    pub elapsed_s: f64,
}

fn read_existing(src: &Path) -> Result<Vec<u8>, TransportError> {
    if !src.exists() {
        return Err(TransportError::FileNotFound(src.display().to_string()));
    }
    fs::read(src).map_err(|e| TransportError::Read(e.to_string()))
}

fn write_out(dest: &Path, bytes: &[u8]) -> Result<(), TransportError> {
    fs::write(dest, bytes).map_err(|e| TransportError::Write(e.to_string()))
}

/// zstd-compress `src` → `dest`. `level` is 1-22 (9 is the empirical
/// sweet spot on AGS data).
pub fn pack(src: &Path, dest: &Path, level: i32) -> Result<PackStats, TransportError> {
    let t0 = Instant::now();
    let src_bytes = read_existing(src)?;
    let src_size = src_bytes.len() as u64;
    let compressed =
        zstd::encode_all(Cursor::new(&src_bytes), level).map_err(|e| TransportError::Zstd {
            op: "encode",
            detail: e.to_string(),
        })?;
    write_out(dest, &compressed)?;
    let out_size = compressed.len() as u64;
    Ok(PackStats {
        bytes: out_size,
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// zstd-decompress `src` → `dest`.
pub fn unpack(src: &Path, dest: &Path) -> Result<UnpackStats, TransportError> {
    let t0 = Instant::now();
    let compressed = read_existing(src)?;
    let decompressed =
        zstd::decode_all(Cursor::new(&compressed)).map_err(|e| TransportError::Zstd {
            op: "decode",
            detail: e.to_string(),
        })?;
    write_out(dest, &decompressed)?;
    Ok(UnpackStats {
        bytes: decompressed.len() as u64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// zstd-compress, then age-encrypt with `password`. Output goes to `dest`
/// (suffix `.zst.age` conventional). The compress-then-encrypt order is
/// load-bearing: zstd needs low-entropy input; encrypted bytes are random.
pub fn lock(
    src: &Path,
    dest: &Path,
    password: &str,
    level: i32,
) -> Result<PackStats, TransportError> {
    let t0 = Instant::now();
    let src_bytes = read_existing(src)?;
    let src_size = src_bytes.len() as u64;
    let compressed =
        zstd::encode_all(Cursor::new(&src_bytes), level).map_err(|e| TransportError::Zstd {
            op: "encode",
            detail: e.to_string(),
        })?;
    let encrypted = encrypt_with_passphrase(&compressed, password)?;
    write_out(dest, &encrypted)?;
    let out_size = encrypted.len() as u64;
    Ok(PackStats {
        bytes: out_size,
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// age-decrypt with `password`, then zstd-decompress.
pub fn unlock(src: &Path, dest: &Path, password: &str) -> Result<UnpackStats, TransportError> {
    let t0 = Instant::now();
    let encrypted = read_existing(src)?;
    let decrypted = decrypt_with_passphrase(&encrypted, password)?;
    let decompressed =
        zstd::decode_all(Cursor::new(&decrypted)).map_err(|e| TransportError::Zstd {
            op: "decode",
            detail: e.to_string(),
        })?;
    write_out(dest, &decompressed)?;
    Ok(UnpackStats {
        bytes: decompressed.len() as u64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// The scrypt work factor (`log2(N)`) laterite pins for passphrase locking.
///
/// `Encryptor::with_user_passphrase` (the convenience we used to call)
/// calibrates the factor to ~1 s on the *encrypting* machine — `log_N` reaches
/// 20+ on fast hardware. That's at/above the ceiling conservative age *decoders*
/// accept: the browser `age-encryption` library (the #295 transport surface)
/// refuses `log_N > 20`, and even at 20 its `@noble/hashes` scrypt exceeds a
/// ~1 GiB `maxmem` cap. Pinning **18** — age's standard tier and
/// `age-encryption`'s own default — makes a laterite-locked file openable
/// everywhere (CLI, `pyrage`, browser) AND makes the work factor deterministic
/// instead of machine-dependent. ~256 MiB / ~0.1–0.5 s to derive: a strong
/// passphrase KDF. (Decryption accepts any factor the file declares, unchanged.)
const SCRYPT_LOG_N: u8 = 18;

/// Encrypt `plaintext` with `passphrase` via age 0.11's scrypt passphrase
/// recipient, pinned to [`SCRYPT_LOG_N`] (not age's machine-calibrated default).
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<Vec<u8>, TransportError> {
    let secret = SecretString::new(passphrase.to_owned().into_boxed_str());
    let mut recipient = age::scrypt::Recipient::new(secret);
    recipient.set_work_factor(SCRYPT_LOG_N);
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .map_err(|e| TransportError::Age {
                op: "recipients",
                detail: e.to_string(),
            })?;
    let mut out: Vec<u8> = Vec::with_capacity(plaintext.len() + 256);
    let mut writer = encryptor
        .wrap_output(&mut out)
        .map_err(|e| TransportError::Age {
            op: "wrap",
            detail: e.to_string(),
        })?;
    writer
        .write_all(plaintext)
        .map_err(|e| TransportError::Age {
            op: "write",
            detail: e.to_string(),
        })?;
    writer.finish().map_err(|e| TransportError::Age {
        op: "finish",
        detail: e.to_string(),
    })?;
    Ok(out)
}

/// Decrypt `ciphertext` with `passphrase` via age 0.11's `Decryptor::new`
/// (a unified struct now — the 0.10 `Passphrase`/`Recipients` enum is gone),
/// guarded by `is_scrypt()` so a key-recipient envelope gets the clear
/// [`TransportError::NotPassphrase`] rejection, then decrypted with a
/// `scrypt::Identity`. A wrong passphrase surfaces as [`TransportError::Age`].
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &str,
) -> Result<Vec<u8>, TransportError> {
    let secret = SecretString::new(passphrase.to_owned().into_boxed_str());
    let decryptor = age::Decryptor::new(ciphertext).map_err(|e| TransportError::Age {
        op: "open",
        detail: e.to_string(),
    })?;
    if !decryptor.is_scrypt() {
        return Err(TransportError::NotPassphrase);
    }
    let identity = age::scrypt::Identity::new(secret);
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| TransportError::Age {
            op: "decrypt (wrong password?)",
            detail: e.to_string(),
        })?;
    let mut out: Vec<u8> = Vec::with_capacity(ciphertext.len());
    reader
        .read_to_end(&mut out)
        .map_err(|e| TransportError::Age {
            op: "read",
            detail: e.to_string(),
        })?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn tmp(tag: &str) -> std::path::PathBuf {
        // Unique per process + tag so parallel tests / reruns don't collide.
        temp_dir().join(format!("lat_transport_{}_{tag}", std::process::id()))
    }

    #[test]
    fn pack_then_unpack_round_trips() {
        let (src, packed, out) = (tmp("p.src"), tmp("p.zst"), tmp("p.out"));
        let payload = b"\"GROUP\",\"PROJ\"\r\nrepetitive AGS-ish content ".repeat(80);
        fs::write(&src, &payload).unwrap();

        let stats = pack(&src, &packed, 9).unwrap();
        assert!(stats.bytes > 0 && stats.ratio > 0.0 && stats.elapsed_s >= 0.0);
        // repetitive input must actually shrink
        assert!(stats.bytes < payload.len() as u64);

        let u = unpack(&packed, &out).unwrap();
        assert_eq!(u.bytes, payload.len() as u64);
        assert_eq!(fs::read(&out).unwrap(), payload);

        for p in [src, packed, out] {
            let _ = fs::remove_file(p);
        }
    }

    #[test]
    fn lock_then_unlock_round_trips_and_rejects_wrong_password() {
        let (src, locked, out) = (tmp("l.src"), tmp("l.age"), tmp("l.out"));
        let payload = b"sensitive AGS payload ".repeat(60);
        fs::write(&src, &payload).unwrap();

        lock(&src, &locked, "hunter2", 9).unwrap();

        // wrong passphrase must fail
        assert!(unlock(&locked, &out, "wrong").is_err());
        // correct passphrase round-trips byte-for-byte
        unlock(&locked, &out, "hunter2").unwrap();
        assert_eq!(fs::read(&out).unwrap(), payload);

        for p in [src, locked, out] {
            let _ = fs::remove_file(p);
        }
    }

    #[test]
    fn missing_source_errors_with_file_not_found() {
        let err = pack(&tmp("absent.src"), &tmp("x.zst"), 9).unwrap_err();
        assert!(matches!(err, TransportError::FileNotFound(_)));
    }

    #[test]
    fn plaintext_bytes_round_trip_through_the_age_envelope() {
        // The byte-level path the consumers reuse directly.
        let msg = b"the quick brown fox jumps over 13 lazy dogs";
        let sealed = encrypt_with_passphrase(msg, "pw").unwrap();
        assert_ne!(sealed, msg, "ciphertext must differ from plaintext");
        assert_eq!(decrypt_with_passphrase(&sealed, "pw").unwrap(), msg);
        assert!(decrypt_with_passphrase(&sealed, "nope").is_err());
    }

    #[test]
    fn lock_pins_scrypt_work_factor() {
        // The age header is ASCII: `-> scrypt <salt> <log_N>`. We pin the factor
        // (not age's machine-calibrated default, which reaches 20+ on fast
        // hardware) so lock output stays openable by conservative age decoders —
        // the browser `age-encryption` lib refuses log_N > 20. Parse the emitted
        // stanza and assert it's SCRYPT_LOG_N, so a fast CI box can't silently
        // calibrate above the interop ceiling.
        let sealed = encrypt_with_passphrase(b"x", "pw").unwrap();
        let header = String::from_utf8_lossy(&sealed[..sealed.len().min(200)]);
        let stanza = header
            .lines()
            .find(|l| l.starts_with("-> scrypt "))
            .expect("age header carries a scrypt stanza");
        let log_n: u8 = stanza.rsplit(' ').next().unwrap().parse().unwrap();
        assert_eq!(log_n, SCRYPT_LOG_N, "lock must pin the scrypt work factor");
    }
}
