//! The emitter's error type — leaf-local so the crate stays free of
//! `laterite-core` (which would be a dependency cycle, since `laterite-core`
//! depends on *this*). `laterite-core` provides `From<EmitError> for
//! CliError` so its `excel` / `db-to-ags4` callers keep using `?`.

use std::fmt;

use laterite_ags4_validator::findings::Findings;

#[derive(Debug)]
pub enum EmitError {
    /// Writing the AGS4 bytes failed (io on the in-memory/streamed writer).
    Write(String),
    /// Re-parsing the emitter's own output for validation failed. The
    /// emitter produces well-formed AGS4, so this is defensive — it would
    /// only fire on a genuine internal bug.
    Reparse(String),
    /// `EmitMode::Strict`: the generated output violates one or more
    /// error-severity AGS4 rules. Carries the findings so the caller sees
    /// exactly what was rejected.
    Invalid(Findings),
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmitError::Write(m) => write!(f, "ags4 emit: write failed: {m}"),
            EmitError::Reparse(m) => {
                write!(f, "ags4 emit: re-parse for validation failed: {m}")
            }
            EmitError::Invalid(found) => {
                let n: usize = found.values().map(Vec::len).sum();
                write!(f, "ags4 emit: strict mode rejected output ({n} finding(s))")
            }
        }
    }
}

impl std::error::Error for EmitError {}
