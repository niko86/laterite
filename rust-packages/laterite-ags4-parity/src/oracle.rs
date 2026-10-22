//! `PyOracle` — the python-ags4 subprocess bridge.
//!
//! Spawns `uv run python <wrapper> …` with cwd = the caller's uv project
//! (so `uv run` resolves that project's deps). [`PyOracle::check`] began as
//! the former `parity.rs::run_py` and [`PyOracle::selfcheck`] as the former
//! inline `--selfcheck` probe — both extracted so `laterite-ags4-corpus-qa`
//! and `laterite-ags4-forge` share one bridge, and both now run through one
//! timeout-bounded spawn (#1158). The
//! caller keeps the *policy* (warn/skip/fail on drift/unavailable);
//! this type is just the mechanism.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// The python-ags4 release the parity catalogue (OBSERVATIONS
/// O-1..O-34 + the `reconcile` arms in [`crate::verdict`]) is encoded
/// against. The `--selfcheck` probe reads the live oracle version and
/// the caller warns loudly on drift: a silent minor bump (e.g. fixing
/// the `rule_6` no-op, or the `rule_20` on-disk behaviour) would
/// invalidate a reconcile arm with **no test failure**. Pinned in
/// `pyproject.toml`; bump deliberately and re-probe. See
/// `ags-wiki/insights/oracle-drift-pin`.
pub const EXPECTED_PYAGS4: &str = "1.2.0";

/// How long the self-check may run before the oracle counts as unavailable
/// (#1158). Its own bound, not the per-file one: on a fresh checkout the
/// first `uv run` installs the project before python starts, and CI's first
/// probe outlasted the per-file timeout and dropped a sweep to Rust-only.
/// Generous so an install fits; finite so a real hang still ends.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Why the oracle couldn't be consulted at all (the caller maps these
/// to its own "skip — optional QA, exit 0" policy verbatim).
#[derive(Debug)]
pub enum OracleError {
    /// `uv`/python ran but python-ags4 isn't importable (probe exited
    /// non-zero) — the former `Ok(_) =>` arm.
    NotImportable,
    /// The `uv` executable itself couldn't be spawned — the former
    /// `Err(e) =>` arm; carries the spawn error text.
    Unavailable(String),
}

/// Why a bounded run produced no output; each caller maps it to its own error.
enum Bounded {
    Spawn(std::io::Error),
    Timeout,
    Wait(std::io::Error),
}

/// Result of the `--selfcheck` probe.
#[derive(Debug, Clone)]
pub struct SelfCheck {
    /// The oracle's reported `python_ags4` version, or `None` when the
    /// `--selfcheck` JSON lacked / didn't parse it (the caller decides
    /// how loudly to warn — same as before).
    pub python_ags4: Option<String>,
}

/// The python bridge. `uv`/`wrapper`/`project`/`timeout` are exactly the
/// former `run_py` parameters; held so one instance serves both the
/// startup self-check and every per-file check (shared across a rayon
/// pool by `&self` — `check` spawns its own `Command`).
pub struct PyOracle {
    uv: String,
    wrapper: PathBuf,
    project: PathBuf,
    timeout: Duration,
    probe_timeout: Duration,
}

impl PyOracle {
    /// `wrapper` is the python script to run and `project` the uv project
    /// whose environment runs it (the child's cwd). Nothing here defaults
    /// either one: a published crate has no repo of its own to find them in
    /// (#1108), so they are whatever the caller passes.
    #[must_use]
    pub fn new(uv: &str, wrapper: PathBuf, project: PathBuf, timeout: Duration) -> Self {
        Self {
            uv: uv.to_string(),
            wrapper,
            project,
            timeout,
            probe_timeout: DEFAULT_PROBE_TIMEOUT,
        }
    }

    /// Bound the self-check by `timeout` instead of [`DEFAULT_PROBE_TIMEOUT`].
    #[must_use]
    pub fn with_probe_timeout(mut self, timeout: Duration) -> Self {
        self.probe_timeout = timeout;
        self
    }

