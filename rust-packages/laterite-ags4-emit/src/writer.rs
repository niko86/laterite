//! AGS4 emitter — the byte-level writer (counterpart to the validator's
//! reader). Moved here from `laterite-ags4-core::ags4_writer` so the browser host
//! can reach it without pulling all of `laterite-ags4-core`.
//!
//! Writes a sequence of group sections to AGS4 plaintext:
//!
//!   "GROUP","<CODE>"
//!   "HEADING","<H1>","<H2>",...
//!   "UNIT","<u1>","<u2>",...
//!   "TYPE","<t1>","<t2>",...
//!   "DATA","<v1>","<v2>",...
//!   <blank line>
//!   "GROUP","<`NEXT_CODE`>"
//!   ...
//!
//! Each cell is double-quote-wrapped (Rule 5); embedded `"` becomes
//! `""`. Lines end CRLF per Rule 2a. Sections separated by a blank line
//! (`\r\n` only).

use std::io::Write;

use crate::error::EmitError;

/// One group ready to emit. Heading / unit / type / row order match the
/// AGS4 file order. `rows` holds one inner Vec per data row, each item
/// being the raw AGS4 string value aligned with `headings`.
pub struct EmitGroup<'a> {
    pub code: &'a str,
    pub headings: Vec<&'a str>,
    pub units: Vec<&'a str>,
    pub types: Vec<&'a str>,
    pub rows: Vec<Vec<String>>,
}

