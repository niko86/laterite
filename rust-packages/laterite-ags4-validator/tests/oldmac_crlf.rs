//! #422 — old-Mac (lone-CR) line terminators.
//!
//! A classic-Mac file terminates rows with a lone `\r` (no `\n`). python-ags4's
//! universal-newline reader handles it gracefully (each row → Rule 2a); our
//! `\n`-only splitter did not — the lone-CR survived as an interior byte, got
//! mislabelled an embedded CR (Rule 6), and `StripEmbeddedCr` deleted it and
//! WELDED rows on a *fix*. These are the acceptance tests for the quote-aware,
//! universal-newline parser: an old-Mac file must parse into the SAME rows as
//! its CRLF twin, fixing must not weld, and a CR *inside a quoted field* must
//! still be embedded content (O-2), never a terminator.

use encoding_rs::UTF_8;
use laterite_ags4_validator::{CheckOptions, fix_document, parse};

/// A small, well-formed AGS4 file rendered with the given line terminator.
fn build(term: &str) -> String {
    const LINES: [&str; 12] = [
        "\"GROUP\",\"PROJ\"",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"",
        "\"UNIT\",\"\",\"\"",
        "\"TYPE\",\"ID\",\"X\"",
        "\"DATA\",\"PR01\",\"Demo Project\"",
        "",
        "\"GROUP\",\"LOCA\"",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_NATN\"",
        "\"UNIT\",\"\",\"m\",\"m\"",
        "\"TYPE\",\"ID\",\"2DP\",\"2DP\"",
        "\"DATA\",\"BH01\",\"1000.00\",\"2000.00\"",
        "\"DATA\",\"BH02\",\"1050.00\",\"2050.00\"",
    ];
    let mut s = LINES.join(term);
    s.push_str(term);
    s
}

#[test]
fn oldmac_parses_into_the_same_rows_as_crlf() {
    let crlf = parse::parse_bytes(build("\r\n").as_bytes(), UTF_8).unwrap();
    let mac = parse::parse_bytes(build("\r").as_bytes(), UTF_8).unwrap();

    // Old-Mac is badly *terminated*, not differently *shaped*: same groups,
    // same headings, same row values as the CRLF twin.
    assert_eq!(
        mac.group_order, crlf.group_order,
        "same groups in same order"
    );
    for code in &crlf.group_order {
        let m = &mac.groups[code];
        let c = &crlf.groups[code];
        assert_eq!(m.headings, c.headings, "{code}: headings");
        let vals = |g: &laterite_ags4_validator::parse::ParsedGroup| -> Vec<Vec<String>> {
            g.rows
                .iter()
                .map(|r| {
                    r.values
                        .iter()
                        .map(|s| s.slice(g.text()).to_string())
                        .collect()
                })
                .collect()
        };
        assert_eq!(vals(m), vals(c), "{code}: row values");
    }
}

#[test]
fn fixing_an_oldmac_file_does_not_weld_rows() {
    let out = fix_document(build("\r").as_bytes(), &CheckOptions::default(), false)
        .expect("fix_document");
    let re = parse::parse_bytes(&out.fixed, UTF_8).expect("re-parse fixed");
    // The rows survive intact (not welded) and the terminators are normalised.
    assert_eq!(
        re.groups.get("LOCA").map(|g| g.rows.len()),
        Some(2),
        "fix must not weld the two LOCA rows"
    );
    assert_eq!(re.groups.get("PROJ").map(|g| g.rows.len()), Some(1));
}

#[test]
fn a_cr_inside_a_quoted_field_is_embedded_not_a_terminator() {
    // O-2 guard: a CR *within quotes* is illegal embedded content (Rule 6), NOT
    // a row terminator — the row must stay whole. Holds before AND after the
    // parser change; it locks the distinction the whole fix rests on.
    let src = "\"GROUP\",\"LOCA\"\r\n\
               \"HEADING\",\"LOCA_ID\"\r\n\
               \"UNIT\",\"\"\r\n\
               \"TYPE\",\"ID\"\r\n\
               \"DATA\",\"a\rb\"\r\n";
    let p = parse::parse_bytes(src.as_bytes(), UTF_8).unwrap();
    assert_eq!(
        p.groups["LOCA"].rows.len(),
        1,
        "embedded CR must not split the row"
    );
    assert_eq!(p.groups["LOCA"].cell(0, 0), Some("a\rb"));
}
