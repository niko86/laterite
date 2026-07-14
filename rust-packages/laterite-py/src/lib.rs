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
//! README): the Rust-CLI shape `{file, findings:{rule:[...]}}` is
//! built *here* with the same `serde_json` (`preserve_order`) calls
//! the `lat` binary uses, so `--json`/`--ndjson` are
//! byte-faithful; the python-ags4 `check_file` dict (with
//! `Metadata`/`Summary`) is assembled in `laterite/compat.py`.

use std::path::Path;

use laterite_ags4_core::error::CliError;
use laterite_ags4_core::index::{Sidecar as CoreSidecar, TierCoverage};
use laterite_ags4_validator::findings::{Severity, Target};
use laterite_ags4_validator::fixes::FixRisk;
use laterite_ags4_validator::{
    CheckOptions, DictVersion, Dictionary, Findings, Fix, ValidatorError, WorldScope,
    fix_document_selective,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3_arrow::PyTable;
use serde_json::{Map, Value};

// S3b (release/v0.1.0-prep): `ags5db_fns` moved to the separate
// `laterite-py-ags5` cdylib. Base wheel ships without DuckDB; the
// AGS5 surface is gated behind the `laterite[ags5]` extra.
mod ags_types_fns;
mod emit;
mod emit_typed;
mod excel_fns;
mod registry_fns;
mod transport_fns;
mod typed_graph;

/// Map an `laterite-ags4-core` `CliError` to a PyRuntimeError, preserving the
/// exit-code label python callers may surface in error messages.
/// Previously lived in `ags5db_fns.rs`; that module moved to the
/// AGS5 cdylib in S3b, so the base-wheel transport + excel functions
/// share this local copy instead.
pub(crate) fn map_cli_err(e: CliError) -> PyErr {
    let code = e.exit_code();
    PyRuntimeError::new_err(format!("laterite error (exit {code}): {e}"))
}

/// Map a `--dict-version` string to the optional override. `None` /
/// `"auto"` ⇒ no override (TRAN_AGS auto-pick). Unknown ⇒ `Err`.
pub(crate) fn parse_dv(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s {
        None | Some("auto") | Some("") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            format!(
                "unknown --dict-version {other:?} (expected auto|{})",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

/// (exit_code, error_kind, message) for a validator error — exit codes
/// mirror the `lat` binary exactly (3 not-found/io, 4
/// not-utf8/not-ags4/unsupported-edition, 5 bad-dict).
fn map_err(e: ValidatorError) -> (i32, String, String) {
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

/// What a validation produced, plus whether a certificate stood in for the rule engine.
struct Checked {
    file: String,
    dict_version: String,
    resolution: String,
    findings: Findings,
    /// A cert answered the CONTENT half. The world half (Rule 20 on-disk) ran regardless.
    certified: bool,
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
    cert: Option<&CoreSidecar>,
) -> Result<Checked, (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let opts = CheckOptions {
        dict_version: over,
        custom_dict: None,
        include_warnings: warnings,
        include_fyi: fyi,
        check_files,
        encoding: enc,
    };

    // Bytes, whatever door they came through — a certificate is a statement about bytes.
    let (label, bytes, world) = if let Some(p) = path {
        let b = std::fs::read(Path::new(p)).map_err(|e| {
            let kind = if e.kind() == std::io::ErrorKind::NotFound {
                "not_found"
            } else {
                "io"
            };
            (3, kind.to_string(), format!("{p}: {e}"))
        })?;
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

    let out = laterite_ags4_trust::check(laterite_ags4_trust::Request {
        bytes: &bytes,
        opts: &opts,
        cert,
        world,
        compat: None,
    })
    .map_err(map_err)?;

    Ok(Checked {
        file: label,
        dict_version: out.dict_version.as_str().to_string(),
        resolution: out.resolution.as_str().to_string(),
        findings: out.findings,
        certified: out.certified,
    })
}

/// Lowercase serde-name of a [`Target`] for the structured PyDict
/// (matches the JSON `target` value).
fn target_str(t: Target) -> &'static str {
    match t {
        Target::Line => "line",
        Target::Heading => "heading",
        Target::Cell => "cell",
        Target::Group => "group",
    }
}

/// Lowercase serde-name of a [`Severity`] for the structured PyDict — delegates
/// to the single producer so it can't drift from the serde token.
fn severity_str(s: Severity) -> &'static str {
    s.as_str()
}

/// `{file, findings:{ "AGS Format Rule N":[{line,group,desc}] }}` —
/// the exact `serde_json` value + insertion order the `lat`
/// binary's `json_value`/`json_string` produce. `preserve_order` (set
/// in Cargo.toml, matching the validator) keeps the key order stable.
fn findings_json(file: &str, found: &Findings) -> String {
    let mut fmap = Map::new();
    for (rule, items) in found {
        // Serialize the engine `Finding` directly — byte-faithful to the
        // CLI's `json_value`. Unset location/severity fields skip, so
        // line-only findings stay `{line,group,desc}`; migrated findings
        // additively gain the rich keys.
        let arr: Vec<Value> = items
            .iter()
            .map(|f| serde_json::to_value(f).unwrap_or(Value::Null))
            .collect();
        fmap.insert(rule.clone(), Value::Array(arr));
    }
    let mut root = Map::new();
    root.insert("file".into(), Value::from(file.to_string()));
    root.insert("findings".into(), Value::Object(fmap));
    serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default()
}

/// One flat JSON object per finding per line (NDJSON) — byte-identical
/// to the binary's `ndjson_string`.
fn findings_ndjson(found: &Findings) -> String {
    let mut s = String::new();
    for (rule, items) in found {
        for f in items {
            // `rule`-first then the serialized `Finding` body — byte-faithful
            // to the CLI's `ndjson_string`.
            let mut o = Map::new();
            o.insert("rule".into(), Value::from(rule.clone()));
            if let Value::Object(body) = serde_json::to_value(f).unwrap_or(Value::Null) {
                o.extend(body);
            }
            s.push_str(&serde_json::to_string(&Value::Object(o)).unwrap_or_default());
            s.push('\n');
        }
    }
    s
}

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
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, include_warnings=false, include_fyi=false, check_files=false, encoding=None, cert=None))]
#[allow(clippy::too_many_arguments)]
fn run_check<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    check_files: bool,
    encoding: Option<String>,
    // The certificate, if the caller named one. The decision to trust it is NOT made in
    // Python — it is made once, in `laterite_ags4_trust`, alongside every other surface.
    // The Python layer used to make it itself, with its own conjunction of predicates.
    cert: Option<PyRef<'py, PySidecar>>,
) -> PyResult<Bound<'py, PyDict>> {
    match validate(
        path.as_deref(),
        text.as_deref(),
        data.as_deref(),
        dict_version.as_deref(),
        include_warnings,
        include_fyi,
        check_files,
        encoding.as_deref(),
        cert.as_ref().map(|c| &c.inner),
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok(checked) => {
            let Checked {
                file,
                dict_version: dv,
                resolution: res,
                findings: found,
                certified,
            } = checked;
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("file", &file)?;
            d.set_item("dict_version", dv)?;
            d.set_item("resolution", res)?;
            // Whether the rule ENGINE was skipped. Never "whether the file was checked":
            // a world check (Rule 20 on-disk) runs even on a certified read.
            d.set_item("certified", certified)?;

            let mut count = 0usize;
            let items = PyList::empty(py);
            for (rule, fs) in &found {
                for f in fs {
                    count += 1;
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
            d.set_item("count", count)?;
            d.set_item("exit_code", if count == 0 { 0 } else { 1 })?;
            d.set_item("findings", items)?;
            d.set_item("json", findings_json(&file, &found))?;
            d.set_item("ndjson", findings_ndjson(&found))?;
            Ok(d)
        }
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
        let kind = serde_json::to_value(f.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        o.set_item("kind", kind)?;
        o.set_item("label", &f.label)?;
        o.set_item("rule", &f.rule)?;
        o.set_item("line", f.line)?;
        o.set_item(
            "risk",
            match f.risk {
                FixRisk::Safe => "safe",
                FixRisk::Risky => "risky",
            },
        )?;
        list.append(o)?;
    }
    Ok(list)
}

/// Headless one-shot mechanical repair of a delivered AGS4 file (the same engine
/// the browser fix UI uses, applied without a UI). Reads from path/text/data,
/// computes fixes against the file's own findings, applies the *safe* set (plus
/// the intent-guessing *risky* set when `include_risky`), and re-validates the
/// result. Returns `(fixed_bytes, residual_findings, applied_fixes, dict_version,
/// resolution)` or the `(exit_code, kind, msg)` error triple. Mirrors the
/// AutoFix path in `laterite-ags4-emit::emit`, but on a file's own bytes rather
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
) -> Result<(Vec<u8>, Findings, Vec<Fix>, String, String, usize), (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
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
    let opts = CheckOptions {
        dict_version: over,
        custom_dict: None,
        include_warnings: true,
        include_fyi: false,
        check_files: false,
        encoding: enc,
    };
    let out = fix_document_selective(&raw, &opts, include_risky, only, exclude).map_err(map_err)?;
    Ok((
        out.fixed,
        out.residual,
        out.applied,
        out.dict_version.as_str().to_string(),
        out.resolution.as_str().to_string(),
        out.risky_available,
    ))
}

/// Headless mechanical fix of `path`/`text`/`data`. Returns a dict the Python
/// `fix()` turns into a `FixResult` (`fixed` bytes, `findings_json` residual
/// `{rule:[…]}`, the `applied` fix list, `fixes_applied` count). On
/// un-fixable input returns the same `{ok:false, …}` shape as `run_check`, so
/// the Python layer raises the mapped exception.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, encoding=None, include_risky=false, only=None, exclude=None))]
#[allow(clippy::too_many_arguments)]
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
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok((bytes, residual, applied, dv, res, risky_available)) => {
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
        .map_err(map_err)?;
    let pb = laterite_ags4_parse::parse_bytes(b, enc)
        .map_err(ValidatorError::from)
        .map_err(map_err)?;
    // KEY headings come from the dictionary; pick the edition from the revision
    // (b)'s TRAN_AGS (forced by dict_version), falling back to the standard.
    let tran = laterite_ags4_validator::tran_ags_of(&pb);
    let dv = laterite_ags4_validator::resolve_dict_version(over, tran.as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);
    Ok(serde_json::to_string(&delta).unwrap_or_else(|_| "{}".to_string()))
}

/// Compare two AGS4 documents (raw `a`/`b` bytes). Returns `{ok:true,
/// delta_json}` (the serialised `RevisionDelta`, which the Python layer parses)
/// or the `{ok:false, error_kind, exit_code, error}` failure dict.
#[pyfunction]
#[pyo3(signature = (a, b, dict_version=None, encoding=None))]
fn diff_files<'py>(
    py: Python<'py>,
    a: Vec<u8>,
    b: Vec<u8>,
    dict_version: Option<String>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
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
    dvr: Option<&str>,
    encoding: Option<&str>,
    tran: (
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
    ),
) -> Result<(Vec<u8>, String, String), (i32, String, String)> {
    use laterite_ags4_merge::{MergeError, MergeOpts, TranStamp, TypeClashMode, merge_parsed};

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
                .map_err(map_err)
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
    .map(|(dv, _)| dv)
    .unwrap_or(laterite_ags4_validator::dict::FALLBACK);

    // A merge-TRAN is synthesised only when both an issue and a date are given.
    let (isno, date, prod, recv, stat) = tran;
    let tran = match (isno, date) {
        (Some(isno), Some(date)) => Some(TranStamp {
            isno: isno.to_string(),
            date: date.to_string(),
            prod: prod.unwrap_or_default().to_string(),
            recv: recv.unwrap_or_default().to_string(),
            stat: stat.unwrap_or_default().to_string(),
            ags: dv.as_str().to_string(),
        }),
        _ => None,
    };

    let opts = MergeOpts {
        on_type_clash: clash,
        edition: dv,
        tran,
        ..Default::default()
    };

    match merge_parsed(&parsed, &opts) {
        Ok(res) => {
            let warnings: Vec<_> = res
                .warnings
                .iter()
                .map(|w| {
                    serde_json::json!({
                        "kind": w.kind,
                        "group": w.group,
                        "heading": w.heading,
                        "message": w.message,
                    })
                })
                .collect();
            let revisions: Vec<_> = res
                .revisions
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "group": r.group,
                        "key": r.key,
                        "changed": r.changed,
                        "winner_file": r.winner_file,
                    })
                })
                .collect();
            Ok((
                res.bytes,
                serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".to_string()),
                serde_json::to_string(&revisions).unwrap_or_else(|_| "[]".to_string()),
            ))
        }
        // An unresolved TYPE conflict / emit failure is a schema-level rejection
        // (exit 6), matching the `lat merge` codes.
        Err(e @ MergeError::TypeConflict { .. }) => {
            Err((6, "type_conflict".to_string(), e.to_string()))
        }
        // Fatal in EVERY mode — no `on_type_clash` value absorbs it (#501). Its own
        // kind so a caller can tell "your files disagree on a column's TYPE, which
        // widen/promote CAN settle" from "your files disagree on its UNIT, which
        // nothing can" — the two need different fixes.
        Err(e @ MergeError::UnitConflict { .. }) => {
            Err((6, "unit_conflict".to_string(), e.to_string()))
        }
        Err(e @ MergeError::Emit(_)) => Err((6, "emit_error".to_string(), e.to_string())),
    }
}

