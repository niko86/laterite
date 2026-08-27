//! `censor()` — anonymise a file with the shared scrub engine (laterite-dev#581).
//!
//! The browser Anonymiser drives the SAME `laterite-ags4-censor` engine the
//! corpus `censor` tool uses (Phase 2 of the laterite-dev#527 convergence), instead of a
//! hand-written TS reimplementation. It's a batch action (Download click), off
//! the render path, so it rides the engine wasm asynchronously in the validator
//! worker rather than a boot-critical main-thread instance.
use crate::boundary::{WasmOptions, decode_opts, to_js};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// `{ text, tally }` — the anonymised file plus the per-action cell/structure
/// counts the Anonymiser surfaces. `tally`'s fields match the leaf's snake_case.
#[cfg(feature = "censor")]
#[derive(Serialize)]
pub(crate) struct CensorDto {
    pub(crate) text: String,
    pub(crate) tally: laterite_ags4_censor::Tally,
}

ts_section! {
    #[cfg(feature = "censor")]
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

#[cfg(feature = "censor")]
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
#[cfg(feature = "censor")]
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct CensorOptions {
    selected_codes: Option<Vec<String>>,
    token: Option<String>,
    drop_custom: Option<bool>,
    include_freetext: Option<bool>,
}

#[cfg(feature = "censor")]
impl WasmOptions for CensorOptions {
    const KEYS: &'static [&'static str] =
        &["selectedCodes", "token", "dropCustom", "includeFreetext"];
    const WHAT: &'static str = "censor options";
}

#[cfg(feature = "censor")]
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

#[cfg(feature = "censor")]
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
/// * `opts` — a `CensorOptions` object; every field optional, so
///   `censor(data, json)` is a complete call. An unrecognised key is refused
///   by name.
///
/// `PROJ_ID`'s filehash is the full 64-hex SHA-256 of `data` (a KEY field —
/// full width so a collision is cryptographically nil); the leaf takes that id
/// precomputed, so this wrapper hashes the bytes.
///
/// Behind the `censor` feature (#330) — with it goes the only sha2/hex use in
/// the crate.
#[cfg(feature = "censor")]
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
#[cfg(feature = "censor")]
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
    use super::*;
    use crate::testdata::err;

    /// The classification SSOT the browser Anonymiser fetches, read from the
    /// same file the engine ships rather than a hand-written stub — a stub would
    /// let this suite pass while the real policy said something else.
    #[cfg(feature = "censor")]
    const SENSITIVE: &str = include_str!("../../laterite-ags4-core/data/sensitive_headings.json");

    // ---------------------------------------------------------------
    // censor_core
    // ---------------------------------------------------------------

    /// A file carrying one heading from each of the categories the tests below
    /// assert on: a location id (pseudonym), a coordinate (blank) and a project
    /// id (filehash).
    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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

    #[cfg(feature = "censor")]
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
}
