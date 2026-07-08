//! Typed CLI errors with exit codes that mirror the Python `_cli.py` contract.
//!
//! The Python CLI documents exit codes 0-8 in its `--help` epilog. Agents
//! and CI rely on those for self-correction without parsing prose. This
//! enum is the Rust mirror: each variant maps to a documented exit code.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error(
        "file has no _spec_* tables (pre-6.5 format); re-ingest from the AGS4 source via `ags5db ags4-to-db`."
    )]
    PreVersion65,

    #[error("unknown group: {code}{}", suggest_hint(.hints))]
    UnknownGroup { code: String, hints: Vec<String> },

    #[error("--where {arg:?}: {reason}")]
    Predicate { arg: String, reason: String },

    #[error("schema error: {0}")]
    Schema(String),

    /// Source data exercises an AGS feature this binary deliberately
    /// doesn't handle yet (currently: AGS4 Record Link / `RL` type, AGS4.1
    /// Rule 11). Exit code 7 so CI / agents can distinguish "we bailed
    /// safely" from a generic schema error.
    #[error("{0}")]
    UnsupportedFeature(String),

    #[error("SQL error: {0}")]
    Sql(String),

    /// Returned from the stub write-command handlers. Lets agents discover
    /// "this binary doesn't do that, here's where to go" without crashing.
    #[error("ags5db: write commands are not in this binary - run `ags5db-py {0}`")]
    NotImpl(String),

    /// `db-to-ags4 --validate`: the emitted AGS4 file was written but
    /// failed the bundled validator. Exit code 10 so CI / agents can
    /// distinguish "emitted but not spec-conformant" from a write error.
    #[error(
        "validation failed: {findings} finding(s) in {file} (run `lat validate {file}` for detail)"
    )]
    Validation { findings: usize, file: String },
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::PreVersion65 => 2,
            Self::FileNotFound(_) => 3,
            Self::UnknownGroup { .. } => 4,
            Self::Predicate { .. } => 5,
            Self::Schema(_) => 6,
            Self::UnsupportedFeature(_) => 7,
            Self::Sql(_) => 8,
            Self::NotImpl(_) => 9,
            Self::Validation { .. } => 10,
        }
    }
}

/// The AGS4 writer/emitter lives in the `laterite-ags4-emit` leaf now; map its
/// error back onto `CliError` so `laterite-ags4-core`'s excel + `ags5db`'s
/// db-to-ags4 callers keep using `?` over `write_ags4`. `write_ags4` yields
/// `Write` or `EmbeddedNewline` (a cell carrying a raw CR/LF, #423); the
/// `Reparse` / `Invalid` variants come from `emit_ags4`'s validity modes
/// (not used by these callers) but are mapped for totality.
impl From<laterite_ags4_emit::EmitError> for CliError {
    fn from(e: laterite_ags4_emit::EmitError) -> Self {
        match e {
            // Preserve the historical "ags4 write: …" Schema message.
            laterite_ags4_emit::EmitError::Write(m) => CliError::Schema(format!("ags4 write: {m}")),
            laterite_ags4_emit::EmitError::Reparse(m) => {
                CliError::Schema(format!("ags4 emit: {m}"))
            }
            laterite_ags4_emit::EmitError::Invalid(found) => {
                let n: usize = found.values().map(Vec::len).sum();
                CliError::Schema(format!(
                    "ags4 emit: strict mode rejected output ({n} finding(s))"
                ))
            }
            e @ laterite_ags4_emit::EmitError::EmbeddedNewline { .. } => {
                CliError::Schema(format!("ags4 write: {e}"))
            }
        }
    }
}

fn suggest_hint(hints: &[String]) -> String {
    if hints.is_empty() {
        String::new()
    } else {
        format!("; did you mean {}?", hints.join(", "))
    }
}
