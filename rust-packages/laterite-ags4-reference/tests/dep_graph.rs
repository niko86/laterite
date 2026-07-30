//! Leaf-purity guard for the reference crate (#568 §4).
//!
//! `laterite-ags4-reference` is a *near-leaf*: it takes `laterite-types` (for
//! `keychain`) and, since #568, `laterite-ags4-parse` (the runtime `.ags`
//! custom-dict reader). Both are wasm-clean sibling leaves — the whole point of
//! adding the parse dep here rather than reimplementing a tokenizer is that it
//! drags in NOTHING wasm-hostile. This pins that: the normal graph must never
//! reach core/validator (an up-dep) or any FFI/heavyweight crate. Mirrors
//! `laterite-ags4-parse`'s `dep_graph.rs`.
//!
//! Until this test, the crate's "near-leaf" status was asserted only by a
//! manifest comment — read by no one, gated by nothing (#557's lesson). The
//! `.ags`-reader dep is exactly the kind of edge that could have smuggled in
//! wasm weight unnoticed; now it can't.

use std::process::Command;

/// Crates that must never appear in `laterite-ags4-reference`'s normal graph.
/// `laterite-types` and `laterite-ags4-parse` are deliberately ABSENT — they are
/// the two allowed workspace deps. The rest would signal wasm-hostility (age,
/// zstd, getrandom-via-age), an up-dep (core/validator), or heavy coupling.
const FORBIDDEN: &[&str] = &[
    "age",
    "zstd",
    "rpassword",
    "secrecy",
    "pyo3",
    "polars",
    "duckdb",
    "calamine",
    "csv",
    "walkdir",
    "rayon",
    "ratatui",
    "laterite-ags4-core",
    "laterite-ags4-validator",
    "laterite-ags4-emit",
];

#[test]
fn reference_leaf_default_dep_graph_is_pure() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(&cargo)
        .args(["tree", "-p", "laterite-ags4-reference", "-e", "normal"])
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
        "laterite-ags4-reference leaked forbidden crate(s): {leaked:?}\n\
         The reference leaf may take only laterite-types + laterite-ags4-parse\n\
         from the workspace, and nothing wasm-hostile.\n\
         Full tree:\n{tree}"
    );
}
