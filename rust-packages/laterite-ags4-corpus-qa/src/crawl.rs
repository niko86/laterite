//! `crawl` — walk a (network) root, select a subset, copy locally.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use laterite_cliutil::{MultiLine, Spinner, progress_bar};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::cli::CrawlArgs;
use crate::manifest::{CrawlManifest, ManifestEntry, SCHEMA};
use crate::output::{self, Ctx, Plan};
// Seedable PRNG (SplitMix64) + Algorithm-R reservoir sampling moved to
// the shared `laterite-ags4-parity` crate so `laterite-ags4-forge` reuses the identical
// deterministic sampler. `reservoir` is now generic over the item
// type; the `PathBuf` call sites in this module are unchanged.
use laterite_ags4_parity::{Rng, reservoir};

fn is_ags(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ags"))
}

/// One subtree → its `.ags` files ≤ `max_bytes`. The shared filter
/// used by both the sequential and parallel walkers. Unreadable
/// dirs/files are counted in `skipped` and skipped — never aborts.
/// `on_dir` fires once per directory entered (the live progress
/// feed); `on_skip` reports an unreadable entry's message — the
/// sequential walker just `eprintln!`s it; the parallel walker routes
/// it through `MultiLine::suspend` so it can't smear the worker area.
fn walk_subtree(
    start: &Path,
    follow: bool,
    max_bytes: Option<u64>,
    skipped: &AtomicU64,
    mut on_dir: impl FnMut(&Path),
    on_skip: impl Fn(&str),
) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(start).follow_links(follow) {
        match entry {
            Err(e) => {
                on_skip(&e.to_string());
                skipped.fetch_add(1, Ordering::Relaxed);
            }
            Ok(e) => {
                if e.file_type().is_dir() {
                    on_dir(e.path());
                    continue;
                }
                if !e.file_type().is_file() || !is_ags(e.path()) {
                    continue;
                }
                if let Some(max) = max_bytes {
                    if e.metadata().map_or(0, |m| m.len()) > max {
                        continue;
                    }
                }
                out.push(e.path().to_path_buf());
            }
        }
    }
    out
}

