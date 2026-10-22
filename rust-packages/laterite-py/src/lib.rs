//! PyO3 module `laterite._laterite_native`.
//!
//! Thin boundary: every bit of AGS4 logic lives in `laterite-ags4-validator`
//! (clean-room, parity-tested) or the local `emit` module. The Python
//! side (`laterite/__init__.py`, `compat.py`, `_cli.py`) builds the
//! python-ags4-shaped dict from the primitives `parse_primitives`
//! returns; the read path's frames come born-typed from the Arrow
//! tables `parse_arrow` hands over (pyo3-arrow capsule, zero-copy).
//!
//! Two error-JSON shapes are deliberately preserved (see the package
//! README): the Rust-CLI shape `{file, findings:{rule:[...]}}` comes from
//! the ENGINE's own renderer (`laterite_ags4_validator::findings`) — the
//! same function the `lat` binary calls, so `--json`/`--ndjson` are
//! byte-faithful by construction rather than by a hand-copy kept in step
//! with a comment (laterite-dev#530); the python-ags4 `check_file` dict (with
//! `Metadata`/`Summary`) is assembled in `laterite/compat.py`.

use std::collections::HashSet;
use std::path::Path;

// The read/validate hot-path is allocation-bound in the parse leaf (~5M small
// allocations for a 25 MB file — dhat, perf-campaign T4-followup), so the process
// allocator's per-alloc cost dominates. mimalloc's per-thread heaps cut this
// wheel's end-to-end read ~22% and validate ~14% for +163 KB on the `.so`
// (measured). Set here because only a final cdylib can choose the allocator; the
// engine leaves the Rust boundary via Arrow release callbacks, so Rust frees what
// Rust allocated (no cross-allocator handoff to polars/pyarrow).
//
// Handoff was never the hazard — CO-RESIDENCY was (#294). We load into processes
// that already contain another mimalloc (pyarrow bundles one as its default
// pool; CPython 3.14 vendors one too), and mimalloc v3 in that situation hands
// out memory the other instance is already using. The version pin that avoids
// it is on the dep in Cargo.toml;
// keep it there, and do not assume this comment's "Rust frees what Rust
// allocated" makes a second allocator in the process safe.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use laterite_ags4_core::error::CliError;
use laterite_ags4_core::index::{Sidecar as CoreSidecar, TierCoverage};
use laterite_ags4_validator::findings::{Severity, Target};
use laterite_ags4_validator::{
    CheckOptions, CodeRewrite, DictVersion, Dictionary, Findings, Fix, FixError, SkippedRewrite,
    ValidatorError, WorldScope, fix_document_recoding, overlay,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3_arrow::PyTable;

// S3b (release/v0.1.0-prep): the DuckDB-bound function module moved to
// a separate cdylib. Base wheel ships without DuckDB; the modules
// below are the ones that stay in it.
mod ags_types_fns;
mod emit_typed;
mod excel_fns;
mod registry_fns;
mod transport_fns;
mod typed_graph;

/// Map an `laterite-ags4-core` `CliError` to a `PyRuntimeError`, preserving the
/// exit-code label python callers may surface in error messages.
/// Previously lived in a DuckDB-bound function module; that module
/// moved to a separate cdylib in S3b, so the base-wheel transport +
/// excel functions share this local copy instead.
pub(crate) fn map_cli_err(e: &CliError) -> PyErr {
    let code = e.exit_code();
    PyRuntimeError::new_err(format!("laterite error (exit {code}): {e}"))
}

/// Map a `--dict-version` string to the optional override. `None` /
/// `"auto"` ⇒ no override (`TRAN_AGS` auto-pick). Unknown ⇒ `Err`.
/// The parse is `hostopts` (#923) — one copy per workspace, not per surface.
pub(crate) fn parse_dv(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    laterite_ags4_hostopts::edition(s).map_err(|e| e.message)
}

/// Parse the `--dict` custom-dictionary override for a Python call, mirroring the CLI's
/// `apply_dict_args` (laterite-dev#568). The dict arrives as a filesystem path OR raw bytes (wasm has
/// no FS, so bytes is the portable spelling every surface shares); the base edition is a
/// property of the dict, detected structurally, unless the caller forces one via
/// `--dict-version` (`over`) or drops it entirely via `dict_replace`. `enc` is the caller's
/// already-resolved source encoding (the same one it hands `CheckOptions`), so the label is
/// resolved once per call.
///
/// Returns `Ok(None)` when no dict was named. Errors use the same `(5, "bad_dict", msg)`
/// shape as the rest of this file so the Python layer raises the mapped exception.
fn build_custom_dict(
    dict_path: Option<&str>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
) -> Result<Option<overlay::CustomDict>, (i32, String, String)> {
    // The ladder is `hostopts` (#923) — one copy per workspace. What stays
    // here is this surface's spelling of the knobs, so an error names what a
    // Python caller actually typed.
    laterite_ags4_hostopts::custom_dict(
        dict_path.map(Path::new),
        dict_bytes,
        dict_replace,
        over,
        enc,
        laterite_ags4_hostopts::DictFlags {
            source: "--dict",
            replace: "dict_replace",
            version: "dict_version",
        },
    )
    .map_err(|e| (e.code, e.kind.to_string(), e.message))
}

/// (`exit_code`, `error_kind`, message) for a validator error — exit codes
/// mirror the `lat` binary exactly (3 not-found/io, 4
/// not-utf8/not-ags4/unsupported-edition, 5 bad-dict).
fn map_err(e: &ValidatorError) -> (i32, String, String) {
    // Delegate to the single producers so codes/kinds can't drift.
    (e.exit_code(), e.kind().to_string(), e.to_string())
}

/// A `TierCoverage` as Python sees it: the count, or `None` if the tier was never run.
fn tier_count(c: TierCoverage) -> Option<u32> {
    match c {
        TierCoverage::NotMeasured => None,
        TierCoverage::Measured { count } => Some(count),
    }
}

/// Rule-engine runs through this module's validate and mint doors, process-wide.
///
/// It exists so a test can see a run that happens entirely in Rust: the mint reuses a
/// handle's Report instead of running the engine again (#1096), and neither a Python
/// mock nor a timing can show that the run was skipped. It counts the two doors that
/// question is about and nothing else: `fix_file`, `diff_files` and the compat shim run
/// the engine without it.
static ENGINE_RUNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn count_engine_run() {
    ENGINE_RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// How many times the validate and mint doors have run the rule engine in this process.
/// A test hook; see [`ENGINE_RUNS`] for what it does not count.
#[pyfunction]
fn engine_runs() -> u64 {
    ENGINE_RUNS.load(std::sync::atomic::Ordering::Relaxed)
}

/// What a validation produced, plus whether a certificate stood in for the rule engine.
struct Checked {
    file: String,
    dict_version: String,
    resolution: String,
    findings: Findings,
    /// The rules `max_findings_per_rule` cut (#1089); empty when nothing was.
    truncated: Truncated,
    /// A cert answered the CONTENT half. The world half (Rule 20 on-disk) ran regardless.
    certified: bool,
    /// If a certificate was offered and NOT used, the stable token for why (the core
    /// `RevalidateReason::as_str`). `None` when no cert was offered, or it was vouched.
    revalidate_reason: Option<String>,
    /// This run as a mint can reuse it (#1096): `Some` only when the engine ran over the
    /// handle's retained parse with every tier on and no world check.
    measured: Option<laterite_ags4_trust::Measurement>,
}

/// Run the validator from a path, in-memory text, or raw bytes — through the ONE door.
///
/// The three modalities used to be three code paths that each assembled the engine call
/// themselves, and they did not agree (the bytes path skipped the O-42 edition guard;
/// `check_files` evaporated silently on text and bytes). They are now one call: read to
/// bytes, name the world you have, and hand it to `trust::check`. A path gets
/// `WorldScope::OnDisk`; text and bytes get `None`, and if they ask for `check_files`
/// anyway they are REFUSED rather than told the file is clean.
#[allow(clippy::too_many_arguments)] // one door for three modalities; the knobs are lat's
fn validate(
    path: Option<&str>,
    text: Option<&str>,
    data: Option<&[u8]>,
    dvr: Option<&str>,
    warnings: bool,
    fyi: bool,
    check_files: bool,
    encoding: Option<&str>,
    dict_path: Option<&str>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    cert: Option<&CoreSidecar>,
    // Did the CALLER name this certificate at THIS call? A cert reached through
    // `read(index=…)` and carried on the handle is a hint — the trust model
    // declines it and says why. A cert named on `validate(index=…)` is an
    // ASSERTION that it belongs to these bytes, so a mismatch is an error.
    // Same distinction `read` already draws, one layer down so the check can use
    // bytes that are already in hand instead of reading the file a second time.
    strict_cert: bool,
    // The parse a read handle already holds of this source, under this decoder
    // (#1090). With a cert, used only once the bytes are shown to match it (#1131):
    // a cert is checked against BYTES, and the parse alone cannot show it any.
    retained: Option<&laterite_ags4_parse::ParsedFile>,
    max_findings_per_rule: Option<usize>,
) -> Result<Checked, (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let custom_dict = build_custom_dict(dict_path, dict_bytes, dict_replace, over, enc)?;
    let mut opts = CheckOptions::default();
    opts.dict_version = over;
    opts.custom_dict = custom_dict;
    opts.include_warnings = warnings;
    opts.include_fyi = fyi;
    opts.check_files = check_files;
    opts.encoding = enc;
    opts.max_findings_per_rule = max_findings_per_rule;

    if let (Some(parsed), None) = (retained, cert) {
        let (label, world) = match (path, text) {
            (Some(p), _) => (
                p.to_string(),
                WorldScope::OnDisk(std::path::PathBuf::from(p)),
            ),
            (None, Some(_)) => ("<text>".to_string(), WorldScope::None),
            (None, None) => ("<bytes>".to_string(), WorldScope::None),
        };
        let (out, measured) = laterite_ags4_trust::check_parsed_measured(parsed, &opts, world)
            .map_err(|e| map_err(&e))?;
        count_engine_run();
        return Ok(Checked {
            file: label,
            dict_version: out.dict_version.as_str().to_string(),
            resolution: out.resolution.as_str().to_string(),
            findings: out.findings,
            truncated: out.truncated,
            certified: out.certified,
            revalidate_reason: None,
            measured,
        });
    }

    let io_err = |p: &str, e: &std::io::Error| {
        let kind = if e.kind() == std::io::ErrorKind::NotFound {
            "not_found"
        } else {
            "io"
        };
        (3, kind.to_string(), format!("{p}: {e}"))
    };
    let stale = |label: &str| {
        (
            4,
            "stale_cert".to_string(),
            format!(
                "the certificate does not match {label} (size / SHA-256 differ) \
                 — the file changed under it; rebuild it with \
                 read(...).validate().certify()"
            ),
        )
    };

    // A handle that carries a cert, and whose question the cert cannot answer for any
    // bytes, runs the engine whatever the hash says (#1131). Matching bytes are the ones
    // read() checked the cert against, and so the ones the handle parsed: run the engine
    // on that parse, as the cert-less branch above does, and the run is a Measurement a
    // later mint can reuse. Hashing is all the bytes are needed for, so a path streams.
    // A cert that would vouch, or bytes that no longer match, take the doors below,
    // which say why in `decide`'s own order.
    if let (Some(parsed), Some(c)) = (retained, cert) {
        let world = path.map_or(WorldScope::None, |p| {
            WorldScope::OnDisk(std::path::PathBuf::from(p))
        });
        let answer = laterite_ags4_trust::split_options(&opts, world.clone())
            .map(|(q, _)| c.answers(&q, &laterite_ags4_trust::engine_id(None)));
        if let Ok(laterite_ags4_trust::Decision::Revalidate(reason)) = answer {
            let fresh = match (path, text, data) {
                (Some(p), _, _) => {
                    let file = std::fs::File::open(Path::new(p)).map_err(|e| io_err(p, &e))?;
                    let size = file.metadata().map_err(|e| io_err(p, &e))?.len();
                    c.size_matches(size)
                        && c.is_fresh_for_reader(file).map_err(|e| io_err(p, &e))?
                }
                (None, Some(t), _) => c.is_fresh_for(t.as_bytes()),
                (None, None, Some(d)) => c.is_fresh_for(d),
                (None, None, None) => false,
            };
            if fresh {
                let (out, measured) =
                    laterite_ags4_trust::check_parsed_measured(parsed, &opts, world)
                        .map_err(|e| map_err(&e))?;
                count_engine_run();
                return Ok(Checked {
                    file: path.map_or_else(
                        || if text.is_some() { "<text>" } else { "<bytes>" }.to_string(),
                        str::to_string,
                    ),
                    dict_version: out.dict_version.as_str().to_string(),
                    resolution: out.resolution.as_str().to_string(),
                    findings: out.findings,
                    truncated: out.truncated,
                    certified: false,
                    revalidate_reason: Some(reason.as_str().to_string()),
                    measured,
                });
            }
        }
    }

    // A path with a cert goes through the door that reads the file itself (#1097): the
    // size settles a cert of another size before a byte is read, and a cert that
    // vouches is checked against a hash taken as the file streams, so the file is
    // never held. Strictness travels with it, so a stale named cert still raises
    // before the engine runs.
    if let (Some(p), Some(c)) = (path, cert) {
        let file = std::fs::File::open(Path::new(p)).map_err(|e| io_err(p, &e))?;
        let out = laterite_ags4_trust::check_reader(laterite_ags4_trust::ReaderRequest {
            file,
            opts: &opts,
            cert: Some(c),
            world: WorldScope::OnDisk(std::path::PathBuf::from(p)),
            compat: None,
            named: strict_cert,
        })
        .map_err(|e| match e {
            laterite_ags4_trust::ReaderError::Io(e) => io_err(p, &e),
            laterite_ags4_trust::ReaderError::StaleCert => stale(p),
            laterite_ags4_trust::ReaderError::Validate(e) => map_err(&e),
        })?;
        if !out.certified {
            count_engine_run();
        }
        return Ok(Checked {
            file: p.to_string(),
            dict_version: out.dict_version.as_str().to_string(),
            resolution: out.resolution.as_str().to_string(),
            findings: out.findings,
            truncated: out.truncated,
            certified: out.certified,
            revalidate_reason: out.revalidate_reason.map(|r| r.as_str().to_string()),
            measured: None,
        });
    }

    // Bytes, whatever door they came through — a certificate is a statement about bytes.
    let (label, bytes, world) = if let Some(p) = path {
        let b = std::fs::read(Path::new(p)).map_err(|e| io_err(p, &e))?;
        (
            p.to_string(),
            b,
            WorldScope::OnDisk(std::path::PathBuf::from(p)),
        )
    } else if let Some(t) = text {
        (
            "<text>".to_string(),
            t.as_bytes().to_vec(),
            WorldScope::None,
        )
    } else if let Some(d) = data {
        ("<bytes>".to_string(), d.to_vec(), WorldScope::None)
    } else {
        return Err((
            5,
            "bad_args".to_string(),
            "one of path, text, or data is required".to_string(),
        ));
    };

    // Fail before the engine, not after: the whole point of a named cert is to
    // NOT do this work, so discovering the mismatch afterwards would cost exactly
    // what the caller was trying to save. Only staleness is fatal — a cert that is
    // genuinely for these bytes but cannot answer THIS question (a different
    // engine fingerprint, a tier it never measured, `check_files`) is not a
    // caller error, and falls through to the trust model's `revalidate_reason`.
    if strict_cert {
        if let Some(c) = cert {
            if !c.is_fresh_for(&bytes) {
                return Err(stale(&label));
            }
        }
    }

    let out = laterite_ags4_trust::check(laterite_ags4_trust::Request {
        bytes: &bytes,
        opts: &opts,
        cert,
        world,
        compat: None,
    })
    .map_err(|e| map_err(&e))?;
    if !out.certified {
        count_engine_run();
    }

    Ok(Checked {
        file: label,
        dict_version: out.dict_version.as_str().to_string(),
        resolution: out.resolution.as_str().to_string(),
        findings: out.findings,
        truncated: out.truncated,
        certified: out.certified,
        revalidate_reason: out.revalidate_reason.map(|r| r.as_str().to_string()),
        // These bytes were read for this call, so no parse a mint holds is the one this
        // run judged. A certified run did not run the engine at all.
        measured: None,
    })
}

/// Lowercase serde-name of a [`Target`] for the structured `PyDict`
/// (matches the JSON `target` value).
fn target_str(t: Target) -> &'static str {
    match t {
        Target::Line => "line",
        Target::Heading => "heading",
        Target::Cell => "cell",
        Target::Group => "group",
    }
}

/// Lowercase serde-name of a [`Severity`] for the structured `PyDict` — delegates
/// to the single producer so it can't drift from the serde token.
fn severity_str(s: Severity) -> &'static str {
    s.as_str()
}

