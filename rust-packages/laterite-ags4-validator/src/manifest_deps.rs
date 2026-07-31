// Reading our own crates out of a `Cargo.toml`'s `[dependencies]`.
//
// **Not a module of this library.** It is `include!`d by `build.rs`, and again by
// `tests/manifest_deps.rs`, so the packaged-manifest path can be exercised by a
// test instead of only by a real `cargo publish`. That indirection exists for a
// specific reason: #158 was a defect that only appeared when the crate was built
// from a tarball, and nothing in the repo could build one, so it sat unnoticed
// behind a build script that reported success. Logic that behaves differently in a
// context we cannot run is logic that needs to be callable outside that context.
//
// It lives under `src/` rather than beside `build.rs` so the `include`
// allowlist that governs the published tarball already carries it — `build.rs`
// cannot compile without it.

/// One of our own crates as a manifest declares it.
struct OwnDep {
    name: String,
    /// The declared relative path. `None` in a PACKAGED manifest: `cargo publish`
    /// rewrites the manifest and strips `path`, keeping `version`. That rewrite is
    /// the whole of #158.
    path: Option<String>,
    /// The version requirement. Always present in a packaged manifest; in this
    /// workspace the version is inherited (`version.workspace = true`), which is a
    /// table with no string `version`, so this is `None` and the path is used.
    version: Option<String>,
    /// `{ workspace = true }` — BOTH path and version live in the workspace root's
    /// `[workspace.dependencies]`, so this entry carries neither and the caller must
    /// look them up there. Note the path found that way is relative to the workspace
    /// root, not to the crate.
    ///
    /// Never true in a packaged manifest: `cargo publish` inlines inherited
    /// dependencies, the same rewrite that strips `path`.
    workspace: bool,
}

/// Our own crates among `manifest_text`'s `[dependencies]`.
///
/// Selected by the `laterite` name prefix, NOT by the presence of a `path` key.
/// Keying on `path` is what let the covered set collapse in a packaged build, since
/// publishing removes it. `laterite_deps_are_exactly_the_path_deps` keeps the prefix
/// honest: in this workspace the two sets must be identical.
///
/// External crates are excluded entirely, which is deliberate and unchanged —
/// hashing the resolved tree would invalidate every certificate on any unrelated
/// `cargo update`.
///
/// `[dev-dependencies]` and `[build-dependencies]` are not read: neither can reach a
/// verdict, and following the dev-dep on `laterite-ags4-core` would walk back into a
/// crate that depends on this one.
fn own_deps_from(manifest_text: &str, whence: &str) -> Vec<OwnDep> {
    // `Table`, not `Value`: a manifest is a TOML *document*, and `Value`'s FromStr
    // parses a bare value — it rejects the leading comment on line 1.
    let doc: toml::Table = manifest_text
        .parse()
        .unwrap_or_else(|e| panic!("engine fingerprint: cannot parse {whence}: {e}"));
    let Some(deps) = doc.get("dependencies").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    let mut out: Vec<OwnDep> = deps
        .iter()
        .filter(|(name, _)| name.starts_with("laterite"))
        .map(|(name, spec)| OwnDep {
            name: name.clone(),
            path: spec
                .get("path")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            // A plain `dep = "1.2"` is a string; the table form carries `version`.
            version: spec
                .as_str()
                .or_else(|| spec.get("version")?.as_str())
                .map(str::to_string),
            workspace: spec
                .get("workspace")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// `[workspace.dependencies]` from a workspace ROOT manifest, as `name -> (path,
/// version)`.
///
/// The inherited half of the lookup above. Both fields are optional here for the
/// same reasons they are on [`OwnDep`]: a packaged manifest has no paths, and an
/// entry may legitimately be version-only.
fn workspace_deps_from(manifest_text: &str, whence: &str) -> Vec<OwnDep> {
    let doc: toml::Table = manifest_text
        .parse()
        .unwrap_or_else(|e| panic!("engine fingerprint: cannot parse {whence}: {e}"));
    let Some(deps) = doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(toml::Value::as_table)
    else {
        return Vec::new();
    };
    deps.iter()
        .filter(|(name, _)| name.starts_with("laterite"))
        .map(|(name, spec)| OwnDep {
            name: name.clone(),
            path: spec
                .get("path")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            version: spec
                .as_str()
                .or_else(|| spec.get("version")?.as_str())
                .map(str::to_string),
            workspace: false,
        })
        .collect()
}
