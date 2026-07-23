//! End-to-end dry run: crawl + validate against the validator's own
//! hand-authored fixtures (no real network share needed). Asserts the
//! manifest/report shape and that the dogfood buckets are sane —
//! crucially, **zero panics** (a panic here is itself a real finding
//! the harness exists to surface).

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_laterite-ags4-corpus-qa")
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../laterite-ags4-validator/tests/fixtures")
}

/// Artifacts are now run-versioned under `<corpus>/runs/<id>/`;
/// resolve one via the `runs/latest` pointer crawl writes.
fn latest_artifact(corpus: &std::path::Path, name: &str) -> PathBuf {
    let id = std::fs::read_to_string(corpus.join("runs").join("latest"))
        .expect("runs/latest pointer")
        .trim()
        .to_string();
    corpus.join("runs").join(id).join(name)
}

#[test]
fn crawl_then_validate_dry_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let corpus = tmp.path();

    // --- crawl --all over the fixtures dir ---------------------------
    let st = Command::new(bin())
        .args(["crawl", "--all", "--quiet", "--root"])
        .arg(fixtures())
        .arg("--corpus-dir")
        .arg(corpus)
        .status()
        .expect("run crawl");
    assert_eq!(st.code(), Some(0), "crawl should exit 0");

    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(latest_artifact(corpus, "manifest.json")).expect("manifest.json"),
    )
    .expect("parse manifest");
    let files = manifest["files"].as_array().expect("files[]");
    assert!(
        files.len() >= 10,
        "expected the fixture corpus, got {}",
        files.len()
    );
    assert_eq!(manifest["selection"], "all");
    // Collision-safe dest names, all under harvested/.
    for f in files {
        let dest = f["dest"].as_str().unwrap();
        assert!(dest.starts_with("harvested/") && dest.contains("__"));
    }
    let has = |needle: &str| {
        files.iter().any(|f| {
            f["source"]
                .as_str()
                .unwrap()
                .replace('\\', "/")
                .ends_with(needle)
        })
    };
    assert!(has("clean_minimal.ags"));
    assert!(has("rule8_dt_bad.ags"));

    // --- validate ----------------------------------------------------
    let st = Command::new(bin())
        .args(["validate", "--quiet", "--corpus-dir"])
        .arg(corpus)
        .status()
        .expect("run validate");
    // The hand-authored fixtures are well-formed enough to only ever
    // produce *findings* (or be clean) — none hard-error or panic — so
    // the triage bucket is empty and validate exits 0. That the
    // validator never crashes on its own corpus is exactly the signal
    // we want.
    assert_eq!(st.code(), Some(0), "no triage expected on the fixtures");

    let report: Value = serde_json::from_str(
        &std::fs::read_to_string(latest_artifact(corpus, "report.json")).expect("report.json"),
    )
    .expect("parse report");

    // The single most important assertion: the validator never panics
    // (or hard-errors) on its own fixtures — a panic here = a real bug.
    assert_eq!(
        report["summary"]["panic"], 0,
        "validator panicked on a fixture"
    );
    assert_eq!(
        report["summary"]["hard_error"], 0,
        "unexpected hard error on a fixture"
    );
    assert!(
        report["summary"]["findings"].as_u64().unwrap() > 0,
        "expected the bad fixtures to produce findings"
    );

    let outcome_for = |needle: &str| -> String {
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| {
                f["source"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(needle)
            })
            .map(|f| f["outcome"]["kind"].as_str().unwrap().to_string())
            .unwrap_or_default()
    };
    assert_eq!(outcome_for("clean_minimal.ags"), "clean");
    assert_eq!(outcome_for("rule8_dt_bad.ags"), "findings");

    // Auto-selection: clean_minimal.ags declares TRAN_AGS "4.2", so
    // it must have been judged against the 4.2 dictionary.
    let dict_for = |needle: &str| -> String {
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| {
                f["source"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(needle)
            })
            .and_then(|f| f["dict_used"].as_str())
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(
        dict_for("clean_minimal.ags"),
        "4.2",
        "TRAN_AGS=4.2 fixture should auto-select the 4.2 dict"
    );

    // --- schema 2: dict_resolution + per-rule counts + clusters -----
    assert_eq!(report["schema"], 2, "report schema bumped to 2");

    // clean_minimal declares an exact bundled edition (4.2) → its
    // resolution is "exact", NOT the fallback (the O-31 distinction).
    let res_for = |needle: &str| -> String {
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| {
                f["source"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(needle)
            })
            .and_then(|f| f["dict_resolution"].as_str())
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(
        res_for("clean_minimal.ags"),
        "exact",
        "TRAN_AGS=4.2 is an exact bundled edition, not a fallback"
    );

    // Per-rule counts: outcome.rules is [[rule, count], …].
    let rule8_file = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| {
            f["source"]
                .as_str()
                .unwrap()
                .replace('\\', "/")
                .ends_with("rule8_dt_bad.ags")
        })
        .expect("rule8_dt_bad.ags in report");
    let rules = rule8_file["outcome"]["rules"].as_array().expect("rules[]");
    assert!(
        rules[0].is_array() && rules[0][1].is_u64(),
        "rules carry per-rule counts [rule,count], got {rules:?}"
    );

    // Clusters: present, and at least one groups the Rule-8 fixtures.
    let clusters = report["clusters"].as_array().expect("clusters[]");
    assert!(!clusters.is_empty(), "fixtures should form ≥1 cluster");
    assert!(
        clusters.iter().any(|cl| cl["signature"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().starts_with("AGS Format Rule 8"))),
        "expected a Rule 8 cluster, got {clusters:?}"
    );
}

