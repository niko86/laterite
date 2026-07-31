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

/// Every `laterite*` dependency must resolve to sources in THIS tree.
///
/// This is what makes the name prefix safe to rely on. It fails loudly if a
/// `laterite*` crate is ever consumed FROM crates.io rather than from the workspace
/// (it would be hashed by version while its sources sat right there), or if an
/// in-workspace crate is ever named without the prefix (it would vanish from the
/// covered set — the #158 failure again, by another route).
///
/// "Resolves to sources" is now two shapes, not one. It used to be a `path` key.
/// Since the publish-prep version requirements landed, a member says
/// `{ workspace = true }` and BOTH the path and the version live in the workspace
/// root's `[workspace.dependencies]`. An assertion that only knew the first shape
/// would read the second as "no path — must be a registry crate", which is the
/// wrong answer about the crate sitting in the next directory.
#[test]
fn every_laterite_dep_resolves_to_sources_in_this_tree() {
    let text = std::fs::read_to_string(manifest()).unwrap();
    let deps = own_deps_from(&text, "manifest");
    assert!(
        !deps.is_empty(),
        "zero is a bad witness: no laterite deps found"
    );

    let ws_text = std::fs::read_to_string(workspace_manifest()).unwrap();
    let ws = workspace_deps_from(&ws_text, "workspace");
    assert!(
        !ws.is_empty(),
        "[workspace.dependencies] has no laterite entries"
    );

    for d in &deps {
        if d.workspace {
            let e = ws.iter().find(|e| e.name == d.name).unwrap_or_else(|| {
                panic!(
                    "`{}` says `workspace = true` but no workspace entry defines it",
                    d.name
                )
            });
            assert!(
                e.path.is_some(),
                "[workspace.dependencies] `{}` has no path — it would resolve from \
                 crates.io, and the fingerprint would identify it by version while its \
                 sources sit in this tree",
                d.name,
            );
        } else {
            assert!(
                d.path.is_some(),
                "`{}` has neither a path nor workspace inheritance — an in-workspace \
                 crate consumed as if it were external",
                d.name,
            );
        }
    }
}

/// Every entry in `[workspace.dependencies]` carries a version, which is the whole
/// reason they exist: `cargo package` rejects a dependency without one, so 7 of the
/// 10 engine-tier crates could not be packaged at all before they were added.
#[test]
fn every_workspace_entry_carries_a_version() {
    let ws_text = std::fs::read_to_string(workspace_manifest()).unwrap();
    let ws = workspace_deps_from(&ws_text, "workspace");
    assert!(!ws.is_empty(), "zero is a bad witness");
    for e in &ws {
        assert!(
            e.version.is_some(),
            "[workspace.dependencies] `{}` has no version — `cargo package` refuses \
             a dependency with no version requirement, because publishing strips the \
             path and leaves nothing to identify it",
            e.name,
        );
    }
}

/// A member that inherits carries neither field itself — the shape the resolver has
/// to look up rather than read.
#[test]
fn an_inherited_dep_carries_neither_path_nor_version() {
    let deps = own_deps_from(
        "[dependencies]\nlaterite-ags4-parse = { workspace = true }\n",
        "inherited",
    );
    assert_eq!(deps.len(), 1);
    assert!(deps[0].workspace);
    assert!(deps[0].path.is_none() && deps[0].version.is_none());
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

fn workspace_manifest() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate dir has a parent")
        .join("Cargo.toml")
}
