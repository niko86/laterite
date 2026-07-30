//! Node-API (napi-rs) bindings for the laterite AGS4 engine — the Node
//! analog of `laterite-py`. Re-expresses the engine surface through `#[napi]`:
//! parse → typed **Arrow IPC `Buffer`** per group (the Node analog of
//! laterite-py's pyo3-arrow capsule, exactly what `laterite-ags4-wasm` frames for the
//! browser), validate, emit. The TS `laterite` package layers the high-level
//! API on top. napi auto-camelCases names (`table_ipc` → `tableIpc`).

use std::io::Cursor;
use std::path::{Path, PathBuf};

// The read/validate hot-path is allocation-bound in the parse leaf (~5M small
// allocations for a 25 MB file — dhat, perf-campaign T4-followup), so the
// allocator's per-alloc cost dominates. mimalloc's per-thread heaps buy the same
// ~22% end-to-end read win the wheel measured; the addon frames each group as
// Arrow IPC bytes that JS decodes host-side, so Rust frees what Rust allocated.
// Set here because only the final cdylib can choose the allocator.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use arrow::ipc::reader::StreamReader;
use laterite_ags4_validator::dict::Dictionary;
use laterite_ags4_validator::findings::{Findings, Severity};
use laterite_ags4_validator::fixes::Fix;
// #168 Phase 3: text/bytes parse through the leaf directly; the FS entry
// (`parse_file_with_encoding`) stays in the validator (it owns NotFound/Io).
use laterite_ags4_parse::{ParsedFile, parse_bytes, parse_str};
use laterite_ags4_validator::parse::parse_file_with_encoding;
use laterite_ags4_validator::{
    CheckOptions, DictVersion, ValidatorError, WorldScope, fix_document_selective, overlay,
    rule_metadata_json, tran_ags_of,
};
use laterite_ags4_types::sql_type;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Map, Value};

mod ags_types_fns;
mod transport_fns;

// --- error protocol -----------------------------------------------------
//
// The Node analog of laterite-py's `{ok:false, error_kind, exit_code}` failure
// dict + `_errors.py::raise_for`. Hard failures (missing / not-AGS4 / bad
// edition) carry a `kind` + `exit_code` so the TS layer maps them to the right
// `Ags4Error` subclass WITHOUT brittle message-matching — byte-faithful to the
// `lat` exit codes (3 not-found/io, 4 not-utf8/not-ags4/unsupported-
// edition, 5 bad-dict/bad-args). `runCheck` returns the failure as data (an
// object, mirroring Python's dict); `parseArrow` returns a `Reading` *handle*,
// so it can't carry an `{ok}` field and instead THROWS this — a `\u{1f}`
// (unit-separator) delimited `kind␟code␟message` the TS `fromNativeError`
// recovers.

const SEP: char = '\u{1f}';

/// `(exit_code, error_kind)` for a `ValidatorError` — a thin alias over the
/// single producers `ValidatorError::exit_code()` / `::kind()` so the codes and
/// tokens can't drift from the validator crate. The message is the `Display`.
fn classify(e: &ValidatorError) -> (i32, &'static str) {
    (e.exit_code(), e.kind())
}

/// A `ValidatorError` as a thrown napi error (for the handle-returning
/// `parseArrow`): `kind␟code␟message`, recovered by the TS `fromNativeError`.
// Internal helper (not a napi boundary) — `classify`/`Display` only ever need
// `&e`, so each call site borrows the propagated error instead of moving it.
fn thrown(e: &ValidatorError) -> Error {
    let (code, kind) = classify(e);
    Error::from_reason(format!("{kind}{SEP}{code}{SEP}{e}"))
}

/// The crate version.
#[napi]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// --- parse → typed Arrow ------------------------------------------------

/// Per-group schema — parallel arrays, one entry per heading.
#[napi(object)]
pub struct GroupMeta {
    pub headings: Vec<String>,
    pub units: Vec<String>,
    /// AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID").
    pub types: Vec<String>,
    /// The SQL/DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
    /// "TIMESTAMP", "VARCHAR", …).
    pub sql_types: Vec<String>,
    /// 1-indexed source line of each DATA row (parallel to the group's rows).
    pub line_numbers: Vec<u32>,
}

/// A parsed AGS4 file held native-side — the Node analog of laterite-py's
/// `Reading` handle (and `laterite-ags4-wasm`'s `ParsedDataset`). Each group's typed
/// `RecordBatch` is built lazily on `tableIpc(code)` and dropped after the
/// bytes are returned, so peak residency is one batch.
#[napi]
pub struct Reading {
    parsed: ParsedFile,
}

#[napi]
impl Reading {
    /// Group codes in file order (the order to load tables in).
    #[napi]
    #[must_use]
    pub fn group_codes(&self) -> Vec<String> {
        self.parsed.group_order.clone()
    }

    /// The file's `TRAN_AGS` edition string, if present.
    #[napi(getter)]
    #[must_use]
    pub fn tran_ags(&self) -> Option<String> {
        tran_ags_of(&self.parsed)
    }

    /// `{headings, units, types, sqlTypes}` for one group, or `null` if the
    /// code isn't present. No Arrow built — cheap metadata only.
    #[napi]
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
    pub fn meta(&self, code: String) -> Option<GroupMeta> {
        let group = self.parsed.groups.get(&code)?;
        let n = group.headings.len();
        let types: Vec<String> = (0..n)
            .map(|i| {
                group
                    .types
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| "X".to_string())
            })
            .collect();
        Some(GroupMeta {
            headings: group.headings.clone(),
            units: (0..n)
                .map(|i| group.units.get(i).cloned().unwrap_or_default())
                .collect(),
            sql_types: types.iter().map(|t| sql_type(t).to_string()).collect(),
            types,
            line_numbers: group.rows.iter().map(|r| r.line).collect(),
        })
    }

    /// One group's rows as an Arrow **IPC stream** (`Buffer`), columns already
    /// correctly typed. The Node analog of the pyo3-arrow capsule: the typed
    /// columns come from the one shared emitter (`laterite_ags4_types::arrow_cols`), the
    /// SAME casting Python/wasm use — so a file types byte-identically across
    /// hosts. Returns `null` if the code isn't in the file.
    #[napi]
    #[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
    pub fn table_ipc(
        &self,
        code: String,
        content_hash: Option<bool>,
        with_keys: Option<bool>,
    ) -> Result<Option<Buffer>> {
        let Some(group) = self.parsed.groups.get(&code) else {
            return Ok(None);
        };
        // Relational layer is always-keyed: a KNOWN group's IPC carries the two
        // content-addressed key columns (`_id` col 0, `_parent_id` col 1) so a
        // cross-group join in `sql()`/`at()` works with no opt-in — the TS
        // `table()` accessor strips them by default. Ids come from the one shared
        // keychain (byte-identical to the DuckDB extension). A custom/passthrough
        // group is absent from the registry → unkeyed IPC (#303).
        //
        // `with_keys` defaults ON (that relational contract). A keys-less read
        // that never joins — the DEFAULT `table(code)`, which strips the columns
        // anyway — passes `false` to skip the keychain compute wholesale rather
        // than build-then-discard it (candidate #6/T6). The two-cache split in
        // `Ags4File` keeps a later `sql()`/`at()` on the same group correct.
        //
        // The hash needs NO registry — it hashes every heading rather than the
        // spec key-chain, so a custom/passthrough group (which gets no `_id` at
        // all) still gets a usable value fingerprint. Keying and hashing are
        // independent knobs, both folded in by the ONE shared
        // `build_group_ipc_synth` (`_id`/`_parent_id` col 0/1, `_content_hash`
        // trailing) so every host gets identical column order by construction —
        // mirrors laterite-py's `Reading::table_for`.
        let reg = laterite_ags4_core::registry::registry();
        let ids = if with_keys.unwrap_or(true) {
            reg.get(&code).map(|_| {
                laterite_ags4_core::keychain::group_row_ids(
                    reg,
                    &code,
                    &group.headings,
                    group.rows.len(),
                    |col, row| group.cell(col, row),
                )
            })
        } else {
            None
        };
        let hashes = if content_hash.unwrap_or(false) {
            Some(laterite_ags4_core::keychain::group_content_hashes(
                &code,
                &group.headings,
                &group.units,
                &group.types,
                group.rows.len(),
                |col, row| group.cell(col, row),
            ))
        } else {
            None
        };
        let buf = laterite_ags4_types::ipc::build_group_ipc_synth(
            &laterite_ags4_types::arrow_cols::SynthColumns {
                ids: ids.as_deref(),
                hashes: hashes.as_deref(),
            },
            &group.headings,
            &group.types,
            group.rows.len(),
            |col, row| group.cell(col, row),
        )
        .map_err(|e| Error::from_reason(format!("arrow ipc for {code}: {e}")))?;
        Ok(Some(buf.into()))
    }

    /// Re-emit byte-faithful AGS4 text from the retained parse (the raw DATA
    /// values, unchanged). = laterite-py's `Reading::emit`.
    #[napi]
    pub fn emit(&self) -> Result<String> {
        // EmitGroup borrows `&str`, so build an owned mirror first. Pad each
        // DATA row to the heading count (a ragged row fills its tail with "").
        struct Owned {
            code: String,
            headings: Vec<String>,
            units: Vec<String>,
            types: Vec<String>,
            rows: Vec<Vec<String>>,
        }
        let owned: Vec<Owned> = self
            .parsed
            .group_order
            .iter()
            .filter_map(|code| {
                let g = self.parsed.groups.get(code)?;
                let n = g.headings.len();
                Some(Owned {
                    code: code.clone(),
                    headings: g.headings.clone(),
                    units: g.units.clone(),
                    types: g.types.clone(),
                    rows: g
                        .rows
                        .iter()
                        .map(|r| {
                            (0..n)
                                .map(|i| r.values.get(i).cloned().unwrap_or_default())
                                .collect()
                        })
                        .collect(),
                })
            })
            .collect();
        let groups: Vec<laterite_ags4_emit::EmitGroup<'_>> = owned
            .iter()
            .map(|o| laterite_ags4_emit::EmitGroup {
                code: &o.code,
                headings: o.headings.iter().map(String::as_str).collect(),
                units: o.units.iter().map(String::as_str).collect(),
                types: o.types.iter().map(String::as_str).collect(),
                rows: o.rows.clone(),
            })
            .collect();
        let mut buf = Vec::new();
        laterite_ags4_emit::write_ags4(&mut buf, &groups)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        String::from_utf8(buf).map_err(|e| Error::from_reason(format!("emit utf8: {e}")))
    }
}

