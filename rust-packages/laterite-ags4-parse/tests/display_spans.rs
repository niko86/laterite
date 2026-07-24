//! The DISPLAY tokenizer contract — inherited from the retired `tokenize_spans`
//! (#533) and now carried by `scan::scan_line(line, DISPLAY)`.
//!
//! These guarantees came originally from the browser's hand-written TS
//! `splitAgsFields`, moved into Rust so the two could not drift, and survive
//! that tokenizer's retirement because they describe the GRAMMAR, not the
//! implementation:
//!   - the token ranges tile the line exactly — no gap, no overlap;
//!   - inner-value bounds satisfy `token_start ≤ value_start ≤ value_end ≤
//!     token_end`, and exclude the surrounding quotes AND the trailing comma;
//!   - the empty line yields exactly one empty field (never zero).
//!
//! Bounds here are BYTES. The browser's code-point offsets are produced by the
//! wasm adapter and pinned in the web wasm test lane (`agsline.test.ts`), which
//! is the only place the units JS actually receives get checked.

use laterite_ags4_parse::scan::{DISPLAY, RawField, scan_line};
use proptest::prelude::*;

/// Concatenating every token's source slice rebuilds the input exactly.
fn reassemble(line: &str, spans: &[RawField]) -> String {
    spans
        .iter()
        .map(|s| &line[s.token_start..s.token_end])
        .collect()
}

/// Inner value of a span, sliced from the original line.
fn inner<'a>(line: &'a str, s: &RawField) -> &'a str {
    &line[s.value_start..s.value_end]
}

#[test]
fn heading_line_splits_into_tokens_with_inner_values() {
    let line = "\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 3);
    assert_eq!(inner(line, &spans[0]), "HEADING");
    assert_eq!(inner(line, &spans[1]), "LOCA_ID");
    assert_eq!(inner(line, &spans[2]), "LOCA_TYPE");
    assert_eq!(reassemble(line, &spans), line);
}

#[test]
fn trailing_comma_is_absorbed_into_the_preceding_token() {
    // `"DATA","BH01",` → 2 fields, the last token keeping its trailing comma.
    let line = "\"DATA\",\"BH01\",";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 2);
    assert_eq!(&line[spans[1].token_start..spans[1].token_end], "\"BH01\",");
    assert!(spans[1].had_comma);
    assert_eq!(reassemble(line, &spans), line);
}

#[test]
fn escaped_quotes_stay_inside_the_field() {
    let line = "\"a\",\"b \"\"c\"\" d\"";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 2);
    // The inner value is the raw slice between the outer quotes — the doubled
    // `""` is kept verbatim (unescaping is a separate concern), and the field
    // says so rather than leaving the caller to discover it.
    assert_eq!(inner(line, &spans[1]), "b \"\"c\"\" d");
    assert!(spans[1].has_escape);
    assert!(!spans[0].has_escape);
    assert_eq!(reassemble(line, &spans), line);
}

#[test]
fn empty_line_yields_one_empty_field() {
    let spans = scan_line("", DISPLAY);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].token_start, 0);
    assert_eq!(spans[0].token_end, 0);
    assert_eq!(spans[0].value_start, 0);
    assert_eq!(spans[0].value_end, 0);
}

#[test]
fn empty_quoted_field_has_a_zero_width_inner_span() {
    // `""` is ubiquitous in AGS4 (an empty cell); its inner span is zero-width
    // just inside the opening quote, NOT a span that includes a quote.
    let line = "\"DATA\",\"\",\"x\"";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[1].value_start, spans[1].value_end);
    assert!(spans[1].quoted);
    assert_eq!(reassemble(line, &spans), line);
}

#[test]
fn unquoted_field_inner_value_is_space_trimmed_under_display() {
    // An unquoted, space-padded value: the token keeps the spaces (tiling), but
    // the inner value is trimmed — what a highlight paints.
    let line = "X,  Y  ,Z";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 3);
    assert_eq!(inner(line, &spans[1]), "Y");
    assert_eq!(&line[spans[1].token_start..spans[1].token_end], "  Y  ,");
    assert!(!spans[1].quoted);
    assert_eq!(reassemble(line, &spans), line);
}

#[test]
fn astral_chars_do_not_split_a_bound() {
    // A 🦀 is 4 UTF-8 bytes. `"` and `,` are ASCII and never appear inside a
    // multi-byte sequence, so every bound lands on a char boundary and slicing
    // by it cannot panic.
    let line = "\"a🦀b\",\"z\"";
    let spans = scan_line(line, DISPLAY);
    assert_eq!(spans.len(), 2);
    assert_eq!(inner(line, &spans[0]), "a🦀b");
    assert_eq!(inner(line, &spans[1]), "z");
    assert_eq!(reassemble(line, &spans), line);
}

/// Every span's bounds are well-ordered and the inner value never escapes the
/// token: `token_start ≤ value_start ≤ value_end ≤ token_end`.
fn bounds_hold(spans: &[RawField]) -> bool {
    spans.iter().all(|s| {
        s.token_start <= s.value_start && s.value_start <= s.value_end && s.value_end <= s.token_end
    })
}

/// A cell char set with the interesting embeddables: quotes, commas, spaces, an
/// accented range, and an astral char — but no line terminators (a single
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
    /// the token ranges tile it exactly and the inner bounds are well-ordered.
    #[test]
    fn display_spans_tile_the_line_and_are_well_bounded(
        line in prop::collection::vec(line_char(), 0..40).prop_map(|cs| cs.into_iter().collect::<String>()),
    ) {
        let spans = scan_line(&line, DISPLAY);
        prop_assert_eq!(reassemble(&line, &spans), line.clone());
        prop_assert!(bounds_hold(&spans));
        // Never zero fields — even the empty line has one empty field.
        prop_assert!(!spans.is_empty());
        // Tiling stated directly: contiguous from 0 to the line's end.
        prop_assert_eq!(spans[0].token_start, 0);
        for w in spans.windows(2) {
            prop_assert_eq!(w[0].token_end, w[1].token_start);
        }
        prop_assert_eq!(spans[spans.len() - 1].token_end, line.len());
    }
}
