//! The emitter's error type — leaf-local so the crate stays free of
//! `laterite-ags4-core` (which would be a dependency cycle, since `laterite-ags4-core`
//! depends on *this*). `laterite-ags4-core` provides `From<EmitError> for
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
    /// A cell value contains an embedded carriage return or line feed.
    /// AGS4 (Rule 6) forbids CR/LF *within* a field and offers no in-field
    /// escape, so writing the bytes raw would split the row on re-parse —
    /// an illegal file (#423). The writer refuses rather than silently fold
    /// the value: a caller that wants it cleaned should fix it first (fold
    /// CR/LF → space via the fix engine), so the mutation is explicit.
    /// `tag` is the row's descriptor; `field` is the offending cell's index
    /// (`0` = the descriptor itself, `1` = the first data value).
    EmbeddedNewline { tag: String, field: usize },
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
            EmitError::EmbeddedNewline { tag, field } => write!(
                f,
                "ags4 emit: cell contains an embedded CR/LF (field {field} of a \
                 \"{tag}\" row); AGS4 Rule 6 forbids newlines within a field and \
                 there is no in-field escape — fix the value (fold CR/LF to a space) \
                 before emitting"
            ),
        }
    }
}

impl std::error::Error for EmitError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each variant renders a specific, non-empty message — a whole-`fmt`
    /// replacement (`Ok(default())` = empty string) must fail every assertion.
    #[test]
    fn display_is_specific_per_variant() {
        assert!(
            EmitError::Write("disk full".into())
                .to_string()
                .contains("write failed")
        );
        assert!(
            EmitError::Reparse("bad utf8".into())
                .to_string()
                .contains("re-parse")
        );
        let nl = EmitError::EmbeddedNewline {
            tag: "DATA".into(),
            field: 2,
        }
        .to_string();
        assert!(nl.contains("Rule 6") && nl.contains("field 2"), "got: {nl}");
    }
}
