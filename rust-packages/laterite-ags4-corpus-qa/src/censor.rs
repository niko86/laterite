//! `censor` — anonymise harvested `.ags` files for sharing.
//!
//! The third corpus role (gather → **clean** → check): scrub the sensitive cell
//! values the SSOT `sensitive_headings.json` classifies, emit hash-named files
//! plus a source-stripped manifest, so a cleaned corpus can be shared /
//! committed with no client data.
//!
//! The scrub ENGINE itself now lives in the shared `laterite-ags4-censor` leaf
//! (laterite-dev#581) — the same engine the browser `Anonymiser` drives via the engine
//! wasm, so the two can't drift. This module is the corpus **wrapper** around
//! it: resolve the classification into a [`Policy`], run the crawl manifest's
//! files through [`censor`] in parallel, name each output by its source hash,
//! and roll the per-file [`Tally`] into the run report.

use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use laterite_ags4_censor::{CensorOptions, Policy, Tally, censor};
use laterite_cliutil::{progress_bar, styled_table};
use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::cli::CensorArgs;
use crate::manifest::{CrawlManifest, ManifestEntry, SCHEMA as MANIFEST_SCHEMA};
use crate::output::{self, Ctx, Report};

/// The embedded SSOT classification — always available, recompiled when the file
/// changes (`include_str!` tracks it). `--sensitive` overrides at runtime. Fed
/// to [`Policy::from_sensitive_json`] in the leaf.
const EMBEDDED_SENSITIVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../laterite-ags4-core/data/sensitive_headings.json"
));

/// Size-stratified selection: sort by size and take N evenly-spaced entries
/// (deterministic) so a sample spans small→large files. `None` (no `--sample`)
/// ⇒ everything, in manifest order.
fn select(entries: &[ManifestEntry], sample: Option<usize>) -> Vec<&ManifestEntry> {
    let Some(n) = sample else {
        return entries.iter().collect();
    };
    if n == 0 || entries.len() <= n {
        return entries.iter().collect();
    }
    let mut by_size: Vec<&ManifestEntry> = entries.iter().collect();
    by_size.sort_by_key(|e| (e.size, e.dest.clone()));
    (0..n)
        .map(|k| {
            // Evenly spaced across the size-sorted list, endpoints included.
            let idx = k * (by_size.len() - 1) / (n - 1);
            by_size[idx]
        })
        .collect()
}

pub fn run(args: &CensorArgs, ctx: Ctx, corpus_dir: &Path) -> Result<i32> {
    // (cleaned-dest, size, sha256, tally) per file; None = skipped (read /
    // decode failure — reported, never silently dropped).
    type Cleaned = (ManifestEntry, Tally);

    let raw = match &args.sensitive {
        Some(p) => std::fs::read_to_string(p)
            .with_context(|| format!("read sensitive list {}", p.display()))?,
        None => EMBEDDED_SENSITIVE.to_string(),
    };
    let policy = Policy::from_sensitive_json(&raw, args.include_freetext)
        .context("parse sensitive_headings.json")?;
    let opts = CensorOptions {
        token: args.token.clone(),
        keywords: args.redact.clone(),
        drop_custom: !args.keep_custom,
    };

    let manifest_path = match &args.manifest {
        Some(p) => p.clone(),
        None => {
            crate::paths::resolve_run_dir(corpus_dir, args.run_id.as_deref())?.join("manifest.json")
        }
    };
    let manifest = CrawlManifest::load(&manifest_path)
        .with_context(|| "load manifest (run `crawl` first?)")?;
    let selected = select(&manifest.files, args.sample);

    let out_dir = args.out_dir.clone();

    if ctx.dry_run {
        let plan = output::Plan::new(
            "censor",
            format!(
                "would anonymise {} file(s) → {}",
                selected.len(),
                out_dir.display()
            ),
        )
        .with("would_clean", selected.len() as u64)
        .with("out_dir", out_dir.display().to_string());
        output::emit(&plan, &ctx)?;
        return Ok(0);
    }

    std::fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;

    let pb = progress_bar(selected.len() as u64, ctx.quiet);
    pb.set_message("censoring");

    let results: Vec<Option<Cleaned>> = selected
        .par_iter()
        .map(|e| {
            let abs = corpus_dir.join(&e.dest);
            let bytes = std::fs::read(&abs).ok()?;
            let text = String::from_utf8(bytes).ok()?; // non-UTF-8 → skip
            // file_id = the SOURCE content hash: PROJ_ID is set to it AND the
            // cleaned file is named by it, so PROJ_ID == filename.
            let (cleaned, tally) = censor(&text, &e.sha256, &policy, &opts);
            let mut h = Sha256::new();
            h.update(cleaned.as_bytes());
            let cleaned_sha = hex::encode(h.finalize());
            let dest = format!("{}.ags", e.sha256);
            std::fs::write(out_dir.join(&dest), &cleaned).ok()?;
            pb.inc(1);
            Some((
                ManifestEntry {
                    source: String::new(), // SCRUBBED — no client path leaks
                    dest,
                    // Cleaned content's hash (integrity); the *name* is the
                    // source hash above.
                    size: cleaned.len() as u64,
                    sha256: cleaned_sha,
                    copy_error: None,
                },
                tally,
            ))
        })
        .collect();
    pb.finish_and_clear();

    let mut entries: Vec<ManifestEntry> = Vec::new();
    let mut tally = Tally::default();
    let mut skipped = 0u64;
    for r in results {
        match r {
            Some((entry, t)) => {
                entries.push(entry);
                tally.merge(&t);
            }
            None => skipped += 1,
        }
    }
    // Deterministic order + dedup identical SOURCE files (same dest, which is
    // the source hash) — collapses the manifest's content duplicates.
    entries.sort_by(|a, b| a.dest.cmp(&b.dest));
    entries.dedup_by(|a, b| a.dest == b.dest);

    // A source-stripped manifest so the cleaned corpus is a drop-in for
    // `validate` / `baseline` (corpus_dir = out_dir).
    let cleaned_manifest = CrawlManifest {
        schema: MANIFEST_SCHEMA,
        created: manifest.created.clone(),
        root: "(censored — source scrubbed)".to_string(),
        selection: "censored".to_string(),
        walk_skipped: 0,
        files: entries.clone(),
    };
    let manifest_out = out_dir.join("manifest.json");
    cleaned_manifest.save(&manifest_out)?;

    let report = CensorReport {
        cleaned: entries.len(),
        skipped,
        out_dir: out_dir.display().to_string(),
        cells: tally.into(),
    };
    output::note(format!("cleaned corpus → {}", out_dir.display()));
    output::emit(&report, &ctx)?;
    Ok(0)
}

