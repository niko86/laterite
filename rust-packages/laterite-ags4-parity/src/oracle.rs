//! `PyOracle` — the python-ags4 subprocess bridge.
//!
//! Spawns `uv run python <wrapper> …` with cwd = the repo root (so
//! `uv run` resolves the project's deps). The body of [`PyOracle::check`]
//! is the verbatim former `parity.rs::run_py`; [`PyOracle::selfcheck`]
//! is the verbatim former inline `--selfcheck` probe — both extracted
//! so `laterite-ags4-corpus-qa` and `laterite-ags4-forge` share one bridge. The
//! caller keeps the *policy* (warn/skip/fail on drift/unavailable);
//! this type is just the mechanism.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

/// Result of the `--selfcheck` probe.
#[derive(Debug, Clone)]
pub struct SelfCheck {
    /// The oracle's reported `python_ags4` version, or `None` when the
    /// `--selfcheck` JSON lacked / didn't parse it (the caller decides
    /// how loudly to warn — same as before).
    pub python_ags4: Option<String>,
}

/// The python bridge. `uv`/`wrapper`/`repo`/`timeout` are exactly the
/// former `run_py` parameters; held so one instance serves both the
/// startup self-check and every per-file check (shared across a rayon
/// pool by `&self` — `check` spawns its own `Command`).
pub struct PyOracle {
    uv: String,
    wrapper: PathBuf,
    repo: PathBuf,
    timeout: Duration,
}

impl PyOracle {
    #[must_use]
    pub fn new(uv: &str, wrapper: PathBuf, repo: PathBuf, timeout: Duration) -> Self {
        Self {
            uv: uv.to_string(),
            wrapper,
            repo,
            timeout,
        }
    }

    /// Probe `uv run python <wrapper> --selfcheck` once. Maps the
    /// former `run()` probe arms exactly: success → `Ok(SelfCheck)`
    /// (the caller compares `python_ags4` vs [`EXPECTED_PYAGS4`]),
    /// ran-but-non-zero → `NotImportable`, spawn-failed → `Unavailable`.
    pub fn selfcheck(&self) -> Result<SelfCheck, OracleError> {
        let probe = Command::new(&self.uv)
            .args(["run", "python"])
            .arg(&self.wrapper)
            .arg("--selfcheck")
            .current_dir(&self.repo)
            .stderr(Stdio::null())
            .output();
        match probe {
            Ok(o) if o.status.success() => {
                // The wrapper emits {"ok":true,"python_ags4":"<ver>"}.
                let v: serde_json::Value =
                    serde_json::from_slice(&o.stdout).unwrap_or(serde_json::Value::Null);
                let python_ags4 = v
                    .get("python_ags4")
                    .and_then(|s| s.as_str())
                    .map(std::string::ToString::to_string);
                Ok(SelfCheck { python_ags4 })
            }
            Ok(_) => Err(OracleError::NotImportable),
            Err(e) => Err(OracleError::Unavailable(e.to_string())),
        }
    }

    /// Run the wrapper on one file with the hard timeout. Returns the
    /// set of rule labels python flagged, or an error reason. (Verbatim
    /// body of the former `parity.rs::run_py`.)
    pub fn check(&self, target: &Path) -> Result<BTreeSet<String>, String> {
        // The child runs with cwd = repo root (so `uv run` resolves the
        // project's deps), so the target must be absolute — it's
        // relative to the *harness* cwd, not the repo.
        let target_abs = std::path::absolute(target).unwrap_or_else(|_| target.to_path_buf());
        let mut child = Command::new(&self.uv)
            .args(["run", "python"])
            .arg(&self.wrapper)
            .arg("--encoding-fallback")
            .arg(&target_abs)
            .current_dir(&self.repo)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn `{} run python`: {e}", self.uv))?;

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
                    if start.elapsed() >= self.timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        let _ = reader.join();
                        return Err("timeout".to_string());
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(format!("wait: {e}")),
            }
        };
        let out = reader.join().unwrap_or_default();

        let v: serde_json::Value = serde_json::from_str(out.trim())
            .map_err(|_| format!("non-JSON output (exit {:?})", status.code()))?;
        if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
            return Err(err.to_string());
        }
        // Keys are already filtered to "AGS Format Rule ..." by the
        // wrapper; a rule "fired" iff its array is non-empty.
        let mut set = BTreeSet::new();
        if let Some(obj) = v.as_object() {
            for (k, val) in obj {
                if val.as_array().is_some_and(|a| !a.is_empty()) {
                    set.insert(k.clone());
                }
            }
        }
        Ok(set)
    }
}
