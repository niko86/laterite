//! Line numbers are PHYSICAL lines — the ones an editor shows and python-ags4
//! reports (#1052). A quoted field spanning a newline stays one record (O-47),
//! reported at the line it starts on; every record after it is numbered past
//! the embedded newlines, not one per record.

use laterite_ags4_parse::{numbered_line_spans, parse_str};

/// PROJ's only DATA row carries a quoted value spanning two physical lines
/// (5 and 6); LOCA follows on lines 8-12.
fn spanning(embedded: &str) -> String {
    format!(
        "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
         \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
         \"DATA\",\"P1\",\"two{embedded}lines\"\r\n\r\n\
         \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
         \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH01\"\r\n"
    )
}

/// One physical newline each: CRLF counts once, not twice.
const EMBEDDED: [&str; 3] = ["\r\n", "\n", "\r"];

#[test]
fn rows_after_a_spanning_field_report_their_physical_line() {
    for nl in EMBEDDED {
        let pf = parse_str(&spanning(nl)).unwrap();
        let proj = &pf.groups["PROJ"];
        assert_eq!(
            proj.rows[0].line, 5,
            "{nl:?}: the spanning row is reported where it starts"
        );

        let loca = &pf.groups["LOCA"];
        assert_eq!(
            (
                loca.group_line,
                loca.heading_line,
                loca.unit_line,
                loca.type_line
            ),
            (8, Some(9), Some(10), Some(11)),
            "{nl:?}"
        );
        assert_eq!(loca.rows[0].line, 12, "{nl:?}");
        assert_eq!(pf.group_records[1].line, 8, "{nl:?}");

        let numbers: Vec<u32> = pf.raw_lines.iter().map(|l| l.number).collect();
        assert_eq!(numbers, [1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12], "{nl:?}");
        // A record count, not a physical one: 11 records over 12 lines.
        assert_eq!(pf.total_lines, 11, "{nl:?}");
    }
}

#[test]
fn numbered_spans_count_every_embedded_newline() {
    // Two embedded newlines in one field: the next record starts two lines on.
    let src = "\"DATA\",\"a\r\nb\nc\"\r\n\"DATA\",\"d\"\r\n";
    let lines: Vec<u32> = numbered_line_spans(src.as_bytes())
        .map(|(n, _)| n)
        .collect();
    assert_eq!(lines, [1, 4]);
}

#[test]
fn a_newline_outside_quotes_is_a_terminator_not_an_embedded_line() {
    let src = "\"DATA\",\"a\"\n\"DATA\",\"b\"\r\"DATA\",\"c\"";
    let lines: Vec<u32> = numbered_line_spans(src.as_bytes())
        .map(|(n, _)| n)
        .collect();
    assert_eq!(lines, [1, 2, 3]);
}
