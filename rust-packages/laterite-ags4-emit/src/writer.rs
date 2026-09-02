//! AGS4 emitter — the byte-level writer (counterpart to the validator's
//! reader). Moved here from `laterite-ags4-core::ags4_writer` so the browser host
//! can reach it without pulling all of `laterite-ags4-core`.
//!
//! Writes a sequence of group sections to AGS4 plaintext:
//!
//! ```text
//! "GROUP","<CODE>"
//! "HEADING","<H1>","<H2>",...
//! "UNIT","<u1>","<u2>",...
//! "TYPE","<t1>","<t2>",...
//! "DATA","<v1>","<v2>",...
//! <blank line>
//! "GROUP","<NEXT_CODE>"
//! ...
//! ```
//!
//! Each cell is double-quote-wrapped (Rule 5); embedded `"` becomes
//! `""`. Lines end CRLF per Rule 2a. Sections separated by a blank line
//! (`\r\n` only).

use std::io::Write;

use laterite_ags4_parse::ParsedFile;
use laterite_ags4_parse::builder::{ParsedFileBuilder, RecordedCell, RowTag};

use crate::error::EmitError;

/// One group ready to emit. Heading / unit / type / row order match the
/// AGS4 file order. `rows` holds one inner Vec per data row, each item
/// being the raw AGS4 string value aligned with `headings`.
///
/// Borrowed, like every other field: the writer only reads. This field used
/// to be owned (`Vec<Vec<String>>`), which made every caller clone its whole
/// formatted table to build the view — for the emit orchestrator that clone
/// was a fourth simultaneous copy of every cell, live across the write and
/// the validating re-parse (measured with `examples/heap_profile.rs`: ~29
/// of ~215 requested bytes per cell at peak; #788).
pub struct EmitGroup<'a> {
    pub code: &'a str,
    pub headings: Vec<&'a str>,
    pub units: Vec<&'a str>,
    pub types: Vec<&'a str>,
    pub rows: &'a [Vec<String>],
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
        write_row(out, &aligned_cells("UNIT", &g.units, g.headings.len(), ""))?;
        // TYPE row — same alignment rule, default missing entries to "X"
        write_row(out, &aligned_cells("TYPE", &g.types, g.headings.len(), "X"))?;
        // DATA rows
        for row in g.rows {
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
/// while still going through the ONE guarded `write_row` — the reason this was moved out
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
    let mut stream = MatrixStream::new(out, trailing_blank_line);
    for (code, matrix) in groups {
        stream.group(code, matrix)?;
    }
    stream.finish().map(drop)
}

/// The canonical `(code, matrix)` blocks of a retained parse — the input
/// shape [`write_ags4_matrix`] serialises for the structured re-emit behind
/// `Ags4File.text` (laterite-py's `Reading::emit`) and the xcheck reference
/// leg (`emit_cases reemit_canonical`). ONE constructor because those two
/// sites carried byte-identical copies that could only drift apart (#847,
/// finishing what #844's shared DATA half started): the HEADING row leads
/// with its literal tag, UNIT/TYPE are tag-prefixed and padded/truncated to
/// the heading count (a ragged descriptor's tail fills with `""` —
/// python-ags4's `_matrix` shape, and the reason a descriptor LONGER than
/// the headings loses its overhang), and DATA rows pad through the parse
/// leaf's `padded_row_strings`.
pub fn canonical_matrix_blocks(parsed: &ParsedFile) -> Vec<(String, Vec<Vec<String>>)> {
    parsed
        .group_order
        .iter()
        .filter_map(|code| {
            let g = parsed.groups.get(code)?;
            let n = g.headings.len();
            let pad = |tag: &str, src: &[String]| {
                let mut row = Vec::with_capacity(n + 1);
                row.push(tag.to_string());
                for i in 0..n {
                    row.push(src.get(i).cloned().unwrap_or_default());
                }
                row
            };
            let mut matrix: Vec<Vec<String>> = Vec::with_capacity(3 + g.rows.len());
            let mut heading = Vec::with_capacity(n + 1);
            heading.push("HEADING".to_string());
            heading.extend(g.headings.iter().cloned());
            matrix.push(heading);
            matrix.push(pad("UNIT", &g.units));
            matrix.push(pad("TYPE", &g.types));
            for r in &g.rows {
                let mut data = Vec::with_capacity(n + 1);
                data.push("DATA".to_string());
                data.extend(g.padded_row_strings(r, n));
                matrix.push(data);
            }
            Some((code.clone(), matrix))
        })
        .collect()
}