// `findings_json` / `findings_ndjson` are the ENGINE's renderers
// (`laterite_ags4_validator::findings`), re-exported here at the old private
// path so the call sites below are unchanged. This module used to carry its own
// copy, marked "byte-identical to the binary's `ndjson_string`" — a comment is
// not a guarantee, and there were three such copies (laterite-dev#530). Now `lat validate
// --json`, `laterite`'s report JSON and Node's all come out of one function.
use laterite_ags4_validator::findings::{Truncated, findings_json, findings_ndjson};
use laterite_ags4_validator::verdict::Verdict;

fn err_dict<'py>(
    py: Python<'py>,
    code: i32,
    kind: &str,
    msg: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("ok", false)?;
    d.set_item("error_kind", kind)?;
    d.set_item("error", msg)?;
    d.set_item("exit_code", code)?;
    Ok(d)
}

/// Validate `path` or `text`. Returns a dict the Python layer turns
/// into a `Report` / a python-ags4-shaped `check_file` dict / CLI
/// output. On un-validatable input returns `{ok:false, error_kind,
/// error, exit_code}` (the Python layer raises the mapped exception;
/// the CLI uses `exit_code` directly).
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, include_warnings=false, include_fyi=false, warnings_as_errors=false, check_files=false, encoding=None, dict_path=None, dict_bytes=None, dict_replace=false, cert=None, strict_cert=false, retained=None, max_findings_per_rule=None))]
#[allow(clippy::too_many_arguments)]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn run_check<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    // The VERDICT dial, distinct from the two display dials above (#321): a
    // warning is shown by default and does not fail, and this opts into it
    // failing. Passing it without `include_warnings` is inert rather than an
    // error — there are no warnings in the report to escalate.
    warnings_as_errors: bool,
    check_files: bool,
    encoding: Option<String>,
    // The custom `--dict` overlay (laterite-dev#568): a path OR raw bytes, plus `dict_replace` to drop
    // the base. wasm has no FS, so bytes is the spelling every surface can share.
    dict_path: Option<String>,
    dict_bytes: Option<Vec<u8>>,
    dict_replace: bool,
    // The certificate, if the caller named one. The decision to trust it is NOT made in
    // Python — it is made once, in `laterite_ags4_trust`, alongside every other surface.
    // The Python layer used to make it itself, with its own conjunction of predicates.
    cert: Option<PyRef<'py, PySidecar>>,
    // Private to the native ABI — neither `validate()` nor `Ags4File.validate()`
    // exposes it. It records WHICH door the cert came through; see `validate`.
    strict_cert: bool,
    // A read handle's `_handle`: validate the parse it already holds rather than
    // parsing the source again (#1090). The caller vouches it was parsed from this
    // source under this `encoding`.
    retained: Option<PyRef<'py, Reading>>,
    // Keep at most this many findings per rule; the counts and the verdict still
    // cover every finding (#1089). `None` keeps them all.
    max_findings_per_rule: Option<usize>,
) -> PyResult<Bound<'py, PyDict>> {
    // Release the GIL for the whole parse+validate compute (pure Rust, touches no
    // Python) so concurrent validators actually parallelise across cores instead
    // of serialising behind the interpreter lock (#8). The cert is the only
    // Python-bound input — clone its small Rust inner out first so the detached
    // closure captures owned data (`Sidecar: Clone`, cheap: the byte index +
    // hashes, not the file).
    let cert_inner = cert.as_ref().map(|c| c.inner.clone());
    let retained = retained.as_ref().map(|r| &r.parsed);
    let outcome = py.detach(|| {
        validate(
            path.as_deref(),
            text.as_deref(),
            data.as_deref(),
            dict_version.as_deref(),
            include_warnings,
            include_fyi,
            check_files,
            encoding.as_deref(),
            dict_path.as_deref(),
            dict_bytes.as_deref(),
            dict_replace,
            cert_inner.as_ref(),
            strict_cert,
            retained,
            max_findings_per_rule,
        )
    });
    match outcome {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok(checked) => {
            let Checked {
                file,
                dict_version: dv,
                resolution: res,
                findings: found,
                truncated,
                certified,
                revalidate_reason,
                measured,
            } = checked;
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("file", &file)?;
            d.set_item("dict_version", dv)?;
            d.set_item("resolution", res)?;
            // Whether the rule ENGINE was skipped. Never "whether the file was checked":
            // a world check (Rule 20 on-disk) runs even on a certified read.
            d.set_item("certified", certified)?;
            // Why a proffered cert didn't help (stable token), or `None`. A reader that
            // passed no cert always sees `None`; a certified read also sees `None`.
            d.set_item("revalidate_reason", revalidate_reason)?;

            // `count` is what the report SHOWS; the verdict is what it
            // CONCLUDES (#321). Both are emitted so the Python layer never has
            // to re-derive either — `Report.is_valid` reads `valid`, not
            // `count == 0`, which is how the two stopped being the same claim.
            // Both count every finding, kept or not (#1089).
            let verdict = Verdict::of(&found, &truncated, warnings_as_errors);
            d.set_item("count", verdict.total())?;
            d.set_item("errors", verdict.errors)?;
            d.set_item("warnings", verdict.warnings)?;
            d.set_item("fyi", verdict.fyi)?;
            d.set_item("valid", verdict.is_valid())?;
            d.set_item("exit_code", verdict.exit_code())?;
            // `{rule: {shown, total}}` for each rule the cap cut; empty without one.
            let cut = PyDict::new(py);
            for (rule, t) in &truncated {
                let o = PyDict::new(py);
                o.set_item("shown", t.shown)?;
                o.set_item("total", t.total)?;
                cut.set_item(rule, o)?;
            }
            d.set_item("truncated", cut)?;
            // The finding list and both serialisations are built on ask, not here
            // (#1089): a file with one finding per line would otherwise hold all
            // three at once — a Python dict per finding plus two whole-report
            // strings — whether or not the caller ever reads them.
            d.set_item(
                "_found",
                CheckedFindings {
                    file,
                    found,
                    truncated,
                    measured,
                },
            )?;
            Ok(d)
        }
    }
}

