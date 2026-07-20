//! `tokenize_spans` — the offset-preserving field tokenizer (#533).
//!
//! This is the Rust home of the browser's `splitAgsFields` (agsline.ts); the
//! TS copy is retired in favour of a tiny wasm wrapping this. The load-bearing
//! guarantees it inherits from the TS unit suite live here now:
//!   - lossless reassembly: concatenating every span's `text` rebuilds the line
//!     byte-for-byte (proptest, arbitrary lines incl. astral chars + quotes);
//!   - inner-value bounds: `start ≤ value_start ≤ value_end ≤ end`, and the
//!     inner value excludes the surrounding quotes AND the trailing comma;
//!   - the empty line yields exactly one empty field (never zero).

use laterite_ags4_parse::{AgsSpan, tokenize_spans};
use proptest::prelude::*;

/// Concatenating every token's `.text` rebuilds the input exactly.
fn reassemble(spans: &[AgsSpan]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}

/// Inner value of a span, sliced from the original line by CODE POINT (the
/// offsets are code-point indices, matching JS `[...raw]`).
fn inner(line_chars: &[char], s: &AgsSpan) -> String {
    line_chars[s.value_start as usize..s.value_end as usize]
        .iter()
        .collect()
}

#[test]
fn heading_line_splits_into_tokens_with_inner_values() {
    let line = "\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"";
    let chars: Vec<char> = line.chars().collect();
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 3);
    assert_eq!(inner(&chars, &spans[0]), "HEADING");
    assert_eq!(inner(&chars, &spans[1]), "LOCA_ID");
    assert_eq!(inner(&chars, &spans[2]), "LOCA_TYPE");
    assert_eq!(reassemble(&spans), line);
}

#[test]
fn trailing_comma_is_absorbed_into_the_preceding_token() {
    // Matches agsline.test.ts: `"DATA","BH01",` → 2 fields, the last token's
    // text keeps its trailing comma.
    let line = "\"DATA\",\"BH01\",";
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[1].text, "\"BH01\",");
    assert_eq!(reassemble(&spans), line);
}

#[test]
fn escaped_quotes_stay_inside_the_field() {
    let line = "\"a\",\"b \"\"c\"\" d\"";
    let chars: Vec<char> = line.chars().collect();
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 2);
    // The inner value is the raw slice between the outer quotes — the doubled
    // `""` is kept verbatim here (unescaping is a separate concern).
    assert_eq!(inner(&chars, &spans[1]), "b \"\"c\"\" d");
    assert_eq!(reassemble(&spans), line);
}

#[test]
fn empty_line_yields_one_empty_field() {
    let spans = tokenize_spans("");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 0);
    assert_eq!(spans[0].value_start, 0);
    assert_eq!(spans[0].value_end, 0);
}

#[test]
fn empty_quoted_field_has_a_zero_width_inner_span() {
    // `""` is ubiquitous in AGS4 (an empty cell); its inner span is zero-width
    // just inside the opening quote, NOT a span that includes a quote.
    let line = "\"DATA\",\"\",\"x\"";
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[1].value_start, spans[1].value_end); // zero width
    assert_eq!(reassemble(&spans), line);
}

#[test]
fn unquoted_field_inner_value_is_space_trimmed() {
    // An unquoted, space-padded value: the token keeps the spaces (lossless),
    // but the inner value is trimmed (what a highlight paints).
    let line = "X,  Y  ,Z";
    let chars: Vec<char> = line.chars().collect();
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 3);
    assert_eq!(inner(&chars, &spans[1]), "Y");
    // Lossless: the token text still carries the padding + comma.
    assert_eq!(spans[1].text, "  Y  ,");
    assert_eq!(reassemble(&spans), line);
}

#[test]
fn astral_chars_do_not_split_an_offset() {
    // A 🦀 is one code point; offsets index by code point, so the field after
    // it recovers cleanly and reassembly is exact.
    let line = "\"a🦀b\",\"z\"";
    let chars: Vec<char> = line.chars().collect();
    let spans = tokenize_spans(line);
    assert_eq!(spans.len(), 2);
    assert_eq!(inner(&chars, &spans[0]), "a🦀b");
    assert_eq!(inner(&chars, &spans[1]), "z");
    assert_eq!(reassemble(&spans), line);
}

/// Every span's offsets are well-ordered and the inner value never escapes the
/// token: `start ≤ value_start ≤ value_end ≤ end`.
fn bounds_hold(spans: &[AgsSpan]) -> bool {
    spans
        .iter()
        .all(|s| s.start <= s.value_start && s.value_start <= s.value_end && s.value_end <= s.end)
}

/// A cell char set with the interesting embeddables: quotes, commas, spaces,
/// an accented range, and an astral char — but no line terminators (a single
/// line's tokenizer never sees them).
fn line_char() -> impl Strategy<Value = char> {
    prop_oneof![
        prop::char::range('a', 'z'),
        prop::char::range('0', '9'),
        Just(' '),
        Just(','),
        Just('"'),
        prop::char::range('\u{00a1}', '\u{017f}'),
        Just('🦀'),
    ]
}

proptest! {
    /// The load-bearing invariant (ported from agsline.test.ts): for ANY line,
    /// concatenating every token's `.text` reproduces the line exactly, and the
    /// inner-value bounds are well-ordered.
    #[test]
    fn tokenize_spans_is_lossless_and_well_bounded(
        line in prop::collection::vec(line_char(), 0..40).prop_map(|cs| cs.into_iter().collect::<String>()),
    ) {
        let spans = tokenize_spans(&line);
        prop_assert_eq!(reassemble(&spans), line.clone());
        prop_assert!(bounds_hold(&spans));
        // Never zero fields — even the empty line has one empty field.
        prop_assert!(!spans.is_empty());
    }
}
