//! The browser's AGS4 line tokenizer + field quoter, as a tiny wasm module
//! (#533). Two thin `#[wasm_bindgen]` wrappers over the shared Rust leaves so
//! the browser drives off the same authority as every other surface:
//!   - [`tokenize_spans`] wraps `laterite_ags4_parse::scan::scan_line`;
//!   - [`quote_field`] wraps `laterite_types::quote_field`.
//!
//! No engine, no validator, no arrow — just the two line primitives, so the
//! compiled artifact stays tiny (a size gate keeps that honest). The old TS
//! copy in `web/src/lib/agsline.ts` is retired against this.

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// One field token as the browser consumes it (`AgsField` in agsline.ts).
/// camelCase so the JS shape matches the TS interface.
///
/// Offsets only — no `text`. The caller already holds the line it passed in, so
/// shipping a per-field copy back across the boundary sent JS its own data,
/// paying a UTF-8→UTF-16 decode and an allocation per field for it. Several
/// consumers never wanted the string at all: the column-width pass needs a
/// LENGTH, which is `end - start`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsField {
    start: u32,
    end: u32,
    value_start: u32,
    value_end: u32,
}

/// Tokenize one AGS4 line into offset-preserving field spans. Returns a JS
/// array of `{start, end, valueStart, valueEnd}` — the browser's `AgsField[]`.
///
/// Offsets are CODE POINTS, matching JS `[...raw]` indexing so an astral char
/// never splits one. The shared scanner works in bytes (which is why the
/// validator's per-line walk is cheap), so the conversion happens HERE — in the
/// one adapter that actually has the requirement — rather than taxing every
/// consumer. One forward pass over the line, replacing the `Vec<char>` the
/// retired tokenizer allocated per line.
#[wasm_bindgen]
pub fn tokenize_spans(line: &str) -> Result<JsValue, JsError> {
    let spans = laterite_ags4_parse::scan::scan_line(line, laterite_ags4_parse::scan::DISPLAY);
    let mut to_cp = ByteToCodePoint::new(line);
    let fields: Vec<JsField> = spans
        .iter()
        .map(|s| {
            // ASCENDING order, which is not the order the struct lists them:
            // the cursor cannot rewind, so asking for `token_end` before
            // `value_start` silently returns a stale count and collapses every
            // inner value to empty. The bounds are monotonic; the QUERIES have
            // to be too.
            let start = to_cp.at(s.token_start);
            let value_start = to_cp.at(s.value_start);
            let value_end = to_cp.at(s.value_end);
            let end = to_cp.at(s.token_end);
            JsField {
                start,
                end,
                value_start,
                value_end,
            }
        })
        .collect();
    serde_wasm_bindgen::to_value(&fields).map_err(|e| JsError::new(&e.to_string()))
}

/// Byte offset → code-point offset, as a single forward cursor.
///
/// No table and no allocation: the scanner emits each field's bounds in
/// NON-DECREASING byte order — `token_start ≤ value_start ≤ value_end ≤
/// token_end`, and the next token begins exactly where this one ends — so one
/// pass over the line answers every query. That property is what makes this
/// cheaper than the `Vec<char>` the retired tokenizer allocated per line,
/// rather than merely a different shape of the same cost.
struct ByteToCodePoint<'a> {
    rest: std::str::Chars<'a>,
    byte: usize,
    cp: u32,
}

impl<'a> ByteToCodePoint<'a> {
    fn new(line: &'a str) -> Self {
        Self {
            rest: line.chars(),
            byte: 0,
            cp: 0,
        }
    }

    /// Code points before `target`. Requires `target` >= every previous target;
    /// the cursor cannot rewind, and a backwards query would silently return a
    /// stale count rather than an obviously wrong one.
    fn at(&mut self, target: usize) -> u32 {
        debug_assert!(
            target >= self.byte,
            "ByteToCodePoint queried backwards - the scanner's monotonic-bounds \
             guarantee no longer holds"
        );
        while self.byte < target {
            match self.rest.next() {
                Some(c) => {
                    self.byte += c.len_utf8();
                    self.cp += 1;
                }
                None => break,
            }
        }
        self.cp
    }
}

/// Quote one raw value as an AGS4 field (wrap in `"`, double any embedded `"`).
/// The wasm face of `laterite_types::quote_field` — the browser's `quoteAgsField`.
#[wasm_bindgen]
pub fn quote_field(value: &str) -> String {
    laterite_types::quote_field(value)
}
