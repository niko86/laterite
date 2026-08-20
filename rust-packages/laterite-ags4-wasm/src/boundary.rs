//! The JS↔Rust glue every door shares: options in, results out.
//!
//! Two jobs, and both exist because the boundary itself cannot be tested. A
//! `JsValue` in a signature puts a function beyond `cargo test` — this crate's
//! only lane — so the *decisions* (which keys are accepted, what to say about a
//! typo, which serializer writes the result) live here as plain Rust the tests
//! can reach, and the marshalling that remains is a thin skin over them.
use serde::Serialize;
use wasm_bindgen::prelude::*;

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
pub(crate) struct TranInput {
    pub(crate) issue: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) producer: Option<String>,
    pub(crate) recipient: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) remarks: Option<String>,
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
    /// top-level guard cannot reach: `decode_opts` enumerates only the
    /// outer object's keys, so a misspelled `producr` inside `tran` slips past
    /// it — but then `producer` is unset, and "all five or none" reports it by
    /// name. Requiredness covers what enumeration does not.
    pub(crate) fn fold(self) -> Result<Option<laterite_ags4_emit::TranStamp>, String> {
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
// none, which is why no options struct in this crate carries it. `decode_opts`
// does the work instead, by enumeration.

/// Binds an options struct to the key list its callers may use.
///
/// **Why a trait and not a `&[&str]` argument to `decode_opts`:** several
/// exports each have their own key list, and four interchangeable `&[&str]`
/// consts passed by hand is a silent failure waiting — hand `CertifyOptions`'
/// keys to `validate` and every `validate` typo is accepted again, while a
/// drift test that only checks each const against its own struct stays green.
/// Bound to the type, the pairing cannot be got wrong.
pub(crate) trait WasmOptions: serde::de::DeserializeOwned + Default {
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
pub(crate) fn decode_opts<T: WasmOptions>(opts: Option<JsValue>) -> Result<T, String> {
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
/// **Every fallible** door goes through this, the two build doors included. The
/// three infallible ones (`validate`, `compute_fixes`, `ParsedDataset::meta`)
/// cannot — they have no `Result` to return — and reach for the same
/// `json_compatible()` serializer directly, which is what
/// `serializer_consistency_tests` holds them to. They used
/// serde-wasm-bindgen's *default* serializer until #212, which writes
/// `undefined` for an absent `Option` where this one writes `null` — so
/// `BuildReport`'s published TS declared `line: number | null` while the runtime
/// handed back `undefined`, and a consumer writing `f.line === null` type-checked
/// clean and missed every time. Having one serializer is now the only thing
/// standing between the next result struct and the same bug; `serializer_
/// consistency_tests` asserts there is still only one.
pub(crate) fn to_js<T: Serialize, J: JsCast>(value: &T) -> Result<J, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map(JsCast::unchecked_into)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::BuildOptions;
    #[cfg(feature = "censor")]
    use crate::censor::CensorOptions;
    #[cfg(feature = "certify")]
    use crate::certify::CertifyOptions;
    #[cfg(feature = "diff")]
    use crate::diff::DiffOptions;
    #[cfg(feature = "merge")]
    use crate::merge::MergeOptions;
    use crate::testdata::err;
    use crate::validate::ValidateOptions;

    /// `KEYS` must name exactly the struct's own serde fields.
    ///
    /// The list is what `decode_opts` accepts and the struct is what
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
        assert_keys_match::<BuildOptions>();
        #[cfg(feature = "certify")]
        assert_keys_match::<CertifyOptions>();
        #[cfg(feature = "merge")]
        assert_keys_match::<MergeOptions>();
        #[cfg(feature = "diff")]
        assert_keys_match::<DiffOptions>();
        #[cfg(feature = "censor")]
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
}
