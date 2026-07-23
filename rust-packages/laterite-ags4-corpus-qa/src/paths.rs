//! Corpus-dir resolution, run-versioned artifact paths, and
//! collision-safe harvested filenames.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};

/// Resolve the corpus working dir: explicit `--corpus-dir` →
/// `$AGS4_CORPUS_DIR` → `./corpus` (relative to the **current working
/// directory**, created on demand).
///
/// The default used to be `CARGO_MANIFEST_DIR/../../corpus` — but
/// `CARGO_MANIFEST_DIR` is baked in at *compile time*, so a copied
/// release binary always wrote back into the original source checkout
/// no matter where it ran (surprising, and wrong off the dev box).
/// CWD-relative `./corpus` is the least-surprise CLI convention and
/// still lands in `<repo>/corpus` when run from the repo root for
/// dogfooding. The resolved path is echoed via the `manifest → …`
/// stderr hint so it's never a mystery where output went.
pub fn corpus_dir(flag: Option<&Path>) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(env) = std::env::var_os("AGS4_CORPUS_DIR") {
        return PathBuf::from(env);
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("corpus")
}

// --- run-versioned artifacts ----------------------------------------
//
// `manifest.json` / `report.json` / `parity.json` used to live at
// fixed paths and were clobbered on every re-run, while `harvested/`
// (content-addressed) accumulated. They now live under
// `<corpus>/runs/<run-id>/` so history is kept; a tiny
// `<corpus>/runs/latest` *pointer file* (NOT a symlink — Windows-
// robust) records the newest run so `validate`/`parity` find the
// crawl's output without a flag. `harvested/` is untouched (it's the
// shared content cache; re-crawl stays cheap).

/// A sortable UTC run id, e.g. `20260515T184500Z`. Lexical order ==
/// chronological order, so `ls runs/` and "newest" are trivial.
pub fn new_run_id() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// `<corpus>/runs/<run_id>` (the caller creates it lazily — `save()`
/// already `create_dir_all`s the artifact's parent).
pub fn run_dir(corpus: &Path, run_id: &str) -> PathBuf {
    corpus.join("runs").join(run_id)
}

/// The run dir the `runs/latest` pointer names, if any.
pub fn latest_run_dir(corpus: &Path) -> Option<PathBuf> {
    let id = std::fs::read_to_string(corpus.join("runs").join("latest")).ok()?;
    let id = id.trim();
    (!id.is_empty()).then(|| run_dir(corpus, id))
}

/// Point `runs/latest` at `run_id` (called by `crawl` after it writes
/// a manifest into the default run dir).
pub fn set_latest_run(corpus: &Path, run_id: &str) -> Result<()> {
    let runs = corpus.join("runs");
    std::fs::create_dir_all(&runs).with_context(|| format!("create {}", runs.display()))?;
    std::fs::write(runs.join("latest"), run_id).with_context(|| "write runs/latest pointer")
}

/// The run id an artifact path belongs to, if it lives under
/// `<corpus>/runs/<id>/…` (and isn't the `latest` pointer itself).
///
/// WHY: `set_latest_run` was only ever called by `crawl`, so a
/// standalone `validate`/`parity` that wrote into `runs/<id>/` left
/// `runs/latest` pointing at the *old* crawl — a later no-arg `parity`
/// then silently read stale results (the `rev-newbinary` trap). This
/// is the pure-path half of the fix: callers run it on the artifact
/// they just wrote and, on `Some(id)`, repoint `latest` so "last
/// activity under runs/ wins". A path *outside* `runs/` (an explicit
/// `--report/--out` elsewhere) yields `None` → `latest` is left
/// alone. No fs touch — `std::path::absolute` normalises without
/// requiring the path to exist, so this works pre- or post-write and
/// for the real "relative artifact + absolute corpus" call shape.
pub fn run_id_under(corpus: &Path, artifact: &Path) -> Option<String> {
    let runs = std::path::absolute(corpus.join("runs")).ok()?;
    let abs = std::path::absolute(artifact).ok()?;
    let rel = abs.strip_prefix(&runs).ok()?;
    let id = rel.components().next()?.as_os_str().to_str()?.to_string();
    (!id.is_empty() && id != "latest").then_some(id)
}

