//! The quote-aware, universal-newline line splitter (#422).
//!
//! `line_spans` recognises `\r\n` / `\n` / lone `\r` OUTSIDE a quoted field as
//! terminators, and a CR/LF INSIDE a quoted field as embedded content (Rule 6),
//! never a boundary. The load-bearing guarantee is that its line boundaries
//! agree with `split_ags_line`'s field grammar — the property test pins exactly
//! that, so the two layers can never disagree about where a line ends (the class
//! of bug this splitter replaces).

use laterite_ags4_parse::{LineTerminator, line_spans, split_ags_line};
use proptest::prelude::*;

/// The `(body, terminator)` list a buffer splits into.
fn split(s: &str) -> Vec<(&str, LineTerminator)> {
    line_spans(s.as_bytes())
        .map(|sp| (&s[sp.start..sp.body_end], sp.term))
        .collect()
}

#[test]
fn well_formed_crlf_is_unchanged() {
    let s = "\"GROUP\",\"PROJ\"\r\n\"DATA\",\"v\"\r\n";
    assert_eq!(
        split(s),
        vec![
            ("\"GROUP\",\"PROJ\"", LineTerminator::Crlf),
            ("\"DATA\",\"v\"", LineTerminator::Crlf),
        ]
    );
}

#[test]
fn terminator_flavours_are_classified() {
    assert_eq!(split("a\r\n"), vec![("a", LineTerminator::Crlf)]);
    assert_eq!(split("a\n"), vec![("a", LineTerminator::Lf)]);
    assert_eq!(split("a\r"), vec![("a", LineTerminator::Cr)]);
    assert_eq!(split("a"), vec![("a", LineTerminator::Unterminated)]);
    // No phantom trailing blank line after a terminated buffer.
    assert_eq!(split("a\r\nb\r\n").len(), 2);
}

#[test]
fn lone_cr_between_quoted_rows_splits() {
    // Classic-Mac: a lone `\r` OUTSIDE quotes terminates the row.
    let s = "\"DATA\",\"BH1\"\r\"DATA\",\"BH2\"\r";
    assert_eq!(
        split(s),
        vec![
            ("\"DATA\",\"BH1\"", LineTerminator::Cr),
            ("\"DATA\",\"BH2\"", LineTerminator::Cr),
        ]
    );
}

#[test]
fn cr_and_lf_inside_quotes_are_embedded_not_terminators() {
    // O-2: a CR (or LF, or CRLF) inside a quoted field is content — the row is
    // NOT split, so `split_ags_line` recovers the whole cell.
    for nl in ["\r", "\n", "\r\n"] {
        let s = format!("\"DATA\",\"a{nl}b\"\r\n");
        let spans = split(&s);
        assert_eq!(spans.len(), 1, "embedded {nl:?} must not split");
        assert_eq!(split_ags_line(spans[0].0), vec!["DATA", &format!("a{nl}b")]);
    }
}

#[test]
fn doubled_quotes_do_not_falsely_close_the_field() {
    // `""` is an escaped quote; a CR right after it is still embedded.
    let s = "\"DATA\",\"he said \"\"hi\"\" now\rmore\"\r\n";
    let spans = split(s);
    assert_eq!(
        spans.len(),
        1,
        "escaped quotes must not end the field early"
    );
    assert_eq!(
        split_ags_line(spans[0].0),
        vec!["DATA", "he said \"hi\" now\rmore"]
    );
}

#[test]
fn unterminated_quote_resyncs_at_the_next_descriptor() {
    // Recovery backstop: an unterminated quote would otherwise swallow the next
    // row as embedded content; a following data descriptor forces a boundary,
    // bounding the runaway to one row.
    let s = "\"DATA\",\"oops\r\"DATA\",\"ok\"\r\n";
    let spans = split(s);
    assert_eq!(spans.len(), 2, "backstop splits at the second \"DATA\"");
    assert_eq!(spans[0].1, LineTerminator::Cr);
}

proptest! {
    /// The load-bearing invariant: the splitter's line boundaries agree with
    /// `split_ags_line`'s field grammar. Rows of arbitrary quoted cells (cells
    /// may embed `,`, `\r`, `\n` — NOT `"`, so no escaping/descriptor ambiguity)
    /// joined by any terminator must round-trip: `line_spans` recovers exactly
    /// the rows, and `split_ags_line` on each body recovers exactly the cells.
    #[test]
    fn splitter_and_split_ags_line_never_disagree(
        rows in prop::collection::vec(
            prop::collection::vec(cell(), 1..5),
            1..4,
        ),
        term in prop_oneof![Just("\r\n"), Just("\n"), Just("\r")],
    ) {
        let mut rendered = String::new();
        for row in &rows {
            for (i, c) in row.iter().enumerate() {
                if i > 0 { rendered.push(','); }
                rendered.push('"');
                rendered.push_str(c);
                rendered.push('"');
            }
            rendered.push_str(term);
        }
        let recovered: Vec<Vec<String>> = line_spans(rendered.as_bytes())
            .map(|sp| split_ags_line(&rendered[sp.start..sp.body_end]))
            .collect();
        prop_assert_eq!(recovered, rows);
    }
}

/// Cell content: letters/digits/space plus the interesting embeddables `,`,
/// `\r`, `\n` — but never `"`, so the rendered row needs no escaping and can't
/// accidentally spell a `"DESCRIPTOR"` that would trip the recovery backstop.
fn cell() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            prop::char::range('a', 'z'),
            prop::char::range('0', '9'),
            Just(' '),
            Just(','),
            Just('\r'),
            Just('\n'),
        ],
        0..6,
    )
    .prop_map(|v| v.into_iter().collect())
}
