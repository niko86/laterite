//! The oracle runs where its CALLER says, not where this crate was built.
//!
//! `build_oracle` used to anchor on this crate's `CARGO_MANIFEST_DIR` and walk
//! up to the repo root for the wrapper script and the uv project. That holds
//! only while the crate is compiled from this repo's tree: from crates.io the
//! manifest dir is a registry cache, and the walk lands nowhere (#1108). So the
//! wrapper and the project are the caller's to pass, and these tests pass ones
//! this repo does not have.

use std::path::{Path, PathBuf};
use std::time::Duration;

use laterite_ags4_parity::PyOracle;
use laterite_ags4_parity::check::build_oracle;

/// A stand-in for `uv` that reports where it was run and what it was handed,
/// so the test reads back the routing rather than python-ags4's answer. Its
/// argv is `run python <wrapper> <flag> [target]`, exactly what the oracle
/// spawns.
#[cfg(unix)]
fn fake_uv(dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("fake-uv");
    std::fs::write(
        &script,
        "#!/bin/sh\n\
         if [ \"$4\" = \"--selfcheck\" ]; then\n\
         \x20 printf '{\"ok\":true,\"python_ags4\":\"%s|%s\"}' \"$(pwd -P)\" \"$3\"\n\
         else\n\
         \x20 printf '{\"%s\":[1],\"%s\":[1,2]}' \"$(pwd -P)\" \"$3\"\n\
         fi\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[cfg(unix)]
#[test]
fn the_oracle_runs_the_callers_wrapper_in_the_callers_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project = tmp.path().join("elsewhere-project");
    std::fs::create_dir(&project).unwrap();
    // Not `tools/py_ags4_check_json.py`, and not under any repo: a default
    // that leaked through would show up as a different path below.
    let wrapper = tmp.path().join("my_wrapper.py");
    let uv = fake_uv(tmp.path());
    let oracle = PyOracle::new(
        uv.to_str().unwrap(),
        wrapper.clone(),
        project.clone(),
        Duration::from_secs(10),
    );

    let project_real = project.canonicalize().unwrap();
    let expected = format!("{}|{}", project_real.display(), wrapper.display());
    let sc = oracle.selfcheck().expect("fake uv runs");
    assert_eq!(sc.python_ags4.as_deref(), Some(expected.as_str()));

    // The per-file run takes the same route. The fake echoes the cwd and the
    // wrapper back as the two "rules" it found.
    let target = tmp.path().join("delivery.ags");
    std::fs::write(&target, b"x").unwrap();
    let counts = oracle.check_counts(&target).expect("fake uv runs");
    assert_eq!(counts.get(&project_real.display().to_string()), Some(&1));
    assert_eq!(counts.get(&wrapper.display().to_string()), Some(&2));
}

#[test]
fn a_wrapper_the_caller_does_not_have_is_no_oracle() {
    // The caller's values are the only ones consulted: pointed at a project
    // with no wrapper in it, there is no oracle — never a silent fallback to
    // this repo's own script.
    let tmp = tempfile::tempdir().expect("tempdir");
    let wrapper = tmp.path().join("absent.py");
    assert!(build_oracle(wrapper, tmp.path().to_path_buf(), 10).is_none());
}
