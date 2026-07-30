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
//! So hash what the verdict actually depends on, in two parts:
//!
//! 1. **This crate's verdict-producing modules** — the rules, plus the parse /
//!    findings / world / catalogue modules they are expressed in terms of, plus
//!    `lib.rs` (it hosts the door `check_parsed` and the dictionary-resolution
//!    policy: which edition a file is judged against is part of the verdict).
//!    `error.rs` and `fixes.rs` are excluded deliberately — an error type and a
//!    repair cannot change what the engine *decides*.
//!
//! 2. **Every in-workspace crate the verdict is expressed THROUGH**, discovered by
//!    walking `[dependencies]` path deps transitively (#550). This half used to be
//!    missing, and it was not a small gap: `laterite-types` owns `format_nsf`, the
//!    formatter that *computes* Rule 8's verdict; `laterite-ags4-parse` owns the
//!    tokenizer that *decides field boundaries*; `laterite-ags4-reference`'s
//!    `build.rs` *generates* the per-edition dictionary tables (the JSON was hashed
//!    while the code projecting it was not). Edit any of them and yesterday's cert
//!    still read `Vouched`.
//!
//! Discovered, not listed, and deliberately so: a hand-written file list is how
//! this crate ended up hashing three-quarters of its own engine. Dev- and
//! build-dependencies are NOT followed — they cannot reach a verdict
//! (`laterite-ags4-core` is a dev-dep here, and following it would be both wrong
//! and circular).
//!
//! # Why in-workspace deps are hashed whole, and external ones not at all
//!
//! The two halves are asymmetric in a way the previous residual note missed.
//!
//! Hashing the whole *resolved* dependency tree would invalidate every certificate
//! on any unrelated `cargo update` — upstream churn we do not control, at a rate we
//! do not choose. That remains excluded, and it is why a dependency bump that
//! changes rule behaviour without touching any hashed file still slips through: a
//! real, stated residual.
//!
//! But **in-workspace path deps are not that case** — no `cargo update` touches
//! them; they move only when we edit them. So the old justification (that covering
//! deps "trades a rare unsound skip for a constant one") was a false dichotomy for
//! exactly the crates that decide verdicts, and they are now covered.
//!
//! Within those crates we hash *every* source file, not a curated subset, because
//! the two failure directions are not equal: an over-broad hash costs a redundant
//! revalidation (safe, and invisible to users — their engine changes per release
//! anyway, and any release touches something), while an under-broad hash is a false
//! clean. When in doubt, hash it. If that ever gets genuinely expensive — a leaf
//! churning on code that cannot reach a verdict, `laterite-types::arrow_cols` say —
//! the answer is to SPLIT the leaf so the validator stops depending on the
//! non-verdict half, not to reintroduce a curated list here. A crate holding both
//! verdict and non-verdict code is a boundary smell, and this file is where it
//! shows up.
//!
//! Also not hashed: `Cargo.toml`, tests, benches, and the crate's own version.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // Names are fed to the digest relative to the WORKSPACE root, not this crate:
    // the covered files now live in sibling crates, and the fingerprint must be
    // reproducible across machines and CI checkouts, so it cannot depend on where
    // the repo happens to live. Canonicalised because the crate dirs below are too
    // (a `strip_prefix` between a canonical and a non-canonical path never matches,
    // and on macOS `/tmp` → `/private/tmp` makes that a real, silent difference).
    let workspace = canon(manifest.parent().expect("crate dir has a parent"));

    // (1) This crate's own verdict-producing modules — the deliberate subset.
    let mut files: Vec<PathBuf> = vec![
        manifest.join("src/lib.rs"),
        manifest.join("src/parse.rs"),
        manifest.join("src/findings.rs"),
        manifest.join("src/world.rs"),
        manifest.join("src/catalogue.rs"),
    ];
    collect_rs(&manifest.join("src/rules"), &mut files);

    // (2) Every in-workspace crate the verdict is expressed through.
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    seen.insert(canon(&manifest)); // don't re-walk this crate under rule (1)
    for rel in path_deps(&manifest.join("Cargo.toml")) {
        collect_crate(&manifest.join(&rel), &mut files, &mut seen);
    }

    // Sort so the digest doesn't depend on directory-walk or dep-declaration order.
    files.sort();
    files.dedup();

    let mut h = Sha256::new();
    let mut names: Vec<String> = Vec::with_capacity(files.len());
    for f in &files {
        let bytes = std::fs::read(f)
            .unwrap_or_else(|e| panic!("engine fingerprint: cannot read {}: {e}", f.display()));
        let name = canon(f)
            .strip_prefix(&workspace)
            .unwrap_or_else(|_| {
                panic!(
                    "engine fingerprint: {} is outside the workspace root {} — the digest would \
                     stop being reproducible across checkouts",
                    f.display(),
                    workspace.display()
                )
            })
            .to_string_lossy()
            .replace('\\', "/");
        h.update(name.as_bytes());
        h.update([0]);
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
        println!("cargo::rerun-if-changed={}", f.display());
        names.push(name);
    }
    let digest = h.finalize();
    // 16 hex chars (64 bits) — plenty to distinguish engine builds, short enough
    // to read in a `.ags.idx` and in a diff.
    let short = digest.iter().take(8).fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    });
    println!("cargo::rustc-env=LATERITE_ENGINE_FINGERPRINT={short}");
    // The digest is opaque, so on its own it can only ever prove that COMPARING
    // fingerprints works — never that the right things went into one. Publish the
    // covered set so a test can hold the coverage floor (`tests/engine_fingerprint.rs`).
    println!(
        "cargo::rustc-env=LATERITE_ENGINE_FINGERPRINT_FILES={}",
        names.join(";")
    );
}

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|e| {
        panic!(
            "engine fingerprint: cannot canonicalize {}: {e}",
            p.display()
        )
    })
}

