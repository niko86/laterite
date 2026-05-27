//! Clean-room AGS4 text emitter.
//!
//! AGS4 Rule 5: every field is enclosed in double quotes; an embedded
//! double quote is doubled (`"` → `""`). Rule 2a: records are CRLF
//! terminated. python-ags4's `dataframe_to_AGS4` reaches the same
//! on-disk bytes via pandas `to_csv(quoting=QUOTE_ALL,
//! lineterminator="\r\n")` plus a hand-written `"GROUP","CODE"` line
//! and a trailing blank line after each group. This emitter produces
//! that structure directly from the spec — not tied to any pandas
//! CSV-dialect quirk — so the Python compat layer can stay
//! dataframe-backend agnostic (it only has to hand us the cell
//! matrix).

/// Wrap one field per Rule 5: surround with `"`, double any embedded
/// `"`.
fn quote_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        if c == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}

fn quote_row(cells: &[String]) -> String {
    let mut out = String::new();
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&quote_field(c));
    }
    out
}

/// One group to emit: its 4-char code and the full cell matrix where
/// `matrix[0]` is the HEADING line (including the literal `"HEADING"`
/// first cell) and every subsequent row starts with its tag
/// (`UNIT`/`TYPE`/`DATA`) — i.e. exactly the rows python-ags4 hands
/// `to_csv` after the GROUP line.
pub struct GroupBlock {
    pub code: String,
    pub matrix: Vec<Vec<String>>,
}

/// Serialise groups to AGS4 text: `"GROUP","CODE"` then every matrix
/// row, all CRLF-terminated, with a blank CRLF line separating groups
/// (matching python-ags4's per-group trailing `\r\n`).
pub fn emit(groups: &[GroupBlock]) -> String {
    let mut s = String::new();
    for g in groups {
        s.push_str(&quote_field("GROUP"));
        s.push(',');
        s.push_str(&quote_field(&g.code));
        s.push_str("\r\n");
        for row in &g.matrix {
            s.push_str(&quote_row(row));
            s.push_str("\r\n");
        }
        s.push_str("\r\n");
    }
    s
}
