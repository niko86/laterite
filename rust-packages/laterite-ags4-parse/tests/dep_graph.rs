//! Leaf-purity guard for the parse crate (#168 §3.1).
//!
//! The whole point of a NEW leaf below both core and validator (rather than
//! folding the parser into core) is that it stays additive + wasm-clean: it
//! must never drag in core's wasm-hostile / FFI-coupling graph. This pins
//! that — the normal dep graph is `encoding_rs` + `memchr` (+ their pure-Rust
//! deps) and nothing else. `serde` is opt-in (default-off), so it doesn't
//! appear here. Mirrors `laterite-ags4-validator`'s `lean_dep_graph.rs`.

use std::process::Command;

/// Crates that must never appear in `laterite-ags4-parse`'s normal graph:
/// the parser stays a pure tokenizer + decoder. `laterite-ags4-types` is listed
/// so `parse` and `types` remain SIBLING leaves (no edge between them).
const FORBIDDEN: &[&str] = &[
    "csv",
    "age",
    "zstd",
    "rpassword",
    "secrecy",
    "pyo3",
    "polars",
    "walkdir",
    "rayon",
    "ratatui",
    "laterite-ags4-types",
];

#[test]
fn parse_leaf_default_dep_graph_is_pure() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(&cargo)
        .args(["tree", "-p", "laterite-ags4-parse", "-e", "normal"])
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
            tree.lines().any(|l| {
                let t = l.trim_start_matches(['├', '└', '│', '─', ' ']);
                t.split_whitespace().next() == Some(*dep)
            })
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "laterite-ags4-parse leaked forbidden crate(s): {leaked:?}\n\
         The parse leaf must stay a pure tokenizer/decoder (encoding_rs + memchr).\n\
         Full tree:\n{tree}"
    );
}
