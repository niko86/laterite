//! Emit `LATERITE_ENGINE_FINGERPRINT` — the identity of the verdict-producing
//! engine, derived from its actual inputs rather than from a human remembering
//! to bump a number.
//!
//! An `.ags.idx` certificate records "this engine, over these bytes, found N
//! findings", and a later run skips re-validation only if the same engine would
//! still say so. Until now the recorded identity was `CARGO_PKG_VERSION`: a
//! hand-bumped semver that does not move when a rule's logic changes. Edit a
//! severity, fix a false positive, add a check — every certificate minted by the
//! old engine keeps claiming to be current, and its stale verdict keeps being
//! trusted. That is a false clean with a perfectly valid-looking cert.
//!
//! So hash what the verdict actually depends on:
//!   * every rule source file (`src/rules/**`, plus the parse + findings + world
//!     modules the rules are expressed in terms of), and
//!   * the bundled reference data — the dictionary and the rules catalogue —
//!     which are as much a part of "what the engine thinks" as the code is.
//!
//! Deliberately NOT hashed: `Cargo.toml`, tests, benches, and the crate's own
//! version. A dependency bump that changes rule behaviour without touching any
//! of these files would slip through — a stated residual, not an oversight: the
//! alternative (hashing the whole resolved dependency tree) invalidates every
//! certificate on any unrelated `cargo update`, which trades a rare unsound skip

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));

    // The engine's own sources. `lib.rs` is in here because it hosts the door
    // (`check_parsed`) and the dictionary-resolution policy (which edition a file
    // is judged against is part of the verdict).
    let mut files: Vec<PathBuf> = vec![
        manifest.join("src/lib.rs"),
        manifest.join("src/parse.rs"),
        manifest.join("src/findings.rs"),
        manifest.join("src/world.rs"),
        manifest.join("src/catalogue.rs"),
    ];
    collect_rs(&manifest.join("src/rules"), &mut files);

    // The reference data. A dictionary edit changes which headings are standard,
    // i.e. it changes verdicts, so it must change the fingerprint. Same crate,
    // same workspace — a relative path is stable here (and if the leaf ever moves,
    // this build fails loudly rather than silently hashing less).
    let reference = manifest.join("../laterite-ags4-reference/data");
    files.push(reference.join("ags_dictionary.json"));
    files.push(reference.join("rules_meta.json"));

    // Sort so the digest doesn't depend on directory-walk order.
    files.sort();

    let mut h = Sha256::new();
    for f in &files {
        let bytes = std::fs::read(f)
            .unwrap_or_else(|e| panic!("engine fingerprint: cannot read {}: {e}", f.display()));
        // Feed the *relative* name, not the absolute one: the fingerprint must be
        // reproducible across machines and CI checkouts, so it cannot depend on
        // where the repo happens to live.
        let name = f
            .strip_prefix(&manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        h.update(name.as_bytes());
        h.update([0]);
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
        println!("cargo::rerun-if-changed={}", f.display());
    }
    let digest = h.finalize();
    // 16 hex chars (64 bits) — plenty to distinguish engine builds, short enough
    // to read in a `.ags.idx` and in a diff.
    let short: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    println!("cargo::rustc-env=LATERITE_ENGINE_FINGERPRINT={short}");
}

/// Every `.rs` under `dir`, recursively.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("engine fingerprint: cannot list {}: {e}", dir.display()));
    for e in entries {
        let p = e.expect("dir entry").path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    println!("cargo::rerun-if-changed={}", dir.display());
}
