//! napi wrappers over `ags5_types` — the AGS4 type system, the Node analog of
//! laterite-py's `ags_types_fns`. The parsing *logic* stays native (one shared
//! engine across hosts); only the data-holder layers (registry, typed-graph) are
//! TS-generated. napi camelCases: `canonical_type` → `canonicalType`, etc.

use napi_derive::napi;

/// AGS spec type code → canonical category label (`"string"`, `"integer"`,
/// `"decimal"`, `"datetime"`, `"date"`, `"time"`, `"bool"`, `"enum"`), or `null`
/// for unknown codes (the TS wrapper re-raises as an error).
#[napi]
pub fn canonical_type(ags_type: String) -> Option<String> {
    ags5_types::canonical_type(&ags_type).map(|c| c.as_str().to_string())
}

/// Presentation hint for a numeric AGS type: `"2DP"` → `"%.2f"`, `"3SF"` →
/// `"%.3g"`, `"1SCI"` → `"%.1e"`; `null` for non-numeric / unknown codes.
#[napi]
pub fn display_hint(ags_type: String) -> Option<String> {
    ags5_types::display_hint(&ags_type)
}

/// Parse an AGS4-shaped raw string into its canonical value — the same engine
/// the read path uses. Permissive: empty / unparseable → `null`. Returns native
/// JS: integer/decimal → number, bool → boolean, string/enum → string,
/// **datetime/date/time → the canonical string** (`"YYYY-MM-DD HH:MM:SS"` /
/// `"YYYY-MM-DD"` / `"HH:MM:SS"`), unknown code → the trimmed input.
#[napi]
pub fn parse_value(raw: Option<String>, ags_type: String) -> serde_json::Value {
    ags5_types::parse_value(raw.as_deref(), &ags_type)
}
