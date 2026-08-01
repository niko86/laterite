//! One error type for the whole surface.
//!
//! Opaque struct + coarse kind, rather than an enum mirroring the engines'.
//! Every engine error type is free to gain a variant — that is the point of the
//! two tiers — and re-exporting them here would make each of those a breaking
//! change for every consumer of this crate.

use std::fmt;

/// What went wrong, coarsely.
///
/// `#[non_exhaustive]` because this list will grow: a consumer must be able to
/// keep compiling when it does. Match with a `_` arm, or use [`Error::kind_str`]
/// if you are routing on the string.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// The file could not be read, written, or found.
    Io,
    /// The bytes are not AGS4 — structurally unparseable, not merely invalid.
    NotAgs4,
    /// A dictionary or edition was requested that does not exist or does not parse.
    BadDictionary,
    /// The data could not be written as valid AGS4.
    Emit,
    /// A caller argument was wrong — an unknown group code, a row index out of
    /// range, an encoding label nothing recognises.
    InvalidArgument,
    /// Something the engine reported that this crate does not classify.
    ///
    /// Not dead weight: the engine names its error kinds as strings and is the
    /// single producer of that domain, so it can add one without this crate
    /// changing. Mapping such a token onto whichever existing kind looked
    /// closest would be a confident wrong answer; this is the honest one. It
    /// carries the engine's own message.
    Other,
}

impl ErrorKind {
    /// The stable wire token for this kind.
    ///
    /// Shared verbatim with the Python, Node and `lat` surfaces, which is what
    /// makes it worth freezing: a tool that routes on laterite's error strings
    /// gets the same tokens whichever binding produced them. These strings are
    /// part of the public API and will not change under a consumer.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Io => "io",
            ErrorKind::NotAgs4 => "not_ags4",
            ErrorKind::BadDictionary => "bad_dict",
            ErrorKind::Emit => "emit",
            ErrorKind::InvalidArgument => "invalid_argument",
            ErrorKind::Other => "error",
        }
    }

    /// The process exit code `lat` uses for this kind, so a wrapper binary can
    /// exit the same way without restating the mapping.
    #[must_use]
    pub fn exit_code(self) -> i32 {
        match self {
            ErrorKind::Io => 2,
            ErrorKind::NotAgs4 | ErrorKind::BadDictionary | ErrorKind::InvalidArgument => 3,
            ErrorKind::Emit => 4,
            ErrorKind::Other => 1,
        }
    }
}

/// An error from any laterite operation.
///
/// Deliberately a struct with private fields, not an enum. Adding a case to a
/// public enum is a breaking change; adding an [`ErrorKind`] to this is not.
pub struct Error {
    kind: ErrorKind,
    message: String,
    /// The engine error, kept only so `{:#}` and an `anyhow`/`eyre` chain render
    /// the underlying detail. It is wrapped in a PRIVATE newtype (see [`Source`]),
    /// so `source()` can be walked and printed but never `downcast_ref` onto an
    /// engine type — which would put that type back in the public API through
    /// the back door.
    source: Option<Source>,
}

struct Source(String);

impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Source {}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub(crate) fn with_source(
        kind: ErrorKind,
        message: impl Into<String>,
        source: impl fmt::Display,
    ) -> Error {
        Error {
            kind,
            message: message.into(),
            source: Some(Source(source.to_string())),
        }
    }

    /// Which coarse category this is.
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The stable wire token — shorthand for `self.kind().as_str()`.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        self.kind.as_str()
    }

    /// The process exit code `lat` would use — shorthand for
    /// `self.kind().exit_code()`.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.kind.exit_code()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .field("source", &self.source)
            .finish()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s as &(dyn std::error::Error + 'static))
    }
}
