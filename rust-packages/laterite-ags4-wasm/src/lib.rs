//! Browser wasm wrapper around the clean-room AGS4 validator.
//!
//! `validate()` replicates the body of `laterite_ags4_validator::check_file_with_dict`
//! (`lib.rs`) but from in-memory bytes with `source = None`, so it runs
//! the entire rule engine **client-side** with no filesystem and nothing
//! uploaded. Rule *violations* come back as data in the report; only
//! un-validatable inputs (not AGS4, unsupported edition, bad arguments)
//! populate `report.error` — nothing throws across the wasm boundary.
//!
//! Phase 2 adds `read()` → typed Arrow IPC for the DuckDB-wasm data
//! explorer; this file is Phase 1 (validator) only.

// #168 Phase 3: parse types + tokenizer come straight from the leaf
// (encoding_rs + memchr only — wasm-safe); the validator dep stays for rules.
use laterite_ags4_parse::{ParsedFile, parse_bytes};
use laterite_ags4_types::sql_type;
use laterite_ags4_validator::{
    CheckOptions, DictVersion, ValidatorError, WorldScope, check_parsed_with_dict,
    dict::Dictionary, dict::FALLBACK, findings, fixes, overlay, resolve_dict_version, tran_ags_of,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Declare a block of TypeScript for the generated `.d.ts`, keeping a plain
/// `const` copy the tests can read.
///
/// Both are needed, and the duplication is only apparent. wasm-bindgen's
/// `typescript_custom_section` **consumes** the item it decorates — the const is
/// gone by the time the rest of the crate compiles — and its parser matches on
/// `syn::Lit::Str`, so it will not accept a reference to a const defined
/// elsewhere. Emitting the same literal token into both positions from one
/// `$src` is what lets `ts_result_shape_tests` read the exact string that ships,
/// rather than a second copy that could drift from it.
///
/// The readable copy is `#[cfg(test)]`: nothing but the tests reads it, so
/// outside them it is dead weight in a binary whose whole point is being small.
macro_rules! ts_section {
    ($(#[$meta:meta])* $name:ident, $section:ident, $src:literal) => {
        $(#[$meta])*
        #[cfg(test)]
        const $name: &str = $src;

        #[wasm_bindgen(typescript_custom_section)]
        const $section: &'static str = $src;
    };
}

/// One rule violation — mirrors the CLI's `{line, group, desc}` JSON,
/// plus the additive rule-aware location/severity fields (omitted when
/// unset so the base shape is unchanged). `target`/`severity` use
/// snake_case to match the engine's serde rename + the TS interface.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct FindingDto {
    line: Option<u32>,
    group: String,
    desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading: Option<String>,
    /// 1-based row ordinal within the group (distinct from `line`); set
    /// for data-row findings so the UI can address the exact cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    data_row: Option<u32>,
    /// Half-open `[start, end)` char-offset span within the raw line —
    /// either carried by the finding (Rules 1/6) or computed here from
    /// `field_index` so every cell/heading finding gets a precise span.
    /// Serialized as a 2-element array to match the TS `[number, number]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    char_span: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

/// Findings for a single rule. The report flattens the engine's
/// `BTreeMap<rule, Vec<Finding>>` into an ordered array of these so the
/// UI can render one collapsible section per rule without re-sorting
/// (the engine already orders by rule label).
#[derive(Serialize)]
struct RuleGroup {
    rule: String,
    /// True number of findings for this rule, **before** any serialization
    /// cap. `items.len()` may be smaller when `max_per_rule` clips the tail;
    /// the UI shows "N of `total`" so a cap never hides the real count.
    total: usize,
    items: Vec<FindingDto>,
}

/// An un-validatable input (not a rule violation). `kind` is a stable
/// machine token the UI switches on; `message` is the human string.
#[derive(Serialize)]
struct ValErr {
    kind: String,
    message: String,
}

/// The whole result of a validation run. `ok` is the verdict
/// (`error.is_none() && finding_count == 0`); when `error` is set the
/// other fields are empty/zero and no rules ran.
#[derive(Serialize)]
struct ValidationReport {
    ok: bool,
    /// The bundled edition the file was judged against (`"4.1.1"`, …),
    /// empty on error.
    dict_version: String,
    /// How that edition was chosen: `forced` / `exact` / `guessed` /
    /// `fallback` (see `DictResolution`), empty on error.
    resolution: String,
    /// True total across every rule — always the full count the engine
    /// found, independent of any serialization cap.
    finding_count: usize,
    /// How many `FindingDto` were actually serialized into `findings`
    /// (the sum of each group's `items.len()`). Equals `finding_count`
    /// when uncapped; smaller when `max_per_rule` clipped some groups, so
    /// the UI can say "showing `shown_count` of `finding_count`".
    shown_count: usize,
    findings: Vec<RuleGroup>,
    error: Option<ValErr>,
    /// Why a proffered `.ags.idx` certificate did NOT stand in for the rule engine,
    /// as the stable snake_case token (`"dictionary_changed"`, `"content_changed"`,
    /// …), else `null`. Present for cross-surface shape parity with `Report`
    /// (laterite-py) and Node's `revalidateReason` (#568 Phase 6). **Structurally
    /// always `null` here:** this surface has no cert-consume door — `validate`
    /// re-runs the engine unconditionally (`certify` only *mints*), so no
    /// certificate is ever offered to accept or reject. The field exists so a JS
    /// consumer reads the same report shape on every surface, and is ready if a
    /// wasm cert-consume path is ever added.
    revalidate_reason: Option<String>,
}

impl ValidationReport {
    fn failure(kind: &str, message: String) -> Self {
        ValidationReport {
            ok: false,
            dict_version: String::new(),
            resolution: String::new(),
            finding_count: 0,
            shown_count: 0,
            findings: Vec::new(),
            error: Some(ValErr {
                kind: kind.to_string(),
                message,
            }),
            revalidate_reason: None,
        }
    }
}

// The TS shapes for the results above. These used to be maintained by hand in
// the *consumer* (`web/src/lib/validator.ts`), because wasm-bindgen types a
// `JsValue` return as `any` — so every consumer had to re-describe the report
// for itself, and a downstream browser consumer duly reported `build_ags4`
// returning `any` as a defect. Declaring them here makes the crate that
// serialises the shape the crate that publishes it.
//
// Nothing in wasm-bindgen binds an interface written here to the serde struct
// above it, so `ts_interfaces_match_the_serde_structs` does: it parses these
// strings and compares field names against the structs' own serialised keys.
// Without that this is prettier `any` — a second hand-maintained mirror, one
// directory closer.
ts_section! {
    TS_VALIDATE_RESULT,
    TS_VALIDATE_RESULT_SECTION,
    r#"
/** One rule violation. */
export interface FindingDto {
  /** 1-based source line, or `null` for a whole-file finding. */
  line: number | null;
  group: string;
  desc: string;
  /** Present only when the finding is narrower than a whole line. */
  target?: "line" | "heading" | "cell" | "group";
  /** Tag-stripped column index — the raw on-line field is `field_index + 1`,
   *  because field 0 is the HEADING tag. */
  field_index?: number;
  heading?: string;
  /** 1-based row ordinal *within the group*, distinct from `line`. */
  data_row?: number;
  /** Half-open `[start, end)` char offsets within the raw line. */
  char_span?: [number, number];
  /** **Absent means `"error"`.** The engine omits the field for errors rather
   *  than writing it, so `severity === undefined` is the most severe case, not
   *  a missing annotation. Read it as `severity ?? "error"`. */
  severity?: "warning" | "fyi";
}

/** Findings for a single rule. */
export interface RuleGroup {
  rule: string;
  /** True per-rule count, **before** any `maxPerRule` cap; `items.length` may
   *  be smaller, which is why both exist. */
  total: number;
  items: FindingDto[];
}

/** An un-validatable input — not a rule violation. */
export interface ValErr {
  /** Stable machine token: `not_ags4` | `unsupported_edition` | `bad_args` | … */
  kind: string;
  message: string;
}

/** The whole result of a validation run. */
export interface ValidationReport {
  /** `error === null && finding_count === 0`. */
  ok: boolean;
  /** The bundled edition judged against (`"4.1.1"`, …); `""` on error. */
  dict_version: string;
  /** How that edition was chosen: `forced` | `exact` | `guessed` | `fallback`;
   *  `""` on error. */
  resolution: string;
  /** True total across every rule, independent of any cap. */
  finding_count: number;
  /** How many findings were actually serialised (≤ `finding_count` when capped). */
  shown_count: number;
  findings: RuleGroup[];
  error: ValErr | null;
  /** Why a proffered `.ags.idx` cert did not stand in for the engine. **Always
   *  `null` on this surface** — `validate` has no cert-consume door, so none is
   *  ever offered to reject. Present so the report shape matches Python's
   *  `Report.revalidate_reason` and Node's `revalidateReason`. */
  revalidate_reason: string | null;
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ValidationReport")]
    pub type ValidationReportJs;
}

/// Map a `ValidatorError` to a `(kind, message)`. In the wasm path only
/// `NotAgs4` / `UnsupportedEdition` are actually reachable — there is no
/// filesystem (so no `NotFound`/`Io`), decode is lossy (non-UTF-8 surfaces
/// as a Rule 1 finding), and we never set `custom_dict` (so no `BadDict`) —
/// but we map every arm so the `match` is total and future-proof.
fn classify(e: &ValidatorError) -> (&'static str, String) {
    // Delegate to the single producer `ValidatorError::kind()`, except the
    // deliberate, allowlisted divergence: with no filesystem, `NotFound`/`Io` are
    // unreachable, so they collapse to `"io"` here (vs the producer's
    // `"not_found"`) purely to keep the match total. Gated in the tests below.
    let kind = match e {
        ValidatorError::NotFound(_) | ValidatorError::Io { .. } => "io",
        other => other.kind(),
    };
    (kind, e.to_string())
}

/// Resolve a UI encoding label to an `encoding_rs` encoding, via the shared label
/// table in the parse leaf so a label means the same thing on every surface.
///
/// An unknown label is an ERROR, not a fallback. It used to return UTF-8 — which
/// reads like leniency and behaves like corruption: `C3 A9` decodes cleanly as `é`
/// in UTF-8 and `Ã©` in cp1252, so a caller who asked for the wrong label got the
/// wrong text and a clean bill of health. Python raised on the same input. The
/// browser's own select only offers UTF-8 / Windows-1252, so the UI cannot trip
/// this; the wasm API is public, and a caller who names a charset we do not know
/// deserves to be told so.
fn resolve_encoding(
    label: Option<&str>,
) -> std::result::Result<&'static encoding_rs::Encoding, String> {
    laterite_ags4_parse::resolve_encoding(label)
        .ok_or_else(|| format!("unknown encoding {:?}", label.unwrap_or("")))
}

/// Map a UI dict-version string to a forced edition. `None` / `"auto"`
/// ⇒ `Ok(None)` (auto-detect from `TRAN_AGS`). An unrecognised string
/// returns `Err(message)` (the caller turns it into a `bad_args`
/// report); we return the short message rather than the whole report so
/// the `Err` variant stays small (clippy `result_large_err`).
fn resolve_dict_override(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            format!(
                "unknown dict_version {other:?}; expected auto|{}",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

/// Build the runtime custom-dictionary overlay (#568) from browser-supplied bytes.
///
/// The wasm sandbox has no filesystem, so — unlike the CLI/Python/Node twins — this
/// has no path arm: a custom dict always arrives as raw bytes (a `Uint8Array` the UI
/// read from a file). `over` forces a base edition (from `dict_version`),
/// `dict_replace` drops the base entirely, and the two cannot both hold (a forced
/// base contradicts a full replacement). `enc` is the caller's already-resolved
/// source encoding — the same one it hands `CheckOptions`.
///
/// Returns `Ok(None)` when no dict was supplied. The error is a short message the
/// caller surfaces on the same channel a bad `dict_version` uses.
fn build_custom_dict(
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
) -> std::result::Result<Option<overlay::CustomDict>, String> {
    let Some(bytes) = dict_bytes else {
        return Ok(None);
    };
    if dict_replace && over.is_some() {
        return Err("dict_replace cannot be combined with dict_version \
             (a forced base contradicts a full replacement)"
            .to_string());
    }
    let base = if dict_replace {
        overlay::BaseSpec::Replace
    } else if let Some(v) = over {
        overlay::BaseSpec::Force(v)
    } else {
        overlay::BaseSpec::Auto
    };
    // The advisory name the cert records is a neutral label — never a filesystem path
    // (the browser has none anyway), matching the in-memory-bytes arm on every surface.
    overlay::parse_dict(bytes, overlay::DictFormat::Auto, enc, base, "custom-dict")
        .map(Some)
        .map_err(|e| format!("bad dict: {e}"))
}

// --- AGS4 production: `build_ags4` (the read path reversed) -----------------

/// One group of input data, deserialised from the browser's JSON. The
/// column `headings` are the AGS headings; `units`/`types` are optional
/// per-heading overrides (the dictionary fills the rest); `rows` cells are
/// JSON values (numbers/strings/bools/null).
#[derive(Deserialize)]
struct GroupInputJson {
    code: String,
    headings: Vec<String>,
    #[serde(default)]
    units: Option<Vec<String>>,
    #[serde(default)]
    types: Option<Vec<String>>,
    rows: Vec<Vec<serde_json::Value>>,
}

/// One emit finding, flattened with its rule label for the JS side.
#[derive(Serialize)]
struct EmitFinding {
    rule: String,
    line: Option<u32>,
    group: String,
    desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

/// One fix the emit actually applied. Deliberately NOT the engine's `Fix`:
/// that carries `edits` (the spans describing how a *proposed* fix would be
/// applied), which say nothing once it has been. `{kind, label, rule, line,
/// risk}` is the shape Python's `BuildResult.applied` and Node's
/// `EmitResult.applied` present, so all three surfaces read identically
/// (#294 F#7). `kind`/`risk` are the serde `snake_case` strings — the enums
/// carry `rename_all`, so embedding them emits exactly the strings the other
/// two hand-map to.
#[derive(Serialize)]
struct AppliedFix {
    kind: fixes::FixKind,
    label: String,
    rule: String,
    line: Option<u32>,
    risk: fixes::FixRisk,
}

/// The `build_ags4` result. `text` is the AGS4 document (UTF-8, CRLF line
/// endings) — the browser wraps it in a `Blob` to download.
///
/// `applied` is the ledger of what AutoFix rewrote; `fixes_applied` is its
/// length, kept because it is the released shape. Both are needed: AutoFix
/// returns only *residual* findings, so without the ledger a caller can say
/// "3 fixes applied" but not which — recoverable only by re-emitting in
/// `report` mode and re-running `compute_fixes`, which re-parses and
/// re-validates to recover what this call already computed.
#[derive(Serialize)]
struct BuildAgs4Report {
    text: String,
    findings: Vec<EmitFinding>,
    applied: Vec<AppliedFix>,
    fixes_applied: usize,
}

// `build_ags4` returning `any` is the specific defect a downstream browser
// consumer reported: the call that produces a *file* told them nothing about
// what it handed back, so `.text` was reached for on faith. Held by
// `ts_interfaces_match_the_serde_structs`; the two enum unions are held by
// `fix_unions_match_the_validators_enums`, which asks serde itself for the
// authoritative variant list rather than trusting this comment.
ts_section! {
    TS_BUILD_RESULT,
    TS_BUILD_RESULT_SECTION,
    r#"
/** A residual finding from an emit — what AutoFix could *not* resolve. */
export interface EmitFinding {
  rule: string;
  line: number | null;
  group: string;
  desc: string;
  /** **Absent means `"error"`** — see `FindingDto.severity`. */
  severity?: "warning" | "fyi";
}

/** One fix the emit actually applied. Deliberately not the engine's proposed
 *  `Fix`: the `edits` spans describe how a fix *would* be applied and say
 *  nothing once it has been. */
export interface AppliedFix {
  kind: "normalize_crlf" | "strip_bom" | "strip_embedded_cr"
      | "rename_duplicate_heading" | "insert_tran_dlim" | "insert_tran_rcon"
      | "reformat_numeric" | "canonicalize_datetime" | "normalize_typography"
      | "pad_short_row";
  label: string;
  /** The exact rule label (`"AGS Format Rule 8"`, …), for cross-linking back
   *  to the originating finding. */
  rule: string;
  line: number | null;
  /** `safe` rewrites are unambiguous from the file alone; `risky` ones guess
   *  intent and are excluded from bulk apply. */
  risk: "safe" | "risky";
}

/** The `build_ags4` / `build_ags4_ipc` result. */
export interface BuildReport {
  /** The AGS4 document: UTF-8, CRLF line endings. */
  text: string;
  /** Findings AutoFix did NOT resolve — residual, not everything found. */
  findings: EmitFinding[];
  /** The ledger of what AutoFix rewrote. Needed alongside `fixes_applied`
   *  because AutoFix returns only residual findings: without it a caller can
   *  say "3 fixes applied" but not which. */
  applied: AppliedFix[];
  /** `applied.length`, kept because it is the released shape. */
  fixes_applied: number;
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BuildReport")]
    pub type BuildReportJs;
}

fn emit_edition(s: Option<&str>) -> Result<DictVersion, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(FALLBACK),
        Some(other) => DictVersion::from_edition(other).ok_or_else(|| {
            format!(
                "unknown edition {other:?}; expected {}",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

fn emit_mode(s: Option<&str>) -> Result<laterite_ags4_emit::EmitMode, String> {
    match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("") | Some("autofix") => Ok(laterite_ags4_emit::EmitMode::AutoFix),
        Some("report") => Ok(laterite_ags4_emit::EmitMode::Report),
        Some("strict") => Ok(laterite_ags4_emit::EmitMode::Strict),
        Some(other) => Err(format!(
            "unknown mode {other:?}; expected autofix|report|strict"
        )),
    }
}

/// The `tran` argument's wire shape — one object, not five positional slots.
///
/// Every field is `Option` here and required by `TranStamp::from_parts`, which
/// is what makes a typo loud on this surface: `{ producer }` misspelled leaves
/// `producer` unset, and "all five or none" reports it by name. That matters,
/// because `serde(deny_unknown_fields)` is a **no-op** under serde-wasm-bindgen
/// — its `ObjectAccess` walks serde's known fields and `Reflect`-gets each,
/// never enumerating what the caller actually passed. Requiredness is doing the
/// work an unknown-key guard cannot do here.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct TranInput {
    issue: Option<String>,
    date: Option<String>,
    producer: Option<String>,
    recipient: Option<String>,
    status: Option<String>,
    description: Option<String>,
    remarks: Option<String>,
}

/// The thin JS→Rust shim. Deliberately holds no policy: the decision about what
/// constitutes a complete stamp lives in `TranStamp::from_parts` in the emit
/// crate, which is host-testable. This crate's tests run on the HOST with no
/// `wasm-bindgen-test`, so anything with `JsValue` in it ships unexecuted.
impl TranInput {
    /// Fold to the shared type. The policy — all five or none — lives in
    /// `TranStamp::from_parts` in the emit crate, so no surface can answer
    /// "is this enough to stamp a TRAN" differently from the others.
    ///
    /// Pure, and that also gives the NESTED object typo protection the
    /// top-level guard cannot reach: `reject_unknown_keys` enumerates only the
    /// outer object's keys, so a misspelled `producr` inside `tran` slips past
    /// it — but then `producer` is unset, and "all five or none" reports it by
    /// name. Requiredness covers what enumeration does not.
    fn fold(self) -> Result<Option<laterite_ags4_emit::TranStamp>, String> {
        let stamp = laterite_ags4_emit::TranStamp::from_parts(
            self.issue,
            self.date,
            self.producer,
            self.recipient,
            self.status,
        )
        .map_err(|e| e.to_string())?;
        Ok(stamp.map(|s| {
            let s = match self.description {
                Some(d) => s.with_description(d),
                None => s,
            };
            match self.remarks {
                Some(r) => s.with_remarks(r),
                None => s,
            }
        }))
    }
}

/// `build_ags4` / `build_ags4_ipc`'s named options.
///
/// `tran` is a NESTED struct rather than a `JsValue`, so serde builds it
/// directly and `TranInput::fold` applies the shared completeness rule. See
/// that method for why the nested object does not need its own key guard.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct BuildOptions {
    dict_version: Option<String>,
    mode: Option<String>,
    synthesise_metadata: Option<bool>,
    tran: Option<TranInput>,
}

impl WasmOptions for BuildOptions {
    const KEYS: &'static [&'static str] = &["dictVersion", "mode", "synthesiseMetadata", "tran"];
    const WHAT: &'static str = "build options";
}

#[wasm_bindgen(typescript_custom_section)]
const TS_BUILD_OPTIONS: &'static str = r#"
/** The transmission a built file represents. All five are required together —
 *  they are REQUIRED `TRAN` headings, so a partial stamp emits a `TRAN` that
 *  fails Rule 10b. Omit the whole object and no `TRAN` is written; Rule 14 then
 *  reports the gap, which is the honest outcome.
 *
 *  `TRAN_AGS`, `TRAN_DLIM` and `TRAN_RCON` are absent on purpose: they describe
 *  the file the emitter is writing, so it fills them. */
export interface TranStamp {
  /** `TRAN_ISNO` — the issue sequence reference. */
  issue: string;
  /** `TRAN_DATE` — `yyyy-mm-dd`. */
  date: string;
  /** `TRAN_PROD` — who produced the file. */
  producer: string;
  /** `TRAN_RECV` — who it is for. */
  recipient: string;
  /** `TRAN_STAT` — e.g. `"FINAL"`. */
  status: string;
  /** `TRAN_DESC` — what was transferred. */
  description?: string;
  /** `TRAN_REM` — free remarks. */
  remarks?: string;
}

/** Named options for `build_ags4` and `build_ags4_ipc`. */
export interface BuildOptions {
  /** The edition to write against. `"auto"` (or omitted) uses the standard. */
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
  /** `"autofix"` (default) repairs what it safely can, `"report"` emits
   *  unmodified with findings, `"strict"` refuses invalid output. */
  mode?: "autofix" | "report" | "strict";
  /** Mint the mandatory catalogs your data doesn't carry — `UNIT` and `TYPE`
   *  from the columns, `ABBR` when `PA` codes are used. Default **false**.
   *  `PROJ`, `DICT` and `TRAN` are never invented; `TRAN` comes from `tran`. */
  synthesiseMetadata?: boolean;
  /** The transmission this file represents. */
  tran?: TranStamp;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BuildOptions")]
    pub type BuildOptionsJs;
}

/// Core of [`build_ags4`], host-testable (no `JsValue`): parse the JSON, run
/// the shared `laterite-ags4-emit` orchestrator, flatten the findings.
fn build_ags4_from_json(
    groups_json: &str,
    edition: Option<&str>,
    mode: Option<&str>,
    synthesise_metadata: bool,
    tran: Option<laterite_ags4_emit::TranStamp>,
) -> Result<BuildAgs4Report, String> {
    let parsed: Vec<GroupInputJson> =
        serde_json::from_str(groups_json).map_err(|e| format!("invalid groups JSON: {e}"))?;
    let groups: Vec<laterite_ags4_emit::GroupInput> = parsed
        .into_iter()
        .map(|g| laterite_ags4_emit::GroupInput {
            code: g.code,
            headings: g.headings,
            units: g.units,
            types: g.types,
            rows: g.rows,
        })
        .collect();
    emit_report(groups, edition, mode, synthesise_metadata, tran)
}

/// Run the shared orchestrator over already-built `GroupInput`s and shape the
/// JS report. The common tail of both input paths (JSON and Arrow IPC).
fn emit_report(
    groups: Vec<laterite_ags4_emit::GroupInput>,
    edition: Option<&str>,
    mode: Option<&str>,
    synthesise_metadata: bool,
    tran: Option<laterite_ags4_emit::TranStamp>,
) -> Result<BuildAgs4Report, String> {
    let opts = laterite_ags4_emit::EmitOpts {
        mode: emit_mode(mode)?,
        edition: emit_edition(edition)?,
        // `None` here means no TRAN is minted and Rule 14 reports the gap. The
        // browser gets the same five caller-supplied fields `merge` already
        // takes, because who sent what to whom is not something the engine can
        // derive — see EmitOpts::tran.
        tran,
        // Synthesis is OFF unless asked for (2026-07-24): no surface mints
        // GROUPs the caller never wrote without being told to. The caller's
        // ability to *ask* is the parity part — Python takes
        // `synthesise_metadata=`, Node `{ synthesiseMetadata }`, and this is
        // the browser's. See EmitOpts::synthesise_metadata.
        synthesise_metadata,
        // Every field listed explicitly, with NO `..default()` tail — on
        // purpose. Inheriting defaults is what made this surface silently lose
        // `synthesise_metadata` when it went opt-in: the option existed, wasm
        // just never passed it, and nothing failed. Spelling the struct out
        // turns the next new EmitOpts field into a compile error here, forcing
        // a decision about whether the browser should expose it.
    };
    let res = laterite_ags4_emit::emit_ags4(&groups, &opts).map_err(|e| e.to_string())?;
    let findings = res
        .findings
        .iter()
        .flat_map(|(rule, items)| {
            items.iter().map(move |f| EmitFinding {
                rule: rule.clone(),
                line: f.line,
                group: f.group.clone(),
                desc: f.desc.clone(),
                severity: match f.severity {
                    findings::Severity::Error => None,
                    s => Some(s.as_str().to_string()),
                },
            })
        })
        .collect();
    let applied = res
        .applied
        .iter()
        .map(|f| AppliedFix {
            kind: f.kind,
            label: f.label.clone(),
            rule: f.rule.clone(),
            line: f.line,
            risk: f.risk,
        })
        .collect();
    Ok(BuildAgs4Report {
        text: String::from_utf8_lossy(&res.bytes).into_owned(),
        findings,
        applied,
        fixes_applied: res.fixes_applied,
    })
}

/// Decode one group's Arrow IPC stream → a [`laterite_ags4_emit::GroupInput`] (the
/// column names are the AGS headings). Uses the shared Arrow→Value transpose.
fn group_from_ipc(code: String, bytes: &[u8]) -> Result<laterite_ags4_emit::GroupInput, String> {
    let reader = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(bytes), None)
        .map_err(|e| format!("arrow ipc: {e}"))?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for b in reader {
        batches.push(b.map_err(|e| format!("arrow ipc batch: {e}"))?);
    }
    // Never emit a synthetic content-addressed key column: drop any `_`-prefixed
    // column a `read(keys=true)` frame might carry back into build. AGS headings
    // never start with "_", so this is safe (a no-op when none are present). (#303)
    let keep: Vec<usize> = (0..schema.fields().len())
        .filter(|&i| !schema.field(i).name().starts_with('_'))
        .collect();
    if keep.len() != schema.fields().len() {
        let pschema = std::sync::Arc::new(
            schema
                .project(&keep)
                .map_err(|e| format!("arrow project schema: {e}"))?,
        );
        let pbatches: Vec<arrow::record_batch::RecordBatch> = batches
            .iter()
            .map(|b| b.project(&keep))
            .collect::<Result<_, _>>()
            .map_err(|e| format!("arrow project batch: {e}"))?;
        return Ok(laterite_ags4_emit::group_from_arrow(
            code,
            pschema.as_ref(),
            &pbatches,
        ));
    }
    Ok(laterite_ags4_emit::group_from_arrow(
        code,
        schema.as_ref(),
        &batches,
    ))
}

/// `(edition, mode, synthesise_metadata, tran)` — what [`emit_report`] takes,
/// once the options object has been folded down to it.
type BuildParts = (
    Option<String>,
    Option<String>,
    bool,
    Option<laterite_ags4_emit::TranStamp>,
);

/// The parts of a [`BuildOptions`] the emit path takes, with the completeness
/// rule applied to `tran` and the documented default for `synthesiseMetadata`.
///
/// Split out because it was the only *decision* the two build exports made:
/// both held an identical five-line fold sitting behind a `JsValue` parameter,
/// so the "all five or none" rule and the synthesis default were enforced twice
/// and testable in neither place.
fn build_parts(o: BuildOptions) -> Result<BuildParts, String> {
    let tran = o.tran.map(TranInput::fold).transpose()?.flatten();
    Ok((
        o.dict_version,
        o.mode,
        o.synthesise_metadata.unwrap_or(false),
        tran,
    ))
}

/// The host-testable core of [`build_ags4`] — everything but the JS decode and
/// the JS serialise.
fn build_ags4_core(groups_json: &str, o: BuildOptions) -> Result<BuildAgs4Report, String> {
    let (edition, mode, synth, tran) = build_parts(o)?;
    build_ags4_from_json(
        groups_json,
        edition.as_deref(),
        mode.as_deref(),
        synth,
        tran,
    )
}

/// The host-testable core of [`build_ags4_ipc`], taking the groups already
/// decoded from the JS array (that walk is genuinely `Reflect` work and stays
/// at the boundary).
fn build_ipc_core(
    inputs: Vec<laterite_ags4_emit::GroupInput>,
    o: BuildOptions,
) -> Result<BuildAgs4Report, String> {
    let (edition, mode, synth, tran) = build_parts(o)?;
    emit_report(inputs, edition.as_deref(), mode.as_deref(), synth, tran)
}

/// Build valid AGS4 from typed/string data in the browser — the data→AGS4
/// producer (the read path reversed), with no server round-trip.
///
/// * `groups_json` — a JSON array of `{ code, headings, units?, types?, rows }`
///   (each row an array of cell values). The headings are the AGS headings;
///   UNIT/TYPE fill from the chosen edition's dictionary where omitted.
/// * `dict_version` — `None`/`"auto"` → `4.1.1`, or `4.0.3|4.0.4|4.1|4.1.1|4.2`.
/// * `mode` — `None`/`"autofix"` (default) | `"report"` | `"strict"`. `"autofix"`
///   repairs what the input contains. It does NOT mint the mandatory
///   UNIT/TYPE/TRAN/ABBR catalogs — that became opt-in on 2026-07-24, so a
///   data-only build reports Rule 14/15/17 rather than silently filling them.
/// * `synthesise_metadata` — `None`/`false` (default) | `true` to mint the
///   mandatory UNIT/TYPE/ABBR catalogs a data-only build is missing, clearing
///   Rule 15/17. Only meaningful under `"autofix"`. This is the browser twin of
///   Python's `synthesise_metadata=` and Node's `{ synthesiseMetadata }`.
/// * `tran_issue` / `tran_date` / `tran_producer` / `tran_recipient` /
///   `tran_status` — the transmission this file represents. Supply them and
///   synthesis stamps a real TRAN; omit them and **no TRAN is written**, so
///   Rule 14 reports the gap.
///
///   That asymmetry is the point. A stub reading `TBC`/`1900-01-01` still
///   SATISFIES Rule 14, so a recipient cannot tell an invented transmission
///   record from a real one and nothing downstream flags it. Who produced a
///   file, for whom, when and at what status is knowable only to the caller —
///   the same reason PROJ and DICT are never synthesised. These are the five
///   arguments `merge` already takes, named identically.
///
/// Returns `{ text, findings, applied, fixes_applied }`; `text` is the AGS4
/// document (UTF-8, CRLF) for the browser to wrap in a `Blob`. `applied` is the
/// ledger of what AutoFix rewrote (`{kind, label, rule, line, risk}` per fix —
/// the same shape Python and Node present), `fixes_applied` its length.
#[wasm_bindgen]
pub fn build_ags4(
    groups_json: &str,
    opts: Option<BuildOptionsJs>,
) -> Result<BuildReportJs, JsError> {
    console_error_panic_hook::set_once();
    let o: BuildOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let report = build_ags4_core(groups_json, o).map_err(|e| JsError::new(&e))?;
    to_js(&report)
}

/// Build valid AGS4 from **columnar Arrow IPC** input — the same as
/// [`build_ags4`] but for large, already-columnar browser data (e.g. a
/// duckdb-wasm query result) without a per-cell JSON round-trip.
///
/// * `groups` — a JS array of `{ code: string, ipc: Uint8Array }`, each `ipc`
///   an Arrow **IPC stream** for one group (its schema's field names are the
///   AGS headings). Order is preserved (put `PROJ` first).
/// * `dict_version` / `mode` / `synthesise_metadata` — as [`build_ags4`].
///
/// Returns the same `{ text, findings, applied, fixes_applied }`. The Arrow→AGS
/// transpose is the read path's IPC reversed.
#[wasm_bindgen]
pub fn build_ags4_ipc(
    // Stays a raw `JsValue` on its own hand-written `Reflect` path below. An
    // array of `{ code, ipc: Uint8Array }` routed through serde would put every
    // `ipc` BYTE through `deserialize_seq` — which is the exact cost this
    // columnar door exists to avoid. Only the OPTION tail moved.
    groups: JsValue,
    opts: Option<BuildOptionsJs>,
) -> Result<BuildReportJs, JsError> {
    use wasm_bindgen::JsCast;
    console_error_panic_hook::set_once();
    let o: BuildOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let arr = js_sys::Array::from(&groups);
    let mut inputs: Vec<laterite_ags4_emit::GroupInput> = Vec::with_capacity(arr.length() as usize);
    for item in arr.iter() {
        let code = js_sys::Reflect::get(&item, &JsValue::from_str("code"))
            .ok()
            .and_then(|v| v.as_string())
            .ok_or_else(|| JsError::new("each group needs a string `code`"))?;
        let ipc_val = js_sys::Reflect::get(&item, &JsValue::from_str("ipc"))
            .map_err(|_| JsError::new("each group needs an `ipc` Uint8Array"))?;
        let ipc = ipc_val
            .dyn_into::<js_sys::Uint8Array>()
            .map_err(|_| JsError::new("group `ipc` must be a Uint8Array"))?
            .to_vec();
        inputs.push(group_from_ipc(code, &ipc).map_err(|e| JsError::new(&e))?);
    }
    let report = build_ipc_core(inputs, o).map_err(|e| JsError::new(&e))?;
    to_js(&report)
}

#[cfg(test)]
mod build_ags4_tests {
    use super::*;

    #[test]
    fn emits_valid_and_canonicalises_typed() {
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],"rows":[["P1","Demo"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01",12.3]]}
        ]"#;
        let r = build_ags4_from_json(json, Some("4.1.1"), Some("autofix"), false, None).unwrap();
        assert!(
            r.text.contains("\"12.30\""),
            "expected canonical 2DP:\n{}",
            r.text
        );
        assert!(
            r.text.contains("\"UNIT\",\"\",\"m\""),
            "dict UNIT fill:\n{}",
            r.text
        );
        // The emitted text re-parses as well-formed AGS4.
        let parsed = parse_bytes(r.text.as_bytes(), encoding_rs::UTF_8).unwrap();
        assert!(parsed.groups.contains_key("LOCA"));
    }

    /// The browser can ASK for metadata synthesis. Synthesis went opt-in on
    /// 2026-07-24 and Python/Node each gained a flag to opt back in, but wasm
    /// took `EmitOpts::default()` with no override — so on this surface the
    /// capability was unreachable, not merely off. Both directions are pinned
    /// here: a canned `true` that never reaches `EmitOpts` fails the second
    /// half, and a hardcoded `true` fails the first.
    #[test]
    fn synthesise_metadata_is_opt_in_and_reachable() {
        // Data only: no TRAN/UNIT/TYPE catalogs, so Rules 14/15/17 fire.
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]},
          {"code":"LOCA","headings":["LOCA_ID"],"rows":[["BH01"]]}
        ]"#;
        let metadata_rules = |r: &BuildAgs4Report| -> Vec<String> {
            let mut v: Vec<String> = r
                .findings
                .iter()
                .map(|f| f.rule.clone())
                .filter(|rule| ["14", "15", "17"].iter().any(|n| rule.contains(n)))
                .collect();
            v.sort();
            v.dedup();
            v
        };

        // Default (and explicit false): the catalogs are NOT minted.
        let off = build_ags4_from_json(json, None, Some("autofix"), false, None).unwrap();
        assert!(
            !metadata_rules(&off).is_empty(),
            "a data-only build should report the missing metadata catalogs:\n{}",
            off.text
        );
        assert!(
            !off.text.contains("\"GROUP\",\"TRAN\""),
            "nothing should be minted unasked:\n{}",
            off.text
        );

        // Opt in WITHOUT a stamp: the derivable catalogs are minted and their
        // findings clear, but no TRAN is invented — Rule 14 keeps reporting.
        let on = build_ags4_from_json(json, None, Some("autofix"), true, None).unwrap();
        assert!(
            !on.text.contains("\"GROUP\",\"TRAN\""),
            "an unstamped build must not invent a TRAN:\n{}",
            on.text
        );
        assert!(
            metadata_rules(&on).len() < metadata_rules(&off).len(),
            "synthesis should clear the DERIVABLE metadata findings (was {:?}, now {:?})",
            metadata_rules(&off),
            metadata_rules(&on)
        );

        // Opt in WITH a stamp: TRAN appears, carrying the caller's values.
        let stamped = build_ags4_from_json(
            json,
            None,
            Some("autofix"),
            true,
            Some(laterite_ags4_emit::TranStamp::new(
                "1",
                "2026-07-30",
                "Acme Ground Engineering",
                "Client Ltd",
                "FINAL",
            )),
        )
        .unwrap();
        assert!(
            stamped.text.contains("\"GROUP\",\"TRAN\""),
            "a stamped build should carry a TRAN:\n{}",
            stamped.text
        );
        assert!(
            stamped.text.contains("Acme Ground Engineering") && !stamped.text.contains("TBC"),
            "the TRAN must be the caller's, never a placeholder:\n{}",
            stamped.text
        );
    }

    #[test]
    fn autofix_pads_a_string_numeric() {
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}
        ]"#;
        let r = build_ags4_from_json(json, None, Some("autofix"), false, None).unwrap();
        assert!(r.fixes_applied >= 1, "AutoFix should apply a safe fix");
        assert!(r.text.contains("\"12.30\""), "{}", r.text);
    }

    #[test]
    fn report_keeps_strings_verbatim() {
        let json = r#"[{"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}]"#;
        let r = build_ags4_from_json(json, None, Some("report"), false, None).unwrap();
        assert!(r.text.contains("\"12.3\""));
        assert_eq!(r.fixes_applied, 0);
        assert!(r.applied.is_empty(), "report mode rewrites nothing");
    }

    /// The count alone cannot say WHICH fix ran, and AutoFix returns only
    /// residual findings — so a caller with `fixes_applied: 1` and no ledger has
    /// no way to review the rewrite short of re-emitting and re-validating.
    /// Asserts the ledger is populated AND agrees with the count, because the
    /// two coming apart is the failure that would look fine in a UI.
    #[test]
    fn autofix_reports_which_fixes_it_applied() {
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}
        ]"#;
        let r = build_ags4_from_json(json, None, Some("autofix"), false, None).unwrap();
        assert_eq!(
            r.applied.len(),
            r.fixes_applied,
            "the ledger and the count must describe the same work"
        );
        let numeric = r
            .applied
            .iter()
            .find(|f| f.kind == fixes::FixKind::ReformatNumeric)
            .expect("the 12.3 → 12.30 pad is a ReformatNumeric fix");
        assert_eq!(numeric.rule, "AGS Format Rule 8");
        assert_eq!(numeric.risk, fixes::FixRisk::Safe);
    }

    #[test]
    fn rejects_unknown_mode_and_edition() {
        let json = r#"[{"code":"LOCA","headings":["LOCA_ID"],"rows":[["BH01"]]}]"#;
        assert!(build_ags4_from_json(json, None, Some("banana"), false, None).is_err());
        assert!(build_ags4_from_json(json, Some("9.9"), None, false, None).is_err());
    }

    #[test]
    fn ipc_columnar_input_emits_valid() {
        use arrow::array::{Float64Array, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        // Serialize a batch to an Arrow IPC stream (the read path's writer).
        fn ipc_bytes(schema: &Arc<Schema>, batch: &RecordBatch) -> Vec<u8> {
            let mut buf = Vec::new();
            let mut w = arrow::ipc::writer::StreamWriter::try_new(&mut buf, schema).unwrap();
            w.write(batch).unwrap();
            w.finish().unwrap();
            drop(w);
            buf
        }

        let proj_schema = Arc::new(Schema::new(vec![Field::new(
            "PROJ_ID",
            DataType::Utf8,
            false,
        )]));
        let proj_batch = RecordBatch::try_new(
            proj_schema.clone(),
            vec![Arc::new(StringArray::from(vec!["P1"]))],
        )
        .unwrap();
        let loca_schema = Arc::new(Schema::new(vec![
            Field::new("LOCA_ID", DataType::Utf8, false),
            Field::new("LOCA_GL", DataType::Float64, true), // a 2DP heading
        ]));
        let loca_batch = RecordBatch::try_new(
            loca_schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["BH01", "BH02"])),
                Arc::new(Float64Array::from(vec![12.3, 13.0])),
            ],
        )
        .unwrap();

        // Decode each IPC stream via the shared transpose, then emit.
        let proj = group_from_ipc("PROJ".into(), &ipc_bytes(&proj_schema, &proj_batch)).unwrap();
        let loca = group_from_ipc("LOCA".into(), &ipc_bytes(&loca_schema, &loca_batch)).unwrap();
        let r = emit_report(
            vec![proj, loca],
            Some("4.1.1"),
            Some("autofix"),
            false,
            None,
        )
        .unwrap();

        assert!(r.text.contains("\"12.30\""), "float64 → 2DP:\n{}", r.text);
        assert!(r.text.contains("\"13.00\""), "{}", r.text);
        assert!(
            r.text.contains("\"UNIT\",\"\",\"m\""),
            "dict fill:\n{}",
            r.text
        );
        let parsed = parse_bytes(r.text.as_bytes(), encoding_rs::UTF_8).unwrap();
        assert!(parsed.groups.contains_key("LOCA"));
    }
}

