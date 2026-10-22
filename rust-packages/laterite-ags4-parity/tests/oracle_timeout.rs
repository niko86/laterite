//! A self-check that never answers is bounded by the oracle's own timeout (#1158).
//!
//! The probe is the first thing every caller runs, and on a fresh checkout it is
//! also the run that builds the venv, so it is the one most likely to stall. It
//! used to wait on the child with no bound, and a hung `uv run` hung its caller.

#![cfg(unix)]

use std::path::Path;
use std::time::{Duration, Instant};

use laterite_ags4_parity::{OracleError, PyOracle};

/// A stand-in for `uv` that sleeps instead of answering. The sleep is a child of
/// the shell, not an `exec`: like the python `uv run` starts, it outlives a kill
/// of its parent and keeps the stdout pipe open.
fn stalled_uv(dir: &Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("stalled-uv");
    std::fs::write(&script, "#!/bin/sh\nsleep 20\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[test]
fn a_selfcheck_that_never_answers_is_unavailable_after_the_timeout() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let uv = stalled_uv(tmp.path());
    let oracle = PyOracle::new(
        uv.to_str().unwrap(),
        tmp.path().join("wrapper.py"),
        tmp.path().to_path_buf(),
        Duration::from_secs(1),
    );

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