/// Parse an AGS4 file (`path`), in-memory `text`, or raw `data` bytes into a
/// `Reading` handle. `encoding`: `"utf-8"` (default) / `"windows-1252"` / a label
/// — applies to `path` / `data` (text is already decoded). Throws the classified
/// `kind␟code␟message` (see the error-protocol note) on bad input.
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn parse_arrow(
    path: Option<String>,
    text: Option<String>,
    data: Option<Uint8Array>,
    encoding: Option<String>,
) -> Result<Reading> {
    // `text` wins, then `data` (raw bytes — V8's ~512 MB string cap doesn't apply,
    // so a web backend can hand a large upload straight in without `.toString()`),
    // then a `path` the engine reads itself. Text is already decoded UTF-8 →
    // `parse_str`; bytes / path decode with the requested encoding.
    let parsed = if let Some(t) = text {
        parse_str(&t)
            .map_err(ValidatorError::from)
            .map_err(|e| thrown(&e))?
    } else if let Some(d) = data {
        let enc = resolve_encoding(encoding.as_deref()).map_err(|e| bad_encoding(&e))?;
        parse_bytes(d.as_ref(), enc)
            .map_err(ValidatorError::from)
            .map_err(|e| thrown(&e))?
    } else if let Some(p) = path {
        let enc = resolve_encoding(encoding.as_deref()).map_err(|e| bad_encoding(&e))?;
        parse_file_with_encoding(Path::new(&p), enc).map_err(|e| thrown(&e))?
    } else {
        return Err(Error::from_reason(format!(
            "bad_args{SEP}5{SEP}provide `path`, `text`, or `data`"
        )));
    };
    Ok(Reading { parsed })
}

// --- validate -----------------------------------------------------------

/// One rule violation (omitting `severity` ⇒ error, matching the engine).
#[napi(object)]
pub struct Finding {
    pub rule: String,
    pub line: Option<u32>,
    pub group: String,
    pub desc: String,
    pub severity: Option<String>,
}

/// The validation report — the Node mirror of laterite-py's `run_check` dict.
/// `ok` is **false only for un-validatable input** (the TS `raiseFor` raises
/// then); rule *violations* come back in `findings` with `ok:true`. `Report`'s
/// `isValid` is the separate `count == 0`. `json`/`ndjson` are byte-identical
/// to `lat validate --json` / `--ndjson`.
#[napi(object)]
pub struct ValidationReport {
    pub ok: bool,
    /// Set (with `error`) only when `ok` is false — the failure kind the TS
    /// `raiseFor` maps to an exception (`not_ags4`, `unsupported_edition`, …).
    pub error_kind: Option<String>,
    pub error: Option<String>,
    /// Mirrors the `lat` binary: 0 valid / 1 findings on success;
    /// 3 not-found/io, 4 not-utf8/not-ags4/bad-edition, 5 bad-dict on failure.
    pub exit_code: i32,
    pub file: String,
    pub dict_version: String,
    pub resolution: String,
    pub count: u32,
    /// Did an `index` certificate stand in for the rule engine? Never "the file was not
    /// checked": a world check (Rule 20's on-disk half) runs even on a certified read.
    pub certified: bool,
    /// If a certificate was offered and NOT used, the stable token for why (the core
    /// `RevalidateReason::as_str`, e.g. `"dictionary_changed"`). `None` when no cert was
    /// offered, or it was vouched.
    pub revalidate_reason: Option<String>,
    pub findings: Vec<Finding>,
    pub json: String,
    pub ndjson: String,
}

impl ValidationReport {
    /// The `{ok:false}` failure report (success fields defaulted) — the data
    /// analog of laterite-py's `err_dict`.
    fn failure(kind: &str, exit_code: i32, message: String) -> Self {
        ValidationReport {
            ok: false,
            error_kind: Some(kind.to_string()),
            error: Some(message),
            exit_code,
            file: String::new(),
            dict_version: String::new(),
            resolution: String::new(),
            count: 0,
            certified: false,
            revalidate_reason: None,
            findings: Vec::new(),
            json: String::new(),
            ndjson: String::new(),
        }
    }
}

// `findings_json` / `findings_ndjson` are the ENGINE's renderers
// (`laterite_ags4_validator::findings`), re-exported here at the old private
// path so the call sites below are unchanged. This module used to carry a copy
// "ported verbatim from laterite-py's findings_ndjson" — the third of three
// hand-copies of one format (#530). `lat validate --json`, laterite-py's report
// JSON and this binding now all come out of the same function.
use laterite_ags4_validator::findings::{findings_json, findings_ndjson};

