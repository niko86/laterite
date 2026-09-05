//! `build_ags4()` / `build_ags4_ipc()` — the read path reversed.
//!
//! Data in (JSON rows, or columnar Arrow IPC), a valid AGS4 document out, with
//! no server round-trip. Both doors fold their options down to the same parts
//! and run the shared `laterite-ags4-emit` orchestrator, so the browser cannot
//! write a file the native surfaces would have written differently.
use crate::boundary::{TranInput, WasmOptions, decode_opts, to_js};
#[cfg(feature = "arrow")]
use laterite_ags4_validator::Dictionary;
use laterite_ags4_validator::{DictVersion, findings, fixes};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// One group of input data, deserialised from the browser's JSON. The
/// column `headings` are the AGS headings; `units`/`types` are optional
/// per-heading overrides (the dictionary fills the rest); `rows` cells land
/// directly on the emit-side [`laterite_ags4_emit::Cell`] — its deserialiser
/// maps each JSON scalar to a variant with no `serde_json::Value`
/// intermediate, so the browser door never builds one (#790).
#[derive(Deserialize)]
struct GroupInputJson {
    code: String,
    headings: Vec<String>,
    #[serde(default)]
    units: Option<Vec<String>>,
    #[serde(default)]
    types: Option<Vec<String>>,
    rows: Vec<Vec<laterite_ags4_emit::Cell>>,
}

/// One emit finding, flattened with its rule label for the JS side.
#[derive(Serialize)]
pub(crate) struct EmitFinding {
    pub(crate) rule: String,
    pub(crate) line: Option<u32>,
    pub(crate) group: String,
    pub(crate) desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) severity: Option<String>,
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
pub(crate) struct AppliedFix {
    pub(crate) kind: fixes::FixKind,
    pub(crate) label: String,
    pub(crate) rule: String,
    pub(crate) line: Option<u32>,
    pub(crate) risk: fixes::FixRisk,
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
pub(crate) struct BuildAgs4Report {
    pub(crate) text: String,
    pub(crate) findings: Vec<EmitFinding>,
    pub(crate) applied: Vec<AppliedFix>,
    pub(crate) fixes_applied: usize,
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
      | "pad_short_row" | "quote_unquoted_row";
  label: string;
  /** The exact rule label (`"AGS Format Rule 8"`, …), for cross-linking back
   *  to the originating finding. */
  rule: string;
  line: number | null;
  /** `safe` rewrites are unambiguous from the file alone; `risky` ones guess
   *  intent and are excluded from bulk apply. */
  risk: "safe" | "risky";
}

/** What a build returns — from `build_ags4`, and from `build_ags4_ipc` where
 *  that feature is built. */
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

// Both parsers are `laterite_ags4_hostopts` (#923) — the one copy of the
// option normalisation every surface shares — narrowed to this boundary's
// message-only error shape.
fn emit_edition(s: Option<&str>) -> Result<DictVersion, String> {
    laterite_ags4_hostopts::edition_or_fallback(s).map_err(|e| e.message)
}

fn emit_mode(s: Option<&str>) -> Result<laterite_ags4_emit::EmitMode, String> {
    laterite_ags4_hostopts::write_mode(s).map_err(|e| e.message)
}

/// `build_ags4` / `build_ags4_ipc`'s named options.
///
/// `tran` is a NESTED struct rather than a `JsValue`, so serde builds it
/// directly and `TranInput::fold` applies the shared completeness rule. See
/// that method for why the nested object does not need its own key guard.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BuildOptions {
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

/** Named options for `build_ags4` — and for `build_ags4_ipc` where that
 *  feature is built. */
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

/// Options for the unchecked door (#881): `dictVersion` ONLY. The judge-coupled
/// knobs are not merely omitted from the TS type — `decode_opts` enumerates
/// `KEYS`, so a caller passing `mode`/`synthesiseMetadata`/`tran` here is
/// refused at runtime rather than silently ignored: there is no verdict for a
/// mode to act on, and synthesis fills gaps only a report would surface.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct UncheckedBuildOptions {
    dict_version: Option<String>,
}

impl WasmOptions for UncheckedBuildOptions {
    const KEYS: &'static [&'static str] = &["dictVersion"];
    const WHAT: &'static str = "unchecked build options";
}

