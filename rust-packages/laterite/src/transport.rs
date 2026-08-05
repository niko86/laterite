//! Compress and encrypt any file — zstd, and zstd + an age passphrase.
//!
//! ```no_run
//! use laterite::transport;
//!
//! // Compress. `.zst` is conventional; nothing enforces it.
//! transport::pack("delivery.ags", "delivery.ags.zst").run()?;
//! transport::unpack("delivery.ags.zst", "delivery.ags")?;
//!
//! // Compress and encrypt, in that order.
//! transport::lock("delivery.ags", "delivery.ags.zst.age", "hunter2").run()?;
//! transport::unlock("delivery.ags.zst.age", "delivery.ags", "hunter2")?;
//!
//! // Or never touch the disk at all.
//! let sealed = transport::lock_bytes(b"...".to_vec(), "hunter2").run()?;
//! let opened = transport::unlock_bytes(&sealed, "hunter2")?;
//! # Ok::<(), laterite::Error>(())
//! ```
//!
//! # Why this is at the crate root
//!
//! It is not AGS4. The envelope is zstd and age over arbitrary bytes, and the
//! only reason it ships in a geotechnical library is that AGS deliveries are
//! large, plain text, and frequently emailed. Putting it under [`crate::ags4`]
//! would say it understands the format, which it does not — it will compress a
//! JPEG. The rule the crate root keeps is that it stays format-neutral, and this
//! module is the first thing that genuinely belongs there.
//!
//! The Python surface makes the same split for the same reason: `laterite.ags4`
//! versus `laterite.transport`.
//!
//! # Interoperability
//!
//! One envelope across every surface. A file sealed here opens with `lat unlock`,
//! Python's `laterite.transport.unlock`, Node's `unlock`, the browser build, and
//! — for the `lock` forms — stock `age` and `pyrage` given the passphrase. The
//! `_bytes` forms produce byte-identical output to the file forms, so which one
//! sealed a blob is not something the opener has to know.
//!
//! `pack` output is plain zstd: stock `zstd -d` decompresses it.
//!
//! # Why compress-then-encrypt
//!
//! Only that order works. zstd needs low-entropy input to find redundancy, and
//! encrypted bytes are indistinguishable from random — encrypt-then-compress
//! reliably makes the file *bigger*. [`lock`] does it in the right order for you.

use std::path::{Path, PathBuf};

use laterite_ags4_core::error::CliError;
use laterite_ags4_core::transport as engine;

use crate::error::{Error, ErrorKind};

/// The zstd level used when none is given.
///
/// 9 is empirical on AGS data rather than the library default: roughly a 10%
/// ratio in a few seconds, where the levels above it buy single-digit percent
/// for minutes. The Python, Node and `lat` surfaces all default to the same
/// number, so a file sealed through any of them is the same size.
pub const DEFAULT_LEVEL: i32 = 9;

/// The scrypt work factor (`log2(N)`) used when none is given.
///
/// Pinned rather than calibrated. age's own convenience constructor tunes the
/// factor to about a second on the *encrypting* machine, which reaches 20+ on
/// fast hardware and then fails to open on conservative decoders that cap what
/// they will attempt. A fixed 18 is the value every laterite surface writes, so
/// "sealed on my laptop, opened in CI" holds.
pub const DEFAULT_WORK_FACTOR: u8 = engine::SCRYPT_LOG_N;

/// Map an engine failure onto the facade's coarse kind.
///
/// Only `FileNotFound` is classified. Everything else — a failed write, a
/// corrupt zstd frame, a wrong passphrase, an envelope sealed to a key
/// recipient rather than a passphrase — arrives here already flattened into
/// `CliError::Schema` by the engine's own transport face, with the detail in
/// the message.
///
/// Recovering a finer mapping would mean matching on that message, which is a
/// second table competing with the engine's, keyed on strings nobody froze.
/// [`ErrorKind::Other`] is documented for exactly this position and carries the
/// engine's own words, so a caller who needs the distinction still reads it.
fn map(err: &CliError, subject: &str) -> Error {
    let kind = match err {
        CliError::FileNotFound(_) => ErrorKind::Io,
        _ => ErrorKind::Other,
    };
    Error::with_source(kind, subject.to_string(), err)
}

/// What a [`pack`] or [`lock`] wrote.
pub struct Packed {
    bytes: u64,
    ratio: f64,
    elapsed_s: f64,
}

impl Packed {
    /// Size of the output file, in bytes.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Output size divided by input size. Smaller is better; `0.1` is a typical
    /// AGS delivery at the default level.
    ///
    /// Above 1.0 is possible and is not a bug — an already-compressed input has
    /// no redundancy left, so the envelope only adds framing.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Wall-clock seconds the operation took.
    #[must_use]
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_s
    }
}

/// What an [`unpack`] or [`unlock`] wrote.
pub struct Unpacked {
    bytes: u64,
    elapsed_s: f64,
}

impl Unpacked {
    /// Size of the recovered file, in bytes.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Wall-clock seconds the operation took.
    #[must_use]
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_s
    }
}

