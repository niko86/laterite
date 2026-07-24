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

    #[error("schema error: {0}")]
    Schema(String),

    /// A group declared the same heading twice (AGS4 Rule 7). Fatal on the read
    /// path by default: rows are keyed by heading name, so continuing would
    /// return the second column's values for the first column's position — a
    /// wrong answer that looks like a complete one. The recovery mode
    /// (`DuplicateHeadings::Recover`) reads the file with the repeats suffixed.
    #[error(
        "duplicate heading {heading:?} in group {group:?} (AGS4 Rule 7) — \
         reading it would silently merge two columns; re-read in recovery mode \
         to keep both, suffixed __2, __3, …"
    )]
    DuplicateHeading { group: String, heading: String },
}

impl CliError {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::FileNotFound(_) => 3,
            // Same class as Schema: the file is structurally unusable as read.
            Self::Schema(_) | Self::DuplicateHeading { .. } => 6,
        }
    }
}
