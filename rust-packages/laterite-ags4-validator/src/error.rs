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

impl ValidatorError {
    /// Stable machine token for the surface error protocols (Python `_errors`,
    /// Node `errors.ts`, the CLI). The single PRODUCER of the error-kind value
    /// domain — every surface delegates here instead of re-mapping the variants
    /// by hand, so the tables can't drift. A new variant forces this match.
    pub fn kind(&self) -> &'static str {
        match self {
            ValidatorError::NotFound(_) => "not_found",
            ValidatorError::Io { .. } => "io",
            ValidatorError::NotAgs4(_) => "not_ags4",
            ValidatorError::BadDict { .. } => "bad_dict",
            ValidatorError::UnsupportedEdition { .. } => "unsupported_edition",
        }
    }

    /// The process exit code each maps to — byte-faithful to `lat`'s
    /// contract (3 = unreadable / io, 4 = not-AGS4 / unsupported-edition, 5 =
    /// bad-dict). The single producer of the exit-code value domain.
    pub fn exit_code(&self) -> i32 {
        match self {
            ValidatorError::NotFound(_) | ValidatorError::Io { .. } => 3,
            ValidatorError::NotAgs4(_) | ValidatorError::UnsupportedEdition { .. } => 4,
            ValidatorError::BadDict { .. } => 5,
        }
    }
}

/// Map the shared parse leaf's terminal into the validator's (#168 Phase 2).
/// The validator's `parse` wrappers convert via this so every caller keeps
/// handling `ValidatorError` unchanged. The leaf's `NotUtf8` / `Structure`
/// only arise in `Reject` mode, which the validator never uses (it decodes
/// lossily and surfaces non-UTF-8 as a Rule 1 finding — O-32), so they map to
/// the closest validator error to keep the conversion total.
impl From<laterite_ags4_parse::ParseError> for ValidatorError {
    fn from(e: laterite_ags4_parse::ParseError) -> Self {
        use laterite_ags4_parse::ParseError as P;
        match e {
            P::NotAgs4(msg) => ValidatorError::NotAgs4(msg),
            P::UnsupportedEdition { found } => ValidatorError::UnsupportedEdition { found },
            P::NotUtf8 => ValidatorError::NotAgs4("file is not valid UTF-8".to_string()),
            P::Structure(msg) => ValidatorError::NotAgs4(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_and_exit_code_table_is_exhaustive() {
        // The single source for the surface error protocols. A new variant makes
        // this match non-exhaustive → this test stops compiling, forcing the
        // token/code to be assigned here (not silently in a surface).
        let cases: Vec<(ValidatorError, &str, i32)> = vec![
            (ValidatorError::NotFound("x".into()), "not_found", 3),
            (
                ValidatorError::Io {
                    path: "x".into(),
                    source: std::io::Error::other("boom"),
                },
                "io",
                3,
            ),
            (ValidatorError::NotAgs4("x".into()), "not_ags4", 4),
            (
                ValidatorError::UnsupportedEdition {
                    found: "3.1".into(),
                },
                "unsupported_edition",
                4,
            ),
            (
                ValidatorError::BadDict {
                    path: "x".into(),
                    reason: "x".into(),
                },
                "bad_dict",
                5,
            ),
        ];
        for (e, kind, code) in &cases {
            assert_eq!(e.kind(), *kind);
            assert_eq!(e.exit_code(), *code);
        }
    }
}