/// Walk `root`, yielding every `.ags` file ≤ `max_bytes`.
///
/// `walk_jobs <= 1` → the original single-threaded `WalkDir` from
/// `root` (byte-for-byte unchanged: live per-dir spinner, exact yield
/// order — existing `--seed` determinism and tests are untouched).
///
/// `walk_jobs > 1` → fan a `WalkDir` out over `root`'s **top-level
/// subdirs** on a rayon pool (the dominant cost on a slow network
/// share is the per-file `stat`/enumeration). To keep `--seed`
/// reproducible the subdir list is **sorted** and the per-subtree
/// results are concatenated in that fixed order (rayon `collect`
/// preserves input order; `WalkDir` is deterministic within a
/// subtree), so the path list — hence the reservoir sample — is
/// identical to the sequential walk.
fn walk(
    root: &Path,
    follow: bool,
    max_bytes: Option<u64>,
    skipped: &AtomicU64,
    quiet: bool,
    walk_jobs: usize,
) -> Vec<PathBuf> {
    if walk_jobs <= 1 {
        // Byte-identical to before: own a single Spinner here (was
        // created/dropped by `run`), same init + per-dir messages,
        // same Drop clear. Determinism/order untouched.
        let sp = Spinner::start("walking…", quiet);
        let mut last_dir = String::new();
        return walk_subtree(
            root,
            follow,
            max_bytes,
            skipped,
            |d| {
                let d = d.display().to_string();
                if d != last_dir {
                    sp.set(&format!("walking {d}"));
                    last_dir = d;
                }
            },
            |m| eprintln!("skip: {m}"),
        );
    }

    // Split root's direct entries into subdirs + root's own files.
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut root_files: Vec<PathBuf> = Vec::new();
    match std::fs::read_dir(root) {
        Ok(rd) => {
            for ent in rd {
                let Ok(ent) = ent else {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                let p = ent.path();
                let is_dir = ent.file_type().is_ok_and(|t| t.is_dir());
                if is_dir {
                    subdirs.push(p);
                } else if is_ags(&p) {
                    let ok = max_bytes
                        .is_none_or(|m| std::fs::metadata(&p).map_or(0, |md| md.len()) <= m);
                    if ok {
                        root_files.push(p);
                    }
                }
            }
        }
        Err(e) => {
            // Can't enumerate root → fall back to a single walk so we
            // still produce a (sequential) result rather than nothing.
            eprintln!("skip: {e}");
            skipped.fetch_add(1, Ordering::Relaxed);
            return walk_subtree(
                root,
                follow,
                max_bytes,
                skipped,
                |_| {},
                |m| eprintln!("skip: {m}"),
            );
        }
    }
    // Deterministic order → reproducible `--seed` sampling.
    subdirs.sort();
    root_files.sort();

    let done = AtomicU64::new(0);
    let files = AtomicU64::new(0);
    let total = subdirs.len() as u64;
    // One header + N worker lines. Each worker shows the nested dir
    // it's currently descending so progress through deep folders is
    // visible (the single counter couldn't). Same TTY/quiet gating
    // as Spinner; cleared on drop.
    let prog = MultiLine::start(
        &format!("walking… 0/{total} dir(s), 0 files"),
        walk_jobs,
        quiet,
    );
    // Fit the per-worker label to the real terminal width (keeping
    // the deepest, changing tail) instead of a fixed cap. Queried
    // once — terminal resize mid-walk is a non-issue here.
    let cols = laterite_cliutil::term_cols();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(walk_jobs)
        .build()
        .expect("build walk thread pool");
    let per_subtree: Vec<Vec<PathBuf>> = pool.install(|| {
        subdirs
            .par_iter()
            .map(|d| {
                // rayon contract: Some(0..num_threads) inside
                // `pool.install`. Fallback never panics; set_line
                // bounds-checks anyway. The rayon→line mapping lives
                // HERE because laterite-cliutil is rayon-free.
                let line = rayon::current_thread_index().unwrap_or(0);
                let v = walk_subtree(
                    d,
                    follow,
                    max_bytes,
                    skipped,
                    |dir| prog.set_line(line, &walk_label(dir, root, cols)),
                    |m| prog.suspend(|| eprintln!("skip: {m}")),
                );
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                let f = files.fetch_add(v.len() as u64, Ordering::Relaxed) + v.len() as u64;
                prog.set_header(&format!("walking… {n}/{total} dir(s), {f} files"));
                prog.set_line(line, "idle"); // until rayon hands it the next subdir
                v
            })
            .collect()
    });
    drop(prog); // clear the area before run() prints the manifest/plan

    let mut out = root_files;
    for v in per_subtree {
        out.extend(v);
    }
    out
}

/// Copy one file, hashing its content in the same pass.
fn copy_one(src: &Path, harvested: &Path) -> ManifestEntry {
    let dest_name = crate::paths::dest_name(src);
    let dest_abs = harvested.join(&dest_name);
    let rel = format!("harvested/{dest_name}");

    let mut entry = ManifestEntry {
        source: src.to_string_lossy().into_owned(),
        dest: rel,
        size: 0,
        sha256: String::new(),
        copy_error: None,
    };
    match std::fs::read(src) {
        Ok(bytes) => {
            entry.size = bytes.len() as u64;
            let mut h = Sha256::new();
            h.update(&bytes);
            entry.sha256 = hex::encode(h.finalize());
            if let Err(e) = std::fs::write(&dest_abs, &bytes) {
                entry.copy_error = Some(format!("write dest: {e}"));
            }
        }
        Err(e) => entry.copy_error = Some(format!("read source: {e}")),
    }
    entry
}

pub fn run(args: &CrawlArgs, ctx: Ctx, corpus_dir: &Path) -> Result<i32> {
    if [args.all, args.sample.is_some(), args.pick]
        .iter()
        .filter(|b| **b)
        .count()
        != 1
    {
        eprintln!("error: exactly one of --all, --sample N, or --pick is required");
        return Ok(5);
    }
    // `--pick` is inherently interactive: fail fast (don't walk a slow
    // share first) when there's no terminal or `--no-input` forbids
    // prompting. Precedes the `tui`-feature check so an agent/CI run
    // gets the right "can't prompt" signal regardless of build.
    if args.pick && (ctx.no_input || !io::stdin().is_terminal() || !io::stdout().is_terminal()) {
        eprintln!(
            "error: --pick needs an interactive terminal{}",
            if ctx.no_input {
                " (--no-input is set)"
            } else {
                ""
            }
        );
        return Ok(5);
    }
    if !args.root.exists() {
        bail!("root not found / unreachable: {}", args.root.display());
    }

    // `walk()` owns its own progress now (a Spinner for --walk-jobs 1,
    // a multi-line area for >1) and clears it on return — no shared
    // spinner to thread/drop here.
    let skipped = AtomicU64::new(0);

    let (selected, selection_label): (Vec<PathBuf>, String) = if let Some(n) = args.sample {
        let mut rng = args.seed.map_or_else(Rng::from_time, Rng::seeded);
        let mut it = walk(
            &args.root,
            args.follow_links,
            args.max_bytes,
            &skipped,
            ctx.quiet,
            args.walk_jobs,
        );
        // Algorithm-R is order-sensitive, so a `--seed` sample must
        // see a STABLE path order — independent of `--walk-jobs` AND
        // of the filesystem's enumeration order (WalkDir never
        // guaranteed a portable order). Sort before sampling: same
        // seed ⇒ same sample, every run, every machine, any jobs.
        it.sort();
        (
            reservoir(it.into_iter(), n, &mut rng),
            format!("sample:{n}"),
        )
    } else {
        let all: Vec<PathBuf> = walk(
            &args.root,
            args.follow_links,
            args.max_bytes,
            &skipped,
            ctx.quiet,
            args.walk_jobs,
        );
        if args.pick {
            #[cfg(feature = "tui")]
            {
                (crate::tui::pick(&all)?, "pick".to_string())
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = all;
                eprintln!(
                    "error: --pick needs the `tui` build feature; rebuild with \
                     --features tui"
                );
                return Ok(5);
            }
        } else {
            (all, "all".to_string())
        }
    };

    if selected.is_empty() {
        eprintln!(
            "no .ags files selected (walked under {})",
            args.root.display()
        );
    }

    // --dry-run: walk + select happened (read-only), but copy nothing
    // and write no manifest. Stat the selected set (cheap vs a copy)
    // for a size preview so you can judge the cost before committing.
    if ctx.dry_run {
        // The size sweep is a `stat` per file — on a slow share that
        // was silent "dead air" after the walk. Show a bar, and fan
        // the stats out over the same pool the copy uses (sum + bucket
        // counts are order-independent → determinism untouched).
        let pb = progress_bar(selected.len() as u64, ctx.quiet);
        pb.set_message("sizing");
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(args.jobs.unwrap_or_else(|| {
                std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
            }))
            .build()
            .context("build stat thread pool")?;
        let (total, buckets): (u64, [u64; 4]) = pool.install(|| {
            selected
                .par_iter()
                .map(|p| {
                    let sz = std::fs::metadata(p).map_or(0, |m| m.len());
                    pb.inc(1);
                    // <1KiB, <1MiB, <10MiB, ≥10MiB (identical thresholds).
                    let b = if sz < 1 << 10 {
                        0
                    } else if sz < 1 << 20 {
                        1
                    } else if sz < 10 << 20 {
                        2
                    } else {
                        3
                    };
                    let mut bk = [0u64; 4];
                    bk[b] = 1;
                    (sz, bk)
                })
                .reduce(
                    || (0u64, [0u64; 4]),
                    |(s1, mut b1), (s2, b2)| {
                        for i in 0..4 {
                            b1[i] += b2[i];
                        }
                        (s1.saturating_add(s2), b1)
                    },
                )
        });
        pb.finish_and_clear();
        let mut sizes = serde_json::Map::new();
        sizes.insert("lt_1KiB".into(), buckets[0].into());
        sizes.insert("lt_1MiB".into(), buckets[1].into());
        sizes.insert("lt_10MiB".into(), buckets[2].into());
        sizes.insert("ge_10MiB".into(), buckets[3].into());
        let plan = Plan::new(
            "crawl",
            format!(
                "would copy {} file(s), {} from {} ({} walk skip(s))",
                selected.len(),
                human_bytes(total),
                args.root.display(),
                skipped.load(Ordering::Relaxed),
            ),
        )
        .with("selection", selection_label)
        .with("would_copy", selected.len() as u64)
        .with("total_bytes", total)
        .with("walk_skipped", skipped.load(Ordering::Relaxed))
        .with("size_buckets", serde_json::Value::Object(sizes));
        output::emit(&plan, &ctx)?;
        return Ok(0);
    }

    let harvested = corpus_dir.join("harvested");
    std::fs::create_dir_all(&harvested)
        .with_context(|| format!("create {}", harvested.display()))?;

    // Concurrent copy.
    let pb = progress_bar(selected.len() as u64, ctx.quiet);
    pb.set_message("copying");
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs.unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
        }))
        .build()
        .context("build copy thread pool")?;
    let entries: Vec<ManifestEntry> = pool.install(|| {
        selected
            .par_iter()
            .map(|src| {
                let e = copy_one(src, &harvested);
                pb.inc(1);
                e
            })
            .collect()
    });
    pb.finish_and_clear();

    let mut entries = entries;
    entries.sort_by(|a, b| a.dest.cmp(&b.dest));
    let copy_failures = entries.iter().filter(|e| e.copy_error.is_some()).count();

    let manifest = CrawlManifest {
        schema: SCHEMA,
        created: Utc::now().to_rfc3339(),
        root: args.root.to_string_lossy().into_owned(),
        selection: selection_label,
        walk_skipped: skipped.load(Ordering::Relaxed),
        files: entries,
    };
    // Run-versioned artifacts: a fresh (or --run-id) run dir under
    // <corpus>/runs/<id>/ so re-crawls don't clobber prior reports.
    // An explicit --manifest path still wins and stays out of runs/.
    let run_id = args.run_id.clone().unwrap_or_else(crate::paths::new_run_id);
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| crate::paths::run_dir(corpus_dir, &run_id).join("manifest.json"));
    manifest.save(&manifest_path)?;
    // Point runs/latest at this run so a later validate/parity finds
    // it with no flag — but only when we used the default run dir
    // (an explicit --manifest means the user manages paths).
    if args.manifest.is_none() {
        crate::paths::set_latest_run(corpus_dir, &run_id)?;
    }

    // manifest.json is the durable artifact; its location + a one-line
    // human summary are stderr hints, the manifest *document* is the
    // stdout payload (table = a counts summary; json/ndjson = the full
    // manifest, so `crawl --json | jq '.files'` works).
    if copy_failures > 0 {
        output::note(format!(
            "{copy_failures} copy error(s) — see manifest.copy_error"
        ));
    }
    if args.manifest.is_none() {
        output::note(format!("runs/latest → {run_id}"));
    }
    output::note(format!("manifest → {}", manifest_path.display()));
    output::emit(&manifest, &ctx)?;
    Ok(0)
}

