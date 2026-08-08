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
//! they work on any file (`.ags`, anything). The `age` envelope is
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

/// zstd-compress `data` in memory. `level` is 1-22 (9 is the empirical sweet
/// spot on AGS data). The filesystem-free core of [`pack`] — same output bytes
/// for the same input, so a `pack_bytes` blob opens with `unpack` / `unpack_bytes`
/// / stock `zstd` interchangeably.
pub fn pack_bytes(data: &[u8], level: i32) -> Result<Vec<u8>, TransportError> {
    zstd::encode_all(Cursor::new(data), level).map_err(|e| TransportError::Zstd {
        op: "encode",
        detail: e.to_string(),
    })
}

/// zstd-decompress `data` in memory — the filesystem-free core of [`unpack`].
pub fn unpack_bytes(data: &[u8]) -> Result<Vec<u8>, TransportError> {
    zstd::decode_all(Cursor::new(data)).map_err(|e| TransportError::Zstd {
        op: "decode",
        detail: e.to_string(),
    })
}

/// zstd-compress `src` → `dest`. `level` is 1-22 (9 is the empirical
/// sweet spot on AGS data).
pub fn pack(src: &Path, dest: &Path, level: i32) -> Result<PackStats, TransportError> {
    let t0 = Instant::now();
    let src_bytes = read_existing(src)?;
    let src_size = src_bytes.len() as u64;
    let compressed = pack_bytes(&src_bytes, level)?;
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
    let decompressed = unpack_bytes(&compressed)?;
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
    log_n: u8,
) -> Result<PackStats, TransportError> {
    let t0 = Instant::now();
    let src_bytes = read_existing(src)?;
    let src_size = src_bytes.len() as u64;
    let encrypted = lock_bytes(&src_bytes, password, level, log_n)?;
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
    let decompressed = unlock_bytes(&encrypted, password)?;
    write_out(dest, &decompressed)?;
    Ok(UnpackStats {
        bytes: decompressed.len() as u64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// zstd-compress then age-encrypt `data` in memory — the filesystem-free core
/// of [`lock`]. The compress-then-encrypt order is load-bearing (zstd needs
/// low-entropy input; encrypted bytes are random). Same `.zst.age` envelope a
/// `lock` file carries, so a `lock_bytes` blob opens with `unlock` /
/// `unlock_bytes` / `pyrage` / the browser, given the passphrase — never
/// touching a plaintext file on disk.
pub fn lock_bytes(
    data: &[u8],
    password: &str,
    level: i32,
    log_n: u8,
) -> Result<Vec<u8>, TransportError> {
    let compressed = pack_bytes(data, level)?;
    encrypt_with_passphrase(&compressed, password, log_n)
}

/// age-decrypt then zstd-decompress `data` in memory — the filesystem-free
/// core of [`unlock`].
pub fn unlock_bytes(data: &[u8], password: &str) -> Result<Vec<u8>, TransportError> {
    let decrypted = decrypt_with_passphrase(data, password)?;
    unpack_bytes(&decrypted)
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
/// The default work factor when a caller doesn't override `log_n`; a lower value
/// trades KDF strength for speed (tests use it to avoid a slow scrypt per case).
pub const SCRYPT_LOG_N: u8 = 18;

/// Encrypt `plaintext` with `passphrase` via age 0.11's scrypt passphrase
/// recipient at work factor `log_n` (`log2(N)`). Pass [`SCRYPT_LOG_N`] for the
/// default; a lower value is cheaper. Rejects `log_n` outside `1..=20` — `0` is
/// invalid and `> 20` yields a file the browser `age-encryption` decoder refuses
/// (breaking the "openable everywhere" guarantee [`SCRYPT_LOG_N`] documents).
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
    log_n: u8,
) -> Result<Vec<u8>, TransportError> {
    if !(1..=20).contains(&log_n) {
        return Err(TransportError::Age {
            op: "work_factor",
            detail: format!("scrypt log_n must be 1..=20, got {log_n}"),
        });
    }
    let secret = SecretString::new(passphrase.to_owned().into_boxed_str());
    let mut recipient = age::scrypt::Recipient::new(secret);
    recipient.set_work_factor(log_n);
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
    let mut identity = age::scrypt::Identity::new(secret);
    // #432: age's default `max_work_factor` is `target_work_factor + 4`, a machine-
    // SPEED heuristic — on a slow / memory-starved machine (e.g. a CI container) it
    // drops below our `SCRYPT_LOG_N` (18) and refuses our OWN files with "Excessive
    // work parameter". Pin it to the encrypt cap (20) so decrypt accepts anything we
    // can produce, machine-independently — the "accepts any factor the file declares"
    // contract this module documents.
    identity.set_max_work_factor(20);
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

        lock(&src, &locked, "hunter2", 9, SCRYPT_LOG_N).unwrap();

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
        let sealed = encrypt_with_passphrase(msg, "pw", SCRYPT_LOG_N).unwrap();
        assert_ne!(sealed, msg, "ciphertext must differ from plaintext");
        assert_eq!(decrypt_with_passphrase(&sealed, "pw").unwrap(), msg);
        assert!(decrypt_with_passphrase(&sealed, "nope").is_err());
    }

    #[test]
    fn decrypt_pins_max_work_factor_so_our_files_open_on_any_machine() {
        // #432: age's default decrypt cap is a machine-SPEED heuristic; on a slow /
        // memory-starved container it drops below our SCRYPT_LOG_N (18) and refuses
        // our OWN files. Prove the mechanism machine-independently: a cap BELOW the
        // file's factor rejects (the bug), while `decrypt_with_passphrase` — which
        // pins the cap at the encrypt max (20) — accepts.
        let sealed = encrypt_with_passphrase(b"secret", "pw", SCRYPT_LOG_N).unwrap();

        // Simulate a starved machine: an identity capped below the file's factor
        // refuses it, exactly as the CI runner did.
        let decryptor = age::Decryptor::new(&sealed[..]).unwrap();
        let mut capped =
            age::scrypt::Identity::new(SecretString::new("pw".to_owned().into_boxed_str()));
        capped.set_max_work_factor(SCRYPT_LOG_N - 1);
        assert!(
            decryptor
                .decrypt(std::iter::once(&capped as &dyn age::Identity))
                .is_err(),
            "a cap below the file's work factor must reject — this is the #432 failure"
        );

        // Our decrypt pins the cap at the encrypt max, so it opens regardless.
        assert_eq!(decrypt_with_passphrase(&sealed, "pw").unwrap(), b"secret");
    }

    #[test]
    fn pack_bytes_unpack_bytes_round_trip() {
        let payload = b"\"GROUP\",\"PROJ\"\r\nrepetitive AGS-ish content ".repeat(80);
        let packed = pack_bytes(&payload, 9).unwrap();
        assert!(packed.len() < payload.len(), "repetitive input must shrink");
        assert_eq!(unpack_bytes(&packed).unwrap(), payload);
    }

    #[test]
    fn lock_bytes_unlock_bytes_round_trip_and_rejects_wrong_password() {
        let payload = b"sensitive AGS payload ".repeat(60);
        let sealed = lock_bytes(&payload, "hunter2", 9, SCRYPT_LOG_N).unwrap();
        assert_ne!(sealed, payload, "sealed bytes differ from plaintext");
        assert!(unlock_bytes(&sealed, "wrong").is_err());
        assert_eq!(unlock_bytes(&sealed, "hunter2").unwrap(), payload);
    }

    #[test]
    fn bytes_and_file_apis_are_interoperable() {
        // The parity guarantee: a `lock_bytes` blob opens with the file `unlock`,
        // and a file `lock` opens with `unlock_bytes` — same envelope either way.
        let payload = b"\"GROUP\",\"LOCA\"\r\nrepetitive ".repeat(50);

        let sealed = lock_bytes(&payload, "pw", 9, SCRYPT_LOG_N).unwrap();
        let (locked, out) = (tmp("i.age"), tmp("i.out"));
        fs::write(&locked, &sealed).unwrap();
        unlock(&locked, &out, "pw").unwrap();
        assert_eq!(fs::read(&out).unwrap(), payload, "lock_bytes → file unlock");

        let (src, locked2) = (tmp("i.src"), tmp("i2.age"));
        fs::write(&src, &payload).unwrap();
        lock(&src, &locked2, "pw", 9, SCRYPT_LOG_N).unwrap();
        let opened = unlock_bytes(&fs::read(&locked2).unwrap(), "pw").unwrap();
        assert_eq!(opened, payload, "file lock → unlock_bytes");

        for p in [locked, out, src, locked2] {
            let _ = fs::remove_file(p);
        }
    }

    #[test]
    fn lock_pins_scrypt_work_factor() {
        // The age header is ASCII: `-> scrypt <salt> <log_N>`. We pin the factor
        // (not age's machine-calibrated default, which reaches 20+ on fast
        // hardware) so lock output stays openable by conservative age decoders —
        // the browser `age-encryption` lib refuses log_N > 20. Parse the emitted
        // stanza and assert it's SCRYPT_LOG_N, so a fast CI box can't silently
        // calibrate above the interop ceiling.
        let sealed = encrypt_with_passphrase(b"x", "pw", SCRYPT_LOG_N).unwrap();
        let header = String::from_utf8_lossy(&sealed[..sealed.len().min(200)]);
        let stanza = header
            .lines()
            .find(|l| l.starts_with("-> scrypt "))
            .expect("age header carries a scrypt stanza");
        let log_n: u8 = stanza.rsplit(' ').next().unwrap().parse().unwrap();
        assert_eq!(log_n, SCRYPT_LOG_N, "lock must pin the scrypt work factor");
    }
}

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
