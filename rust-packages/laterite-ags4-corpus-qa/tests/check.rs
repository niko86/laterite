//! `check` answers exactly what `laterite-ags4-forge check` answers (#1109).
//!
//! The two-engine comparison is moving out of forge, and the claim that makes
//! the move safe is that nothing changed on the way: same verdicts, same
//! per-rule counts, same document. Both commands are thin calls into
//! `laterite_ags4_parity::check`, so this test runs the two BINARIES side by
//! side over vendored python-ags4 files and compares their whole output — the
//! one place a wiring difference (a flag dropped, a different oracle, a
//! different validator option) would show.
//!
//! The forge binary is read from beside this crate's own, which `cargo test
//! --workspace` (the CI test job) always builds in the same invocation. Run on
//! its own, this crate does not build forge, so a missing binary FAILS with the
//! command to build it rather than passing on half the comparison. This test
//! retires with forge's `check`.
//!
//! python-ags4 is optional QA here as everywhere else: with no oracle both
//! commands give a Rust-only verdict, and that still has to match. But a run
//! that compared only the Rust half says so on stderr, every time — a green
//! tick that checked half the claim must not look like one that checked all of
//! it.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn corpus_qa() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_laterite-ags4-corpus-qa"))
}

fn forge() -> PathBuf {
    let p = corpus_qa().with_file_name(format!(
        "laterite-ags4-forge{}",
        std::env::consts::EXE_SUFFIX
    ));
    assert!(
        p.exists(),
        "no forge binary at {} — this test compares against it. Build it \
         (`cargo build -p laterite-ags4-forge`, same profile) or run \
         `cargo test --workspace`, which builds both.",
        p.display()
    );
    p
}

fn vendored(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/pyags4-tests")
        .join(name)
}

/// A spread of shapes: a clean-ish example, a Rule 8 value fault, a Rule 10
/// key fault, and a Rule 19 naming fault — enough that a dropped validator
/// tier or a different oracle would move at least one verdict or count.
const FILES: &[&str] = &[
    "example1.ags",
    "4.1-rule8-1.ags",
    "4.1-rule10-1.ags",
    "4.1-rule19-1.ags",
];

fn run(bin: &Path, args: &[PathBuf]) -> (Option<i32>, Value) {
    let out = Command::new(bin)
        .arg("check")
        .args(args)
        .args(["--json", "--quiet"])
        .output()
        .unwrap_or_else(|e| panic!("run {}: {e}", bin.display()));
    let doc = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{} check printed no JSON document ({e}); stderr:\n{}",
            bin.display(),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code(), doc)
}

/// Straight to the process's stderr, not through `eprintln!`: libtest
/// captures the print macros and shows them only for a FAILING test, so an
/// `eprintln!` here would vanish on exactly the run it exists to qualify.
fn say_what_was_compared(doc: &Value, what: &str) {
    use std::io::Write;
    if doc["oracle"] == Value::Bool(false) {
        let _ = writeln!(
            std::io::stderr(),
            "NOTE ({what}): python-ags4 was unavailable, so only the Rust half \
             was compared — both commands gave Rust-only verdicts and the python \
             side of the claim went UNCHECKED on this run."
        );
    }
}

#[test]
fn a_sweep_matches_forge_verdict_for_verdict_and_count_for_count() {
    let files: Vec<PathBuf> = FILES.iter().map(|f| vendored(f)).collect();
    let (ours_code, ours) = run(&corpus_qa(), &files);
    let (theirs_code, theirs) = run(&forge(), &files);
    say_what_was_compared(&ours, "sweep");

    // The positive control: an empty sweep would match trivially.
    let entries = ours["files"].as_array().expect("files[]");
    assert_eq!(entries.len(), FILES.len(), "{ours:#}");
    assert!(
        entries.iter().any(|f| f["rust_rule_counts"]
            .as_object()
            .is_some_and(|m| !m.is_empty())),
        "no file raised a rule, so the counts went uncompared: {ours:#}"
    );

    assert_eq!(ours, theirs, "corpus-qa check and forge check disagree");
    assert_eq!(
        ours_code, theirs_code,
        "same verdicts, different exit codes"
    );
}

#[test]
fn a_single_file_keeps_forges_single_file_document() {
    let file = vec![vendored("4.1-rule8-1.ags")];
    let (ours_code, ours) = run(&corpus_qa(), &file);
    let (theirs_code, theirs) = run(&forge(), &file);
    say_what_was_compared(&ours, "single file");

    // Schema 2 is the one-file shape a caller reads `.verdict` straight off.
    assert_eq!(ours["schema"], 2, "{ours:#}");
    assert!(ours["verdict"].is_string(), "{ours:#}");
    assert_eq!(ours, theirs, "corpus-qa check and forge check disagree");
    assert_eq!(ours_code, theirs_code, "same verdict, different exit code");
}
