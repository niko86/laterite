//! The identity doors: what this package is, and what engine is inside it.
//!
//! Plain-Rust signatures, unlike almost everything else here — which is why
//! they are the four exports a native `cargo test` can drive end to end. That
//! matters most for the identity pair: `engine_fingerprint` exists precisely
//! because a *constant* once stood in for a real answer (laterite-dev#556), and a constant
//! is what these would silently become if someone replaced a crate lookup with
//! a literal.
use wasm_bindgen::prelude::*;

/// The AGS4 rule catalogue as the gated `rules_meta.json` JSON string — the
/// browser parses it into typed rule entries. Mirrors `laterite.list_rules()` /
/// `lat rules`. No input.
#[wasm_bindgen]
pub fn list_rules() -> String {
    laterite_ags4_validator::rule_metadata_json().to_string()
}

/// The crate version — the same answer Node's `version()` gives, from the same
/// `CARGO_PKG_VERSION`.
///
/// It exists because `ags4-compliance`'s wasm runner HARD-CODED `version: "0.5.1"`
/// (the dev satellite's tools/compliance/emit_js.mjs) — a literal true when it
/// was written, that the workspace moved past to 0.7.0 while nothing compared it
/// back. The harness then printed "wasm v0.5.1" next to three 0.7.0 surfaces and
/// called the comparison 4-laterite identity. The build was current; only the
/// report lied. Node had this all along and asked the module; wasm had nothing to
/// ask, which is why someone wrote a constant instead. (laterite-dev#556)
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

#[cfg(test)]
mod tests {
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
