//! Targeted coverage for `lib.rs` internals surfaced by the mutation sweep
//! (coverage campaign, Rust phase): the `ParsedGroup` accessors, the line
//! terminator/scan machinery, the encoding + BOM + AGS3 gates of the walk, and
//! the `split_ags_line` / `field_span` tokenizers. Each test pins a specific
//! behaviour a survived mutant would otherwise have changed silently.

use laterite_ags4_parse::{
    LineTerminator, ParseError, field_span, line_spans, parse_bytes, parse_str, split_ags_line,
};

/// A minimal two-column LOCA group with one DATA row.
const LOCA: &str = "\"GROUP\",\"LOCA\"\r\n\
                    \"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
                    \"UNIT\",\"\",\"m\"\r\n\
                    \"TYPE\",\"ID\",\"2DP\"\r\n\
                    \"DATA\",\"BH01\",\"523145.10\"\r\n";

#[test]
fn parsed_group_cell_and_col_read_by_position_and_name() {
    let pf = parse_str(LOCA).unwrap();
    let g = &pf.groups["LOCA"];
    // cell(col, row) — by position, None past the row's width or the row count
    assert_eq!(g.cell(0, 0), Some("BH01"));
    assert_eq!(g.cell(1, 0), Some("523145.10"));
    assert_eq!(g.cell(2, 0), None, "past the row width");
    assert_eq!(g.cell(0, 1), None, "past the row count");
    // col(name) — first matching heading index, None when absent
    assert_eq!(g.col("LOCA_ID"), Some(0));
    assert_eq!(g.col("LOCA_NATE"), Some(1));
    assert_eq!(g.col("NOPE"), None);
}

#[test]
fn line_terminator_as_str_is_the_literal_terminator() {
    assert_eq!(LineTerminator::Crlf.as_str(), "\r\n");
    assert_eq!(LineTerminator::Lf.as_str(), "\n");
    assert_eq!(LineTerminator::Cr.as_str(), "\r");
    assert_eq!(LineTerminator::Unterminated.as_str(), "");
}

#[test]
fn split_ags_line_skips_post_quote_junk_and_splits_unquoted() {
    // junk between a closed quote and the next comma is discarded
    assert_eq!(split_ags_line("\"a\"xx,\"b\""), ["a", "b"]);
    // a plain unquoted line splits on every comma
    assert_eq!(split_ags_line("a,b,c"), ["a", "b", "c"]);
}

#[test]
fn field_span_returns_inner_char_offsets_quoted_and_unquoted() {
    // field_index skips the leading tag: index 0 => the 2nd field on the line
    let quoted = "\"AB\",\"CD\",\"EF\"";
    assert_eq!(field_span(quoted, 0), Some((6, 8))); // "CD"
    assert_eq!(field_span(quoted, 1), Some((11, 13))); // "EF"
    let unquoted = "AB,CD,EF";
    assert_eq!(field_span(unquoted, 0), Some((3, 5))); // CD
    assert_eq!(field_span(unquoted, 1), Some((6, 8))); // EF
    assert_eq!(field_span("AB,CD", 5), None, "past the last field");
}

#[test]
fn line_spans_keep_an_escaped_quote_from_swallowing_the_newline() {
    // a doubled "" inside a quoted field must not desync the scanner: the field
    // closes at the real quote, and the trailing \n still splits a second line
    let spans: Vec<_> = line_spans(b"\"aaaaaa\"\"b\"\nNEXT").collect();
    assert_eq!(
        spans.len(),
        2,
        "the escaped quote must not swallow the newline"
    );
    assert_eq!(spans[0].term, LineTerminator::Lf);
    assert_eq!(spans[0].body_end, 11);
}

#[test]
fn parse_bytes_strips_a_leading_bom_for_decode_only() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(LOCA.as_bytes());
    let pf = parse_bytes(&bytes, encoding_rs::UTF_8).unwrap();
    assert!(pf.has_bom, "BOM must be recorded");
    assert!(pf.groups.contains_key("LOCA"), "line-1 group detection");
    // The BOM is stripped for DECODE (it must not survive into the line text) but
    // still counted in the byte offsets. Group detection alone can't see this —
    // the tag is read BOM-tolerantly — so assert the decoded line-1 text directly.
    assert!(
        pf.line_text(&pf.raw_lines[0]).starts_with("\"GROUP\""),
        "BOM leaked into the decoded line-1 text: {:?}",
        pf.line_text(&pf.raw_lines[0])
    );
    // the byte offsets still index the original bytes (BOM bytes are counted)
    assert!(pf.byte_offsets_source_true);
}

