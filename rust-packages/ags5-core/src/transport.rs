//! Transport helpers — zstd compression + age passphrase encryption.
//!
//! Stage F2a-2d: extracted from `commands/{pack,unpack,lock,unlock}.rs`
//! into a CLI-dep-free lib API so `laterite-py` can expose them. The
//! commands still own the spinner / dry-run / output rendering; the
//! pure data work lives here.
//!
//! The `age` envelope is interoperable with the python-side `pyrage`
//! library — same on-disk format, both link the same Rust `age` crate
//! under the hood.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::Instant;

use age::secrecy::SecretString;

use crate::error::CliError;

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

/// zstd-compress `src` → `dest`. `level` is 1-22 (9 is the empirical
/// sweet spot on AGS data — see `commands/pack.rs` docstring).
pub fn pack(src: &Path, dest: &Path, level: i32) -> Result<PackStats, CliError> {
    if !src.exists() {
        return Err(CliError::FileNotFound(src.display().to_string()));
    }
    let t0 = Instant::now();
    let src_bytes = fs::read(src).map_err(|e| CliError::Schema(format!("read: {}", e)))?;
    let src_size = src_bytes.len() as u64;
    let compressed = zstd::encode_all(Cursor::new(&src_bytes), level)
        .map_err(|e| CliError::Schema(format!("zstd encode: {}", e)))?;
    fs::write(dest, &compressed).map_err(|e| CliError::Schema(format!("write: {}", e)))?;
    let elapsed_s = t0.elapsed().as_secs_f64();
    let out_size = compressed.len() as u64;
    Ok(PackStats {
        bytes: out_size,
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s,
    })
}

/// zstd-decompress `src` → `dest`.
pub fn unpack(src: &Path, dest: &Path) -> Result<UnpackStats, CliError> {
    if !src.exists() {
        return Err(CliError::FileNotFound(src.display().to_string()));
    }
    let t0 = Instant::now();
    let compressed = fs::read(src).map_err(|e| CliError::Schema(format!("read: {}", e)))?;
    let decompressed = zstd::decode_all(Cursor::new(&compressed))
        .map_err(|e| CliError::Schema(format!("zstd decode: {}", e)))?;
    fs::write(dest, &decompressed).map_err(|e| CliError::Schema(format!("write: {}", e)))?;
    let elapsed_s = t0.elapsed().as_secs_f64();
    Ok(UnpackStats {
        bytes: decompressed.len() as u64,
        elapsed_s,
    })
}

/// zstd-compress, then age-encrypt with `password`. Mirrors
/// `ags5db lock`. Output goes to `dest` (suffix `.zst.age`
/// conventional). The compress-then-encrypt order is load-bearing:
/// zstd needs low-entropy input; encrypted bytes are random.
pub fn lock(src: &Path, dest: &Path, password: &str, level: i32) -> Result<PackStats, CliError> {
    if !src.exists() {
        return Err(CliError::FileNotFound(src.display().to_string()));
    }
    let t0 = Instant::now();
    let src_bytes = fs::read(src).map_err(|e| CliError::Schema(format!("read: {}", e)))?;
    let src_size = src_bytes.len() as u64;
    let compressed = zstd::encode_all(Cursor::new(&src_bytes), level)
        .map_err(|e| CliError::Schema(format!("zstd encode: {}", e)))?;
    let encrypted = encrypt_with_passphrase(&compressed, password)?;
    fs::write(dest, &encrypted).map_err(|e| CliError::Schema(format!("write: {}", e)))?;
    let elapsed_s = t0.elapsed().as_secs_f64();
    let out_size = encrypted.len() as u64;
    Ok(PackStats {
        bytes: out_size,
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s,
    })
}

/// age-decrypt with `password`, then zstd-decompress. Mirrors
/// `ags5db unlock`.
pub fn unlock(src: &Path, dest: &Path, password: &str) -> Result<UnpackStats, CliError> {
    if !src.exists() {
        return Err(CliError::FileNotFound(src.display().to_string()));
    }
    let t0 = Instant::now();
    let encrypted = fs::read(src).map_err(|e| CliError::Schema(format!("read: {}", e)))?;
    let decrypted = decrypt_with_passphrase(&encrypted, password)?;
    let decompressed = zstd::decode_all(Cursor::new(&decrypted))
        .map_err(|e| CliError::Schema(format!("zstd decode: {}", e)))?;
    fs::write(dest, &decompressed).map_err(|e| CliError::Schema(format!("write: {}", e)))?;
    let elapsed_s = t0.elapsed().as_secs_f64();
    Ok(UnpackStats {
        bytes: decompressed.len() as u64,
        elapsed_s,
    })
}

/// Encrypt `plaintext` with `passphrase` via age 0.10's
/// `Encryptor::with_user_passphrase` (scrypt under the hood).
/// `pub` so the bin's `commands/lock.rs` can call it directly.
pub fn encrypt_with_passphrase(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, CliError> {
    let secret = SecretString::new(passphrase.to_owned());
    let encryptor = age::Encryptor::with_user_passphrase(secret);
    let mut out: Vec<u8> = Vec::with_capacity(plaintext.len() + 256);
    let mut writer = encryptor
        .wrap_output(&mut out)
        .map_err(|e| CliError::Schema(format!("age wrap: {}", e)))?;
    writer
        .write_all(plaintext)
        .map_err(|e| CliError::Schema(format!("age write: {}", e)))?;
    writer
        .finish()
        .map_err(|e| CliError::Schema(format!("age finish: {}", e)))?;
    Ok(out)
}

/// Decrypt `ciphertext` with `passphrase` via age 0.10's two-step
/// `Decryptor::new` → match on `Passphrase` arm → `.decrypt`.
/// Wrong passphrase / non-passphrase envelopes surface as
/// `CliError::Schema`.
pub fn decrypt_with_passphrase(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>, CliError> {
    let secret = SecretString::new(passphrase.to_owned());
    let decryptor = match age::Decryptor::new(ciphertext)
        .map_err(|e| CliError::Schema(format!("age open: {}", e)))?
    {
        age::Decryptor::Passphrase(d) => d,
        age::Decryptor::Recipients(_) => {
            return Err(CliError::Schema(
                "file is encrypted to a key recipient, not a passphrase; \
                 ags5db unlock only supports passphrase-locked files"
                    .into(),
            ));
        }
    };
    let mut reader = decryptor
        .decrypt(&secret, None)
        .map_err(|e| CliError::Schema(format!("age decrypt (wrong password?): {}", e)))?;
    let mut out: Vec<u8> = Vec::with_capacity(ciphertext.len());
    reader
        .read_to_end(&mut out)
        .map_err(|e| CliError::Schema(format!("age read: {}", e)))?;
    Ok(out)
}