/// Parse the `--dict` custom-dictionary override for a node call, mirroring the CLI's
/// `apply_dict_args` and laterite-py's helper (#568). The dict arrives as a filesystem
/// path OR raw bytes; the base edition is detected structurally from the dict itself
/// unless `over` forces it (`dictVersion`) or `dict_replace` drops it. `enc` is the
/// caller's already-resolved source encoding — the same one it hands `CheckOptions`.
///
/// Returns `Ok(None)` when no dict was named; errors use the `(exit_code, kind, message)`
/// shape the surface's failure reports take.
fn build_custom_dict(
    dict_path: Option<&str>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
) -> std::result::Result<Option<overlay::CustomDict>, (i32, &'static str, String)> {
    // Where the bytes come from, and the advisory name the cert records (basename for a
    // path, a neutral label for in-memory bytes — never a filesystem path, #568 §4).
    let (bytes, name): (Vec<u8>, String) = if let Some(p) = dict_path {
        let b = std::fs::read(Path::new(p))
            .map_err(|e| (5, "bad_dict", format!("cannot read dict {p}: {e}")))?;
        let name = Path::new(p)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("custom-dict")
            .to_string();
        (b, name)
    } else if let Some(b) = dict_bytes {
        (b.to_vec(), "custom-dict".to_string())
    } else {
        return Ok(None);
    };
    // A forced base and "no base" cannot both hold — same contradiction the CLI exits 5 on.
    if dict_replace && over.is_some() {
        return Err((
            5,
            "bad_dict",
            "dictReplace cannot be combined with dictVersion \
             (a forced base contradicts a full replacement)"
                .to_string(),
        ));
    }
    let base = if dict_replace {
        overlay::BaseSpec::Replace
    } else if let Some(v) = over {
        overlay::BaseSpec::Force(v)
    } else {
        overlay::BaseSpec::Auto
    };
    overlay::parse_dict(&bytes, overlay::DictFormat::Auto, enc, base, &name)
        .map(Some)
        .map_err(|e| (5, "bad_dict", format!("bad dict {name}: {e}")))
}

/// Run the validator from a `path`, in-memory `text`, or raw `data` — mirrors
/// laterite-py's `validate` helper. Every modality reaches the same door: a path
/// via `check_file_with_dict` (which can also answer `--check-files`), text/bytes
/// via `check_parsed_with_dict` (which cannot, and says so). The edition is
/// resolved INSIDE the door, so all three modalities agree on which dictionary
/// judged the file. Returns `(file, dict_version, resolution, findings)` or the
/// `(exit_code, error_kind, message)` of a hard failure.
#[allow(clippy::type_complexity)]
fn validate_inner(
    path: Option<&str>,
    text: Option<&str>,
    data: Option<&[u8]>,
    opts: &CheckOptions,
    cert: Option<&laterite_ags4_core::index::Sidecar>,
) -> std::result::Result<
    (String, String, String, Findings, bool, Option<String>),
    (i32, &'static str, String),
> {
    let map = |e: ValidatorError| {
        let (code, kind) = classify(&e);
        (code, kind, e.to_string())
    };

    // Bytes, whatever door they came through — a certificate is a statement about bytes.
    // A path can answer `checkFiles` (there is a directory to look beside); text and
    // bytes cannot, and the door REFUSES them rather than reporting Rule 20 clean.
    let (label, bytes, world) = if let Some(p) = path {
        let b = std::fs::read(Path::new(p)).map_err(|e| {
            let kind = if e.kind() == std::io::ErrorKind::NotFound {
                "not_found"
            } else {
                "io"
            };
            (3, kind, format!("{p}: {e}"))
        })?;
        (p.to_string(), b, WorldScope::OnDisk(PathBuf::from(p)))
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
            "bad_args",
            "provide `path`, `text` or `data`".to_string(),
        ));
    };

    let out = laterite_ags4_trust::check(laterite_ags4_trust::Request {
        bytes: &bytes,
        opts,
        cert,
        world,
        compat: None,
    })
    .map_err(map)?;

    Ok((
        label,
        out.dict_version.as_str().to_string(),
        out.resolution.as_str().to_string(),
        out.findings,
        out.certified,
        out.revalidate_reason.map(|r| r.as_str().to_string()),
    ))
}

/// Validate an AGS4 file (`path`) or `text` against the AGS4 rules. `dict_version`
/// `None`/`"auto"` auto-detects from `TRAN_AGS`, else forces an edition. Returns
/// the `{ok:false}` failure report (not a throw) for un-validatable input.
///
/// Severity tiers track importance (like a compiler): errors **and WARNINGs** are
/// returned by default (`includeWarnings` defaults to `true`); pass `false` for
/// errors-only. `includeFyi` (default `false`) adds the low-signal FYI tier.
#[napi]
#[allow(clippy::too_many_arguments)] // the napi surface mirrors lat's flags
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn run_check(
    path: Option<String>,
    text: Option<String>,
    data: Option<Uint8Array>,
    dict_version: Option<String>,
    include_warnings: Option<bool>,
    include_fyi: Option<bool>,
    check_files: Option<bool>,
    encoding: Option<String>,
    // The custom `--dict` overlay (#568): a path OR raw bytes, plus `dictReplace` to drop
    // the base. Same currency every surface shares.
    dict_path: Option<String>,
    dict_bytes: Option<Uint8Array>,
    dict_replace: Option<bool>,
    // The certificate, if the caller named one. Whether to TRUST it is not decided here,
    // nor in TypeScript — it is decided once, in `laterite_ags4_trust`, for every surface.
    cert: Option<&Sidecar>,
) -> Result<ValidationReport> {
    let forced = match resolve_edition(dict_version.as_deref()) {
        Ok(v) => v,
        Err(msg) => return Ok(ValidationReport::failure("bad_dict", 5, msg)),
    };
    // A bad ENCODING is reported the same way a bad EDITION is — as a failure the
    // caller sees, not a silent fallback to UTF-8 (which used to hand back findings
    // that were artefacts of the wrong decoder, blaming the file for a typo).
    let enc = match resolve_encoding(encoding.as_deref()) {
        Ok(e) => e,
        Err(msg) => return Ok(ValidationReport::failure("bad_args", 5, msg)),
    };
    let custom_dict = match build_custom_dict(
        dict_path.as_deref(),
        dict_bytes.as_deref(),
        dict_replace.unwrap_or(false),
        forced,
        enc,
    ) {
        Ok(cd) => cd,
        Err((code, kind, msg)) => return Ok(ValidationReport::failure(kind, code, msg)),
    };
    let opts = CheckOptions {
        dict_version: forced,
        include_warnings: include_warnings.unwrap_or(true),
        include_fyi: include_fyi.unwrap_or(false),
        check_files: check_files.unwrap_or(false),
        encoding: enc,
        custom_dict,
    };
    let (file, dv, res, found, certified, revalidate_reason) = match validate_inner(
        path.as_deref(),
        text.as_deref(),
        data.as_deref(),
        &opts,
        cert.map(|c| &c.inner),
    ) {
        Ok(t) => t,
        Err((code, kind, msg)) => return Ok(ValidationReport::failure(kind, code, msg)),
    };
    let findings: Vec<Finding> = found
        .iter()
        .flat_map(|(rule, items)| {
            items.iter().map(move |f| Finding {
                rule: rule.clone(),
                line: f.line,
                group: f.group.clone(),
                desc: f.desc.clone(),
                severity: match f.severity {
                    Severity::Error => None,
                    s => Some(s.as_str().to_string()),
                },
            })
        })
        .collect();
    // Bounded by the number of validator findings in a file, which can't
    // exceed the file's cell count — far below u32::MAX for any real file.
    #[allow(clippy::cast_possible_truncation)]
    let count = findings.len() as u32;
    Ok(ValidationReport {
        ok: true,
        certified,
        revalidate_reason,
        error_kind: None,
        error: None,
        exit_code: i32::from(count != 0),
        json: findings_json(&file, &found),
        ndjson: findings_ndjson(&found),
        file,
        dict_version: dv,
        resolution: res,
        count,
        findings,
    })
}

// --- fix (mechanical repair) + rule catalogue ---------------------------

/// The AGS4 rule catalogue as the gated `rules_meta.json` — byte-identical to
/// laterite-py's `list_rules()` and `lat rules --json`. The TS
/// layer parses it into typed `RuleMeta[]`. No input file.
#[napi]
#[must_use]
pub fn list_rules() -> String {
    rule_metadata_json().to_string()
}

