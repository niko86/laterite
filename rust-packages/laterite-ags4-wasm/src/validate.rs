//! `validate()` — the whole rule engine, client-side, over in-memory bytes.
//!
//! Infallible by contract: a bad edition, a bad encoding, an unparseable
//! dictionary and a file that is not AGS4 all come back as
//! `report.error = {kind, message}` rather than as a thrown exception, because
//! the UI already renders that channel. The report shapes below are what a
//! consumer reads, and the TS beside them is what it reads them through.
use crate::boundary::{WasmOptions, decode_opts};
use crate::resolve::{build_custom_dict, classify, resolve_dict_override, resolve_encoding};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::{
    CheckOptions, ValidatorError, WorldScope, check_parsed_with_dict, findings, verdict::Verdict,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// One rule violation — mirrors the CLI's `{line, group, desc}` JSON,
/// plus the additive rule-aware location/severity fields (omitted when
/// unset so the base shape is unchanged). `target`/`severity` use
/// snake_case to match the engine's serde rename + the TS interface.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct FindingDto {
    pub(crate) line: Option<u32>,
    pub(crate) group: String,
    pub(crate) desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) field_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) heading: Option<String>,
    /// 1-based row ordinal within the group (distinct from `line`); set
    /// for data-row findings so the UI can address the exact cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data_row: Option<u32>,
    /// Half-open `[start, end)` char-offset span within the raw line —
    /// either carried by the finding (Rules 1/6) or computed here from
    /// `field_index` so every cell/heading finding gets a precise span.
    /// Serialized as a 2-element array to match the TS `[number, number]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) char_span: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) severity: Option<String>,
}

/// Findings for a single rule. The report flattens the engine's
/// `BTreeMap<rule, Vec<Finding>>` into an ordered array of these so the
/// UI can render one collapsible section per rule without re-sorting
/// (the engine already orders by rule label).
#[derive(Serialize)]
pub(crate) struct RuleGroup {
    pub(crate) rule: String,
    /// True number of findings for this rule, **before** any serialization
    /// cap. `items.len()` may be smaller when `max_per_rule` clips the tail;
    /// the UI shows "N of `total`" so a cap never hides the real count.
    pub(crate) total: usize,
    pub(crate) items: Vec<FindingDto>,
}

/// An un-validatable input (not a rule violation). `kind` is a stable
/// machine token the UI switches on; `message` is the human string.
#[derive(Serialize)]
pub(crate) struct ValErr {
    pub(crate) kind: String,
    pub(crate) message: String,
}

/// The whole result of a validation run. `ok` is the verdict
/// (`error.is_none() && finding_count == 0`); when `error` is set the
/// other fields are empty/zero and no rules ran.
#[derive(Serialize)]
pub(crate) struct ValidationReport {
    pub(crate) ok: bool,
    /// The bundled edition the file was judged against (`"4.1.1"`, …),
    /// empty on error.
    pub(crate) dict_version: String,
    /// How that edition was chosen: `forced` / `exact` / `guessed` /
    /// `fallback` (see `DictResolution`), empty on error.
    pub(crate) resolution: String,
    /// True total across every rule — always the full count the engine
    /// found, independent of any serialization cap.
    pub(crate) finding_count: usize,
    /// The verdict (#321). Not `finding_count == 0`: a warning is shown by
    /// default and does not fail, so a file can be `valid` with findings.
    /// `ok` above is this surface's historical spelling of the same answer and
    /// now follows it; the duplication is deliberate for one release so the
    /// browser consumer can migrate off `ok`.
    pub(crate) valid: bool,
    /// Per-tier counts. They sum to `finding_count`, and let the UI colour a
    /// report without re-walking `findings` (which the cap may have clipped).
    pub(crate) errors: usize,
    pub(crate) warnings: usize,
    pub(crate) fyi: usize,
    /// How many `FindingDto` were actually serialized into `findings`
    /// (the sum of each group's `items.len()`). Equals `finding_count`
    /// when uncapped; smaller when `max_per_rule` clipped some groups, so
    /// the UI can say "showing `shown_count` of `finding_count`".
    pub(crate) shown_count: usize,
    pub(crate) findings: Vec<RuleGroup>,
    pub(crate) error: Option<ValErr>,
    /// Why a proffered `.ags.idx` certificate did NOT stand in for the rule engine,
    /// as the stable snake_case token (`"dictionary_changed"`, `"content_changed"`,
    /// …), else `null`. Present for cross-surface shape parity with `Report`
    /// (laterite-py) and Node's `revalidateReason` (laterite-dev#568 Phase 6). **Structurally
    /// always `null` here:** this surface has no cert-consume door — `validate`
    /// re-runs the engine unconditionally (`certify` only *mints*), so no
    /// certificate is ever offered to accept or reject. The field exists so a JS
    /// consumer reads the same report shape on every surface, and is ready if a
    /// wasm cert-consume path is ever added.
    pub(crate) revalidate_reason: Option<String>,
}