/// A pending [`pack`]. Configure it, then [`Pack::run`].
pub struct Pack {
    src: PathBuf,
    dest: PathBuf,
    level: i32,
}

/// zstd-compress a file.
///
/// The output is plain zstd — `zstd -d` opens it, and so does [`unpack`].
pub fn pack(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Pack {
    Pack {
        src: src.as_ref().to_path_buf(),
        dest: dest.as_ref().to_path_buf(),
        level: DEFAULT_LEVEL,
    }
}

impl Pack {
    /// Compress at this zstd level (1–22) instead of [`DEFAULT_LEVEL`].
    #[must_use]
    pub fn level(mut self, level: i32) -> Pack {
        self.level = level;
        self
    }

    /// Compress, writing to the destination.
    ///
    /// # Errors
    /// [`ErrorKind::Io`] if the source is missing; [`ErrorKind::Other`] if the
    /// write or the compression itself fails.
    pub fn run(self) -> Result<Packed, Error> {
        engine::pack(&self.src, &self.dest, self.level)
            .map(|s| Packed {
                bytes: s.bytes,
                ratio: s.ratio,
                elapsed_s: s.elapsed_s,
            })
            .map_err(|e| map(&e, &format!("cannot pack {}", self.src.display())))
    }
}

/// A pending [`pack_bytes`]. Configure it, then [`PackBytes::run`].
pub struct PackBytes {
    data: Vec<u8>,
    level: i32,
}

/// zstd-compress bytes already in memory, never touching a filesystem.
///
/// Byte-identical to what [`pack`] would write for the same input and level.
pub fn pack_bytes(data: impl Into<Vec<u8>>) -> PackBytes {
    PackBytes {
        data: data.into(),
        level: DEFAULT_LEVEL,
    }
}

impl PackBytes {
    /// Compress at this zstd level (1–22) instead of [`DEFAULT_LEVEL`].
    #[must_use]
    pub fn level(mut self, level: i32) -> PackBytes {
        self.level = level;
        self
    }

    /// Compress, returning the sealed bytes.
    ///
    /// # Errors
    /// [`ErrorKind::Other`] if compression fails.
    pub fn run(self) -> Result<Vec<u8>, Error> {
        engine::pack_bytes(&self.data, self.level)
            .map_err(|e| map(&e, &format!("cannot pack {} bytes", self.data.len())))
    }
}

/// zstd-decompress a file.
///
/// A plain function rather than a builder because there is nothing to
/// configure — the frame carries everything needed to open it.
///
/// # Errors
/// [`ErrorKind::Io`] if the source is missing; [`ErrorKind::Other`] if it is not
/// zstd, or the write fails.
pub fn unpack(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<Unpacked, Error> {
    let src = src.as_ref();
    engine::unpack(src, dest.as_ref())
        .map(|s| Unpacked {
            bytes: s.bytes,
            elapsed_s: s.elapsed_s,
        })
        .map_err(|e| map(&e, &format!("cannot unpack {}", src.display())))
}

/// zstd-decompress bytes already in memory.
///
/// # Errors
/// [`ErrorKind::Other`] if the bytes are not a zstd frame.
pub fn unpack_bytes(data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    let data = data.as_ref();
    engine::unpack_bytes(data).map_err(|e| map(&e, &format!("cannot unpack {} bytes", data.len())))
}

/// A pending [`lock`]. Configure it, then [`Lock::run`].
pub struct Lock {
    src: PathBuf,
    dest: PathBuf,
    password: String,
    level: i32,
    log_n: u8,
}

/// zstd-compress a file, then encrypt it with an age passphrase.
///
/// `.zst.age` is the conventional suffix. The result opens with [`unlock`], with
/// `lat unlock`, with the Python and Node surfaces, and with stock `age` given
/// the passphrase.
pub fn lock(src: impl AsRef<Path>, dest: impl AsRef<Path>, password: impl Into<String>) -> Lock {
    Lock {
        src: src.as_ref().to_path_buf(),
        dest: dest.as_ref().to_path_buf(),
        password: password.into(),
        level: DEFAULT_LEVEL,
        log_n: DEFAULT_WORK_FACTOR,
    }
}

impl Lock {
    /// Compress at this zstd level (1–22) instead of [`DEFAULT_LEVEL`].
    #[must_use]
    pub fn level(mut self, level: i32) -> Lock {
        self.level = level;
        self
    }

    /// Use this scrypt work factor instead of [`DEFAULT_WORK_FACTOR`].
    ///
    /// Raising it slows an attacker and slows the legitimate opener equally;
    /// past about 20, conservative age decoders refuse the file outright rather
    /// than spend the memory. Leave it alone unless you control both ends.
    #[must_use]
    pub fn work_factor(mut self, log_n: u8) -> Lock {
        self.log_n = log_n;
        self
    }

    /// Compress, encrypt, and write to the destination.
    ///
    /// # Errors
    /// [`ErrorKind::Io`] if the source is missing; [`ErrorKind::Other`] if the
    /// write, the compression or the encryption fails.
    pub fn run(self) -> Result<Packed, Error> {
        engine::lock(
            &self.src,
            &self.dest,
            &self.password,
            self.level,
            self.log_n,
        )
        .map(|s| Packed {
            bytes: s.bytes,
            ratio: s.ratio,
            elapsed_s: s.elapsed_s,
        })
        .map_err(|e| map(&e, &format!("cannot lock {}", self.src.display())))
    }
}

/// A pending [`lock_bytes`]. Configure it, then [`LockBytes::run`].
pub struct LockBytes {
    data: Vec<u8>,
    password: String,
    level: i32,
    log_n: u8,
}

/// zstd-compress and encrypt bytes in memory — the plaintext never reaches a
/// disk.
///
/// This is the form that exists because the file form is not always allowed: a
/// service that receives an upload and must hand back a sealed blob has nowhere
/// to put a plaintext temporary, and creating one is the thing being avoided.
pub fn lock_bytes(data: impl Into<Vec<u8>>, password: impl Into<String>) -> LockBytes {
    LockBytes {
        data: data.into(),
        password: password.into(),
        level: DEFAULT_LEVEL,
        log_n: DEFAULT_WORK_FACTOR,
    }
}

impl LockBytes {
    /// Compress at this zstd level (1–22) instead of [`DEFAULT_LEVEL`].
    #[must_use]
    pub fn level(mut self, level: i32) -> LockBytes {
        self.level = level;
        self
    }

    /// Use this scrypt work factor instead of [`DEFAULT_WORK_FACTOR`].
    #[must_use]
    pub fn work_factor(mut self, log_n: u8) -> LockBytes {
        self.log_n = log_n;
        self
    }

    /// Compress, encrypt, and return the sealed bytes.
    ///
    /// # Errors
    /// [`ErrorKind::Other`] if compression or encryption fails.
    pub fn run(self) -> Result<Vec<u8>, Error> {
        engine::lock_bytes(&self.data, &self.password, self.level, self.log_n)
            .map_err(|e| map(&e, &format!("cannot lock {} bytes", self.data.len())))
    }
}

/// Decrypt an age passphrase envelope, then zstd-decompress it.
///
/// # Errors
/// [`ErrorKind::Io`] if the source is missing; [`ErrorKind::Other`] if the
/// passphrase is wrong, the envelope is sealed to a key recipient rather than a
/// passphrase, or the write fails.
pub fn unlock(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    password: impl AsRef<str>,
) -> Result<Unpacked, Error> {
    let src = src.as_ref();
    engine::unlock(src, dest.as_ref(), password.as_ref())
        .map(|s| Unpacked {
            bytes: s.bytes,
            elapsed_s: s.elapsed_s,
        })
        .map_err(|e| map(&e, &format!("cannot unlock {}", src.display())))
}

/// Decrypt and decompress bytes in memory.
///
/// # Errors
/// [`ErrorKind::Other`] if the passphrase is wrong, the envelope is not a
/// passphrase envelope, or the payload is not zstd.
pub fn unlock_bytes(data: impl AsRef<[u8]>, password: impl AsRef<str>) -> Result<Vec<u8>, Error> {
    let data = data.as_ref();
    engine::unlock_bytes(data, password.as_ref())
        .map_err(|e| map(&e, &format!("cannot unlock {} bytes", data.len())))
}

// Debug is hand-written on the two password-carrying builders, and the reason is
// not style. `#[derive(Debug)]` prints every field, so a `dbg!(builder)` or a
// `tracing::debug!(?builder)` — or any struct that derives Debug and happens to
// contain one of these — would put the passphrase in a log file. Redacting is
// the only safe default, and it has to be unconditional: a caller cannot opt in
// to something they never knew was printing.

impl std::fmt::Debug for Pack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pack")
            .field("src", &self.src)
            .field("dest", &self.dest)
            .field("level", &self.level)
            .finish()
    }
}

impl std::fmt::Debug for PackBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackBytes")
            .field("bytes", &self.data.len())
            .field("level", &self.level)
            .finish()
    }
}

impl std::fmt::Debug for Lock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lock")
            .field("src", &self.src)
            .field("dest", &self.dest)
            .field("password", &"<redacted>")
            .field("level", &self.level)
            .field("work_factor", &self.log_n)
            .finish()
    }
}

impl std::fmt::Debug for LockBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockBytes")
            .field("bytes", &self.data.len())
            .field("password", &"<redacted>")
            .field("level", &self.level)
            .field("work_factor", &self.log_n)
            .finish()
    }
}

impl std::fmt::Debug for Packed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Packed")
            .field("bytes", &self.bytes)
            .field("ratio", &self.ratio)
            .field("elapsed_s", &self.elapsed_s)
            .finish()
    }
}

impl std::fmt::Debug for Unpacked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Unpacked")
            .field("bytes", &self.bytes)
            .field("elapsed_s", &self.elapsed_s)
            .finish()
    }
}