#[wasm_bindgen(typescript_custom_section)]
const TS_UNCHECKED_BUILD_OPTIONS: &'static str = r#"
/** Options for `build_ags4_unchecked` — `dictVersion` only. The judged door's
 *  `mode` / `synthesiseMetadata` / `tran` are gone, not defaulted: there is no
 *  verdict for a mode to act on, and synthesis fills gaps only a report would
 *  surface. Passing them is refused at runtime, never silently ignored. */
export interface UncheckedBuildOptions {
  /** The edition to write against. `"auto"` (or omitted) uses the standard. */
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "UncheckedBuildOptions")]
    pub type UncheckedBuildOptionsJs;
}

/// The JSON door's decode, shared by the judged and unchecked cores (#881):
/// one mapping from the browser's `{code, headings, units?, types?, rows}`
/// array to the engine's `GroupInput`s, so the doors cannot drift at the
/// input.
fn groups_from_json(groups_json: &str) -> Result<Vec<laterite_ags4_emit::GroupInput>, String> {
    let parsed: Vec<GroupInputJson> =
        serde_json::from_str(groups_json).map_err(|e| format!("invalid groups JSON: {e}"))?;
    Ok(parsed
        .into_iter()
        .map(|g| laterite_ags4_emit::GroupInput {
            code: g.code,
            headings: g.headings,
            units: g.units,
            types: g.types,
            rows: g.rows,
        })
        .collect())
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
    emit_report(
        groups_from_json(groups_json)?,
        edition,
        mode,
        synthesise_metadata,
        tran,
    )
}

/// The host-testable core of [`build_ags4_unchecked`] (#881): the same JSON
/// decode and the same edition resolution as the judged door, handed to the
/// engine's judge-free entry. Bytes out, nothing validated — pinned
/// byte-identical to the judged `report` build by test, exactly the #858
/// contract Python ships.
pub(crate) fn build_ags4_unchecked_core(
    groups_json: &str,
    edition: Option<&str>,
) -> Result<Vec<u8>, String> {
    let groups = groups_from_json(groups_json)?;
    laterite_ags4_emit::emit_ags4_unchecked(groups, emit_edition(edition)?)
        .map_err(|e| e.to_string())
}

/// Run the shared orchestrator over already-built `GroupInput`s and shape the
/// JS report — the JSON door's tail. The Arrow IPC door shares the opts
/// assembly and shaping ([`emit_opts`], [`shape_report`]) but enters the
/// engine through its own streaming call.
fn emit_report(
    groups: Vec<laterite_ags4_emit::GroupInput>,
    edition: Option<&str>,
    mode: Option<&str>,
    synthesise_metadata: bool,
    tran: Option<laterite_ags4_emit::TranStamp>,
) -> Result<BuildAgs4Report, String> {
    let opts = emit_opts(edition, mode, synthesise_metadata, tran)?;
    let res = laterite_ags4_emit::emit_ags4(&groups, &opts).map_err(|e| e.to_string())?;
    Ok(shape_report(&res))
}

/// The browser's `EmitOpts` — one assembly for both doors.
fn emit_opts(
    edition: Option<&str>,
    mode: Option<&str>,
    synthesise_metadata: bool,
    tran: Option<laterite_ags4_emit::TranStamp>,
) -> Result<laterite_ags4_emit::EmitOpts, String> {
    Ok(laterite_ags4_emit::EmitOpts {
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
    })
}

/// `EmitResult` → the JS-facing report — shared by both doors, so the browser
/// cannot grow two report shapes.
fn shape_report(res: &laterite_ags4_emit::EmitResult) -> BuildAgs4Report {
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
    BuildAgs4Report {
        text: String::from_utf8_lossy(&res.bytes).into_owned(),
        findings,
        applied,
        fixes_applied: res.fixes_applied,
    }
}