// ---------------------------------------------------------------------------
// Options objects
// ---------------------------------------------------------------------------
//
// The browser reached these verbs through positional slots, so `validate`'s
// third argument was a bare `true` at every call site and the eighth was
// unreachable without passing five `undefined`s. Named fields fix that, but
// they introduce a hazard positional arguments structurally cannot have: a
// MISSPELLED key. You cannot typo slot 3; you can very easily write
// `synthesizeMetadata`.
//
// `#[serde(deny_unknown_fields)]` does NOT catch it here. serde-wasm-bindgen's
// `ObjectAccess::next_key_seed` walks serde's list of KNOWN fields and
// `Reflect`-gets each one — it never enumerates what the caller actually
// passed, so an unrecognised key is invisible to serde and the option silently
// takes its default. Writing that attribute would look like protection and be
// none, which is why it is deliberately absent below. `reject_unknown_keys`
// does the work instead, by enumeration.

/// Binds an options struct to the key list its callers may use.
///
/// **Why a trait and not a `&[&str]` argument to `decode_opts`:** several
/// exports each have their own key list, and four interchangeable `&[&str]`
/// consts passed by hand is a silent failure waiting — hand `CertifyOptions`'
/// keys to `validate` and every `validate` typo is accepted again, while a
/// drift test that only checks each const against its own struct stays green.
/// Bound to the type, the pairing cannot be got wrong.
trait WasmOptions: serde::de::DeserializeOwned + Default {
    /// Every accepted key, in the caller's camelCase spelling. Kept honest
    /// against the struct's own serde names by `option_keys_match_the_structs`.
    const KEYS: &'static [&'static str];
    /// What to call this object in an error message.
    const WHAT: &'static str;
}

/// Is `present` an accepted key, and if not, what should we say about it?
///
/// **Pure, and host-testable, on purpose.** `ci.yml` runs this crate's tests on
/// the HOST and the crate carries no `wasm-bindgen-test`, so anything holding a
/// `JsValue` ships with zero executed coverage. Only the key *enumeration*
/// genuinely needs wasm; the decision and the message do not, so they live here
/// where the test suite can actually reach them.
fn unknown_key(known: &[&str], present: &str) -> Option<String> {
    if known.contains(&present) {
        return None;
    }
    // The realistic typos are casing (`DictVersion`) and the s/z spelling split
    // (`synthesizeMetadata`), not arbitrary edit distance — so normalise exactly
    // those two and offer a direct suggestion when one matches.
    let norm = |s: &str| s.to_ascii_lowercase().replace('z', "s");
    Some(match known.iter().find(|k| norm(k) == norm(present)) {
        Some(k) => format!("unknown option {present:?} — did you mean {k:?}?"),
        None => format!(
            "unknown option {present:?}; expected one of {}",
            known.join(", ")
        ),
    })
}

/// Decode an options object, refusing keys the struct does not know.
///
/// Returns the message rather than a `JsError` so each export can route it into
/// the channel it already uses: `validate` folds it into a
/// `ValidationReport::failure("bad_args", …)` like every other caller mistake
/// it reports, while `certify` — already fallible — throws. One decoder, two
/// existing channels, no new third way for an argument to be wrong.
fn decode_opts<T: WasmOptions>(opts: Option<JsValue>) -> Result<T, String> {
    use wasm_bindgen::JsCast;

    let Some(v) = opts.filter(|v| !v.is_undefined() && !v.is_null()) else {
        return Ok(T::default());
    };
    if !v.is_object() {
        return Err(format!(
            "{} must be an object of named options, e.g. {{ {} }}",
            T::WHAT,
            T::KEYS.first().copied().unwrap_or_default()
        ));
    }
    let obj: &js_sys::Object = v.unchecked_ref();
    for key in js_sys::Object::keys(obj).iter() {
        if let Some(k) = key.as_string()
            && let Some(msg) = unknown_key(T::KEYS, &k)
        {
            return Err(format!("{}: {msg}", T::WHAT));
        }
    }
    serde_wasm_bindgen::from_value(v).map_err(|e| format!("{}: {e}", T::WHAT))
}

/// Serialise a plain report into its declared TS type — json-compatible, so the
/// JS side sees objects and `null` rather than `Map`/`undefined`, the same shape
/// the CLI's `--json` emits.
///
/// One helper instead of the same three lines at the end of every export. The
/// tail names `JsValue`, so every copy of it was a line `cargo test` could never
/// reach: collapsing them shrinks the untestable boundary to one place rather
/// than spreading it across every door.
///
/// **Every** door goes through this, the two build doors included. They used
/// serde-wasm-bindgen's *default* serializer until #212, which writes
/// `undefined` for an absent `Option` where this one writes `null` — so
/// `BuildReport`'s published TS declared `line: number | null` while the runtime
/// handed back `undefined`, and a consumer writing `f.line === null` type-checked
/// clean and missed every time. Having one serializer is now the only thing
/// standing between the next result struct and the same bug; `serializer_
/// consistency_tests` asserts there is still only one.
fn to_js<T: Serialize, J: JsCast>(value: &T) -> Result<J, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map(JsCast::unchecked_into)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// `validate`'s named options. Note the ABSENT `deny_unknown_fields` — see the
/// module comment above; `decode_opts` enumerates instead.
///
/// `Serialize` is test-only: it costs nothing in the shipped wasm and, unlike a
/// `cfg(test)` `deny_unknown_fields`, it fabricates no behaviour — the drift
/// test reads the SAME `rename_all` config the shipped deserialize path uses.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct ValidateOptions {
    dict_version: Option<String>,
    /// Defaults to **true** (`.unwrap_or(true)` at the call site, not here).
    /// The positional parameter this replaces was a required `bool`, so there
    /// was no default to preserve — but Python and Node both promise warnings
    /// ON, and a plain `Option<bool>` silently unwrapping to `false` would have
    /// made the browser the one surface that disagreed.
    warnings: Option<bool>,
    fyi: Option<bool>,
    encoding: Option<String>,
    max_per_rule: Option<u32>,
    #[serde(with = "serde_bytes")]
    dictionary: Option<Vec<u8>>,
    dict_replace: Option<bool>,
}

impl WasmOptions for ValidateOptions {
    const KEYS: &'static [&'static str] = &[
        "dictVersion",
        "warnings",
        "fyi",
        "encoding",
        "maxPerRule",
        "dictionary",
        "dictReplace",
    ];
    const WHAT: &'static str = "validate options";
}

/// `certify`'s named options — `ValidateOptions`' dictionary half plus the
/// clock. No `warnings`/`fyi`/`maxPerRule`: the mint measures every tier itself
/// and reports counts, so there is nothing for a caller to include or exclude.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct CertifyOptions {
    dict_version: Option<String>,
    encoding: Option<String>,
    /// RFC-3339, from the browser (`new Date().toISOString()`). Required —
    /// wasm has no clock — but typed `Option` so its absence is OUR error
    /// message naming the field, not serde's generic missing-field text.
    checked_at: Option<String>,
    #[serde(with = "serde_bytes")]
    dictionary: Option<Vec<u8>>,
    dict_replace: Option<bool>,
}

impl WasmOptions for CertifyOptions {
    const KEYS: &'static [&'static str] = &[
        "dictVersion",
        "encoding",
        "checkedAt",
        "dictionary",
        "dictReplace",
    ];
    const WHAT: &'static str = "certify options";
}

// The TypeScript the consumer actually sees. Hand-written rather than derived:
// a generator emits `dictVersion?: string`, where this gives the real edition
// union and per-field docs — and those unions are the single biggest ergonomic
// win in the change. Revisit `tsify` if the struct count grows past four, or if
// the RESULT interfaces (which nothing currently guards) start to drift.
#[wasm_bindgen(typescript_custom_section)]
const TS_OPTIONS: &'static str = r#"
/** Named options for `validate`. */
export interface ValidateOptions {
  /** Force an AGS4 edition instead of reading the file's `TRAN_AGS`.
   *  `"auto"` (and omitting it) reads the file's own `TRAN_AGS`. */
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
  /** Surface WARNING-severity findings. Default **true**. */
  warnings?: boolean;
  /** Surface FYI-severity findings. Default **false**. */
  fyi?: boolean;
  /** `"utf-8"` (default) or `"windows-1252"` for legacy files. */
  encoding?: "utf-8" | "windows-1252";
  /** Cap findings *serialised* per rule. Every rule still runs over every
   *  line and the reported totals stay uncapped — this only bounds how much
   *  crosses the wasm→JS boundary for an interactive view. */
  maxPerRule?: number;
  /** A custom AGS4 dictionary (`.ags` or JSON) as raw bytes. */
  dictionary?: Uint8Array;
  /** With `dictionary`, replace the bundled base entirely rather than
   *  overlaying on it. Contradicts `dictVersion`; both is a `bad_dict` error. */
  dictReplace?: boolean;
}

/** Named options for `certify`. */
export interface CertifyOptions {
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
  encoding?: "utf-8" | "windows-1252";
  /** RFC-3339, e.g. `new Date().toISOString()`. Required: wasm has no clock. */
  checkedAt: string;
  dictionary?: Uint8Array;
  dictReplace?: boolean;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ValidateOptions")]
    pub type ValidateOptionsJs;
    #[wasm_bindgen(typescript_type = "CertifyOptions")]
    pub type CertifyOptionsJs;
}

