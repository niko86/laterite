//! Tokenizer + char-span tests — moved verbatim from the validator's
//! `parse.rs` (its `tests` + `proptest_suite` modules) when `split_ags_line`
//! / `field_span` were lifted into this leaf. The contracts are unchanged.

use laterite_ags4_parse::{field_span, split_ags_line};

#[test]
fn splits_quoted_fields_and_unescapes() {
    assert_eq!(
        split_ags_line(r#""DATA","BH01","100.50""#),
        vec!["DATA", "BH01", "100.50"],
    );
    assert_eq!(
        split_ags_line(r#""DATA","he said ""hi""","x""#),
        vec!["DATA", r#"he said "hi""#, "x"],
    );
    assert_eq!(split_ags_line(r#""HEADING","",""#), vec!["HEADING", "", ""]);
}

#[test]
fn field_span_points_at_inner_value_char_offsets() {
    let line = r#""DATA","BH01","100.50""#;
    let chars: Vec<char> = line.chars().collect();
    let slice = |(s, e): (u32, u32)| chars[s as usize..e as usize].iter().collect::<String>();
    assert_eq!(slice(field_span(line, 0).unwrap()), "BH01");
    assert_eq!(slice(field_span(line, 1).unwrap()), "100.50");
    assert_eq!(field_span(line, 2), None);

    let esc = r#""DATA","he said ""hi""","x""#;
    let echars: Vec<char> = esc.chars().collect();
    let span = field_span(esc, 0).unwrap();
    let got: String = echars[span.0 as usize..span.1 as usize].iter().collect();
    assert_eq!(got, r#"he said ""hi"""#);

    let empty = r#""HEADING","",""#;
    let (s, e) = field_span(empty, 0).unwrap();
    assert_eq!(s, e, "empty field is a zero-width span");

    // Multibyte: a `°` (2 bytes, 1 char) ahead of the target shifts the
    // span by ONE char, not two bytes.
    let mb = r#""DATA","°C","42""#;
    let mchars: Vec<char> = mb.chars().collect();
    let mslice = |(s, e): (u32, u32)| mchars[s as usize..e as usize].iter().collect::<String>();
    assert_eq!(mslice(field_span(mb, 0).unwrap()), "°C");
    assert_eq!(mslice(field_span(mb, 1).unwrap()), "42");
}

mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    fn encode_field(field: &str) -> String {
        format!("\"{}\"", field.replace('"', "\"\""))
    }
    fn encode_line(fields: &[String]) -> String {
        fields
            .iter()
            .map(|f| encode_field(f))
            .collect::<Vec<_>>()
            .join(",")
    }
    fn field_value() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                prop::char::range(' ', '~'),
                Just('"'),
                Just(','),
                prop::char::range('\u{00a1}', '\u{017f}'),
                prop::char::range('\u{4e00}', '\u{4e80}'),
                Just('°'),
                Just('🦀'),
            ]
            .prop_filter("no line terminators", |c| *c != '\r' && *c != '\n'),
            0..12,
        )
        .prop_map(|cs| cs.into_iter().collect())
    }

    proptest! {
        #[test]
        fn split_ags_line_round_trips_well_formed(fields in prop::collection::vec(field_value(), 0..8)) {
            let line = encode_line(&fields);
            prop_assert_eq!(split_ags_line(&line), fields, "line={:?}", line);
        }

        #[test]
        fn split_ags_line_never_panics(line in ".*") {
            let _ = split_ags_line(&line);
            prop_assert!(true);
        }

        #[test]
        fn field_span_never_panics(line in ".*", idx in 0u32..32) {
            let _ = field_span(&line, idx);
            prop_assert!(true);
        }

        #[test]
        fn field_span_stable_across_line_terminator(
            fields in prop::collection::vec(field_value(), 2..6),
            idx in 0u32..4,
        ) {
            let base = encode_line(&fields);
            prop_assume!((idx as usize + 1) < fields.len());
            let bare = field_span(&base, idx);
            prop_assert_eq!(bare, field_span(&format!("{base}\n"), idx), "LF shifted the span");
            prop_assert_eq!(bare, field_span(&format!("{base}\r\n"), idx), "CRLF shifted the span");
        }

        #[test]
        fn field_span_inner_slice_matches_field(
            fields in prop::collection::vec(field_value(), 2..6),
            idx in 0u32..4,
        ) {
            prop_assume!((idx as usize + 1) < fields.len());
            let line = encode_line(&fields);
            let chars: Vec<char> = line.chars().collect();
            let (s, e) = field_span(&line, idx).expect("field exists");
            let raw: String = chars[s as usize..e as usize].iter().collect();
            let reparsed = split_ags_line(&format!("\"{raw}\""));
            prop_assert_eq!(&reparsed, &vec![fields[idx as usize + 1].clone()]);
        }

        /// 6h — multibyte: `field_span` counts CHARS, so a `°`/`é` field's
        /// char span differs from its byte span by the multibyte delta.
        #[test]
        fn field_span_is_char_not_byte(prefix in "[a-z]{0,5}", tail in "[a-z]{0,5}") {
            let line = format!("\"DATA\",\"{prefix}°{tail}\",\"x\"");
            let (s, e) = field_span(&line, 0).unwrap();
            let char_len = (e - s) as usize;
            // value has prefix + 1 (°) + tail chars
            prop_assert_eq!(char_len, prefix.chars().count() + 1 + tail.chars().count());
        }
    }
}
