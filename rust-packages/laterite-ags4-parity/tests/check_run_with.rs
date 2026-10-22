//! `check::run_with` takes the caller's oracle and never probes for one (#1157).
//!
//! A caller that probes python-ags4 itself, to say why it is missing and how to
//! fix it, used to pay for a second self-check inside `check::run`. These tests
//! read the probe count back from a fake `uv` that logs each invocation.

use std::path::{Path, PathBuf};
use std::time::Duration;

use laterite_ags4_parity::PyOracle;
use laterite_ags4_parity::check::run_with;
use laterite_cliutil::OutputMode;
use laterite_cliutil::report::Ctx;

fn ctx() -> Ctx {
    Ctx {
        mode: OutputMode::Json,
        quiet: true,
        dry_run: false,
        no_input: false,
        compact: true,
        no_color: true,
    }
}

fn two_files(dir: &Path) -> Vec<PathBuf> {
    ["a.ags", "b.ags"]
        .iter()
        .map(|name| {
            let p = dir.join(name);
            std::fs::write(&p, "\"GROUP\",\"PROJ\"\r\n").unwrap();
            p
        })
        .collect()
}

/// A stand-in for `uv` that appends the flag it was run with (`--selfcheck` or
/// `--encoding-fallback`) to `log`, then answers as the wrapper would.
#[cfg(unix)]
fn logging_uv(dir: &Path, log: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("logging-uv");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\n\
             echo \"$4\" >> '{}'\n\
             if [ \"$4\" = \"--selfcheck\" ]; then\n\
             \x20 printf '{{\"ok\":true,\"python_ags4\":\"1.2.0\"}}'\n\
             else\n\
             \x20 printf '{{}}'\n\
             fi\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[cfg(unix)]
#[test]
fn a_given_oracle_is_used_without_a_second_selfcheck() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("calls.log");
    let uv = logging_uv(tmp.path(), &log);
    let oracle = PyOracle::new(
        uv.to_str().unwrap(),
        tmp.path().join("wrapper.py"),
        tmp.path().to_path_buf(),
        Duration::from_secs(30),
    );
    let files = two_files(tmp.path());

    run_with(&files, Some(&oracle), &ctx()).expect("report written");

    let calls = std::fs::read_to_string(&log).unwrap_or_default();
    let calls: Vec<&str> = calls.lines().collect();
    assert!(
        !calls.contains(&"--selfcheck"),
        "run_with probed an oracle it was handed: {calls:?}"
    );
    assert_eq!(
        calls
            .iter()
            .filter(|c| **c == "--encoding-fallback")
            .count(),
        files.len(),
        "each file should still be checked by python-ags4: {calls:?}"
    );
}

#[test]
fn no_oracle_is_a_rust_only_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let files = two_files(tmp.path());
    // Nothing to spawn: a None oracle never reaches for uv at all.
    let code = run_with(&files, None, &ctx()).expect("report written");
    assert!(code == 0 || code == 1, "a sweep exits 0 or 1, got {code}");
}

#[test]
fn a_missing_path_is_exit_3() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let missing = tmp.path().join("absent.ags");
    assert_eq!(run_with(&[missing], None, &ctx()).expect("no write"), 3);
}