/// Validate AGS4 bytes in the browser.
///
/// * `data` — the file bytes (from a `FileReader`/textarea, never uploaded).
/// * `opts` — a [`ValidateOptions`] object; every field optional, so
///   `validate(bytes)` is a complete call. An unrecognised key is REFUSED by
///   name rather than silently taking its default (see the options module
///   comment for why serde cannot do that itself).
///
/// This took EIGHT positional arguments, of which three were `bool`. Reaching
/// `dictReplace` meant passing five `undefined`s, and swapping the two adjacent
/// severity flags compiled cleanly on every surface that called it.
///
/// Defaults, matching every other surface: `warnings` **on**, `fyi` **off**,
/// `dictReplace` **off**, edition read from the file's `TRAN_AGS`.
///
/// Infallible by design. A bad option — unknown key, unknown edition, unknown
/// encoding, unparseable dictionary — comes back as
/// `report.error = { kind: "bad_args" | "bad_dict", message }`, the same channel
/// the caller already handles, rather than as a thrown exception that the UI
/// would have to catch somewhere else.
///
/// Returns a [`ValidationReport`] as a plain JS object (json-compatible:
/// `None` → `null`, matching the CLI's `--json`).
#[wasm_bindgen]
pub fn validate(data: &[u8], opts: Option<ValidateOptionsJs>) -> ValidationReportJs {
    console_error_panic_hook::set_once();

    // A bad option key is a caller mistake, and `run` already reports every
    // other caller mistake — unknown edition, unknown encoding, unparseable
    // dictionary — as `bad_args`/`bad_dict` in the report itself. Throwing here
    // instead would split one error channel in two: some argument errors the UI
    // renders through `report.error.kind`, others arriving as a rejected
    // promise. `validate` stays infallible.
    // One serialise for both outcomes: a decode failure is reported through the
    // very same `ValidationReport` a rule failure is, so there is nothing to
    // return early for.
    let report = match decode_opts(opts.map(JsValue::from)) {
        Ok(o) => run(data, &o),
        Err(message) => ValidationReport::failure("bad_args", message),
    };
    // json_compatible so the JS side sees plain objects + null (not Map
    // / undefined) — same shape the CLI emits.
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    report
        .serialize(&serializer)
        .expect("ValidationReport is plain data and always serialises")
        .unchecked_into()
}

/// Mint a `.ags.idx` validity certificate, entirely client-side (#360). Returns the
/// certificate JSON for the browser to download.
///
/// * `checked_at` — an RFC-3339 timestamp from the browser (`new Date()
///   .toISOString()`): wasm has no clock, so the caller supplies it.
///
/// Errors if the file can't be parsed, or has **errors** — warnings and FYI findings are
/// *measured and recorded*, not fatal.
///
/// The mint is the shared one (`laterite_ags4_trust::mint`), so a browser-minted
/// certificate is byte-for-byte the same statement as one from `lat certify`. It used to
/// be assembled here: this surface ran an ERRORS-ONLY validation and then wrote
/// `warnings: 0, fyi: 0` into the stamp — a claim to have measured two tiers it had never
/// looked at, which a later `validate --warnings --index` on any surface would have
/// believed. The mint now measures every tier itself, and there is no parameter left
/// through which this function could assert one.
///
/// It also cannot record an on-disk `FILE/` check: the wasm sandbox has no filesystem, and
/// the stamp no longer has a field in which to say otherwise.
///
/// `dict_bytes` / `dict_replace` mint the certificate against a custom dictionary
/// (#568), the same overlay `validate` accepts: the stamp records the dict's
/// `{name, hash}` so a later `validate --index` on any surface re-validates (never
/// silently vouches) when the effective dictionary differs (O-48, record-not-contract).
#[wasm_bindgen]
pub fn certify(data: &[u8], opts: CertifyOptionsJs) -> Result<String, JsError> {
    console_error_panic_hook::set_once();

    // certify is already fallible, so its decode errors throw — the channel it
    // has, not a new one. (Contrast `validate`, which reports them.)
    let o: CertifyOptions = decode_opts(Some(JsValue::from(opts))).map_err(|m| JsError::new(&m))?;
    certify_core(data, &o).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`certify`], for the same reason [`run`] is one:
/// a `JsValue` in the signature puts a function beyond `cargo test`, which is
/// the only lane this crate has.
fn certify_core(data: &[u8], o: &CertifyOptions) -> Result<String, String> {
    let checked_at = o.checked_at.clone().ok_or_else(|| {
        "certify options: `checkedAt` is required — wasm has no clock, so the caller supplies \
         the timestamp (e.g. new Date().toISOString())"
            .to_string()
    })?;

    let dict_over = resolve_dict_override(o.dict_version.as_deref())?;
    let encoding = resolve_encoding(o.encoding.as_deref())?;
    let custom_dict = build_custom_dict(
        o.dictionary.as_deref(),
        o.dict_replace.unwrap_or(false),
        dict_over,
        encoding,
    )?;

    let opts = CheckOptions {
        dict_version: dict_over,
        encoding,
        custom_dict,
        ..CheckOptions::default()
    };
    let sidecar =
        laterite_ags4_trust::mint(data, &opts, checked_at, None).map_err(|e| e.to_string())?;
    let json = sidecar.to_json().map_err(|e| e.to_string())?;
    String::from_utf8(json).map_err(|e| e.to_string())
}

/// The AGS4 rule catalogue as the gated `rules_meta.json` JSON string — the
/// browser parses it into typed rule entries. Mirrors `laterite.list_rules()` /
/// `lat rules`. No input.
#[wasm_bindgen]
pub fn list_rules() -> String {
    laterite_ags4_validator::rule_metadata_json().to_string()
}

/// The browser highlight's char-span, by precedence: a finding-CARRIED span
/// (Rules 1/6 attach one directly) wins; otherwise, for a field-targeted finding
/// that carries a `field_index`, derive the inner-value span from the raw source
/// line via the parse leaf's [`laterite_ags4_parse::field_span`].
///
/// This derivation exists on NO other surface — `char_span` is serialized only
/// by wasm, and it drives the browser's cell/heading highlight — yet before #555
/// part 1 it had zero test coverage (and `field_span`, the leaf it calls, had
/// none either). Extracted from `run`'s finding-mapping closure so the precedence
/// AND the offset it produces are pinned by the `derive_char_span_*` tests.
fn derive_char_span(
    carried: Option<(u32, u32)>,
    field_index: Option<u32>,
    raw_line: Option<&str>,
) -> Option<[u32; 2]> {
    carried
        .map(|(s, e)| [s, e])
        .or_else(|| laterite_ags4_parse::field_span(raw_line?, field_index?).map(|(s, e)| [s, e]))
}

#[cfg(test)]
mod char_span_tests {
    use super::derive_char_span;

    // A DATA row whose tag-stripped column 1 is the value "10.5".
    const LINE: &str = r#""DATA","BH1","10.5""#;

    #[test]
    fn carried_span_wins_over_derivation() {
        // Rules 1/6 attach an explicit span; it is used verbatim even when a
        // field_index + line are also present — the derivation must not override.
        assert_eq!(
            derive_char_span(Some((3, 7)), Some(1), Some(LINE)),
            Some([3, 7])
        );
    }

    #[test]
    fn derives_inner_value_span_from_field_index() {
        // No carried span → derive from the raw line. field_index is the
        // TAG-STRIPPED column, so on `"DATA","BH1","10.5"` index 1 is the value
        // "10.5" at chars 14..18 and index 0 is "BH1" at 8..11 — the exact
        // offsets the browser highlights, and the first coverage of both this
        // derivation and the leaf `field_span` it calls.
        assert_eq!(derive_char_span(None, Some(1), Some(LINE)), Some([14, 18]));
        assert_eq!(derive_char_span(None, Some(0), Some(LINE)), Some([8, 11]));
    }

    #[test]
    fn none_without_field_index() {
        // A whole-line / whole-group finding (no field_index) gets no span.
        assert_eq!(derive_char_span(None, None, Some(LINE)), None);
    }

    #[test]
    fn none_without_raw_line() {
        // field_index present but the source line unavailable → no span, no panic.
        assert_eq!(derive_char_span(None, Some(1), None), None);
    }
}

/// The host-testable core of [`validate`]: no `JsValue`, so `cargo test` can
/// actually reach it (this crate's tests run on the host and it carries no
/// `wasm-bindgen-test`).
///
/// Takes the decoded options struct rather than eight positional parameters.
/// Three of those were `bool` — `include_warnings`, `include_fyi`,
/// `dict_replace` — so `run(data, None, true, true, None, None, None, false)`
/// was a legal call in which swapping the two adjacent flags compiled cleanly
/// and silently changed what the report contained. Field names remove that; the
/// defaults still resolve here, so every door gets the same ones.
fn run(data: &[u8], o: &ValidateOptions) -> ValidationReport {
    let include_warnings = o.warnings.unwrap_or(true);
    let include_fyi = o.fyi.unwrap_or(false);
    let max_per_rule = o.max_per_rule.map(|c| c as usize);
    let dict_bytes = o.dictionary.as_deref();

    let dict_over = match resolve_dict_override(o.dict_version.as_deref()) {
        Ok(v) => v,
        Err(message) => return ValidationReport::failure("bad_args", message),
    };
    let encoding = match resolve_encoding(o.encoding.as_deref()) {
        Ok(e) => e,
        // Same channel a bad dict_version uses: the caller SEES the bad label,
        // instead of getting findings that are artefacts of a UTF-8 fallback.
        Err(message) => return ValidationReport::failure("bad_args", message),
    };
    // The custom-dict overlay is resolved (base detected, delta built, hash minted)
    // once here, before parsing the delivery — a bad dictionary is the DICTIONARY's
    // problem and is reported as such, on the same channel a bad dict_version uses.
    let custom_dict = match build_custom_dict(
        dict_bytes,
        o.dict_replace.unwrap_or(false),
        dict_over,
        encoding,
    ) {
        Ok(c) => c,
        Err(message) => return ValidationReport::failure("bad_dict", message),
    };

    let parsed = match parse_bytes(data, encoding) {
        Ok(p) => p,
        Err(e) => {
            // #168 Phase 3: convert the leaf's ParseError via the validator's
            // `From` bridge so `classify` (and the surfaced text) is unchanged.
            let (kind, message) = classify(&ValidatorError::from(e));
            return ValidationReport::failure(kind, message);
        }
    };

    let opts = CheckOptions {
        dict_version: dict_over,
        include_warnings,
        include_fyi,
        encoding,
        custom_dict,
        ..CheckOptions::default()
    };

    // The one door: it resolves the edition, applies the O-42 4.0.3→4.0.4 content
    // guard, runs the rules, and emits the transparency FYI. The browser used to
    // assemble those steps itself and left the guard out, so a file mislabelled
    // 4.0.3 was judged here against a different dictionary than `lat` used on the
    // same bytes. `WorldScope::None` is the only scope the browser can name —
    // `OnDisk` needs a path, and the wasm sandbox has no filesystem — so this run is
    // filesystem-free by construction, not by a coincidentally-false flag.
    let (found, dv, kind) = match check_parsed_with_dict(&parsed, &opts, &WorldScope::None) {
        Ok(r) => r,
        Err(e) => return ValidationReport::failure(e.kind(), e.to_string()),
    };

    let finding_count = findings::count(&found);
    // Raw line text by 1-based line number — `raw_lines` is sequential
    // from 1, but index by `number` defensively rather than assuming the
    // offset. Used to compute a precise char span for cell/heading
    // findings that carry a `field_index` but no explicit `char_span`.
    // Built once (O(lines)) rather than re-scanned per finding, so the
    // span computation below stays O(1) per finding.
    let line_index: std::collections::HashMap<u32, &str> = parsed
        .raw_lines
        .iter()
        .map(|rl| (rl.number, rl.text.as_str()))
        .collect();
    let raw_line = |line: Option<u32>| -> Option<&str> { line_index.get(&line?).copied() };
    // Cap per rule on *serialization* only — every rule already ran over
    // every line above; we just clip the tail each group contributes to
    // the JS payload. `total` preserves the true per-rule count and
    // `shown_count` sums what actually crossed the boundary.
    //
    // On top of the per-rule cap, a GLOBAL budget bounds the TOTAL rows that
    // cross the boundary in the interactive (capped) mode — so a file with
    // hundreds of dirty rules can't structured-clone 100k–300k objects (a
    // multi-second freeze / OOM on a weak machine), no matter how many rules
    // fire. It only applies when `max_per_rule` is set; the download path
    // (`max_per_rule = None`) is intentionally uncapped so the full report is
    // available. `finding_count` stays the true total, so the UI's "showing N
    // of M" reflects the cap.
    const MAX_SHOWN_TOTAL: usize = 30_000;
    let global_budget = max_per_rule.map(|_| MAX_SHOWN_TOTAL);
    let mut shown_count = 0usize;
    let findings_out = found
        .into_iter()
        .map(|(rule, items)| {
            let total = items.len();
            let per_rule = max_per_rule.map_or(total, |c| total.min(c));
            let take = match global_budget {
                Some(budget) => per_rule.min(budget.saturating_sub(shown_count)),
                None => per_rule,
            };
            shown_count += take;
            RuleGroup {
                rule,
                total,
                items: items
                    .into_iter()
                    .take(take)
                    .map(|f| {
                        // target omitted when whole-line (the default); the
                        // finer targets serialize as their snake_case token,
                        // matching the engine + TS string union.
                        let target = match f.location.target {
                            findings::Target::Line => None,
                            findings::Target::Heading => Some("heading".to_string()),
                            findings::Target::Cell => Some("cell".to_string()),
                            findings::Target::Group => Some("group".to_string()),
                        };
                        // severity emitted as the lowercase token via the single
                        // producer `Severity::as_str` (gated == the serde rename),
                        // rather than re-spelling error/warning/fyi here. Optional
                        // (TS treats it so) and lets the UI pick the row-band
                        // colour without inferring a default.
                        let severity = Some(f.severity.as_str().to_string());
                        // Span precedence (see `derive_char_span`): a
                        // finding-carried span (Rules 1/6) wins; else derive the
                        // inner-value span from the raw line for field-targeted
                        // findings so cell/heading findings highlight precisely.
                        let char_span = derive_char_span(
                            f.location.char_span,
                            f.location.field_index,
                            raw_line(f.line),
                        );
                        FindingDto {
                            line: f.line,
                            group: f.group,
                            desc: f.desc,
                            target,
                            field_index: f.location.field_index,
                            heading: f.location.heading,
                            data_row: f.location.data_row,
                            char_span,
                            severity,
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    ValidationReport {
        ok: finding_count == 0,
        dict_version: dv.as_str().to_string(),
        resolution: kind.as_str().to_string(),
        finding_count,
        shown_count,
        findings: findings_out,
        error: None,
        // No cert-consume path on this surface (see the field's doc): the engine
        // always ran, so there is no proffered certificate to have rejected.
        revalidate_reason: None,
    }
}

#[cfg(test)]
mod options_tests {
    use super::{
        BuildOptions, CensorOptions, CertifyOptions, DiffOptions, MergeOptions, ValidateOptions,
        WasmOptions, unknown_key,
    };

    /// `KEYS` must name exactly the struct's own serde fields.
    ///
    /// The list is what `reject_unknown_keys` accepts and the struct is what
    /// serde reads; nothing but this test makes them agree. Drift in either
    /// direction is a silent bug — a key present in `KEYS` but absent from the
    /// struct is accepted and then ignored, and one present in the struct but
    /// absent from `KEYS` is refused despite working.
    ///
    /// Sorted on BOTH sides deliberately: `serde_json`'s `preserve_order` IS on
    /// for this crate (via laterite-ags4-validator / -core / -reference, which
    /// it depends on), so `to_value` yields declaration order, not sorted. An
    /// equality check against an assumed-sorted Map would pass or fail on field
    /// ORDER rather than field NAMES.
    fn assert_keys_match<T: WasmOptions + serde::Serialize>() {
        let v = serde_json::to_value(T::default()).expect("options are plain data");
        let mut from_struct: Vec<String> = v
            .as_object()
            .expect("options serialise as an object")
            .keys()
            .cloned()
            .collect();
        let mut declared: Vec<String> = T::KEYS.iter().map(|s| (*s).to_string()).collect();
        from_struct.sort();
        declared.sort();
        assert_eq!(
            declared,
            from_struct,
            "{}: KEYS and the struct's serde fields have drifted",
            T::WHAT
        );
    }

    #[test]
    fn option_keys_match_the_structs() {
        assert_keys_match::<ValidateOptions>();
        assert_keys_match::<CertifyOptions>();
        assert_keys_match::<BuildOptions>();
        assert_keys_match::<MergeOptions>();
        assert_keys_match::<DiffOptions>();
        assert_keys_match::<CensorOptions>();
    }

    /// A misspelled key is REFUSED, by name, with a suggestion.
    ///
    /// This is the whole reason the guard exists. `#[serde(deny_unknown_fields)]`
    /// cannot do it under serde-wasm-bindgen — `ObjectAccess` walks serde's
    /// KNOWN fields and `Reflect`-gets each, so it never sees a key the caller
    /// invented. Under positional arguments this failure mode did not exist:
    /// you cannot typo slot 3.
    #[test]
    fn a_misspelled_option_is_refused_and_the_right_one_suggested() {
        // Exact match: accepted.
        assert!(unknown_key(ValidateOptions::KEYS, "dictVersion").is_none());

        // The s/z spelling split — the realistic typo for a British-spelled API.
        let msg = unknown_key(&["synthesiseMetadata"], "synthesizeMetadata")
            .expect("a z-spelling must not be silently ignored");
        assert!(
            msg.contains("did you mean") && msg.contains("synthesiseMetadata"),
            "suggest the real key: {msg}"
        );

        // Casing.
        let msg = unknown_key(ValidateOptions::KEYS, "DictVersion").expect("casing must not pass");
        assert!(msg.contains("dictVersion"), "suggest the real key: {msg}");

        // Nothing close: list what IS accepted rather than guessing.
        let msg = unknown_key(ValidateOptions::KEYS, "wibble").expect("unknown must not pass");
        assert!(msg.contains("expected one of"), "{msg}");
        assert!(msg.contains("maxPerRule"), "list the real keys: {msg}");
        assert!(!msg.contains("did you mean"), "no false suggestion: {msg}");
    }

    /// Omitted flags take the values every other surface promises.
    ///
    /// `warnings` is the one that matters: the positional parameter it replaces
    /// was a REQUIRED `bool`, so there was no default to preserve — and a plain
    /// `Option<bool>` unwrapping to `false` would have made the browser the
    /// only surface that hides warnings unless asked.
    #[test]
    fn omitted_flags_default_the_way_every_other_surface_does() {
        let o = ValidateOptions::default();
        assert!(o.warnings.unwrap_or(true), "warnings default ON");
        assert!(!o.fyi.unwrap_or(false), "fyi default OFF");
        assert!(!o.dict_replace.unwrap_or(false), "dict_replace default OFF");

        // And an explicit `false` still wins — the default must not be a floor.
        let quiet = ValidateOptions {
            warnings: Some(false),
            ..Default::default()
        };
        assert!(!quiet.warnings.unwrap_or(true));
    }
}

/// The result interfaces published in the `typescript_custom_section` blocks are
/// hand-written strings. wasm-bindgen copies them into the `.d.ts` verbatim and
/// never compares them to the Rust structs they claim to describe — so on their
/// own they are a second hand-maintained mirror, one directory closer to the
/// engine than the app's was. These tests are what make them a contract.
///
/// Host-testable by construction: they read the `const &str` and serialise plain
/// structs, so no `JsValue` and no `wasm-bindgen-test` (which `ci.yml` does not
/// run for this crate) is involved.
#[cfg(test)]
mod ts_result_shape_tests {
    use super::*;

    /// Field names declared by one `export interface` inside a TS source block.
    ///
    /// Deliberately a small hand-rolled scan rather than a TS parser: it only
    /// needs to survive the shape *we* write here — `name?: type;` one per line,
    /// with `/** … */` doc comments and `|` continuation lines between. A field
    /// line is one containing `:` before any `/`, outside a doc comment.
    fn declared_fields(block: &str, interface: &str) -> Vec<String> {
        let start = block
            .find(&format!("export interface {interface} {{"))
            .unwrap_or_else(|| panic!("no `export interface {interface}` in the TS block"));
        let body = &block[start..];
        let end = body
            .find("\n}")
            .unwrap_or_else(|| panic!("unterminated `interface {interface}`"));
        let mut fields = Vec::new();
        let mut in_doc = false;
        for line in body[..end].lines().skip(1) {
            let t = line.trim();
            if in_doc {
                in_doc = !t.contains("*/");
                continue;
            }
            if t.starts_with("/**") {
                // A one-line `/** … */` opens and closes on the same line.
                in_doc = !t.contains("*/");
                continue;
            }
            let Some((name, _)) = t.split_once(':') else {
                continue; // a `|` union continuation line
            };
            let name = name.trim().trim_end_matches('?');
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                fields.push(name.to_string());
            }
        }
        assert!(!fields.is_empty(), "parsed no fields out of {interface}");
        fields
    }

    /// Serialised keys of a value, as serde actually emits them.
    fn serde_keys<T: Serialize>(v: &T) -> Vec<String> {
        serde_json::to_value(v)
            .expect("plain data")
            .as_object()
            .expect("serialises as an object")
            .keys()
            .cloned()
            .collect()
    }

    /// Sorted on both sides: `serde_json`'s `preserve_order` IS on for this crate
    /// (via the validator / core / reference deps), so key order is declaration
    /// order, and an unsorted compare would fail on ORDER rather than NAMES.
    fn assert_same(interface: &str, mut declared: Vec<String>, mut actual: Vec<String>) {
        declared.sort();
        declared.dedup();
        actual.sort();
        assert_eq!(
            declared, actual,
            "TS `{interface}` and its Rust struct have drifted"
        );
    }

    /// Every field must be populated here — a `None` in a `skip_serializing_if`
    /// field would vanish from the serialised keys and the test would then be
    /// asserting that an OPTIONAL field may be undeclared, which is the drift it
    /// exists to catch.
    fn a_finding() -> FindingDto {
        FindingDto {
            line: Some(1),
            group: "LOCA".into(),
            desc: "d".into(),
            target: Some("cell".into()),
            field_index: Some(0),
            heading: Some("LOCA_ID".into()),
            data_row: Some(1),
            char_span: Some([0, 1]),
            severity: Some("warning".into()),
        }
    }

    #[test]
    fn ts_interfaces_match_the_serde_structs() {
        assert_same(
            "FindingDto",
            declared_fields(TS_VALIDATE_RESULT, "FindingDto"),
            serde_keys(&a_finding()),
        );
        assert_same(
            "RuleGroup",
            declared_fields(TS_VALIDATE_RESULT, "RuleGroup"),
            serde_keys(&RuleGroup {
                rule: "AGS Format Rule 1".into(),
                total: 1,
                items: vec![a_finding()],
            }),
        );
        assert_same(
            "ValErr",
            declared_fields(TS_VALIDATE_RESULT, "ValErr"),
            serde_keys(&ValErr {
                kind: "not_ags4".into(),
                message: "m".into(),
            }),
        );
        assert_same(
            "ValidationReport",
            declared_fields(TS_VALIDATE_RESULT, "ValidationReport"),
            serde_keys(&ValidationReport::failure("not_ags4", "m".into())),
        );
        assert_same(
            "EmitFinding",
            declared_fields(TS_BUILD_RESULT, "EmitFinding"),
            serde_keys(&EmitFinding {
                rule: "AGS Format Rule 1".into(),
                line: Some(1),
                group: "LOCA".into(),
                desc: "d".into(),
                severity: Some("warning".into()),
            }),
        );
        assert_same(
            "AppliedFix",
            declared_fields(TS_BUILD_RESULT, "AppliedFix"),
            serde_keys(&AppliedFix {
                kind: fixes::FixKind::StripBom,
                label: "l".into(),
                rule: "AGS Format Rule 1".into(),
                line: Some(1),
                risk: fixes::FixRisk::Safe,
            }),
        );
        assert_same(
            "BuildReport",
            declared_fields(TS_BUILD_RESULT, "BuildReport"),
            serde_keys(&BuildAgs4Report {
                text: String::new(),
                findings: Vec::new(),
                applied: Vec::new(),
                fixes_applied: 0,
            }),
        );
        assert_same(
            "CensorTally",
            declared_fields(TS_CENSOR_RESULT, "CensorTally"),
            serde_keys(&laterite_ags4_censor::Tally::default()),
        );
        assert_same(
            "CensorResult",
            declared_fields(TS_CENSOR_RESULT, "CensorResult"),
            serde_keys(&CensorDto {
                text: String::new(),
                tally: laterite_ags4_censor::Tally::default(),
            }),
        );
        assert_same(
            "GroupMeta",
            declared_fields(TS_GROUP_META, "GroupMeta"),
            serde_keys(&GroupMeta {
                headings: Vec::new(),
                units: Vec::new(),
                types: Vec::new(),
                sql_types: Vec::new(),
            }),
        );

        let cell = || laterite_ags4_diff::CellDelta {
            heading: "LOCA_ID".into(),
            ags_type: "ID".into(),
            a: Some("BH01".into()),
            b: Some("BH02".into()),
        };
        let row = || laterite_ags4_diff::RowDelta {
            kind: "changed",
            key: vec!["BH01".into()],
            line_a: Some(1),
            line_b: Some(1),
            cells: vec![cell()],
        };
        assert_same(
            "CellDelta",
            declared_fields(TS_DIFF_RESULT, "CellDelta"),
            serde_keys(&cell()),
        );
        assert_same(
            "RowDelta",
            declared_fields(TS_DIFF_RESULT, "RowDelta"),
            serde_keys(&row()),
        );
        assert_same(
            "GroupDelta",
            declared_fields(TS_DIFF_RESULT, "GroupDelta"),
            serde_keys(&laterite_ags4_diff::GroupDelta {
                code: "LOCA".into(),
                added: 0,
                removed: 0,
                changed: 1,
                headings_added: Vec::new(),
                headings_removed: Vec::new(),
                keyed: true,
                key_headings: vec!["LOCA_ID".into()],
                rows: vec![row()],
            }),
        );
        assert_same(
            "RevisionDelta",
            declared_fields(TS_DIFF_RESULT, "RevisionDelta"),
            serde_keys(&laterite_ags4_diff::RevisionDelta {
                groups: Vec::new(),
                groups_added: Vec::new(),
                groups_removed: Vec::new(),
                total_added: 0,
                total_removed: 0,
                total_changed: 0,
            }),
        );

        // Straight off the real builder rather than a hand-made value: these
        // three carry `skip_serializing_if` fields (`unit`, `parent`), and a
        // literal with them set to `None` would drop the keys and quietly assert
        // that an optional field may go undeclared.
        let dict = laterite_ags4_validator::dict::dictionary_dto(FALLBACK);
        let group = dict
            .groups
            .iter()
            .find(|g| g.parent.is_some())
            .expect("a non-root group");
        let heading = group
            .headings
            .iter()
            .find(|h| h.unit.is_some())
            .expect("a heading with a unit");
        assert_same(
            "DictHeading",
            declared_fields(TS_DICT_RESULT, "DictHeading"),
            serde_keys(heading),
        );
        assert_same(
            "DictGroup",
            declared_fields(TS_DICT_RESULT, "DictGroup"),
            serde_keys(group),
        );
        assert_same(
            "StandardDict",
            declared_fields(TS_DICT_RESULT, "StandardDict"),
            serde_keys(&dict),
        );

        let edit = || fixes::SpanEdit {
            line: 1,
            start: 0,
            end: 1,
            replacement: "b".into(),
            expected: "a".into(),
        };
        assert_same(
            "SpanEdit",
            declared_fields(TS_FIXES_RESULT, "SpanEdit"),
            serde_keys(&edit()),
        );
        assert_same(
            "Fix",
            declared_fields(TS_FIXES_RESULT, "Fix"),
            serde_keys(&fixes::Fix {
                kind: fixes::FixKind::StripBom,
                label: "l".into(),
                rule: "AGS Format Rule 1".into(),
                line: Some(1),
                risk: fixes::FixRisk::Safe,
                edits: vec![edit()],
            }),
        );
    }

    /// The parser must be able to fail. Without this, a `declared_fields` that
    /// silently returned the wrong thing would make every assertion above pass
    /// against itself.
    #[test]
    fn the_interface_parser_can_see_a_missing_field() {
        let mut fields = declared_fields(TS_VALIDATE_RESULT, "ValErr");
        assert_eq!(fields, vec!["kind", "message"]);
        fields.pop();
        assert_ne!(fields, serde_keys(&a_finding()));
    }

    /// `AppliedFix.kind` / `.risk` are unions over enums owned by
    /// **laterite-ags4-validator**. A new `FixKind` variant there would otherwise
    /// silently fall outside the union we publish here, and the `.d.ts` would lie
    /// about a value consumers can actually receive.
    ///
    /// Rather than keep a second hand-written variant list to compare against,
    /// this asks serde for the authoritative one: deserialising a bogus token
    /// fails with `unknown variant ..., expected one of ...`, which enumerates
    /// every variant the enum really has.
    #[test]
    fn fix_unions_match_the_validators_enums() {
        fn variants<'de, T: serde::Deserialize<'de>>() -> Vec<String> {
            // `.err()` rather than `expect_err`: the latter needs `T: Debug`,
            // and these enums are only required to be Deserialize.
            let err = serde_json::from_str::<T>("\"__not_a_variant__\"")
                .err()
                .expect("a bogus token must not deserialise");
            let msg = err.to_string();
            // Read the backticked tokens rather than a fixed phrase: serde says
            // "expected one of `a`, `b`, `c`" for three or more variants but
            // "expected `safe` or `risky`" for two, and FixRisk has two. The
            // first backticked token is always the bogus name we passed in.
            let all: Vec<String> = msg
                .split('`')
                .skip(1)
                .step_by(2)
                .map(str::to_string)
                .collect();
            let (first, rest) = all
                .split_first()
                .unwrap_or_else(|| panic!("serde changed its unknown-variant message: {msg}"));
            assert_eq!(
                first, "__not_a_variant__",
                "unexpected message shape: {msg}"
            );
            assert!(!rest.is_empty(), "no variants listed in: {msg}");
            rest.to_vec()
        }

        fn union_members(block: &str, field: &str) -> Vec<String> {
            let at = block.find(field).expect("field is declared");
            let body = &block[at..];
            let end = body.find(';').expect("field declaration ends in `;`");
            body[..end]
                .split('"')
                .skip(1)
                .step_by(2)
                .map(str::to_string)
                .collect()
        }

        // Both blocks: `AppliedFix` (what `build_ags4` reports it did) and `Fix`
        // (what `compute_fixes` offers) publish the SAME two unions from the same
        // two enums, in two separately-written strings. Checking only one leaves
        // the other free to drift.
        for (block, what) in [(TS_BUILD_RESULT, "AppliedFix"), (TS_FIXES_RESULT, "Fix")] {
            for (field, mut actual) in [
                ("kind:", variants::<fixes::FixKind>()),
                ("risk:", variants::<fixes::FixRisk>()),
            ] {
                let mut declared = union_members(block, field);
                declared.sort();
                actual.sort();
                assert_eq!(
                    declared, actual,
                    "TS `{what}.{field}` union and the validator's enum have drifted"
                );
            }
        }
    }
}

#[cfg(test)]
mod dict_overlay_tests {
    use super::{ValidateOptions, run};

    // The #568 Phase-3 end-to-end fixtures, shared with the validator's `custom_dict.rs`
    // so the browser is proven against the same bytes: a bespoke `XTRA` group hung off
    // the standard `SAMP`, and a delivery that uses it.
    const DELIVERY: &[u8] = include_bytes!(
        "../../laterite-ags4-validator/tests/fixtures/custom_dict/delivery_with_xtra.ags"
    );
    const DICT_JSON: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/custom_dict/xtra.dict.json");

    fn xtra_findings(report: &super::ValidationReport) -> usize {
        report
            .findings
            .iter()
            .flat_map(|g| &g.items)
            .filter(|f| f.group == "XTRA")
            .count()
    }

    #[test]
    fn dict_bytes_make_a_bespoke_group_valid_in_the_browser() {
        // Acceptance #3 (browser leg) + #5: the same bytes the CLI/py/node smokes use.
        // Warnings + FYI on so the unknown-group findings are actually surfaced.
        let err_msg = |r: &super::ValidationReport| {
            r.error
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_default()
        };
        let opts = |dictionary: Option<&[u8]>, dict_replace: bool| ValidateOptions {
            warnings: Some(true),
            fyi: Some(true),
            dictionary: dictionary.map(<[u8]>::to_vec),
            dict_replace: Some(dict_replace),
            ..Default::default()
        };
        let without = run(DELIVERY, &opts(None, false));
        assert!(
            without.error.is_none(),
            "delivery parses: {}",
            err_msg(&without)
        );
        assert!(
            xtra_findings(&without) > 0,
            "the bundled dictionary must flag the unknown XTRA group"
        );

        // With the custom dictionary supplied as bytes (the browser's only form),
        // XTRA is a first-class group and draws no findings.
        let with = run(DELIVERY, &opts(Some(DICT_JSON), false));
        assert!(
            with.error.is_none(),
            "delivery parses with dict: {}",
            err_msg(&with)
        );
        assert_eq!(
            xtra_findings(&with),
            0,
            "the overlay makes XTRA recognised ({} residual XTRA findings)",
            xtra_findings(&with)
        );

        // Acceptance #5: `revalidate_reason` is a present field on the report — always
        // `None` here because this surface has no cert-consume door (see the field's doc).
        assert!(with.revalidate_reason.is_none());
    }

    #[test]
    fn dict_replace_with_a_forced_base_is_a_bad_dict_error() {
        // The one contradiction: a full replacement cannot also force a base edition.
        let report = run(
            DELIVERY,
            &ValidateOptions {
                dict_version: Some("4.2".into()),
                warnings: Some(true),
                fyi: Some(true),
                dictionary: Some(DICT_JSON.to_vec()),
                dict_replace: Some(true),
                ..Default::default()
            },
        );
        let err = report
            .error
            .expect("contradiction is surfaced, not silently ignored");
        assert_eq!(err.kind, "bad_dict");
    }
}

// ---------------------------------------------------------------------
// Apply-Fixes: compute_fixes() / apply_fixes() — a separate surface from
// validate() so the byte-faithful finding JSON is never perturbed. Both
// reuse the validate() skeleton (resolve_encoding + parse_bytes + the
// dict resolution + run_all) so a fix is computed against exactly the
// same findings the report shows.
// ---------------------------------------------------------------------

/// Compute the safe fixes for AGS4 bytes in the browser.
///
/// Parses + runs the full rule engine (FYI on, so e.g. the Rule 1 BOM
/// path is seen), then `laterite_ags4_validator::fixes::compute_fixes`. Returns a
/// JSON-compatible `Fix[]` (empty array on a parse error — there's
/// nothing to fix in an un-parseable file, and `validate` already
/// surfaces the error to the UI).
#[wasm_bindgen]
pub fn compute_fixes(
    data: &[u8],
    dict_version: Option<String>,
    encoding_label: Option<String>,
) -> FixesJs {
    console_error_panic_hook::set_once();
    let fixes = compute_fixes_core(data, dict_version.as_deref(), encoding_label.as_deref());
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    fixes
        .serialize(&serializer)
        .expect("Fixes is plain data and always serialises")
        .unchecked_into()
}

/// The host-testable core of [`compute_fixes`].
///
/// Every failure yields an EMPTY fix list rather than an error, and that is the
/// whole design: this door has no error channel, so the alternative to "no
/// fixes" is fixes computed against the wrong decoding or the wrong dictionary —
/// offering the user a button that silently corrupts their file. Four distinct
/// ways in (bad edition, bad encoding, unparseable bytes, a failed check) all
/// had to collapse to the same empty answer, and none of them could be reached
/// from a test while they lived behind a `FixesJs` return type.
fn compute_fixes_core(
    data: &[u8],
    dict_version: Option<&str>,
    encoding_label: Option<&str>,
) -> Vec<laterite_ags4_validator::fixes::Fix> {
    let Ok(dict_over) = resolve_dict_override(dict_version) else {
        return Vec::new();
    };
    // An unknown label yields no fixes rather than fixes computed against the
    // wrong decoding — silently "fixing" text we mis-decoded is the worst
    // option on the table.
    let Ok(encoding) = resolve_encoding(encoding_label) else {
        return Vec::new();
    };
    let Ok(parsed) = parse_bytes(data, encoding) else {
        return Vec::new();
    };
    let opts = CheckOptions {
        dict_version: dict_over,
        include_fyi: true,
        encoding,
        ..CheckOptions::default()
    };
    // Through the door, so the fixes offered in the browser are computed against the
    // same dictionary `lat fix` would use on the same bytes (the O-42 guard included).
    let Ok((found, _dv, _kind)) = check_parsed_with_dict(&parsed, &opts, &WorldScope::None) else {
        return Vec::new();
    };
    laterite_ags4_validator::fixes::compute_fixes(&parsed, &found)
}

/// Apply a user-selected subset of fixes to AGS4 bytes, returning the new
/// file as **UTF-8 bytes** (a JS `Uint8Array`). The input is decoded with
/// `encoding_label` (capturing whether it carried a BOM), the fixes are
/// applied by the shared engine, and the result is always re-encoded as
/// UTF-8 — so applying to a cp1252 file also normalises its encoding
/// (Rule-1-friendly, and the caller resets its encoding select to utf-8).
///
/// Throws on an unknown `encoding_label`. This used to be infallible and fell back
/// to UTF-8 — meaning it would REWRITE a file it had just mis-decoded, which is the
/// one place a silent fallback does permanent damage. The worker already turns a
/// throw into an `ok: false` reply, and the UI's encoding select is a closed union,
/// so the browser cannot reach this path; a direct wasm caller can, and should be
/// told rather than handed a corrupted file.
#[wasm_bindgen]
pub fn apply_fixes(
    data: &[u8],
    encoding_label: Option<String>,
    fixes_json: JsValue,
) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    // A fix list that will not deserialise becomes an EMPTY list, so the call
    // returns the file unchanged rather than throwing. Only the decode is
    // JS-shaped; the work is in the core below.
    let fixes: Vec<laterite_ags4_validator::fixes::Fix> =
        serde_wasm_bindgen::from_value(fixes_json).unwrap_or_default();
    apply_fixes_core(data, encoding_label.as_deref(), &fixes).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`apply_fixes`]: decode with the caller's
/// encoding, apply, re-encode as UTF-8.
///
/// The BOM capture is the load-bearing part — `apply_fixes` honours "keep the
/// BOM" when `StripBom` was not among the selected fixes, so this has to read
/// the raw bytes rather than the decoded text (`encoding_rs` eats the mark).
fn apply_fixes_core(
    data: &[u8],
    encoding_label: Option<&str>,
    fixes: &[laterite_ags4_validator::fixes::Fix],
) -> Result<Vec<u8>, String> {
    let encoding = resolve_encoding(encoding_label)?;
    // Decode to text + capture the BOM the same way the engine does, so
    // apply_fixes can honour "keep the BOM" when StripBom isn't selected.
    let has_bom = data.starts_with(&[0xEF, 0xBB, 0xBF]);
    let (text, _enc, _had) = encoding.decode(data);
    let out = laterite_ags4_validator::fixes::apply_fixes(&text, has_bom, fixes);
    Ok(out.into_bytes())
}

// ---------------------------------------------------------------------
// AGS4 ↔ XLSX (#359). The FS-free laterite-ags4-excel cores (`ags4_bytes_to_xlsx` /
// `xlsx_bytes_to_ags4`) drive the browser Excel surface: the Tools pane hands
// us bytes and gets bytes + warnings back, no filesystem. calamine reads and
// rust_xlsxwriter writes — both pure-Rust and wasm-clean.
// ---------------------------------------------------------------------

/// The result of an Excel conversion: the output `bytes` (a JS `Uint8Array` —
/// the `.xlsx` or `.ags` file), plus the `warnings` and counts the UI surfaces
/// (dropped non-Rule-19 columns, skipped sheets, …).
#[wasm_bindgen]
pub struct ExcelResult {
    bytes: Vec<u8>,
    warnings: Vec<String>,
    sheets: usize,
    rows: usize,
}

#[wasm_bindgen]
impl ExcelResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn sheets(&self) -> usize {
        self.sheets
    }
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.rows
    }
}

/// AGS4 bytes → an `.xlsx` workbook (one sheet per group, python-ags4's
/// layout). `JsError` if the input carries no valid AGS4 groups.
#[wasm_bindgen]
pub fn ags4_to_xlsx(
    data: &[u8],
    recover_duplicate_headings: Option<bool>,
) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    ags4_to_xlsx_core(data, recover_duplicate_headings.unwrap_or(false))
        .map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`ags4_to_xlsx`].
fn ags4_to_xlsx_core(data: &[u8], recover_duplicate_headings: bool) -> Result<ExcelResult, String> {
    use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ReadOptions};
    // Duplicate headings are fatal by default here as on every read surface; the
    // browser caller opts into the suffixed recovery read.
    let opts = ReadOptions {
        duplicate_headings: if recover_duplicate_headings {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
    };
    let (bytes, stats) = laterite_ags4_excel::ags4_bytes_to_xlsx_with(data, None, opts)
        .map_err(|e| e.to_string())?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
}

/// An `.xlsx` workbook's bytes → AGS4 bytes. Each sheet with a `HEADING` column
/// becomes a group; non-Rule-19 columns and non-`UNIT`/`TYPE`/`DATA` rows are
/// dropped (surfaced in `warnings`). `format_numeric` re-pads DATA cells to
/// their column's TYPE (mirrors python-ags4's `convert_to_text`). `JsError` if
/// no sheet yields a valid group.
#[wasm_bindgen]
pub fn xlsx_to_ags4(data: &[u8], format_numeric: bool) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    xlsx_to_ags4_core(data, format_numeric).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`xlsx_to_ags4`].
fn xlsx_to_ags4_core(data: &[u8], format_numeric: bool) -> Result<ExcelResult, String> {
    let (bytes, stats) =
        laterite_ags4_excel::xlsx_bytes_to_ags4(data, format_numeric).map_err(|e| e.to_string())?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
}

