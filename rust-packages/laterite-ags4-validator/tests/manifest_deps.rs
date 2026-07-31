//! The manifest reading behind `ENGINE_FINGERPRINT`, including the shape it only
//! ever sees after `cargo publish` has rewritten the manifest (#158).
//!
//! This exists because that shape was previously unreachable from any test. The
//! defect was not subtle once seen — publishing strips `path`, the walk keyed on
//! `path`, so the walk found nothing — but nothing in the repo could produce a
//! packaged manifest, and a build script that quietly covers less still exits 0. The
//! logic is `include!`d from `src/manifest_deps.rs` (build scripts are not linkable)
//! so the packaged branch can be fed a manifest directly.

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/manifest_deps.rs"));

/// What `cargo publish` leaves behind: `path` gone, `version` kept. Taken from the
/// real error cargo emits when a path dep has no version requirement — "the packaged
/// dependency will use the version from crates.io, the `path` specification will be
/// removed from the dependency declaration".
const PACKAGED: &str = r#"
[package]
name = "laterite-ags4-validator"
version = "0.9.0"

[dependencies]
laterite-ags4-reference = { version = "0.9.0" }
laterite-ags4-types = { version = "0.9.0" }
laterite-ags4-parse = { version = "0.9.0" }
thiserror = "2"
deunicode = "1"
"#;

/// A packaged manifest still yields every one of our crates, each identified by the
/// version whose content crates.io cannot change.
///
/// Before #158 this returned NOTHING, and the fingerprint went on to describe a
/// quarter of the engine without a word about the rest.
#[test]
fn a_packaged_manifest_still_finds_our_crates() {
    let deps = own_deps_from(PACKAGED, "PACKAGED");
    let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "laterite-ags4-parse",
            "laterite-ags4-reference",
            "laterite-ags4-types"
        ],
        "a published manifest has no `path` keys — selecting on one covers nothing",
    );
    for d in &deps {
        assert!(d.path.is_none(), "{}: publish strips `path`", d.name);
        assert_eq!(
            d.version.as_deref(),
            Some("0.9.0"),
            "{}: the version requirement is the only identity left, so it must be read",
            d.name,
        );
    }
}

/// External crates never enter the digest, packaged or not. Hashing the resolved
/// tree would invalidate every certificate on any unrelated `cargo update`.
#[test]
fn external_crates_are_never_ours() {
    for text in [PACKAGED, &std::fs::read_to_string(manifest()).unwrap()] {
        let names: Vec<String> = own_deps_from(text, "t")
            .into_iter()
            .map(|d| d.name)
            .collect();
        for ext in ["thiserror", "deunicode", "sha2", "toml", "chrono", "serde"] {
            assert!(!names.contains(&ext.to_string()), "{ext} is not ours");
        }
    }
}

/// The `laterite` prefix must select exactly the set the old `path` key did.
///
/// This is what makes the prefix safe to rely on. It fails loudly if a `laterite*`
/// crate is ever depended on FROM crates.io rather than in-workspace (it would be
/// hashed by version while its sources sat right there), or if an in-workspace crate
/// is ever named without the prefix (it would vanish from the covered set — the #158
/// failure again, by a different route).
#[test]
fn laterite_deps_are_exactly_the_path_deps() {
    let text = std::fs::read_to_string(manifest()).unwrap();
    let by_prefix: Vec<String> = own_deps_from(&text, "manifest")
        .into_iter()
        .map(|d| d.name)
        .collect();

    let doc: toml::Table = text.parse().unwrap();
    let mut by_path: Vec<String> = doc["dependencies"]
        .as_table()
        .unwrap()
        .iter()
        .filter(|(_, spec)| spec.get("path").is_some())
        .map(|(name, _)| name.clone())
        .collect();
    by_path.sort();

    assert_eq!(
        by_prefix, by_path,
        "the `laterite` prefix and the in-workspace path deps have diverged — the \
         fingerprint would either hash a crate by version while its sources are \
         present, or drop one entirely",
    );
    assert!(!by_path.is_empty(), "zero is a bad witness for both sets");
}

/// A dependency table with no `[dependencies]` at all is empty, not a panic — a leaf
/// crate legitimately has none.
#[test]
fn a_manifest_without_dependencies_is_empty_not_an_error() {
    assert!(own_deps_from("[package]\nname = \"x\"\n", "bare").is_empty());
}

fn manifest() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}
