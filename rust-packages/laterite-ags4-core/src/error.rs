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
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::FileNotFound(_) => 3,
            Self::Schema(_) => 6,
        }
    }
}