// ---------------------------------------------------------------------
// Phase 2: read() -> typed Arrow IPC for the DuckDB-wasm data explorer.
//
// AGS4 isn't a format DuckDB reads natively. We parse it in Rust, build
// ONE correctly-typed Arrow RecordBatch per group, and hand JS the IPC
// bytes; DuckDB-wasm's `insertArrowFromIPCStream` ingests it as the final
// typed table — no per-cell JS objects, no staging table, no TRY_CAST.
//
// Typing uses the SAME laterite_ags4_types::{canonical_type, parse_value,
// parse_datetime} the native DuckDB conversion uses, off the file's own
// TYPE row (convert.rs does the same), so the explorer casts a file
// IDENTICALLY to the native DuckDB conversion — parity by construction.
// ---------------------------------------------------------------------

/// A parsed AGS4 file held in wasm memory as the lightweight string
/// `ParsedFile`. Each group's typed `RecordBatch` is built lazily on the
/// `arrow_ipc(code)` call and dropped after the bytes are returned, so
/// peak residency is one batch — not every group at once.
#[wasm_bindgen]
pub struct ParsedDataset {
    parsed: ParsedFile,
}

ts_section! {
    TS_GROUP_META,
    TS_GROUP_META_SECTION,
    r#"
/** Per-group schema: four PARALLEL arrays, one entry per heading, so
 *  `headings[i]` / `units[i]` / `types[i]` / `sql_types[i]` describe the same
 *  column. `meta()` returns `null` for a code the file does not contain. */
export interface GroupMeta {
  headings: string[];
  units: string[];
  /** AGS TYPE codes from the file's TYPE row (`"2DP"`, `"DT"`, `"ID"`, …). */
  types: string[];
  /** The DuckDB column type each heading lands as (`"DOUBLE"`, `"BIGINT"`,
   *  `"TIMESTAMP"`, `"VARCHAR"`, …) — what the table will report. */
  sql_types: string[];
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "GroupMeta | null")]
    pub type GroupMetaJs;
}

/// Per-group schema for the UI: parallel arrays (one entry per heading).
#[derive(Serialize)]
struct GroupMeta {
    headings: Vec<String>,
    units: Vec<String>,
    /// AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID").
    types: Vec<String>,
    /// The DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
    /// "TIMESTAMP", "VARCHAR", …) — what the table will report.
    sql_types: Vec<String>,
}

#[wasm_bindgen]
impl ParsedDataset {
    /// Group codes in file order (the order to load tables in).
    pub fn group_codes(&self) -> Vec<String> {
        self.parsed.group_order.clone()
    }

    /// `{headings, units, types, sql_types}` for one group, or `null` if
    /// the code isn't present.
    pub fn meta(&self, code: &str) -> GroupMetaJs {
        let Some(meta) = self.meta_core(code) else {
            return JsValue::NULL.unchecked_into();
        };
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        meta.serialize(&serializer)
            .unwrap_or(JsValue::NULL)
            .unchecked_into()
    }

    /// One group's rows as an Arrow IPC **stream** (Uint8Array), columns
    /// already correctly typed. Built lazily here and dropped on return.
    ///
    /// `keys` (default `false`) prepends the two content-addressed key columns
    /// `_id`/`_parent_id` — the SAME UUIDv8s the wheel / Node / DuckDB
    /// extension produce (via the one shared keychain). Pass `true` when feeding
    /// duckdb-wasm so cross-group joins (`s._parent_id = l._id`) resolve; leave
    /// it off (the default) for a plain typed frame. A custom/passthrough group
    /// carries no keys, so `keys` is a no-op for it. (#303)
    ///
    /// `content_hash` (default `false`) appends a trailing `_content_hash`
    /// value fingerprint (SHA-256 over the typed, blank-normalised heading
    /// values) — the SAME hash Node/Python produce via the one shared
    /// keychain. Unlike `keys` this needs no registry entry, so a
    /// custom/passthrough group still gets a usable `_content_hash` even
    /// without an `_id`. (#448)
    pub fn arrow_ipc(
        &self,
        code: &str,
        keys: Option<bool>,
        content_hash: Option<bool>,
    ) -> Result<Vec<u8>, JsError> {
        self.arrow_ipc_core(code, keys.unwrap_or(false), content_hash.unwrap_or(false))
            .map_err(|m| JsError::new(&m))
    }
}

/// The host-testable half of [`ParsedDataset`]. Same methods, plain Rust types —
/// the `#[wasm_bindgen]` block above is now only defaults and error marshalling.
impl ParsedDataset {
    /// The core of [`ParsedDataset::meta`]: `None` for a code the file does not
    /// contain, which the caller renders as JS `null`.
    ///
    /// The parallel-array contract lives here — `headings[i]` / `units[i]` /
    /// `types[i]` / `sql_types[i]` describe the same column, and a file whose
    /// UNIT or TYPE row is SHORTER than its HEADING row (common, and legal
    /// enough to reach the explorer) must still produce four arrays of equal
    /// length. That padding — `""` for a missing unit, `"X"` for a missing type
    /// — is the reason this is not a one-liner, and it was unreachable from a
    /// test while it sat behind a `GroupMetaJs` return.
    fn meta_core(&self, code: &str) -> Option<GroupMeta> {
        let group = self.parsed.groups.get(code)?;
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
        })
    }

    /// The core of [`ParsedDataset::arrow_ipc`], with the two flags already
    /// defaulted.
    fn arrow_ipc_core(
        &self,
        code: &str,
        keys: bool,
        content_hash: bool,
    ) -> Result<Vec<u8>, String> {
        let group = self
            .parsed
            .groups
            .get(code)
            .ok_or_else(|| format!("group {code:?} not in dataset"))?;

        // Typed columns + IPC framing both come from laterite-ags4-types now
        // (`ipc::build_group_ipc_synth` = the shared `arrow_cols` cast + StreamWriter,
        // `_id`/`_parent_id` col 0/1, `_content_hash` trailing) — the SAME
        // composition the napi host frames, so the browser, Node and Python type
        // a file byte-identically by construction. Framed here only for
        // duckdb-wasm.
        let reg = laterite_ags4_core::registry::registry();
        let ids = (keys && reg.get(code).is_some()).then(|| {
            laterite_ags4_core::keychain::group_row_ids(
                reg,
                code,
                &group.headings,
                group.rows.len(),
                |col, row| group.cell(col, row),
            )
        });
        let hashes = if content_hash {
            Some(laterite_ags4_core::keychain::group_content_hashes(
                code,
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
        .map_err(|e| format!("arrow ipc for {code}: {e}"))?;
        Ok(buf)
    }
}

/// Parse AGS4 bytes into a typed dataset for the explorer. Validation is
/// a separate concern (`validate`); this is permissive — it builds typed
/// columns for whatever parsed, so the explorer works even on a file with
/// findings. Only an unparseable-as-AGS4 input returns `Err`.
#[wasm_bindgen]
pub fn read(data: &[u8], encoding_label: Option<String>) -> Result<ParsedDataset, JsError> {
    console_error_panic_hook::set_once();
    read_core(data, encoding_label.as_deref()).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`read`]. The `ParseError` → `ValidatorError`
/// bridge is the point: an unparseable file must report the SAME text here as
/// it does from `validate`, and only this conversion makes that true.
fn read_core(data: &[u8], encoding_label: Option<&str>) -> Result<ParsedDataset, String> {
    let encoding = resolve_encoding(encoding_label)?;
    let parsed = parse_bytes(data, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    Ok(ParsedDataset { parsed })
}

// The `diff` result. The shapes are `laterite-ags4-diff`'s, not this crate's —
// but publishing them here is still right: this is the door a JS caller comes
// through, and the alternative (what the web app did) is every consumer keeping
// its own copy. `ts_interfaces_match_the_serde_structs` binds these to the leaf's
// real structs, so "owned elsewhere" does not mean "unchecked here".
ts_section! {
    TS_DIFF_RESULT,
    TS_DIFF_RESULT_SECTION,
    r#"
/** One changed cell of a row matched on both sides. */
export interface CellDelta {
  heading: string;
  /** The AGS TYPE code the two cells were compared AS — a numeric compare is
   *  value-wise, so `"1.50"` and `"1.5"` are equal under `2DP` but differ as
   *  raw text. */
  type: string;
  /** Raw value on each side; `null` when that side's row is shorter than the
   *  heading list. */
  a: string | null;
  b: string | null;
}

/** One row's verdict. */
export interface RowDelta {
  kind: "added" | "removed" | "changed";
  /** The KEY values identifying the row — or the whole-row tuple when the
   *  group has no dictionary KEY headings (see `GroupDelta.keyed`). */
  key: string[];
  line_a: number | null;
  line_b: number | null;
  /** Populated only for `kind === "changed"`. */
  cells: CellDelta[];
}

/** One group's change summary. */
export interface GroupDelta {
  code: string;
  /** TRUE totals, independent of any `maxRowsPerGroup` cap — so `rows.length`
   *  may be smaller than `added + removed + changed`. */
  added: number;
  removed: number;
  changed: number;
  /** Structural change: headings present on only one side. */
  headings_added: string[];
  headings_removed: string[];
  /** `false` ⇒ rows were matched on the whole-row tuple because the dictionary
   *  gave this group no KEY headings. Matching is weaker; a row that changed
   *  in every cell reads as one removal plus one addition. */
  keyed: boolean;
  key_headings: string[];
  rows: RowDelta[];
}

/** The `diff` result: a KEY-aware, type-aware comparison of two files. */
export interface RevisionDelta {
  /** Groups with at least one row or heading change, in `b`'s file order,
   *  then the groups only `a` had. */
  groups: GroupDelta[];
  groups_added: string[];
  groups_removed: string[];
  total_added: number;
  total_removed: number;
  total_changed: number;
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "RevisionDelta")]
    pub type RevisionDeltaJs;
}

/// `diff`'s named options. `encoding`, not `encodingLabel` — see [`MergeOptions`].
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct DiffOptions {
    encoding: Option<String>,
    max_rows_per_group: Option<u32>,
}

impl WasmOptions for DiffOptions {
    const KEYS: &'static [&'static str] = &["encoding", "maxRowsPerGroup"];
    const WHAT: &'static str = "diff options";
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DIFF_OPTIONS: &'static str = r#"
/** Named options for `diff`. */
export interface DiffOptions {
  /** `"utf-8"` (default) or `"windows-1252"`, applied to BOTH inputs. */
  encoding?: "utf-8" | "windows-1252";
  /** Cap how many per-row deltas each group SERIALISES. The
   *  `added`/`removed`/`changed` counts stay true totals either way, so a cap
   *  bounds the payload without lying about the size of the change. Omit for
   *  everything. */
  maxRowsPerGroup?: number;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "DiffOptions")]
    pub type DiffOptionsJs;
}

