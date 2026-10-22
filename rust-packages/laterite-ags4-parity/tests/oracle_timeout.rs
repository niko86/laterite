//! A self-check that never answers is bounded by the oracle's probe timeout (#1158).
//!
//! The probe is the first thing every caller runs, and on a fresh checkout it is
//! also the run that installs the project, so it is the one most likely to stall.
//! It used to wait on the child with no bound, and a hung `uv run` hung its caller.
//! It is bounded by its own timeout, not the per-file one: that install is
//! slower than any per-file run, and CI's first probe outlasted the per-file
//! timeout and dropped a whole sweep to Rust-only.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use laterite_ags4_parity::{OracleError, PyOracle};

/// A stand-in for `uv` running `body`. The sleeps in it are children of the
/// shell, not an `exec`: like the python `uv run` starts, they outlive a kill of
/// their parent and keep the stdout pipe open.
fn fake_uv(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("fake-uv");
    std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

fn oracle(dir: &Path, uv: &Path, per_file: Duration) -> PyOracle {
    PyOracle::new(
        uv.to_str().unwrap(),
        dir.join("wrapper.py"),
        dir.to_path_buf(),
        per_file,
    )
}

#[test]
fn a_selfcheck_that_never_answers_is_unavailable_after_the_probe_timeout() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let uv = fake_uv(tmp.path(), "sleep 20");
    // A per-file timeout the probe must NOT be using: the run ends at the
    // probe's second, not this minute.
    let oracle =
        oracle(tmp.path(), &uv, Duration::from_secs(60)).with_probe_timeout(Duration::from_secs(1));

    let start = Instant::now();
    let got = oracle.selfcheck();
    let took = start.elapsed();

    match got {
        Err(OracleError::Unavailable(why)) => {
            assert!(
                why.contains("timed out"),
                "reason does not name the timeout: {why}"
            );
        }
        other => panic!("a stalled self-check should be Unavailable, got {other:?}"),
    }
    assert!(
        took < Duration::from_secs(10),
        "the self-check waited {took:?} for a child it should have abandoned after 1s"
    );
}

#[test]
fn a_selfcheck_slower_than_a_file_still_answers() {
    // The CI shape: the first `uv run` installs the project before python
    // starts, so the probe outlasts the per-file timeout and is still healthy.
    let tmp = tempfile::tempdir().expect("tempdir");
    let uv = fake_uv(
        tmp.path(),
        "sleep 2\nprintf '{\"ok\":true,\"python_ags4\":\"1.2.0\"}'",
    );
    let oracle = oracle(tmp.path(), &uv, Duration::from_secs(1));

    let sc = oracle
        .selfcheck()
        .expect("a probe within the probe timeout answers");
    assert_eq!(sc.python_ags4.as_deref(), Some("1.2.0"));
}