/// A walk-progress label for `dir`: `d{N} {path}` where `N` is the
/// recursion depth (folders below the crawl `root`) and `path` is the
/// root-relative path, left-elided so the whole line fits the
/// terminal — keeping the deepest, changing tail. WHY: indicatif does
/// NOT width-truncate `set_message`, so an un-fitted long UNC path
/// (`\\srv\share\…`) smears the multi-line area. `cols` is the real
/// stderr width (`None` off a TTY → a generous fixed fallback; the
/// line isn't drawn there anyway).
fn walk_label(dir: &Path, root: &Path, cols: Option<usize>) -> String {
    let relp = dir.strip_prefix(root).unwrap_or(dir);
    let depth = relp.components().count();
    let rel = relp.display().to_string();
    let prefix = format!("d{depth} ");
    // Reserve: indicatif's `{spinner} ` (~2) + the depth prefix + a
    // right margin so the line never hugs the edge / wraps.
    let budget = cols
        .map_or(72, |c| c.saturating_sub(prefix.chars().count() + 3))
        .max(12);
    let n = rel.chars().count();
    if n <= budget {
        format!("{prefix}{rel}")
    } else {
        let tail: String = rel.chars().skip(n - (budget - 1)).collect();
        format!("{prefix}…{tail}")
    }
}

