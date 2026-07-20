//! The browser's AGS4 line tokenizer + field quoter, as a tiny wasm module
//! (#533). Two thin `#[wasm_bindgen]` wrappers over the shared Rust leaves so
//! the browser drives off the same authority as every other surface:
//!   - [`tokenize_spans`] wraps `laterite_ags4_parse::tokenize_spans`;
//!   - [`quote_field`] wraps `laterite_types::quote_field`.
//!
//! No engine, no validator, no arrow — just the two line primitives, so the
//! compiled artifact stays tiny (a size gate keeps that honest). The old TS
//! copy in `web/src/lib/agsline.ts` is retired against this.

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// One field token as the browser consumes it (`AgsField` in agsline.ts).
/// camelCase so the JS shape is unchanged from the retired TS tokenizer.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsField {
    text: String,
    start: u32,
    end: u32,
    value_start: u32,
    value_end: u32,
}

/// Tokenize one AGS4 line into offset-preserving field spans. Returns a JS
/// array of `{text, start, end, valueStart, valueEnd}` — the browser's
/// `AgsField[]`, with the lossless-reassembly + code-point-offset guarantees of
/// the underlying `tokenize_spans`.
#[wasm_bindgen]
pub fn tokenize_spans(line: &str) -> Result<JsValue, JsError> {
    let fields: Vec<JsField> = laterite_ags4_parse::tokenize_spans(line)
        .into_iter()
        .map(|s| JsField {
            text: s.text,
            start: s.start,
            end: s.end,
            value_start: s.value_start,
            value_end: s.value_end,
        })
        .collect();
    serde_wasm_bindgen::to_value(&fields).map_err(|e| JsError::new(&e.to_string()))
}

/// Quote one raw value as an AGS4 field (wrap in `"`, double any embedded `"`).
/// The wasm face of `laterite_types::quote_field` — the browser's `quoteAgsField`.
#[wasm_bindgen]
pub fn quote_field(value: &str) -> String {
    laterite_types::quote_field(value)
}
