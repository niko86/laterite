//! `certify()` — minting an `.ags.idx` validity certificate client-side (#360).
//!
//! The mint is the shared one (`laterite_ags4_trust::mint`), so a browser-minted
//! certificate is byte-for-byte the same statement as one from `lat certify`.
//! Behind the `certify` feature: it is the only door onto `laterite-ags4-trust`,
//! so a build without it drops the whole certificate stack.
use crate::boundary::{WasmOptions, decode_opts};
use crate::resolve::{build_custom_dict, resolve_dict_override, resolve_encoding};
use laterite_ags4_validator::CheckOptions;
use wasm_bindgen::prelude::*;

/// `certify`'s named options — `ValidateOptions`' dictionary half plus the
/// clock. No `warnings`/`fyi`/`maxPerRule`: the mint measures every tier itself
/// and reports counts, so there is nothing for a caller to include or exclude.
#[cfg(feature = "certify")]
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct CertifyOptions {
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

#[cfg(feature = "certify")]
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

// Split from `validate`'s `TS_OPTIONS` when `certify` became a feature (#330): the
// generated `.d.ts` is the published API reference, so a build without the
// export must not still declare its options interface. Every gated surface's TS
// rides the same `#[cfg]` as the export it describes, for that reason.
#[cfg(feature = "certify")]
#[wasm_bindgen(typescript_custom_section)]
const TS_CERTIFY_OPTIONS: &'static str = r#"
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

#[cfg(feature = "certify")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CertifyOptions")]
    pub type CertifyOptionsJs;
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
/// (laterite-dev#568), the same overlay `validate` accepts: the stamp records the dict's
/// `{name, hash}` so a later `validate --index` on any surface re-validates (never
/// silently vouches) when the effective dictionary differs (O-48, record-not-contract).
///
/// Behind the `certify` feature — it is the only door onto `laterite-ags4-trust`,
/// so a build without it drops the whole certificate stack.
#[cfg(feature = "certify")]
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
#[cfg(feature = "certify")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::{CLEAN, LOCA_A, err};

    /// Certify options carrying only the clock — the one field with no default,
    /// since wasm cannot read one.
    #[cfg(feature = "certify")]
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
    #[cfg(feature = "certify")]
    #[test]
    fn certify_stamps_the_unified_engine_identity() {
        let json = match certify_core(CLEAN, &stamped_at("2020-01-01T00:00:00Z")) {
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
    #[cfg(feature = "certify")]
    #[test]
    fn a_browser_minted_certificate_measured_every_tier_it_names() {
        let json = certify_core(CLEAN, &stamped_at("2020-01-01T00:00:00Z"))
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
    #[cfg(feature = "certify")]
    #[test]
    fn a_browser_minted_certificate_cannot_claim_a_world_check() {
        let json = certify_core(CLEAN, &stamped_at("2020-01-01T00:00:00Z"))
            .unwrap_or_else(|_| panic!("a clean minimal AGS4 file must certify"));
        assert!(
            !json.contains("check_files"),
            "the stamp must carry no world claim at all: {json}"
        );
    }

    // ---------------------------------------------------------------
    // certify_core's error arms
    // ---------------------------------------------------------------

    #[cfg(feature = "certify")]
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

    #[cfg(feature = "certify")]
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

    #[cfg(feature = "certify")]
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

    const DICT_JSON: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/custom_dict/xtra.dict.json");

    #[cfg(feature = "certify")]
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

    #[cfg(feature = "certify")]
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

    #[cfg(feature = "certify")]
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