/// [`write_ags4_matrix`], one group at a time — the streaming door (#805).
///
/// A caller with many groups need not hold every group's matrix (nor the
/// whole output) live at once: convert one group, [`Self::group`] it, drop
/// it, repeat, then [`Self::finish`]. The bytes are IDENTICAL to a single
/// [`write_ags4_matrix`] call over the same groups — that function is now a
/// thin loop over this type, so the two cannot drift, and the differential
/// test pins the equality anyway.
///
/// Failure shape is per ROW (the row writer scans before it emits), but a mid-stream
/// error necessarily leaves the earlier groups already written to `out` —
/// a caller whose `out` is a real file and whose contract is
/// refuse-without-touching (the compat write's) must stage to a temp file
/// and rename on success.
pub struct MatrixStream<W: Write> {
    out: W,
    trailing_blank_line: bool,
    wrote_any: bool,
}

impl<W: Write> MatrixStream<W> {
    pub fn new(out: W, trailing_blank_line: bool) -> Self {
        Self {
            out,
            trailing_blank_line,
            wrote_any: false,
        }
    }

    /// Write one group: the section separator (between groups only, same as
    /// `write_ags4`'s `if i > 0` — the private emitter folded this into a
    /// per-group trailing blank, which is why dropping that blank for `.text`
    /// also needs this to keep the separator), the `GROUP` row, then the
    /// matrix rows verbatim.
    pub fn group(&mut self, code: &str, matrix: &[Vec<String>]) -> Result<(), EmitError> {
        if self.wrote_any {
            self.out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
        }
        self.wrote_any = true;
        write_row(&mut self.out, &["GROUP", code])?;
        for row in matrix {
            let cells: Vec<&str> = row.iter().map(String::as_str).collect();
            write_row(&mut self.out, &cells)?;
        }
        Ok(())
    }

    /// Close the stream, returning the writer. python-ags4 appends a blank
    /// line AFTER the final group too; the canonical shape does not — this is
    /// the only byte-level difference between the two.
    pub fn finish(mut self) -> Result<W, EmitError> {
        if self.trailing_blank_line && self.wrote_any {
            self.out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
        }
        Ok(self.out)
    }
}

/// The ONE padding authority for the aligned descriptor rows: pad `cells` to
/// `n` entries, `fill` standing in for both a MISSING entry and an EMPTY one
/// — `""` for UNIT (where the empty-replacement is a no-op) and `"X"` for
/// TYPE (the free-text default). Shared by the batch loop above and the
/// recording section writer below, so what lands in the file cannot drift
/// between them.
fn aligned_cells<'a>(tag: &'a str, cells: &[&'a str], n: usize, fill: &'a str) -> Vec<&'a str> {
    let mut row: Vec<&str> = Vec::with_capacity(n + 1);
    row.push(tag);
    for i in 0..n {
        let v = cells.get(i).copied().unwrap_or(fill);
        row.push(if v.is_empty() { fill } else { v });
    }
    row
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
    row_atomic_guard(cells)?;
    // The cell loop below is the one shape this function shares with
    // `write_row_recorded`, and the split is the DESIGN, not an oversight
    // (#857 asked): the recorded twin observes `out.len()` around every
    // cell write, which only a concrete byte buffer can yield — a shared
    // loop over `W: Write` would either lose that observation or force a
    // per-cell callback on this, the allocation-free hot path. The guard
    // above and the quoting authority are the shared parts, and both ARE
    // shared; the differential test pins the two loops' bytes identical.
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.write_all(b",").map_err(|e| io_err(&e))?;
        }
        // One shared authority for AGS4 field quoting (laterite-ags4-types) — the
        // exact primitive the browser tokenizer's wasm calls, so the escaping
        // rule can't drift between the writer and the browser (laterite-dev#533). Streams
        // straight into `out`, so the writer's hot path stays allocation-free.
        laterite_ags4_types::write_quoted_field(out, c).map_err(|e| io_err(&e))?;
    }
    out.write_all(b"\r\n").map_err(|e| io_err(&e))?;
    Ok(())
}