/// The bundled standard dictionary for `edition` as JSON — the
/// `{ags_edition, groups:[{code, contents, parent, headings:[…]}]}` shape the
/// browser and `laterite.registry.dictionary()` also render, from the ONE shared
/// `dict::dictionary_dto` builder (#294 F#6). `None`/`"auto"` → the fallback
/// edition; else 4.0.3|4.0.4|4.1|4.1.1|4.2. The TS `registry.dictionary()` parses
/// it. (The generated `GROUPS` stays the default union registry.)
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn registry_dictionary_json(edition: Option<String>) -> Result<String> {
    let version = resolve_edition(edition.as_deref())
        .map_err(Error::from_reason)?
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dto = laterite_ags4_validator::dict::dictionary_dto(version);
    serde_json::to_string(&dto).map_err(|e| Error::from_reason(e.to_string()))
}

/// Compare two AGS4 documents (raw `a` baseline / `b` revision bytes) — the
/// revision diff, mirroring laterite-py's `diff()` and the wasm `diff()`.
/// `dict_version` `None`/`"auto"` resolves the KEY-heading edition from the
/// revision (`b`)'s `TRAN_AGS`, else forces it; `encoding` decodes both sides
/// (default UTF-8). Returns the serialised `RevisionDelta` JSON — the shared
/// `laterite-ags4-diff` leaf's `{groups, groups_added, groups_removed,
/// total_added, total_removed, total_changed}` shape — that the TS `diff()`
/// parses. Parse failure throws the mapped error (`NotAgs4Error`, …).
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn diff(
    a: Uint8Array,
    b: Uint8Array,
    dict_version: Option<String>,
    encoding: Option<String>,
) -> Result<String> {
    // A bad edition throws in the `kind␟code␟message` shape the TS `fromNativeError`
    // maps to BadDictError (parse errors below reuse `thrown` for the same reason).
    let forced = resolve_edition(dict_version.as_deref())
        .map_err(|m| Error::from_reason(format!("bad_dict{SEP}5{SEP}{m}")))?;
    let enc = resolve_encoding(encoding.as_deref()).map_err(|e| bad_encoding(&e))?;
    let pa = parse_bytes(a.as_ref(), enc)
        .map_err(ValidatorError::from)
        .map_err(|e| thrown(&e))?;
    let pb = parse_bytes(b.as_ref(), enc)
        .map_err(ValidatorError::from)
        .map_err(|e| thrown(&e))?;
    // KEY headings come from the dictionary; pick the edition from the revision
    // (b)'s TRAN_AGS (unless dict_version forces it), falling back to the standard.
    let tran = laterite_ags4_validator::tran_ags_of(&pb);
    let dv = laterite_ags4_validator::resolve_dict_version(forced, tran.as_deref())
        .map_or(laterite_ags4_validator::dict::FALLBACK, |(dv, _)| dv);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);
    serde_json::to_string(&delta).map_err(|e| Error::from_reason(e.to_string()))
}

/// The merge result. `bytes` is the reconciled AGS4 document; `warningsJson` and
/// `revisionsJson` are the advisory-notes and per-row-revision audits (arrays of
/// `{kind,group,heading,message}` / `{group,key,changed,winnerFile}`) that the TS
/// `merge()` parses — the same shape PyO3's `merge()` returns.
#[napi(object)]
pub struct MergeOutput {
    pub bytes: Buffer,
    pub warnings_json: String,
    pub revisions_json: String,
}

/// Reconcile N AGS4 deliveries of one project into one file (raw `files` bytes,
/// ≥2) — the Node port of laterite-py's `merge()`, over the SAME shared
/// `laterite-ags4-merge` leaf the CLI uses. Files merge in argument order (a
/// later file wins a KEY conflict); rows are identified by their dictionary KEY
/// headings. A heading two files typed differently throws `MergeConflictError`
/// unless `onTypeClash` settles it — `"widen"` falls back to `X` (raw values kept),
/// `"promote"` keeps the greatest nDP precision (zero-padding the coarser values).
/// `tranIssue` + `tranDate` (both) stamp a
/// synthesised merge-TRAN. The edition is the newest file's `TRAN_AGS` unless
/// `dictVersion` forces it. Parse failure throws the mapped error.
#[napi]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn merge(
    files: Vec<Uint8Array>,
    on_type_clash: Option<String>,
    dict_version: Option<String>,
    encoding: Option<String>,
    tran_issue: Option<String>,
    tran_date: Option<String>,
    tran_producer: Option<String>,
    tran_recipient: Option<String>,
    tran_status: Option<String>,
) -> Result<MergeOutput> {
    use laterite_ags4_merge::{MergeError, MergeOpts, TranStamp, TypeClashMode, merge_parsed};

    if files.len() < 2 {
        return Err(Error::from_reason(format!(
            "bad_args{SEP}5{SEP}merge needs at least two files"
        )));
    }
    let forced = resolve_edition(dict_version.as_deref())
        .map_err(|m| Error::from_reason(format!("bad_dict{SEP}5{SEP}{m}")))?;
    let enc = resolve_encoding(encoding.as_deref()).map_err(|e| bad_encoding(&e))?;
    let parsed: Vec<_> = files
        .iter()
        .map(|b| {
            parse_bytes(b.as_ref(), enc)
                .map_err(ValidatorError::from)
                .map_err(|e| thrown(&e))
        })
        .collect::<Result<_>>()?;

    // Edition from the newest (last) file's TRAN_AGS, forced by dictVersion.
    let dv = laterite_ags4_validator::resolve_dict_version(
        forced,
        parsed
            .last()
            .and_then(laterite_ags4_validator::tran_ags_of)
            .as_deref(),
    )
    .map_or(laterite_ags4_validator::dict::FALLBACK, |(dv, _)| dv);

    // A merge-TRAN is synthesised only when both an issue and a date are given.
    let tran = match (tran_issue, tran_date) {
        (Some(isno), Some(date)) => Some(TranStamp {
            isno,
            date,
            prod: tran_producer.unwrap_or_default(),
            recv: tran_recipient.unwrap_or_default(),
            stat: tran_status.unwrap_or_default(),
            ags: dv.as_str().to_string(),
        }),
        _ => None,
    };

    // One vocabulary for every surface: the accepted tokens and the rejection
    // message come from the merge crate's FromStr, so Node cannot drift from the CLI.
    let clash: TypeClashMode = on_type_clash
        .as_deref()
        .unwrap_or("error")
        .parse()
        .map_err(|m: String| napi::Error::new(napi::Status::InvalidArg, m))?;

    let opts = MergeOpts {
        on_type_clash: clash,
        edition: dv,
        tran,
        ..Default::default()
    };

    match merge_parsed(&parsed, &opts) {
        Ok(res) => {
            let warnings: Vec<Value> = res
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
            let revisions: Vec<Value> = res
                .revisions
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "group": r.group,
                        "key": r.key,
                        "changed": r.changed,
                        "winnerFile": r.winner_file,
                    })
                })
                .collect();
            Ok(MergeOutput {
                bytes: res.bytes.into(),
                warnings_json: serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".into()),
                revisions_json: serde_json::to_string(&revisions).unwrap_or_else(|_| "[]".into()),
            })
        }
        // A strict TYPE conflict / emit failure is a schema-level rejection (exit 6);
        // throw in the SEP form the TS `fromNativeError` maps to MergeConflictError.
        // UnitConflict rides the same channel: it is a schema-level rejection too,
        // and the message carries the distinction (no mode absorbs a unit clash —
        // #501). Grouped rather than split because the TS side has one
        // MergeConflictError, and its `.message` is what a caller reads.
        Err(
            e @ (MergeError::TypeConflict { .. }
            | MergeError::UnitConflict { .. }
            | MergeError::Emit(_)),
        ) => Err(Error::from_reason(format!("merge_conflict{SEP}6{SEP}{e}"))),
    }
}