/// Resolve the run dir for a default artifact path. Precedence:
/// explicit `--run-id` → the `runs/latest` pointer. Errors with an
/// actionable hint when neither exists (call only when a default
/// path is actually needed — explicit `--manifest/--report/--out`
/// bypass this entirely).
pub fn resolve_run_dir(corpus: &Path, run_id: Option<&str>) -> Result<PathBuf> {
    if let Some(id) = run_id {
        return Ok(run_dir(corpus, id));
    }
    latest_run_dir(corpus).with_context(|| {
        format!(
            "no run found under {} — run `crawl` first, or pass --run-id / \
             explicit --manifest/--report",
            corpus.join("runs").display()
        )
    })
}

/// Collision-safe destination filename for a harvested file.
///
/// `{14 hex of sha256(source-path-bytes)}__{sanitised original name}`.
/// Path-based (not content-based) → idempotent across re-crawls and
/// two different dirs with the same filename never collide. The
/// sanitised tail is `[A-Za-z0-9._-]` only, so a hostile share name
/// can never contain `/`, `\`, or `..` and escape the harvested dir.
pub fn dest_name(source: &Path) -> String {
    let mut h = Sha256::new();
    h.update(source.to_string_lossy().as_bytes());
    let digest = h.finalize();
    let prefix = hex::encode(&digest[..7]); // 14 hex chars

    let raw = source
        .file_name()
        .map_or_else(|| "unnamed".into(), |s| s.to_string_lossy().into_owned());
    let mut name: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    // Guard against an all-dots / empty name (e.g. ".." → "..").
    if name.is_empty() || name.chars().all(|c| c == '.') {
        name = format!("file_{name}");
    }
    format!("{prefix}__{name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_id_under_matches_artifacts_inside_runs_only() {
        // The real validate/parity shape: the artifact is always built
        // by joining onto `corpus`, so the runs/<id> prefix holds
        // whether `corpus` is relative or absolute (both go through
        // std::path::absolute consistently).
        for corpus in [
            Path::new("sandbox").to_path_buf(),
            std::env::temp_dir().join("ags4-corpus"),
        ] {
            let inside = corpus
                .join("runs")
                .join("20260516T120000Z")
                .join("report.json");
            assert_eq!(
                run_id_under(&corpus, &inside).as_deref(),
                Some("20260516T120000Z"),
                "artifact under runs/<id>/ should yield that id",
            );

            // An explicit --report/--out outside runs/ → leave latest
            // alone (None).
            assert_eq!(run_id_under(&corpus, &corpus.join("external.json")), None);

            // The `latest` pointer path itself is never a run id.
            assert_eq!(
                run_id_under(&corpus, &corpus.join("runs").join("latest")),
                None,
            );
        }
    }

    #[test]
    fn dest_name_is_idempotent_and_collision_distinct() {
        // Use the native separator: `Path::file_name()` only segments
        // `\` on Windows, so a literal UNC string fails on Unix where
        // the whole path is treated as one big filename and the
        // `__delivery.ags` tail never appears.
        #[cfg(windows)]
        let (a, b) = (
            Path::new(r"\\srv\share\projA\delivery.ags"),
            Path::new(r"\\srv\share\projB\delivery.ags"),
        );
        #[cfg(not(windows))]
        let (a, b) = (
            Path::new("/srv/share/projA/delivery.ags"),
            Path::new("/srv/share/projB/delivery.ags"),
        );
        // Same source path → identical dest every time.
        assert_eq!(dest_name(a), dest_name(a));
        // Same filename, different dirs → different dest.
        assert_ne!(dest_name(a), dest_name(b));
        assert!(dest_name(a).ends_with("__delivery.ags"));
    }

    #[test]
    fn dest_name_sanitises_hostile_names() {
        let d = dest_name(Path::new(r"\\srv\x\..\..\evil name #1.ags"));
        let tail = d.split("__").nth(1).unwrap();
        assert!(!tail.contains('/') && !tail.contains('\\') && !tail.contains(".."));
        assert!(
            tail.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        );
    }

    #[test]
    fn dest_name_handles_nameless_and_dot_names() {
        // `/a/..` → Path::file_name() is None → safe "unnamed".
        let d = dest_name(Path::new("/a/.."));
        assert!(d.ends_with("__unnamed"), "got {d}");
        // A literal all-dots component IS a file_name → guarded.
        let d2 = dest_name(Path::new("a/..."));
        assert!(d2.contains("__file_"), "got {d2}");
    }
}
