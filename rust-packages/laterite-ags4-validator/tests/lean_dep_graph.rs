//! Lean dep-graph guard (the engine-purity invariant).
//!
//! `laterite-ags4-validator` is the clean-room engine that everything embeds —
//! transitive: if any of these heavy / FFI-coupling crates leaked into
//! the engine's *normal* dep graph, every consumer would inherit it —
//! most damagingly, `pyo3` would make the shipped `lat-db` binary link
//! libpython and require a Python runtime, the exact thing
//! "validator stays pyo3-free" intent in
//! `dec-python-imports-rust-library.md` was documented but unenforced;
//! this test enforces it. It matters most as `laterite-py`'s PyO3
//! surface grows (Stage B of the staged-adoption roadmap).
//!
//! Mechanism: shell `cargo tree -p laterite-ags4-validator -e normal` and assert
//! none of the forbidden crates appear. Since the CLI split (the
//! `lat` crate now owns comfy-table/indicatif and the optional
//! ratatui/crossterm TUI), the validator is a pure library leaf — its
//! normal graph is just phf/thiserror/chrono/encoding_rs + their deps.
//! `cargo tree` only resolves; it does not build, so there is no
//! target-dir lock contention inside `cargo test`.

use std::process::Command;

/// Crates that must never appear in the validator library's normal
/// dependency graph (matches the documented "no walkdir/rayon/ratatui"
/// guarantee + the pyo3-free invariant):
/// - `pyo3` — would make the shipped `lat-db` binary link libpython
///   and require a Python runtime (the thing dec-rust-drives-python
///   forbids). The single most important entry.
/// - `polars` — Rust↔Python ABI coupling the engine must stay free of.
/// - `walkdir` / `rayon` — dev/QA-only crates (corpus-qa/forge) that
///   would bloat the engine if they leaked in.
/// - `ratatui` — the TUI framework; it lives in the `lat` crate
///   behind that crate's opt-in `tui` feature. Since the CLI split the
///   validator has no dependency path to it at all; this entry stays as
///   a guard against it ever creeping back into the engine.
///
/// `crossterm` is deliberately NOT listed: historically it arrived
/// transitively via `laterite-cliutil → comfy-table` (terminal capability
/// detection for the findings table). That whole chain moved to
/// `lat` in the CLI split, so it no longer appears in the
/// validator's normal graph — but it was never a forbidden crate.
const FORBIDDEN: &[&str] = &["pyo3", "polars", "walkdir", "rayon", "ratatui"];

#[test]
fn validator_default_dep_graph_has_no_forbidden_crates() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(&cargo)
        .args(["tree", "-p", "laterite-ags4-validator", "-e", "normal"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run `cargo tree`");
    assert!(
        out.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);
    let leaked: Vec<&str> = FORBIDDEN
        .iter()
        .copied()
        .filter(|dep| {
            // word-boundary-ish match: `cargo tree` prints `name v1.2.3`
            tree.lines().any(|l| {
                let t = l.trim_start_matches(['├', '└', '│', '─', ' ']);
                t.split_whitespace().next() == Some(*dep)
            })
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "laterite-ags4-validator's normal dep graph leaked forbidden crate(s): {leaked:?}\n\
         The engine must stay pyo3/polars/walkdir/rayon/ratatui/crossterm-free.\n\
         Full tree:\n{tree}"
    );
}