/// Compare two AGS4 files.
///
/// * `opts` — a [`DiffOptions`] object; every field optional, so `diff(a, b)`
///   is a complete call. An unrecognised key is refused by name.
#[wasm_bindgen]
pub fn diff(a: &[u8], b: &[u8], opts: Option<DiffOptionsJs>) -> Result<RevisionDeltaJs, JsError> {
    console_error_panic_hook::set_once();
    let o: DiffOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let delta = diff_core(a, b, &o).map_err(|m| JsError::new(&m))?;
    to_js(&delta)
}

/// The host-testable core of [`diff`]: decode both files, resolve the edition,
/// and run the shared comparison.
///
/// Which edition is a real decision and it is made here — KEY headings come
/// from the dictionary, and picking the wrong one silently changes what counts
/// as "the same row". It reads `b`'s `TRAN_AGS` (the newer file) and falls back
/// to the standard, and neither half of that could be reached from a test while
/// it sat behind `RevisionDeltaJs`.
fn diff_core(
    a: &[u8],
    b: &[u8],
    o: &DiffOptions,
) -> Result<laterite_ags4_diff::RevisionDelta, String> {
    let encoding = resolve_encoding(o.encoding.as_deref())?;
    let pa = parse_bytes(a, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    let pb = parse_bytes(b, encoding).map_err(|e| ValidatorError::from(e).to_string())?;

    // KEY headings come from the dictionary; pick the edition from the
    // revision's TRAN_AGS (the "new" file), falling back to the standard.
    let dv = resolve_dict_version(None, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dict = Dictionary::bundled(dv);
    let cap = o.max_rows_per_group.map(|c| c as usize);

    // The KEY-aware/type-aware comparison itself lives in the shared
    // laterite-ags4-diff leaf (so PyO3 + the CLI reuse it); this only parses,
    // resolves the dictionary, and hands the result back.
    Ok(laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, cap))
}

/// The result of a merge: the reconciled `bytes` (a JS `Uint8Array` — the merged
/// `.ags` file), plus `warnings_json` and `revisions_json` (the audit arrays the
/// Tools UI parses — the same shape PyO3 / Node return).
#[wasm_bindgen]
pub struct MergeResult {
    bytes: Vec<u8>,
    warnings_json: String,
    revisions_json: String,
}

#[wasm_bindgen]
impl MergeResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings_json(&self) -> String {
        self.warnings_json.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn revisions_json(&self) -> String {
        self.revisions_json.clone()
    }
}

/// `merge`'s named options.
///
/// `encoding` rather than `encodingLabel`: every other surface calls this
/// concept `encoding`, and the browser was the only one carrying the `_label`
/// suffix. See the README's note on which exports still take it positionally —
/// the ones not yet migrated keep the old name until they are, so the split is
/// a recorded state rather than an accident.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct MergeOptions {
    /// Force the edition instead of reading it from `b`'s `TRAN_AGS`.
    ///
    /// Added because the cross-surface parity gate found it missing: Python's
    /// `merge` and `lat merge` both take it, and this surface hard-coded `None`
    /// — so a browser user merging files whose `TRAN_AGS` was wrong or absent
    /// had no way to say so, while every other door did.
    dict_version: Option<String>,
    encoding: Option<String>,
    on_type_clash: Option<String>,
    tran: Option<TranInput>,
}

impl WasmOptions for MergeOptions {
    const KEYS: &'static [&'static str] = &["dictVersion", "encoding", "onTypeClash", "tran"];
    const WHAT: &'static str = "merge options";
}

#[wasm_bindgen(typescript_custom_section)]
const TS_MERGE_OPTIONS: &'static str = r#"
/** Named options for `merge`. */
export interface MergeOptions {
  /** Force the edition rather than reading it from `b`'s `TRAN_AGS`. */
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
  /** `"utf-8"` (default) or `"windows-1252"`, applied to BOTH inputs. */
  encoding?: "utf-8" | "windows-1252";
  /** What to do when two files declare a different AGS TYPE for one heading.
   *  `"error"` (default) refuses; `"widen"` falls back to `X`, keeping raw
   *  values but discarding the type; `"promote"` keeps the greatest `nDP`
   *  precision, zero-padding the coarser values. */
  onTypeClash?: "error" | "widen" | "promote";
  /** The transmission the MERGED file represents — it genuinely is a new one.
   *  Omit it and `TRAN` is reconciled like any other group (newest wins), with
   *  a warning noting no merge-transmission stamp was supplied.
   *
   *  `remarks` is APPENDED to merge's own provenance note ("Merged from N
   *  deliveries: …") rather than replacing it: both are true of the result. */
  tran?: TranStamp;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "MergeOptions")]
    pub type MergeOptionsJs;
}

/// Merge two AGS4 deliveries of one project into one file (`a` then `b` — `b`
/// wins a KEY conflict). Rows are matched by their dictionary KEY headings.
///
/// * `opts` — a [`MergeOptions`] object; every field optional, so
///   `merge(a, b)` is a complete call. An unrecognised key is refused by name.
///
/// A heading the two files typed differently is a `JsError` unless
/// `onTypeClash` settles it. A complete `tran` stamps a synthesised merge-TRAN;
/// omit it and TRAN is reconciled like any other group, with a warning. The
/// edition is `b`'s `TRAN_AGS`, falling back to the standard.
///
/// The returned [`MergeResult`] is a wasm-owned handle: read its getters and
/// call `.free()`. The web worker leaked one per merge until this was written
/// down here.
#[wasm_bindgen]
pub fn merge(a: &[u8], b: &[u8], opts: Option<MergeOptionsJs>) -> Result<MergeResult, JsError> {
    console_error_panic_hook::set_once();
    let o: MergeOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    merge_core(a, b, o).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`merge`].
///
/// Four separate decisions live in here, all of which change the output file
/// and none of which `cargo test` could reach behind a `MergeOptionsJs`: the
/// TRAN completeness rule, the encoding, the edition (an explicit
/// `dictVersion` overriding `b`'s `TRAN_AGS`), and the type-clash vocabulary —
/// which is deliberately parsed by the merge crate's own `FromStr` so the
/// browser cannot accept a token the CLI rejects, or word the rejection
/// differently.
fn merge_core(a: &[u8], b: &[u8], o: MergeOptions) -> Result<MergeResult, String> {
    use laterite_ags4_merge::{MergeOpts, TypeClashMode, merge_parsed};

    let tran = o.tran.map(TranInput::fold).transpose()?.flatten();
    let encoding = resolve_encoding(o.encoding.as_deref())?;
    let pa = parse_bytes(a, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    let pb = parse_bytes(b, encoding).map_err(|e| ValidatorError::from(e).to_string())?;

    // Edition from the newest file (b)'s TRAN_AGS, falling back to the standard.
    let over = resolve_dict_override(o.dict_version.as_deref())?;
    let dv = resolve_dict_version(over, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);

    // One vocabulary for every surface: accepted tokens + rejection message come
    // from the merge crate's FromStr, so the browser cannot drift from the CLI.
    let clash: TypeClashMode = o.on_type_clash.as_deref().unwrap_or("error").parse()?;

    let opts = MergeOpts {
        on_type_clash: clash,
        edition: dv,
        tran,
        ..Default::default()
    };

    let res = merge_parsed(&[pa, pb], &opts).map_err(|e| e.to_string())?;
    let warnings: Vec<_> = res
        .warnings
        .iter()
        .map(|w| {
            serde_json::json!({
                "kind": w.kind, "group": w.group,
                "heading": w.heading, "message": w.message,
            })
        })
        .collect();
    let revisions: Vec<_> = res
        .revisions
        .iter()
        .map(|r| {
            serde_json::json!({
                "group": r.group, "key": r.key,
                "changed": r.changed, "winnerFile": r.winner_file,
            })
        })
        .collect();
    Ok(MergeResult {
        bytes: res.bytes,
        warnings_json: serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".into()),
        revisions_json: serde_json::to_string(&revisions).unwrap_or_else(|_| "[]".into()),
    })
}

// ---------------------------------------------------------------------
// dictionary() -> the bundled STANDARD dictionary for an edition.
//
// The Tools reference (Dictionary browser / Template generator) used to fetch
// a static scaffolded dictionary — a single fixed edition where ~91% of
// headings had EMPTY descriptions.
// This exposes the validator's real per-edition standard dictionary instead:
// canonical names + descriptions + units + types + status, selectable across
// 4.0.3 … 4.2 (the same data the engine validates against).
// ---------------------------------------------------------------------

/// The crate version — the same answer Node's `version()` gives, from the same
/// `CARGO_PKG_VERSION`.
///
/// It exists because `ags4-compliance`'s wasm runner HARD-CODED `version: "0.5.1"`
/// (the dev satellite's tools/compliance/emit_js.mjs) — a literal true when it
/// was written, that the workspace moved past to 0.7.0 while nothing compared it
/// back. The harness then printed "wasm v0.5.1" next to three 0.7.0 surfaces and
/// called the comparison 4-laterite identity. The build was current; only the
/// report lied. Node had this all along and asked the module; wasm had nothing to
/// ask, which is why someone wrote a constant instead. (#556)
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The version of the validation engine underneath — a hand-bumped semver.
///
/// Distinct from [`version`] since the tiers split (#202): this package carries
/// the product number, the engine carries its own. Useful for humans, useless as
/// an identity — edit a rule without bumping the crate and this is unchanged.
#[wasm_bindgen]
pub fn engine_version() -> String {
    laterite_ags4_validator::VERSION.to_string()
}

/// The identity of the engine that produces verdicts — a build-time digest over
/// every rule source, the dictionary, and the rules catalogue.
///
/// The same hazard this module's [`version`] was written for, one level down. A
/// report that prints matching version numbers across surfaces has shown they
/// shipped together, not that they agree on the rules; only this can show that.
/// Two surfaces reporting the same fingerprint ARE running the same engine.
#[wasm_bindgen]
pub fn engine_fingerprint() -> String {
    laterite_ags4_validator::ENGINE_FINGERPRINT.to_string()
}

// The `dictionary` result — `laterite-ags4-reference`'s `DictionaryDto`, which
// PyO3 and Node also render, from the one shared builder. Bound to that struct
// by `ts_interfaces_match_the_serde_structs`.
ts_section! {
    TS_DICT_RESULT,
    TS_DICT_RESULT_SECTION,
    r#"
/** One heading in the standard dictionary. */
export interface DictHeading {
  name: string;
  /** `KEY` | `REQUIRED` | `OTHER` — whether the AGS standard requires it. */
  status: string;
  /** AGS TYPE code (`ID`, `X`, `2DP`, `DT`, …). */
  type: string;
  /** Absent when the heading is unitless — not `""`. */
  unit?: string;
  description: string;
}

/** One group in the standard dictionary. */
export interface DictGroup {
  code: string;
  /** The group's standard description — its "contents". */
  contents: string;
  /** Absent for a root group (`PROJ`). */
  parent?: string;
  headings: DictHeading[];
}

/** One bundled edition of the AGS4 standard dictionary: groups sorted by code,
 *  each group's headings in canonical dictionary order. */
export interface StandardDict {
  /** The edition this is for (`"4.1.1"`, …). */
  ags_edition: string;
  groups: DictGroup[];
}
"#
}

// The `compute_fixes` result — `laterite-ags4-validator`'s `fixes::Fix`. The
// `kind`/`risk` unions are the same two enums `AppliedFix` carries, and are
// checked against the enums themselves by `fix_unions_match_the_validators_enums`
// rather than trusted as prose.
ts_section! {
    TS_FIXES_RESULT,
    TS_FIXES_RESULT_SECTION,
    r#"
/** One in-line text edit: replace the half-open char range `[start, end)` on a
 *  1-based line. */
export interface SpanEdit {
  line: number;
  start: number;
  end: number;
  replacement: string;
  /** What the span should currently hold. The engine SKIPS the edit if it does
   *  not match, so a stale fix computed against older bytes cannot corrupt the
   *  file — it simply does nothing. */
  expected: string;
}

/** One fix the engine can apply. */
export interface Fix {
  kind: "normalize_crlf" | "strip_bom" | "strip_embedded_cr"
      | "rename_duplicate_heading" | "insert_tran_dlim" | "insert_tran_rcon"
      | "reformat_numeric" | "canonicalize_datetime" | "normalize_typography"
      | "pad_short_row";
  label: string;
  /** The exact rule label (`"AGS Format Rule 8"`, …), for cross-linking back to
   *  the finding it resolves. */
  rule: string;
  /** Anchor line for ordering/preview; `null` for whole-file kinds. */
  line: number | null;
  /** `safe` is bulk-applicable; `risky` guesses intent and is opt-in only. */
  risk: "safe" | "risky";
  /** EMPTY for the byte-level kinds (`normalize_crlf`, `strip_bom`), which
   *  operate on the whole document rather than a span. */
  edits: SpanEdit[];
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "StandardDict")]
    pub type StandardDictJs;
    #[wasm_bindgen(typescript_type = "Fix[]")]
    pub type FixesJs;
}

/// Serialise the bundled standard dictionary for `dict_version`
/// (`None`/`"auto"` → the [`FALLBACK`] edition; else `4.0.3|4.0.4|4.1|4.1.1|
/// 4.2`). Groups are sorted by code; each group's headings keep the canonical
/// dictionary order. Returns the web reference UI's `{ags_edition, groups:[…]}`
/// shape — built by the shared `dict::dictionary_dto` (#294 F#6), the same
/// source `laterite.registry.dictionary()` and Node's render.
#[wasm_bindgen]
pub fn dictionary(dict_version: Option<String>) -> Result<StandardDictJs, JsError> {
    console_error_panic_hook::set_once();
    let dto = dictionary_core(dict_version.as_deref()).map_err(|m| JsError::new(&m))?;
    to_js(&dto)
}

/// The host-testable core of [`dictionary`]: resolve the edition (`None`/`auto`
/// → [`FALLBACK`]) and build the shared DTO.
fn dictionary_core(
    dict_version: Option<&str>,
) -> Result<laterite_ags4_validator::dict::DictionaryDto, String> {
    let version = resolve_dict_override(dict_version)?.unwrap_or(FALLBACK);
    Ok(laterite_ags4_validator::dict::dictionary_dto(version))
}

// ---------------------------------------------------------------------
// censor() -> anonymise a file with the shared scrub engine (#581).
//
// The browser Anonymiser drives the SAME `laterite-ags4-censor` engine the
// corpus `censor` tool uses (Phase 2 of the #527 convergence), instead of a
// hand-written TS reimplementation. It's a batch action (Download click), off
// the render path, so it rides the engine wasm asynchronously in the validator
// worker rather than a boot-critical main-thread instance.
// ---------------------------------------------------------------------

/// `{ text, tally }` — the anonymised file plus the per-action cell/structure
/// counts the Anonymiser surfaces. `tally`'s fields match the leaf's snake_case.
#[derive(Serialize)]
struct CensorDto {
    text: String,
    tally: laterite_ags4_censor::Tally,
}

ts_section! {
    TS_CENSOR_RESULT,
    TS_CENSOR_RESULT_SECTION,
    r#"
/** Per-action counts from an anonymisation pass. Every field is a count of
 *  cells or structures affected, so `0` everywhere means nothing matched. */
export interface CensorTally {
  pseudonym: number;
  blank: number;
  token: number;
  /** Bracketed geological units stripped from description cells. */
  brackets: number;
  /** Substrings replaced by the keyword pass. */
  keyword: number;
  /** Custom (non-dictionary) columns deleted. */
  dropped_cols: number;
  /** Custom (non-dictionary) groups deleted. */
  dropped_groups: number;
  /** Orphaned DICT/ABBR definition rows of dropped custom groups/headings. */
  dropped_defs: number;
}

/** The `censor` result: the anonymised file plus what was changed. */
export interface CensorResult {
  text: string;
  tally: CensorTally;
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CensorResult")]
    pub type CensorResultJs;
}

/// `censor`'s named options — the four policy knobs, off the argument list.
///
/// This was the widest export in the crate at six positional arguments, four of
/// them a `JsValue`/`&str`/`bool`/`bool` tail. `censor(d, j, null, "X", true,
/// false)` is unreadable at the call site and two adjacent booleans are exactly
/// where a silent transposition lives.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct CensorOptions {
    selected_codes: Option<Vec<String>>,
    token: Option<String>,
    drop_custom: Option<bool>,
    include_freetext: Option<bool>,
}

impl WasmOptions for CensorOptions {
    const KEYS: &'static [&'static str] =
        &["selectedCodes", "token", "dropCustom", "includeFreetext"];
    const WHAT: &'static str = "censor options";
}

#[wasm_bindgen(typescript_custom_section)]
const TS_CENSOR_OPTIONS: &'static str = r#"
/** Named options for `censor`. */
export interface CensorOptions {
  /** Restrict the policy to these heading codes — the user's ticked columns.
   *  Omit (or `null`) to apply it to EVERY classified heading, which is the
   *  broader action, so the default is the safe one. */
  selectedCodes?: string[] | null;
  /** Replacement for token/brackets hits. Default `"[REDACTED]"`. */
  token?: string;
  /** Also delete non-dictionary groups and columns, plus the DICT/ABBR rows
   *  that defined them. Default **false** — this discards data, so it is opt-in. */
  dropCustom?: boolean;
  /** Tokenise description free-text rather than only stripping its `[units]`.
   *  Default **false**. */
  includeFreetext?: boolean;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CensorOptions")]
    pub type CensorOptionsJs;
}

/// Anonymise `data` with the shared engine.
///
/// * `sensitive_json` — the classification SSOT (`sensitive_headings.json`).
///   Required: without it there is no policy and nothing would be scrubbed,
///   which is the one outcome a caller must never get by accident.
/// * `opts` — a [`CensorOptions`] object; every field optional, so
///   `censor(data, json)` is a complete call. An unrecognised key is refused
///   by name.
///
/// `PROJ_ID`'s filehash is the full 64-hex SHA-256 of `data` (a KEY field —
/// full width so a collision is cryptographically nil); the leaf takes that id
/// precomputed, so this wrapper hashes the bytes.
#[wasm_bindgen]
pub fn censor(
    data: &[u8],
    sensitive_json: &str,
    opts: Option<CensorOptionsJs>,
) -> Result<CensorResultJs, JsError> {
    console_error_panic_hook::set_once();
    let o: CensorOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let dto = censor_core(data, sensitive_json, o).map_err(|m| JsError::new(&m))?;
    to_js(&dto)
}

/// The host-testable core of [`censor`].
///
/// This is the one door where an untested default LEAKS. `selectedCodes`
/// omitted means "every classified heading" — the WIDER scrub — so forgetting
/// the option over-redacts rather than under-redacts, and the difference
/// between those two behaviours is a data-protection outcome, not a
/// preference. `PROJ_ID`'s filehash is the full 64-hex SHA-256 of the input
/// bytes (a KEY field, so full width), computed here because the leaf takes it
/// precomputed.
fn censor_core(data: &[u8], sensitive_json: &str, o: CensorOptions) -> Result<CensorDto, String> {
    let include_freetext = o.include_freetext.unwrap_or(false);
    let drop_custom = o.drop_custom.unwrap_or(false);
    let token = o.token.unwrap_or_else(|| "[REDACTED]".to_string());

    // Lossy decode (matches the Anonymiser's `TextDecoder({fatal:false})`): a
    // browser anonymises what it can rather than skipping non-UTF-8 outright.
    let text = String::from_utf8_lossy(data);
    let file_id = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(data));

    let mut policy =
        laterite_ags4_censor::Policy::from_sensitive_json(sensitive_json, include_freetext)
            .map_err(|e| e.to_string())?;
    // Omitted/null → keep the full policy (every classified heading); a list
    // restricts it to the browser's column selection. Note the asymmetry is
    // deliberate: the default is the WIDER scrub, so forgetting the option
    // over-redacts rather than leaking.
    if let Some(codes) = o.selected_codes {
        policy.retain_codes(&codes.into_iter().collect());
    }

    let opts = laterite_ags4_censor::CensorOptions {
        token,
        keywords: Vec::new(),
        drop_custom,
    };
    let (out_text, tally) = laterite_ags4_censor::censor(&text, &file_id, &policy, &opts);
    Ok(CensorDto {
        text: out_text,
        tally,
    })
}

#[cfg(test)]
mod tests {
    //! Parity-by-construction guard for `read()`'s typed-Arrow path.
    //!
    //! `build_column` is the whole casting surface (the wasm-bindgen
    //! wrappers above only marshal it), and it casts through the SAME
    //! `laterite_ags4_types` fns — off the file's TYPE row — that every other
    //! DuckDB bridge uses: `laterite-node/ts/duckdb.ts` on the Node surface, and
    //! the `laterite_ags4` DuckDB extension in its own repo. (There is no DuckDB
    //! crate in THIS workspace; core is deliberately DuckDB-free.) So asserting
    //! the Arrow `DataType` + cell values here proves the explorer casts a file
    //! identically to those bridges, with no DuckDB/Node/wasm runtime.
    //! The datetime oracle is computed independently via `chrono`.
    use super::*;
    // `Array` provides `is_null`/`len`; ArrayRef/DataType/TimeUnit assert the
    // shape of what the shared laterite-ags4-types builder hands back.
    use arrow::array::{
        Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray,
        TimestampMicrosecondArray,
    };
    use arrow::datatypes::{DataType, TimeUnit};
    use chrono::NaiveDate;

    // Exercises every canonical category: ID/X -> Utf8, 2DP -> Float64,
    // DT -> Timestamp (full datetime, date-only -> midnight, empty ->
    // null), 0DP -> Int64, YN -> Bool. BH03's blank coords/dates check
    // the null path; SAMP is a child group with a YN column.
    const FIXTURE: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_STAR\",\"LOCA_ENDD\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\",\"m\",\"yyyy-mm-dd\",\"yyyy-mm-dd\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"DT\",\"DT\",\"X\"\r\n\
\"DATA\",\"BH01\",\"523145.67\",\"2020-08-18 09:30:00\",\"2020-08-19\",\"first\"\r\n\
\"DATA\",\"BH02\",\"523200.00\",\"2020-08-20\",\"\",\"second\"\r\n\
\"DATA\",\"BH03\",\"\",\"\",\"\",\"third\"\r\n\
\r\n\
\"GROUP\",\"GEOL\"\r\n\
\"HEADING\",\"LOCA_ID\",\"GEOL_STAT\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"0DP\"\r\n\
\"DATA\",\"BH01\",\"1\"\r\n\
\"DATA\",\"BH01\",\"2\"\r\n\
\r\n\
\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_DEPTH_OK\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"YN\"\r\n\
\"DATA\",\"BH01\",\"Y\"\r\n\
\"DATA\",\"BH01\",\"N\"\r\n";

    fn parsed() -> ParsedFile {
        parse_bytes(FIXTURE, encoding_rs::UTF_8).expect("fixture parses")
    }

    /// Build the typed column for `group`'s heading `name`, returning the
    /// array + its `DataType`. Routes through the shared laterite-ags4-types builder
    /// (the production path), feeding it this column's cells.
    fn column(file: &ParsedFile, group: &str, name: &str) -> (ArrayRef, DataType) {
        let g = &file.groups[group];
        let col = g.headings.iter().position(|h| h == name).expect("heading");
        let ags_type = &g.types[col];
        laterite_ags4_types::arrow_cols::build_column(g.rows.len(), ags_type, |row| {
            g.rows
                .get(row)
                .and_then(|r| r.values.get(col))
                .map(String::as_str)
        })
    }

    fn micros(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(hh, mm, ss)
            .unwrap()
            .and_utc()
            .timestamp_micros()
    }

    const CLEAN_FIXTURE: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

