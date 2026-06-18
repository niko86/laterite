//! Transport — zstd compression + age passphrase encryption, the Node port of
//! laterite-py's `transport_fns`. Reimplemented directly on `zstd` + `age`
//! (decoupled from `.ags5db` / laterite-ags4-core) as general file-envelope helpers:
//! compress/encrypt ANY file. The age envelope is interoperable with the Python
//! side (`pyrage`) — same `age` crate, same on-disk format. napi camelCases:
//! `transport_pack` → `transportPack`, `elapsed_s` → `elapsedS`.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::Instant;

use age::secrecy::SecretString;
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

fn read_existing(src: &str) -> Result<Vec<u8>> {
    let path = Path::new(src);
    if !path.exists() {
        return Err(Error::from_reason(format!("file not found: {src}")));
    }
    fs::read(path).map_err(|e| Error::from_reason(format!("read {src}: {e}")))
}

fn write_out(dest: &str, bytes: &[u8]) -> Result<()> {
    fs::write(dest, bytes).map_err(|e| Error::from_reason(format!("write {dest}: {e}")))
}

/// zstd-compress `src` → `dest`. `level` is 1–22 (default 9, the AGS sweet spot).
#[napi]
pub fn transport_pack(src: String, dest: String, level: Option<i32>) -> Result<PackStats> {
    let t0 = Instant::now();
    let input = read_existing(&src)?;
    let src_size = input.len() as u64;
    let compressed = zstd::encode_all(Cursor::new(&input), level.unwrap_or(9))
        .map_err(|e| Error::from_reason(format!("zstd encode: {e}")))?;
    write_out(&dest, &compressed)?;
    let out_size = compressed.len() as u64;
    Ok(PackStats {
        bytes: BigInt::from(out_size),
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// zstd-decompress `src` → `dest`.
#[napi]
pub fn transport_unpack(src: String, dest: String) -> Result<UnpackStats> {
    let t0 = Instant::now();
    let compressed = read_existing(&src)?;
    let out = zstd::decode_all(Cursor::new(&compressed))
        .map_err(|e| Error::from_reason(format!("zstd decode: {e}")))?;
    write_out(&dest, &out)?;
    Ok(UnpackStats {
        bytes: BigInt::from(out.len() as u64),
        elapsed_s: t0.elapsed().as_secs_f64(),
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
    let t0 = Instant::now();
    let input = read_existing(&src)?;
    let src_size = input.len() as u64;
    let compressed = zstd::encode_all(Cursor::new(&input), level.unwrap_or(9))
        .map_err(|e| Error::from_reason(format!("zstd encode: {e}")))?;
    let encrypted = encrypt_with_passphrase(&compressed, &password)?;
    write_out(&dest, &encrypted)?;
    let out_size = encrypted.len() as u64;
    Ok(PackStats {
        bytes: BigInt::from(out_size),
        ratio: out_size as f64 / src_size.max(1) as f64,
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

/// age-decrypt with `password`, then zstd-decompress → `dest`. Wrong passphrase
/// / non-passphrase envelopes surface as an error.
#[napi]
pub fn transport_unlock(src: String, dest: String, password: String) -> Result<UnpackStats> {
    let t0 = Instant::now();
    let encrypted = read_existing(&src)?;
    let decrypted = decrypt_with_passphrase(&encrypted, &password)?;
    let out = zstd::decode_all(Cursor::new(&decrypted))
        .map_err(|e| Error::from_reason(format!("zstd decode: {e}")))?;
    write_out(&dest, &out)?;
    Ok(UnpackStats {
        bytes: BigInt::from(out.len() as u64),
        elapsed_s: t0.elapsed().as_secs_f64(),
    })
}

fn encrypt_with_passphrase(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::new(passphrase.to_owned());
    let encryptor = age::Encryptor::with_user_passphrase(secret);
    let mut out: Vec<u8> = Vec::with_capacity(plaintext.len() + 256);
    let mut writer = encryptor
        .wrap_output(&mut out)
        .map_err(|e| Error::from_reason(format!("age wrap: {e}")))?;
    writer
        .write_all(plaintext)
        .map_err(|e| Error::from_reason(format!("age write: {e}")))?;
    writer
        .finish()
        .map_err(|e| Error::from_reason(format!("age finish: {e}")))?;
    Ok(out)
}

fn decrypt_with_passphrase(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::new(passphrase.to_owned());
    let decryptor = match age::Decryptor::new(ciphertext)
        .map_err(|e| Error::from_reason(format!("age open: {e}")))?
    {
        age::Decryptor::Passphrase(d) => d,
        age::Decryptor::Recipients(_) => {
            return Err(Error::from_reason(
                "file is encrypted to a key recipient, not a passphrase",
            ));
        }
    };
    let mut reader = decryptor
        .decrypt(&secret, None)
        .map_err(|e| Error::from_reason(format!("age decrypt (wrong password?): {e}")))?;
    let mut out: Vec<u8> = Vec::with_capacity(ciphertext.len());
    reader
        .read_to_end(&mut out)
        .map_err(|e| Error::from_reason(format!("age read: {e}")))?;
    Ok(out)
}

// In a `cargo test` build the napi registration glue that references these
// (private-module) `#[napi]` fns isn't emitted, so dead_code flags them + the
// stats structs + the io/age helpers. A real round-trip exercises all of them
// and pins the envelope behaviour (compress↔decompress, encrypt↔decrypt,
// wrong-password rejection).
#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    fn tmp(tag: &str) -> String {
        // Unique per process + tag so parallel tests / reruns don't collide.
        temp_dir()
            .join(format!("lat_node_transport_{}_{tag}", std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn pack_then_unpack_round_trips() {
        let (src, packed, out) = (tmp("p.src"), tmp("p.zst"), tmp("p.out"));
        let payload = b"\"GROUP\",\"PROJ\"\r\nrepetitive AGS-ish content ".repeat(80);
        fs::write(&src, &payload).unwrap();

        let stats = transport_pack(src.clone(), packed.clone(), None).unwrap();
        assert!(stats.ratio > 0.0 && stats.elapsed_s >= 0.0);
        let _ = &stats.bytes; // touch the BigInt field

        let u = transport_unpack(packed.clone(), out.clone()).unwrap();
        assert!(u.elapsed_s >= 0.0);
        let _ = &u.bytes;
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

        let stats =
            transport_lock(src.clone(), locked.clone(), "hunter2".to_string(), None).unwrap();
        let _ = (&stats.bytes, stats.ratio, stats.elapsed_s);

        // wrong passphrase must fail
        assert!(transport_unlock(locked.clone(), out.clone(), "wrong".to_string()).is_err());
        // correct passphrase round-trips
        transport_unlock(locked.clone(), out.clone(), "hunter2".to_string()).unwrap();
        assert_eq!(fs::read(&out).unwrap(), payload);

        for p in [src, locked, out] {
            let _ = fs::remove_file(p);
        }
    }

    #[test]
    fn missing_source_errors() {
        assert!(transport_pack(tmp("absent.src"), tmp("x.zst"), None).is_err());
    }
}
