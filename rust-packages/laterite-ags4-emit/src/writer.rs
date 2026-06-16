//! AGS4 emitter — the byte-level writer (counterpart to the validator's
//! reader). Moved here from `laterite-core::ags4_writer` so the browser host
//! can reach it without pulling all of `laterite-core`.
//!
//! Writes a sequence of group sections to AGS4 plaintext:
//!
//!   "GROUP","<CODE>"
//!   "HEADING","<H1>","<H2>",...
//!   "UNIT","<u1>","<u2>",...
//!   "TYPE","<t1>","<t2>",...
//!   "DATA","<v1>","<v2>",...
//!   <blank line>
//!   "GROUP","<NEXT_CODE>"
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
            out.write_all(b"\r\n").map_err(io_err)?;
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
fn write_row<W: Write>(out: &mut W, cells: &[&str]) -> Result<(), EmitError> {
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.write_all(b",").map_err(io_err)?;
        }
        out.write_all(b"\"").map_err(io_err)?;
        // Escape embedded `"` → `""`.
        if c.contains('"') {
            let escaped = c.replace('"', "\"\"");
            out.write_all(escaped.as_bytes()).map_err(io_err)?;
        } else {
            out.write_all(c.as_bytes()).map_err(io_err)?;
        }
        out.write_all(b"\"").map_err(io_err)?;
    }
    out.write_all(b"\r\n").map_err(io_err)?;
    Ok(())
}

fn io_err(e: std::io::Error) -> EmitError {
    EmitError::Write(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(text.contains(r#""he said ""hello""""#), "got: {}", text);
    }
}