impl ValidationReport {
    pub(crate) fn failure(kind: &str, message: String) -> Self {
        ValidationReport {
            ok: false,
            dict_version: String::new(),
            resolution: String::new(),
            finding_count: 0,
            // A failure is not a verdict: nothing was validated. `error` below
            // is what a consumer must read; these are inert.
            valid: false,
            errors: 0,
            warnings: 0,
            fyi: 0,
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
  /** `error === null && valid` — this surface's historical spelling of the
   *  verdict. **Deprecated in favour of `valid`**, which every surface now
   *  spells the same way; kept for one release so consumers can migrate. */
  ok: boolean;
  /** The bundled edition judged against (`"4.1.1"`, …); `""` on error. */
  dict_version: string;
  /** How that edition was chosen: `forced` | `exact` | `guessed` | `fallback`;
   *  `""` on error. */
  resolution: string;
  /** True total across every rule, independent of any cap. */
  finding_count: number;
  /** The verdict. **Not** `finding_count === 0`: a warning is reported by
   *  default and does not fail, so a file can be `valid` with findings. Pass
   *  `warningsAsErrors: true` to make warnings fatal. */
  valid: boolean;
  /** Per-tier counts, summing to `finding_count`. Use these to colour a report
   *  rather than re-walking `findings`, which a cap may have clipped. */
  errors: number;
  warnings: number;
  fyi: number;
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

/// `validate`'s named options. Note the ABSENT `deny_unknown_fields` — see the
/// `boundary` module's comment on why serde cannot catch a typo here;
/// `decode_opts` enumerates instead.
///
/// `Serialize` is test-only: it costs nothing in the shipped wasm and, unlike a
/// `cfg(test)` `deny_unknown_fields`, it fabricates no behaviour — the drift
/// test reads the SAME `rename_all` config the shipped deserialize path uses.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ValidateOptions {
    dict_version: Option<String>,
    /// Defaults to **true** (`.unwrap_or(true)` at the call site, not here).
    /// The positional parameter this replaces was a required `bool`, so there
    /// was no default to preserve — but Python and Node both promise warnings
    /// ON, and a plain `Option<bool>` silently unwrapping to `false` would have
    /// made the browser the one surface that disagreed.
    warnings: Option<bool>,
    fyi: Option<bool>,
    /// The VERDICT dial (#321), separate from the two display dials above.
    /// Defaults to **false**: a warning is shown and does not fail. Python and
    /// Node default the same way, so the browser is not the odd surface out.
    warnings_as_errors: Option<bool>,
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
        "warningsAsErrors",
        "encoding",
        "maxPerRule",
        "dictionary",
        "dictReplace",
    ];
    const WHAT: &'static str = "validate options";
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
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ValidateOptions")]
    pub type ValidateOptionsJs;
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

/// The browser highlight's char-span, by precedence: a finding-CARRIED span
/// (Rules 1/6 attach one directly) wins; otherwise, for a field-targeted finding
/// that carries a `field_index`, derive the inner-value span from the raw source
/// line via the parse leaf's [`laterite_ags4_parse::field_span`].
///
/// This derivation exists on NO other surface — `char_span` is serialized only
/// by wasm, and it drives the browser's cell/heading highlight — yet before laterite-dev#555
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
    // Computed before `found` is consumed by the serialization walk below, and
    // from the FULL finding set — the `max_per_rule` cap clips what crosses the
    // boundary, never what the verdict was reached from.
    let verdict = Verdict::of(&found, o.warnings_as_errors.unwrap_or(false));
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
        ok: verdict.is_valid(),
        dict_version: dv.as_str().to_string(),
        resolution: kind.as_str().to_string(),
        finding_count,
        valid: verdict.is_valid(),
        errors: verdict.errors,
        warnings: verdict.warnings,
        fyi: verdict.fyi,
        shown_count,
        findings: findings_out,
        error: None,
        // No cert-consume path on this surface (see the field's doc): the engine
        // always ran, so there is no proffered certificate to have rejected.
        revalidate_reason: None,
    }
}

#[cfg(test)]
mod tests {
    //! `run()` — plain all along, and the one place in this crate where a
    //! caller mistake has to come back as DATA rather than an exception.
    //!
    //! Which `kind` each arm produces is the part a consumer switches on, and
    //! none of the four was exercised while they sat behind `validate`'s
    //! `JsValue` return.
    use super::*;
    use crate::testdata::LOCA_A;

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

    // The laterite-dev#568 Phase-3 end-to-end fixtures, shared with the validator's `custom_dict.rs`
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
}
