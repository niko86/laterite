//! Property: any cell text (sans raw line breaks) survives write → split — the
//! general form of the Rule-1 quote-escape fix. The writer's quote-everything,
//! `"`→`""`-escaping output must round-trip through the tokenizer for arbitrary
//! values, including embedded `"` and `,`.
//!
//! A raw `\r`/`\n` INSIDE a cell has no faithful AGS4 encoding (Rule 6), so the
//! writer now *rejects* it — `EmitError::EmbeddedNewline` (#423), asserted by
//! `writer::tests::embedded_newline_in_a_cell_is_rejected_by_flavour`. This
//! round-trip covers the *emittable* cells (the escaping invariant), so its
//! strategy still excludes line terminators — a rejected cell has no round-trip.

use laterite_ags4_emit::{EmitGroup, write_ags4};
use laterite_ags4_validator::parse::split_ags_line;
use proptest::prelude::*;

fn cell_no_crlf() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            prop::char::range(' ', '~'),
            Just('"'),
            Just(','),
            prop::char::range('\u{00a1}', '\u{017f}'),
            Just('°'),
            Just('\u{201c}'),
            Just('🦀'),
        ]
        .prop_filter("no line terminators", |c| *c != '\r' && *c != '\n'),
        0..10,
    )
    .prop_map(|cs| cs.into_iter().collect())
}

proptest! {
    #[test]
    fn write_row_round_trips_through_split(cells in prop::collection::vec(cell_no_crlf(), 1..8)) {
        // DATA rows are emitted verbatim (not aligned to the heading count), so
        // the headings/units/types shape is irrelevant to this round-trip.
        let g = EmitGroup {
            code: "PROJ",
            headings: vec!["PROJ_ID"],
            units: vec![""],
            types: vec!["X"],
            rows: vec![cells.clone()],
        };
        let mut buf = Vec::new();
        write_ags4(&mut buf, &[g]).expect("write_ags4");
        let text = String::from_utf8(buf).expect("output is UTF-8");
        let data_line = text
            .lines()
            .find(|l| l.starts_with("\"DATA\""))
            .expect("a DATA line");
        let mut expected: Vec<String> = vec!["DATA".to_string()];
        expected.extend(cells);
        prop_assert_eq!(split_ags_line(data_line), expected);
    }
}