    /// Certify options carrying only the clock — the one field with no default,
    /// since wasm cannot read one.
    fn stamped_at(when: &str) -> CertifyOptions {
        CertifyOptions {
            checked_at: Some(when.to_string()),
            ..Default::default()
        }
    }

    /// A wasm-minted certificate now carries the shared engine identity — it used
    /// to stamp "laterite-ags4-wasm", which siloed browser certs from every other
    /// surface. Now a cert downloaded from the web app is one every surface can read.
    /// (#430 PR 1a)
    #[test]
    fn certify_stamps_the_unified_engine_identity() {
        let json = match certify_core(CLEAN_FIXTURE, &stamped_at("2020-01-01T00:00:00Z")) {
            Ok(s) => s,
            Err(_) => panic!("a clean minimal AGS4 file must certify"),
        };
        assert!(
            json.contains("laterite_ags4") && !json.contains("laterite-ags4-wasm"),
            "wasm cert must stamp the unified engine identity, got: {json}"
        );
    }

    /// The browser used to run an ERRORS-ONLY validation and then write `warnings: 0,
    /// fyi: 0` into the stamp — two tiers it had never measured, on a claim any other
    /// surface would have believed. Every tier a wasm cert names, it looked at.
    #[test]
    fn a_browser_minted_certificate_measured_every_tier_it_names() {
        let json = certify_core(CLEAN_FIXTURE, &stamped_at("2020-01-01T00:00:00Z"))
            .unwrap_or_else(|_| panic!("a clean minimal AGS4 file must certify"));
        let v: serde_json::Value = serde_json::from_str(&json).expect("the cert is JSON");
        for tier in ["errors", "warnings", "fyi"] {
            assert_eq!(
                v["validation"][tier]["state"], "measured",
                "the {tier} tier must be MEASURED, not asserted: {json}"
            );
            assert_eq!(v["validation"][tier]["count"], 0, "{tier}");
        }
    }

    /// The sandbox has no filesystem, and the stamp has no field in which to pretend
    /// otherwise: nothing a browser mints can claim Rule 20's on-disk half ran.
    #[test]
    fn a_browser_minted_certificate_cannot_claim_a_world_check() {
        let json = certify_core(CLEAN_FIXTURE, &stamped_at("2020-01-01T00:00:00Z"))
            .unwrap_or_else(|_| panic!("a clean minimal AGS4 file must certify"));
        assert!(
            !json.contains("check_files"),
            "the stamp must carry no world claim at all: {json}"
        );
    }

    #[test]
    fn id_and_x_are_utf8() {
        let file = parsed();
        for name in ["LOCA_ID", "LOCA_REM"] {
            let (arr, dt) = column(&file, "LOCA", name);
            assert_eq!(dt, DataType::Utf8, "{name}");
            assert!(arr.as_any().is::<StringArray>());
        }
        let (rem, _) = column(&file, "LOCA", "LOCA_REM");
        let rem = rem.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(rem.value(0), "first");
        assert_eq!(rem.value(2), "third");
    }

