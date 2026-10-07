//! Apply-Fixes: `compute_fixes()` / `apply_fixes()`.
//!
//! A separate surface from `validate()` so the byte-faithful finding JSON is
//! never perturbed. Both reuse the validate skeleton — resolve the encoding,
//! parse, resolve the dictionary, run the rules — so a fix is computed against
//! exactly the findings the report shows.
use crate::boundary::{WasmOptions, decode_opts};
use crate::resolve::{resolve_dict_override, resolve_encoding};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::fixes::{Recode, compute_recode_fix};
use laterite_ags4_validator::{CheckOptions, CodeRewrite, WorldScope, check_parsed_with_dict};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// `compute_fixes`' named options. `dictVersion` and `encoding` were
/// positional until the ABBR code rewrite (#1024) needed two more: an export
/// takes its inputs, then one options object.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct FixOptions {
    /// Force the edition instead of reading the file's `TRAN_AGS`.
    dict_version: Option<String>,
    /// The source's text encoding (a WHATWG label); UTF-8 when absent.
    encoding: Option<String>,
    /// `"keep"` (default) | `"standard"`, parsed by the engine's `FromStr`
    /// so the browser refuses exactly what every other surface refuses.
    on_code_case: Option<String>,
    /// `{heading: {fromCode: toCode}}` — caller-named code rewrites.
    recode: Option<Recode>,
}

impl WasmOptions for FixOptions {
    const KEYS: &'static [&'static str] = &["dictVersion", "encoding", "onCodeCase", "recode"];
    const WHAT: &'static str = "fix options";
}

impl FixOptions {
    fn rewrite(&mut self) -> Result<CodeRewrite, String> {
        Ok(CodeRewrite {
            on_code_case: self.on_code_case.as_deref().unwrap_or("keep").parse()?,
            recode: self.recode.take().unwrap_or_default(),
        })
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_FIX_OPTIONS: &'static str = r#"
/** Named options for `compute_fixes`. */
export interface FixOptions {
  /** Force the edition (`"4.0.3"`…`"4.2"`) instead of reading `TRAN_AGS`. */
  dictVersion?: string;
  /** The source's text encoding (a WHATWG label); UTF-8 when absent. */
  encoding?: string;
  /* `onCodeCase` and `recode` ask for an ABBR code rewrite. The engine never
   * chooses one: without them no code is rewritten. */
  /** `"keep"` (default) leaves ABBR codes that differ only by letter case as
   *  written; `"standard"` rewrites such a set to the edition's standard code
   *  when exactly one matches it ignoring case. */
  onCodeCase?: "keep" | "standard";
  /** Rewrites you name, `{heading: {fromCode: toCode}}`, applied before
   *  `onCodeCase`. A heading the file does not type `PA`, or a code it does
   *  not use, throws. */
  recode?: Record<string, Record<string, string>>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "FixOptions")]
    pub type FixOptionsJs;
}

