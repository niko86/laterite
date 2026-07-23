//! Run-versioned artifact paths — the corpus-qa `paths.rs` convention.
//!
//! Work dir resolution: explicit `--out-dir` → `$AGS4_FORGE_DIR` →
//! `./forge-runs` (CWD-relative — the least-surprise convention; a
//! copied release binary must not write back into the source checkout
//! via a compile-time `CARGO_MANIFEST_DIR`). Each run gets a sortable
//! UTC dir under `<out>/runs/<id>/`; a `runs/latest` pointer *file*
//! (not a symlink — Windows-robust) names the newest.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

/// `--out-dir` → `$AGS4_FORGE_DIR` → `./forge-runs`.
pub fn forge_dir(flag: Option<&Path>) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(env) = std::env::var_os("AGS4_FORGE_DIR") {
        return PathBuf::from(env);
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("forge-runs")
}

/// Sortable UTC run id — lexical order == chronological.
pub fn new_run_id() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

pub fn run_dir(out: &Path, run_id: &str) -> PathBuf {
    out.join("runs").join(run_id)
}

/// Point `runs/latest` at `run_id`.
pub fn set_latest_run(out: &Path, run_id: &str) -> Result<()> {
    let runs = out.join("runs");
    std::fs::create_dir_all(&runs).with_context(|| format!("create {}", runs.display()))?;
    std::fs::write(runs.join("latest"), run_id).with_context(|| "write runs/latest pointer")
}

/// Reject an out-dir that resolves inside the validator's asserted-
/// clean fixtures tree — forge must NEVER write there (corpus-qa e2e
/// asserts that dir hard-error-free; confirmed reproducers go to
/// `ags-wiki/.bootstrap/probes/`, not here).
pub fn guard_out_dir(out: &Path) -> Result<()> {
    let abs = std::path::absolute(out).unwrap_or_else(|_| out.to_path_buf());
    let s = abs.to_string_lossy().replace('\\', "/");
    if s.contains("laterite-ags4-validator/tests/fixtures") {
        anyhow::bail!(
            "refusing --out-dir inside laterite-ags4-validator/tests/fixtures \
             (that dir is asserted hard-error-free; forge output must not \
             pollute it)"
        );
    }
    Ok(())
}
