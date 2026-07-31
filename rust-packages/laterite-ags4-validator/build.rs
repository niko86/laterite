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
//!    missing, and it was not a small gap: `laterite-ags4-types` owns `format_nsf`, the
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
//! churning on code that cannot reach a verdict, `laterite-ags4-types::arrow_cols` say —
//! the answer is to SPLIT the leaf so the validator stops depending on the
//! non-verdict half, not to reintroduce a curated list here. A crate holding both
//! verdict and non-verdict code is a boundary smell, and this file is where it
//! shows up.
//!
//! Also not hashed: `Cargo.toml`, tests, benches, and the crate's own version.
//!
//! # When the sources are not there to hash (#158)
//!
//! Everything above describes a build from this workspace. Built from a crates.io
//! tarball it was silently false: `cargo publish` REWRITES the packaged manifest and
//! strips the `path` key from every dependency, keeping only `version`. So
//! `path_deps` found nothing, the recursion never happened, and the covered set
//! collapsed from **29 files across 4 crates to 14 files in 1** — the validator's
//! own. Nothing failed. `walk` panics on a missing directory, but this code never
//! asked for one; it just stopped.
//!
//! That is not a cosmetic narrowing. A consumer pinning
//! `laterite-ags4-validator 0.1.0` picks up `laterite-ags4-parse 0.1.1` on any
//! `cargo update` — the tokenizer that decides where fields end — and every
//! certificate they hold keeps reading `Vouched` against an engine that now decides
//! differently. Precisely the bug #550 fixed, reopened for registry consumers only.
//!
//! So when an in-workspace dependency's sources are NOT reachable, its identity goes
//! into the digest as `name@version` instead. That is sound in exactly the place it
//! is used: crates.io is immutable, so a published `laterite-ags4-parse 0.1.1` can
//! never be different bytes than it was. The objection that sank `CARGO_PKG_VERSION`
//! for THIS crate — a local edit does not move it — cannot happen to a registry
//! artefact, because there is no local edit.
//!
//! The consequence, accepted deliberately: the same source tree yields a different
//! fingerprint depending on whether it was built here or from a tarball. That lands
//! on the SAFE side of the asymmetry this file already commits to — a mismatch costs
//! one redundant revalidation, never a false clean.
//!
//! "In-workspace" is decided by the `laterite` name prefix rather than by the
//! presence of a `path` key, because the `path` key is the very thing publishing
//! removes. `laterite_deps_are_exactly_the_path_deps` holds that prefix honest: in
//! this workspace the two sets must be identical, so a `laterite*` dependency that
//! is NOT ours, or an in-workspace crate that is not named `laterite*`, fails loudly
//! rather than being silently mis-sorted.
//!
//! One thing this still does not normalise: file names are fed to the digest
//! relative to the crate directory's PARENT, which in a registry checkout is
//! `laterite-ags4-validator-0.9.0` rather than `laterite-ags4-validator`. Published
//! builds therefore also re-fingerprint on a version bump with no code change. Same
//! safe direction, left alone rather than fixed here because changing the naming
//! moves the fingerprint for every existing certificate.

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

    // (2) Every in-workspace crate the verdict is expressed through — by source
    // where the sources are reachable, else by the `name@version` that identifies
    // the immutable published artefact (#158).
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut pinned: BTreeSet<String> = BTreeSet::new();
    seen.insert(canon(&manifest)); // don't re-walk this crate under rule (1)
    resolve_deps(&manifest, &mut files, &mut seen, &mut pinned);

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

    // Then the deps identified by version rather than by source. Fed with a `dep:`
    // marker and no content, so a pinned entry can never collide with a file whose
    // path happens to look like `name@version`.
    for p in &pinned {
        h.update(b"dep:");
        h.update(p.as_bytes());
        h.update([0]);
        names.push(p.clone());
    }

    coverage_floor(&names, &pinned);

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

/// Refuse to emit a fingerprint that covers obviously too little.
///
/// `tests/engine_fingerprint.rs` already holds a coverage floor, and it is the
/// better place for the detailed one — it can name specific files and say why each
/// matters. But a test cannot run where this defect lives: tests are not in the
/// published tarball at all, so the packaged build that silently fingerprinted a
/// quarter of the engine had nothing checking it and never would have.
///
/// Hence a floor HERE, in the one piece of code that runs in every build context.
/// It is deliberately coarse — an exact count would be a second thing to update on
/// every refactor, and a floor that people edit reflexively stops being a floor.
/// What it asserts is structural: the rule modules were found, and every one of our
/// own dependencies was accounted for somehow.
fn coverage_floor(names: &[String], pinned: &BTreeSet<String>) {
    let rules = names
        .iter()
        // No extension test: `collect_rs` only ever collects `.rs`, so anything
        // under `src/rules/` in the covered set is a rule source by construction.
        .filter(|n| n.contains("/src/rules/"))
        .count();
    assert!(
        rules >= 5,
        "engine fingerprint: only {rules} rule source(s) covered — the walk over \
         src/rules found almost nothing, so the digest describes an engine that is not \
         the one being built.\nCovered: {}",
        names.join(", "),
    );

    // At least one dependency must be accounted for, by either route. Zero means the
    // manifest parse silently produced nothing — the exact shape of #158, where
    // `[dependencies]` was read but every entry filtered away.
    let by_source = names
        .iter()
        .any(|n| !n.contains('@') && !n.starts_with(CRATE));
    assert!(
        by_source || !pinned.is_empty(),
        "engine fingerprint: not one dependency was covered, by source or by version. \
         The verdict runs through laterite-ags4-parse, -types and -reference; a digest \
         over this crate alone would vouch for certificates those crates can invalidate.",
    );
}