/// A validation's findings, held natively until the Python `Report` asks for them
/// in one of its three shapes (#1089). Nothing is cached: each call renders anew,
/// so a caller who reads one shape never also holds the other two.
///
/// It also carries the run's `Measurement`, when there is one, for `Sidecar.mint` to
/// reuse (#1096). It stays native for the same reason the trust crate keeps the
/// type's fields private: Python cannot build one, so the mint cannot be handed counts
/// the engine did not produce.
#[pyclass(frozen)]
struct CheckedFindings {
    file: String,
    found: Findings,
    truncated: Truncated,
    measured: Option<laterite_ags4_trust::Measurement>,
}

#[pymethods]
impl CheckedFindings {
    /// One `{rule, line, group, desc, …}` dict per finding, in rule order.
    fn items<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let items = PyList::empty(py);
        for (rule, fs) in &self.found {
            for f in fs {
                let o = PyDict::new(py);
                o.set_item("rule", rule)?;
                o.set_item("line", f.line)?;
                o.set_item("group", &f.group)?;
                o.set_item("desc", &f.desc)?;
                // Additively surface the rich location/severity fields
                // when set away from default — existing readers keyed on
                // line/group/desc are unaffected. Target/Severity render
                // as their lowercase serde names; char_span as a 2-tuple.
                let loc = &f.location;
                if loc.target != Target::Line {
                    o.set_item("target", target_str(loc.target))?;
                }
                if let Some(fi) = loc.field_index {
                    o.set_item("field_index", fi)?;
                }
                if let Some(h) = &loc.heading {
                    o.set_item("heading", h)?;
                }
                if let Some(dr) = loc.data_row {
                    o.set_item("data_row", dr)?;
                }
                if let Some((a, b)) = loc.char_span {
                    o.set_item("char_span", (a, b))?;
                }
                if f.severity != Severity::Error {
                    o.set_item("severity", severity_str(f.severity))?;
                }
                items.append(o)?;
            }
        }
        Ok(items)
    }

    /// Byte-identical to `lat validate --json`.
    fn json(&self) -> String {
        findings_json(&self.file, &self.found, &self.truncated)
    }

    /// Byte-identical to `lat validate --ndjson`.
    fn ndjson(&self) -> String {
        findings_ndjson(&self.found, &self.truncated)
    }
}

/// Serialise a slice of [`Fix`] into the Python `applied`-ledger shape — one
/// `{kind, label, rule, line, risk}` dict per fix — shared by `fix()`'s
/// `FixResult.applied` and `build_ags4`'s `BuildResult.applied` so both surfaces
/// present an identical record (#294 F#7).
pub(crate) fn fixes_to_pylist<'py>(py: Python<'py>, fixes: &[Fix]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for f in fixes {
        let o = PyDict::new(py);
        // The engine's own spellings (#937) — this function used to derive
        // `kind` by serde round-trip and `risk` by hand-matched literals,
        // one drift-prone mechanism each.
        o.set_item("kind", f.kind.as_str())?;
        o.set_item("label", &f.label)?;
        o.set_item("rule", &f.rule)?;
        o.set_item("line", f.line)?;
        o.set_item("risk", f.risk.as_str())?;
        list.append(o)?;
    }
    Ok(list)
}

/// The shared staged atomic write (`hostopts::staged_write_io`, #938) behind
/// a Python door: a temp file in the destination's own directory + rename,
/// so `out` never holds a partial write. The `_io` variant + PyO3's
/// `io::Error` conversion is what keeps this door's error contract exactly
/// what the old in-Python dance raised — the right `OSError` SUBCLASS
/// (`FileNotFoundError` for a missing directory, `PermissionError`, …), not
/// a flattened base `OSError` with the shape lost in a message.
#[pyfunction]
fn staged_write(path: &str, data: &[u8]) -> PyResult<()> {
    laterite_ags4_hostopts::staged_write_io(Path::new(path), data).map_err(PyErr::from)
}

/// Headless one-shot mechanical repair of a delivered AGS4 file (the same engine
/// the browser fix UI uses, applied without a UI). Reads from path/text/data,
/// computes fixes against the file's own findings, applies the *safe* set (plus
/// the intent-guessing *risky* set when `include_risky`), and re-validates the
/// result. Returns `(fixed_bytes, residual_findings, applied_fixes, dict_version,
/// resolution)` or the `(exit_code, kind, msg)` error triple. Mirrors the
/// `AutoFix` path in `laterite-ags4-emit::emit`, but on a file's own bytes rather
/// than freshly-built data.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn fix_core(
    path: Option<&str>,
    text: Option<&str>,
    data: Option<&[u8]>,
    dvr: Option<&str>,
    encoding: Option<&str>,
    include_risky: bool,
    only: Option<&[String]>,
    exclude: &[String],
    dict_path: Option<&str>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    rewrite: (&str, laterite_ags4_validator::fixes::Recode),
) -> Result<FixCoreOut, (i32, String, String)> {
    // Parsed like merge's `on_code_case`, so an unknown token is refused in
    // the same words on both verbs (#1024).
    let rewrite = CodeRewrite {
        on_code_case: rewrite
            .0
            .parse()
            .map_err(|m: String| (5, "bad_args".to_string(), m))?,
        recode: rewrite.1,
    };
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let custom_dict = build_custom_dict(dict_path, dict_bytes, dict_replace, over, enc)?;
    // The source bytes — fix needs the raw document to apply byte/char edits to,
    // so unlike `validate` we always materialise bytes (a path is read here).
    let raw: Vec<u8> = if let Some(p) = path {
        std::fs::read(p).map_err(|e| {
            let kind = if e.kind() == std::io::ErrorKind::NotFound {
                "not_found"
            } else {
                "io"
            };
            (3, kind.to_string(), format!("{p}: {e}"))
        })?
    } else if let Some(d) = data {
        d.to_vec()
    } else if let Some(t) = text {
        t.as_bytes().to_vec()
    } else {
        return Err((
            5,
            "bad_args".to_string(),
            "one of path, text, or data is required".to_string(),
        ));
    };

    // The orchestration (parse → run → compute → apply → re-validate) lives once,
    // in the validator's `fix_document`; the CLI shares it.
    // The residual re-validation tier matches `validate()`'s default (errors +
    // warnings) so a fix reports what it left behind at the same tier the user
    // sees on validate — was errors + FYI, which both under- and over-reported
    // vs the other surfaces (#294 Batch C).
    let mut opts = CheckOptions::default();
    opts.dict_version = over;
    opts.custom_dict = custom_dict;
    opts.include_warnings = true;
    opts.include_fyi = false;
    opts.check_files = false;
    opts.encoding = enc;
    let out = fix_document_recoding(&raw, &opts, include_risky, only, exclude, &rewrite).map_err(
        |e| match e {
            FixError::Validator(e) => map_err(&e),
            // A recode naming what the file does not have is the caller's
            // argument — `bad_args`, as merge's is.
            FixError::Recode(e) => (5, "bad_args".to_string(), e.to_string()),
        },
    )?;
    Ok((
        out.fixed,
        out.residual,
        out.applied,
        out.dict_version.as_str().to_string(),
        out.resolution.as_str().to_string(),
        out.risky_available,
        out.skipped,
    ))
}

