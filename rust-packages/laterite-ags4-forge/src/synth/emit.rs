//! The streaming AGS4 emitter — turns a [`ProjectModel`] into bytes.
//!
//! It writes each group's GROUP/HEADING/UNIT/TYPE/DATA lines with every
//! field double-quoted, CRLF line terminators (Rule 2a), a single blank
//! line between consecutive groups (none before the first / after the
//! last), and a trailing CRLF — byte-for-byte the layout the old
//! `assemble()` produced. Streaming (`io::Write`) so a later GB-scale
//! PR never has to materialise the whole file as one `String`.

use std::io::{self, Write};

use super::model::{Group, ProjectModel, Row};

/// Write `model` to `w` as AGS4 bytes (see module doc for the exact
/// layout contract).
pub fn emit<W: Write>(model: &ProjectModel, w: &mut W) -> io::Result<()> {
    for (i, group) in model.groups.iter().enumerate() {
        // One blank line between consecutive groups — never before the
        // first, never a trailing blank after the last.
        if i > 0 {
            write_line(w, "")?;
        }
        emit_group(group, w)?;
    }
    Ok(())
}

/// Convenience wrapper: emit into a `String` (the back-compat
/// `synth()` return shape). AGS4 is ASCII-quoted text, so the bytes are
/// always valid UTF-8.
pub fn emit_to_string(model: &ProjectModel) -> String {
    let mut buf = Vec::new();
    emit(model, &mut buf).expect("writing to a Vec<u8> cannot fail");
    String::from_utf8(buf).expect("AGS4 emitter output is valid UTF-8")
}

fn emit_group<W: Write>(group: &Group, w: &mut W) -> io::Result<()> {
    write_line(w, &row_line(&["GROUP", &group.code]))?;
    write_line(w, &descriptor_line("HEADING", &group.headings))?;
    write_line(w, &descriptor_line("UNIT", &group.units))?;
    write_line(w, &descriptor_line("TYPE", &group.types))?;
    for row in &group.rows {
        write_line(w, &data_line(row))?;
    }
    Ok(())
}

/// A DATA line, honouring the row's [`RowFault`](super::model::RowFault)s:
/// the `DATA` tag and every value are quoted, except a cell marked
/// `Unquote` (which emits raw — a Rule 5 violation). A fault-free row
/// therefore emits exactly as `descriptor_line("DATA", …)` would.
fn data_line(row: &Row) -> String {
    let mut cells = Vec::with_capacity(row.values.len() + 1);
    cells.push("\"DATA\"".to_string());
    for (i, value) in row.values.iter().enumerate() {
        cells.push(if row.is_unquoted(i) {
            value.clone()
        } else {
            format!("\"{value}\"")
        });
    }
    cells.join(",")
}

/// `"<tag>","f0","f1",…` — the AGS4 line shape, every field quoted.
fn descriptor_line(tag: &str, fields: &[String]) -> String {
    let mut cells = Vec::with_capacity(fields.len() + 1);
    cells.push(tag.to_string());
    cells.extend(fields.iter().cloned());
    row_line(&cells.iter().map(String::as_str).collect::<Vec<_>>())
}

/// Join `cells` as quoted, comma-separated AGS4 fields (no terminator).
fn row_line(cells: &[&str]) -> String {
    cells
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(",")
}

/// One AGS4 line: the content followed by the CRLF terminator.
fn write_line<W: Write>(w: &mut W, content: &str) -> io::Result<()> {
    w.write_all(content.as_bytes())?;
    w.write_all(b"\r\n")
}
