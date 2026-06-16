//! napi wrappers over `laterite_types` — the AGS4 type system, the Node analog of
//! laterite-py's `ags_types_fns`. The parsing *logic* stays native (one shared
//! engine across hosts); only the data-holder layers (registry, typed-graph) are
//! TS-generated. napi camelCases: `canonical_type` → `canonicalType`, etc.

use napi_derive::napi;

/// AGS spec type code → canonical category label (`"string"`, `"integer"`,
/// `"decimal"`, `"datetime"`, `"date"`, `"time"`, `"bool"`, `"enum"`), or `null`
/// for unknown codes (the TS wrapper re-raises as an error).
#[napi]
pub fn canonical_type(ags_type: String) -> Option<String> {
    laterite_types::canonical_type(&ags_type).map(|c| c.as_str().to_string())
}

/// Presentation hint for a numeric AGS type: `"2DP"` → `"%.2f"`, `"3SF"` →
/// `"%.3g"`, `"1SCI"` → `"%.1e"`; `null` for non-numeric / unknown codes.
#[napi]
pub fn display_hint(ags_type: String) -> Option<String> {
    laterite_types::display_hint(&ags_type)
}

/// Parse an AGS4-shaped raw string into its canonical value — the same engine
/// the read path uses. Permissive: empty / unparseable → `null`. Returns native
/// JS: integer/decimal → number, bool → boolean, string/enum → string,
/// **datetime/date/time → the canonical string** (`"YYYY-MM-DD HH:MM:SS"` /
/// `"YYYY-MM-DD"` / `"HH:MM:SS"`), unknown code → the trimmed input.
#[napi]
pub fn parse_value(raw: Option<String>, ags_type: String) -> serde_json::Value {
    laterite_types::parse_value(raw.as_deref(), &ags_type)
}

// These `#[napi]` fns live in a private module, so in a `cargo test` build —
// where the napi registration glue that references them isn't emitted — the
// dead_code lint flags them. Exercising them here both clears that noise and
// pins the napi-layer behaviour (the parsing logic itself is covered in
// laterite-types; this guards the Node binding).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_type_known_and_unknown() {
        assert_eq!(
            canonical_type("2DP".to_string()).as_deref(),
            Some("decimal")
        );
        assert_eq!(canonical_type("YN".to_string()).as_deref(), Some("bool"));
        assert!(canonical_type("NOT_A_TYPE".to_string()).is_none());
    }

    #[test]
    fn display_hint_numeric_only() {
        assert_eq!(display_hint("2DP".to_string()).as_deref(), Some("%.2f"));
        assert!(display_hint("X".to_string()).is_none());
    }

    #[test]
    fn parse_value_typed_and_permissive() {
        // decimal → number
        assert_eq!(
            parse_value(Some("12.50".to_string()), "2DP".to_string()).as_f64(),
            Some(12.5)
        );
        // empty / None → null (permissive)
        assert!(parse_value(Some("".to_string()), "2DP".to_string()).is_null());
        assert!(parse_value(None, "2DP".to_string()).is_null());
    }
}
