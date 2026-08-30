//! Apply-Fixes: `compute_fixes()` / `apply_fixes()`.
//!
//! A separate surface from `validate()` so the byte-faithful finding JSON is
//! never perturbed. Both reuse the validate skeleton — resolve the encoding,
//! parse, resolve the dictionary, run the rules — so a fix is computed against
//! exactly the findings the report shows.
use crate::resolve::{resolve_dict_override, resolve_encoding};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::{CheckOptions, WorldScope, check_parsed_with_dict};
use serde::Serialize;
use wasm_bindgen::prelude::*;

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
      | "pad_short_row" | "quote_unquoted_row";
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
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Fix[]")]
    pub type FixesJs;
}