/// `fix_core`'s success: the fixed bytes, residual, applied fixes, edition,
/// resolution, withheld-risky count and the requested rewrites left undone.
type FixCoreOut = (
    Vec<u8>,
    Findings,
    Vec<Fix>,
    String,
    String,
    usize,
    Vec<SkippedRewrite>,
);

/// Headless mechanical fix of `path`/`text`/`data`. Returns a dict the Python
/// `fix()` turns into a `FixResult` (`fixed` bytes, `findings_json` residual
/// `{rule:[…]}`, the `applied` fix list, `fixes_applied` count). On
/// un-fixable input returns the same `{ok:false, …}` shape as `run_check`, so
/// the Python layer raises the mapped exception.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, encoding=None, include_risky=false, only=None, exclude=None, dict_path=None, dict_bytes=None, dict_replace=false, on_code_case="keep", recode=None))]
#[allow(clippy::too_many_arguments)]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn fix_file<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    dict_version: Option<String>,
    encoding: Option<String>,
    include_risky: bool,
    only: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    dict_path: Option<String>,
    dict_bytes: Option<Vec<u8>>,
    dict_replace: bool,
    on_code_case: &str,
    recode: Option<laterite_ags4_validator::fixes::Recode>,
) -> PyResult<Bound<'py, PyDict>> {
    let exclude = exclude.unwrap_or_default();
    match fix_core(
        path.as_deref(),
        text.as_deref(),
        data.as_deref(),
        dict_version.as_deref(),
        encoding.as_deref(),
        include_risky,
        only.as_deref(),
        &exclude,
        dict_path.as_deref(),
        dict_bytes.as_deref(),
        dict_replace,
        (on_code_case, recode.unwrap_or_default()),
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok((bytes, residual, applied, dv, res, risky_available, skipped)) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("fixed", PyBytes::new(py, &bytes))?;
            d.set_item("dict_version", dv)?;
            d.set_item("resolution", res)?;
            d.set_item("risky_available", risky_available)?;
            d.set_item(
                "findings_json",
                serde_json::to_string(&residual).unwrap_or_else(|_| "{}".into()),
            )?;
            d.set_item("fixes_applied", applied.len())?;
            d.set_item("applied", fixes_to_pylist(py, &applied)?)?;
            // Straight off the engine struct's derive, so the shape is the
            // one every surface reports.
            d.set_item(
                "skipped_json",
                serde_json::to_string(&skipped).unwrap_or_else(|_| "[]".into()),
            )?;
            Ok(d)
        }
    }
}

/// The rule catalogue (title / severity / fixable / cited observations per rule)
/// as the engine's gated `rules_meta.json` — the single source
/// `laterite.list_rules()` parses. Read-only; no file argument; the JSON is
/// compile-time-embedded so this never touches disk.
#[pyfunction]
fn list_rules() -> &'static str {
    laterite_ags4_validator::rule_metadata_json()
}

/// The KEY-aware/type-aware revision diff of two AGS4 documents (`a` baseline,
/// `b` revision), serialised as a `RevisionDelta` JSON string — the same diff
/// the browser's Tools tab produces, via the shared `laterite-ags4-diff` leaf.
/// Rows are matched by the group's dictionary KEY headings; cells are compared
/// through the typed value, so a formatting-only change (`"1.0"` → `"1.00"`) is
/// suppressed. The dictionary edition is the revision's `TRAN_AGS` (forced by
/// `dict_version`), falling back to the bundled default — like the wasm `diff()`.
fn diff_core(
    a: &[u8],
    b: &[u8],
    dvr: Option<&str>,
    encoding: Option<&str>,
) -> Result<String, (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let pa = laterite_ags4_parse::parse_bytes(a, enc)
        .map_err(ValidatorError::from)
        .map_err(|e| map_err(&e))?;
    let pb = laterite_ags4_parse::parse_bytes(b, enc)
        .map_err(ValidatorError::from)
        .map_err(|e| map_err(&e))?;
    // KEY headings come from the dictionary; pick the edition from the revision
    // (b)'s TRAN_AGS (forced by dict_version), falling back to the standard.
    let tran = laterite_ags4_validator::tran_ags_of(&pb);
    let dv = laterite_ags4_validator::resolve_dict_version(over, tran.as_deref())
        .map_or(laterite_ags4_validator::dict::FALLBACK, |(dv, _)| dv);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);
    // The shared pretty render, NOT a local to_string: `_cli.py` prints this
    // string verbatim (its json.loads → json.dumps round trip escaped non-ASCII,
    // #542), and the Python API's json.loads is indifferent to layout.
    Ok(laterite_ags4_diff::delta_json(&delta))
}

/// Compare two AGS4 documents (raw `a`/`b` bytes). Returns `{ok:true,
/// delta_json}` (the serialised `RevisionDelta`, which the Python layer parses)
/// or the `{ok:false, error_kind, exit_code, error}` failure dict.
#[pyfunction]
#[pyo3(signature = (a, b, dict_version=None, encoding=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn diff_files(
    py: Python<'_>,
    a: Vec<u8>,
    b: Vec<u8>,
    dict_version: Option<String>,
    encoding: Option<String>,
) -> PyResult<Bound<'_, PyDict>> {
    match diff_core(&a, &b, dict_version.as_deref(), encoding.as_deref()) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok(delta_json) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("delta_json", delta_json)?;
            Ok(d)
        }
    }
}