/// This crate's own directory name, used to tell its files apart from a dependency's
/// in the covered set. Not `CARGO_PKG_NAME`: the names in the covered set are paths,
/// and in a registry checkout the directory carries the version too.
const CRATE: &str = "laterite-ags4-validator";

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|e| {
        panic!(
            "engine fingerprint: cannot canonicalize {}: {e}",
            p.display()
        )
    })
}

// `OwnDep` + `own_deps_from` — shared verbatim with `tests/manifest_deps.rs` so the
// packaged-manifest branch is reachable from a test rather than only from a real
// publish. See that file's header for why it is `include!`d rather than a module.
include!("src/manifest_deps.rs");

/// The `[workspace.dependencies]` entry for `name`, plus the workspace root it was
/// found in (paths there are relative to it, not to the member).
///
/// Walks up from `start` looking for a manifest with a `[workspace]` table, rather
/// than assuming the root is one level up. It is today — every member is a direct
/// child of `rust-packages/` — but that is a layout detail, and a build script that
/// silently resolves nothing when it changes is the failure this whole file is about.
fn workspace_entry(start: &Path, name: &str) -> Option<(OwnDep, PathBuf)> {
    for root in start.ancestors().skip(1) {
        let manifest = root.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).ok()?;
        if !text.contains("[workspace]") {
            continue;
        }
        let found = workspace_deps_from(&text, &manifest.display().to_string())
            .into_iter()
            .find(|d| d.name == name)?;
        return Some((found, root.to_path_buf()));
    }
    None
}

/// [`own_deps_from`] over a manifest on disk.
fn own_deps(manifest_toml: &Path) -> Vec<OwnDep> {
    let text = std::fs::read_to_string(manifest_toml).unwrap_or_else(|e| {
        panic!(
            "engine fingerprint: cannot read {}: {e}",
            manifest_toml.display()
        )
    });
    own_deps_from(&text, &manifest_toml.display().to_string())
}

/// One in-workspace crate's hash-relevant files — every source file, its `build.rs`
/// (it GENERATES verdict inputs: `laterite-ags4-reference`'s projects the per-edition
/// dictionary tables), and its bundled `data/` — then recurse into its own path deps.
///
/// `seen` is keyed on the canonical dir because the graph is a diamond, not a tree:
/// the validator depends on `laterite-ags4-types` directly AND through
/// `laterite-ags4-reference`. Without it those files would be hashed twice — harmless
/// for the digest's correctness, but it would make the covered set a lie.
fn collect_crate(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    pinned: &mut BTreeSet<String>,
) {
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
    resolve_deps(&dir, out, seen, pinned);
}

/// Account for every one of our own crates that `dir`'s manifest depends on —
/// by SOURCE where the sources are there, by `name@version` where they are not.
///
/// The second arm is the packaged case: publishing strips `path`, so there is no
/// directory to walk, and the version requirement left behind is the only handle on
/// the dependency's identity. It is a sufficient one, because that version's content
/// is immutable on crates.io.
///
/// A `{ workspace = true }` entry carries neither, and is resolved against the
/// workspace root's `[workspace.dependencies]` first — where the path is relative to
/// that root rather than to `dir`. Publishing inlines those entries, so this arm is
/// in-tree only; it exists because the version requirements the publish set needs
/// live in the workspace table, and reading a member manifest alone now sees
/// `{ workspace = true }` and nothing else.
///
/// A dependency with none of the three is a hard error rather than a skip. Skipping is
/// exactly what this code did before, and it is why a published build fingerprinted a
/// quarter of its own engine without saying so.
fn resolve_deps(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    pinned: &mut BTreeSet<String>,
) {
    for dep in own_deps(&dir.join("Cargo.toml")) {
        let (dep, base) = if dep.workspace {
            match workspace_entry(dir, &dep.name) {
                Some((inherited, root)) => (inherited, root),
                None => panic!(
                    "engine fingerprint: `{}` of {} is declared `workspace = true` but no \
                     `[workspace.dependencies]` entry defines it",
                    dep.name,
                    dir.display(),
                ),
            }
        } else {
            (dep, dir.to_path_buf())
        };
        let by_path = dep.path.as_ref().map(|rel| base.join(rel));
        match (by_path, &dep.version) {
            (Some(p), _) if p.is_dir() => collect_crate(&p, out, seen, pinned),
            (_, Some(v)) => {
                pinned.insert(format!("{}@{v}", dep.name));
            }
            (_, None) => panic!(
                "engine fingerprint: dependency `{}` of {} has neither reachable sources \
                 nor a version requirement, so nothing identifies it — refusing to emit a \
                 fingerprint that silently omits a crate the verdict runs through",
                dep.name,
                dir.display(),
            ),
        }
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