    /// Probe `uv run python <wrapper> --selfcheck` once. Maps the
    /// former `run()` probe arms exactly: success → `Ok(SelfCheck)`
    /// (the caller compares `python_ags4` vs [`EXPECTED_PYAGS4`]),
    /// ran-but-non-zero → `NotImportable`, spawn-failed → `Unavailable`.
    ///
    /// Bounded by the probe timeout (#1158), so an expiry is `Unavailable`
    /// rather than a hang.
    pub fn selfcheck(&self) -> Result<SelfCheck, OracleError> {
        match self.run_bounded(&[OsStr::new("--selfcheck")], self.probe_timeout) {
            Ok((status, out)) if status.success() => {
                // The wrapper emits {"ok":true,"python_ags4":"<ver>"}.
                let v: serde_json::Value =
                    serde_json::from_str(&out).unwrap_or(serde_json::Value::Null);
                let python_ags4 = v
                    .get("python_ags4")
                    .and_then(|s| s.as_str())
                    .map(std::string::ToString::to_string);
                Ok(SelfCheck { python_ags4 })
            }
            Ok(_) => Err(OracleError::NotImportable),
            Err(Bounded::Spawn(e)) => Err(OracleError::Unavailable(e.to_string())),
            Err(Bounded::Timeout) => Err(OracleError::Unavailable(format!(
                "self-check timed out after {:?}",
                self.probe_timeout
            ))),
            Err(Bounded::Wait(e)) => Err(OracleError::Unavailable(format!("wait: {e}"))),
        }
    }

    /// `uv run python <wrapper> <args…>` in the caller's project, stdout
    /// captured, bounded by `timeout`.
    ///
    /// On expiry the child is killed and its stdout reader is left behind
    /// rather than joined: `uv run`'s python is a grandchild that outlives
    /// the kill and holds the pipe open, so a join would wait on it anyway.
    fn run_bounded(
        &self,
        args: &[&OsStr],
        timeout: Duration,
    ) -> Result<(ExitStatus, String), Bounded> {
        let mut child = Command::new(&self.uv)
            .args(["run", "python"])
            .arg(&self.wrapper)
            .args(args)
            .current_dir(&self.project)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(Bounded::Spawn)?;

        // Read stdout on a thread so a large/slow writer can't deadlock
        // the timeout poll.
        let mut stdout = child.stdout.take().expect("piped stdout");
        let reader = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stdout.read_to_string(&mut s);
            s
        });

        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break st,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(Bounded::Timeout);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(Bounded::Wait(e)),
            }
        };
        Ok((status, reader.join().unwrap_or_default()))
    }

    /// Run the wrapper on one file with the hard timeout. Returns the
    /// set of rule labels python flagged, or an error reason.
    ///
    /// Presence only — what the verdict is allowed to see. A caller that
    /// wants to *show* how many times each rule fired asks
    /// [`Self::check_counts`]; both run python exactly once.
    pub fn check(&self, target: &Path) -> Result<BTreeSet<String>, String> {
        Ok(self.check_counts(target)?.into_keys().collect())
    }

    /// The same run, keeping the per-rule finding counts python reported.
    ///
    /// The counts are for a reader, never for `classify` — the two
    /// validators split one defect across rules differently (O-11/O-16/
    /// O-22/O-26), so a count comparison manufactures divergences the
    /// presence model exists to avoid. Observable, never judged.
    pub fn check_counts(&self, target: &Path) -> Result<BTreeMap<String, u64>, String> {
        // The child runs with cwd = the uv project (so `uv run` resolves
        // its deps), so the target must be absolute — it's relative to
        // the *harness* cwd, not the project.
        let target_abs = std::path::absolute(target).unwrap_or_else(|_| target.to_path_buf());
        let (status, out) = self
            .run_bounded(
                &[OsStr::new("--encoding-fallback"), target_abs.as_os_str()],
                self.timeout,
            )
            .map_err(|e| match e {
                Bounded::Spawn(e) => format!("spawn `{} run python`: {e}", self.uv),
                Bounded::Timeout => "timeout".to_string(),
                Bounded::Wait(e) => format!("wait: {e}"),
            })?;

        let v: serde_json::Value = serde_json::from_str(out.trim())
            .map_err(|_| format!("non-JSON output (exit {:?})", status.code()))?;
        if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
            return Err(err.to_string());
        }
        // Keys are already filtered to "AGS Format Rule ..." by the
        // wrapper; a rule "fired" iff its array is non-empty, and the
        // array's length is how many times it fired.
        let mut counts = BTreeMap::new();
        if let Some(obj) = v.as_object() {
            for (k, val) in obj {
                let n = val.as_array().map_or(0, Vec::len);
                if n > 0 {
                    counts.insert(k.clone(), u64::try_from(n).unwrap_or(u64::MAX));
                }
            }
        }
        Ok(counts)
    }
}