/// The row-atomic Rule-6 refusal both row writers share: scan the whole row
/// BEFORE any byte lands, so a rejected row leaves no partial output — the
/// writers stream cell-by-cell, and checking mid-loop would already have
/// flushed the earlier cells of the same row (#423).
fn row_atomic_guard(cells: &[&str]) -> Result<(), EmitError> {
    if let Some(field) = cells.iter().position(|c| c.contains(['\r', '\n'])) {
        return Err(EmitError::EmbeddedNewline {
            tag: cells.first().copied().unwrap_or("").to_string(),
            field,
        });
    }
    Ok(())
}

fn io_err(e: &std::io::Error) -> EmitError {
    EmitError::Write(e.to_string())
}

/// [`write_row`] into a byte buffer, additionally recording where each
/// cell's LOGICAL value landed (between its quotes — escapes still doubled,
/// flagged as such). Same #423 guard, same quoting authority
/// (`write_quoted_field`), and the positions are READ BACK from the buffer
/// around each write rather than predicted from the escaping rule — the
/// record observes the writer, it does not model it (the M4 tokenizer
/// lesson, `dec-parse-cell-representation`). Returns the row body's range
/// (tag included, CRLF excluded).
fn write_row_recorded(
    out: &mut Vec<u8>,
    cells: &[&str],
    recs: &mut Vec<RecordedCell>,
) -> Result<std::ops::Range<usize>, EmitError> {
    row_atomic_guard(cells)?;
    // The cell loop deliberately mirrors `write_row`'s rather than sharing
    // it (#857): this side's whole job is reading `out.len()` back around
    // each cell write — an observation a `W: Write` sink cannot yield — so
    // the loops share their guard and their quoting authority, and split
    // exactly where the observation happens. The differential test holds
    // their bytes identical.
    recs.clear();
    let start = out.len();
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        let quoted_at = out.len();
        laterite_ags4_types::write_quoted_field(out, c).map_err(|e| io_err(&e))?;
        recs.push(RecordedCell {
            start: quoted_at + 1,
            end: out.len() - 1,
            has_escape: c.contains('"'),
        });
    }
    let end = out.len();
    out.extend_from_slice(b"\r\n");
    Ok(start..end)
}

/// A builder-side failure surfaced while recording — an internal limit (the
/// `u32` span space) rather than an io fault, but a Write error is what the
/// callers' contract already carries for "the output could not be produced".
/// By value because `map_err` hands one over.
#[allow(clippy::needless_pass_by_value)]
fn builder_err(e: laterite_ags4_parse::ParseError) -> EmitError {
    EmitError::Write(format!("verdict record: {e:?}"))
}