/// The `[dependencies]` entries of `manifest_toml` that are in-workspace path deps,
/// as declared relative paths.
///
/// `[dev-dependencies]` and `[build-dependencies]` are deliberately not read: neither
/// can reach a verdict, and following the dev-dep on `laterite-ags4-core` would walk
/// back into a crate that depends on this one.
fn path_deps(manifest_toml: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(manifest_toml).unwrap_or_else(|e| {
        panic!(
            "engine fingerprint: cannot read {}: {e}",
            manifest_toml.display()
        )
    });
    // `Table`, not `Value`: a manifest is a TOML *document*, and `Value`'s FromStr
    // parses a bare value — it rejects the leading comment on line 1.
    let doc: toml::Table = text.parse().unwrap_or_else(|e| {
        panic!(
            "engine fingerprint: cannot parse {}: {e}",
            manifest_toml.display()
        )
    });
    let Some(deps) = doc.get("dependencies").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    let mut out: Vec<String> = deps
        .values()
        .filter_map(|spec| spec.get("path")?.as_str().map(str::to_string))
        .collect();
    out.sort();
    out
}

/// One in-workspace crate's hash-relevant files — every source file, its `build.rs`
/// (it GENERATES verdict inputs: `laterite-ags4-reference`'s projects the per-edition
/// dictionary tables), and its bundled `data/` — then recurse into its own path deps.
///
/// `seen` is keyed on the canonical dir because the graph is a diamond, not a tree:
/// the validator depends on `laterite-types` directly AND through
/// `laterite-ags4-reference`. Without it those files would be hashed twice — harmless
/// for the digest's correctness, but it would make the covered set a lie.
fn collect_crate(dir: &Path, out: &mut Vec<PathBuf>, seen: &mut BTreeSet<PathBuf>) {
    let dir = canon(dir);
    if !seen.insert(dir.clone()) {
        return;
    }
    collect_rs(&dir.join("src"), out);
    let build_rs = dir.join("build.rs");
    if build_rs.is_file() {
        out.push(build_rs);
    }
    let data = dir.join("data");
    if data.is_dir() {
        collect_any(&data, out);
    }
    for rel in path_deps(&dir.join("Cargo.toml")) {
        collect_crate(&dir.join(&rel), out, seen);
    }
}

/// Every `.rs` under `dir`, recursively.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    walk(dir, out, &|p| p.extension().is_some_and(|x| x == "rs"));
}

/// Every file under `dir`, recursively — bundled reference data, whatever its suffix.
fn collect_any(dir: &Path, out: &mut Vec<PathBuf>) {
    walk(dir, out, &|_| true);
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, keep: &dyn Fn(&Path) -> bool) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("engine fingerprint: cannot list {}: {e}", dir.display()));
    for e in entries {
        let p = e.expect("dir entry").path();
        if p.is_dir() {
            walk(&p, out, keep);
        } else if keep(&p) {
            out.push(p);
        }
    }
    println!("cargo::rerun-if-changed={}", dir.display());
}