#[test]
fn parse_bytes_honours_the_requested_encoding_and_marks_a_re_decoded_line() {
    // 0xA9 is © in Windows-1252 but invalid UTF-8. Decoding under the requested
    // encoding yields the ©; ignoring it (UTF-8 lossy) would yield U+FFFD.
    let src = "\"GROUP\",\"LOCA\"\r\n\
               \"HEADING\",\"LOCA_ID\"\r\n\
               \"UNIT\",\"\"\r\n\
               \"TYPE\",\"X\"\r\n\
               \"DATA\",\"\u{00A9}TAG\"\r\n";
    let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(src);
    let pf = parse_bytes(&bytes, encoding_rs::WINDOWS_1252).unwrap();
    assert_eq!(pf.groups["LOCA"].cell(0, 0), Some("\u{00A9}TAG"));
    // a non-borrowed (re-decoded) line means the offsets can't be vouched for
    assert!(
        !pf.byte_offsets_source_true,
        "a re-decoded line is not source-true"
    );
}

#[test]
fn ags3_markers_are_reported_as_an_unsupported_edition() {
    for marker in ["**ABBR", "<UNITS>", "<CONT>,x"] {
        assert!(
            matches!(
                parse_str(marker),
                Err(ParseError::UnsupportedEdition { .. })
            ),
            "{marker:?} should be flagged as AGS3"
        );
    }
}

#[test]
fn escaped_cells_read_unescaped_through_the_fixup_region() {
    // The dec-parse-cell-representation contract: a cell whose source carries
    // `""` escapes is unescaped ONCE at parse into the buffer's fix-up region,
    // so cell() hands back the logical value while the retained line text
    // stays raw — and both profiles agree on the value.
    let src = "\"GROUP\",\"PROJ\"\r\n\
               \"HEADING\",\"PROJ_ID\",\"PROJ_MEMO\"\r\n\
               \"DATA\",\"P1\",\"say \"\"hi\"\" twice\"\r\n";
    let rich = laterite_ags4_parse::parse_str(src).unwrap();
    let g = &rich.groups["PROJ"];
    assert_eq!(g.cell(0, 0), Some("P1"));
    assert_eq!(g.cell(1, 0), Some("say \"hi\" twice"));
    // The raw-line overlay keeps the source form, escapes included.
    let data_line = rich.raw_lines.last().unwrap();
    assert!(rich.line_text(data_line).contains("\"\"hi\"\""));
    // Same logical values under the lean profile (no raw-line overlay).
    let lean = laterite_ags4_parse::parse_bytes_opts(
        src.as_bytes(),
        laterite_ags4_parse::ParseOptions::lean(),
    )
    .unwrap();
    assert_eq!(lean.groups["PROJ"].cell(1, 0), Some("say \"hi\" twice"));
    // And the span reads agree with the owned tokenizer, field for field.
    let line = "\"DATA\",\"P1\",\"say \"\"hi\"\" twice\"";
    let owned = laterite_ags4_parse::split_ags_line(line);
    let vals: Vec<&str> = (0..owned.len() - 1)
        .map(|c| g.cell(c, 0).unwrap())
        .collect();
    assert_eq!(
        vals,
        owned[1..].iter().map(String::as_str).collect::<Vec<_>>()
    );
}

/// LOCA with a ragged DATA row (one value short of the two headings) and a
/// long DATA row (one value past them) — the shapes `value_at` /
/// `padded_row_strings` exist to make safe (#844).
const LOCA_RAGGED: &str = "\"GROUP\",\"LOCA\"\r\n\
                           \"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
                           \"UNIT\",\"\",\"m\"\r\n\
                           \"TYPE\",\"ID\",\"2DP\"\r\n\
                           \"DATA\",\"BH01\"\r\n\
                           \"DATA\",\"BH02\",\"1.00\",\"extra\"\r\n";

#[test]
fn value_at_reads_row_relative_against_the_owning_buffer() {
    let pf = parse_str(LOCA_RAGGED).unwrap();
    let g = &pf.groups["LOCA"];
    let short = &g.rows[0];
    let long = &g.rows[1];
    assert_eq!(g.value_at(short, 0), Some("BH01"));
    assert_eq!(g.value_at(short, 1), None, "past the ragged row's width");
    assert_eq!(
        g.value_at(long, 2),
        Some("extra"),
        "past the headings is fine"
    );
    // agrees with the (col, row) accessor where both apply
    assert_eq!(g.value_at(short, 0), g.cell(0, 0));
}

#[test]
fn padded_row_strings_pads_and_truncates_to_n() {
    let pf = parse_str(LOCA_RAGGED).unwrap();
    let g = &pf.groups["LOCA"];
    // a short row's tail fills with ""
    assert_eq!(g.padded_row_strings(&g.rows[0], 2), vec!["BH01", ""]);
    // a long row truncates to n
    assert_eq!(g.padded_row_strings(&g.rows[1], 2), vec!["BH02", "1.00"]);
    // n = 0 materialises nothing
    assert!(g.padded_row_strings(&g.rows[0], 0).is_empty());
}