/// Reconcile N AGS4 deliveries of one project into a single file (the same leaf
/// `lat merge` uses). Files are merged in argument order — a later file wins a
/// KEY conflict — with rows identified by their dictionary KEY headings, so a
/// re-sorted borehole list still merges onto its prior self. A column two files
/// typed differently is settled by `on_type_clash` — `"error"` (default), `"widen"`
/// (fall back to `X`, raw values kept) or `"promote"` (keep the greatest nDP
/// precision, zero-padding the coarser values). Returns `(merged_bytes,
/// warnings_json, revisions_json)` or the `(exit_code, kind, message)` failure
/// triple. When `tran` carries an issue + date, a single merge-TRAN row is
/// synthesised for the output.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn merge_core(
    files: &[Vec<u8>],
    on_type_clash: &str,
    on_missing_tran: &str,
    dict_rows: &str,
    row_order: &str,
    on_code_case: &str,
    recode: laterite_ags4_merge::Recode,
    dvr: Option<&str>,
    encoding: Option<&str>,
    tran: (
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
    ),
) -> Result<(Vec<u8>, String, String), (i32, String, String)> {
    use laterite_ags4_merge::{
        CodeCaseMode, DictRows, MergeError, MergeOpts, MissingTranMode, RowOrder, TranStamp,
        TypeClashMode, merge_parsed,
    };

    if files.len() < 2 {
        return Err((
            5,
            "bad_args".to_string(),
            "merge needs at least two files".to_string(),
        ));
    }
    // One vocabulary for every surface: the mode strings and this message come from
    // the merge crate's FromStr, so Python cannot accept a token the CLI rejects.
    let clash: TypeClashMode = on_type_clash
        .parse()
        .map_err(|m: String| (5, "bad_args".to_string(), m))?;
    // Same discipline: the tokens and the message are the merge crate's FromStr,
    // so Python cannot accept a value the CLI rejects, or reject one it accepts.
    let missing_tran: MissingTranMode = on_missing_tran
        .parse()
        .map_err(|m: String| (5, "bad_args".to_string(), m))?;
    let dict_rows: DictRows = dict_rows
        .parse()
        .map_err(|m: String| (5, "bad_args".to_string(), m))?;
    let row_order: RowOrder = row_order
        .parse()
        .map_err(|m: String| (5, "bad_args".to_string(), m))?;
    // Parsed like `on_type_clash`, so an unknown token is refused in the
    // merge crate's own words (#1010).
    let on_code_case: CodeCaseMode = on_code_case
        .parse()
        .map_err(|m: String| (5, "bad_args".to_string(), m))?;
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let parsed: Vec<_> = files
        .iter()
        .map(|b| {
            laterite_ags4_parse::parse_bytes(b, enc)
                .map_err(ValidatorError::from)
                .map_err(|e| map_err(&e))
        })
        .collect::<Result<_, _>>()?;

    // Edition from the newest (last) file's TRAN_AGS, forced by dict_version —
    // the same resolution the CLI and diff use.
    let dv = laterite_ags4_validator::resolve_dict_version(
        over,
        parsed
            .last()
            .and_then(laterite_ags4_validator::tran_ags_of)
            .as_deref(),
    )
    .map_or(laterite_ags4_validator::dict::FALLBACK, |(dv, _)| dv);

    // All five or none — the shared rule. `ags` is merge's to fill from the
    // edition it resolved; a caller-stated value could only contradict it.
    let (isno, date, prod, recv, stat, desc, rem) = tran;
    let tran = TranStamp::from_parts(
        isno.map(str::to_string),
        date.map(str::to_string),
        prod.map(str::to_string),
        recv.map(str::to_string),
        stat.map(str::to_string),
        desc.map(str::to_string),
        rem.map(str::to_string),
    )
    .map_err(|e| (5, "bad_args".to_string(), e.to_string()))?;

    let opts = MergeOpts {
        on_type_clash: clash,
        on_missing_tran: missing_tran,
        edition: dv,
        tran,
        dict_rows,
        row_order,
        on_code_case,
        recode,
        ..Default::default()
    };

    match merge_parsed(&parsed, &opts) {
        Ok(res) => {
            // Straight off the engine structs — their Serialize derive owns the
            // wire shape, so this binding can't drift from the other two (#542).
            Ok((
                res.bytes,
                serde_json::to_string(&res.warnings).unwrap_or_else(|_| "[]".to_string()),
                serde_json::to_string(&res.revisions).unwrap_or_else(|_| "[]".to_string()),
            ))
        }
        // An unresolved TYPE conflict / emit failure is a schema-level rejection
        // (exit 6), matching the `lat merge` codes.
        Err(e @ MergeError::TypeConflict { .. }) => {
            Err((6, "type_conflict".to_string(), e.to_string()))
        }
        // Fatal in EVERY mode — no `on_type_clash` value absorbs it (laterite-dev#501). Its own
        // kind so a caller can tell "your files disagree on a column's TYPE, which
        // widen/promote CAN settle" from "your files disagree on its UNIT, which
        // nothing can" — the two need different fixes.
        Err(e @ MergeError::UnitConflict { .. }) => {
            Err((6, "unit_conflict".to_string(), e.to_string()))
        }
        // Its own kind at the same exit code, matching the two above: all three are
        // merge refusals a caller may want to route on separately, because the three
        // need different fixes (settle the type, reconcile the UNIT, supply a stamp).
        Err(e @ MergeError::MissingTran) => Err((6, "missing_tran".to_string(), e.to_string())),
        // A requested code rewrite that would merge two distinct rows: a merge
        // refusal like the three above, with its own token for the same reason.
        Err(e @ MergeError::KeyCollision { .. }) => {
            Err((6, "key_collision".to_string(), e.to_string()))
        }
        // A recode naming what the inputs do not have is the caller's argument,
        // not the files' fault — the class an unknown mode token is in.
        Err(e @ MergeError::Recode(_)) => Err((5, "bad_args".to_string(), e.to_string())),
        Err(e @ MergeError::Emit(_)) => Err((6, "emit_error".to_string(), e.to_string())),
    }
}

/// Merge raw AGS4 documents (`files`, ≥2). Returns `{ok:true, merged (bytes),
/// warnings_json, revisions_json}` — the Python layer parses the two JSON
/// strings — or the `{ok:false, error_kind, exit_code, error}` failure dict.
#[pyfunction]
#[pyo3(signature = (files, on_type_clash="error", on_missing_tran="reconcile", dict_version=None, encoding=None, tran_issue=None, tran_date=None, tran_producer=None, tran_recipient=None, tran_status=None, tran_description=None, tran_remarks=None, dict_rows="keep", row_order="input", on_code_case="keep", recode=None))]
#[allow(clippy::too_many_arguments)]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn merge_files<'py>(
    py: Python<'py>,
    files: Vec<Vec<u8>>,
    on_type_clash: &str,
    on_missing_tran: &str,
    dict_version: Option<String>,
    encoding: Option<String>,
    tran_issue: Option<String>,
    tran_date: Option<String>,
    tran_producer: Option<String>,
    tran_recipient: Option<String>,
    tran_status: Option<String>,
    tran_description: Option<String>,
    tran_remarks: Option<String>,
    dict_rows: &str,
    row_order: &str,
    on_code_case: &str,
    recode: Option<laterite_ags4_merge::Recode>,
) -> PyResult<Bound<'py, PyDict>> {
    let tran = (
        tran_issue.as_deref(),
        tran_date.as_deref(),
        tran_producer.as_deref(),
        tran_recipient.as_deref(),
        tran_status.as_deref(),
        tran_description.as_deref(),
        tran_remarks.as_deref(),
    );
    match merge_core(
        &files,
        on_type_clash,
        on_missing_tran,
        dict_rows,
        row_order,
        on_code_case,
        recode.unwrap_or_default(),
        dict_version.as_deref(),
        encoding.as_deref(),
        tran,
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok((bytes, warnings_json, revisions_json)) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("merged", pyo3::types::PyBytes::new(py, &bytes))?;
            d.set_item("warnings_json", warnings_json)?;
            d.set_item("revisions_json", revisions_json)?;
            Ok(d)
        }
    }
}

/// Parse `path` or `text` into per-group primitives (string cells —
/// AGS4 is a text format). `headings`/`units`/`types`/row `values`
/// exclude the leading row tag (matching `ParsedGroup`); the Python
/// side prepends the literal `HEADING`/`UNIT`/`TYPE`/`DATA` when it
/// needs the python-ags4-shaped frame.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn parse_primitives(
    py: Python<'_>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'_, PyDict>> {
    let Some(enc) = laterite_ags4_parse::resolve_encoding(encoding.as_deref()) else {
        let label = encoding.as_deref().unwrap_or("");
        return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
    };
    let parsed = match (path.as_deref(), text.as_deref(), data.as_deref()) {
        (Some(p), _, _) => {
            laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
        }
        (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
        (_, _, Some(d)) => laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from),
        _ => {
            return err_dict(py, 5, "bad_args", "one of path, text, or data is required");
        }
    };
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(&e);
            return err_dict(py, c, &k, &m);
        }
    };

    let d = PyDict::new(py);
    d.set_item("ok", true)?;
    d.set_item("group_order", pf.group_order.clone())?;
    d.set_item("tran_ags", laterite_ags4_validator::tran_ags_of(&pf))?;

    let groups = PyDict::new(py);
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let gd = PyDict::new(py);
        gd.set_item("group_line", g.group_line)?;
        gd.set_item("heading_line", g.heading_line)?;
        gd.set_item("unit_line", g.unit_line)?;
        gd.set_item("type_line", g.type_line)?;
        gd.set_item("headings", g.headings.clone())?;
        gd.set_item("units", g.units.clone())?;
        gd.set_item("types", g.types.clone())?;
        let rows = PyList::empty(py);
        for r in &g.rows {
            let ro = PyDict::new(py);
            ro.set_item("line", r.line)?;
            ro.set_item(
                "values",
                g.row_spans(r)
                    .iter()
                    .map(|s| s.slice(g.text()))
                    .collect::<Vec<_>>(),
            )?;
            rows.append(ro)?;
        }
        gd.set_item("rows", rows)?;
        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

/// Parse to the python-ags4-shaped ("compat") all-`Utf8` Arrow — one table per
/// group, built Rust-side with NO per-cell `PyObject` boxing (unlike
/// `parse_primitives`, which crosses the whole file as Python strings). Each
/// group's `table` carries a leading `HEADING` tag column then one raw-string
/// column per heading (positional names `c0`, `c1`, … — the Python side owns
/// python-ags4's duplicate-heading renaming and the `rename=False` raise). Also
/// returns `group_records` (every GROUP occurrence — for the duplicate-GROUP
/// raise) and, per group, `ragged` (DATA rows whose field count differs from the
/// heading count — for the ragged-row raise), so the compat reader needs no
/// second file scan. This is the fast path behind `compat.AGS4_to_dataframe`.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None, only_groups=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn parse_compat_arrow(
    py: Python<'_>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
    only_groups: Option<Vec<String>>,
) -> PyResult<Bound<'_, PyDict>> {
    let Some(enc) = laterite_ags4_parse::resolve_encoding(encoding.as_deref()) else {
        let label = encoding.as_deref().unwrap_or("");
        return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
    };
    if path.is_none() && text.is_none() && data.is_none() {
        return err_dict(py, 5, "bad_args", "one of path, text, or data is required");
    }
    // Release the GIL for the parse (pure Rust, the dominant ~142 ms) so
    // concurrent compat reads parallelise across cores (#8); only the cheaper
    // per-group Arrow build + PyO3 crossing below stays under the lock.
    let parsed = py.detach(
        || match (path.as_deref(), text.as_deref(), data.as_deref()) {
            (Some(p), _, _) => {
                laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
            }
            (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
            (_, _, Some(d)) => {
                laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from)
            }
            _ => unreachable!("input presence checked above"),
        },
    );
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(&e);
            return err_dict(py, c, &k, &m);
        }
    };

    let d = PyDict::new(py);
    d.set_item("ok", true)?;
    d.set_item("group_order", pf.group_order.clone())?;
    d.set_item("tran_ags", laterite_ags4_validator::tran_ags_of(&pf))?;
    // Every GROUP occurrence (the parse dedups/merges groups but keeps this list)
    // so the reader can raise on a duplicate GROUP with both line numbers.
    d.set_item(
        "group_records",
        pf.group_records
            .iter()
            .map(|gr| (gr.code.clone(), gr.line))
            .collect::<Vec<_>>(),
    )?;

    // `only_groups` narrows which groups get an Arrow `table` built and crossed —
    // and NOTHING else. Every group still contributes its headings, line anchors,
    // `ragged` list and `group_records` entry, because python-ags4's hard raises
    // (duplicate GROUP, ragged row, duplicate heading under `rename=False`) fire
    // on the whole file regardless of narrowing. Narrowing the raises instead
    // would be a behaviour change wearing a performance change's clothes.
    //
    // An EMPTY list means "all", matching the caller: `AGS4_to_dataframe` reads
    // `only_groups if only_groups else <every group>`, so `[]` is falsy there.
    let wanted: Option<HashSet<&str>> = only_groups
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| v.iter().map(String::as_str).collect());

    let groups = PyDict::new(py);
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let gd = PyDict::new(py);
        gd.set_item("headings", g.headings.clone())?;
        gd.set_item("group_line", g.group_line)?;
        gd.set_item("heading_line", g.heading_line)?;
        gd.set_item("unit_line", g.unit_line)?;
        gd.set_item("type_line", g.type_line)?;
        gd.set_item(
            "line_numbers",
            g.rows.iter().map(|r| r.line).collect::<Vec<_>>(),
        )?;
        // Ragged DATA rows (raw field count != heading count) — python-ags4
        // raises on the first. Usually empty; only the offenders cross.
        let n_head = g.headings.len();
        let ragged: Vec<(u32, usize)> = g
            .rows
            .iter()
            .filter(|r| r.n_values() != n_head)
            .map(|r| (r.line, r.n_values()))
            .collect();
        gd.set_item("ragged", ragged)?;

        // The one narrowed step. A skipped group simply has no `table` key; the
        // caller only reaches for one on a group it asked for.
        if wanted.as_ref().is_none_or(|w| w.contains(code.as_str())) {
            let batch = laterite_ags4_types::arrow_cols::build_record_batch_compat(
                &g.headings,
                &g.units,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            )
            .map_err(|e| PyRuntimeError::new_err(format!("compat arrow for {code}: {e}")))?;
            let schema = batch.schema();
            gd.set_item("table", PyTable::try_new(vec![batch], schema)?)?;
        }

        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

