//! Structured edits to a real AGS4 file (#655).
//!
//! The investigation behind #653 produced three wrong results in a row from
//! hand-manipulating AGS text: a value containing a comma torn in half by a
//! naive split, line endings converted by a well-meaning text reader, and a
//! ragged row that made one validator bail and looked for a while like a
//! divergence. None of those were interesting; all of them cost a session.
//! This is the layer that makes them impossible, so constructing a repro or a
//! fixture stops meaning "edit the text and hope".
//!
//! **Untouched lines are byte-verbatim.** The file is walked as lines with
//! their own terminators recorded, edits are applied by line, and anything no
//! operation names is written back exactly as it arrived — so a run with no
//! operations returns the input unchanged, and a one-cell edit leaves every
//! other byte alone. That is the property a reproducer needs: the difference
//! between the input and the output IS the edit.
//!
//! **A touched line is rebuilt canonically** — every field re-quoted, inner
//! quotes doubled. It has to be: splicing a value that contains a comma into a
//! field that was not quoted would tear the row, which is one of the three
//! failures above. Rebuilding is confined to the lines an operation names, so
//! it can never surprise a line nobody asked about.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use laterite_ags4_parse::{ParseOptions, line_spans, parse_bytes_opts, split_ags_line};
use laterite_ags4_types::quote_field;

/// One structured edit. Rows are 1-indexed over a group's DATA rows, the way
/// a reader counts them — not source lines, which move as edits land.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Op {
    /// Write `value` into one cell. A blank is this with an empty value —
    /// spelled separately at the CLI because "blank it" is a different
    /// intention from "set it to nothing" and reads better in a patch file.
    #[serde(rename = "set")]
    SetCell {
        group: String,
        row: usize,
        heading: String,
        value: String,
    },
    /// Append a DATA row. Cells not named are empty, and the row is padded to
    /// the group's heading count so it can never be ragged — the shape that
    /// makes python-ags4's parser bail (O-37).
    AddRow {
        group: String,
        /// Keyed by heading, because a row is addressed by name here — the
        /// positional form is what makes a hand-built row ragged.
        #[serde(default)]
        cells: BTreeMap<String, String>,
    },
    DeleteRow {
        group: String,
        row: usize,
    },
    /// Remove the group entirely: its GROUP/HEADING/UNIT/TYPE rows, its DATA
    /// rows, and the blank line that separated it from the next group.
    DeleteGroup {
        group: String,
    },
    /// Remove one heading and its cell from every row of the group,
    /// descriptor rows included, so the arity stays consistent.
    DeleteColumn {
        group: String,
        heading: String,
    },
}

