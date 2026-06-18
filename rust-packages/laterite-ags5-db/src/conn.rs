//! DuckDB connection helper.
//!
//! Every read-side command opens the .ags5db file with the same pattern:
//! check existence, open read-only, surface schema errors via `CliError`.
//! Factored here so commands don't repeat the boilerplate.

use duckdb::{AccessMode, Config, Connection};
use laterite_ags4_core::error::CliError;
use std::path::Path;

pub fn open_readonly(path: &Path) -> Result<Connection, CliError> {
    if !path.exists() {
        return Err(CliError::FileNotFound(path.display().to_string()));
    }
    let config = Config::default()
        .access_mode(AccessMode::ReadOnly)
        .map_err(|e| CliError::Schema(format!("config: {}", e)))?;
    Connection::open_with_flags(path, config).map_err(|e| CliError::Schema(e.to_string()))
}