/// A parsed AGS4 file held Rust-side. `parse_arrow` crosses only cheap
/// per-group metadata; the raw `ParsedFile` stays HERE and serves two roles
/// on demand: `table_for` builds a group's typed Arrow table on first touch
/// (per-group lazy reads), and `emit` re-emits byte-faithful AGS4 for
/// `write()`. No O(cells) `PyObject` dict crosses the boundary, and there is no
/// second AGS formatter — emit reproduces the source DATA values exactly. The
/// Python `Ags4File` holds one of these as its `_handle`.
#[pyclass]
struct Reading {
    parsed: laterite_ags4_parse::ParsedFile,
}

#[pymethods]
impl Reading {
    /// Reconstruct spec-correct AGS4 text from the retained parse (CRLF,
    /// every field quoted, blank line between groups) — byte-faithful to
    /// the source DATA values it re-emits.
    fn emit(&self) -> String {
        // The block build is the shared constructor beside the writer — one
        // copy for this surface and the xcheck reference leg, so the two
        // cannot drift apart (#847 retired the byte-identical pair).
        let blocks = laterite_ags4_emit::canonical_matrix_blocks(&self.parsed);
        // The ONE guarded verbatim writer (was this crate's private `emit::emit`, now
        // gone). `trailing_blank_line = false` — the canonical shape every other surface
        // emits, so `.text` no longer carries the trailing blank line that made the Python
        // re-emit disagree with Node's. `.expect` is sound: these cells come from a PARSED
        // AGS4 file, whose tokeniser already split on CR/LF, so no cell can contain one —
        // the guard has nothing to fire on. (`compat`, which accepts arbitrary frames, is
        // the fallible caller.)
        let mut out = Vec::new();
        laterite_ags4_emit::write_ags4_matrix(&mut out, &blocks, false)
            .expect("re-emitted parsed cells cannot contain an embedded CR/LF");
        String::from_utf8(out).expect("AGS4 re-emit is valid UTF-8")
    }

    /// The source line of each DATA row of `code`, or `None` if `code` isn't in
    /// the file — read off the retained parse when asked, not held per group.
    fn line_numbers(&self, code: &str) -> Option<Vec<u32>> {
        let g = self.parsed.groups.get(code)?;
        Some(g.rows.iter().map(|r| r.line).collect())
    }

    /// `{group: parent}` for every group in the file that names a parent —
    /// the group its `_parent_id` hashes into, resolved exactly as
    /// `table_for` keys it: the registry for a standard group, the file's own
    /// `DICT_PGRP` for a file-declared one (`-`/blank = none). `.at(…,
    /// by="_id")` reads this to know which groups are direct children, so a
    /// declared child of a standard group is narrowed, not passed through.
    fn parents(&self) -> std::collections::HashMap<String, String> {
        let reg = laterite_ags4_core::registry::registry();
        // The DICT walk only when a group the registry lacks needs it.
        let mut fd = None;
        self.parsed
            .groups
            .keys()
            .filter_map(|code| {
                let parent = match reg.get(code) {
                    Some(desc) => desc.parent.clone(),
                    None => fd
                        .get_or_insert_with(|| {
                            laterite_ags4_core::effective_dict::FileDict::from_parsed(&self.parsed)
                        })
                        .parent(code)
                        .map(str::to_string),
                };
                parent
                    .filter(|p| !p.is_empty() && p != "-")
                    .map(|p| (code.clone(), p))
            })
            .collect()
    }

    /// Build ONE group's typed Arrow table on demand. The read path is
    /// per-group lazy: `read()` / `scan()` only pay for the groups actually
    /// touched, so a 69-group file you query two groups of builds two
    /// `RecordBatches`, not 69. Returns `None` if `code` isn't in the file.
    /// Same shared emitter (`laterite_ags4_types::arrow_cols`) as the old eager build —
    /// byte-identical columns, and still the SAME cast the browser's IPC path
    /// uses. The Python `Ags4File` memoises the result per code.
    /// `content_hash` appends a trailing `_content_hash` column — the typed,
    /// blank-insensitive fingerprint of the row's whole VALUE (keychain's
    /// `content_hash`), as opposed to `_id`, which fingerprints its IDENTITY.
    /// Opt-in at BUILD time, not projection: hashing every cell costs a
    /// `parse_value` pass + a SHA-256 per row, so a caller who never asks pays
    /// nothing and the default batch stays byte-identical (a `SELECT *` through
    /// `.sql()` is unchanged — which projection-time stripping could not have
    /// promised). Appended LAST so `_id`/`_parent_id` keep columns 0/1.
    #[pyo3(signature = (code, content_hash=false, with_keys=true))]
    fn table_for(
        &self,
        code: &str,
        content_hash: bool,
        with_keys: bool,
    ) -> PyResult<Option<PyTable>> {
        let Some(g) = self.parsed.groups.get(code) else {
            return Ok(None);
        };
        // Relational layer is **always-keyed**: a KNOWN group's batch carries the
        // two content-addressed key columns (`_id` col 0, `_parent_id` col 1) so a
        // cross-group join in `.sql()` works with no opt-in — the Python frame
        // accessor strips them by default. The ids come from the one shared
        // keychain (`group_row_ids` → `keychain::row_ids`), so they are byte-
        // identical across every surface that shares this crate. A group the
        // registry does not know mints from the Rule 18 effective dictionary
        // instead (#815): its file-declared KEY tuple — the same declarations
        // the validator keys Rule 10a off — with the declared parent chain, so
        // `child._parent_id == parent._id` holds across the registry/file
        // boundary. Declared keyless (or not declared at all) → unkeyed
        // (`ids: None`, #303's original shape).
        //
        // The hash needs NO registry — it hashes every heading rather than the
        // spec key-chain, so a custom/passthrough group (which gets no `_id` at
        // all) still gets a usable value fingerprint. Types come from the file's
        // OWN TYPE row: canonicalisation is per-file, which is why two files that
        // disagree on a column's TYPE may not dedup. Keying and hashing are
        // independent knobs, both folded in by the ONE shared
        // `build_record_batch_synth` (`_id`/`_parent_id` col 0/1, `_content_hash`
        // trailing) so every host gets identical column order by construction.
        // `with_keys=false` (the frame fast-path: keys=False + xn="string") SKIPS
        // the content-addressed `group_row_ids` — its per-row HashMap-of-every-
        // heading + SHA-256 is the dominant read cost, and a plain frame read
        // strips `_id`/`_parent_id` right back off, so computing them was pure
        // overhead. The id MATH is untouched: every keyed caller (`.sql()` /
        // `.at()` / `keys=True`, all default `with_keys=true`) still gets the
        // byte-identical ids the certificate + cross-surface gates pin.
        let ids = if with_keys {
            let reg = laterite_ags4_core::registry::registry();
            if reg.get(code).is_some() {
                Some(laterite_ags4_core::keychain::group_row_ids(
                    reg,
                    code,
                    &g.headings,
                    g.rows.len(),
                    |col, row| g.cell(col, row),
                ))
            } else {
                // The DICT walk is paid only on this (rare) branch — a
                // standard group's read builds no FileDict.
                let fd = laterite_ags4_core::effective_dict::FileDict::from_parsed(&self.parsed);
                let v = laterite_ags4_core::keychain::group_row_ids_effective(
                    reg,
                    &fd,
                    code,
                    &g.headings,
                    g.rows.len(),
                    |col, row| g.cell(col, row),
                );
                (!v.is_empty()).then_some(v)
            }
        } else {
            None
        };
        let hashes = if content_hash {
            Some(laterite_ags4_core::keychain::group_content_hashes(
                code,
                &g.headings,
                &g.units,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            ))
        } else {
            None
        };
        // The UNIT row travels too: a `DT` column declared `hh:mm` or
        // `yyyy-mm` can only be typed against it (#999).
        let batch = laterite_ags4_types::arrow_cols::build_record_batch_synth_with_units(
            &laterite_ags4_types::arrow_cols::SynthColumns {
                ids: ids.as_deref(),
                hashes: hashes.as_deref(),
            },
            &g.headings,
            &g.types,
            &g.units,
            g.rows.len(),
            |col, row| g.cell(col, row),
        )
        .map_err(|e| PyRuntimeError::new_err(format!("arrow batch for {code}: {e}")))?;

        let schema = batch.schema();
        Ok(Some(PyTable::try_new(vec![batch], schema)?))
    }
}