/// The gogcli/CLI output contract: `--dry-run` mutates nothing,
/// `--output json` puts a parseable document on stdout, and
/// `--compact` drops the heavy per-file array while keeping the
/// summary. (stderr hints are not asserted — only stdout is the
/// machine contract.)
#[test]
fn dry_run_mutates_nothing_and_json_compact_modes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let corpus = tmp.path();
    let fx = fixtures();

    // 1. crawl --dry-run --output json: exit 0, JSON plan on stdout,
    //    and NOTHING written (no manifest, no harvested/).
    let out = Command::new(bin())
        .args([
            "crawl",
            "--all",
            "--dry-run",
            "--quiet",
            "--output",
            "json",
            "--root",
        ])
        .arg(&fx)
        .arg("--corpus-dir")
        .arg(corpus)
        .output()
        .expect("run crawl --dry-run");
    assert_eq!(out.status.code(), Some(0), "dry-run crawl exits 0");
    let v: Value = serde_json::from_slice(&out.stdout).expect("crawl --dry-run stdout is JSON");
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["action"], "crawl");
    assert!(
        v["would_copy"].as_u64().unwrap() >= 10,
        "would_copy counts the fixtures"
    );
    assert!(
        !corpus.join("runs").exists(),
        "dry-run must not create the runs/ tree (writes no manifest)"
    );
    assert!(
        !corpus.join("harvested").exists(),
        "dry-run must not create harvested/"
    );

    // 2. real crawl (writes the manifest), then validate --output json
    //    — the report document must be valid JSON on stdout.
    let st = Command::new(bin())
        .args(["crawl", "--all", "--quiet", "--root"])
        .arg(&fx)
        .arg("--corpus-dir")
        .arg(corpus)
        .status()
        .expect("run crawl");
    assert_eq!(st.code(), Some(0));
    assert!(latest_artifact(corpus, "manifest.json").exists());

    let out = Command::new(bin())
        .args(["validate", "--quiet", "--output", "json", "--corpus-dir"])
        .arg(corpus)
        .output()
        .expect("run validate --output json");
    assert_eq!(out.status.code(), Some(0), "no triage on the fixtures");
    let v: Value =
        serde_json::from_slice(&out.stdout).expect("validate --output json stdout is JSON");
    assert_eq!(v["schema"], 2);
    assert_eq!(v["summary"]["panic"], 0);
    assert!(v["summary"]["findings"].as_u64().unwrap() > 0);
    assert!(v["files"].is_array(), "non-compact keeps files[]");
    assert!(v["clusters"].is_array(), "non-compact has clusters[]");

    // 3. --compact drops files[] but keeps the summary AND clusters.
    let out = Command::new(bin())
        .args([
            "validate",
            "--quiet",
            "--output",
            "json",
            "--compact",
            "--corpus-dir",
        ])
        .arg(corpus)
        .output()
        .expect("run validate --compact");
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).expect("compact stdout is JSON");
    assert!(v.get("files").is_none(), "--compact must drop files[]");
    assert!(
        v["summary"]["findings"].as_u64().unwrap() > 0,
        "--compact keeps the summary"
    );
    assert!(
        v["clusters"].is_array(),
        "--compact keeps clusters[] (the token-lean high-signal view)"
    );

    // 4. validate --dry-run writes no report.json (mutate nothing).
    let fresh = tempfile::tempdir().expect("tempdir");
    let fc = fresh.path();
    let st = Command::new(bin())
        .args(["crawl", "--all", "--quiet", "--root"])
        .arg(&fx)
        .arg("--corpus-dir")
        .arg(fc)
        .status()
        .expect("run crawl (fresh)");
    assert_eq!(st.code(), Some(0));
    let out = Command::new(bin())
        .args([
            "validate",
            "--dry-run",
            "--quiet",
            "--output",
            "json",
            "--corpus-dir",
        ])
        .arg(fc)
        .output()
        .expect("run validate --dry-run");
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).expect("validate --dry-run JSON");
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["action"], "validate");
    assert!(
        !latest_artifact(fc, "report.json").exists(),
        "validate --dry-run must not write a report"
    );
}

/// `--seed` reproducibility is independent of `--walk-jobs`: a seeded
/// sample picks the same files single- and multi-threaded (the CLI
/// plumbing of the determinism the unit test proves on a subdir tree).
#[test]
fn seeded_sample_is_walk_jobs_invariant() {
    let fx = fixtures();
    let sources = |walk_jobs: &str| -> Vec<String> {
        let tmp = tempfile::tempdir().expect("tempdir");
        let st = Command::new(bin())
            .args([
                "crawl",
                "--sample",
                "5",
                "--seed",
                "42",
                "--quiet",
                "--walk-jobs",
                walk_jobs,
                "--root",
            ])
            .arg(&fx)
            .arg("--corpus-dir")
            .arg(tmp.path())
            .status()
            .expect("run crawl --sample");
        assert_eq!(st.code(), Some(0));
        let m: Value = serde_json::from_str(
            &std::fs::read_to_string(latest_artifact(tmp.path(), "manifest.json"))
                .expect("manifest.json"),
        )
        .expect("parse manifest");
        let mut s: Vec<String> = m["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["source"].as_str().unwrap().to_string())
            .collect();
        s.sort();
        s
    };
    let one = sources("1");
    assert_eq!(one.len(), 5, "sampled 5 of the fixtures");
    assert_eq!(
        one,
        sources("4"),
        "same --seed ⇒ same sample at --walk-jobs 1 vs 4"
    );
}