/// Raw group cells for the Node CLI `lat read` — a JSON string
/// `{"order":[...],"groups":{code:{"headings":[...],"rows":[[cell,...]]}}}`,
/// straight from core's read codec (no typing), so `lat read --json` / `--csv`
/// match the Rust binary and Python byte-for-byte (#430).
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn read_groups_raw(path: String, recover_duplicate_headings: Option<bool>) -> Result<String> {
    let parsed = laterite_ags4_core::ags4_codec::read_ags4_with(
        Path::new(&path),
        read_opts_from(recover_duplicate_headings),
    )
    .map_err(|e| Error::from_reason(e.to_string()))?;
    let mut groups = Map::new();
    for code in &parsed.order {
        if let Some(g) = parsed.get(code) {
            let rows: Vec<Value> = g
                .rows
                .iter()
                .map(|row| {
                    Value::Array(
                        g.headings
                            .iter()
                            .map(|h| Value::from(row.get(h.as_str()).cloned().unwrap_or_default()))
                            .collect(),
                    )
                })
                .collect();
            let mut gd = Map::new();
            gd.insert("headings".to_string(), Value::from(g.headings.clone()));
            gd.insert("rows".to_string(), Value::Array(rows));
            groups.insert(code.clone(), Value::Object(gd));
        }
    }
    let out = serde_json::json!({ "order": parsed.order, "groups": Value::Object(groups) });
    serde_json::to_string(&out).map_err(|e| Error::from_reason(e.to_string()))
}

/// `lat read --json` for one group: the rendered string, from core's ONE JSON
/// writer. `ts/cli.ts` used to build this with `JSON.stringify(x, null, 2)`
/// while the binary used `serde_json` and Python used `json.dumps` — three
/// different JSON libraries kept byte-identical by hand-discipline, with no gate
/// on `read` output (#530).
#[napi]
#[must_use]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn render_read_json(headings: Vec<String>, rows: Vec<Vec<String>>) -> String {
    laterite_ags4_core::read_render::render_rows_json(&headings, &rows)
}

/// `lat read --csv` for one group: the rendered string, from core's ONE CSV
/// writer (RFC-4180-ish quoting). Replaces `ts/cli.ts`'s hand-ported `csvRow`.
#[napi]
#[must_use]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn render_read_csv(headings: Vec<String>, rows: Vec<Vec<String>>) -> String {
    laterite_ags4_core::read_render::render_rows_csv(&headings, &rows)
}

// --- the .ags.idx certificate (#294 Batch E / #14) ----------------------

/// A `TierCoverage` as JS sees it: the count, or `null` if the tier was never run.
fn tier_count(c: laterite_ags4_core::index::TierCoverage) -> Option<u32> {
    match c {
        laterite_ags4_core::index::TierCoverage::NotMeasured => None,
        laterite_ags4_core::index::TierCoverage::Measured { count } => Some(count),
    }
}

/// The `.ags.idx` validity certificate — the Node mirror of laterite-py's
/// `Sidecar` pyclass, wrapping the ONE core `laterite_ags4_core::index::Sidecar`
/// so a Node-minted cert is byte-identical + checker-compatible with Python and
/// `lat certify`. `Ags4File.certify()` mints one; `read(file, {index})` consumes
/// it; a fresh + engine-matching cert lets a later `.validate()` skip the rule
/// engine.
///
/// The DuckDB extension also reads these, but "checker-compatible" is the wrong
/// word for it: it never runs the checker. It consumes only the byte-offset index
/// for a sliced read, gating on size — so it shares the FORMAT with the doors
/// above, not the trust decision.
#[napi]
pub struct Sidecar {
    inner: laterite_ags4_core::index::Sidecar,
}

#[napi]
impl Sidecar {
    /// **Mint** a certificate for `data` — validating it here, first.
    ///
    /// This replaces `assemble`, whose signature was
    /// `(data, edition, checkedAt, warnings?, fyi?, …)`: the caller told it what the
    /// verdict had been, and the OPTIONAL counts defaulted to zero. Nothing on the TS
    /// side ever passed them. So every certificate this addon produced recorded "0
    /// warnings, 0 FYI" without anything having looked, and a later warnings request read
    /// that zero and skipped the engine.
    ///
    /// There is no parameter here through which a caller could assert a verdict. `mint`
    /// runs the rules itself, with both tiers on, and records what they returned. It
    /// refuses a file with ERRORS; warnings and FYI are recorded, not fatal.
    #[napi(factory)]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
    pub fn mint(
        data: Uint8Array,
        checked_at: String,
        dict_version: Option<String>,
        encoding: Option<String>,
        compat: Option<String>,
        dict_path: Option<String>,
        dict_bytes: Option<Uint8Array>,
        dict_replace: Option<bool>,
    ) -> Result<Sidecar> {
        let forced = resolve_edition(dict_version.as_deref()).map_err(Error::from_reason)?;
        let enc = laterite_ags4_parse::resolve_encoding(encoding.as_deref()).ok_or_else(|| {
            Error::from_reason(format!(
                "unknown encoding {:?}",
                encoding.unwrap_or_default()
            ))
        })?;
        // The cert records WHICH custom dictionary judged the file (#568, O-48): a mint
        // against a `--dict` overlay stamps its {name, hash}, and a later read naming a
        // different dict revalidates rather than inheriting a stale verdict.
        let custom_dict = build_custom_dict(
            dict_path.as_deref(),
            dict_bytes.as_deref(),
            dict_replace.unwrap_or(false),
            forced,
            enc,
        )
        .map_err(|(_, _, msg)| Error::from_reason(msg))?;
        let opts = CheckOptions {
            dict_version: forced,
            encoding: enc,
            custom_dict,
            ..CheckOptions::default()
        };
        let inner = laterite_ags4_trust::mint(data.as_ref(), &opts, checked_at, compat)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Sidecar { inner })
    }

    /// Parse a certificate from its on-disk `.ags.idx` JSON bytes, rejecting an
    /// unknown format version. Throws on malformed / unsupported JSON.
    #[napi(factory)]
    #[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
    pub fn from_json(data: Uint8Array) -> Result<Sidecar> {
        let inner = laterite_ags4_core::index::Sidecar::from_json(data.as_ref())
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Sidecar { inner })
    }

    /// Serialise to the on-disk `.ags.idx` JSON (pretty).
    #[napi]
    pub fn to_json(&self) -> Result<Buffer> {
        self.inner
            .to_json()
            .map(Buffer::from)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Is this certificate still current for `data`? Strong check: format version +
    /// byte length + SHA-256. A mismatch means the source changed under the cert.
    #[napi]
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
    pub fn is_fresh_for(&self, data: Uint8Array) -> bool {
        self.inner.is_fresh_for(data.as_ref())
    }

    #[napi(getter)]
    #[must_use]
    pub fn version(&self) -> u32 {
        self.inner.version
    }
    /// The certified source's byte length (a JS number; AGS files are well within
    /// the 2^53 safe-integer range).
    #[napi(getter)]
    #[must_use]
    pub fn size(&self) -> f64 {
        self.inner.file.size as f64
    }
    #[napi(getter)]
    #[must_use]
    pub fn sha256(&self) -> String {
        self.inner.file.sha256.clone()
    }
    /// The AGS edition the rules were run against.
    #[napi(getter)]
    #[must_use]
    pub fn edition(&self) -> String {
        self.inner.validation.edition.edition().to_string()
    }
    /// Was that edition FORCED (`dictVersion`), or auto-resolved from `TRAN_AGS`? One
    /// fact with the edition string, not two — a forced run and an auto run can name the
    /// same edition having applied different dictionaries.
    #[napi(getter)]
    #[must_use]
    pub fn edition_forced(&self) -> bool {
        self.inner.validation.edition.is_forced()
    }
    #[napi(getter)]
    #[must_use]
    pub fn validator(&self) -> String {
        self.inner.validation.validator.clone()
    }
    /// The fingerprint of the rule engine that produced this verdict — a hash of the rule
    /// sources and the bundled dictionary, NOT the addon's version. A rule can change
    /// without a version bump; this cannot.
    #[napi(getter)]
    #[must_use]
    pub fn engine(&self) -> String {
        self.inner.validation.engine.clone()
    }
    #[napi(getter)]
    #[must_use]
    pub fn compat(&self) -> Option<String> {
        self.inner.validation.compat.clone()
    }
    #[napi(getter)]
    #[must_use]
    pub fn checked_at(&self) -> String {
        self.inner.validation.checked_at.clone()
    }
    /// The decoder the certified bytes were READ through (`"UTF-8"`, `"windows-1252"`, …).
    /// The rules judge the TEXT the bytes decode to, and two decoders can reach two
    /// verdicts on one unchanged file — so a cert minted under one does not answer a
    /// request made under another.
    #[napi(getter)]
    #[must_use]
    pub fn encoding(&self) -> String {
        self.inner.validation.encoding.clone()
    }

    /// Findings of each tier that the validation **measured** — or `null` if it never ran
    /// that tier's rules. `null` is the point: the old format stored a plain number that
    /// defaulted to 0, so "found none" and "never looked" were the same value.
    #[napi(getter)]
    #[must_use]
    pub fn errors(&self) -> Option<u32> {
        tier_count(self.inner.validation.errors)
    }
    #[napi(getter)]
    #[must_use]
    pub fn warnings(&self) -> Option<u32> {
        tier_count(self.inner.validation.warnings)
    }
    #[napi(getter)]
    #[must_use]
    pub fn fyi(&self) -> Option<u32> {
        tier_count(self.inner.validation.fyi)
    }
}