/// A read-only handle to an `.ags.idx` validity **certificate** + byte-offset
/// index (the core `laterite_ags4_core::index::Sidecar`). `Ags4File.certify`
/// mints one over an already-clean file; `read(index=…)` loads + freshness-checks
/// one to skip re-validation. Core owns the format and can *read* a cert with no
/// validator, but cannot *mint* (it doesn't depend on the validator); minting fills
/// the validator identity here and trusts that the CALLER (the Python `certify`)
/// confirmed a clean validation first.
#[pyclass(name = "Sidecar")]
struct PySidecar {
    inner: CoreSidecar,
}

#[pymethods]
impl PySidecar {
    /// **Mint** a certificate for `data` — validating it here, first.
    ///
    /// This replaces `assemble`, whose signature was
    /// `(data, edition, checked_at, warnings=0, fyi=0, …)`: the caller told it what the
    /// verdict had been, and the DEFAULTS asserted a clean one. Nothing in the Python
    /// layer ever passed those two arguments, so every certificate this wheel ever
    /// produced recorded "0 warnings, 0 FYI" without anything having looked. A later
    /// `--show-warnings` request read the zero and skipped the engine.
    ///
    /// So there is no longer a parameter through which a caller can assert a verdict.
    /// `mint` runs the rules itself, with both tiers on, and records what they returned.
    /// It refuses a file with ERRORS (warnings are recorded, not fatal — a delivery may
    /// legitimately carry them). Raises `ValueError` on an uncertifiable or unparseable
    /// file.
    ///
    /// `retained` + `measured` are a read handle's parse and the findings of its latest
    /// validation. They are not a verdict a caller asserts: `measured` is the engine's own
    /// output, which Python cannot construct. The trust crate reuses it instead of running
    /// the rules again only if that run answered this mint's question, over `retained`,
    /// and `data` still parses to `retained` (#1096). Otherwise the rules run as before.
    #[staticmethod]
    #[pyo3(signature = (data, checked_at, dict_version=None, encoding=None, compat=None, dict_path=None, dict_bytes=None, dict_replace=false, retained=None, measured=None))]
    #[allow(clippy::too_many_arguments)]
    // PyO3 boundary: an optional pyclass argument arrives as an owned `PyRef`.
    #[allow(clippy::needless_pass_by_value)]
    fn mint(
        data: &[u8],
        checked_at: String,
        dict_version: Option<&str>,
        encoding: Option<&str>,
        compat: Option<String>,
        dict_path: Option<&str>,
        dict_bytes: Option<&[u8]>,
        dict_replace: bool,
        retained: Option<PyRef<'_, Reading>>,
        measured: Option<PyRef<'_, CheckedFindings>>,
    ) -> PyResult<Self> {
        let over = parse_dv(dict_version).map_err(pyo3::exceptions::PyValueError::new_err)?;
        let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "unknown encoding {:?}",
                encoding.unwrap_or("")
            ))
        })?;
        // The cert records WHICH custom dictionary judged the file (laterite-dev#568, O-48): a mint
        // against a `--dict` overlay stamps its {name, hash}, and a later read that names a
        // different dict revalidates rather than inheriting a stale verdict.
        let custom_dict = build_custom_dict(dict_path, dict_bytes, dict_replace, over, enc)
            .map_err(|(_, _, m)| pyo3::exceptions::PyValueError::new_err(m))?;
        let mut opts = CheckOptions::default();
        opts.dict_version = over;
        opts.custom_dict = custom_dict;
        opts.encoding = enc;
        let prior = retained
            .as_deref()
            .zip(measured.as_deref().and_then(|m| m.measured.as_ref()));
        let minted = if let Some((reading, m)) = prior {
            let out = laterite_ags4_trust::mint_measured(
                data,
                &opts,
                checked_at,
                compat,
                &reading.parsed,
                m,
            );
            if out.engine_ran {
                count_engine_run();
            }
            out.result
        } else {
            let minted = laterite_ags4_trust::mint(data, &opts, checked_at, compat);
            if !matches!(minted, Err(laterite_ags4_trust::MintError::Validate(_))) {
                count_engine_run();
            }
            minted
        };
        let inner = minted.map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Parse a certificate from its on-disk JSON bytes, rejecting an unknown
    /// format version. Raises `ValueError` on malformed / unsupported JSON.
    #[staticmethod]
    fn from_json(data: &[u8]) -> PyResult<Self> {
        let inner = CoreSidecar::from_json(data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Is this certificate still current for `data`? Strong check: format version
    /// + byte length + SHA-256. A mismatch means the source changed under the cert
    /// (its byte offsets and clean verdict are now lies), so it must be rebuilt.
    fn is_fresh_for(&self, data: &[u8]) -> bool {
        self.inner.is_fresh_for(data)
    }

    /// Serialise to the on-disk `.ags.idx` JSON (pretty). Raises on a serialize
    /// failure (not expected for a well-formed cert).
    fn to_json<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let bytes = self
            .inner
            .to_json()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(pyo3::types::PyBytes::new(py, &bytes))
    }

    /// The byte-offset index — `{group_code: [(start, end), …]}` in file order (an
    /// insertion-ordered dict). Locates each group's bytes for a sliced read.
    ///
    /// A **list** of spans, not one span: a code can appear in more than one section,
    /// and the single-span shape this replaces could only express the first — so a
    /// sliced read of a redeclared group silently returned a subset of its rows. A
    /// list with more than one entry means "re-parse the file"; it does not mean
    /// "pick one".
    fn index<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        for code in &self.inner.order {
            if let Some(spans) = self.inner.groups.get(code) {
                let list: Vec<(u64, u64)> = spans.clone();
                d.set_item(code, list)?;
            }
        }
        Ok(d)
    }

    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }
    #[getter]
    fn size(&self) -> u64 {
        self.inner.file.size
    }
    #[getter]
    fn sha256(&self) -> &str {
        &self.inner.file.sha256
    }
    /// The AGS edition the rules were run against.
    #[getter]
    fn edition(&self) -> &str {
        self.inner.validation.edition.edition()
    }
    /// Was that edition FORCED (`dict_version=`), or auto-resolved from `TRAN_AGS`?
    /// One fact with the edition string, not two — a forced run and an auto run can
    /// name the same edition having applied different dictionaries, and the predicate
    /// this replaces compared the two apart.
    #[getter]
    fn edition_forced(&self) -> bool {
        self.inner.validation.edition.is_forced()
    }
    #[getter]
    fn validator(&self) -> &str {
        &self.inner.validation.validator
    }
    /// The fingerprint of the rule engine that produced this verdict — a hash of the
    /// rule sources and the bundled dictionary, NOT the wheel's version. A rule can
    /// change without a version bump; this cannot.
    #[getter]
    fn engine(&self) -> &str {
        &self.inner.validation.engine
    }
    #[getter]
    fn compat(&self) -> Option<&str> {
        self.inner.validation.compat.as_deref()
    }
    #[getter]
    fn checked_at(&self) -> &str {
        &self.inner.validation.checked_at
    }
    /// The decoder the certified bytes were READ through (`"UTF-8"`, `"windows-1252"`, …).
    ///
    /// Part of the verdict, not trivia: the rules judge the TEXT the bytes decode to, and
    /// two decoders can reach two verdicts on one unchanged file. A certificate minted
    /// under one decoder does not answer a request made under another.
    #[getter]
    fn encoding(&self) -> &str {
        &self.inner.validation.encoding
    }
    #[getter]
    fn etag(&self) -> Option<&str> {
        self.inner.file.etag.as_deref()
    }
    #[getter]
    fn last_modified(&self) -> Option<&str> {
        self.inner.file.last_modified.as_deref()
    }

    /// Findings of each tier that the validation **measured** — or `None` if it never
    /// ran that tier's rules.
    ///
    /// `None` is the point. The old format stored a bare `int` that defaulted to `0`, so
    /// "I found no warnings" and "I never looked for warnings" were the same value, and
    /// a consumer could not tell them apart. It read the zero and trusted it.
    #[getter]
    fn errors(&self) -> Option<u32> {
        tier_count(self.inner.validation.errors)
    }
    #[getter]
    fn warnings(&self) -> Option<u32> {
        tier_count(self.inner.validation.warnings)
    }
    #[getter]
    fn fyi(&self) -> Option<u32> {
        tier_count(self.inner.validation.fyi)
    }

    #[getter]
    fn order(&self) -> Vec<String> {
        self.inner.order.clone()
    }

    /// The groups the file's own DICT declares (#768), sorted. `None` means the
    /// cert predates the field (nothing measured); `[]` means measured and the
    /// file declares nothing. Names only — the definitions stay in `DICT`,
    /// which `index` locates.
    #[getter]
    fn defines(&self) -> Option<Vec<String>> {
        self.inner.defines.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "<Sidecar v{} {} groups {} bytes edition={:?} by {} engine {}>",
            self.inner.version,
            self.inner.order.len(),
            self.inner.file.size,
            self.inner.validation.edition.edition(),
            self.inner.validation.validator,
            self.inner.validation.engine,
        )
    }
}

