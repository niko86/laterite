//! Lean dep-graph guard (the engine-purity invariant).
//!
//! `ags4-validator` is the clean-room engine that everything embeds —
//! transitive: if any of these heavy / FFI-coupling crates leaked into
//! the engine's *normal* dep graph, every consumer would inherit it —
//! most damagingly, `pyo3` would make the shipped `ags5db` binary link
//! libpython and require a Python runtime, the exact thing
//! "validator stays pyo3-free" intent in
//! `dec-python-imports-rust-library.md` was documented but unenforced;
//! this test enforces it. It matters most as `laterite-py`'s PyO3
//! surface grows (Stage B of the staged-adoption roadmap).
//!
//! Mechanism: shell `cargo tree -p ags4-validator -e normal` (default
//! features — `tui`/ratatui/crossterm stay opt-in) and assert none of
//! the forbidden crates appear. `cargo tree` only resolves; it does
//! not build, so there is no target-dir lock contention inside
//! `cargo test`.

use std::process::Command;

/// Crates that must never appear in the validator library's normal
/// dependency graph (matches the documented "no walkdir/rayon/ratatui"
/// guarantee + the pyo3-free invariant):
/// - `pyo3` — would make the shipped `ags5db` binary link libpython
///   and require a Python runtime (the thing dec-rust-drives-python
///   forbids). The single most important entry.
/// - `polars` — Rust↔Python ABI coupling the engine must stay free of.
/// - `walkdir` / `rayon` — dev/QA-only crates (corpus-qa/forge) that
///   would bloat the engine if they leaked in.
/// - `ratatui` — the TUI framework; must stay behind the opt-in `tui`
///   feature, never in the default graph.
///
/// `crossterm` is deliberately NOT listed: it arrives transitively via
/// `ags-cliutil → comfy-table → crossterm` (terminal capability
/// detection for the findings table), a legitimate normal dep — not
/// the `tui` path. The historical guarantee never excluded it.
const FORBIDDEN: &[&str] = &["pyo3", "polars", "walkdir", "rayon", "ratatui"];

#[test]
fn validator_default_dep_graph_has_no_forbidden_crates() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(&cargo)
        .args(["tree", "-p", "ags4-validator", "-e", "normal"])
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
        "ags4-validator's normal dep graph leaked forbidden crate(s): {leaked:?}\n\
         The engine must stay pyo3/polars/walkdir/rayon/ratatui/crossterm-free.\n\
         Full tree:\n{tree}"
    );
}