impl Op {
    fn group(&self) -> &str {
        match self {
            Op::SetCell { group, .. }
            | Op::AddRow { group, .. }
            | Op::DeleteRow { group, .. }
            | Op::DeleteGroup { group }
            | Op::DeleteColumn { group, .. } => group,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    Parse(String),
    NoSuchGroup(String),
    NoSuchHeading {
        group: String,
        heading: String,
    },
    NoSuchRow {
        group: String,
        row: usize,
        rows: usize,
    },
    /// The file declares one GROUP code twice. Every locator here — a row
    /// number, a heading, "the group's lines" — then means two different
    /// things, and the parse leaf is first-seen-wins for rows but
    /// last-seen-wins for headings, so an edit would silently mix them.
    DuplicateGroup {
        group: String,
        lines: Vec<u32>,
    },
    /// A row too short to carry the column being dropped. Removing the column
    /// from its siblings and not from this row leaves exactly the ragged row
    /// this layer exists to prevent.
    ShortRow {
        group: String,
        line: u32,
        fields: usize,
        headings: usize,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::Parse(e) => write!(f, "the file could not be parsed: {e}"),
            EditError::NoSuchGroup(g) => write!(f, "no group {g:?} in this file"),
            EditError::NoSuchHeading { group, heading } => {
                write!(f, "{group} has no heading {heading:?}")
            }
            EditError::NoSuchRow { group, row, rows } => write!(
                f,
                "{group} has {rows} data row(s); there is no row {row} \
                 (rows are 1-indexed)"
            ),
            EditError::DuplicateGroup { group, lines } => write!(
                f,
                "{group} is declared {} times (lines {}), so a row number or \
                 heading here would name two different things; split or merge \
                 the sections before editing",
                lines.len(),
                lines
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EditError::ShortRow {
                group,
                line,
                fields,
                headings,
            } => write!(
                f,
                "{group} line {line} carries {fields} value(s) for {headings} \
                 heading(s), so the column cannot be dropped from it without \
                 leaving the row ragged; repair or delete that row first"
            ),
        }
    }
}

impl std::error::Error for EditError {}

/// Rebuild a line from its field values — the leading tag included, since
/// `split_ags_line` returns it as field 0.
///
/// Quoting is [`quote_field`], not a local copy of it. The validator carried a
/// hand-port of the numeric formatters once, kept honest only by a comment
/// saying where it came from, and nothing checked that the two agreed — so it
/// could have judged a value by a different formatter than the one that writes
/// it. A file this crate emits has to be quoted by the function everything
/// else in the repo quotes by.
pub(crate) fn rebuild(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| quote_field(f))
        .collect::<Vec<_>>()
        .join(",")
}

/// What a line becomes. Absence from the plan is the byte-verbatim default,
/// which is why there is no `Keep`: not deciding is the guarantee.
#[derive(Debug, Clone)]
enum Line {
    Replace(String),
    Drop,
}

/// A row an earlier operation appended, held as fields rather than as a
/// rendered line: a later operation in the same patch has to be able to edit
/// it. A patch that adds a row and then drops a column would otherwise leave
/// exactly one ragged row behind — the shape this layer exists to prevent,
/// produced by the layer itself.
#[derive(Debug, Clone)]
struct Pending {
    group: String,
    fields: Vec<String>,
}

/// The last source line the group occupies: its last DATA row, or failing
/// that the last descriptor row it actually has, or its GROUP line.
fn last_line(g: &laterite_ags4_parse::ParsedGroup) -> u32 {
    g.rows
        .last()
        .map(|r| r.line)
        .into_iter()
        .chain(g.type_line)
        .chain(g.unit_line)
        .chain(g.heading_line)
        .chain(std::iter::once(g.group_line))
        .max()
        .unwrap_or(g.group_line)
}

/// Where an operation sits in the canonical order. Operations are applied in
/// this order regardless of the order they were written, which is what makes
/// every combination of them mean one thing.
///
/// Without it the answer depended on the sequence: `delete-group PROJ` then
/// `set PROJ:1:…` resurrected a lone `"DATA"` line under no group at all,
/// because the set overwrote the delete; `delete-column` then `add-row` built
/// the new row against the headings the column had already left. Both are the
/// orphaned/ragged row this layer exists to prevent, produced by the layer.
/// Writes land first and removals last, so a removal always wins over a write
/// to the same place — the reading a patch author expects, since asking to
/// delete a thing and also to edit it can only mean the delete.
fn rank(op: &Op) -> u8 {
    match op {
        Op::SetCell { .. } => 0,
        Op::AddRow { .. } => 1,
        Op::DeleteRow { .. } => 2,
        Op::DeleteColumn { .. } => 3,
        Op::DeleteGroup { .. } => 4,
    }
}

/// Apply `ops` to `text`, returning the new file.
///
/// Operations are resolved against the file as it arrived, so a patch reads
/// the way its author wrote it: row 2 means the second row of the original
/// group, whatever else the patch does to the group. They are then applied in
/// the canonical order [`rank`] describes, so the result does not depend on
/// which order they were listed in.
pub fn apply(text: &str, ops: &[Op]) -> Result<String, EditError> {
    // `validating()` is the profile that retains `raw_lines`, which is the whole
    // basis of the byte-verbatim guarantee: no raw lines, no untouched lines.
    let parsed = parse_bytes_opts(text.as_bytes(), ParseOptions::validating())
        .map_err(|e| EditError::Parse(format!("{e:?}")))?;

    // The REAL terminator per line. `RawLine::had_crlf` answers only "was it
    // CRLF", and `Cr` is a third variant the reader accepts (classic Mac), so
    // reconstructing from that bool silently rewrote a CR-terminated file to LF
    // on a no-op. `Unterminated` is a variant too, which is what makes a file
    // with no final newline come back without one — no truncation afterwards.
    let terminators: Vec<&'static str> = line_spans(text.as_bytes())
        .map(|span| span.term.as_str())
        .collect();

    // A code declared twice makes every locator ambiguous, and the parse leaf
    // resolves the halves inconsistently (rows first-seen-wins, headings
    // last-seen-wins). Refuse before anything is written rather than mix them.
    let mut seen: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    for record in &parsed.group_records {
        seen.entry(record.code.as_str())
            .or_default()
            .push(record.line);
    }
    if let Some((code, lines)) = seen.iter().find(|(_, lines)| lines.len() > 1) {
        return Err(EditError::DuplicateGroup {
            group: (*code).to_string(),
            lines: lines.clone(),
        });
    }

    // Line number -> what happens to it. Absent means Keep.
    let mut plan: BTreeMap<u32, Line> = BTreeMap::new();
    // Line number -> rows appended after it.
    let mut inserts: BTreeMap<u32, Vec<Pending>> = BTreeMap::new();

    let line_text = |n: u32| -> String {
        parsed
            .raw_lines
            .iter()
            .find(|l| l.number == n)
            .map(|l| l.text.clone())
            .unwrap_or_default()
    };
    // A line already edited by an earlier op must be edited FURTHER, not from
    // the source — two set-cells on one row both have to land.
    let current = |plan: &BTreeMap<u32, Line>, n: u32| -> String {
        match plan.get(&n) {
            Some(Line::Replace(s)) => s.clone(),
            _ => line_text(n),
        }
    };

    let mut ordered: Vec<&Op> = ops.iter().collect();
    ordered.sort_by_key(|op| rank(op));

    for op in ordered {
        let code = op.group();
        let g = parsed
            .groups
            .get(code)
            .ok_or_else(|| EditError::NoSuchGroup(code.to_string()))?;
        match op {
            Op::SetCell {
                row,
                heading,
                value,
                group,
            } => {
                let col = g.col(heading).ok_or_else(|| EditError::NoSuchHeading {
                    group: group.clone(),
                    heading: heading.clone(),
                })?;
                let data = g
                    .rows
                    .get(row.wrapping_sub(1))
                    .ok_or(EditError::NoSuchRow {
                        group: group.clone(),
                        row: *row,
                        rows: g.rows.len(),
                    })?;
                let mut fields = split_ags_line(&current(&plan, data.line));
                // +1 for the leading "DATA" tag. A short row is padded rather
                // than refused: the edit the caller asked for is unambiguous,
                // and leaving the row ragged would be the worse answer. Padding
                // reaches the group's FULL arity, not just the target column —
                // stopping at the column is what "leaving it ragged" means.
                let at = col + 1;
                let want = (g.headings.len() + 1).max(at + 1);
                if fields.len() < want {
                    fields.resize(want, String::new());
                }
                fields[at].clone_from(value);
                plan.insert(data.line, Line::Replace(rebuild(&fields)));
            }
            Op::AddRow { cells, group } => {
                let mut values = vec![String::new(); g.headings.len()];
                for (heading, value) in cells {
                    let col = g.col(heading).ok_or_else(|| EditError::NoSuchHeading {
                        group: group.clone(),
                        heading: heading.clone(),
                    })?;
                    values[col].clone_from(value);
                }
                let mut fields = vec!["DATA".to_string()];
                fields.extend(values);
                let pending = Pending {
                    group: group.clone(),
                    fields,
                };
                // After the LAST line the group actually has, so the row lands
                // inside its own group rather than at the top of the next one.
                // The max, not a first-match chain: a chain that consulted
                // TYPE then HEADING put the row between HEADING and UNIT in a
                // group that has no TYPE row.
                let after = last_line(g);
                inserts.entry(after).or_default().push(pending);
            }
            Op::DeleteRow { row, group } => {
                let data = g
                    .rows
                    .get(row.wrapping_sub(1))
                    .ok_or(EditError::NoSuchRow {
                        group: group.clone(),
                        row: *row,
                        rows: g.rows.len(),
                    })?;
                plan.insert(data.line, Line::Drop);
            }
            Op::DeleteGroup { group } => {
                for pendings in inserts.values_mut() {
                    pendings.retain(|p| &p.group != group);
                }
                let last = last_line(g);
                for n in g.group_line..=last {
                    plan.insert(n, Line::Drop);
                }
                // The separator that followed it, if the next line is blank —
                // leaving it behind doubles the gap on every deletion.
                if last < parsed.total_lines && line_text(last + 1).trim().is_empty() {
                    plan.insert(last + 1, Line::Drop);
                }
            }
            Op::DeleteColumn { heading, group } => {
                let col = g.col(heading).ok_or_else(|| EditError::NoSuchHeading {
                    group: group.clone(),
                    heading: heading.clone(),
                })?;
                let at = col + 1;
                let lines = [g.heading_line, g.unit_line, g.type_line]
                    .into_iter()
                    .flatten()
                    .chain(g.rows.iter().map(|r| r.line));
                for n in lines {
                    if matches!(plan.get(&n), Some(Line::Drop)) {
                        continue; // already gone; a removal outranks a rewrite
                    }
                    let mut fields = split_ags_line(&current(&plan, n));
                    if at >= fields.len() {
                        // Dropping the column from this row's siblings and not
                        // from this row is how a ragged row gets made. Refuse
                        // by name rather than produce one silently.
                        return Err(EditError::ShortRow {
                            group: group.clone(),
                            line: n,
                            fields: fields.len().saturating_sub(1),
                            headings: g.headings.len(),
                        });
                    }
                    fields.remove(at);
                    plan.insert(n, Line::Replace(rebuild(&fields)));
                }
                // Rows this patch has already appended are part of the group
                // too, and nothing else will come back for them.
                for pending in inserts.values_mut().flatten() {
                    if &pending.group == group && at < pending.fields.len() {
                        pending.fields.remove(at);
                    }
                }
            }
        }
    }

    let mut out = String::with_capacity(text.len());
    if parsed.has_bom {
        out.push('\u{feff}');
    }
    for line in &parsed.raw_lines {
        // Each line keeps its OWN terminator, which is what lets a file with
        // mixed endings survive an edit to one of them.
        let terminator = terminators
            .get(line.number as usize - 1)
            .copied()
            .unwrap_or("");
        match plan.get(&line.number) {
            Some(Line::Drop) => {}
            Some(Line::Replace(s)) => {
                out.push_str(s);
                out.push_str(terminator);
            }
            None => {
                out.push_str(&line.text);
                out.push_str(terminator);
            }
        }
        // An appended row survives its anchor being deleted — deleting row 1
        // is not a reason to lose the row this patch added. A DELETED GROUP is,
        // and that is handled where the group is dropped.
        for added in inserts.get(&line.number).into_iter().flatten() {
            out.push_str(&rebuild(&added.fields));
            // The anchor's terminator may be `Unterminated` (it was the last
            // line); a row written after it needs a real one.
            out.push_str(if terminator.is_empty() {
                "\r\n"
            } else {
                terminator
            });
        }
    }
    Ok(out)
}

/// A patch file: a list of operations, `.toml` or `.json` by extension —
/// the same two-format rule `forge strategy` already uses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    #[serde(default)]
    pub op: Vec<Op>,
}