/// Write an AGS4 file. Sections emitted in the order given.
pub fn write_ags4<W: Write>(out: &mut W, groups: &[EmitGroup<'_>]) -> Result<(), EmitError> {
    for (i, g) in groups.iter().enumerate() {
        if i > 0 {
            // Blank line separator (AGS4 doesn't strictly require this,
            // but every emitter I've seen produces it and python-ags4 +
            // our own reader treat the blank as a section break).
            out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
        }
        // GROUP row
        write_row(out, &["GROUP", g.code])?;
        // HEADING row
        let mut h_row = Vec::with_capacity(g.headings.len() + 1);
        h_row.push("HEADING");
        h_row.extend(g.headings.iter().copied());
        write_row(out, &h_row)?;
        // UNIT row — pad to heading length so column count is stable
        write_aligned(out, "UNIT", &g.units, g.headings.len())?;
        // TYPE row — same alignment rule, default missing entries to "X"
        write_aligned_with_default(out, "TYPE", &g.types, g.headings.len(), "X")?;
        // DATA rows
        for row in &g.rows {
            let mut cells: Vec<&str> = Vec::with_capacity(row.len() + 1);
            cells.push("DATA");
            cells.extend(row.iter().map(String::as_str));
            write_row(out, &cells)?;
        }
    }
    Ok(())
}

/// Write AGS4 from a **pre-shaped cell matrix**, one group at a time, verbatim.
///
/// Where [`write_ags4`] is a *structured* writer (it owns the `HEADING`/`UNIT`/`TYPE`/
/// `DATA` tags, aligns columns, and defaults a missing TYPE to `X`), this is a *dumb*
/// serializer: it quotes and CRLF-terminates exactly the rows it is handed, adding
/// nothing. Each group is a `(code, matrix)` where `matrix[0]` is the full HEADING line
/// (including the literal `"HEADING"` first cell) and every later row already carries its
/// `UNIT`/`TYPE`/`DATA` tag in column 0 — i.e. python-ags4's dataframe shape.
///
/// It exists so the `laterite.compat` drop-in can stay **byte-faithful to python-ags4**
/// (which serializes its dataframe with no structural interpretation and no `X` default),
/// while still going through the ONE guarded [`write_row`] — the reason this was moved out
/// of `laterite-py`'s own private emitter, which lacked the embedded-CR/LF guard and so
/// could split a DATA row across two physical lines (#423). Every consumer now shares that
/// guard; there is no unguarded emitter left.
///
/// `trailing_blank_line` appends one blank CRLF line after the final group — the shape
/// python-ags4's `dataframe_to_AGS4` produces (`compat` passes `true`; the structured
/// re-emit that backs `Ags4File.text` passes `false`, matching every other surface).
pub fn write_ags4_matrix<W: Write>(
    out: &mut W,
    groups: &[(String, Vec<Vec<String>>)],
    trailing_blank_line: bool,
) -> Result<(), EmitError> {
    for (i, (code, matrix)) in groups.iter().enumerate() {
        // Blank line BETWEEN groups (both shapes) — the section separator, same as
        // `write_ags4`'s `if i > 0`. The private emitter folded this into a per-group
        // trailing blank, which is why dropping that blank for `.text` also needs this
        // to keep the separator.
        if i > 0 {
            out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
        }
        write_row(out, &["GROUP", code])?;
        for row in matrix {
            let cells: Vec<&str> = row.iter().map(String::as_str).collect();
            write_row(out, &cells)?;
        }
    }
    // python-ags4 appends a blank line AFTER the final group too; the canonical shape
    // does not. This is the only byte-level difference between the two.
    if trailing_blank_line && !groups.is_empty() {
        out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
    }
    Ok(())
}

fn write_aligned<W: Write>(
    out: &mut W,
    tag: &str,
    cells: &[&str],
    n: usize,
) -> Result<(), EmitError> {
    let mut row: Vec<&str> = Vec::with_capacity(n + 1);
    row.push(tag);
    for i in 0..n {
        row.push(cells.get(i).copied().unwrap_or(""));
    }
    write_row(out, &row)
}

fn write_aligned_with_default<W: Write>(
    out: &mut W,
    tag: &str,
    cells: &[&str],
    n: usize,
    default: &str,
) -> Result<(), EmitError> {
    let mut row: Vec<&str> = Vec::with_capacity(n + 1);
    row.push(tag);
    for i in 0..n {
        let v = cells.get(i).copied().unwrap_or(default);
        row.push(if v.is_empty() { default } else { v });
    }
    write_row(out, &row)
}

/// Emit one CSV row in AGS4's quote-everything style + CRLF terminator.
///
/// Refuses a cell containing an embedded CR/LF: AGS4 (Rule 6) forbids CR/LF
/// within a field and offers no in-field escape, so writing the bytes raw
/// would split the row into extra rows on re-parse — an illegal file (#423).
/// We fail loudly (naming the cell) rather than silently fold it; the
/// writer's contract is faithful-or-fail, and a caller that wants the value
/// cleaned fixes it first (fold CR/LF → space) so the mutation is explicit.
fn write_row<W: Write>(out: &mut W, cells: &[&str]) -> Result<(), EmitError> {
    // Scan the whole row up front so a rejected row leaves NO partial bytes
    // (row-atomic): the writer streams cell-by-cell, so checking mid-loop
    // would already have flushed the earlier cells of the same row.
    if let Some(field) = cells.iter().position(|c| c.contains(['\r', '\n'])) {
        return Err(EmitError::EmbeddedNewline {
            tag: cells.first().copied().unwrap_or("").to_string(),
            field,
        });
    }
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.write_all(b",").map_err(|e| io_err(&e))?;
        }
        // One shared authority for AGS4 field quoting (laterite-types) — the
        // exact primitive the browser tokenizer's wasm calls, so the escaping
        // rule can't drift between the writer and the browser (#533). Streams
        // straight into `out`, so the writer's hot path stays allocation-free.
        laterite_types::write_quoted_field(out, c).map_err(|e| io_err(&e))?;
    }
    out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
    Ok(())
}