/// One applied fix — the Node mirror of laterite-py's `applied[]` entries.
/// `kind`/`risk` are the serde `snake_case` strings (`strip_bom`, `safe`, …) so
/// the shape is identical across Python / CLI / Node.
#[napi(object)]
pub struct AppliedFix {
    pub kind: String,
    pub label: String,
    pub rule: String,
    pub line: Option<u32>,
    pub risk: String,
}

/// Map the engine's `Fix` records to the napi `AppliedFix` shape — shared by
/// `fix()`'s `FixReport.applied` and `buildAgs4`'s `EmitResult.applied` so both
/// present an identical ledger (#294 F#7). `kind`/`risk` are serde-serialised so
/// they match Python / CLI byte-for-byte.
fn to_applied_fixes(fixes: &[Fix]) -> Vec<AppliedFix> {
    let s = |v: serde_json::Value| v.as_str().map(String::from).unwrap_or_default();
    fixes
        .iter()
        .map(|f| AppliedFix {
            kind: serde_json::to_value(f.kind).map(s).unwrap_or_default(),
            label: f.label.clone(),
            rule: f.rule.clone(),
            line: f.line,
            risk: serde_json::to_value(f.risk).map(s).unwrap_or_default(),
        })
        .collect()
}

/// The repair report — the Node mirror of laterite-py's `fix_file` dict. `ok` is
/// false only for un-fixable input (the TS layer raises then). `fixed` is the
/// repaired bytes (the original verbatim when nothing applied); `residual` is
/// what could *not* be mechanically fixed.
#[napi(object)]
pub struct FixReport {
    pub ok: bool,
    pub error_kind: Option<String>,
    pub error: Option<String>,
    pub exit_code: i32,
    pub fixed: Buffer,
    pub dict_version: String,
    pub resolution: String,
    pub fixes_applied: u32,
    pub applied: Vec<AppliedFix>,
    pub residual: Vec<Finding>,
}

impl FixReport {
    fn failure(kind: &str, exit_code: i32, message: String) -> Self {
        FixReport {
            ok: false,
            error_kind: Some(kind.to_string()),
            error: Some(message),
            exit_code,
            fixed: Buffer::from(Vec::<u8>::new()),
            dict_version: String::new(),
            resolution: String::new(),
            fixes_applied: 0,
            applied: Vec::new(),
            residual: Vec::new(),
        }
    }
}

/// Mechanically repair an AGS4 file (`path`) / `text` / `data`: apply the SAFE
/// fixes (plus the risky set when `includeRisky`, narrowed by `only` / widened-
/// back by `exclude`), re-validate, and return the fixed bytes + residual
/// findings. Mirrors laterite-py's `fix()` / `lat fix`; the single
/// `fix_document_selective` orchestration is shared. The TS layer wraps this into
/// a `FixResult` (`.bytes` / `.text` / `.save(path)`) and adds `inPlace` / `out`
/// write-back on top.
#[napi]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn fix_file(
    path: Option<String>,
    text: Option<String>,
    data: Option<Uint8Array>,
    dict_version: Option<String>,
    encoding: Option<String>,
    include_risky: Option<bool>,
    // Per-rule selection (#394): `only` (when set) keeps just those fixable-rule
    // labels, then `exclude` drops any of them — the same short labels
    // (`"8"`, `"2a"`) laterite-py's `fix(only=, exclude=)` takes. The shared
    // `fix_document_selective` applies the risk gate first, so a rule whose only
    // fix is risky still needs `include_risky` even when named in `only`.
    only: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    dict_path: Option<String>,
    dict_bytes: Option<Uint8Array>,
    dict_replace: Option<bool>,
) -> Result<FixReport> {
    let raw: Vec<u8> = if let Some(t) = text {
        t.into_bytes()
    } else if let Some(d) = data {
        d.to_vec()
    } else if let Some(p) = path {
        match std::fs::read(&p) {
            Ok(b) => b,
            Err(e) => return Ok(FixReport::failure("io", 3, format!("{p}: {e}"))),
        }
    } else {
        return Ok(FixReport::failure(
            "bad_args",
            5,
            "provide `path`, `text`, or `data`".to_string(),
        ));
    };
    let forced = match resolve_edition(dict_version.as_deref()) {
        Ok(v) => v,
        Err(msg) => return Ok(FixReport::failure("bad_dict", 5, msg)),
    };
    let enc = match resolve_encoding(encoding.as_deref()) {
        Ok(e) => e,
        Err(msg) => return Ok(FixReport::failure("bad_args", 5, msg)),
    };
    let custom_dict = match build_custom_dict(
        dict_path.as_deref(),
        dict_bytes.as_deref(),
        dict_replace.unwrap_or(false),
        forced,
        enc,
    ) {
        Ok(cd) => cd,
        Err((code, kind, msg)) => return Ok(FixReport::failure(kind, code, msg)),
    };
    let opts = CheckOptions {
        dict_version: forced,
        encoding: enc,
        custom_dict,
        // The residual re-validation tier matches this surface's `validate()`
        // default (errors + warnings) — not `CheckOptions::default()`'s
        // errors-only, which under-reported what the fix left behind (#294 C).
        include_warnings: true,
        include_fyi: false,
        ..CheckOptions::default()
    };
    let exclude = exclude.unwrap_or_default();
    let outcome = match fix_document_selective(
        &raw,
        &opts,
        include_risky.unwrap_or(false),
        only.as_deref(),
        &exclude,
    ) {
        Ok(o) => o,
        Err(e) => {
            let (code, kind) = classify(&e);
            return Ok(FixReport::failure(kind, code, e.to_string()));
        }
    };
    let applied = to_applied_fixes(&outcome.applied);
    let residual: Vec<Finding> = outcome
        .residual
        .iter()
        .flat_map(|(rule, items)| {
            items.iter().map(move |f| Finding {
                rule: rule.clone(),
                line: f.line,
                group: f.group.clone(),
                desc: f.desc.clone(),
                severity: match f.severity {
                    Severity::Error => None,
                    s => Some(s.as_str().to_string()),
                },
            })
        })
        .collect();
    // Bounded by the number of fixes applied, which can't exceed the
    // file's finding count — far below u32::MAX for any real file.
    #[allow(clippy::cast_possible_truncation)]
    let fixes_applied = applied.len() as u32;
    Ok(FixReport {
        ok: true,
        error_kind: None,
        error: None,
        exit_code: i32::from(!residual.is_empty()),
        fixed: Buffer::from(outcome.fixed),
        dict_version: outcome.dict_version.as_str().to_string(),
        resolution: outcome.resolution.as_str().to_string(),
        fixes_applied,
        applied,
        residual,
    })
}

// --- emit (data → AGS4) -------------------------------------------------