/// Compute the safe fixes for AGS4 bytes in the browser.
///
/// Parses + runs the full rule engine (FYI on, so e.g. the Rule 1 BOM
/// path is seen), then `laterite_ags4_validator::fixes::compute_fixes`. Returns a
/// JSON-compatible `Fix[]` (empty array on a parse error — there's
/// nothing to fix in an un-parseable file, and `validate` already
/// surfaces the error to the UI).
///
/// * `opts` — a `FixOptions` object; every field optional, so
///   `compute_fixes(data)` is a complete call. An unrecognised key is refused
///   by name.
///
/// `onCodeCase` / `recode` ask for an ABBR code rewrite (#1024): it comes back
/// as one `recode_abbreviation` fix, safe, under Rule 16. A set whose rewrite
/// would give two rows one KEY is left out of it. Throws only on the caller's
/// own mistake — an unknown option or mode, or a `recode` the file cannot
/// honour — since those are arguments to correct, not a file to give up on.
#[wasm_bindgen]
pub fn compute_fixes(data: &[u8], opts: Option<FixOptionsJs>) -> Result<FixesJs, JsError> {
    console_error_panic_hook::set_once();
    let mut o: FixOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let rewrite = o.rewrite().map_err(|m| JsError::new(&m))?;
    let fixes = compute_fixes_core(
        data,
        o.dict_version.as_deref(),
        o.encoding.as_deref(),
        &rewrite,
    )
    .map_err(|m| JsError::new(&m))?;
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    Ok(fixes
        .serialize(&serializer)
        .expect("Fixes is plain data and always serialises")
        .unchecked_into())
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
///
/// A requested rewrite the file cannot honour is the one `Err`: it is the
/// caller's argument, and an empty list would read as "nothing to rewrite".
fn compute_fixes_core(
    data: &[u8],
    dict_version: Option<&str>,
    encoding_label: Option<&str>,
    rewrite: &CodeRewrite,
) -> Result<Vec<laterite_ags4_validator::fixes::Fix>, String> {
    let Ok(dict_over) = resolve_dict_override(dict_version) else {
        return Ok(Vec::new());
    };
    // An unknown label yields no fixes rather than fixes computed against the
    // wrong decoding — silently "fixing" text we mis-decoded is the worst
    // option on the table.
    let Ok(encoding) = resolve_encoding(encoding_label) else {
        return Ok(Vec::new());
    };
    let Ok(parsed) = parse_bytes(data, encoding) else {
        return Ok(Vec::new());
    };
    let opts = CheckOptions {
        dict_version: dict_over,
        include_fyi: true,
        encoding,
        ..CheckOptions::default()
    };
    // Through the door, so the fixes offered in the browser are computed against the
    // same dictionary `lat fix` would use on the same bytes (the O-42 guard included).
    let Ok((found, dv, _kind)) = check_parsed_with_dict(&parsed, &opts, &WorldScope::None) else {
        return Ok(Vec::new());
    };
    let dict = laterite_ags4_validator::dict::Dictionary::bundled(dv);
    let mut fixes = laterite_ags4_validator::fixes::compute_fixes(&parsed, &found, dict);
    if rewrite.is_requested() {
        let planned = compute_recode_fix(&parsed, dict, rewrite).map_err(|e| e.to_string())?;
        fixes.extend(planned.fix);
    }
    Ok(fixes)
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
      | "pad_short_row" | "quote_unquoted_row" | "trim_abbreviation"
      | "recode_abbreviation";
  label: string;
  /** The exact rule label (`"AGS Format Rule 8"`, …), for cross-linking back to
   *  the finding it resolves. */
  rule: string;
  /** Anchor line for ordering/preview; `null` for whole-file kinds. */
  line: number | null;
  /** `safe` is bulk-applicable; `risky` guesses intent and is opt-in only. */
  risk: "safe" | "risky";
  /** EMPTY for the byte-level kinds (`normalize_crlf`, `strip_bom`), which
   *  operate on the whole document rather than a span. On a
   *  `recode_abbreviation` fix, an edit spanning a whole line with an empty
   *  `replacement` removes that line. */
  edits: SpanEdit[];
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::{CLEAN, err};

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

    // ---------------------------------------------------------------
    // compute_fixes_core — four ways to return nothing
    // ---------------------------------------------------------------

    /// `compute_fixes_core` with no rewrite asked for — the four ways to
    /// nothing below are about the file, never the options.
    fn core(
        data: &[u8],
        dict_version: Option<&str>,
        encoding_label: Option<&str>,
    ) -> Vec<laterite_ags4_validator::fixes::Fix> {
        compute_fixes_core(data, dict_version, encoding_label, &CodeRewrite::default())
            .expect("no rewrite, nothing to refuse")
    }

    /// One delivery: ABBR declares `Undisturbed` and `UNDISTURBED` under
    /// `TRIG_COND`, and two TRIG rows use one each.
    const CASE_PAIR: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\
        \"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\"GROUP\",\"ABBR\"\r\n\
        \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
        \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
        \"DATA\",\"TRIG_COND\",\"Undisturbed\",\"d\"\r\n\
        \"DATA\",\"TRIG_COND\",\"UNDISTURBED\",\"d\"\r\n\r\n\
        \"GROUP\",\"TRIG\"\r\n\"HEADING\",\"SPEC_REF\",\"TRIG_COND\"\r\n\
        \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"PA\"\r\n\
        \"DATA\",\"1\",\"Undisturbed\"\r\n\"DATA\",\"2\",\"UNDISTURBED\"\r\n";

    fn rewrite(mut opts: FixOptions) -> Result<Vec<laterite_ags4_validator::fixes::Fix>, String> {
        compute_fixes_core(CASE_PAIR, Some("4.1.1"), None, &opts.rewrite()?)
    }

    #[test]
    fn recode_and_on_code_case_offer_one_safe_recode_fix() {
        // #1024: neither asked for, no recode fix; `standard` and the named
        // recode each offer one, and applying it settles the pair.
        let kind = laterite_ags4_validator::FixKind::RecodeAbbreviation;
        assert!(
            !core(CASE_PAIR, Some("4.1.1"), None)
                .iter()
                .any(|f| f.kind == kind)
        );
        let std = rewrite(FixOptions {
            on_code_case: Some("standard".into()),
            ..Default::default()
        })
        .expect("computes");
        let mut recode = Recode::new();
        recode
            .entry("TRIG_COND".into())
            .or_default()
            .insert("Undisturbed".into(), "UNDISTURBED".into());
        let named = rewrite(FixOptions {
            recode: Some(recode),
            ..Default::default()
        })
        .expect("computes");
        for fixes in [&std, &named] {
            let r: Vec<_> = fixes.iter().filter(|f| f.kind == kind).collect();
            assert_eq!(r.len(), 1, "{fixes:?}");
            assert_eq!(r[0].risk, laterite_ags4_validator::fixes::FixRisk::Safe);
        }
        let out = apply_fixes_core(CASE_PAIR, None, &std).expect("applies");
        let text = String::from_utf8(out).expect("utf-8");
        assert!(!text.contains("\"Undisturbed\""), "{text}");
        assert_eq!(
            apply_fixes_core(CASE_PAIR, None, &named).expect("applies"),
            text.as_bytes()
        );
    }

    #[test]
    fn a_bad_mode_or_recode_is_refused_by_name() {
        let e = rewrite(FixOptions {
            on_code_case: Some("first".into()),
            ..Default::default()
        })
        .expect_err("refused");
        assert!(e.contains("keep, standard"), "{e}");
        let mut recode = Recode::new();
        recode
            .entry("SPEC_REF".into())
            .or_default()
            .insert("1".into(), "2".into());
        let e = rewrite(FixOptions {
            recode: Some(recode),
            ..Default::default()
        })
        .expect_err("refused");
        assert!(e.contains("SPEC_REF"), "{e}");
    }

    #[test]
    fn a_fixable_file_yields_fixes() {
        // LF line endings breach Rule 2 and are safely fixable, so this is the
        // baseline the four failure paths below are contrasted against — without
        // it, "returns empty" proves nothing.
        let lf: Vec<u8> = CLEAN.iter().copied().filter(|&b| b != b'\r').collect();
        let fixes = core(&lf, None, None);
        assert!(
            !fixes.is_empty(),
            "a file with LF endings must offer at least the CRLF fix"
        );
    }

    #[test]
    fn an_unknown_edition_yields_no_fixes() {
        assert!(core(CLEAN, Some("4.9"), None).is_empty());
    }

    #[test]
    fn an_unknown_encoding_yields_no_fixes() {
        // The important one. This door has no error channel, so the alternative
        // to "no fixes" is fixes computed against text we mis-decoded — a button
        // that silently corrupts the user's file.
        let lf: Vec<u8> = CLEAN.iter().copied().filter(|&b| b != b'\r').collect();
        assert!(
            !core(&lf, None, None).is_empty(),
            "the fixture must be fixable, or the next assertion proves nothing"
        );
        assert!(core(&lf, None, Some("klingon-1")).is_empty());
    }

    #[test]
    fn unparseable_bytes_yield_no_fixes() {
        assert!(core(b"not an ags file at all", None, None).is_empty());
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
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Fix[]")]
    pub type FixesJs;
}