/// Merge raw AGS4 documents (`files`, ≥2). Returns `{ok:true, merged (bytes),
/// warnings_json, revisions_json}` — the Python layer parses the two JSON
/// strings — or the `{ok:false, error_kind, exit_code, error}` failure dict.
#[pyfunction]
#[pyo3(signature = (files, on_type_clash="error", dict_version=None, encoding=None, tran_issue=None, tran_date=None, tran_producer=None, tran_recipient=None, tran_status=None))]
#[allow(clippy::too_many_arguments)]
fn merge_files<'py>(
    py: Python<'py>,
    files: Vec<Vec<u8>>,
    on_type_clash: &str,
    dict_version: Option<String>,
    encoding: Option<String>,
    tran_issue: Option<String>,
    tran_date: Option<String>,
    tran_producer: Option<String>,
    tran_recipient: Option<String>,
    tran_status: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let tran = (
        tran_issue.as_deref(),
        tran_date.as_deref(),
        tran_producer.as_deref(),
        tran_recipient.as_deref(),
        tran_status.as_deref(),
    );
    match merge_core(
        &files,
        on_type_clash,
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
fn parse_primitives<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match laterite_ags4_parse::resolve_encoding(encoding.as_deref()) {
        Some(e) => e,
        None => {
            let label = encoding.as_deref().unwrap_or("");
            return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
        }
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
            let (c, k, m) = map_err(e);
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
            ro.set_item("values", r.values.clone())?;
            rows.append(ro)?;
        }
        gd.set_item("rows", rows)?;
        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

/// A parsed AGS4 file held Rust-side. `parse_arrow` crosses only cheap
/// per-group metadata; the raw `ParsedFile` stays HERE and serves two roles
/// on demand: `table_for` builds a group's typed Arrow table on first touch
/// (per-group lazy reads), and `emit` re-emits byte-faithful AGS4 for
/// `write()`. No O(cells) PyObject dict crosses the boundary, and there is no
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
        let blocks: Vec<emit::GroupBlock> = self
            .parsed
            .group_order
            .iter()
            .filter_map(|code| {
                let g = self.parsed.groups.get(code)?;
                let n = g.headings.len();
                // tag + each of the n columns, padded/truncated like the old
                // Python `_matrix` (a ragged DATA row fills its tail with "").
                let pad = |tag: &str, src: &[String]| {
                    let mut row = Vec::with_capacity(n + 1);
                    row.push(tag.to_string());
                    for i in 0..n {
                        row.push(src.get(i).cloned().unwrap_or_default());
                    }
                    row
                };
                let mut matrix: Vec<Vec<String>> = Vec::with_capacity(3 + g.rows.len());
                let mut heading = Vec::with_capacity(n + 1);
                heading.push("HEADING".to_string());
                heading.extend(g.headings.iter().cloned());
                matrix.push(heading);
                matrix.push(pad("UNIT", &g.units));
                matrix.push(pad("TYPE", &g.types));
                for r in &g.rows {
                    matrix.push(pad("DATA", &r.values));
                }
                Some(emit::GroupBlock {
                    code: code.clone(),
                    matrix,
                })
            })
            .collect();
        emit::emit(&blocks)
    }

    /// Build ONE group's typed Arrow table on demand. The read path is
    /// per-group lazy: `read()` / `scan()` only pay for the groups actually
    /// touched, so a 69-group file you query two groups of builds two
    /// RecordBatches, not 69. Returns `None` if `code` isn't in the file.
    /// Same shared emitter (`laterite_types::arrow_cols`) as the old eager build —
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
    #[pyo3(signature = (code, content_hash=false))]
    fn table_for(&self, code: &str, content_hash: bool) -> PyResult<Option<PyTable>> {
        let Some(g) = self.parsed.groups.get(code) else {
            return Ok(None);
        };
        // Relational layer is **always-keyed**: a KNOWN group's batch carries the
        // two content-addressed key columns (`_id` col 0, `_parent_id` col 1) so a
        // cross-group join in `.sql()` works with no opt-in — the Python frame
        // accessor strips them by default. The ids come from the one shared
        // keychain (`group_row_ids` → `keychain::row_ids`), so they are byte-
        // identical to the `.ags5db` extension's. A custom/passthrough group is
        // absent from the registry → it has no spec keys → unkeyed batch (#303).
        let reg = laterite_ags4_core::registry::registry();
        let batch = if reg.get(code).is_some() {
            let ids = laterite_ags4_core::keychain::group_row_ids(
                reg,
                code,
                &g.headings,
                g.rows.len(),
                |col, row| g.cell(col, row),
            );
            laterite_types::arrow_cols::build_record_batch_with_ids(
                &ids,
                &g.headings,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            )
        } else {
            laterite_types::arrow_cols::build_record_batch(
                &g.headings,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            )
        }
        .map_err(|e| PyRuntimeError::new_err(format!("arrow batch for {code}: {e}")))?;

        // Unlike the ids above, this needs NO registry — it hashes every
        // heading rather than the spec key-chain, so a custom/passthrough group
        // (which gets no `_id` at all) still gets a usable value fingerprint.
        // Types come from the file's OWN TYPE row: canonicalisation is per-file,
        // which is why two files that disagree on a column's TYPE may not dedup.
        let batch = if content_hash {
            let hashes = laterite_ags4_core::keychain::group_content_hashes(
                code,
                &g.headings,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            );
            let mut fields: Vec<std::sync::Arc<arrow::datatypes::Field>> =
                batch.schema().fields().iter().cloned().collect();
            fields.push(std::sync::Arc::new(arrow::datatypes::Field::new(
                "_content_hash",
                arrow::datatypes::DataType::Utf8,
                false,
            )));
            let mut columns = batch.columns().to_vec();
            columns.push(std::sync::Arc::new(arrow::array::StringArray::from(hashes)));
            arrow::record_batch::RecordBatch::try_new(
                std::sync::Arc::new(arrow::datatypes::Schema::new(fields)),
                columns,
            )
            .map_err(|e| PyRuntimeError::new_err(format!("_content_hash for {code}: {e}")))?
        } else {
            batch
        };

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
    #[staticmethod]
    #[pyo3(signature = (data, checked_at, dict_version=None, encoding=None, compat=None))]
    fn mint(
        data: &[u8],
        checked_at: String,
        dict_version: Option<&str>,
        encoding: Option<&str>,
        compat: Option<String>,
    ) -> PyResult<Self> {
        let over = parse_dv(dict_version).map_err(pyo3::exceptions::PyValueError::new_err)?;
        let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "unknown encoding {:?}",
                encoding.unwrap_or("")
            ))
        })?;
        let opts = CheckOptions {
            dict_version: over,
            encoding: enc,
            ..CheckOptions::default()
        };
        let inner = laterite_ags4_trust::mint(data, &opts, checked_at, compat)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
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
/// (headings/units/types/line_numbers). The typed Arrow table for a group is
/// NOT built here — it is built lazily, per group, on first touch via the
/// `_handle`'s `Reading::table_for` (cast by the one shared emitter
/// `laterite_types::arrow_cols`, byte-identical to the browser's IPC path). The
/// raw parse stays Rust-side in that `Reading` handle, also feeding
/// byte-faithful `write()`; no per-cell PyObject rows cross the boundary.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None))]
fn parse_arrow<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match laterite_ags4_parse::resolve_encoding(encoding.as_deref()) {
        Some(e) => e,
        None => {
            let label = encoding.as_deref().unwrap_or("");
            return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
        }
    };
    let parsed = match (path.as_deref(), text.as_deref(), data.as_deref()) {
        (Some(p), _, _) => {
            laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
        }
        (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
        (_, _, Some(d)) => laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from),
        _ => return err_dict(py, 5, "bad_args", "one of path, text, or data is required"),
    };
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(e);
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
        gd.set_item(
            "line_numbers",
            g.rows.iter().map(|r| r.line).collect::<Vec<_>>(),
        )?;
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
    for h in dict.group_headings(group) {
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
fn read_groups_raw<'py>(py: Python<'py>, path: String) -> PyResult<Bound<'py, PyDict>> {
    let parsed = laterite_ags4_core::ags4_codec::read_ags4(std::path::Path::new(&path))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("order", PyList::new(py, &parsed.order)?)?;
    let groups = PyDict::new(py);
    for code in &parsed.order {
        if let Some(g) = parsed.get(code) {
            let gd = PyDict::new(py);
            gd.set_item("headings", PyList::new(py, &g.headings)?)?;
            let rows = PyList::empty(py);
            for row in &g.rows {
                let cells: Vec<&str> = g
                    .headings
                    .iter()
                    .map(|h| row.get(h).map(String::as_str).unwrap_or(""))
                    .collect();
                rows.append(PyList::new(py, &cells)?)?;
            }
            gd.set_item("rows", rows)?;
            groups.set_item(code, gd)?;
        }
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

#[pymodule]
fn _laterite_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_check, m)?)?;
    m.add_function(wrap_pyfunction!(fix_file, m)?)?;
    m.add_function(wrap_pyfunction!(list_rules, m)?)?;
    m.add_function(wrap_pyfunction!(diff_files, m)?)?;
    m.add_function(wrap_pyfunction!(merge_files, m)?)?;
    m.add_function(wrap_pyfunction!(parse_primitives, m)?)?;
    m.add_function(wrap_pyfunction!(parse_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(read_groups_raw, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_dict, m)?)?;
    m.add_function(wrap_pyfunction!(dict_group_unit_type, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_from_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_compat, m)?)?;
    m.add_class::<PySidecar>()?;
    registry_fns::register(m)?;
    ags_types_fns::register(m)?;
    transport_fns::register(m)?;
    typed_graph::register(m)?;
    excel_fns::register(m)?;
    Ok(())
}