/// One group's SECTION through the recording writer, into the verdict
/// builder's buffer: the same bytes `write_ags4` emits for this group — the
/// `!first` blank separator, the GROUP/HEADING/UNIT/TYPE rows (one padding
/// authority, `aligned_cells`), the DATA rows — pinned byte-identical by the
/// differential test below, plus the structural record the writer-built
/// verdict is assembled from (`dec-emit-streamed-verdict`).
pub(crate) fn write_section_recorded(
    b: &mut ParsedFileBuilder,
    g: &EmitGroup<'_>,
    first: bool,
) -> Result<(), EmitError> {
    let mut recs: Vec<RecordedCell> = Vec::new();
    if !first {
        let at = b.written();
        b.buf().extend_from_slice(b"\r\n");
        b.record_row(RowTag::Blank, at..at, &[])
            .map_err(builder_err)?;
    }
    let n = g.headings.len();

    let body = write_row_recorded(b.buf(), &["GROUP", g.code], &mut recs)?;
    b.record_row(RowTag::Group, body, &recs[1..])
        .map_err(builder_err)?;

    let mut h_row: Vec<&str> = Vec::with_capacity(n + 1);
    h_row.push("HEADING");
    h_row.extend(g.headings.iter().copied());
    let body = write_row_recorded(b.buf(), &h_row, &mut recs)?;
    b.record_row(RowTag::Heading, body, &recs[1..])
        .map_err(builder_err)?;

    let body = write_row_recorded(b.buf(), &aligned_cells("UNIT", &g.units, n, ""), &mut recs)?;
    b.record_row(RowTag::Unit, body, &recs[1..])
        .map_err(builder_err)?;

    let body = write_row_recorded(b.buf(), &aligned_cells("TYPE", &g.types, n, "X"), &mut recs)?;
    b.record_row(RowTag::Type, body, &recs[1..])
        .map_err(builder_err)?;

    for row in g.rows {
        let mut cells: Vec<&str> = Vec::with_capacity(row.len() + 1);
        cells.push("DATA");
        cells.extend(row.iter().map(String::as_str));
        let body = write_row_recorded(b.buf(), &cells, &mut recs)?;
        b.record_row(RowTag::Data, body, &recs[1..])
            .map_err(builder_err)?;
    }
    Ok(())
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
        let rows_1 = vec![vec!["P1".into(), "Test".into()]];
        let groups = vec![EmitGroup {
            code: "PROJ",
            headings: vec!["PROJ_ID", "PROJ_NAME"],
            units: vec!["", ""],
            types: vec!["X", "X"],
            rows: rows_1.as_slice(),
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
        let rows_2 = vec![vec![r#"he said "hello""#.into()]];
        let groups = vec![EmitGroup {
            code: "PROJ",
            headings: vec!["NOTE"],
            units: vec![""],
            types: vec!["X"],
            rows: rows_2.as_slice(),
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
            let rows_3 = vec![vec!["BH1".into(), bad.into()]];
            let groups = vec![EmitGroup {
                code: "LOCA",
                headings: vec!["LOCA_ID", "LOCA_REM"],
                units: vec!["", ""],
                types: vec!["ID", "X"],
                // The offending value sits in the 2nd data column (field 2 of
                // the DATA row: field 0 = "DATA", field 1 = "BH1").
                rows: rows_3.as_slice(),
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
        let rows_4 = vec![vec!["BH1".into(), "line one \t line two".into()]];
        let groups = vec![EmitGroup {
            code: "LOCA",
            headings: vec!["LOCA_ID", "LOCA_REM"],
            units: vec!["", ""],
            types: vec!["ID", "X"],
            rows: rows_4.as_slice(),
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
        let rows_5 = vec![vec!["P1".into()]];
        let rows_6 = vec![vec!["BH01".into()]];
        let groups = vec![
            EmitGroup {
                code: "PROJ",
                headings: vec!["PROJ_ID"],
                units: vec![""],
                types: vec!["ID"],
                rows: rows_5.as_slice(),
            },
            EmitGroup {
                code: "LOCA",
                headings: vec!["LOCA_ID"],
                units: vec![""],
                types: vec!["ID"],
                rows: rows_6.as_slice(),
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

    /// The streaming door and the batch door are ONE serializer: identical
    /// bytes for the same groups, across both trailing shapes and the empty
    /// input. `write_ags4_matrix` is a loop over `MatrixStream`, so this is a
    /// pin against future divergence, not a proof of today's.
    #[test]
    fn matrix_stream_matches_the_batch_door_byte_for_byte() {
        let groups: Vec<(String, Vec<Vec<String>>)> = vec![
            (
                "PROJ".into(),
                vec![
                    vec!["HEADING".into(), "PROJ_ID".into()],
                    vec!["UNIT".into(), String::new()],
                    vec!["TYPE".into(), "ID".into()],
                    vec!["DATA".into(), "P1".into()],
                ],
            ),
            (
                "LOCA".into(),
                vec![
                    vec!["HEADING".into(), "LOCA_ID".into()],
                    vec!["DATA".into(), "quote \"inside\"".into()],
                ],
            ),
        ];
        for trailing in [false, true] {
            for n in 0..=groups.len() {
                let subset = &groups[..n];
                let mut batch = Vec::new();
                write_ags4_matrix(&mut batch, subset, trailing).expect("batch door");
                let mut stream = MatrixStream::new(Vec::new(), trailing);
                for (code, matrix) in subset {
                    stream.group(code, matrix).expect("stream group");
                }
                let streamed = stream.finish().expect("stream finish");
                assert_eq!(batch, streamed, "trailing={trailing} n={n}");
            }
        }
    }

    /// The stream refuses an embedded newline exactly as the batch door does
    /// — the #423 guard rides `write_row`, which both share.
    #[test]
    fn matrix_stream_keeps_the_embedded_newline_refusal() {
        let mut stream = MatrixStream::new(Vec::new(), true);
        let bad = vec![vec!["DATA".into(), "torn\nrow".into()]];
        assert!(stream.group("PROJ", &bad).is_err());
    }

    /// The recording section writer and `write_ags4` are ONE section shape:
    /// byte-identical output over every awkward case the shape carries —
    /// separator discipline, UNIT/TYPE padding + the `X` default, escaped
    /// quotes, ragged rows, an empty group. This is what licenses the
    /// streamed pipeline to replace the batch write (dec-emit-streamed-
    /// verdict); a drift between them is a byte-fidelity bug, not a nit.
    #[test]
    fn recorded_sections_match_write_ags4_byte_for_byte() {
        let rows_a = vec![
            vec!["P1".into(), "say \"hi\"".into()],
            vec!["P2".into()], // ragged — shorter than the headings
        ];
        let rows_b: Vec<Vec<String>> = vec![];
        let groups = [
            EmitGroup {
                code: "PROJ",
                headings: vec!["PROJ_ID", "PROJ_NAME"],
                units: vec![""],       // short: padding fills the tail
                types: vec!["ID", ""], // empty: the X default fills
                rows: rows_a.as_slice(),
            },
            EmitGroup {
                code: "ZZZZ",
                headings: vec![],
                units: vec![],
                types: vec![],
                rows: rows_b.as_slice(),
            },
        ];
        for n in 1..=groups.len() {
            let mut batch = Vec::new();
            write_ags4(&mut batch, &groups[..n]).expect("batch door");
            let mut b = ParsedFileBuilder::new();
            for (i, g) in groups[..n].iter().enumerate() {
                write_section_recorded(&mut b, g, i == 0).expect("recorded door");
            }
            let (built, emitted_len) = b.finish().expect("assembles");
            assert_eq!(
                batch,
                built.text.as_bytes()[..emitted_len],
                "the recorded writer drifted from write_ags4 at n={n}"
            );
        }
    }

    /// What the record RESOLVES matches what was written: the escaped cell
    /// reads back unescaped, the ragged row keeps its own arity, and the
    /// padded/defaulted descriptor rows carry the padded values.
    #[test]
    fn recorded_structure_resolves_to_the_written_values() {
        let rows = vec![vec!["P1".into(), "say \"hi\"".into()], vec!["P2".into()]];
        let g = EmitGroup {
            code: "PROJ",
            headings: vec!["PROJ_ID", "PROJ_NAME"],
            units: vec![""],
            types: vec!["ID", ""],
            rows: rows.as_slice(),
        };
        let mut b = ParsedFileBuilder::new();
        write_section_recorded(&mut b, &g, true).expect("recorded door");
        let (built, _) = b.finish().expect("assembles");
        let pg = &built.groups["PROJ"];
        assert_eq!(pg.headings, ["PROJ_ID", "PROJ_NAME"]);
        assert_eq!(pg.units, ["", ""], "padded to heading arity");
        assert_eq!(pg.types, ["ID", "X"], "empty TYPE defaulted to X");
        assert_eq!(pg.cell(1, 0), Some("say \"hi\""), "unescaped via fix-up");
        assert_eq!(pg.rows[1].n_values(), 1, "ragged row keeps its arity");
        assert_eq!(pg.cell(1, 1), None);
    }

    #[test]
    fn canonical_blocks_pad_descriptors_and_data_to_heading_arity() {
        // The #847 constructor's whole contract: tags in column 0, and every
        // row squared to the heading count — a short UNIT tail fills with "",
        // a TYPE longer than the headings loses its overhang (the `_matrix`
        // shape both former copies implemented), and a ragged DATA row pads
        // through the parse leaf's shared helper.
        let text = "\"GROUP\",\"PROJ\"\r\n\
                    \"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                    \"UNIT\",\"u1\"\r\n\
                    \"TYPE\",\"ID\",\"X\",\"2DP\"\r\n\
                    \"DATA\",\"P1\"\r\n";
        let parsed = laterite_ags4_parse::parse_str(text).expect("fixture parses");
        let blocks = canonical_matrix_blocks(&parsed);
        assert_eq!(blocks.len(), 1);
        let (code, matrix) = &blocks[0];
        assert_eq!(code, "PROJ");
        assert_eq!(matrix[0], ["HEADING", "PROJ_ID", "PROJ_NAME"]);
        assert_eq!(matrix[1], ["UNIT", "u1", ""], "short UNIT tail fills");
        assert_eq!(matrix[2], ["TYPE", "ID", "X"], "long TYPE overhang drops");
        assert_eq!(matrix[3], ["DATA", "P1", ""], "ragged DATA row pads");
    }
}