/// Decode one group's Arrow IPC stream → a [`laterite_ags4_emit::ArrowGroup`]
/// (the column names are the AGS headings) for the shared streaming door.
#[cfg(feature = "arrow")]
fn group_from_ipc(code: String, bytes: &[u8]) -> Result<laterite_ags4_emit::ArrowGroup, String> {
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
        return Ok(laterite_ags4_emit::ArrowGroup {
            code,
            schema: pschema,
            batches: pbatches,
            units: None,
            types: None,
        });
    }
    // The door renders a typed temporal column at the precision its heading's
    // declared UNIT asks for, from the opts' edition (#695). The browser must
    // answer like Python and Node.
    Ok(laterite_ags4_emit::ArrowGroup {
        code,
        schema,
        batches,
        units: None,
        types: None,
    })
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
pub(crate) fn build_ags4_core(
    groups_json: &str,
    o: BuildOptions,
) -> Result<BuildAgs4Report, String> {
    let (edition, mode, synth, tran) = build_parts(o)?;
    build_ags4_from_json(
        groups_json,
        edition.as_deref(),
        mode.as_deref(),
        synth,
        tran,
    )
}

/// The one-call reference for [`build_ags4_ipc`]'s per-group session loop
/// (#905): same parts, same emit, driven over an already-decoded `Vec`. The
/// differential test holds the two shapes together; nothing ships through
/// this path any more, so it is test-only.
#[cfg(all(test, feature = "arrow"))]
fn build_ipc_core(
    inputs: Vec<laterite_ags4_emit::ArrowGroup>,
    o: BuildOptions,
) -> Result<BuildAgs4Report, String> {
    let (edition, mode, synth, tran) = build_parts(o)?;
    let opts = emit_opts(edition.as_deref(), mode.as_deref(), synth, tran)?;
    let res = laterite_ags4_emit::emit_ags4_from_arrow(inputs, &opts).map_err(|e| e.to_string())?;
    Ok(shape_report(&res))
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

/// [`build_ags4`] with NO validity verdict — the browser's half of the #858
/// unchecked pair (#881), taking the same `groups_json` and returning raw
/// bytes.
///
/// The caller is choosing to ship unchecked bytes: nothing here confirms the
/// output satisfies any AGS4 rule, and nothing downstream will. The bytes are
/// **identical to the judged `"report"` build's** (same dictionary fills, same
/// canonical formatting, same order — pinned by test); what is removed is the
/// rule engine, which is most of what the judged call spends its time on (the
/// decomposition is on #858). No `mode` / `synthesiseMetadata` / `tran`:
/// passing them is refused, never silently ignored. Returns a `Uint8Array`
/// (UTF-8, CRLF) — bytes being the universal output form is what lets this
/// door exist in a browser at all.
#[wasm_bindgen]
pub fn build_ags4_unchecked(
    groups_json: &str,
    opts: Option<UncheckedBuildOptionsJs>,
) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    let o: UncheckedBuildOptions =
        decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    build_ags4_unchecked_core(groups_json, o.dict_version.as_deref()).map_err(|e| JsError::new(&e))
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
///
/// Behind the `arrow` feature: a build without it still writes AGS4 through
/// [`build_ags4`], which takes the same data as JSON.
#[cfg(feature = "arrow")]
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
    // Decode each group as the streamed emit consumes it (#905, queue M9):
    // a whole-file `Vec<ArrowGroup>` here was the door's entire prize at its
    // gate rung — one group's IPC copy, batches and formatted cells are live
    // at a time, and they drop inside `push`. The one-call `build_ipc_core`
    // stays as the differential tests' reference for this loop.
    let (edition, mode, synth, tran) = build_parts(o).map_err(|e| JsError::new(&e))?;
    let eopts = emit_opts(edition.as_deref(), mode.as_deref(), synth, tran)
        .map_err(|e| JsError::new(&e))?;
    let dict = Dictionary::bundled(eopts.edition);
    let mut session = laterite_ags4_emit::ArrowEmitSession::new(&eopts, &dict);
    let arr = js_sys::Array::from(&groups);
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
        session
            .push(group_from_ipc(code, &ipc).map_err(|e| JsError::new(&e))?)
            .map_err(|e| JsError::new(&e.to_string()))?;
    }
    let res = session.finish().map_err(|e| JsError::new(&e.to_string()))?;
    let report = shape_report(&res);
    to_js(&report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::err;
    use laterite_ags4_parse::parse_bytes;
    use laterite_ags4_validator::dict::FALLBACK;

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

    /// #881: the browser's unchecked door returns exactly the judged `report`
    /// build's bytes — the same #858 contract Python pins. Held on a clean
    /// build AND a dirty one (a data-only build with a non-canonical string,
    /// reporting its missing catalogs): the dirty case is where a judge
    /// shaping the bytes would show, and asserting it demonstrably draws
    /// findings keeps the identity falsifiable.
    #[test]
    fn unchecked_bytes_equal_the_judged_report_bytes() {
        let clean = r#"[
          {"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],"rows":[["P1","Demo"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01",12.3]]}
        ]"#;
        let judged =
            build_ags4_from_json(clean, Some("4.1.1"), Some("report"), false, None).unwrap();
        assert_eq!(
            build_ags4_unchecked_core(clean, Some("4.1.1")).unwrap(),
            judged.text.as_bytes(),
            "clean build: unchecked bytes must be the judged report text"
        );

        let dirty = r#"[
          {"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}
        ]"#;
        let judged =
            build_ags4_from_json(dirty, Some("4.1.1"), Some("report"), false, None).unwrap();
        assert!(
            !judged.findings.is_empty(),
            "the dirty fixture must draw findings, or the identity proves nothing"
        );
        let bytes = build_ags4_unchecked_core(dirty, Some("4.1.1")).unwrap();
        assert_eq!(
            bytes,
            judged.text.as_bytes(),
            "dirty build: the judge must not have been shaping the bytes"
        );
    }

    /// #881: a zero-group build refuses identically through both doors — the
    /// refusal lives in the shared assembly, upstream of the judge, and must
    /// not drift between them on this surface either.
    #[test]
    fn unchecked_zero_group_refusal_matches_the_judged_door() {
        let Err(judged) = build_ags4_from_json("[]", None, Some("report"), false, None) else {
            panic!("a zero-group judged build must refuse");
        };
        let Err(unchecked) = build_ags4_unchecked_core("[]", None) else {
            panic!("a zero-group unchecked build must refuse");
        };
        assert_eq!(unchecked, judged);
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

    #[cfg(feature = "arrow")]
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

        // Decode each IPC stream into ArrowGroups, then run the Arrow door.
        let proj = group_from_ipc("PROJ".into(), &ipc_bytes(&proj_schema, &proj_batch)).unwrap();
        let loca = group_from_ipc("LOCA".into(), &ipc_bytes(&loca_schema, &loca_batch)).unwrap();
        let opts = emit_opts(Some("4.1.1"), Some("autofix"), false, None).unwrap();
        let r = shape_report(
            &laterite_ags4_emit::emit_ags4_from_arrow(vec![proj, loca], &opts).unwrap(),
        );

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
    #[cfg(feature = "arrow")]
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

    #[cfg(feature = "arrow")]
    #[test]
    fn non_ipc_bytes_are_reported_as_an_arrow_error() {
        let msg = err(group_from_ipc("LOCA".into(), b"definitely not arrow"));
        assert!(
            msg.contains("arrow ipc"),
            "the caller needs to know it was the IPC decode, got: {msg}"
        );
    }

    #[cfg(feature = "arrow")]
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
        use arrow::array::StringArray;
        let group = group_from_ipc("LOCA".into(), &ipc).expect("decodes");
        let names: Vec<&str> = group
            .schema
            .fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect();
        assert_eq!(
            names,
            ["LOCA_ID", "LOCA_NATE"],
            "underscore-prefixed columns must be dropped"
        );
        // And the values must stay aligned with the columns that survived — a
        // projection that dropped the schema entry but not the data would put
        // "u-1" under LOCA_ID.
        let col = group.batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("utf8");
        assert_eq!(col.value(0), "BH01");
        let col = group.batches[0]
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("utf8");
        assert_eq!(col.value(0), "100.00");
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn a_frame_with_no_underscore_columns_is_passed_through_whole() {
        // The other side of the same branch: when nothing needs dropping the
        // projection is skipped entirely, and the columns must be unchanged.
        let ipc = ipc_of(&["LOCA_ID", "LOCA_NATE"], &[&["BH01", "100.00"]]);
        let group = group_from_ipc("LOCA".into(), &ipc).expect("decodes");
        let names: Vec<&str> = group
            .schema
            .fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect();
        assert_eq!(names, ["LOCA_ID", "LOCA_NATE"]);
        assert_eq!(group.batches[0].num_rows(), 1);
    }

    #[cfg(feature = "arrow")]
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
}