    #[test]
    fn two_dp_is_float64_with_nulls() {
        let file = parsed();
        let (arr, dt) = column(&file, "LOCA", "LOCA_NATE");
        assert_eq!(dt, DataType::Float64);
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.value(0), 523145.67);
        assert_eq!(a.value(1), 523200.00);
        assert!(a.is_null(2), "blank 2DP cell -> null");
    }

    #[test]
    fn dt_is_timestamp_full_dateonly_and_null() {
        let file = parsed();
        let (star, dt) = column(&file, "LOCA", "LOCA_STAR");
        assert_eq!(dt, DataType::Timestamp(TimeUnit::Microsecond, None));
        let star = star
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        // full datetime kept; date-only promoted to midnight; blank null.
        assert_eq!(star.value(0), micros(2020, 8, 18, 9, 30, 0));
        assert_eq!(star.value(1), micros(2020, 8, 20, 0, 0, 0));
        assert!(star.is_null(2));

        let (end, _) = column(&file, "LOCA", "LOCA_ENDD");
        let end = end
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert_eq!(end.value(0), micros(2020, 8, 19, 0, 0, 0));
        assert!(end.is_null(1), "blank DT cell -> null");
    }

    #[test]
    fn zero_dp_is_int64() {
        let file = parsed();
        let (arr, dt) = column(&file, "GEOL", "GEOL_STAT");
        assert_eq!(dt, DataType::Int64);
        let a = arr.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(a.value(0), 1);
        assert_eq!(a.value(1), 2);
    }

    #[test]
    fn yn_is_bool() {
        let file = parsed();
        let (arr, dt) = column(&file, "SAMP", "SAMP_DEPTH_OK");
        assert_eq!(dt, DataType::Boolean);
        let a = arr.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(a.value(0));
        assert!(!a.value(1));
    }

    #[test]
    fn ragged_short_row_yields_nulls_not_panic() {
        // A data row shorter than the heading count must null the missing
        // tail columns, never panic or misalign — the explorer has to
        // survive malformed real-world files.
        let bytes = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\"\r\n";
        let file = parse_bytes(bytes, encoding_rs::UTF_8).unwrap();
        let (arr, _) = column(&file, "LOCA", "LOCA_NATE");
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.len(), 1);
        assert!(a.is_null(0));
    }

    // --- encoding resolution + transcode (the cp1252/UTF-8 path) ---
    // This is the bug class behind the web app's mojibake guard + the
    // "applying a fix also normalises encoding" promise; it had zero coverage.

    #[test]
    fn resolve_encoding_maps_the_offered_labels() {
        assert_eq!(resolve_encoding(None).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("")).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("utf-8")).unwrap().name(), "UTF-8");
        for label in ["windows-1252", "CP1252", "latin1", "ISO-8859-1"] {
            assert_eq!(
                resolve_encoding(Some(label)).unwrap().name(),
                "windows-1252",
                "{label}"
            );
        }
    }

    /// An unknown label is an ERROR here, as it always was on Python.
    ///
    /// This assertion used to say the opposite — "an unknown label falls back to
    /// UTF-8 (lossy), not an error" — which is how the bug survived: it was not an
    /// oversight, it was *codified*. But the fallback is not lossy, it is silent:
    /// `C3 A9` decodes cleanly as `é` in UTF-8 and `Ã©` in cp1252, so a caller who
    /// typo'd their label got the wrong text and no error at all. `apply_fixes` would
    /// then rewrite the file from that mis-decode.
    #[test]
    fn an_unknown_encoding_label_is_an_error_not_a_fallback() {
        assert!(resolve_encoding(Some("not-a-charset")).is_err());
        assert!(resolve_encoding(Some("cp1252x")).is_err());
        // ...and the label is named, so the caller can see their typo.
        assert!(
            resolve_encoding(Some("cp1252x"))
                .unwrap_err()
                .contains("cp1252x")
        );
    }

    #[test]
    fn classify_collapses_notfound_and_io_to_io() {
        // The deliberate, allowlisted divergence from the producer: with no
        // filesystem `NotFound`/`Io` are unreachable, so both collapse to "io"
        // (the producer's `kind()` returns "not_found" for `NotFound`). Everything
        // else delegates verbatim. Pins the divergence in-crate.
        assert_eq!(classify(&ValidatorError::NotFound("x".into())).0, "io");
        assert_eq!(
            classify(&ValidatorError::Io {
                path: "x".into(),
                source: std::io::Error::other("x"),
            })
            .0,
            "io"
        );
        assert_eq!(classify(&ValidatorError::NotAgs4("x".into())).0, "not_ags4");
    }

    #[test]
    fn resolve_dict_override_accepts_every_bundled_edition() {
        use laterite_ags4_validator::DictVersion;
        for ed in DictVersion::ALL {
            assert!(
                resolve_dict_override(Some(ed.as_str())).is_ok(),
                "bundled edition {} must resolve",
                ed.as_str()
            );
        }
        assert!(resolve_dict_override(Some("auto")).unwrap().is_none());
        assert!(resolve_dict_override(None).unwrap().is_none());
        // A bogus label errors, and the message lists EVERY bundled edition —
        // proving it derives from `DictVersion::ALL`, not a stale hand-list.
        let err = resolve_dict_override(Some("9.9")).unwrap_err();
        for ed in DictVersion::ALL {
            assert!(
                err.contains(ed.as_str()),
                "message must list {}",
                ed.as_str()
            );
        }
    }

    #[test]
    fn byte_0xe9_is_replacement_under_utf8_but_e_acute_under_cp1252() {
        let data = [b'a', 0xE9, b'b']; // 0xE9 = 'é' in cp1252, invalid UTF-8
        let (utf8, _, had_errors) = resolve_encoding(Some("utf-8")).unwrap().decode(&data);
        assert!(had_errors, "0xE9 is not valid UTF-8");
        assert!(
            utf8.contains('\u{FFFD}'),
            "lossy decode inserts U+FFFD: {utf8:?}"
        );
        let (cp, _, had) = resolve_encoding(Some("windows-1252"))
            .unwrap()
            .decode(&data);
        assert!(!had);
        assert_eq!(cp, "aéb");
    }

    #[test]
    fn apply_fixes_encoding_path_transcodes_cp1252_to_utf8() {
        // Mirror apply_fixes' encoding pipeline (decode → apply → into_bytes)
        // without the wasm-bindgen JsValue: a cp1252 0xE9 byte must come back as
        // the UTF-8 encoding of 'é' (0xC3 0xA9), even with no fixes selected.
        let data = [b'a', 0xE9, b'b'];
        let encoding = resolve_encoding(Some("windows-1252")).unwrap();
        let (text, _, _) = encoding.decode(&data);
        let out = laterite_ags4_validator::fixes::apply_fixes(&text, false, &[]).into_bytes();
        assert_eq!(out, vec![b'a', 0xC3, 0xA9, b'b']);
    }

    // --- dictionary() data source ---
    // The JsValue wrapper can't be built off-wasm, so assert the per-edition
    // standard dictionary the export serialises actually carries real names +
    // descriptions (the scaffolded merged JSON the UI used to fetch did not).

    #[test]
    fn bundled_dictionary_has_real_names_and_descriptions() {
        let d = Dictionary::bundled(DictVersion::V4_1_1);
        let codes: Vec<&str> = d.group_codes().collect();
        assert!(codes.contains(&"LOCA"), "LOCA must be a standard group");
        assert!(
            !d.group("LOCA").unwrap().desc.trim().is_empty(),
            "LOCA must have a real group description",
        );
        let e = d.heading("LOCA", "LOCA_ID").expect("LOCA_ID heading");
        assert_eq!(e.status, "KEY");
        assert_eq!(e.ags_type, "ID");
        assert!(
            !e.desc.trim().is_empty(),
            "LOCA_ID must have a real description, got {:?}",
            e.desc,
        );
    }

    #[test]
    fn bundled_dictionary_differs_across_editions() {
        // 4.2 added groups over 4.0.3 — the per-edition dicts are not identical,
        // which is the whole point of making the browser edition-selectable.
        let n_403 = Dictionary::bundled(DictVersion::V4_0_3)
            .group_codes()
            .count();
        let n_42 = Dictionary::bundled(DictVersion::V4_2).group_codes().count();
        assert!(n_403 > 0 && n_42 > 0);
        assert!(
            n_42 >= n_403,
            "4.2 should have at least as many groups as 4.0.3"
        );
    }

    #[test]
    fn arrow_ipc_keys_match_the_shared_golden_and_default_strips() {
        // SAME fixture + golden UUIDv8s as the Python (test_content_keys.py) and
        // Node (p3-content-keys.test.ts) tests — the ids come from the ONE shared
        // keychain, so matching here proves the wasm produces byte-identical keys
        // (a cross-surface parity check, ahead of Phase 6's full proof). (#303)
        const SRC: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\
\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"PROJ_ID\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\"DATA\",\"BH1\",\"P1\"\r\n";
        let ds = ParsedDataset {
            parsed: parse_bytes(SRC, encoding_rs::UTF_8).expect("parses"),
        };

        // First-row string cell of `col` in an IPC stream, or None (missing col / null).
        let first = |ipc: &[u8], col: &str| -> Option<String> {
            let mut r =
                arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(ipc.to_vec()), None)
                    .unwrap();
            let batch = r.next().unwrap().unwrap();
            let i = batch.schema().index_of(col).ok()?;
            let a = batch
                .column(i)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            if a.is_null(0) {
                None
            } else {
                Some(a.value(0).to_string())
            }
        };

        // keys=true → the golden UUIDv8s; child._parent_id links to parent._id;
        // a root group's _parent_id is NULL.
        // `unwrap_or_else(|_| panic!(…))`, NOT `.expect(…)`: the error is a
        // `JsError`, which does not implement `Debug`, so `.expect()` will not
        // compile. This read `.ok().expect(…)` for that reason and clippy's
        // `ok_expect` fires on it — but taking clippy's suggestion literally
        // breaks the build, so the escape is to drop the error explicitly.
        let proj = ds
            .arrow_ipc("PROJ", Some(true), None)
            .unwrap_or_else(|_| panic!("PROJ keyed"));
        let loca = ds
            .arrow_ipc("LOCA", Some(true), None)
            .unwrap_or_else(|_| panic!("LOCA keyed"));
        assert_eq!(
            first(&proj, "_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(
            first(&loca, "_id").as_deref(),
            Some("a7025a6f-d9b8-83b6-8fad-81c0c744edbc"),
        );
        assert_eq!(
            first(&loca, "_parent_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(first(&proj, "_parent_id"), None);

        // The default (no keys) strips: a plain frame carries no `_id` column.
        let plain = ds
            .arrow_ipc("PROJ", None, None)
            .unwrap_or_else(|_| panic!("PROJ plain"));
        assert!(
            first(&plain, "_id").is_none(),
            "default arrow_ipc must not carry _id",
        );
    }
}

/// The four exports whose signatures are plain Rust — no `JsValue`, no
/// `JsError` — and which are therefore reachable from a NATIVE `cargo test`.
///
/// Every other `#[wasm_bindgen]` export in this file takes or returns a JS type,
/// so it can only be driven from the browser (or the `wasm-engine` xcheck leg).
/// These four cannot: they are pure metadata doors, and until now nothing called
/// them from anywhere except JavaScript. That mattered most for the two identity
/// doors — `engine_fingerprint` exists precisely because a *constant* once stood
/// in for a real answer here (#556), and a constant is exactly what these would
/// silently become if someone replaced the crate lookup with a literal.
#[cfg(test)]
mod metadata_door_tests {
    use super::*;

    #[test]
    fn version_is_the_crate_version_not_a_literal() {
        // The bug this whole family of doors was written for: a hand-written
        // version string that kept printing while the workspace moved past it.
        // Asserting against `CARGO_PKG_VERSION` is what makes a pasted literal
        // fail rather than merely look plausible.
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert!(
            version().split('.').count() >= 3,
            "not a semver: {}",
            version()
        );
    }

    #[test]
    fn engine_version_comes_from_the_validator_not_this_crate() {
        // The tiers split in #202: this package carries the PRODUCT number and the
        // engine carries its own. They are equal today, so a test that only
        // compared them would pass while reading the wrong one — assert the source
        // instead.
        assert_eq!(engine_version(), laterite_ags4_validator::VERSION);
    }

    #[test]
    fn engine_fingerprint_is_the_validator_digest() {
        assert_eq!(
            engine_fingerprint(),
            laterite_ags4_validator::ENGINE_FINGERPRINT
        );
    }

    #[test]
    fn engine_fingerprint_is_a_well_formed_digest() {
        // 16 hex chars — `build.rs` truncates the SHA-256. A placeholder or an
        // empty string would compare EQUAL across two surfaces and mean nothing,
        // which is the one failure this value exists to prevent.
        let fp = engine_fingerprint();
        assert_eq!(fp.len(), 16, "fingerprint {fp:?} is not 16 chars");
        assert!(
            fp.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "fingerprint {fp:?} is not lowercase hex"
        );
    }

    #[test]
    fn the_version_and_the_fingerprint_are_different_answers() {
        // They answer different questions — "which release" vs "which rules" — so
        // a surface that wired one door to the other would still look sensible.
        assert_ne!(version(), engine_fingerprint());
    }

    #[test]
    fn list_rules_is_the_validator_catalogue_verbatim() {
        // The browser parses this into typed rule entries, so it must be the
        // catalogue itself and not a re-serialisation that could reorder or
        // re-shape it.
        assert_eq!(list_rules(), laterite_ags4_validator::rule_metadata_json());
    }

    #[test]
    fn list_rules_is_parseable_json_describing_real_rules() {
        // A door the browser JSON.parses. If it ever returned a Rust Debug string
        // or an error message, every consumer would fail at parse time with no
        // clue which surface produced it.
        let parsed: serde_json::Value =
            serde_json::from_str(&list_rules()).expect("rule catalogue is JSON");
        assert!(
            parsed.is_object() || parsed.is_array(),
            "expected a JSON object/array, got {parsed}"
        );
        assert!(
            list_rules().contains("Rule") || list_rules().contains("rule"),
            "the catalogue mentions no rules at all"
        );
    }
}

#[cfg(test)]
mod core_door_tests {
    //! The extracted cores of the `#[wasm_bindgen]` exports.
    //!
    //! Every export in this crate names a JS type — `JsValue`, `JsError`, one of
    //! the `*Js` aliases — and this crate has no `wasm-bindgen-test` lane, so
    //! `cargo test` cannot call a single one of them. That is a measurement
    //! problem only if the exports are thin. They were not: option folding,
    //! edition resolution, the encoding decision, the leak-safe censor default
    //! and four separate "return nothing rather than the wrong thing" arms all
    //! lived inside signatures no test could enter.
    //!
    //! So the logic moved out and these tests hold it. What is left at the
    //! boundary is decode → core → marshal, which is genuinely browser-only and
    //! is covered by the `wasm-engine` xcheck leg instead.
    use super::*;

    const CLEAN: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

    /// The classification SSOT the browser Anonymiser fetches, read from the
    /// same file the engine ships rather than a hand-written stub — a stub would
    /// let this suite pass while the real policy said something else.
    const SENSITIVE: &str = include_str!("../../laterite-ags4-core/data/sensitive_headings.json");

    /// Two groups, one keyed child, one heading typed `2DP` — enough to exercise
    /// edition resolution, KEY matching and a type clash.
    pub(super) const LOCA_A: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"100.00\"\r\n\
\"DATA\",\"BH02\",\"200.00\"\r\n";

    /// `LOCA_A` with BH01 moved, BH02 gone and BH03 new — one of each verdict.
    pub(super) const LOCA_B: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"999.00\"\r\n\
\"DATA\",\"BH03\",\"300.00\"\r\n";

    /// `LOCA_A` with `LOCA_NATE` typed `X` instead of `2DP` — the clash.
    const LOCA_CLASH: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH09\",\"nine\"\r\n";

    fn err(r: Result<impl Sized, String>) -> String {
        match r {
            Ok(_) => panic!("expected an error, got Ok"),
            Err(m) => m,
        }
    }

    // ---------------------------------------------------------------
    // TranInput::fold / build_parts — the "all five or none" rule
    // ---------------------------------------------------------------

    #[test]
    fn a_partial_tran_names_what_is_missing() {
        // The reason `tran` is a nested struct of Options rather than five
        // positional slots: `deny_unknown_fields` is a NO-OP under
        // serde-wasm-bindgen, so a misspelled `producr` cannot be caught by
        // enumeration — it arrives as an unset `producer`, and requiredness is
        // what turns that into a named error instead of a silently absent TRAN.
        let partial = TranInput {
            issue: Some("1".into()),
            date: Some("2020-08-18".into()),
            producer: None,
            recipient: Some("ACME".into()),
            status: Some("FINAL".into()),
            ..Default::default()
        };
        let msg = err(partial.fold());
        assert!(
            msg.to_ascii_lowercase().contains("producer"),
            "the missing field must be named, got: {msg}"
        );
    }

    #[test]
    fn an_entirely_absent_tran_is_not_an_error() {
        // "None" is a legitimate answer — no TRAN is written and Rule 14 reports
        // the gap, which is the honest outcome. Only a PARTIAL stamp is a
        // mistake.
        let folded = TranInput::default().fold().expect("an empty tran is legal");
        assert!(folded.is_none(), "an empty tran must fold to no stamp");
    }

    #[test]
    fn all_five_fold_to_a_stamp_and_the_extras_attach() {
        let full = TranInput {
            issue: Some("1".into()),
            date: Some("2020-08-18".into()),
            producer: Some("ACME Drilling".into()),
            recipient: Some("ACME Consulting".into()),
            status: Some("FINAL".into()),
            description: Some("Phase 2 boreholes".into()),
            remarks: Some("re-issued".into()),
        };
        let stamp = full.fold().expect("a complete tran folds").expect("some");
        // description/remarks are optional EXTRAS, not part of the five — they
        // must survive the fold rather than being dropped by it.
        let rendered = format!("{stamp:?}");
        assert!(
            rendered.contains("Phase 2 boreholes") && rendered.contains("re-issued"),
            "the optional extras were dropped: {rendered}"
        );
    }

    #[test]
    fn synthesise_metadata_defaults_to_off() {
        // Opt-in since 2026-07-24: no surface mints GROUPs the caller never
        // wrote unless told to. The browser lost this once already by inheriting
        // a default, so the default is asserted rather than assumed.
        let (_, _, synth, _) = build_parts(BuildOptions::default()).expect("defaults fold");
        assert!(!synth, "synthesis must be off unless asked for");
    }

    #[test]
    fn a_partial_tran_stops_a_build_before_it_emits() {
        // The fold runs inside build_parts, so an incomplete stamp fails the
        // whole call rather than quietly emitting a file with no TRAN.
        let o = BuildOptions {
            tran: Some(TranInput {
                issue: Some("1".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let msg = err(build_ags4_core("[]", o));
        assert!(
            !msg.contains("invalid groups JSON"),
            "the tran must be rejected before the groups are parsed, got: {msg}"
        );
    }

    // ---------------------------------------------------------------
    // build_ags4_core / emit_mode / emit_edition
    // ---------------------------------------------------------------

    #[test]
    fn malformed_groups_json_says_so() {
        let msg = err(build_ags4_core("not json at all", BuildOptions::default()));
        assert!(
            msg.contains("invalid groups JSON"),
            "expected a JSON error, got: {msg}"
        );
    }

    #[test]
    fn an_unknown_mode_lists_the_three_that_exist() {
        let o = BuildOptions {
            mode: Some("autofixx".into()),
            ..Default::default()
        };
        let msg = err(build_ags4_core("[]", o));
        assert!(
            msg.contains("autofix") && msg.contains("report") && msg.contains("strict"),
            "the rejection must list the accepted modes, got: {msg}"
        );
    }

    #[test]
    fn every_documented_mode_is_accepted() {
        // The TS union says "autofix" | "report" | "strict"; a mode the .d.ts
        // promises but the Rust rejects is a lie the type checker cannot catch.
        for mode in ["autofix", "report", "strict"] {
            assert!(
                emit_mode(Some(mode)).is_ok(),
                "documented mode {mode:?} was rejected"
            );
        }
        // Absent and empty both mean "the default", not "an unknown mode".
        assert!(emit_mode(None).is_ok());
        assert!(emit_mode(Some("")).is_ok());
        // Case-insensitively, since the browser passes user-facing strings.
        assert!(emit_mode(Some("AutoFix")).is_ok());
    }

    #[test]
    fn an_unknown_edition_lists_the_editions() {
        let o = BuildOptions {
            dict_version: Some("4.9".into()),
            ..Default::default()
        };
        let msg = err(build_ags4_core("[]", o));
        assert!(
            msg.contains("4.1.1") && msg.contains("unknown edition"),
            "the rejection must list the real editions, got: {msg}"
        );
    }

    #[test]
    fn auto_and_absent_mean_the_fallback_edition() {
        assert_eq!(emit_edition(None).expect("absent"), FALLBACK);
        assert_eq!(emit_edition(Some("auto")).expect("auto"), FALLBACK);
        assert_eq!(emit_edition(Some("")).expect("empty"), FALLBACK);
    }

    #[test]
    fn a_built_file_carries_the_rows_it_was_given() {
        let groups = r#"[{"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],
                          "rows":[["P1","Test project"]]}]"#;
        let report = build_ags4_core(groups, BuildOptions::default()).expect("builds");
        assert!(
            report.text.contains("\"DATA\",\"P1\",\"Test project\""),
            "the data row is missing from the output:\n{}",
            report.text
        );
        // CRLF is Rule 2 and the browser downloads this as a file — a stray LF
        // makes the artefact invalid on arrival.
        assert!(report.text.contains("\r\n"), "output must be CRLF");
    }

    // ---------------------------------------------------------------
    // group_from_ipc — the columnar door
    // ---------------------------------------------------------------

    /// One group as an Arrow IPC stream, the shape `build_ags4_ipc` receives.
    fn ipc_of(names: &[&str], rows: &[&[&str]]) -> Vec<u8> {
        use arrow::array::StringArray;
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let fields: Vec<Field> = names
            .iter()
            .map(|n| Field::new(*n, DataType::Utf8, true))
            .collect();
        let schema = Arc::new(Schema::new(fields));
        let cols: Vec<arrow::array::ArrayRef> = (0..names.len())
            .map(|c| {
                let vals: Vec<&str> = rows.iter().map(|r| r[c]).collect();
                Arc::new(StringArray::from(vals)) as arrow::array::ArrayRef
            })
            .collect();
        let batch = arrow::record_batch::RecordBatch::try_new(schema.clone(), cols).expect("batch");
        let mut buf = Vec::new();
        {
            let mut w = arrow::ipc::writer::StreamWriter::try_new(&mut buf, &schema).expect("w");
            w.write(&batch).expect("write");
            w.finish().expect("finish");
        }
        buf
    }

    #[test]
    fn non_ipc_bytes_are_reported_as_an_arrow_error() {
        let msg = err(group_from_ipc("LOCA".into(), b"definitely not arrow"));
        assert!(
            msg.contains("arrow ipc"),
            "the caller needs to know it was the IPC decode, got: {msg}"
        );
    }

    #[test]
    fn a_synthetic_key_column_never_becomes_an_ags_heading() {
        // The #303 round trip: `read(keys=true)` prepends `_id`/`_parent_id`, and
        // feeding that frame straight back into build must not emit them as
        // headings — AGS headings never start with "_", so they are dropped.
        // Nothing exercised the projection branch that does the dropping.
        let ipc = ipc_of(
            &["_id", "LOCA_ID", "_parent_id", "LOCA_NATE"],
            &[&["u-1", "BH01", "u-0", "100.00"]],
        );
        let group = group_from_ipc("LOCA".into(), &ipc).expect("decodes");
        assert_eq!(
            group.headings,
            vec!["LOCA_ID".to_string(), "LOCA_NATE".to_string()],
            "underscore-prefixed columns must be dropped"
        );
        // And the values must stay aligned with the columns that survived — a
        // projection that dropped the schema entry but not the data would put
        // "u-1" under LOCA_ID.
        assert_eq!(group.rows[0][0], "BH01");
        assert_eq!(group.rows[0][1], "100.00");
    }

    #[test]
    fn a_frame_with_no_underscore_columns_is_passed_through_whole() {
        // The other side of the same branch: when nothing needs dropping the
        // projection is skipped entirely, and the columns must be unchanged.
        let ipc = ipc_of(&["LOCA_ID", "LOCA_NATE"], &[&["BH01", "100.00"]]);
        let group = group_from_ipc("LOCA".into(), &ipc).expect("decodes");
        assert_eq!(
            group.headings,
            vec!["LOCA_ID".to_string(), "LOCA_NATE".to_string()]
        );
        assert_eq!(group.rows.len(), 1);
    }

    #[test]
    fn the_columnar_and_json_doors_build_the_same_file() {
        // Two input shapes, one emitter — the whole point of `build_ags4_ipc`
        // existing alongside `build_ags4`. If they can diverge, the columnar
        // door is a second implementation rather than a second door.
        let ipc = ipc_of(&["PROJ_ID", "PROJ_NAME"], &[&["P1", "Test project"]]);
        let from_ipc = build_ipc_core(
            vec![group_from_ipc("PROJ".into(), &ipc).expect("decodes")],
            BuildOptions::default(),
        )
        .expect("ipc builds");
        let from_json = build_ags4_core(
            r#"[{"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],
                 "rows":[["P1","Test project"]]}]"#,
            BuildOptions::default(),
        )
        .expect("json builds");
        assert_eq!(
            from_ipc.text, from_json.text,
            "the two build doors disagree on the same data"
        );
    }

    // ---------------------------------------------------------------
    // compute_fixes_core — four ways to return nothing
    // ---------------------------------------------------------------

    #[test]
    fn a_fixable_file_yields_fixes() {
        // LF line endings breach Rule 2 and are safely fixable, so this is the
        // baseline the four failure paths below are contrasted against — without
        // it, "returns empty" proves nothing.
        let lf: Vec<u8> = CLEAN.iter().copied().filter(|&b| b != b'\r').collect();
        let fixes = compute_fixes_core(&lf, None, None);
        assert!(
            !fixes.is_empty(),
            "a file with LF endings must offer at least the CRLF fix"
        );
    }

    #[test]
    fn an_unknown_edition_yields_no_fixes() {
        assert!(compute_fixes_core(CLEAN, Some("4.9"), None).is_empty());
    }

    #[test]
    fn an_unknown_encoding_yields_no_fixes() {
        // The important one. This door has no error channel, so the alternative
        // to "no fixes" is fixes computed against text we mis-decoded — a button
        // that silently corrupts the user's file.
        let lf: Vec<u8> = CLEAN.iter().copied().filter(|&b| b != b'\r').collect();
        assert!(
            !compute_fixes_core(&lf, None, None).is_empty(),
            "the fixture must be fixable, or the next assertion proves nothing"
        );
        assert!(compute_fixes_core(&lf, None, Some("klingon-1")).is_empty());
    }

    #[test]
    fn unparseable_bytes_yield_no_fixes() {
        assert!(compute_fixes_core(b"not an ags file at all", None, None).is_empty());
    }

    // ---------------------------------------------------------------
    // apply_fixes_core
    // ---------------------------------------------------------------

    #[test]
    fn applying_fixes_with_an_unknown_encoding_is_refused() {
        // This used to fall back to UTF-8 and REWRITE a file it had just
        // mis-decoded — the one place a silent fallback does permanent damage.
        let msg = err(apply_fixes_core(CLEAN, Some("klingon-1"), &[]));
        assert!(
            msg.to_ascii_lowercase().contains("encoding")
                || msg.to_ascii_lowercase().contains("klingon"),
            "the rejection must name the encoding, got: {msg}"
        );
    }

    #[test]
    fn applying_no_fixes_returns_the_bytes_unchanged() {
        let out = apply_fixes_core(CLEAN, None, &[]).expect("applies");
        assert_eq!(out, CLEAN, "an empty fix list must be a no-op");
    }

    #[test]
    fn a_bom_survives_when_stripping_it_was_not_selected() {
        // apply_fixes honours "keep the BOM" only because the core reads the RAW
        // bytes for it — encoding_rs eats the mark during decode, so a version
        // that inspected the decoded text would drop it silently.
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice(CLEAN);
        let out = apply_fixes_core(&bom, None, &[]).expect("applies");
        assert!(
            out.starts_with(&[0xEF, 0xBB, 0xBF]),
            "the BOM was dropped without a strip_bom fix being selected"
        );
    }

    #[test]
    fn a_cp1252_file_comes_back_as_utf8() {
        // Applying to a cp1252 file also normalises its encoding, which is why
        // the UI resets its encoding select afterwards. 0xB0 is DEGREE SIGN in
        // cp1252 and invalid as standalone UTF-8.
        let mut cp = Vec::from(&b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n"[..]);
        cp.extend_from_slice(b"\"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"90");
        cp.push(0xB0);
        cp.extend_from_slice(b"\"\r\n");
        let out = apply_fixes_core(&cp, Some("windows-1252"), &[]).expect("applies");
        let text = String::from_utf8(out).expect("output must be valid UTF-8");
        assert!(
            text.contains('\u{00B0}'),
            "the degree sign did not survive the re-encode: {text}"
        );
    }

    // ---------------------------------------------------------------
    // read_core + ParsedDataset
    // ---------------------------------------------------------------

    #[test]
    fn reading_with_an_unknown_encoding_is_refused() {
        let msg = err(read_core(CLEAN, Some("klingon-1")));
        assert!(!msg.is_empty(), "an unknown encoding must be reported");
    }

    #[test]
    fn an_unreadable_file_reports_the_validator_error_text() {
        // The ParseError -> ValidatorError bridge. It exists so an unparseable
        // file says the same thing here as it does from `validate`; without the
        // conversion the browser would show two different messages for one
        // problem depending on which door the user came through.
        let msg = err(read_core(b"nothing resembling ags4", None));
        let via_validate = ValidatorError::from(
            parse_bytes(b"nothing resembling ags4", encoding_rs::UTF_8)
                .expect_err("must not parse"),
        )
        .to_string();
        assert_eq!(
            msg, via_validate,
            "read and validate must agree on the text"
        );
    }

    #[test]
    fn group_codes_come_back_in_file_order() {
        // The explorer loads tables in this order, and PROJ must land before its
        // children — an alphabetical sort would put LOCA first and break the
        // foreign keys on insert.
        let ds = read_core(LOCA_A, None).expect("reads");
        assert_eq!(
            ds.group_codes(),
            vec!["PROJ".to_string(), "LOCA".to_string()]
        );
    }

    #[test]
    fn meta_is_none_for_a_group_the_file_lacks() {
        let ds = read_core(LOCA_A, None).expect("reads");
        assert!(ds.meta_core("SAMP").is_none());
    }

    #[test]
    fn meta_returns_four_arrays_of_equal_length() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        let n = m.headings.len();
        assert_eq!(n, 2);
        assert_eq!(m.units.len(), n);
        assert_eq!(m.types.len(), n);
        assert_eq!(m.sql_types.len(), n);
    }

    #[test]
    fn a_short_unit_or_type_row_is_padded_not_truncated() {
        // The parallel-array contract is what the UI indexes by, so a file whose
        // UNIT/TYPE rows are shorter than its HEADING row must still yield four
        // equal-length arrays. Truncating instead would silently mislabel every
        // column after the short one.
        let ragged: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH01\",\"1.00\",\"note\"\r\n";
        let ds = read_core(ragged, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        assert_eq!(m.headings.len(), 3);
        assert_eq!(m.units.len(), 3, "units must pad to the heading count");
        assert_eq!(m.types.len(), 3, "types must pad to the heading count");
        // A missing TYPE becomes "X" (free text), which is the safe assumption —
        // it casts to VARCHAR rather than guessing a numeric column.
        assert_eq!(m.types[2], "X");
        assert_eq!(m.units[2], "");
        assert_eq!(m.sql_types[2], sql_type("X"));
    }

    #[test]
    fn sql_types_are_derived_from_the_files_own_type_row() {
        // Parity by construction: the explorer must report the column types the
        // native DuckDB conversion would produce, and both read them off the
        // file's TYPE row through the same `sql_type`.
        let ds = read_core(LOCA_A, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        assert_eq!(m.types, vec!["ID".to_string(), "2DP".to_string()]);
        assert_eq!(
            m.sql_types,
            vec![sql_type("ID").to_string(), sql_type("2DP").to_string()]
        );
    }

    #[test]
    fn arrow_ipc_names_the_group_it_could_not_find() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let msg = err(ds.arrow_ipc_core("ZZZZ", false, false));
        assert!(
            msg.contains("ZZZZ"),
            "the missing code must appear in the error, got: {msg}"
        );
    }

    #[test]
    fn keys_and_content_hash_are_off_by_default_and_add_columns_when_asked() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let plain = ds.arrow_ipc_core("LOCA", false, false).expect("plain");
        let keyed = ds.arrow_ipc_core("LOCA", true, false).expect("keyed");
        let hashed = ds.arrow_ipc_core("LOCA", false, true).expect("hashed");

        // Asserted through the IPC bytes rather than a length comparison: the
        // column NAMES are the contract duckdb-wasm joins on.
        let names = |ipc: &[u8]| -> Vec<String> {
            let r = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(ipc), None)
                .expect("ipc reads");
            r.schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect()
        };
        let plain_names = names(&plain);
        assert!(
            !plain_names.iter().any(|n| n.starts_with('_')),
            "the default frame must carry no synthetic columns, got {plain_names:?}"
        );
        assert!(names(&keyed).contains(&"_id".to_string()));
        assert!(names(&hashed).contains(&"_content_hash".to_string()));
    }

    // ---------------------------------------------------------------
    // diff_core
    // ---------------------------------------------------------------

    #[test]
    fn diffing_with_an_unknown_encoding_is_refused() {
        let o = DiffOptions {
            encoding: Some("klingon-1".into()),
            ..Default::default()
        };
        assert!(diff_core(LOCA_A, LOCA_B, &o).is_err());
    }

    #[test]
    fn an_unparseable_side_is_reported() {
        let o = DiffOptions::default();
        assert!(diff_core(b"junk", LOCA_B, &o).is_err());
        assert!(diff_core(LOCA_A, b"junk", &o).is_err());
    }

    #[test]
    fn a_file_diffed_against_itself_reports_nothing() {
        let d = diff_core(LOCA_A, LOCA_A, &DiffOptions::default()).expect("diffs");
        assert_eq!((d.total_added, d.total_removed, d.total_changed), (0, 0, 0));
        assert!(
            d.groups.is_empty(),
            "no group should be reported as changed"
        );
    }

    #[test]
    fn rows_are_matched_by_key_not_position() {
        // BH01 changed value, BH02 is gone, BH03 is new. Matching by position
        // instead would report two changes and no add/remove.
        let d = diff_core(LOCA_A, LOCA_B, &DiffOptions::default()).expect("diffs");
        assert_eq!(d.total_changed, 1, "BH01's coordinate changed");
        assert_eq!(d.total_removed, 1, "BH02 is gone");
        assert_eq!(d.total_added, 1, "BH03 is new");
        let loca = d.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert!(loca.keyed, "LOCA has dictionary KEY headings");
        assert!(loca.key_headings.contains(&"LOCA_ID".to_string()));
    }

    #[test]
    fn a_row_cap_bounds_the_payload_without_lying_about_the_totals() {
        // The documented contract: `maxRowsPerGroup` caps what each group
        // SERIALISES, and the added/removed/changed counts stay true totals. A
        // cap that also truncated the counts would tell the user a three-row
        // change was a one-row change.
        let capped = DiffOptions {
            max_rows_per_group: Some(1),
            ..Default::default()
        };
        let d = diff_core(LOCA_A, LOCA_B, &capped).expect("diffs");
        let loca = d.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert_eq!(loca.rows.len(), 1, "the cap must bound the serialised rows");
        assert_eq!(
            loca.added + loca.removed + loca.changed,
            3,
            "the totals must survive the cap"
        );
        assert_eq!(d.total_added + d.total_removed + d.total_changed, 3);
    }

    // ---------------------------------------------------------------
    // merge_core
    // ---------------------------------------------------------------

    #[test]
    fn an_unknown_type_clash_token_is_refused_in_the_merge_crates_own_words() {
        // The vocabulary is parsed by laterite-ags4-merge's FromStr precisely so
        // the browser cannot accept a token the CLI rejects, or word the
        // rejection differently. Asserted against that crate's own list, not a
        // literal here — a copy would drift the moment a fourth mode appears.
        let o = MergeOptions {
            on_type_clash: Some("widenn".into()),
            ..Default::default()
        };
        let msg = err(merge_core(LOCA_A, LOCA_B, o));
        for mode in laterite_ags4_merge::TypeClashMode::ALL {
            assert!(
                msg.contains(mode.as_str()),
                "the rejection must list {:?}, got: {msg}",
                mode.as_str()
            );
        }
    }

    #[test]
    fn every_documented_clash_mode_is_accepted() {
        for mode in laterite_ags4_merge::TypeClashMode::ALL {
            let o = MergeOptions {
                on_type_clash: Some(mode.as_str().to_string()),
                ..Default::default()
            };
            // `error` legitimately fails on the clashing pair, so merge two files
            // that do NOT clash — this asserts the TOKEN is accepted.
            assert!(
                merge_core(LOCA_A, LOCA_B, o).is_ok(),
                "documented mode {:?} was rejected",
                mode.as_str()
            );
        }
    }

    #[test]
    fn a_type_clash_is_fatal_by_default_and_widen_settles_it() {
        let strict = merge_core(LOCA_A, LOCA_CLASH, MergeOptions::default());
        assert!(
            strict.is_err(),
            "two files typing LOCA_NATE differently must not merge silently"
        );
        let widened = merge_core(
            LOCA_A,
            LOCA_CLASH,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen settles the clash");
        let text = String::from_utf8(widened.bytes()).expect("utf-8");
        assert!(
            text.contains("BH01") && text.contains("BH09"),
            "both deliveries' rows must survive the widen:\n{text}"
        );
    }

    #[test]
    fn merge_refuses_an_unknown_edition_and_an_unknown_encoding() {
        let bad_dict = MergeOptions {
            dict_version: Some("4.9".into()),
            ..Default::default()
        };
        assert!(merge_core(LOCA_A, LOCA_B, bad_dict).is_err());
        let bad_enc = MergeOptions {
            encoding: Some("klingon-1".into()),
            ..Default::default()
        };
        assert!(merge_core(LOCA_A, LOCA_B, bad_enc).is_err());
    }

    #[test]
    fn a_partial_tran_stops_a_merge() {
        let o = MergeOptions {
            tran: Some(TranInput {
                issue: Some("1".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let msg = err(merge_core(LOCA_A, LOCA_B, o));
        assert!(
            !msg.is_empty(),
            "an incomplete merge-TRAN must be reported, not silently dropped"
        );
    }

    #[test]
    fn the_merge_result_getters_return_what_the_core_built() {
        // These three getters are the entire JS-visible surface of a merge, and
        // the web worker reads them before calling `.free()`. Empty JSON here
        // would look exactly like "nothing to report".
        let res = merge_core(LOCA_A, LOCA_B, MergeOptions::default()).expect("merges");
        assert!(!res.bytes().is_empty(), "the merged file must have bytes");
        // Valid JSON arrays, not the "[]" fallback that a serialisation failure
        // would produce indistinguishably from a genuinely empty audit.
        let w: serde_json::Value =
            serde_json::from_str(&res.warnings_json()).expect("warnings are JSON");
        let r: serde_json::Value =
            serde_json::from_str(&res.revisions_json()).expect("revisions are JSON");
        assert!(w.is_array() && r.is_array());
        assert!(
            r.as_array().is_some_and(|a| !a.is_empty()),
            "BH01 differs between the two deliveries, so a revision must be recorded"
        );
    }

    // ---------------------------------------------------------------
    // dictionary_core
    // ---------------------------------------------------------------

    #[test]
    fn dictionary_defaults_to_the_fallback_edition() {
        let dto = dictionary_core(None).expect("default");
        assert_eq!(
            dto.ags_edition,
            dictionary_core(Some("auto")).unwrap().ags_edition
        );
        assert!(!dto.groups.is_empty(), "the dictionary must have groups");
    }

    #[test]
    fn dictionary_refuses_an_unknown_edition() {
        let msg = err(dictionary_core(Some("4.9")));
        assert!(msg.contains("4.9") || msg.contains("unknown"), "got: {msg}");
    }

    #[test]
    fn each_edition_returns_its_own_dictionary() {
        // A resolver that ignored its argument would return the fallback for
        // every edition and pass any single-edition assertion.
        let a = dictionary_core(Some("4.0.3")).expect("4.0.3");
        let b = dictionary_core(Some("4.2")).expect("4.2");
        assert_ne!(a.ags_edition, b.ags_edition);
        assert!(
            a.groups.len() != b.groups.len()
                || a.groups.iter().map(|g| g.headings.len()).sum::<usize>()
                    != b.groups.iter().map(|g| g.headings.len()).sum::<usize>(),
            "4.0.3 and 4.2 returned identical dictionaries"
        );
    }

    #[test]
    fn a_dictionary_group_carries_the_descriptions_the_reference_ui_needs() {
        // The reason this door exists: the Tools reference used to fetch a
        // scaffolded dictionary where ~91% of headings had EMPTY descriptions.
        let dto = dictionary_core(Some("4.1.1")).expect("4.1.1");
        let loca = dto.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert!(!loca.contents.is_empty(), "the group needs its description");
        assert!(
            loca.headings
                .iter()
                .filter(|h| !h.description.is_empty())
                .count()
                > loca.headings.len() / 2,
            "most headings should carry a description"
        );
    }

    // ---------------------------------------------------------------
    // censor_core
    // ---------------------------------------------------------------

    /// A file carrying one heading from each of the categories the tests below
    /// assert on: a location id (pseudonym), a coordinate (blank) and a project
    /// id (filehash).
    const SENSITIVE_FILE: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"SECRET-PROJECT\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"523145.67\"\r\n";

    #[test]
    fn omitting_selected_codes_scrubs_every_classified_heading() {
        // The leak-safety default, and the one that must never regress: an
        // omitted `selectedCodes` means the WIDER scrub, so forgetting the
        // option over-redacts rather than leaving coordinates in the file.
        let out =
            censor_core(SENSITIVE_FILE, SENSITIVE, CensorOptions::default()).expect("censors");
        assert!(
            !out.text.contains("523145.67"),
            "the coordinate survived the default policy:\n{}",
            out.text
        );
        assert!(
            !out.text.contains("SECRET-PROJECT"),
            "the project id survived the default policy:\n{}",
            out.text
        );
    }

    #[test]
    fn selected_codes_restricts_the_policy_to_those_columns() {
        let out = censor_core(
            SENSITIVE_FILE,
            SENSITIVE,
            CensorOptions {
                selected_codes: Some(vec!["LOCA_NATE".to_string()]),
                ..Default::default()
            },
        )
        .expect("censors");
        assert!(
            !out.text.contains("523145.67"),
            "the selected coordinate must still be scrubbed"
        );
        assert!(
            out.text.contains("SECRET-PROJECT"),
            "an unselected heading must be left alone:\n{}",
            out.text
        );
    }

    #[test]
    fn proj_id_becomes_the_full_sha256_of_the_input_bytes() {
        // Full 64 hex, not a prefix: PROJ_ID is a KEY field, so the width is what
        // makes a collision cryptographically nil rather than merely unlikely.
        let out =
            censor_core(SENSITIVE_FILE, SENSITIVE, CensorOptions::default()).expect("censors");
        let expected = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(SENSITIVE_FILE));
        assert_eq!(expected.len(), 64);
        assert!(
            out.text.contains(&expected),
            "PROJ_ID should be the file's own SHA-256:\n{}",
            out.text
        );
    }

    #[test]
    fn the_replacement_token_defaults_and_can_be_overridden() {
        let custom = censor_core(
            SENSITIVE_FILE,
            SENSITIVE,
            CensorOptions {
                token: Some("***".to_string()),
                ..Default::default()
            },
        )
        .expect("censors");
        assert!(
            !custom.text.contains("[REDACTED]"),
            "a custom token must replace the default entirely"
        );
    }

    #[test]
    fn a_malformed_classification_document_is_refused() {
        // Without a policy nothing would be scrubbed — the one outcome a caller
        // must never get by accident, so it has to be an error rather than an
        // empty policy.
        let msg = err(censor_core(
            SENSITIVE_FILE,
            "{ not valid json",
            CensorOptions::default(),
        ));
        assert!(!msg.is_empty(), "a bad policy document must be reported");
    }

    #[test]
    fn the_tally_counts_what_was_actually_changed() {
        // A tally of zeroes alongside a scrubbed file would tell the Anonymiser's
        // UI that nothing happened.
        let out =
            censor_core(SENSITIVE_FILE, SENSITIVE, CensorOptions::default()).expect("censors");
        let total = out.tally.pseudonym + out.tally.blank + out.tally.token;
        assert!(
            total > 0,
            "the tally reported no changes on a scrubbed file"
        );
    }

    // ---------------------------------------------------------------
    // the Excel pair
    // ---------------------------------------------------------------

    #[test]
    fn an_ags_file_becomes_a_workbook_with_a_sheet_per_group() {
        let res = ags4_to_xlsx_core(CLEAN, false).expect("converts");
        assert!(res.sheets() > 0, "no sheets were written");
        assert!(res.rows() > 0, "no rows were written");
        // The xlsx magic — a zip container. Anything else is not a workbook,
        // however plausible the byte count.
        assert_eq!(&res.bytes()[..2], b"PK", "output must be a zip/xlsx");
        assert!(res.warnings().len() < 100, "warnings should be bounded");
    }

    #[test]
    fn duplicate_headings_are_fatal_unless_recovery_is_asked_for() {
        // Fatal by default on every read surface; the browser opts into the
        // suffixed recovery read. Both halves matter — a default that recovered
        // would silently invent column names.
        let dup: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"BH01\",\"BH01\"\r\n";
        assert!(
            ags4_to_xlsx_core(dup, false).is_err(),
            "duplicate headings must be fatal by default"
        );
        assert!(
            ags4_to_xlsx_core(dup, true).is_ok(),
            "recovery must be reachable from the browser"
        );
    }

    #[test]
    fn non_workbook_bytes_are_refused() {
        assert!(xlsx_to_ags4_core(b"not a workbook", false).is_err());
    }

    #[test]
    fn a_workbook_round_trips_back_to_the_same_groups() {
        // The pair is only useful if it is a pair: converting out and back must
        // preserve the groups, not merely produce *some* AGS4.
        let book = ags4_to_xlsx_core(CLEAN, false).expect("to xlsx");
        let back = xlsx_to_ags4_core(&book.bytes(), false).expect("from xlsx");
        let text = String::from_utf8(back.bytes()).expect("utf-8");
        for group in ["PROJ", "TRAN", "UNIT", "TYPE"] {
            assert!(
                text.contains(&format!("\"GROUP\",\"{group}\"")),
                "{group} did not survive the round trip:\n{text}"
            );
        }
    }

    // ---------------------------------------------------------------
    // certify_core's error arms
    // ---------------------------------------------------------------

    #[test]
    fn certify_requires_the_caller_to_supply_the_clock() {
        // wasm has no clock, so `checkedAt` is the one option with no default.
        // The message has to say why, or a caller reads "required" and guesses.
        let msg = err(certify_core(CLEAN, &CertifyOptions::default()));
        assert!(
            msg.contains("checkedAt"),
            "the missing option must be named, got: {msg}"
        );
        assert!(
            msg.contains("clock") || msg.contains("toISOString"),
            "the message should explain why the caller supplies it, got: {msg}"
        );
    }

    #[test]
    fn certify_refuses_an_unknown_edition_and_an_unknown_encoding() {
        let bad_dict = CertifyOptions {
            checked_at: Some("2020-01-01T00:00:00Z".into()),
            dict_version: Some("4.9".into()),
            ..Default::default()
        };
        assert!(certify_core(CLEAN, &bad_dict).is_err());
        let bad_enc = CertifyOptions {
            checked_at: Some("2020-01-01T00:00:00Z".into()),
            encoding: Some("klingon-1".into()),
            ..Default::default()
        };
        assert!(certify_core(CLEAN, &bad_enc).is_err());
    }

    #[test]
    fn certify_refuses_a_file_with_errors() {
        // Warnings and FYI findings are measured and recorded; ERRORS are fatal.
        // A certificate over a broken file is exactly the claim the trust model
        // exists to prevent.
        assert!(
            certify_core(b"not an ags file", &CertifyOptions::default()).is_err(),
            "an unparseable file must not certify"
        );
    }
}

#[cfg(test)]
mod run_and_overlay_tests {
    //! `run()` and `build_custom_dict()` — plain functions all along, and the
    //! only two places in this crate where a caller mistake has to come back as
    //! DATA rather than an exception.
    //!
    //! `validate` is infallible by contract: a bad edition, a bad encoding, an
    //! unparseable dictionary and a file that is not AGS4 all arrive as
    //! `report.error = {kind, message}`, because the UI already renders that
    //! channel and a thrown exception would have to be caught somewhere else.
    //! Which `kind` each one produces is the part a consumer switches on, and
    //! none of the four arms was exercised.
    use super::core_door_tests::{LOCA_A, LOCA_B};
    use super::*;

    const DELIVERY: &[u8] = include_bytes!(
        "../../laterite-ags4-validator/tests/fixtures/custom_dict/delivery_with_xtra.ags"
    );
    const DICT_JSON: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/custom_dict/xtra.dict.json");

    fn kind_of(r: &ValidationReport) -> String {
        r.error.as_ref().map(|e| e.kind.clone()).unwrap_or_default()
    }

    #[test]
    fn a_bad_edition_is_reported_not_thrown() {
        let r = run(
            LOCA_A,
            &ValidateOptions {
                dict_version: Some("4.9".into()),
                ..Default::default()
            },
        );
        assert!(!r.ok);
        assert_eq!(kind_of(&r), "bad_args");
        assert!(
            r.error.as_ref().is_some_and(|e| e.message.contains("4.9")),
            "the rejected edition must appear in the message"
        );
    }

    #[test]
    fn a_bad_encoding_is_reported_rather_than_falling_back_to_utf8() {
        // The caller SEES the bad label instead of receiving findings that are
        // artefacts of a silent UTF-8 fallback — findings which would look
        // exactly like real ones.
        let r = run(
            LOCA_A,
            &ValidateOptions {
                encoding: Some("klingon-1".into()),
                ..Default::default()
            },
        );
        assert_eq!(kind_of(&r), "bad_args");
        assert_eq!(
            r.finding_count, 0,
            "no findings may be invented on a failure"
        );
    }

    #[test]
    fn an_unparseable_dictionary_is_the_dictionarys_problem() {
        // A distinct kind from `bad_args` on purpose: the delivery is fine and
        // the DICTIONARY is broken, and the UI points at a different input.
        let r = run(
            DELIVERY,
            &ValidateOptions {
                dictionary: Some(b"{ not a dictionary".to_vec()),
                ..Default::default()
            },
        );
        assert_eq!(kind_of(&r), "bad_dict");
    }

    #[test]
    fn a_file_that_is_not_ags4_is_classified_as_such() {
        let r = run(
            b"nothing resembling ags4 at all",
            &ValidateOptions::default(),
        );
        assert_eq!(kind_of(&r), "not_ags4");
        assert!(r.dict_version.is_empty(), "no edition was judged");
    }

    #[test]
    fn a_failure_report_carries_the_full_shape_a_consumer_reads() {
        // Every surface returns the same report shape; a failure that omitted
        // fields would make the browser the one door a consumer had to special-case.
        let r = run(b"junk", &ValidateOptions::default());
        assert!(!r.ok);
        assert_eq!(r.finding_count, 0);
        assert_eq!(r.shown_count, 0);
        assert!(r.findings.is_empty());
        assert!(
            r.revalidate_reason.is_none(),
            "this surface has no cert-consume door, so it is structurally null"
        );
    }

    #[test]
    fn warnings_are_on_and_fyi_off_by_default() {
        // Python and Node both promise warnings ON. A plain `Option<bool>`
        // unwrapping to false here would have made the browser the one surface
        // that quietly disagreed, and no test would have noticed.
        let defaults = run(DELIVERY, &ValidateOptions::default());
        let explicit_off = run(
            DELIVERY,
            &ValidateOptions {
                warnings: Some(false),
                ..Default::default()
            },
        );
        let with_fyi = run(
            DELIVERY,
            &ValidateOptions {
                fyi: Some(true),
                ..Default::default()
            },
        );
        assert!(
            defaults.finding_count >= explicit_off.finding_count,
            "warnings must be included by default"
        );
        assert!(
            with_fyi.finding_count >= defaults.finding_count,
            "fyi must be excluded by default"
        );
    }

    #[test]
    fn max_per_rule_caps_what_crosses_the_boundary_not_what_was_found() {
        // `finding_count` is the true total and `shown_count` is what was
        // serialised, so the UI can say "showing N of M". A cap that also
        // reduced the total would under-report the state of the file.
        let uncapped = run(
            DELIVERY,
            &ValidateOptions {
                warnings: Some(true),
                fyi: Some(true),
                ..Default::default()
            },
        );
        let capped = run(
            DELIVERY,
            &ValidateOptions {
                warnings: Some(true),
                fyi: Some(true),
                max_per_rule: Some(1),
                ..Default::default()
            },
        );
        assert_eq!(
            capped.finding_count, uncapped.finding_count,
            "the true total must survive the cap"
        );
        assert!(
            capped.shown_count <= uncapped.shown_count,
            "the cap must reduce what is serialised"
        );
        assert_eq!(
            capped.shown_count,
            capped.findings.iter().map(|g| g.items.len()).sum::<usize>(),
            "shown_count must equal what is actually in the payload"
        );
        for g in &capped.findings {
            assert!(g.items.len() <= 1, "rule {:?} exceeded the cap", g.rule);
            assert!(
                g.total >= g.items.len(),
                "the per-rule total must be the true count"
            );
        }
    }

    #[test]
    fn a_cell_targeted_finding_serialises_its_target() {
        // `target` drives the browser's cell highlight, and it is omitted for
        // the whole-line default — so a mis-mapped variant is invisible until a
        // user clicks a finding and lands on the wrong thing.
        let bad_type: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"not a number\"\r\n";
        let r = run(
            bad_type,
            &ValidateOptions {
                warnings: Some(true),
                fyi: Some(true),
                ..Default::default()
            },
        );
        let targets: Vec<&str> = r
            .findings
            .iter()
            .flat_map(|g| &g.items)
            .filter_map(|f| f.target.as_deref())
            .collect();
        assert!(
            targets.contains(&"cell"),
            "a value that does not match its TYPE is a CELL finding, got {targets:?}"
        );
    }

    /// A SELF-CONTAINED dictionary — every group roots itself. A replacement has
    /// no bundled edition behind it, so anything it references it must define.
    const STANDALONE_DICT: &[u8] = br#"{
      "groups": {
        "PROJ": {
          "parent": null,
          "description": "Project",
          "headings": [
            {"name": "PROJ_ID", "type": "ID", "status": "KEY"}
          ]
        },
        "LOCA": {
          "parent": null,
          "description": "Location",
          "headings": [
            {"name": "LOCA_ID",   "type": "ID",  "status": "KEY"},
            {"name": "LOCA_NATE", "type": "2DP", "status": "OTHER", "unit": "m"}
          ]
        }
      }
    }"#;

    #[test]
    fn a_custom_dictionary_can_replace_the_bundled_one_outright() {
        // `dictReplace` is a different question from `dictVersion`: replace says
        // "this file IS the dictionary", force says "judge against edition X".
        // The replace branch had never been taken by any test.
        let r = run(
            LOCA_A,
            &ValidateOptions {
                dictionary: Some(STANDALONE_DICT.to_vec()),
                dict_replace: Some(true),
                ..Default::default()
            },
        );
        assert_ne!(
            kind_of(&r),
            "bad_dict",
            "a self-contained dictionary must be accepted as a replacement: {:?}",
            r.error.as_ref().map(|e| e.message.clone())
        );
    }

    #[test]
    fn a_delta_dictionary_cannot_stand_in_as_a_whole_replacement() {
        // The realistic mistake: `xtra.dict.json` hangs a bespoke group off the
        // standard `SAMP`, which is exactly right as an OVERLAY and incoherent
        // as a REPLACEMENT — under replace there is no bundled edition, so
        // `SAMP` is undefined. Refusing it by name beats validating a delivery
        // against a dictionary with a dangling parent.
        let r = run(
            DELIVERY,
            &ValidateOptions {
                dictionary: Some(DICT_JSON.to_vec()),
                dict_replace: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(kind_of(&r), "bad_dict");
        let msg = r
            .error
            .as_ref()
            .map(|e| e.message.clone())
            .unwrap_or_default();
        assert!(
            msg.contains("XTRA") && msg.contains("SAMP"),
            "the message must name both the group and the parent it cannot find, got: {msg}"
        );
    }

    #[test]
    fn replacing_and_forcing_the_dictionary_at_once_is_a_contradiction() {
        // A forced base and a full replacement cannot both be honoured, so the
        // combination is refused by name rather than one silently winning.
        let r = run(
            DELIVERY,
            &ValidateOptions {
                dictionary: Some(DICT_JSON.to_vec()),
                dict_replace: Some(true),
                dict_version: Some("4.1.1".into()),
                ..Default::default()
            },
        );
        assert_eq!(kind_of(&r), "bad_dict");
        let msg = r
            .error
            .as_ref()
            .map(|e| e.message.clone())
            .unwrap_or_default();
        assert!(
            msg.contains("dict_replace") && msg.contains("dict_version"),
            "both halves of the contradiction must be named, got: {msg}"
        );
    }

    #[test]
    fn build_custom_dict_is_a_no_op_without_bytes() {
        let none = build_custom_dict(None, false, None, encoding_rs::UTF_8).expect("no dict");
        assert!(
            none.is_none(),
            "no bytes means no overlay, not an empty one"
        );
    }

    // ---------------------------------------------------------------
    // The remaining gaps in the emit + merge tails
    // ---------------------------------------------------------------

    #[test]
    fn a_complete_tran_without_the_optional_extras_still_folds() {
        // The `None => s` arms: description and remarks are genuinely optional,
        // so the five-field stamp has to survive their absence. Both arms were
        // dark because every existing test supplied them.
        let five_only = TranInput {
            issue: Some("1".into()),
            date: Some("2020-08-18".into()),
            producer: Some("ACME Drilling".into()),
            recipient: Some("ACME Consulting".into()),
            status: Some("FINAL".into()),
            description: None,
            remarks: None,
        };
        let stamp = five_only.fold().expect("folds").expect("some");
        let rendered = format!("{stamp:?}");
        assert!(
            rendered.contains("ACME Drilling"),
            "the five required fields must survive: {rendered}"
        );
    }

    #[test]
    fn a_tran_stamp_reaches_the_emitted_file() {
        // The fold is only worth anything if the stamp lands in the output —
        // otherwise "all five or none" guards a value that is then dropped.
        let o = BuildOptions {
            synthesise_metadata: Some(true),
            tran: Some(TranInput {
                issue: Some("7".into()),
                date: Some("2020-08-18".into()),
                producer: Some("ACME Drilling".into()),
                recipient: Some("ACME Consulting".into()),
                status: Some("FINAL".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let report = build_ags4_core(
            r#"[{"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]}]"#,
            o,
        )
        .expect("builds");
        assert!(
            report.text.contains("ACME Drilling") && report.text.contains("\"GROUP\",\"TRAN\""),
            "the supplied transmission must be written:\n{}",
            report.text
        );
    }

    #[test]
    fn emit_findings_are_all_errors_which_is_why_the_severity_field_is_absent() {
        // `EmitFinding.severity` is `skip_serializing_if = is_none` and its TS
        // documents "absent means error". The emit path produces ONLY error-
        // severity findings today, so the non-error arm of that mapping is
        // unreachable rather than untested — asserted here so that the day emit
        // grows a warning, this fails and someone revisits the mapping instead
        // of discovering it in a browser.
        let report = build_ags4_core(
            r#"[{"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],"rows":[["P1","x"]]}]"#,
            BuildOptions {
                mode: Some("report".into()),
                ..Default::default()
            },
        )
        .expect("builds");
        assert!(
            !report.findings.is_empty(),
            "a data-only build must report the missing mandatory groups"
        );
        assert!(
            report.findings.iter().all(|f| f.severity.is_none()),
            "emit produced a non-error severity — the mapping arm is now live"
        );
    }

    /// `LOCA_A` typed `2DP`; this types the same heading `3DP` — a clash
    /// entirely inside the nDP family, which is what `promote` can join.
    const LOCA_3DP: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"3DP\"\r\n\
\"DATA\",\"BH09\",\"9.123\"\r\n";

    fn warning_kinds(res: &MergeResult) -> Vec<String> {
        let v: serde_json::Value =
            serde_json::from_str(&res.warnings_json()).expect("warnings are JSON");
        v.as_array()
            .expect("array")
            .iter()
            .filter_map(|w| w.get("kind").and_then(|k| k.as_str()).map(str::to_string))
            .collect()
    }

    #[test]
    fn a_widened_type_clash_is_recorded_in_the_merge_warnings() {
        // Widening keeps both deliveries' rows but DISCARDS the column's type,
        // which is a loss the audit trail has to carry — a silent widen is
        // indistinguishable from the files having agreed.
        let res = merge_core(
            LOCA_A,
            LOCA_3DP,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen settles the clash");
        let kinds = warning_kinds(&res);
        assert!(
            kinds.iter().any(|k| k == "type_widened"),
            "a discarded type must be warned about, got {kinds:?}"
        );
        // The warning has to say WHICH column lost its type, or it is unactionable.
        let v: serde_json::Value = serde_json::from_str(&res.warnings_json()).expect("JSON");
        let widened = v
            .as_array()
            .expect("array")
            .iter()
            .find(|w| w.get("kind").and_then(|k| k.as_str()) == Some("type_widened"))
            .expect("the widen warning");
        assert_eq!(
            widened.get("heading").and_then(|h| h.as_str()),
            Some("LOCA_NATE")
        );
    }

    #[test]
    fn promote_keeps_the_column_numeric_and_says_so() {
        // The other resolution: `{2DP, 3DP}` joins inside the nDP family, so the
        // column stays numeric at the greater precision rather than falling back
        // to raw text.
        let res = merge_core(
            LOCA_A,
            LOCA_3DP,
            MergeOptions {
                on_type_clash: Some("promote".into()),
                ..Default::default()
            },
        )
        .expect("promote settles the clash");
        assert!(
            warning_kinds(&res).iter().any(|k| k == "type_promoted"),
            "a promotion changes what the file asserts and must be recorded"
        );
        let text = String::from_utf8(res.bytes()).expect("utf-8");
        assert!(
            text.contains("\"3DP\""),
            "the column should keep the greatest precision, not widen to X:\n{text}"
        );
    }

    #[test]
    fn a_typed_vs_x_clash_widens_silently_by_design() {
        // NOT a wart — the documented lattice behaviour, verified through the
        // browser door. `TypeClashMode::Widen`'s own doc states it: "Typed-vs-`X`
        // resolves silently (`X` trivially absorbs a typed value); two *different*
        // non-`X` types warn", and laterite-ags4-merge's acceptance suite pins
        // both halves (`typed_vs_x_widen_is_silent`, `non_x_vs_non_x_warns_under_
        // lenient`).
        //
        // The reasoning is that merge made no CHOICE here: once one file declares
        // `X`, the merged column can only be `X` — promote cannot reach it either
        // — so there is no resolution to report. A warning would fire on every
        // typed-vs-freetext column in a real delivery and drown the ones that
        // record an actual decision.
        //
        // Asserted here because a surface can drop a warning its engine emitted:
        // this proves the browser reports exactly what merge reports, silence
        // included.
        let clash: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH09\",\"nine\"\r\n";
        let res = merge_core(
            LOCA_A,
            clash,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen accepts the pair");
        let text = String::from_utf8(res.bytes()).expect("utf-8");
        assert!(
            text.contains("\"ID\",\"X\""),
            "the column widens to X:\n{text}"
        );
        assert!(
            !warning_kinds(&res).iter().any(|k| k == "type_widened"),
            "typed-vs-X is the trivial widen and must stay silent — if merge's \
             lattice changed, change it here and in TypeClashMode's doc together"
        );
    }

    #[test]
    fn merging_two_identical_files_changes_nothing() {
        // The degenerate case, and a useful control on the revision audit: if a
        // file merged with itself reports revisions, "changed" means nothing.
        let res = merge_core(LOCA_A, LOCA_A, MergeOptions::default()).expect("merges");
        let revisions: serde_json::Value =
            serde_json::from_str(&res.revisions_json()).expect("JSON");
        let changed = revisions
            .as_array()
            .expect("array")
            .iter()
            .filter(|r| r.get("changed").and_then(serde_json::Value::as_bool) == Some(true))
            .count();
        assert_eq!(changed, 0, "a file merged with itself has no revisions");
    }

    #[test]
    fn an_explicit_edition_overrides_the_files_own_tran_ags() {
        // The parity gap this option was added to close: Python's `merge` and
        // `lat merge` both take it, and this surface used to hard-code None — so
        // a browser user merging files with a wrong or absent TRAN_AGS had no
        // way to say so. Proven by forcing an edition the inputs do not name.
        let forced = merge_core(
            LOCA_A,
            LOCA_B,
            MergeOptions {
                dict_version: Some("4.0.3".into()),
                ..Default::default()
            },
        )
        .expect("merges against the forced edition");
        assert!(!forced.bytes().is_empty());
    }
}

#[cfg(test)]
mod dictionary_and_target_tests {
    //! The last three plain-Rust arms: forcing an edition UNDER a custom
    //! dictionary, the group-targeted finding, and `certify`'s dictionary half.
    use super::core_door_tests::LOCA_A;
    use super::*;

    const DELIVERY: &[u8] = include_bytes!(
        "../../laterite-ags4-validator/tests/fixtures/custom_dict/delivery_with_xtra.ags"
    );
    const DICT_JSON: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/custom_dict/xtra.dict.json");
    const CLEAN: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

    #[test]
    fn a_custom_dictionary_can_be_overlaid_on_a_forced_edition() {
        // The third `BaseSpec` arm. `dictReplace` is refused alongside
        // `dictVersion` (they contradict), but an OVERLAY on a forced base is
        // coherent and is the combination a consultancy actually uses: our
        // bespoke groups, judged against the edition the client mandated.
        let r = run(
            DELIVERY,
            &ValidateOptions {
                dictionary: Some(DICT_JSON.to_vec()),
                dict_version: Some("4.1.1".into()),
                ..Default::default()
            },
        );
        assert_ne!(
            r.error.as_ref().map(|e| e.kind.clone()).unwrap_or_default(),
            "bad_dict",
            "an overlay on a forced base must be accepted: {:?}",
            r.error.as_ref().map(|e| e.message.clone())
        );
        assert_eq!(
            r.dict_version, "4.1.1",
            "the forced edition must be the one reported"
        );
        assert_eq!(
            r.resolution, "forced",
            "and it must be reported as forced, not guessed"
        );
    }

    #[test]
    fn a_group_targeted_finding_serialises_its_target() {
        // Rule 19: a GROUP name must be exactly four uppercase letters. The
        // finding targets the GROUP rather than a line or a cell, and that
        // variant of the mapping had never been produced.
        let bad_group: &[u8] = b"\"GROUP\",\"loca\"\r\n\
\"HEADING\",\"LOCA_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH01\"\r\n";
        let r = run(
            bad_group,
            &ValidateOptions {
                warnings: Some(true),
                fyi: Some(true),
                ..Default::default()
            },
        );
        let targets: Vec<&str> = r
            .findings
            .iter()
            .flat_map(|g| &g.items)
            .filter_map(|f| f.target.as_deref())
            .collect();
        assert!(
            targets.contains(&"group"),
            "a bad GROUP name is a GROUP finding, got {targets:?}"
        );
    }

    #[test]
    fn certify_reports_a_broken_dictionary_rather_than_certifying_without_it() {
        // Certifying against a dictionary that would not load must FAIL, not
        // quietly fall back to the bundled one — the certificate records which
        // dictionary was used, so a silent fallback would mint a true-looking
        // statement about a validation that never happened.
        let o = CertifyOptions {
            checked_at: Some("2020-01-01T00:00:00Z".into()),
            dictionary: Some(b"{ not a dictionary".to_vec()),
            ..Default::default()
        };
        let msg = match certify_core(CLEAN, &o) {
            Ok(_) => panic!("a broken dictionary must not certify"),
            Err(m) => m,
        };
        assert!(
            msg.contains("bad dict"),
            "the dictionary must be named as the problem, got: {msg}"
        );
    }

    #[test]
    fn a_certificate_records_the_custom_dictionary_it_was_minted_against() {
        // O-48, record-not-contract: the stamp carries the dict's identity so a
        // later `validate --index` on ANY surface re-validates rather than
        // silently vouching when the effective dictionary differs. A cert that
        // forgot which dictionary it used would be trusted against the wrong one.
        //
        // `CLEAN` and not a fixture with findings — a certificate asserts an
        // error-clean validation, so an erroring file cannot mint one at all and
        // both sides of this comparison would be `Err` (which is how the first
        // draft of this test managed to assert nothing at all).
        let plain = certify_core(
            CLEAN,
            &CertifyOptions {
                checked_at: Some("2020-01-01T00:00:00Z".into()),
                ..Default::default()
            },
        )
        .expect("the clean fixture certifies");
        let with_dict = certify_core(
            CLEAN,
            &CertifyOptions {
                checked_at: Some("2020-01-01T00:00:00Z".into()),
                dictionary: Some(DICT_JSON.to_vec()),
                ..Default::default()
            },
        )
        .expect("the clean fixture certifies against an overlay too");

        assert_ne!(
            plain, with_dict,
            "the custom dictionary left no trace in the certificate"
        );
        assert!(
            with_dict.contains("custom-dict"),
            "the advisory dictionary name must be recorded: {with_dict}"
        );
        assert!(
            !plain.contains("custom-dict"),
            "a cert minted without a dictionary must not claim one: {plain}"
        );
    }

    #[test]
    fn certify_refuses_a_parseable_file_that_has_errors() {
        // The trust model's whole point: warnings and FYI findings are measured
        // and recorded, ERRORS are fatal. The message has to say how many, or a
        // user cannot tell "your file is broken" from "certify is broken".
        let msg = match certify_core(
            LOCA_A,
            &CertifyOptions {
                checked_at: Some("2020-01-01T00:00:00Z".into()),
                ..Default::default()
            },
        ) {
            Ok(_) => panic!("a file with error findings must not certify"),
            Err(m) => m,
        };
        assert!(
            msg.contains("error-severity"),
            "the refusal must name what blocked it, got: {msg}"
        );
    }
}

#[cfg(test)]
mod serializer_consistency_tests {
    //! One serializer for the whole crate, asserted against the source.
    //!
    //! serde-wasm-bindgen ships two that differ in what an absent `Option`
    //! becomes: the default writes `undefined`, `json_compatible()` writes
    //! `null`. Both are reachable, neither is wrong, and choosing the wrong one
    //! is invisible from Rust — it surfaces only as a `=== null` check that never
    //! fires in a browser, against a published `.d.ts` that promised `null`.
    //! That is precisely what `build_ags4` and `build_ags4_ipc` did.
    //!
    //! `cargo test` cannot inspect the JS value — that is the boundary — so the
    //! invariant is enforced where it IS visible: the source may not name a
    //! second serializer.

    const SRC: &str = include_str!("lib.rs");

    #[test]
    fn every_serialisation_goes_through_the_json_compatible_serializer() {
        // The default serializer, reached either by constructing it directly or
        // via the free function that wraps it. Neither may appear outside this
        // test's own mentions of them.
        let this_module = SRC
            .find("mod serializer_consistency_tests")
            .expect("this module is in the source it reads");
        for banned in ["serde_wasm_bindgen::to_value(", "Serializer::new()"] {
            let total = SRC.match_indices(banned).count();
            let here = SRC[this_module..].match_indices(banned).count();
            assert_eq!(
                total, here,
                "{banned:?} bypasses `to_js`: it writes `undefined` for an absent \
                 Option where this crate's published .d.ts promises `null`. \
                 Serialise through `to_js` instead."
            );
        }
    }

    #[test]
    fn the_build_doors_serialise_through_to_js() {
        // Belt to the above's braces, and the more direct statement: these two
        // are the doors that regressed, so name them.
        for door in ["pub fn build_ags4(", "pub fn build_ags4_ipc("] {
            let at = SRC.find(door).unwrap_or_else(|| panic!("{door} exists"));
            let body_end = SRC[at..].find("\n}\n").expect("the function ends");
            assert!(
                SRC[at..at + body_end].contains("to_js(&report)"),
                "{door} no longer serialises through to_js"
            );
        }
    }

    #[test]
    fn the_published_ts_still_declares_the_nullable_field_that_caught_this() {
        // `line` is `Option<u32>` on both EmitFinding and AppliedFix and is
        // declared `number | null`. If that declaration ever changes, the
        // serializer choice has to be revisited in the same breath — so fail
        // loudly rather than let the two drift apart again.
        assert_eq!(
            super::TS_BUILD_RESULT
                .matches("line: number | null")
                .count(),
            2,
            "EmitFinding and AppliedFix should each declare a nullable line"
        );
    }
}