#[derive(Debug, serde::Serialize)]
pub struct CensorReport {
    pub cleaned: usize,
    pub skipped: u64,
    pub out_dir: String,
    #[serde(flatten)]
    cells: TallySer,
}

// Flattened, serde-friendly view of the leaf's cell/structure tallies.
#[derive(Debug, serde::Serialize)]
struct TallySer {
    pseudonym_cells: u64,
    blanked_cells: u64,
    tokenised_cells: u64,
    bracket_units_stripped: u64,
    keyword_hits: u64,
    custom_columns_dropped: u64,
    custom_groups_dropped: u64,
    orphan_defs_dropped: u64,
}

impl From<Tally> for TallySer {
    fn from(t: Tally) -> Self {
        TallySer {
            pseudonym_cells: t.pseudonym,
            blanked_cells: t.blank,
            tokenised_cells: t.token,
            bracket_units_stripped: t.brackets,
            keyword_hits: t.keyword,
            custom_columns_dropped: t.dropped_cols,
            custom_groups_dropped: t.dropped_groups,
            orphan_defs_dropped: t.dropped_defs,
        }
    }
}

impl Report for CensorReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        let c = &self.cells;
        writeln!(
            w,
            "{}",
            styled_table(
                &["Censor", "Value"],
                vec![
                    vec!["files cleaned".into(), self.cleaned.to_string()],
                    vec!["skipped (non-utf8/io)".into(), self.skipped.to_string()],
                    vec!["pseudonymised cells".into(), c.pseudonym_cells.to_string()],
                    vec!["blanked cells".into(), c.blanked_cells.to_string()],
                    vec!["tokenised cells".into(), c.tokenised_cells.to_string()],
                    vec![
                        "bracket units stripped".into(),
                        c.bracket_units_stripped.to_string()
                    ],
                    vec!["keyword hits".into(), c.keyword_hits.to_string()],
                    vec![
                        "custom columns dropped".into(),
                        c.custom_columns_dropped.to_string()
                    ],
                    vec![
                        "custom groups dropped".into(),
                        c.custom_groups_dropped.to_string()
                    ],
                    vec![
                        "orphan defs dropped".into(),
                        c.orphan_defs_dropped.to_string()
                    ],
                    vec!["out dir".into(), self.out_dir.clone()],
                ],
                ctx.colour(),
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_is_deterministic_and_spans_sizes() {
        let mk = |dest: &str, size: u64| ManifestEntry {
            source: String::new(),
            dest: dest.to_string(),
            size,
            sha256: String::new(),
            copy_error: None,
        };
        let entries = vec![
            mk("a", 10),
            mk("b", 50),
            mk("c", 30),
            mk("d", 5),
            mk("e", 90),
        ];
        // Sample of 3 spans smallest→largest by size, deterministic.
        let picked: Vec<&str> = select(&entries, Some(3))
            .iter()
            .map(|e| e.dest.as_str())
            .collect();
        assert_eq!(picked, vec!["d", "c", "e"]); // sizes 5, 30, 90
        // No sample ⇒ everything in manifest order.
        let all: Vec<&str> = select(&entries, None)
            .iter()
            .map(|e| e.dest.as_str())
            .collect();
        assert_eq!(all, vec!["a", "b", "c", "d", "e"]);
    }
}