/// One group of columnar input — its code + an Arrow IPC stream (`Buffer`)
/// whose column names are the AGS headings.
#[napi(object)]
pub struct GroupIpc {
    pub code: String,
    pub ipc: Buffer,
}

/// The emit result. `bytes` is the AGS4 document; `findingsJson` is the
/// validator's `{rule:[…]}` map on the output; `applied` is the safe-fix ledger
/// `AutoFix` made (same shape as `fix()`'s `FixReport.applied`); `fixesApplied`
/// is its length.
#[napi(object)]
pub struct EmitResult {
    pub bytes: Buffer,
    pub findings_json: String,
    pub applied: Vec<AppliedFix>,
    pub fixes_applied: u32,
}

/// Build valid AGS4 from per-group **Arrow IPC** streams (the columnar
/// producer; the read boundary reversed). = `laterite-ags4-wasm`'s `to_ags4_ipc`.
// napi boundary: owns the deserialized input (needless_pass_by_value); and napi
// always deserializes a JS object into HashMap<_, _, RandomState> — no caller
// can supply a different hasher, so genericizing over `S: BuildHasher` here
// would be unreachable generality (implicit_hasher).
#[napi]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::implicit_hasher)]
pub fn emit_ags4_from_ipc(
    groups: Vec<GroupIpc>,
    edition: Option<String>,
    mode: Option<String>,
    // Per-heading UNIT/TYPE overrides, keyed `{code → {heading → value}}` (#294
    // F#9). A group/heading absent from the map keeps the dictionary default.
    units: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    types: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    // Off unless asked: no surface mints GROUPs the caller never wrote without
    // being told to (2026-07-24). See EmitOpts::synthesise_metadata.
    synthesise_metadata: Option<bool>,
) -> Result<EmitResult> {
    let opts = laterite_ags4_emit::EmitOpts {
        mode: resolve_mode(mode.as_deref())?,
        edition: resolve_edition(edition.as_deref())
            .map_err(Error::from_reason)?
            .unwrap_or(DictVersion::V4_1_1),
        synthesise_metadata: synthesise_metadata.unwrap_or(false),
    };
    let mut inputs = Vec::with_capacity(groups.len());
    for g in groups {
        let u = units.as_ref().and_then(|m| m.get(&g.code));
        let t = types.as_ref().and_then(|m| m.get(&g.code));
        inputs.push(group_from_ipc(g.code, &g.ipc, u, t)?);
    }
    let res = laterite_ags4_emit::emit_ags4(&inputs, &opts)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    let findings_json = serde_json::to_string(&res.findings).unwrap_or_else(|_| "{}".into());
    // Bounded the same way as `fixes_applied` in `fix_ags4` above.
    #[allow(clippy::cast_possible_truncation)]
    let fixes_applied = res.fixes_applied as u32;
    Ok(EmitResult {
        bytes: res.bytes.into(),
        findings_json,
        applied: to_applied_fixes(&res.applied),
        fixes_applied,
    })
}

fn group_from_ipc(
    code: String,
    bytes: &[u8],
    units: Option<&std::collections::HashMap<String, String>>,
    types: Option<&std::collections::HashMap<String, String>>,
) -> Result<laterite_ags4_emit::GroupInput> {
    let reader = StreamReader::try_new(Cursor::new(bytes), None)
        .map_err(|e| Error::from_reason(format!("arrow ipc: {e}")))?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for b in reader {
        batches.push(b.map_err(|e| Error::from_reason(format!("arrow ipc batch: {e}")))?);
    }
    Ok(laterite_ags4_emit::group_from_arrow_with_meta(
        code,
        schema.as_ref(),
        &batches,
        units,
        types,
    ))
}

// --- helpers ------------------------------------------------------------

/// Resolve an encoding label, or say why not.
///
/// This used to be infallible — `…resolve_encoding(label).unwrap_or(UTF_8)`. An
/// unknown label silently became UTF-8, which is a corruption vector, not a
/// convenience: `C3 A9` is `é` in UTF-8 and `Ã©` in cp1252, both decode cleanly, so
/// a caller who typed `cp1252x` got the wrong text with no error at all — while the
/// same typo raised on Python. Now it raises here too (`bad_args`, exit 5 — the same
/// kind and code Python uses), so one label means one thing on every surface.
fn resolve_encoding(
    label: Option<&str>,
) -> std::result::Result<&'static encoding_rs::Encoding, String> {
    laterite_ags4_parse::resolve_encoding(label)
        .ok_or_else(|| format!("unknown encoding {:?}", label.unwrap_or("")))
}

/// The `kind␟code␟message` shape TS's `fromNativeError` maps to a typed error.
// Internal helper (not a napi boundary) — the message is only ever formatted,
// never owned, so each call site borrows the `String` `resolve_encoding` returns.
fn bad_encoding(msg: &str) -> Error {
    Error::from_reason(format!("bad_args{SEP}5{SEP}{msg}"))
}

