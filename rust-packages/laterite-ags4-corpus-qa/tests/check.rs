//! `check`: the two-engine comparison over real files (#1109).
//!
//! `check` came here from forge's `check` subcommand. Until forge left this
//! workspace (#1111) this test ran the two binaries side by side and compared
//! their whole output; that comparison retired with forge's binary. What stays
//! is the shape a caller reads: a sweep reports every file and at least one
//! rule count, and a single file keeps the schema-2 document with `.verdict`
//! straight off it.
//!
//! python-ags4 is optional QA here as everywhere else: with no oracle the
//! verdict is Rust-only. A run that checked only the Rust half says so on
//! stderr, every time — a green tick that checked half the claim must not look
//! like one that checked all of it.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn corpus_qa() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_laterite-ags4-corpus-qa"))
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
             was checked — the verdicts are Rust-only and the python side of \
             the claim went UNCHECKED on this run."
        );
    }
}

#[test]
fn a_sweep_reports_every_file_and_its_rule_counts() {
    let files: Vec<PathBuf> = FILES.iter().map(|f| vendored(f)).collect();
    let (_code, ours) = run(&corpus_qa(), &files);
    say_what_was_compared(&ours, "sweep");

    // The positive control: an empty sweep would match trivially.
    let entries = ours["files"].as_array().expect("files[]");
    assert_eq!(entries.len(), FILES.len(), "{ours:#}");
    assert!(
        entries.iter().any(|f| f["rust_rule_counts"]
            .as_object()
            .is_some_and(|m| !m.is_empty())),
        "no file raised a rule, so the counts went unreported: {ours:#}"
    );
}

#[test]
fn a_single_file_keeps_the_single_file_document() {
    let file = vec![vendored("4.1-rule8-1.ags")];
    let (_code, ours) = run(&corpus_qa(), &file);
    say_what_was_compared(&ours, "single file");

    // Schema 2 is the one-file shape a caller reads `.verdict` straight off.
    assert_eq!(ours["schema"], 2, "{ours:#}");
    assert!(ours["verdict"].is_string(), "{ours:#}");
}
