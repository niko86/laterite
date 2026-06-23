//! Validator errors.
//!
//! Rule *violations* are findings, not errors — they don't fail
//! `check_file`. `ValidatorError` is only for the cases where we can't
//! validate at all: the file is missing, can't be read, or isn't even
//! structurally an AGS4 file. (Invalid *encoding* is no longer one of
//! these — it's decoded lossily and reported as a Rule 1 finding; see
//! `parse_file` and O-32.)

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("read error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Retained for public-API back-compat only — **`parse_file` no
    /// longer raises this**. Invalid UTF-8 is now decoded lossily
    /// (`String::from_utf8_lossy`, mirroring python-ags4's
    /// `errors="replace"`) so cp1252/latin1 inputs validate and surface
    /// non-ASCII as a Rule 1 finding instead of becoming a
    /// zero-rules-evaluated black hole. Kept (not removed) so the
    /// error/exit-code map, the parity string, and downstream `match`
    /// arms stay compilable; it is now unreachable from the library.
    /// See O-32.
    #[error(
        "{0} is not valid UTF-8. This validator is UTF-8 only; convert first, e.g. \
         `iconv -f cp1252 -t utf-8 in.ags > out.ags`"
    )]
    NotUtf8(PathBuf),

    /// The file couldn't be parsed into AGS4 structure at all (e.g. no
    /// GROUP rows). We can't run rules against a non-AGS4 file.
    #[error("not a parseable AGS4 file: {0}")]
    NotAgs4(String),

    /// `--dict <path>` override couldn't be loaded.
    #[error("custom dictionary {path}: {reason}")]
    BadDict { path: PathBuf, reason: String },

    /// The file declares a `TRAN_AGS` edition we deliberately don't
    /// support (AGS 3.x — nothing is specced for it). We refuse rather
    /// than silently validate it against an AGS4 dictionary. (Unknown
    /// AGS4.x editions don't land here — they fall back to 4.1.1; see
    /// `resolve_dict_version`.)
    #[error(
        "AGS edition {found:?} is not supported — only AGS4 editions \
         4.0.3/4.0.4/4.1/4.1.1/4.2 are bundled"
    )]
    UnsupportedEdition { found: String },
}

/// Map the shared parse leaf's terminal into the validator's (#168 Phase 2).
/// The validator's `parse` wrappers convert via this so every caller keeps
/// handling `ValidatorError` unchanged. `NotUtf8` carries no path in the leaf
/// (it's FS-free) and is unreachable on the validator's lossy path anyway
/// (the validator never uses `Reject` mode), so an empty path is fine.
impl From<laterite_ags4_parse::ParseError> for ValidatorError {
    fn from(e: laterite_ags4_parse::ParseError) -> Self {
        use laterite_ags4_parse::ParseError as P;
        match e {
            P::NotAgs4(msg) => ValidatorError::NotAgs4(msg),
            P::UnsupportedEdition { found } => ValidatorError::UnsupportedEdition { found },
            P::NotUtf8 => ValidatorError::NotUtf8(PathBuf::new()),
            // Unreachable on the validator's lenient path (it never sets
            // `strict_structure`), but the match must be total — a structural
            // hard-fail is closest to "not a parseable AGS4 file".
            P::Structure(msg) => ValidatorError::NotAgs4(msg),
        }
    }
}