/// Plain-`String` error (not napi) so `run_check` can surface a bad edition as
/// a `{ok:false}` failure report while `emit_ags4_from_ipc` throws it.
fn resolve_edition(s: Option<&str>) -> std::result::Result<Option<DictVersion>, String> {
    match s.map(str::trim) {
        None | Some("" | "auto") => Ok(None),
        Some(o) => DictVersion::from_edition(o).map(Some).ok_or_else(|| {
            format!(
                "unknown dict_version {o:?}; expected auto|{}",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

fn resolve_mode(s: Option<&str>) -> Result<laterite_ags4_emit::EmitMode> {
    match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("" | "autofix") => Ok(laterite_ags4_emit::EmitMode::AutoFix),
        Some("report") => Ok(laterite_ags4_emit::EmitMode::Report),
        Some("strict") => Ok(laterite_ags4_emit::EmitMode::Strict),
        Some(o) => Err(Error::from_reason(format!(
            "unknown mode {o:?}; expected autofix|report|strict"
        ))),
    }
}

// --- Excel ↔ AGS4 (laterite-excel) --------------------------------------
// Binds the same AGS4↔XLSX converter Python exposes as to_excel/from_excel
// (#358 — closes the node-Excel capability gap). Path-based, like the Python
// binding: Node has a filesystem, so no in-memory round-trip is needed.

/// The outcome of an Excel conversion (mirrors `laterite_excel::ExcelStats`).
#[napi(object)]
pub struct ExcelStats {
    /// Worksheets written (AGS4→XLSX) or read (XLSX→AGS4).
    pub sheets_written: u32,
    /// DATA rows written across all sheets.
    pub rows_written: u32,
    /// Non-fatal conversion warnings.
    pub warnings: Vec<String>,
}

impl From<laterite_excel::ExcelStats> for ExcelStats {
    // `sheets_written` is bounded by the AGS4 dictionary's group count (174
    // max); `rows_written` needs billions of in-memory rows to overflow —
    // physically unreachable (RAM exhausts long before this cast could
    // truncate).
    #[allow(clippy::cast_possible_truncation)]
    fn from(s: laterite_excel::ExcelStats) -> Self {
        ExcelStats {
            sheets_written: s.sheets_written as u32,
            rows_written: s.rows_written as u32,
            warnings: s.warnings,
        }
    }
}

/// Write an AGS4 file's groups to an `.xlsx` — one worksheet per group.
/// `orderedKeys` forces the worksheet order; otherwise AGS4 source order.
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn ags4_to_excel(
    ags_path: String,
    xlsx_path: String,
    ordered_keys: Option<Vec<String>>,
) -> Result<ExcelStats> {
    laterite_excel::ags4_to_excel(Path::new(&ags_path), Path::new(&xlsx_path), ordered_keys)
        .map(ExcelStats::from)
        .map_err(|e| Error::from_reason(e.to_string()))
}

/// Read an `.xlsx` back into an AGS4 file. `formatNumericColumns` (default
/// true) re-applies AGS4 numeric formatting to numeric-looking columns.
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn excel_to_ags4(
    xlsx_path: String,
    ags_path: String,
    format_numeric_columns: Option<bool>,
) -> Result<ExcelStats> {
    laterite_excel::excel_to_ags4(
        Path::new(&xlsx_path),
        Path::new(&ags_path),
        format_numeric_columns.unwrap_or(true),
    )
    .map(ExcelStats::from)
    .map_err(|e| Error::from_reason(e.to_string()))
}

/// An in-memory Excel conversion result: the produced `bytes` (an `.xlsx`
/// workbook or an `.ags` document) plus the same conversion stats. The bytes
/// twin of the path functions, so an uploaded workbook / a fixed handle needn't
/// hit disk — the same FS-free cores the browser surface uses.
#[napi(object)]
pub struct ExcelBytesResult {
    pub bytes: Buffer,
    pub sheets_written: u32,
    pub rows_written: u32,
    pub warnings: Vec<String>,
}

/// AGS4 bytes → `.xlsx` workbook bytes (one worksheet per group). `orderedKeys`
/// forces the worksheet order; otherwise AGS4 source order. The bytes twin of
/// `ags4ToExcel`.
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn ags4_bytes_to_xlsx(
    data: Uint8Array,
    ordered_keys: Option<Vec<String>>,
    recover_duplicate_headings: Option<bool>,
) -> Result<ExcelBytesResult> {
    let opts = read_opts_from(recover_duplicate_headings);
    let (xlsx, stats) = laterite_excel::ags4_bytes_to_xlsx_with(&data, ordered_keys, opts)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    // Bounded the same way as `ExcelStats::from` above (group count /
    // physical RAM limits).
    #[allow(clippy::cast_possible_truncation)]
    let (sheets_written, rows_written) = (stats.sheets_written as u32, stats.rows_written as u32);
    Ok(ExcelBytesResult {
        bytes: Buffer::from(xlsx),
        sheets_written,
        rows_written,
        warnings: stats.warnings,
    })
}

/// `.xlsx` workbook bytes → AGS4 bytes. `formatNumericColumns` (default true)
/// re-applies AGS4 numeric formatting to numeric-looking columns. The bytes twin
/// of `excelToAgs4`.
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn xlsx_bytes_to_ags4(
    data: Uint8Array,
    format_numeric_columns: Option<bool>,
) -> Result<ExcelBytesResult> {
    let (ags, stats) =
        laterite_excel::xlsx_bytes_to_ags4(&data, format_numeric_columns.unwrap_or(true))
            .map_err(|e| Error::from_reason(e.to_string()))?;
    // Bounded the same way as `ExcelStats::from` above (group count /
    // physical RAM limits).
    #[allow(clippy::cast_possible_truncation)]
    let (sheets_written, rows_written) = (stats.sheets_written as u32, stats.rows_written as u32);
    Ok(ExcelBytesResult {
        bytes: Buffer::from(ags),
        sheets_written,
        rows_written,
        warnings: stats.warnings,
    })
}

/// What THIS SURFACE resolves an encoding label to — the canonical `encoding_rs`
/// name (`"UTF-8"`, `"windows-1252"`, `"ISO-8859-15"`), or `null` if it refuses.
///
/// Deliberately routed through this crate's OWN `resolve_encoding` wrapper, not the
/// parse leaf directly. That distinction is the whole point: the leaf was always
/// correct, and the bug lived in the wrapper *above* it (`…resolve_encoding(label)
/// .unwrap_or(UTF_8)`), which turned every unknown label into a silent UTF-8 decode.
/// A census that asked the leaf would have agreed with itself and seen nothing. This
/// reports what a Node caller actually gets, so a reintroduced fallback shows up as
/// `"cp1252x" -> "UTF-8"` and the surface census fails.
#[napi]
#[must_use]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn resolve_encoding_label(label: Option<String>) -> Option<String> {
    resolve_encoding(label.as_deref())
        .ok()
        .map(|e| e.name().to_string())
}

/// The bundled AGS4 editions, oldest first — `["4.0.3", … "4.2"]`.
///
/// GENERATED all the way down: `DictVersion::ALL` is emitted by the reference leaf's
/// build.rs from `ags_dictionary.json`. Exposed so no JS-side list of editions is
/// hand-written — the CLI's `--dict-version` census reads this, and it is the same
/// const the Rust binary and the Python wheel answer with, so the three launchers
/// cannot disagree about which editions exist.
#[napi]
#[must_use]
pub fn editions() -> Vec<String> {
    laterite_ags4_validator::dict::DictVersion::ALL
        .iter()
        .map(|v| v.as_str().to_string())
        .collect()
}

/// The `--on-type-clash` modes merge accepts, in declaration order —
/// `["error", "widen", "promote"]`.
///
/// GENERATED, same as [`editions`]: the set is owned by `TypeClashMode::ALL` in
/// laterite-ags4-merge. Exposed so the JS launcher does not keep a hand-written
/// copy — it had two (the census `values` for `merge --on-type-clash`, and the
/// unknown-mode error message in `cli.ts`), and a fourth mode added to the enum
/// would have reached neither (#555).
#[napi]
#[must_use]
pub fn type_clash_modes() -> Vec<String> {
    laterite_ags4_merge::TypeClashMode::ALL
        .iter()
        .map(|m| m.as_str().to_string())
        .collect()
}

/// The edition `auto` falls back to when a file's `TRAN_AGS` is missing or
/// unrecognised (the union's `fallback_edition`, generated).
#[napi]
#[must_use]
pub fn fallback_edition() -> String {
    laterite_ags4_validator::dict::FALLBACK.as_str().to_string()
}

/// Parent chain from `code` up to the registry root — `[code, parent, …, root]`
/// (a root group returns `[code]`). Raises for an unknown code, so a root is
/// distinguishable from a miss.
///
/// The walk is `laterite_ags4_core::registry::ancestor_chain`, the ONE Rust
/// definition of the group tree — the same leaf function the Python wheel binds
/// (`registry_fns.rs::registry_ancestor_chain`). Node used to re-walk `.parent`
/// pointers in TypeScript (`ts/registry.ts`), a hand-kept-in-sync copy of that
/// logic; routing through the binding removes it so the tree can't drift from the
/// leaf (#532, #527).
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn registry_ancestor_chain(code: String) -> Result<Vec<String>> {
    let reg = laterite_ags4_core::registry::registry();
    if reg.get(&code).is_none() {
        return Err(Error::from_reason(format!("unknown group code: {code:?}")));
    }
    Ok(laterite_ags4_core::registry::ancestor_chain(reg, &code)
        .into_iter()
        .map(|g| g.code.clone())
        .collect())
}

/// The KEY heading names a group inherits from its DIRECT parent — the
/// intersection of its KEY headings with the parent's — sorted for determinism
/// (the TS facade wraps this in a Set). Empty for a root; raises for an unknown
/// code.
///
/// The intersection is `laterite_ags4_core::registry::inherited_key_names`, the
/// same leaf function the Python wheel binds; Node used to re-implement the
/// KEY-intersection logic in TypeScript. Deleting that copy is the point of #532
/// (part of the #527 leaf-convergence arc).
#[napi]
#[allow(clippy::needless_pass_by_value)] // napi boundary: owns the deserialized input
pub fn registry_inherited_key_names(code: String) -> Result<Vec<String>> {
    let reg = laterite_ags4_core::registry::registry();
    let g = reg
        .get(&code)
        .ok_or_else(|| Error::from_reason(format!("unknown group code: {code:?}")))?;
    let mut names: Vec<String> = laterite_ags4_core::registry::inherited_key_names(reg, g)
        .into_iter()
        .collect();
    names.sort();
    Ok(names)
}

/// Map the surface-level boolean onto core's read policy. Duplicate headings are
/// fatal by default on every read surface; a caller opts into recovery.
fn read_opts_from(recover: Option<bool>) -> laterite_ags4_core::ags4_codec::ReadOptions {
    use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ReadOptions};
    ReadOptions {
        duplicate_headings: if recover.unwrap_or(false) {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
    }
}