/// Compact human byte size (binary units), e.g. `6.2 GiB`. Local to
/// crawl — only the dry-run preview needs it.
fn human_bytes(n: u64) -> String {
    const U: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n} B")
    } else {
        format!("{v:.1} {}", U[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // (`reservoir`/`Rng` determinism is now unit-tested in
    // `laterite-ags4-parity` — `cargo test -p laterite-ags4-parity`. This module keeps
    // the *walk* determinism + the `--seed`-stable-across-jobs guard
    // that depends on the shared sampler.)

    #[test]
    fn parallel_walk_is_deterministic_and_seed_stable() {
        // A tree with subdirs so --walk-jobs >1 actually fans out.
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let mk = |rel: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b"x").unwrap();
        };
        for rel in [
            "top.ags",
            "z_sub/a.ags",
            "z_sub/deep/b.ags",
            "a_sub/c.ags",
            "a_sub/d.ags",
            "m_sub/e.ags",
            "m_sub/skip.txt", // non-.ags ignored
        ] {
            mk(rel);
        }
        let walk_n = |jobs: usize| {
            let sk = AtomicU64::new(0);
            // quiet=true → no progress object built (no-op gating).
            let mut v = walk(root, false, None, &sk, true, jobs);
            v.sort();
            v
        };
        // Same SET of files regardless of thread count (sorted — the
        // exact order crawl::run feeds the reservoir).
        let one = walk_n(1);
        let four = walk_n(4);
        assert_eq!(one.len(), 6, "6 .ags files discovered, .txt ignored");
        assert_eq!(one, four, "walk-jobs must not change the file set/order");
        // Parallel walk is itself stable run-to-run.
        assert_eq!(walk_n(4), walk_n(4));
        // The real --seed guarantee: identical sample across jobs.
        let s1 = reservoir(one.clone().into_iter(), 3, &mut Rng::seeded(42));
        let s4 = reservoir(four.clone().into_iter(), 3, &mut Rng::seeded(42));
        assert_eq!(s1, s4, "same seed ⇒ same sample at any --walk-jobs");
    }

    #[test]
    fn walk_label_shows_depth_and_fits_width_keeping_tail() {
        let root = Path::new("/r");
        // Depth = folders below root; root-relative path shown.
        assert_eq!(walk_label(Path::new("/r/a"), root, Some(80)), "d1 a");
        assert_eq!(
            walk_label(Path::new("/r/a/b/c"), root, Some(80)),
            "d3 a/b/c"
        );
        // Narrow width → left-elided, depth prefix kept, tail (the
        // changing/deepest part) preserved, whole thing within budget.
        let deep = Path::new("/r/aaaa/bbbb/cccc/dddd/eeee/ffff/gggg/hhhh");
        let lbl = walk_label(deep, root, Some(28));
        assert!(lbl.starts_with("d8 …"), "depth prefix + elision: {lbl:?}");
        assert!(lbl.ends_with("hhhh"), "keeps the deepest tail: {lbl:?}");
        assert!(lbl.chars().count() <= 28 - 3, "fits the width: {lbl:?}");
        // Unknown width (off a TTY) → generous fallback, no panic.
        assert!(walk_label(deep, root, None).starts_with("d8 "));
    }
}