fn io_err(e: &std::io::Error) -> EmitError {
    EmitError::Write(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix_two_groups() -> Vec<(String, Vec<Vec<String>>)> {
        let g = |code: &str, id: &str| {
            (
                code.to_string(),
                vec![
                    vec!["HEADING".into(), format!("{code}_ID")],
                    vec!["UNIT".into(), String::new()],
                    vec!["TYPE".into(), "ID".into()],
                    vec!["DATA".into(), id.into()],
                ],
            )
        };
        vec![g("PROJ", "P1"), g("LOCA", "BH01")]
    }

    /// The canonical shape (`.text`, every non-compat surface): a blank line BETWEEN
    /// groups, and NONE after the last.
    #[test]
    fn matrix_canonical_shape_has_no_trailing_blank() {
        let mut out = Vec::new();
        write_ags4_matrix(&mut out, &matrix_two_groups(), false).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.contains("\"DATA\",\"P1\"\r\n\r\n\"GROUP\",\"LOCA\""),
            "one blank BETWEEN"
        );
        assert!(
            s.ends_with("\"DATA\",\"BH01\"\r\n"),
            "no trailing blank: {s:?}"
        );
        assert!(!s.ends_with("\r\n\r\n"));
    }

    /// The python-ags4 shape (`compat`): additionally a blank line AFTER the final group.
    #[test]
    fn matrix_python_shape_adds_a_trailing_blank() {
        let mut out = Vec::new();
        write_ags4_matrix(&mut out, &matrix_two_groups(), true).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.ends_with("\"DATA\",\"BH01\"\r\n\r\n"),
            "trailing blank present: {s:?}"
        );
        // The two shapes differ by EXACTLY that one trailing CRLF.
        let mut canonical = Vec::new();
        write_ags4_matrix(&mut canonical, &matrix_two_groups(), false).unwrap();
        assert_eq!(s, format!("{}\r\n", String::from_utf8(canonical).unwrap()));
    }

    /// The whole reason this moved into the shared crate: the verbatim path is now GUARDED.
    /// A cell with an embedded newline is refused, not serialized into a torn, illegal file.
    #[test]
    fn matrix_refuses_an_embedded_newline() {
        let groups = vec![(
            "PROJ".to_string(),
            vec![
                vec!["HEADING".into(), "PROJ_ID".into()],
                vec!["UNIT".into(), String::new()],
                vec!["TYPE".into(), "ID".into()],
                vec!["DATA".into(), "line1\r\nline2".into()],
            ],
        )];
        let mut out = Vec::new();
        let err = write_ags4_matrix(&mut out, &groups, true).expect_err("must refuse");
        assert!(matches!(err, EmitError::EmbeddedNewline { .. }));
    }

    #[test]
    fn round_trip_via_reader() {
        // Build a tiny fixture, write it, confirm headings / rows survive.
        let groups = vec![EmitGroup {
            code: "PROJ",
            headings: vec!["PROJ_ID", "PROJ_NAME"],
            units: vec!["", ""],
            types: vec!["X", "X"],
            rows: vec![vec!["P1".into(), "Test".into()]],
        }];
        let mut buf: Vec<u8> = Vec::new();
        write_ags4(&mut buf, &groups).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("\"GROUP\",\"PROJ\""));
        assert!(text.contains("\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\""));
        assert!(text.contains("\"DATA\",\"P1\",\"Test\""));
        // CRLF line endings (Rule 2a).
        assert!(text.contains("\r\n"));
    }

    #[test]
    fn embedded_quotes_are_doubled() {
        let groups = vec![EmitGroup {
            code: "PROJ",
            headings: vec!["NOTE"],
            units: vec![""],
            types: vec!["X"],
            rows: vec![vec![r#"he said "hello""#.into()]],
        }];
        let mut buf: Vec<u8> = Vec::new();
        write_ags4(&mut buf, &groups).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains(r#""he said ""hello""""#), "got: {text}");
    }

    /// #423: a cell carrying a raw CR/LF has no faithful AGS4 encoding (Rule 6
    /// forbids it, no in-field escape exists), so the writer refuses rather
    /// than emit bytes that would split the row on re-parse. Each newline
    /// flavour — lone CR, lone LF, CRLF — must be rejected, and the error must
    /// name the offending cell so a caller can locate it.
    #[test]
    fn embedded_newline_in_a_cell_is_rejected_by_flavour() {
        for bad in ["line one\rline two", "line one\nline two", "a\r\nb"] {
            let groups = vec![EmitGroup {
                code: "LOCA",
                headings: vec!["LOCA_ID", "LOCA_REM"],
                units: vec!["", ""],
                types: vec!["ID", "X"],
                // The offending value sits in the 2nd data column (field 2 of
                // the DATA row: field 0 = "DATA", field 1 = "BH1").
                rows: vec![vec!["BH1".into(), bad.into()]],
            }];
            let mut buf: Vec<u8> = Vec::new();
            let err = write_ags4(&mut buf, &groups)
                .expect_err("a cell with an embedded newline must not emit");
            match err {
                EmitError::EmbeddedNewline { tag, field } => {
                    assert_eq!(tag, "DATA", "names the row descriptor");
                    assert_eq!(field, 2, "names the offending field index");
                }
                other => panic!("expected EmbeddedNewline, got {other:?}"),
            }
            // The writer streams, so the group header rows precede the failing
            // DATA row (callers discard the partial buffer on Err) — but the
            // offending DATA row's own bytes are never emitted: "BH1" is a data
            // value, absent from the GROUP/HEADING/UNIT/TYPE rows.
            let text = String::from_utf8_lossy(&buf);
            assert!(
                !text.contains("BH1"),
                "the DATA row must not be emitted: {text:?}"
            );
            assert!(
                !text.contains("line two"),
                "the bad cell must not be emitted: {text:?}"
            );
        }
    }

    /// The reject is surgical: a value that merely *looks* multi-line to a
    /// human but carries no CR/LF (spaces, tabs) still emits fine.
    #[test]
    fn cell_without_cr_or_lf_still_emits() {
        let groups = vec![EmitGroup {
            code: "LOCA",
            headings: vec!["LOCA_ID", "LOCA_REM"],
            units: vec!["", ""],
            types: vec!["ID", "X"],
            rows: vec![vec!["BH1".into(), "line one \t line two".into()]],
        }];
        let mut buf: Vec<u8> = Vec::new();
        write_ags4(&mut buf, &groups).expect("no CR/LF, so nothing to reject");
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("\"line one \t line two\""), "got: {text}");
    }

    /// `write_ags4` puts ONE blank line BETWEEN groups (the `i > 0` guard) and
    /// none before the first — a single-group test can't see either edge.
    #[test]
    fn write_ags4_separates_groups_with_a_blank_line_and_no_leading_one() {
        let groups = vec![
            EmitGroup {
                code: "PROJ",
                headings: vec!["PROJ_ID"],
                units: vec![""],
                types: vec!["ID"],
                rows: vec![vec!["P1".into()]],
            },
            EmitGroup {
                code: "LOCA",
                headings: vec!["LOCA_ID"],
                units: vec![""],
                types: vec!["ID"],
                rows: vec![vec!["BH01".into()]],
            },
        ];
        let mut buf = Vec::new();
        write_ags4(&mut buf, &groups).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.contains("\"DATA\",\"P1\"\r\n\r\n\"GROUP\",\"LOCA\""),
            "one blank line BETWEEN groups: {s:?}"
        );
        assert!(
            s.starts_with("\"GROUP\",\"PROJ\""),
            "no leading blank: {s:?}"
        );
    }

    /// The matrix writer's `i > 0` guard likewise emits no blank BEFORE the
    /// first group — the existing shape tests check the between/trailing blanks
    /// but not the leading edge, so `> 0 → >= 0` slipped through.
    #[test]
    fn write_ags4_matrix_has_no_leading_blank_before_the_first_group() {
        let mut out = Vec::new();
        write_ags4_matrix(&mut out, &matrix_two_groups(), false).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(
            s.starts_with("\"GROUP\",\"PROJ\""),
            "must not start with a blank line: {s:?}"
        );
    }
}