impl Patch {
    pub fn load(path: &Path) -> anyhow::Result<Vec<Op>> {
        let text = std::fs::read_to_string(path)?;
        let patch: Patch = if path.extension().is_some_and(|e| e == "json") {
            serde_json::from_str(&text)?
        } else {
            toml::from_str(&text)?
        };
        Ok(patch.op)
    }

    /// The worked example the `--help` points at. A patch file is the form
    /// that survives review and re-running; the flags are for one-liners.
    #[must_use]
    pub fn template() -> String {
        r#"# forge edit --patch — operations apply to the file as it arrived,
# so `row` always counts the ORIGINAL data rows, 1-indexed.

[[op]]
kind = "set"
group = "LOCA"
row = 1
heading = "LOCA_ID"
value = "BH1"

[[op]]
kind = "add-row"
group = "LOCA"
cells = { LOCA_ID = "BH2", LOCA_REM = "a value, with a comma" }

# [[op]]
# kind = "delete-row"
# group = "LOCA"
# row = 2

# [[op]]
# kind = "delete-column"
# group = "LOCA"
# heading = "LOCA_REM"

# [[op]]
# kind = "delete-group"
# group = "LOCA"
"#
        .to_string()
    }
}

/// Parse one `--set GROUP:ROW:HEADING=VALUE` (and its siblings). Split from
/// the right on `=` so a value may contain one; split from the left on `:`
/// so it may contain those too.
pub fn parse_flag(kind: &str, spec: &str) -> anyhow::Result<Op> {
    let bad = |want: &str| anyhow::anyhow!("--{kind} {spec:?} is not `{want}`");
    let row = |s: &str| -> anyhow::Result<usize> {
        s.parse::<usize>()
            .map_err(|_| anyhow::anyhow!("--{kind} {spec:?}: {s:?} is not a row number"))
    };
    match kind {
        "set" | "blank" => {
            let (locator, value) = if kind == "set" {
                spec.split_once('=')
                    .ok_or_else(|| bad("GROUP:ROW:HEADING=VALUE"))?
            } else {
                (spec, "")
            };
            let mut parts = locator.splitn(3, ':');
            let (g, r, h) = (parts.next(), parts.next(), parts.next());
            let (Some(g), Some(r), Some(h)) = (g, r, h) else {
                return Err(bad(if kind == "set" {
                    "GROUP:ROW:HEADING=VALUE"
                } else {
                    "GROUP:ROW:HEADING"
                }));
            };
            Ok(Op::SetCell {
                group: g.to_string(),
                row: row(r)?,
                heading: h.to_string(),
                value: value.to_string(),
            })
        }
        "delete-row" => {
            let (g, r) = spec.split_once(':').ok_or_else(|| bad("GROUP:ROW"))?;
            Ok(Op::DeleteRow {
                group: g.to_string(),
                row: row(r)?,
            })
        }
        "delete-column" => {
            let (g, h) = spec.split_once(':').ok_or_else(|| bad("GROUP:HEADING"))?;
            Ok(Op::DeleteColumn {
                group: g.to_string(),
                heading: h.to_string(),
            })
        }
        "delete-group" => Ok(Op::DeleteGroup {
            group: spec.to_string(),
        }),
        "add-row" => Ok(Op::AddRow {
            group: spec.to_string(),
            cells: BTreeMap::new(),
        }),
        other => Err(anyhow::anyhow!("unknown operation {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CRLF, a quoted comma, an embedded quote, a blank separator line and a
    /// group with no data rows — everything the naive edit-the-text approach
    /// breaks, in one file.
    const FILE: &str = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"Site A, Phase 2\"\r\n",
        "\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n",
        "\"UNIT\",\"\",\"m\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
        "\"DATA\",\"BH1\",\"100.00\",\"the \"\"good\"\" one\"\r\n",
        "\"DATA\",\"BH2\",\"200.00\",\"\"\r\n",
    );

    fn set(group: &str, row: usize, heading: &str, value: &str) -> Op {
        Op::SetCell {
            group: group.into(),
            row,
            heading: heading.into(),
            value: value.into(),
        }
    }

    /// Read one cell back through the parser rather than by string search —
    /// a test that greps the output would pass on a file the parser cannot
    /// read, which is the failure this whole layer exists to prevent.
    fn cell(text: &str, group: &str, row: usize, heading: &str) -> String {
        let p = laterite_ags4_parse::parse_str(text).expect("output must re-parse");
        let g = p.groups.get(group).expect("group");
        g.cell(g.col(heading).expect("heading"), row - 1)
            .expect("cell")
            .to_string()
    }

    #[test]
    fn a_no_op_edit_returns_the_input_unchanged() {
        assert_eq!(apply(FILE, &[]).unwrap(), FILE);
    }

    /// The no-op guarantee has to survive the two shapes a text round-trip
    /// silently normalises: a mixed-terminator file and one that does not end
    /// in a newline.
    #[test]
    fn a_no_op_preserves_mixed_terminators_and_a_missing_final_newline() {
        let mixed = "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\n\
                     \"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"";
        assert_eq!(apply(mixed, &[]).unwrap(), mixed);
    }

    #[test]
    fn a_bom_survives_a_no_op() {
        let with_bom = format!("\u{feff}{FILE}");
        assert_eq!(apply(&with_bom, &[]).unwrap(), with_bom);
    }

    #[test]
    fn setting_one_cell_leaves_every_other_line_byte_identical() {
        let out = apply(FILE, &[set("LOCA", 2, "LOCA_NATE", "222.00")]).unwrap();
        let before: Vec<_> = FILE.lines().collect();
        let after: Vec<_> = out.lines().collect();
        assert_eq!(before.len(), after.len());
        for (i, (b, a)) in before.iter().zip(&after).enumerate() {
            // Index 11 is LOCA's SECOND data row — the one the edit names.
            if i == 11 {
                assert_ne!(b, a, "the edited line must change");
            } else {
                assert_eq!(b, a, "line {} must be byte-identical", i + 1);
            }
        }
        assert_eq!(cell(&out, "LOCA", 2, "LOCA_NATE"), "222.00");
    }

    /// The comma is the whole point: it is the character that turns a value
    /// into two fields under any edit that does not understand quoting.
    #[test]
    fn a_value_containing_a_comma_survives_every_operation() {
        let comma = "north, then east";
        let out = apply(
            FILE,
            &[
                set("LOCA", 1, "LOCA_REM", comma),
                Op::AddRow {
                    group: "LOCA".into(),
                    cells: BTreeMap::from([
                        ("LOCA_ID".into(), "BH3".into()),
                        ("LOCA_REM".into(), comma.into()),
                    ]),
                },
                Op::DeleteColumn {
                    group: "LOCA".into(),
                    heading: "LOCA_NATE".into(),
                },
            ],
        )
        .unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), comma);
        assert_eq!(cell(&out, "LOCA", 3, "LOCA_REM"), comma);
        assert_eq!(cell(&out, "LOCA", 3, "LOCA_ID"), "BH3");
    }

    /// A quote inside a value has to come back out as one quote, not two and
    /// not zero — the other half of the quoting contract.
    #[test]
    fn an_embedded_quote_round_trips_through_an_unrelated_edit() {
        let out = apply(FILE, &[set("LOCA", 1, "LOCA_ID", "BH1a")]).unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), "the \"good\" one");
    }

    #[test]
    fn two_edits_to_one_row_both_land() {
        let out = apply(
            FILE,
            &[
                set("LOCA", 1, "LOCA_ID", "X1"),
                set("LOCA", 1, "LOCA_REM", "y"),
            ],
        )
        .unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "X1");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), "y");
    }

    #[test]
    fn blanking_a_cell_empties_it_without_removing_the_field() {
        let out = apply(FILE, &[set("LOCA", 1, "LOCA_NATE", "")]).unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_NATE"), "");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), "the \"good\" one");
    }

    #[test]
    fn deleting_a_row_removes_only_that_row() {
        let out = apply(
            FILE,
            &[Op::DeleteRow {
                group: "LOCA".into(),
                row: 1,
            }],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["LOCA"].rows.len(), 1);
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "BH2");
        assert_eq!(p.groups["PROJ"].rows.len(), 1);
    }

    /// Rows are resolved against the file as it arrived, so a patch reads the
    /// way its author wrote it — deleting row 1 must not renumber row 2 out
    /// from under the next operation.
    #[test]
    fn row_numbers_address_the_original_file_not_the_edited_one() {
        let out = apply(
            FILE,
            &[
                Op::DeleteRow {
                    group: "LOCA".into(),
                    row: 1,
                },
                set("LOCA", 2, "LOCA_ID", "kept"),
            ],
        )
        .unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "kept");
    }

    #[test]
    fn an_added_row_lands_inside_its_own_group_and_is_never_ragged() {
        let out = apply(
            FILE,
            &[Op::AddRow {
                group: "PROJ".into(),
                cells: BTreeMap::from([("PROJ_ID".into(), "P2".into())]),
            }],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["PROJ"];
        assert_eq!(g.rows.len(), 2);
        assert_eq!(g.rows[1].values.len(), g.headings.len());
        assert_eq!(cell(&out, "PROJ", 2, "PROJ_ID"), "P2");
        assert_eq!(cell(&out, "PROJ", 2, "PROJ_NAME"), "");
        // …and the group it landed in front of is untouched.
        assert_eq!(p.groups["LOCA"].rows.len(), 2);
    }

    #[test]
    fn deleting_a_group_takes_its_descriptor_rows_and_its_separator() {
        let out = apply(
            FILE,
            &[Op::DeleteGroup {
                group: "PROJ".into(),
            }],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert!(!p.groups.contains_key("PROJ"));
        assert_eq!(p.groups["LOCA"].rows.len(), 2);
        assert!(
            !out.starts_with("\r\n"),
            "the separator blank line must go with the group: {out:?}"
        );
    }

    #[test]
    fn deleting_a_column_keeps_every_row_the_same_arity() {
        let out = apply(
            FILE,
            &[Op::DeleteColumn {
                group: "LOCA".into(),
                heading: "LOCA_NATE".into(),
            }],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_REM"]);
        assert_eq!(g.units.len(), 2, "the UNIT row loses its cell too");
        assert!(g.rows.iter().all(|r| r.values.len() == 2));
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), "the \"good\" one");
    }

    #[test]
    fn a_group_that_is_not_there_is_named_in_the_error() {
        let e = apply(FILE, &[set("XXXX", 1, "A", "b")]).unwrap_err();
        assert_eq!(e, EditError::NoSuchGroup("XXXX".into()));
        assert!(e.to_string().contains("XXXX"), "{e}");
    }

    #[test]
    fn a_heading_that_is_not_there_is_named_in_the_error() {
        let e = apply(FILE, &[set("LOCA", 1, "LOCA_NOPE", "b")]).unwrap_err();
        assert!(matches!(e, EditError::NoSuchHeading { .. }));
        assert!(e.to_string().contains("LOCA_NOPE"), "{e}");
    }

    /// Rows are 1-indexed, so row 0 and row n+1 are both out of range — and
    /// the message has to say which convention it is counting in, because
    /// off-by-one is the whole failure mode.
    #[test]
    fn a_row_out_of_range_says_how_many_there_are() {
        for row in [0, 3] {
            let e = apply(FILE, &[set("LOCA", row, "LOCA_ID", "b")]).unwrap_err();
            assert_eq!(
                e,
                EditError::NoSuchRow {
                    group: "LOCA".into(),
                    row,
                    rows: 2
                }
            );
            assert!(e.to_string().contains("1-indexed"), "{e}");
        }
    }

    /// A failed operation must not half-apply the ones before it. Asserting
    /// only `is_err()` would not say that — the name's whole claim is about
    /// what did NOT happen, so the test has to look at the file.
    #[test]
    fn a_failing_operation_leaves_the_file_alone() {
        let ops = [
            set("LOCA", 1, "LOCA_ID", "X1"),
            set("LOCA", 9, "LOCA_ID", "y"),
        ];
        assert!(apply(FILE, &ops).is_err());
        // Nothing is written before the whole patch resolves, so the observable
        // is that the surviving op alone still starts from the original file.
        let after = apply(FILE, &ops[..1]).unwrap();
        assert_eq!(cell(&after, "LOCA", 1, "LOCA_ID"), "X1");
        assert_eq!(cell(&after, "LOCA", 2, "LOCA_ID"), "BH2");
    }

    /// A classic-Mac file. `RawLine` records only "was it CRLF", and a lone
    /// `\r` is a third terminator the reader accepts — so reconstructing the
    /// file from that bool rewrote every line of it on a no-op.
    #[test]
    fn a_cr_terminated_file_survives_a_no_op() {
        let mac = "\"GROUP\",\"PROJ\"\r\"HEADING\",\"PROJ_ID\"\r\"UNIT\",\"\"\r\
                   \"TYPE\",\"ID\"\r\"DATA\",\"P1\"\r";
        assert_eq!(apply(mac, &[]).unwrap(), mac);
    }

    /// One code declared twice makes every locator here mean two things, and
    /// the parse leaf resolves the halves inconsistently — rows first-seen-wins,
    /// headings last-seen-wins. Editing would mix them silently, and the
    /// group-deletion range would swallow every group declared in between.
    #[test]
    fn a_duplicate_group_code_is_refused_by_name() {
        let dup = format!("{FILE}\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n");
        let e = apply(&dup, &[set("LOCA", 1, "LOCA_ID", "x")]).unwrap_err();
        assert!(matches!(e, EditError::DuplicateGroup { .. }), "{e:?}");
        assert!(e.to_string().contains("LOCA"), "{e}");
        // …and refused even when nothing names the duplicated group, because
        // any deletion range would still cross it.
        assert!(apply(&dup, &[]).is_err());
    }

    /// A row too short to carry the column being dropped. Removing it from the
    /// siblings and not from this row is how a ragged row gets made.
    #[test]
    fn a_row_too_short_for_the_column_is_refused_rather_than_left_ragged() {
        let ragged = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n",
            "\"UNIT\",\"\",\"m\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
            "\"DATA\",\"BH1\",\"1.00\",\"ok\"\r\n",
            "\"DATA\",\"BH2\"\r\n",
        );
        let e = apply(
            ragged,
            &[Op::DeleteColumn {
                group: "LOCA".into(),
                heading: "LOCA_REM".into(),
            }],
        )
        .unwrap_err();
        assert!(matches!(e, EditError::ShortRow { line: 6, .. }), "{e:?}");
        assert!(e.to_string().contains("ragged"), "{e}");
    }

    /// A template that does not load is the classic broken worked example —
    /// and the only reader who finds out is the one who copied it.
    #[test]
    fn the_patch_template_parses_as_a_patch() {
        let patch: Patch = toml::from_str(&Patch::template()).expect("template must load");
        assert_eq!(patch.op.len(), 2, "the uncommented ops");
        assert!(matches!(patch.op[0], Op::SetCell { .. }));
        assert!(matches!(patch.op[1], Op::AddRow { .. }));
    }

    /// Every operation the template shows, commented or not, has to name a
    /// `kind` the loader accepts — a commented example is still copied.
    #[test]
    fn every_kind_the_template_shows_is_a_kind_that_loads() {
        let uncommented: String = Patch::template()
            .lines()
            .map(|l| l.trim_start_matches("# ").trim_start_matches('#'))
            .filter(|l| !l.starts_with("forge edit") && !l.starts_with("so `row`"))
            .collect::<Vec<_>>()
            .join("\n");
        let patch: Patch = toml::from_str(&uncommented).expect("every commented op must load");
        assert_eq!(
            patch.op.len(),
            5,
            "set, add-row, delete-row, -column, -group"
        );
    }

    /// The flag grammar and the patch `kind` names are the same vocabulary —
    /// two spellings of one operation would be a documentation trap.
    #[test]
    fn the_flag_names_and_the_patch_kinds_agree() {
        for kind in [
            "set",
            "add-row",
            "delete-row",
            "delete-column",
            "delete-group",
        ] {
            let spec = match kind {
                "set" => "LOCA:1:LOCA_ID=x",
                "delete-row" => "LOCA:1",
                "delete-column" => "LOCA:LOCA_ID",
                _ => "LOCA",
            };
            let op = parse_flag(kind, spec).unwrap();
            let json = serde_json::to_value(&op).unwrap();
            assert_eq!(json["kind"], kind, "flag --{kind} must serialise as {kind}");
        }
    }

    /// A value may contain `=` and `:`; the locator may not. Splitting from
    /// the wrong end is how a remark like `depth: 3=4m` loses its tail.
    #[test]
    fn a_set_value_may_contain_the_delimiters() {
        let op = parse_flag("set", "LOCA:1:LOCA_REM=depth: 3=4m").unwrap();
        assert_eq!(op, set("LOCA", 1, "LOCA_REM", "depth: 3=4m"));
    }

    #[test]
    fn blank_is_set_to_nothing() {
        assert_eq!(
            parse_flag("blank", "LOCA:2:LOCA_REM").unwrap(),
            set("LOCA", 2, "LOCA_REM", "")
        );
    }

    #[test]
    fn a_malformed_flag_says_what_the_shape_should_be() {
        for (kind, spec, want) in [
            ("set", "LOCA:1:LOCA_ID", "GROUP:ROW:HEADING=VALUE"),
            ("set", "LOCA=x", "GROUP:ROW:HEADING=VALUE"),
            ("delete-row", "LOCA", "GROUP:ROW"),
            ("delete-column", "LOCA", "GROUP:HEADING"),
        ] {
            let e = parse_flag(kind, spec).unwrap_err().to_string();
            assert!(e.contains(want), "--{kind} {spec:?} said: {e}");
        }
        let e = parse_flag("delete-row", "LOCA:one")
            .unwrap_err()
            .to_string();
        assert!(e.contains("not a row number"), "{e}");
    }

    /// The defect this layer would otherwise introduce itself: append a row,
    /// then drop a column, and exactly one row keeps the old arity.
    #[test]
    fn a_column_dropped_after_a_row_is_added_takes_the_added_row_too() {
        let out = apply(
            FILE,
            &[
                Op::AddRow {
                    group: "LOCA".into(),
                    cells: BTreeMap::from([("LOCA_ID".into(), "BH3".into())]),
                },
                Op::DeleteColumn {
                    group: "LOCA".into(),
                    heading: "LOCA_NATE".into(),
                },
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings.len(), 2);
        assert!(
            g.rows.iter().all(|r| r.values.len() == 2),
            "no row may be left ragged: {:?}",
            g.rows
        );
    }

    /// …and a group deleted after a row was added to it must not leave the
    /// row behind, orphaned under whatever group follows.
    #[test]
    fn a_group_deleted_after_a_row_is_added_takes_the_added_row_too() {
        let out = apply(
            FILE,
            &[
                Op::AddRow {
                    group: "PROJ".into(),
                    cells: BTreeMap::from([("PROJ_ID".into(), "P2".into())]),
                },
                Op::DeleteGroup {
                    group: "PROJ".into(),
                },
            ],
        )
        .unwrap();
        assert!(!out.contains("P2"), "{out}");
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["LOCA"].rows.len(), 2);
    }

    /// Operations apply in a canonical order, not the order they were written,
    /// so a patch cannot mean two things. Deleting a group and also editing a
    /// row in it can only mean the delete — before, the set overwrote the drop
    /// and resurrected a lone `"DATA"` line under no group at all.
    #[test]
    fn deleting_a_group_beats_an_edit_to_a_row_in_it_whichever_order_they_are_in() {
        let del = Op::DeleteGroup {
            group: "PROJ".into(),
        };
        let edit = set("PROJ", 1, "PROJ_NAME", "renamed");
        for ops in [
            vec![del.clone(), edit.clone()],
            vec![edit.clone(), del.clone()],
        ] {
            let out = apply(FILE, &ops).unwrap();
            assert!(!out.contains("renamed"), "{out}");
            let p = laterite_ags4_parse::parse_str(&out).unwrap();
            assert!(!p.groups.contains_key("PROJ"));
            assert_eq!(p.groups["LOCA"].rows.len(), 2, "LOCA is untouched");
        }
    }

    /// The same for a deleted ROW: a set on it cannot survive the delete.
    #[test]
    fn deleting_a_row_beats_an_edit_to_it_whichever_order_they_are_in() {
        let del = Op::DeleteRow {
            group: "LOCA".into(),
            row: 1,
        };
        let edit = set("LOCA", 1, "LOCA_ID", "ghost");
        for ops in [vec![del.clone(), edit.clone()], vec![edit, del]] {
            let out = apply(FILE, &ops).unwrap();
            assert!(!out.contains("ghost"), "{out}");
            let p = laterite_ags4_parse::parse_str(&out).unwrap();
            assert_eq!(p.groups["LOCA"].rows.len(), 1);
        }
    }

    /// A column dropped and a row added in one patch: the row must come out at
    /// the surviving arity whichever order the two were written in. Only one
    /// order used to work, because the fixup ran inside the `DeleteColumn` arm.
    #[test]
    fn a_column_drop_and_a_row_add_agree_whichever_order_they_are_in() {
        let add = Op::AddRow {
            group: "LOCA".into(),
            cells: BTreeMap::from([("LOCA_ID".into(), "BH3".into())]),
        };
        let drop = Op::DeleteColumn {
            group: "LOCA".into(),
            heading: "LOCA_NATE".into(),
        };
        for ops in [
            vec![add.clone(), drop.clone()],
            vec![drop.clone(), add.clone()],
        ] {
            let out = apply(FILE, &ops).unwrap();
            let p = laterite_ags4_parse::parse_str(&out).unwrap();
            let g = &p.groups["LOCA"];
            assert_eq!(g.headings.len(), 2);
            assert_eq!(g.rows.len(), 3);
            assert!(
                g.rows.iter().all(|r| r.values.len() == 2),
                "no row may be left ragged: {:?}",
                g.rows
            );
        }
    }

    /// A group with HEADING and UNIT but no TYPE row and no data. The anchor
    /// used to be a first-match chain that consulted TYPE then HEADING, so the
    /// appended row landed BETWEEN the HEADING and UNIT rows.
    #[test]
    fn a_row_added_to_a_descriptor_only_group_lands_after_its_last_line() {
        let sparse = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\"\r\n",
            "\"UNIT\",\"\"\r\n",
        );
        let out = apply(
            sparse,
            &[Op::AddRow {
                group: "PROJ".into(),
                cells: BTreeMap::from([("PROJ_ID".into(), "P1".into())]),
            }],
        )
        .unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines[2].starts_with("\"UNIT\""),
            "UNIT must stay third: {out}"
        );
        assert!(lines[3].starts_with("\"DATA\""), "the row goes last: {out}");
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["PROJ"].rows.len(), 1);
    }

    /// A short row named by a SET is padded to the group's FULL arity, not
    /// just to the column being written — stopping at the column is what
    /// "leaving the row ragged" means.
    #[test]
    fn setting_a_cell_on_a_short_row_pads_it_to_the_full_arity() {
        let ragged = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n",
            "\"UNIT\",\"\",\"m\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
            "\"DATA\",\"BH1\"\r\n",
        );
        let out = apply(ragged, &[set("LOCA", 1, "LOCA_ID", "BH9")]).unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["LOCA"].rows[0].values.len(), 3, "{out}");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "BH9");
    }

    /// The comma has to survive the operations the earlier test does NOT
    /// exercise: blanking a neighbouring cell, deleting another row, and
    /// deleting another group.
    #[test]
    fn a_comma_survives_the_remaining_operations_too() {
        let comma = "north, then east";
        let out = apply(
            FILE,
            &[
                set("LOCA", 2, "LOCA_REM", comma),
                set("LOCA", 2, "LOCA_NATE", ""),
                Op::DeleteRow {
                    group: "LOCA".into(),
                    row: 1,
                },
                Op::DeleteGroup {
                    group: "PROJ".into(),
                },
            ],
        )
        .unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), comma);
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_NATE"), "");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "BH2");
    }
}