/// Parse `path` or `text` for `read()` / `scan()`: per-group metadata only
/// (`headings/units/types/line_numbers`). The typed Arrow table for a group is
/// NOT built here — it is built lazily, per group, on first touch via the
/// `_handle`'s `Reading::table_for` (cast by the one shared emitter
/// `laterite_ags4_types::arrow_cols`, byte-identical to the browser's IPC path). The
/// raw parse stays Rust-side in that `Reading` handle, also feeding
/// byte-faithful `write()`; no per-cell `PyObject` rows cross the boundary.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn parse_arrow(
    py: Python<'_>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'_, PyDict>> {
    let Some(enc) = laterite_ags4_parse::resolve_encoding(encoding.as_deref()) else {
        let label = encoding.as_deref().unwrap_or("");
        return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
    };
    if path.is_none() && text.is_none() && data.is_none() {
        return err_dict(py, 5, "bad_args", "one of path, text, or data is required");
    }
    // Release the GIL for the parse (pure Rust) so concurrent reads parallelise
    // across cores rather than serialising behind the interpreter lock (#8). The
    // input-presence check is above so the closure needs no GIL-bound early exit.
    let parsed = py.detach(
        || match (path.as_deref(), text.as_deref(), data.as_deref()) {
            (Some(p), _, _) => {
                laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
            }
            (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
            (_, _, Some(d)) => {
                laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from)
            }
            _ => unreachable!("input presence checked above"),
        },
    );
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(&e);
            return err_dict(py, c, &k, &m);
        }
    };

    let d = PyDict::new(py);
    d.set_item("ok", true)?;
    d.set_item("group_order", pf.group_order.clone())?;
    d.set_item("tran_ags", laterite_ags4_validator::tran_ags_of(&pf))?;

    let groups = PyDict::new(py);
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let gd = PyDict::new(py);
        gd.set_item("group_line", g.group_line)?;
        gd.set_item("heading_line", g.heading_line)?;
        gd.set_item("unit_line", g.unit_line)?;
        gd.set_item("type_line", g.type_line)?;
        gd.set_item("headings", g.headings.clone())?;
        gd.set_item("units", g.units.clone())?;
        gd.set_item("types", g.types.clone())?;
        // No eager line-number list either (#1094): one Python int per DATA row,
        // held for the handle's life, for a method few callers touch —
        // `Reading::line_numbers` reads it off the retained parse on ask.
        // No eager Arrow table here: the read path is per-group lazy — the
        // typed table is built on first touch via `Reading::table_for` (the
        // `_handle` below), so this loop only crosses cheap metadata.
        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    // Keep the parse Rust-side for byte-faithful write (no per-cell PyObject
    // rows cross; Reading::emit reproduces the source DATA values exactly).
    d.set_item("_handle", Reading { parsed: pf })?;
    Ok(d)
}

/// Resolve the bundled edition exactly as the engine does. Returns
/// `(version, resolution)` e.g. `("4.1.1", "fallback")`. Raises
/// `ValueError` on a bad override or an unsupported (AGS3) edition.
#[pyfunction]
#[pyo3(signature = (tran_ags=None, override_version=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn resolve_dict(
    tran_ags: Option<String>,
    override_version: Option<String>,
) -> PyResult<(String, String)> {
    let over =
        parse_dv(override_version.as_deref()).map_err(pyo3::exceptions::PyValueError::new_err)?;
    match laterite_ags4_validator::resolve_dict_version(over, tran_ags.as_deref()) {
        Ok((dv, res)) => Ok((dv.as_str().to_string(), res.as_str().to_string())),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
    }
}

/// Return the bundled standard dictionary's UNIT/TYPE for each
/// heading in one group, for a given edition. Used by
/// `laterite.compat.convert_to_text(df, dictionary='<version>')` to
/// recover UNIT/TYPE rows when the caller's frame has them dropped
/// (e.g. after `convert_to_numeric` which strips them).
///
/// Returns `{heading: (unit, ags_type)}` — only headings defined in
/// the standard dictionary for that group are included; non-standard
/// headings (the file's own DICT group's territory) return as missing
/// keys.
///
/// Raises `ValueError` on an unknown edition string.
#[pyfunction]
fn dict_group_unit_type<'py>(
    py: Python<'py>,
    edition: &str,
    group: &str,
) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
    let dv = parse_dv(Some(edition))
        .map_err(pyo3::exceptions::PyValueError::new_err)?
        .ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "edition cannot be empty/auto; pass one of {}",
                laterite_ags4_validator::editions_joined("/")
            ))
        })?;
    let dict = laterite_ags4_validator::dict::Dictionary::bundled(dv);
    let out = pyo3::types::PyDict::new(py);
    for h in dict.group_headings(group).iter() {
        if let Some(entry) = dict.heading(group, h) {
            out.set_item(*h, (entry.unit, entry.ags_type))?;
        }
    }
    Ok(out)
}

/// Raw group cells for the CLI `lat read` — the group `order` + per-group
/// `{headings, rows}` (rows as string lists in heading order), straight from
/// core's read codec (no typing). So the Rust binary and the Python `lat read`
/// agree byte-for-byte on `read --json` / `--csv` (#430 PR 2).
#[pyfunction]
#[pyo3(signature = (path, recover_duplicate_headings = false, truncate_excess_fields = false))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn read_groups_raw(
    py: Python<'_>,
    path: String,
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
) -> PyResult<Bound<'_, PyDict>> {
    let read_opts = laterite_ags4_core::ags4_codec::ReadOptions::from_flags(
        recover_duplicate_headings,
        truncate_excess_fields,
    );
    let parsed =
        laterite_ags4_core::ags4_codec::read_ags4_with(std::path::Path::new(&path), read_opts)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("order", PyList::new(py, parsed.order())?)?;
    let groups = PyDict::new(py);
    for code in parsed.order() {
        if let Some(g) = parsed.get(code) {
            let gd = PyDict::new(py);
            gd.set_item("headings", PyList::new(py, g.headings())?)?;
            let rows = PyList::empty(py);
            // Positional projection straight off the span-backed accessors
            // (#900) — one row's worth of borrowed cells at a time.
            for i in 0..g.n_rows() {
                let cells: Vec<&str> = g.row_cells(i).collect();
                rows.append(PyList::new(py, &cells)?)?;
            }
            gd.set_item("rows", rows)?;
            groups.set_item(code, gd)?;
        }
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

/// `lat read --json` for one group: the rendered string, from core's ONE JSON
/// writer. `_cli.py` used to build this with Python's `json.dumps(indent=2,
/// ensure_ascii=False)` while the binary used `serde_json` and Node used
/// `JSON.stringify` — three different JSON libraries kept byte-identical by
/// hand-discipline, with no gate on `read` output (laterite-dev#530).
#[pyfunction]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn render_read_json(headings: Vec<String>, rows: Vec<Vec<String>>) -> String {
    laterite_ags4_core::read_render::render_rows_json(&headings, &rows)
}

/// `lat read --csv` for one group: the rendered string, from core's ONE CSV
/// writer (RFC-4180-ish quoting). Replaces `_cli.py`'s hand-ported `_csv_row`.
#[pyfunction]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn render_read_csv(headings: Vec<String>, rows: Vec<Vec<String>>) -> String {
    laterite_ags4_core::read_render::render_rows_csv(&headings, &rows)
}

/// The validation engine's hand-bumped semver — see `engine_fingerprint`.
#[pyfunction]
fn engine_version() -> &'static str {
    laterite_ags4_validator::VERSION
}

/// The identity of the engine that produces verdicts — a build-time digest over
/// every rule source, the dictionary, and the rules catalogue.
///
/// Exposed because the wheel's own version cannot answer "which rules ran". The
/// two numbers are independent since the tiers split (#202): a wheel version says
/// which release you installed, this says what is inside it. Two surfaces
/// reporting the same fingerprint ARE running the same engine; two reporting the
/// same release number merely shipped together.
#[pyfunction]
fn engine_fingerprint() -> &'static str {
    laterite_ags4_validator::ENGINE_FINGERPRINT
}

#[pymodule]
fn _laterite_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    m.add_function(wrap_pyfunction!(engine_fingerprint, m)?)?;
    m.add_function(wrap_pyfunction!(engine_runs, m)?)?;
    m.add_function(wrap_pyfunction!(run_check, m)?)?;
    m.add_function(wrap_pyfunction!(fix_file, m)?)?;
    m.add_function(wrap_pyfunction!(staged_write, m)?)?;
    m.add_function(wrap_pyfunction!(list_rules, m)?)?;
    m.add_function(wrap_pyfunction!(diff_files, m)?)?;
    m.add_function(wrap_pyfunction!(merge_files, m)?)?;
    m.add_function(wrap_pyfunction!(parse_primitives, m)?)?;
    m.add_function(wrap_pyfunction!(parse_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(parse_compat_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(read_groups_raw, m)?)?;
    m.add_function(wrap_pyfunction!(render_read_json, m)?)?;
    m.add_function(wrap_pyfunction!(render_read_csv, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_dict, m)?)?;
    m.add_function(wrap_pyfunction!(dict_group_unit_type, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_from_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(
        emit_typed::emit_ags4_from_arrow_unchecked,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_compat, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_compat_to_path, m)?)?;
    m.add_class::<PySidecar>()?;
    registry_fns::register(m)?;
    ags_types_fns::register(m)?;
    transport_fns::register(m)?;
    typed_graph::register(m)?;
    excel_fns::register(m)?;
    Ok(())
}
