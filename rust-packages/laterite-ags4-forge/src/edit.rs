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
    /// Add a heading the group does not have. It lands in every descriptor
    /// row the group actually carries and as an EMPTY cell in every data row,
    /// so the arity stays consistent and no rule trips by accident. The cells
    /// start empty on purpose: the caller decides what goes in them rather
    /// than inheriting a guess.
    AddColumn {
        group: String,
        heading: String,
    },
    /// Rewrite the UNIT a heading declares. An empty value leaves the UNIT
    /// present but UNDEFINED, which is a file shape nothing else in this
    /// crate can produce and the one the undefined-units rule needs.
    SetUnit {
        group: String,
        heading: String,
        unit: String,
    },
    /// Declare a heading's TYPE, and by default re-format the column's values
    /// to satisfy it, so the projected file is still spec-valid and any fault
    /// injected afterwards is the only one in it. With `reformat` off the
    /// declaration moves and the values do not — which is how a type-invalid
    /// cell gets made on purpose.
    SetType {
        group: String,
        heading: String,
        #[serde(rename = "type")]
        ags_type: String,
        #[serde(default = "reformat_by_default")]
        reformat: bool,
    },
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
    /// Insert a DATA row at a position rather than appending one, so a fault
    /// can be planted mid-group. `at` counts the file's ORIGINAL data rows,
    /// 1-indexed, like every other row locator here: the new row becomes that
    /// position, and the rows from there on move down.
    InsertRow {
        group: String,
        at: usize,
        /// Keyed by heading, for the same reason `AddRow` is: a positional
        /// row is how a hand-built row ends up ragged.
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

/// Re-formatting is the default because it is the safe half: a patch that
/// declares a type without saying what to do about the values means "make the
/// column that type", not "leave a contradiction behind".
fn reformat_by_default() -> bool {
    true
}

impl Op {
    fn group(&self) -> &str {
        match self {
            Op::AddColumn { group, .. }
            | Op::SetUnit { group, .. }
            | Op::SetType { group, .. }
            | Op::SetCell { group, .. }
            | Op::AddRow { group, .. }
            | Op::InsertRow { group, .. }
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
    /// The group has no UNIT (or, later, TYPE) row at all — a shape forge
    /// itself can manufacture. Writing one would be a DIFFERENT operation:
    /// inventing a descriptor row the file never had. Doing that silently is
    /// how a patch stops meaning what it says.
    MissingDescriptor {
        group: String,
        row: &'static str,
    },
    /// The group already has this heading. Adding a second one would make
    /// every locator naming it mean two columns, which is the ambiguity this
    /// layer refuses everywhere else (see `DuplicateGroup`).
    DuplicateHeading {
        group: String,
        heading: String,
    },
    /// A TYPE token the AGS type system cannot read at all. Dictionary
    /// membership is deliberately NOT the test — declaring a type the
    /// dictionary never pairs with a heading is a fault forge exists to
    /// manufacture — but a token nothing can read produces a file whose
    /// invalidity the caller did not choose.
    UnknownType {
        group: String,
        heading: String,
        ags_type: String,
    },
    /// A value that cannot be rendered to satisfy the type being declared.
    /// Refused rather than mangled, and refused before the file is written at
    /// all: half a projected column is worse than none of one.
    Unprojectable {
        group: String,
        heading: String,
        row: usize,
        value: String,
        ags_type: String,
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
            EditError::MissingDescriptor { group, row } => write!(
                f,
                "{group} has no {row} row, so there is nothing to edit; add \
                 one before declaring anything on it"
            ),
            EditError::DuplicateHeading { group, heading } => write!(
                f,
                "{group} already has a heading {heading:?}; adding a second \
                 would make every locator naming it mean two columns"
            ),
            EditError::UnknownType {
                group,
                heading,
                ags_type,
            } => write!(
                f,
                "{ags_type:?} is not a type the AGS type system can read, so \
                 {group}/{heading} cannot be declared as one"
            ),
            EditError::Unprojectable {
                group,
                heading,
                row,
                value,
                ags_type,
            } => write!(
                f,
                "{group}/{heading} row {row}: {value:?} cannot be written as \
                 {ags_type}; blank the cell, correct it, or declare the type \
                 without re-formatting"
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

/// A group's headings as this patch has left them so far.
///
/// Rows are deliberately strict — every operation resolves a row number
/// against the file as it arrived, so a patch reads the way its author wrote
/// it. Headings cannot be that strict for much longer: a patch will be able
/// to CREATE a column, and an operation later in the same patch has to be
/// able to name it. Resolving straight off the parse would refuse that
/// heading as unknown, because the parse happened before the column existed.
///
/// Nothing creates a column yet, so `added` is always empty and this returns
/// exactly what the parse returned — the bytes this module writes are
/// unchanged. It exists so the operation that does create one registers it in
/// ONE place, instead of every arm learning to look in two (#723).
#[derive(Debug, Default)]
struct Shape {
    /// Group code -> headings this patch appended, in creation order. They
    /// sit after the parsed headings because that is where a rebuilt row puts
    /// their cells.
    added: BTreeMap<String, Vec<String>>,
}

impl Shape {
    /// Column index of `heading`, counting columns this patch created.
    /// `None` is the caller's `NoSuchHeading`.
    fn col(&self, g: &laterite_ags4_parse::ParsedGroup, heading: &str) -> Option<usize> {
        g.col(heading).or_else(|| {
            let added = self.added.get(&g.code)?;
            added
                .iter()
                .position(|h| h == heading)
                .map(|i| g.headings.len() + i)
        })
    }

    /// How many cells a row of this group should carry: the width a short row
    /// is padded to, and the width a ragged one is measured against.
    fn arity(&self, g: &laterite_ags4_parse::ParsedGroup) -> usize {
        g.headings.len() + self.added.get(&g.code).map_or(0, Vec::len)
    }
}

/// The last line the group's DESCRIPTOR rows occupy — where a row inserted at
/// position 1 has to land. Not `last_line`, which counts the data rows too, and
/// not a first-match chain over HEADING/UNIT/TYPE: a group missing its TYPE row
/// would then anchor above its UNIT row.
fn descriptor_end(g: &laterite_ags4_parse::ParsedGroup) -> u32 {
    [g.heading_line, g.unit_line, g.type_line]
        .into_iter()
        .flatten()
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
        Op::AddColumn { .. } => 0,
        Op::SetUnit { .. } => 1,
        Op::SetType { .. } => 2,
        Op::SetCell { .. } => 3,
        Op::AddRow { .. } => 4,
        Op::InsertRow { .. } => 5,
        Op::DeleteRow { .. } => 6,
        Op::DeleteColumn { .. } => 7,
        Op::DeleteGroup { .. } => 8,
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
    // `validating()` for its lossy decode — this walk must never hard-fail.
    // (Every profile retains `raw_lines` since the span rewrite; the
    // byte-verbatim guarantee no longer hangs on the profile choice.)
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
    // Group -> the columns to drop from it, resolved against the ORIGINAL
    // headings and applied right-to-left after every other operation.
    let mut columns: BTreeMap<String, std::collections::BTreeSet<usize>> = BTreeMap::new();
    // Line number -> rows appended after it.
    let mut inserts: BTreeMap<u32, Vec<Pending>> = BTreeMap::new();

    let line_text = |n: u32| -> String {
        parsed
            .raw_lines
            .iter()
            .find(|l| l.number == n)
            .map(|l| parsed.line_text(l).to_string())
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

    // Every heading lookup and every arity calculation below goes through
    // this, never straight to the parse — see `Shape`.
    let mut shape = Shape::default();

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
                let col = shape
                    .col(g, heading)
                    .ok_or_else(|| EditError::NoSuchHeading {
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
                // `col` is an index INTO `headings`, so the group's arity is
                // always the larger bound — there is no separate "at least the
                // target column" case to defend against. `resize` truncates
                // when it shrinks, so an over-long row is left alone.
                let want = shape.arity(g) + 1;
                if fields.len() < want {
                    fields.resize(want, String::new());
                }
                fields[at].clone_from(value);
                plan.insert(data.line, Line::Replace(rebuild(&fields)));
            }
            Op::AddColumn { group, heading } => {
                if shape.col(g, heading).is_some() {
                    return Err(EditError::DuplicateHeading {
                        group: group.clone(),
                        heading: heading.clone(),
                    });
                }
                // A group with no HEADING row has nowhere to put a column
                // name. Refuse rather than invent the row — the same answer
                // the descriptor edits give.
                let hl = g.heading_line.ok_or_else(|| EditError::MissingDescriptor {
                    group: group.clone(),
                    row: "HEADING",
                })?;
                // Registered BEFORE the lines are written, so `arity` already
                // counts the new column and the padding below reaches it.
                // This is also what lets a later operation in the same patch
                // name the heading (#723).
                shape
                    .added
                    .entry(group.clone())
                    .or_default()
                    .push(heading.clone());
                let col = shape
                    .col(g, heading)
                    .expect("the column was just registered");
                let want = shape.arity(g) + 1;
                let at = col + 1;

                // Every line the group owns, descriptor rows included, so the
                // arity stays consistent. Pad, then WRITE the cell — a row
                // that was already over-long has a value sitting where the new
                // column goes, and leaving it there would make a column that
                // was asked to start empty start with somebody else's data.
                let lines: Vec<u32> = [g.heading_line, g.unit_line, g.type_line]
                    .into_iter()
                    .flatten()
                    .chain(g.rows.iter().map(|r| r.line))
                    .collect();
                for n in lines {
                    if matches!(plan.get(&n), Some(Line::Drop)) {
                        continue; // already gone; a removal outranks a write
                    }
                    let mut fields = split_ags_line(&current(&plan, n));
                    if fields.len() < want {
                        fields.resize(want, String::new());
                    }
                    fields[at] = if n == hl {
                        heading.clone()
                    } else {
                        String::new()
                    };
                    plan.insert(n, Line::Replace(rebuild(&fields)));
                }
                // Rows this patch appends are built to `shape.arity`, which
                // now includes this column, so they need nothing here — and
                // `AddColumn` ranks ahead of `AddRow` precisely so that holds.
            }
            Op::SetUnit {
                group,
                heading,
                unit,
            } => {
                let col = shape
                    .col(g, heading)
                    .ok_or_else(|| EditError::NoSuchHeading {
                        group: group.clone(),
                        heading: heading.clone(),
                    })?;
                let line = g.unit_line.ok_or_else(|| EditError::MissingDescriptor {
                    group: group.clone(),
                    row: "UNIT",
                })?;
                let mut fields = split_ags_line(&current(&plan, line));
                // The parser does NOT pad the UNIT row — it carries the raw
                // fields the file had, which may be fewer than the headings.
                // Padding to the group's arity is what stops a write to a late
                // column running off the end, and it is the answer `SetCell`
                // already gives for a short DATA row.
                let want = shape.arity(g) + 1;
                if fields.len() < want {
                    fields.resize(want, String::new());
                }
                fields[col + 1].clone_from(unit);
                plan.insert(line, Line::Replace(rebuild(&fields)));
            }
            Op::SetType {
                group,
                heading,
                ags_type,
                reformat,
            } => {
                // Before anything is planned: a token nothing can read would
                // otherwise rewrite the declaration and then fail on the first
                // value, which is the half-written column this refuses.
                if !crate::project::is_known_type(ags_type) {
                    return Err(EditError::UnknownType {
                        group: group.clone(),
                        heading: heading.clone(),
                        ags_type: ags_type.clone(),
                    });
                }
                let col = shape
                    .col(g, heading)
                    .ok_or_else(|| EditError::NoSuchHeading {
                        group: group.clone(),
                        heading: heading.clone(),
                    })?;
                let line = g.type_line.ok_or_else(|| EditError::MissingDescriptor {
                    group: group.clone(),
                    row: "TYPE",
                })?;
                let want = shape.arity(g) + 1;
                let mut fields = split_ags_line(&current(&plan, line));
                if fields.len() < want {
                    fields.resize(want, String::new());
                }
                fields[col + 1].clone_from(ags_type);
                plan.insert(line, Line::Replace(rebuild(&fields)));

                if *reformat {
                    // The unit as THIS PATCH has left it, not as the file
                    // arrived. A `--set-unit` on the same heading is what an
                    // author means to project a DT column against, and
                    // descriptor edits rank ahead of this one precisely so it
                    // has already landed by now.
                    let unit = g
                        .unit_line
                        .map(|ul| {
                            split_ags_line(&current(&plan, ul))
                                .get(col + 1)
                                .cloned()
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    for (i, data) in g.rows.iter().enumerate() {
                        let mut fields = split_ags_line(&current(&plan, data.line));
                        if fields.len() < want {
                            fields.resize(want, String::new());
                        }
                        let before = fields[col + 1].clone();
                        let after =
                            crate::project::project(&before, ags_type, &unit).ok_or_else(|| {
                                EditError::Unprojectable {
                                    group: group.clone(),
                                    heading: heading.clone(),
                                    row: i + 1,
                                    value: before.clone(),
                                    ags_type: ags_type.clone(),
                                }
                            })?;
                        fields[col + 1] = after;
                        plan.insert(data.line, Line::Replace(rebuild(&fields)));
                    }
                }
            }
            Op::AddRow { cells, group } => {
                let mut values = vec![String::new(); shape.arity(g)];
                for (heading, value) in cells {
                    let col = shape
                        .col(g, heading)
                        .ok_or_else(|| EditError::NoSuchHeading {
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
            Op::InsertRow { group, at, cells } => {
                // Refused rather than silently appended: appending is what
                // `AddRow` is for, and a typo that quietly becomes an append
                // gives a reproducer that does not reproduce.
                if *at == 0 || *at > g.rows.len() {
                    return Err(EditError::NoSuchRow {
                        group: group.clone(),
                        row: *at,
                        rows: g.rows.len(),
                    });
                }
                // Anchored AFTER the line before the target row, so the new
                // row takes that position and the rest move down. Position 1
                // has no earlier row, so it anchors to the descriptors.
                let anchor = if *at == 1 {
                    descriptor_end(g)
                } else {
                    g.rows[at - 2].line
                };
                let mut values = vec![String::new(); shape.arity(g)];
                for (heading, value) in cells {
                    let col = shape
                        .col(g, heading)
                        .ok_or_else(|| EditError::NoSuchHeading {
                            group: group.clone(),
                            heading: heading.clone(),
                        })?;
                    values[col].clone_from(value);
                }
                let mut fields = vec!["DATA".to_string()];
                fields.extend(values);
                // Pushed, not replaced: two insertions at one position keep
                // the order they were listed in, which `sort_by_key` preserves
                // because it is stable.
                inserts.entry(anchor).or_default().push(Pending {
                    group: group.clone(),
                    fields,
                });
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
                // Resolved here against the ORIGINAL headings, applied below.
                // Two columns dropped from one group used to shift each
                // other: the second removal's index was computed against the
                // headings the first had already shortened, so it took the
                // wrong field or ran off the end and reported the row as too
                // short. Collecting them and removing right-to-left is what
                // makes the pair mean the same thing in either order.
                let col = shape
                    .col(g, heading)
                    .ok_or_else(|| EditError::NoSuchHeading {
                        group: group.clone(),
                        heading: heading.clone(),
                    })?;
                columns.entry(group.clone()).or_default().insert(col);
            }
        }
    }

    // Right-to-left, so an earlier removal cannot move a later one's index.
    for (code, cols) in &columns {
        let g = &parsed.groups[code];
        let lines: Vec<u32> = [g.heading_line, g.unit_line, g.type_line]
            .into_iter()
            .flatten()
            .chain(g.rows.iter().map(|r| r.line))
            .collect();
        for col in cols.iter().rev() {
            let at = col + 1;
            for n in &lines {
                if matches!(plan.get(n), Some(Line::Drop)) {
                    continue; // already gone; a removal outranks a rewrite
                }
                let mut fields = split_ags_line(&current(&plan, *n));
                if at >= fields.len() {
                    // Dropping the column from this row's siblings and not
                    // from this row is how a ragged row gets made. Refuse by
                    // name rather than produce one silently.
                    return Err(EditError::ShortRow {
                        group: code.clone(),
                        line: *n,
                        fields: fields.len().saturating_sub(1),
                        headings: shape.arity(g),
                    });
                }
                fields.remove(at);
                plan.insert(*n, Line::Replace(rebuild(&fields)));
            }
            // Rows this patch has already appended are part of the group too,
            // and nothing else will come back for them.
            for pending in inserts.values_mut().flatten() {
                if &pending.group == code && at < pending.fields.len() {
                    pending.fields.remove(at);
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
                out.push_str(parsed.line_text(line));
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
# so `row` always counts the ORIGINAL data rows, 1-indexed. They also apply in
# a canonical order rather than the order written here, so a patch means one
# thing however it is arranged: columns are created, then declarations are
# written, then cells, then rows, and removals last.
#
# The worked example PROJECTS a column into a type the scaffolds never emit:
# create it, declare its UNIT and TYPE, put a value in it. Add the type code to
# the file's TYPE group as well (an add-row below) and both engines return no
# findings — without that, an undeclared type code is a Rule 17.

[[op]]
kind = "add-column"
group = "LOCA"
heading = "LOCA_GL"

[[op]]
kind = "set-unit"
group = "LOCA"
heading = "LOCA_GL"
unit = "m"

[[op]]
kind = "set-type"
group = "LOCA"
heading = "LOCA_GL"
type = "3SF"
# reformat = false     # declare the type and leave every value alone

[[op]]
kind = "set"
group = "LOCA"
row = 1
heading = "LOCA_GL"
value = "21.5"

# [[op]]
# kind = "add-row"
# group = "TYPE"
# cells = { TYPE_TYPE = "3SF", TYPE_DESC = "3 significant figures, rounded" }

# [[op]]
# kind = "insert-row"
# group = "LOCA"
# at = 1
# cells = { LOCA_ID = "BH0" }

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

/// The enum, the spelling table and `as_str` are all emitted from the one
/// list in the invocation below, so a spelling cannot exist without its
/// variant or a variant without its spelling — and `parse_flag`'s match and
/// `cmd.rs`'s spelling→clap-field match are both exhaustive over the enum,
/// so a new row refuses to COMPILE until each has its arm. Adding an edit op
/// used to be a three-file ritual coupled by a bare string, where a typo in
/// `cmd.rs`'s tuple table fell through `parse_flag`'s catch-all at runtime;
/// now the places a new operation must touch fail the build, not a review.
macro_rules! op_kinds {
    ($(($spelling:literal, $variant:ident)),+ $(,)?) => {
        /// One command-line spelling of one edit operation. `Blank` and
        /// `SetTypeRaw` are distinct HERE even though each is a second
        /// spelling of another variant's patch `kind` — one kind in a patch
        /// file, two intentions at the command line (see `parse_flag`).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OpKind { $($variant),+ }

        impl OpKind {
            /// Every spelling, in the fixed order the flags apply (`cmd.rs`
            /// iterates this table, so this order IS the application order).
            pub const SPELLINGS: &'static [(&'static str, OpKind)] =
                &[$(($spelling, OpKind::$variant)),+];

            pub const fn as_str(self) -> &'static str {
                match self { $(OpKind::$variant => $spelling),+ }
            }
        }
    };
}

op_kinds![
    ("set", Set),
    ("blank", Blank),
    ("add-column", AddColumn),
    ("set-unit", SetUnit),
    ("set-type", SetType),
    ("set-type-raw", SetTypeRaw),
    ("add-row", AddRow),
    ("insert-row", InsertRow),
    ("delete-row", DeleteRow),
    ("delete-column", DeleteColumn),
    ("delete-group", DeleteGroup),
];

/// `--{kind}` in the flag error messages — the spelling, verbatim.
impl std::fmt::Display for OpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The string door, for callers that arrive with a spelling rather than a
/// flag. Patch files never come through here (serde dispatches on the `kind`
/// tag itself); this owns the "unknown operation" vocabulary that used to be
/// `parse_flag`'s catch-all — now the only place an unknown spelling can
/// still be said, since the flag path starts from the table.
impl std::str::FromStr for OpKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::SPELLINGS
            .iter()
            .find(|&&(spelling, _)| spelling == s)
            .map(|&(_, kind)| kind)
            .ok_or_else(|| anyhow::anyhow!("unknown operation {s:?}"))
    }
}

/// Parse one `--set GROUP:ROW:HEADING=VALUE` (and its siblings). Split from
/// the right on `=` so a value may contain one; split from the left on `:`
/// so it may contain those too.
pub fn parse_flag(kind: OpKind, spec: &str) -> anyhow::Result<Op> {
    let bad = |want: &str| anyhow::anyhow!("--{kind} {spec:?} is not `{want}`");
    let row = |s: &str| -> anyhow::Result<usize> {
        s.parse::<usize>()
            .map_err(|_| anyhow::anyhow!("--{kind} {spec:?}: {s:?} is not a row number"))
    };
    match kind {
        OpKind::Set | OpKind::Blank => {
            let (locator, value) = if matches!(kind, OpKind::Set) {
                spec.split_once('=')
                    .ok_or_else(|| bad("GROUP:ROW:HEADING=VALUE"))?
            } else {
                (spec, "")
            };
            let mut parts = locator.splitn(3, ':');
            let (g, r, h) = (parts.next(), parts.next(), parts.next());
            let (Some(g), Some(r), Some(h)) = (g, r, h) else {
                return Err(bad(if matches!(kind, OpKind::Set) {
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
        OpKind::AddColumn => {
            let (g, h) = spec.split_once(':').ok_or_else(|| bad("GROUP:HEADING"))?;
            Ok(Op::AddColumn {
                group: g.to_string(),
                heading: h.to_string(),
            })
        }
        OpKind::SetUnit => {
            // The locator is split, never the whole spec: AGS unit strings
            // routinely carry colons (`yyyy-mm-ddThh:mm`), and a four-part
            // colon form would be ambiguous for exactly the type this
            // machinery exists to reach. An empty value is legal and load
            // bearing — it is what leaves the UNIT undefined.
            let (locator, unit) = spec
                .split_once('=')
                .ok_or_else(|| bad("GROUP:HEADING=UNIT"))?;
            let (g, h) = locator
                .split_once(':')
                .ok_or_else(|| bad("GROUP:HEADING=UNIT"))?;
            Ok(Op::SetUnit {
                group: g.to_string(),
                heading: h.to_string(),
                unit: unit.to_string(),
            })
        }
        OpKind::SetType | OpKind::SetTypeRaw => {
            let (locator, ags_type) = spec
                .split_once('=')
                .ok_or_else(|| bad("GROUP:HEADING=TYPE"))?;
            let (grp, h) = locator
                .split_once(':')
                .ok_or_else(|| bad("GROUP:HEADING=TYPE"))?;
            Ok(Op::SetType {
                group: grp.to_string(),
                heading: h.to_string(),
                ags_type: ags_type.to_string(),
                // `--set-type-raw` is a SPELLING of this operation, the way
                // `--blank` is a spelling of `set`: one kind in a patch file,
                // two intentions at the command line.
                reformat: matches!(kind, OpKind::SetType),
            })
        }
        OpKind::InsertRow => {
            let (g, at) = spec.split_once(':').ok_or_else(|| bad("GROUP:POSITION"))?;
            Ok(Op::InsertRow {
                group: g.to_string(),
                at: row(at)?,
                cells: BTreeMap::new(),
            })
        }
        OpKind::DeleteRow => {
            let (g, r) = spec.split_once(':').ok_or_else(|| bad("GROUP:ROW"))?;
            Ok(Op::DeleteRow {
                group: g.to_string(),
                row: row(r)?,
            })
        }
        OpKind::DeleteColumn => {
            let (g, h) = spec.split_once(':').ok_or_else(|| bad("GROUP:HEADING"))?;
            Ok(Op::DeleteColumn {
                group: g.to_string(),
                heading: h.to_string(),
            })
        }
        OpKind::DeleteGroup => Ok(Op::DeleteGroup {
            group: spec.to_string(),
        }),
        OpKind::AddRow => Ok(Op::AddRow {
            group: spec.to_string(),
            cells: BTreeMap::new(),
        }),
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
        assert_eq!(g.rows[1].n_values(), g.headings.len());
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
        assert!(g.rows.iter().all(|r| r.n_values() == 2));
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
        assert_eq!(patch.op.len(), 4, "the uncommented ops");
        assert!(matches!(patch.op[0], Op::AddColumn { .. }));
        assert!(matches!(patch.op[1], Op::SetUnit { .. }));
        assert!(matches!(patch.op[2], Op::SetType { .. }));
        assert!(matches!(patch.op[3], Op::SetCell { .. }));
    }

    /// Every operation the template shows, commented or not, has to name a
    /// `kind` the loader accepts — a commented example is still copied.
    #[test]
    fn every_kind_the_template_shows_is_a_kind_that_loads() {
        // Everything before the first `[[op]]` is prose. Dropping it by
        // POSITION rather than by matching its opening words removes a hidden
        // coupling: the old filter named two specific sentences, so rewording
        // the header would have fed English to the TOML parser.
        let uncommented: String = Patch::template()
            .lines()
            .skip_while(|l| !l.contains("[[op]]"))
            .map(|l| l.trim_start_matches("# ").trim_start_matches('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let patch: Patch = toml::from_str(&uncommented).expect("every commented op must load");
        assert_eq!(
            patch.op.len(),
            9,
            "add-column, set-unit, set-type, set, add-row, insert-row, delete-row, \
             -column, -group"
        );
    }

    /// The flag grammar and the patch `kind` names are the same vocabulary —
    /// two spellings of one operation would be a documentation trap. The
    /// sweep runs over the whole spelling table, so it also covers the two
    /// deliberate second spellings the old hand-list had to footnote away.
    #[test]
    fn the_flag_names_and_the_patch_kinds_agree() {
        for &(spelling, kind) in OpKind::SPELLINGS {
            let spec = match kind {
                OpKind::Set => "LOCA:1:LOCA_ID=x",
                OpKind::Blank => "LOCA:1:LOCA_ID",
                OpKind::AddColumn => "LOCA:LOCA_NEW",
                OpKind::SetUnit => "LOCA:LOCA_ID=m",
                OpKind::SetType | OpKind::SetTypeRaw => "LOCA:LOCA_ID=X",
                OpKind::InsertRow | OpKind::DeleteRow => "LOCA:1",
                OpKind::DeleteColumn => "LOCA:LOCA_ID",
                OpKind::AddRow | OpKind::DeleteGroup => "LOCA",
            };
            let op = parse_flag(kind, spec).unwrap();
            let json = serde_json::to_value(&op).unwrap();
            // `--blank` / `--set-type-raw` are second spellings of `set` /
            // `set-type`: one kind in a patch file, two flag intentions.
            let patch_kind = match kind {
                OpKind::Blank => "set",
                OpKind::SetTypeRaw => "set-type",
                _ => spelling,
            };
            assert_eq!(
                json["kind"], patch_kind,
                "flag --{kind} must serialise as {patch_kind}"
            );
        }
    }

    /// The table and `as_str` come from one macro list, so a mismatch cannot
    /// be written; what this pins is the `FromStr` wiring over that table
    /// (a swapped tuple index would still compile) and the one message left
    /// for an unknown spelling now that the flag path starts from the table.
    #[test]
    fn every_spelling_round_trips_and_an_unknown_one_is_refused() {
        for &(spelling, kind) in OpKind::SPELLINGS {
            assert_eq!(spelling.parse::<OpKind>().unwrap(), kind);
            assert_eq!(kind.as_str(), spelling);
        }
        let e = "frobnicate".parse::<OpKind>().unwrap_err().to_string();
        assert_eq!(e, "unknown operation \"frobnicate\"");
    }

    /// A value may contain `=` and `:`; the locator may not. Splitting from
    /// the wrong end is how a remark like `depth: 3=4m` loses its tail.
    #[test]
    fn a_set_value_may_contain_the_delimiters() {
        let op = parse_flag(OpKind::Set, "LOCA:1:LOCA_REM=depth: 3=4m").unwrap();
        assert_eq!(op, set("LOCA", 1, "LOCA_REM", "depth: 3=4m"));
    }

    #[test]
    fn blank_is_set_to_nothing() {
        assert_eq!(
            parse_flag(OpKind::Blank, "LOCA:2:LOCA_REM").unwrap(),
            set("LOCA", 2, "LOCA_REM", "")
        );
    }

    #[test]
    fn a_malformed_flag_says_what_the_shape_should_be() {
        for (kind, spec, want) in [
            (OpKind::Set, "LOCA:1:LOCA_ID", "GROUP:ROW:HEADING=VALUE"),
            (OpKind::Set, "LOCA=x", "GROUP:ROW:HEADING=VALUE"),
            (OpKind::DeleteRow, "LOCA", "GROUP:ROW"),
            (OpKind::DeleteColumn, "LOCA", "GROUP:HEADING"),
        ] {
            let e = parse_flag(kind, spec).unwrap_err().to_string();
            assert!(e.contains(want), "--{kind} {spec:?} said: {e}");
        }
        let e = parse_flag(OpKind::DeleteRow, "LOCA:one")
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
            g.rows.iter().all(|r| r.n_values() == 2),
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
                g.rows.iter().all(|r| r.n_values() == 2),
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
        assert_eq!(p.groups["LOCA"].rows[0].n_values(), 3, "{out}");
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

    /// Deleting the LAST group in a file: there is no separator line after it,
    /// so the lookahead must not read past the end. A mutation sweep found
    /// this — every deletion test until now removed a group with another one
    /// behind it.
    #[test]
    fn deleting_the_last_group_in_the_file_does_not_read_past_the_end() {
        let out = apply(
            FILE,
            &[Op::DeleteGroup {
                group: "LOCA".into(),
            }],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert!(!p.groups.contains_key("LOCA"));
        assert_eq!(p.groups["PROJ"].rows.len(), 1);
        // The blank separator BEFORE it belonged to PROJ's section and stays;
        // what must not happen is a panic or a swallowed PROJ row.
        assert!(out.contains("\"DATA\",\"P1\""), "{out}");
    }

    /// A row appended to one group while a column is dropped from ANOTHER.
    /// The pending-row fixup is guarded on the group; without that guard it
    /// would strip a field from a row that has nothing to do with the column.
    #[test]
    fn dropping_a_column_leaves_a_row_added_to_a_different_group_alone() {
        let out = apply(
            FILE,
            &[
                Op::AddRow {
                    group: "PROJ".into(),
                    cells: BTreeMap::from([("PROJ_ID".into(), "P2".into())]),
                },
                Op::DeleteColumn {
                    group: "LOCA".into(),
                    heading: "LOCA_NATE".into(),
                },
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["PROJ"].headings.len(), 2);
        assert_eq!(p.groups["PROJ"].rows.len(), 2);
        assert!(
            p.groups["PROJ"].rows.iter().all(|r| r.n_values() == 2),
            "PROJ's appended row must keep PROJ's arity: {:?}",
            p.groups["PROJ"].rows
        );
        assert_eq!(cell(&out, "PROJ", 2, "PROJ_ID"), "P2");
        assert_eq!(p.groups["LOCA"].headings.len(), 2);
    }

    /// TWO columns dropped from one group. Each removal shortens the line, so
    /// the second one's index — computed against the ORIGINAL headings — lands
    /// somewhere else, or past the end. A mutation sweep pointed at the guard
    /// that was silently absorbing it.
    #[test]
    fn dropping_two_columns_from_one_group_removes_both_whichever_order() {
        let first = Op::DeleteColumn {
            group: "LOCA".into(),
            heading: "LOCA_NATE".into(),
        };
        let second = Op::DeleteColumn {
            group: "LOCA".into(),
            heading: "LOCA_REM".into(),
        };
        for ops in [
            vec![first.clone(), second.clone()],
            vec![second.clone(), first.clone()],
        ] {
            let out = apply(FILE, &ops).unwrap();
            let p = laterite_ags4_parse::parse_str(&out).unwrap();
            let g = &p.groups["LOCA"];
            assert_eq!(g.headings, ["LOCA_ID"], "both columns must go: {out}");
            assert!(
                g.rows.iter().all(|r| r.n_values() == 1),
                "no row may be left ragged: {:?}",
                g.rows
            );
            assert_eq!(cell(&out, "LOCA", 1, "LOCA_ID"), "BH1");
        }
    }

    fn set_unit(group: &str, heading: &str, unit: &str) -> Op {
        Op::SetUnit {
            group: group.into(),
            heading: heading.into(),
            unit: unit.into(),
        }
    }

    /// Read a declared UNIT back through the parser, for the same reason
    /// `cell` does: a test that grepped the output would pass on a file the
    /// parser cannot read.
    fn unit_of(text: &str, group: &str, heading: &str) -> String {
        let p = laterite_ags4_parse::parse_str(text).expect("output must re-parse");
        let g = p.groups.get(group).expect("group");
        g.units
            .get(g.col(heading).expect("heading"))
            .expect("UNIT cell")
            .clone()
    }

    #[test]
    fn setting_a_unit_touches_the_unit_row_and_nothing_else() {
        let out = apply(FILE, &[set_unit("LOCA", "LOCA_NATE", "mm")]).unwrap();
        let before: Vec<_> = FILE.lines().collect();
        let after: Vec<_> = out.lines().collect();
        assert_eq!(before.len(), after.len());
        for (i, (b, a)) in before.iter().zip(&after).enumerate() {
            // Index 8 is LOCA's UNIT row — the only line the operation names.
            if i == 8 {
                assert_ne!(b, a, "the UNIT row must change");
            } else {
                assert_eq!(b, a, "line {} must be byte-identical", i + 1);
            }
        }
        assert_eq!(unit_of(&out, "LOCA", "LOCA_NATE"), "mm");
    }

    /// The point of the operation. An empty UNIT is PRESENT and undefined,
    /// never a removed field: dropping the field would shorten the row and
    /// silently change what every column after it means.
    #[test]
    fn an_empty_unit_leaves_the_field_in_place() {
        let out = apply(FILE, &[set_unit("LOCA", "LOCA_NATE", "")]).unwrap();
        assert_eq!(unit_of(&out, "LOCA", "LOCA_NATE"), "");
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(
            p.groups["LOCA"].units.len(),
            3,
            "the row must keep its arity: {out}"
        );
    }

    /// The grammar decision, pinned. AGS date-time headings declare units that
    /// carry colons, so a locator that ate the whole spec would be ambiguous
    /// for exactly the type this machinery exists to reach.
    #[test]
    fn a_unit_may_contain_colons() {
        let out = apply(
            FILE,
            &[set_unit("LOCA", "LOCA_NATE", "yyyy-mm-ddThh:mm:ss")],
        )
        .unwrap();
        assert_eq!(unit_of(&out, "LOCA", "LOCA_NATE"), "yyyy-mm-ddThh:mm:ss");
    }

    #[test]
    fn setting_a_unit_on_a_heading_that_is_not_there_is_refused() {
        assert_eq!(
            apply(FILE, &[set_unit("LOCA", "LOCA_NOPE", "mm")]),
            Err(EditError::NoSuchHeading {
                group: "LOCA".into(),
                heading: "LOCA_NOPE".into(),
            })
        );
    }

    /// A group with no UNIT row is a shape forge itself manufactures. Refusing
    /// by name is the answer: writing the row would be a different operation,
    /// and writing it silently would make the patch mean something its author
    /// never asked for.
    #[test]
    fn a_group_with_no_unit_row_is_refused_by_name() {
        let no_unit = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\"\r\n",
            "\"TYPE\",\"ID\"\r\n",
            "\"DATA\",\"P1\"\r\n",
        );
        assert_eq!(
            apply(no_unit, &[set_unit("PROJ", "PROJ_ID", "m")]),
            Err(EditError::MissingDescriptor {
                group: "PROJ".into(),
                row: "UNIT",
            })
        );
    }

    /// The parser does not pad the UNIT row, so a file may declare fewer units
    /// than it has headings. A write past the end has to grow the row rather
    /// than run off it — the alternatives are a panic and a torn row.
    #[test]
    fn a_short_unit_row_grows_rather_than_tearing() {
        let short = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
            "\"DATA\",\"BH1\",\"100.00\",\"x\"\r\n",
        );
        let out = apply(short, &[set_unit("LOCA", "LOCA_REM", "%")]).unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(p.groups["LOCA"].units, ["", "", "%"]);
    }

    /// Listing order must not decide the answer: a UNIT edit and a cell edit
    /// in one patch mean the same thing whichever way round they are written.
    #[test]
    fn a_unit_edit_and_a_cell_edit_commute() {
        let a = apply(
            FILE,
            &[
                set_unit("LOCA", "LOCA_NATE", "mm"),
                set("LOCA", 1, "LOCA_NATE", "1.00"),
            ],
        )
        .unwrap();
        let b = apply(
            FILE,
            &[
                set("LOCA", 1, "LOCA_NATE", "1.00"),
                set_unit("LOCA", "LOCA_NATE", "mm"),
            ],
        )
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(unit_of(&a, "LOCA", "LOCA_NATE"), "mm");
        assert_eq!(cell(&a, "LOCA", 1, "LOCA_NATE"), "1.00");
    }

    #[test]
    fn the_set_unit_flag_splits_the_locator_and_not_the_value() {
        assert_eq!(
            parse_flag(OpKind::SetUnit, "LOCA:LOCA_DATE=yyyy-mm-ddThh:mm:ss").unwrap(),
            set_unit("LOCA", "LOCA_DATE", "yyyy-mm-ddThh:mm:ss")
        );
        // Legal, and the whole reason the operation exists.
        assert_eq!(
            parse_flag(OpKind::SetUnit, "LOCA:LOCA_NATE=").unwrap(),
            set_unit("LOCA", "LOCA_NATE", "")
        );
        assert!(
            parse_flag(OpKind::SetUnit, "LOCA:LOCA_NATE").is_err(),
            "a spec with no `=` names no unit"
        );
        assert!(
            parse_flag(OpKind::SetUnit, "LOCA=mm").is_err(),
            "a spec with no `:` names no heading"
        );
    }

    fn set_type(group: &str, heading: &str, ags_type: &str) -> Op {
        Op::SetType {
            group: group.into(),
            heading: heading.into(),
            ags_type: ags_type.into(),
            reformat: true,
        }
    }

    fn set_type_raw(group: &str, heading: &str, ags_type: &str) -> Op {
        Op::SetType {
            group: group.into(),
            heading: heading.into(),
            ags_type: ags_type.into(),
            reformat: false,
        }
    }

    /// A DT column at full precision, so a projection down to a date has
    /// something real to do.
    const DATED: &str = concat!(
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_DATE\"\r\n",
        "\"UNIT\",\"\",\"yyyy-mm-ddThh:mm:ss\"\r\n",
        "\"TYPE\",\"ID\",\"DT\"\r\n",
        "\"DATA\",\"BH1\",\"2026-08-26T00:00:00\"\r\n",
    );

    fn declared_type(text: &str, group: &str, heading: &str) -> String {
        let p = laterite_ags4_parse::parse_str(text).expect("output must re-parse");
        let g = p.groups.get(group).expect("group");
        g.types
            .get(g.col(heading).expect("heading"))
            .expect("TYPE cell")
            .clone()
    }

    #[test]
    fn retyping_reformats_the_column_and_touches_nothing_else() {
        let out = apply(FILE, &[set_type("LOCA", "LOCA_NATE", "3DP")]).unwrap();
        assert_eq!(declared_type(&out, "LOCA", "LOCA_NATE"), "3DP");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_NATE"), "100.000");
        assert_eq!(cell(&out, "LOCA", 2, "LOCA_NATE"), "200.000");
        // The other columns keep their values, quoting and all.
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_REM"), "the \"good\" one");
        // PROJ is untouched entirely.
        let before: Vec<_> = FILE.lines().collect();
        let after: Vec<_> = out.lines().collect();
        for i in 0..6 {
            assert_eq!(before[i], after[i], "line {} must be byte-identical", i + 1);
        }
    }

    /// The fault-injection path: the declaration moves and the values do not,
    /// which is the only way to get a cell that contradicts its own type.
    #[test]
    fn the_raw_spelling_declares_the_type_and_leaves_the_values() {
        let out = apply(FILE, &[set_type_raw("LOCA", "LOCA_NATE", "DT")]).unwrap();
        assert_eq!(declared_type(&out, "LOCA", "LOCA_NATE"), "DT");
        assert_eq!(
            cell(&out, "LOCA", 1, "LOCA_NATE"),
            "100.00",
            "the value must survive a retype it cannot satisfy"
        );
    }

    /// Projecting to a text type keeps the text exactly, including the digits
    /// a numeric projection would have re-rendered.
    #[test]
    fn projecting_into_a_text_type_passes_the_value_through() {
        let out = apply(FILE, &[set_type("LOCA", "LOCA_NATE", "X")]).unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_NATE"), "100.00");
    }

    #[test]
    fn an_unreadable_type_token_is_refused() {
        assert_eq!(
            apply(FILE, &[set_type("LOCA", "LOCA_NATE", "NOPE")]),
            Err(EditError::UnknownType {
                group: "LOCA".into(),
                heading: "LOCA_NATE".into(),
                ags_type: "NOPE".into(),
            })
        );
    }

    /// A type the dictionary never pairs with this heading is NOT refused —
    /// manufacturing that mismatch is the tool's job. Only a token the type
    /// system cannot read at all is.
    #[test]
    fn a_type_the_dictionary_would_not_pair_is_still_allowed() {
        let out = apply(FILE, &[set_type("LOCA", "LOCA_REM", "YN")]);
        // LOCA_REM's text cannot be projected to YN, so this refuses on the
        // VALUE — proving it got past the type check rather than failing it.
        assert!(
            matches!(out, Err(EditError::Unprojectable { .. })),
            "must fail on the value, not the token: {out:?}"
        );
    }

    #[test]
    fn a_value_that_cannot_be_projected_is_refused_naming_its_row() {
        assert_eq!(
            apply(FILE, &[set_type("LOCA", "LOCA_REM", "2DP")]),
            Err(EditError::Unprojectable {
                group: "LOCA".into(),
                heading: "LOCA_REM".into(),
                row: 1,
                value: "the \"good\" one".into(),
                ags_type: "2DP".into(),
            })
        );
    }

    #[test]
    fn a_group_with_no_type_row_is_refused_by_name() {
        let no_type = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"DATA\",\"P1\"\r\n",
        );
        assert_eq!(
            apply(no_type, &[set_type("PROJ", "PROJ_ID", "X")]),
            Err(EditError::MissingDescriptor {
                group: "PROJ".into(),
                row: "TYPE",
            })
        );
    }

    /// The interaction that makes descriptor edits rank ahead of everything
    /// else: a UNIT set in the SAME patch is what the DT projection reads. If
    /// it read the file's original unit the value would come back unchanged,
    /// because at full precision there is nothing to drop.
    #[test]
    fn a_unit_set_in_the_same_patch_decides_the_dt_projection() {
        let out = apply(
            DATED,
            &[
                set_type("LOCA", "LOCA_DATE", "DT"),
                Op::SetUnit {
                    group: "LOCA".into(),
                    heading: "LOCA_DATE".into(),
                    unit: "yyyy-mm-dd".into(),
                },
            ],
        )
        .unwrap();
        assert_eq!(unit_of(&out, "LOCA", "LOCA_DATE"), "yyyy-mm-dd");
        assert_eq!(
            cell(&out, "LOCA", 1, "LOCA_DATE"),
            "2026-08-26",
            "the projection must read the unit this patch declared"
        );
    }

    /// A DT value carrying a real time is refused rather than truncated. The
    /// asymmetry with the numeric families is deliberate and documented where
    /// the projection lives.
    #[test]
    fn a_dt_projection_that_would_discard_a_real_time_is_refused() {
        let timed = DATED.replace("2026-08-26T00:00:00", "2026-08-26T09:15:00");
        let out = apply(
            &timed,
            &[
                Op::SetUnit {
                    group: "LOCA".into(),
                    heading: "LOCA_DATE".into(),
                    unit: "yyyy-mm-dd".into(),
                },
                set_type("LOCA", "LOCA_DATE", "DT"),
            ],
        );
        assert!(
            matches!(out, Err(EditError::Unprojectable { row: 1, .. })),
            "{out:?}"
        );
    }

    #[test]
    fn the_two_set_type_spellings_differ_only_in_reformatting() {
        assert_eq!(
            parse_flag(OpKind::SetType, "LOCA:LOCA_NATE=3DP").unwrap(),
            set_type("LOCA", "LOCA_NATE", "3DP")
        );
        assert_eq!(
            parse_flag(OpKind::SetTypeRaw, "LOCA:LOCA_NATE=3DP").unwrap(),
            set_type_raw("LOCA", "LOCA_NATE", "3DP")
        );
        assert!(parse_flag(OpKind::SetType, "LOCA:LOCA_NATE").is_err());
        assert!(parse_flag(OpKind::SetType, "LOCA=3DP").is_err());
    }

    /// A patch file that omits `reformat` means re-format: the safe half.
    #[test]
    fn a_patch_omitting_reformat_still_reformats() {
        let patch = r#"
[[op]]
kind = "set-type"
group = "LOCA"
heading = "LOCA_NATE"
type = "0DP"
"#;
        let ops: Patch = toml::from_str(patch).expect("patch loads");
        assert_eq!(ops.op, vec![set_type("LOCA", "LOCA_NATE", "0DP")]);
        let out = apply(FILE, &ops.op).unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_NATE"), "100");
    }

    fn add_column(group: &str, heading: &str) -> Op {
        Op::AddColumn {
            group: group.into(),
            heading: heading.into(),
        }
    }

    #[test]
    fn adding_a_column_reaches_every_row_the_group_owns() {
        let out = apply(FILE, &[add_column("LOCA", "LOCA_GL")]).unwrap();
        let p = laterite_ags4_parse::parse_str(&out).expect("output must re-parse");
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_NATE", "LOCA_REM", "LOCA_GL"]);
        assert_eq!(g.units.len(), 4, "the UNIT row must keep pace: {out}");
        assert_eq!(g.types.len(), 4, "the TYPE row must keep pace: {out}");
        for (i, row) in g.rows.iter().enumerate() {
            assert_eq!(row.n_values(), 4, "row {} must not be ragged", i + 1);
        }
        // Empty on purpose: the caller decides what goes in, not the tool.
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_GL"), "");
        assert_eq!(cell(&out, "LOCA", 2, "LOCA_GL"), "");
        assert_eq!(unit_of(&out, "LOCA", "LOCA_GL"), "");
        assert_eq!(declared_type(&out, "LOCA", "LOCA_GL"), "");
        // PROJ is a different group and must not have moved at all.
        let before: Vec<_> = FILE.lines().collect();
        let after: Vec<_> = out.lines().collect();
        for i in 0..6 {
            assert_eq!(before[i], after[i], "line {} must be byte-identical", i + 1);
        }
    }

    #[test]
    fn a_heading_the_group_already_has_is_refused() {
        assert_eq!(
            apply(FILE, &[add_column("LOCA", "LOCA_NATE")]),
            Err(EditError::DuplicateHeading {
                group: "LOCA".into(),
                heading: "LOCA_NATE".into(),
            })
        );
    }

    /// A heading the dictionary never heard of is ACCEPTED. Forbidding it
    /// would remove a class of fault the tool exists to manufacture, and
    /// judging heading names is the validator's job.
    #[test]
    fn a_heading_the_dictionary_does_not_know_is_accepted() {
        let out = apply(FILE, &[add_column("LOCA", "LOCA_INVENTED")]).unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert!(p.groups["LOCA"].col("LOCA_INVENTED").is_some());
    }

    #[test]
    fn a_group_with_no_heading_row_is_refused_by_name() {
        let headless = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"TYPE\",\"ID\"\r\n",
            "\"DATA\",\"P1\"\r\n",
        );
        assert_eq!(
            apply(headless, &[add_column("PROJ", "PROJ_NAME")]),
            Err(EditError::MissingDescriptor {
                group: "PROJ".into(),
                row: "HEADING",
            })
        );
    }

    /// The payoff of #723: a locator naming a column that did not exist when
    /// the file was parsed still resolves, because the patch created it.
    #[test]
    fn a_column_created_in_the_same_patch_can_be_written() {
        let out = apply(
            FILE,
            &[
                set("LOCA", 1, "LOCA_GL", "12.34"),
                add_column("LOCA", "LOCA_GL"),
            ],
        )
        .unwrap();
        assert_eq!(
            cell(&out, "LOCA", 1, "LOCA_GL"),
            "12.34",
            "creation ranks ahead of the write, whatever order they were listed in"
        );
    }

    /// One patch projects a file: create the column, declare its UNIT and
    /// TYPE, and fill it.
    #[test]
    fn a_column_created_in_the_same_patch_can_be_declared_and_filled() {
        let out = apply(
            FILE,
            &[
                add_column("LOCA", "LOCA_GL"),
                Op::SetUnit {
                    group: "LOCA".into(),
                    heading: "LOCA_GL".into(),
                    unit: "m".into(),
                },
                set_type_raw("LOCA", "LOCA_GL", "2DP"),
                set("LOCA", 1, "LOCA_GL", "12.34"),
            ],
        )
        .unwrap();
        assert_eq!(unit_of(&out, "LOCA", "LOCA_GL"), "m");
        assert_eq!(declared_type(&out, "LOCA", "LOCA_GL"), "2DP");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_GL"), "12.34");
        assert_eq!(cell(&out, "LOCA", 2, "LOCA_GL"), "");
    }

    /// A row appended by the same patch is built to the group's arity, which
    /// by then includes the new column — `AddColumn` ranks ahead of `AddRow`
    /// exactly so this holds without a second pass.
    #[test]
    fn a_row_appended_in_the_same_patch_carries_the_new_column() {
        let out = apply(
            FILE,
            &[
                Op::AddRow {
                    group: "LOCA".into(),
                    cells: BTreeMap::from([("LOCA_ID".to_string(), "BH3".to_string())]),
                },
                add_column("LOCA", "LOCA_GL"),
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.rows.len(), 3);
        for (i, row) in g.rows.iter().enumerate() {
            assert_eq!(row.n_values(), 4, "row {} must not be ragged", i + 1);
        }
        assert_eq!(cell(&out, "LOCA", 3, "LOCA_ID"), "BH3");
        assert_eq!(cell(&out, "LOCA", 3, "LOCA_GL"), "");
    }

    /// The parser does not pad the descriptor rows, so a file may declare
    /// fewer units than headings. Adding a column has to bring every row up,
    /// or the new cell lands in a different column on each row.
    #[test]
    fn adding_a_column_brings_a_short_descriptor_row_up() {
        let short = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"2DP\"\r\n",
            "\"DATA\",\"BH1\",\"100.00\"\r\n",
        );
        let out = apply(short, &[add_column("LOCA", "LOCA_GL")]).unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_NATE", "LOCA_GL"]);
        assert_eq!(g.units, ["", "", ""]);
        assert_eq!(g.rows[0].n_values(), 3);
    }

    /// Adding and dropping the same column in one patch is a no-op on the
    /// group's shape. Removals rank last, so the pair cannot half-apply.
    #[test]
    fn adding_then_dropping_a_column_leaves_the_original_shape() {
        let out = apply(
            FILE,
            &[
                add_column("LOCA", "LOCA_GL"),
                Op::DeleteColumn {
                    group: "LOCA".into(),
                    heading: "LOCA_GL".into(),
                },
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_NATE", "LOCA_REM"]);
        assert!(g.rows.iter().all(|r| r.n_values() == 3), "{out}");
    }

    #[test]
    fn the_add_column_flag_takes_a_group_and_a_heading() {
        assert_eq!(
            parse_flag(OpKind::AddColumn, "LOCA:LOCA_GL").unwrap(),
            add_column("LOCA", "LOCA_GL")
        );
        assert!(parse_flag(OpKind::AddColumn, "LOCA").is_err());
    }

    fn insert_row(group: &str, at: usize, cells: &[(&str, &str)]) -> Op {
        Op::InsertRow {
            group: group.into(),
            at,
            cells: cells
                .iter()
                .map(|(h, v)| ((*h).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    fn ids(text: &str, group: &str, heading: &str) -> Vec<String> {
        let p = laterite_ags4_parse::parse_str(text).expect("output must re-parse");
        let g = p.groups.get(group).expect("group");
        let col = g.col(heading).expect("heading");
        (0..g.rows.len())
            .map(|r| g.cell(col, r).unwrap_or_default().to_string())
            .collect()
    }

    #[test]
    fn a_row_inserted_at_a_position_becomes_that_position() {
        let out = apply(FILE, &[insert_row("LOCA", 2, &[("LOCA_ID", "BH1a")])]).unwrap();
        assert_eq!(ids(&out, "LOCA", "LOCA_ID"), ["BH1", "BH1a", "BH2"]);
    }

    /// Position 1 has no earlier row to sit after, so it anchors to the
    /// group's descriptors — never above them, and never in the group before.
    #[test]
    fn a_row_inserted_at_position_one_lands_under_the_descriptors() {
        let out = apply(FILE, &[insert_row("LOCA", 1, &[("LOCA_ID", "BH0")])]).unwrap();
        assert_eq!(ids(&out, "LOCA", "LOCA_ID"), ["BH0", "BH1", "BH2"]);
        // PROJ, which sits ABOVE LOCA in the file, must be untouched.
        assert_eq!(cell(&out, "PROJ", 1, "PROJ_ID"), "P1");
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert_eq!(
            p.groups["PROJ"].rows.len(),
            1,
            "the row went to LOCA: {out}"
        );
    }

    /// A group missing its TYPE row still anchors below the rows it has,
    /// which a first-match chain over HEADING/UNIT/TYPE would get wrong.
    #[test]
    fn position_one_anchors_below_the_descriptor_rows_a_group_actually_has() {
        let no_type = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"DATA\",\"BH1\"\r\n",
        );
        let out = apply(no_type, &[insert_row("LOCA", 1, &[("LOCA_ID", "BH0")])]).unwrap();
        assert_eq!(ids(&out, "LOCA", "LOCA_ID"), ["BH0", "BH1"]);
        assert!(
            out.starts_with("\"GROUP\",\"LOCA\"\r\n\"HEADING\""),
            "the descriptors must still come first: {out}"
        );
    }

    #[test]
    fn a_position_past_the_last_row_is_refused_rather_than_appended() {
        assert_eq!(
            apply(FILE, &[insert_row("LOCA", 3, &[])]),
            Err(EditError::NoSuchRow {
                group: "LOCA".into(),
                row: 3,
                rows: 2,
            })
        );
        assert_eq!(
            apply(FILE, &[insert_row("LOCA", 0, &[])]),
            Err(EditError::NoSuchRow {
                group: "LOCA".into(),
                row: 0,
                rows: 2,
            })
        );
    }

    /// A group with no data rows has no position to insert at. `--add-row` is
    /// the operation for that, and saying so beats guessing.
    #[test]
    fn a_group_with_no_rows_refuses_every_position() {
        let empty = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\"\r\n",
            "\"UNIT\",\"\"\r\n",
            "\"TYPE\",\"ID\"\r\n",
        );
        assert!(matches!(
            apply(empty, &[insert_row("LOCA", 1, &[])]),
            Err(EditError::NoSuchRow { rows: 0, .. })
        ));
    }

    /// Row numbers keep counting the ORIGINAL file, so an insertion does not
    /// renumber the rows a later operation names.
    #[test]
    fn an_insertion_does_not_renumber_the_rows_a_later_operation_names() {
        let out = apply(
            FILE,
            &[
                insert_row("LOCA", 1, &[("LOCA_ID", "BH0")]),
                set("LOCA", 2, "LOCA_ID", "renamed"),
            ],
        )
        .unwrap();
        assert_eq!(
            ids(&out, "LOCA", "LOCA_ID"),
            ["BH0", "BH1", "renamed"],
            "row 2 must still mean the ORIGINAL row 2"
        );
    }

    #[test]
    fn two_insertions_at_one_position_keep_the_order_they_were_listed_in() {
        let out = apply(
            FILE,
            &[
                insert_row("LOCA", 1, &[("LOCA_ID", "first")]),
                insert_row("LOCA", 1, &[("LOCA_ID", "second")]),
            ],
        )
        .unwrap();
        assert_eq!(
            ids(&out, "LOCA", "LOCA_ID"),
            ["first", "second", "BH1", "BH2"]
        );
    }

    /// The shape this whole layer exists to prevent: an inserted row and a
    /// dropped column in one patch must not leave a row of the wrong width.
    #[test]
    fn an_insertion_and_a_column_drop_leave_no_ragged_row() {
        let out = apply(
            FILE,
            &[
                insert_row("LOCA", 1, &[("LOCA_ID", "BH0")]),
                Op::DeleteColumn {
                    group: "LOCA".into(),
                    heading: "LOCA_REM".into(),
                },
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        let g = &p.groups["LOCA"];
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_NATE"]);
        assert!(g.rows.iter().all(|r| r.n_values() == 2), "{out}");
    }

    /// Deleting the group wins over inserting into it — the same reading the
    /// canonical order gives every other write.
    #[test]
    fn an_insertion_into_a_deleted_group_does_not_resurrect_it() {
        let out = apply(
            FILE,
            &[
                insert_row("LOCA", 1, &[("LOCA_ID", "BH0")]),
                Op::DeleteGroup {
                    group: "LOCA".into(),
                },
            ],
        )
        .unwrap();
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert!(!p.groups.contains_key("LOCA"), "LOCA must be gone: {out}");
        assert!(
            !out.contains("BH0"),
            "the inserted row must go with it: {out}"
        );
    }

    /// A column created by the same patch reaches an inserted row too.
    #[test]
    fn a_row_inserted_in_the_same_patch_carries_a_created_column() {
        let out = apply(
            FILE,
            &[
                add_column("LOCA", "LOCA_GL"),
                insert_row("LOCA", 1, &[("LOCA_GL", "9.99")]),
            ],
        )
        .unwrap();
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_GL"), "9.99");
        let p = laterite_ags4_parse::parse_str(&out).unwrap();
        assert!(p.groups["LOCA"].rows.iter().all(|r| r.n_values() == 4));
    }

    #[test]
    fn the_insert_row_flag_takes_a_group_and_a_position() {
        assert_eq!(
            parse_flag(OpKind::InsertRow, "LOCA:2").unwrap(),
            insert_row("LOCA", 2, &[])
        );
        assert!(parse_flag(OpKind::InsertRow, "LOCA").is_err());
        assert!(parse_flag(OpKind::InsertRow, "LOCA:x").is_err());
    }

    /// The whole projection, listed four ways. Canonical order — not listing
    /// order — decides the result, and the strongest statement of that is
    /// byte-identity rather than "the cells came out right".
    #[test]
    fn a_projection_means_the_same_thing_in_any_order() {
        let create = add_column("LOCA", "LOCA_GL");
        let unit = Op::SetUnit {
            group: "LOCA".into(),
            heading: "LOCA_GL".into(),
            unit: "m".into(),
        };
        let ty = set_type_raw("LOCA", "LOCA_GL", "3SF");
        let fill = set("LOCA", 1, "LOCA_GL", "21.5");
        let insert = insert_row("LOCA", 1, &[("LOCA_ID", "BH0")]);

        let orders: [Vec<Op>; 4] = [
            vec![
                create.clone(),
                unit.clone(),
                ty.clone(),
                fill.clone(),
                insert.clone(),
            ],
            vec![
                insert.clone(),
                fill.clone(),
                ty.clone(),
                unit.clone(),
                create.clone(),
            ],
            vec![
                fill.clone(),
                create.clone(),
                insert.clone(),
                ty.clone(),
                unit.clone(),
            ],
            vec![
                ty.clone(),
                insert.clone(),
                unit.clone(),
                create.clone(),
                fill.clone(),
            ],
        ];
        let first = apply(FILE, &orders[0]).unwrap();
        for (i, ops) in orders.iter().enumerate().skip(1) {
            assert_eq!(
                apply(FILE, ops).unwrap(),
                first,
                "order {i} must give the same bytes"
            );
        }

        // And the result is the one that was asked for, not merely a stable
        // wrong answer — an ordering test passes just as readily on both.
        assert_eq!(unit_of(&first, "LOCA", "LOCA_GL"), "m");
        assert_eq!(declared_type(&first, "LOCA", "LOCA_GL"), "3SF");
        // Row 1 of the OUTPUT is the inserted row. The fill named row 1 of the
        // INPUT, which the insertion pushed down to row 2. Both readings are
        // correct, and the difference between them is precisely what original
        // numbering keeps out of a patch: inside the patch there is only ever
        // one answer to "row 1".
        assert_eq!(cell(&first, "LOCA", 1, "LOCA_ID"), "BH0");
        assert_eq!(cell(&first, "LOCA", 1, "LOCA_GL"), "");
        assert_eq!(cell(&first, "LOCA", 2, "LOCA_GL"), "21.5");
        let p = laterite_ags4_parse::parse_str(&first).unwrap();
        assert!(p.groups["LOCA"].rows.iter().all(|r| r.n_values() == 4));
    }

    /// A projection does not care whether the file it is given came out of an
    /// earlier run: two passes and one patch agree, because every operation
    /// resolves against the file it actually received.
    #[test]
    fn two_passes_and_one_patch_agree() {
        let edit = set("LOCA", 1, "LOCA_NATE", "111.00");
        let projection = [
            add_column("LOCA", "LOCA_GL"),
            Op::SetUnit {
                group: "LOCA".into(),
                heading: "LOCA_GL".into(),
                unit: "m".into(),
            },
            set_type_raw("LOCA", "LOCA_GL", "3SF"),
            set("LOCA", 2, "LOCA_GL", "21.5"),
        ];

        let after_first = apply(FILE, std::slice::from_ref(&edit)).unwrap();
        let two_passes = apply(&after_first, &projection).unwrap();

        let mut both = vec![edit];
        both.extend(projection);
        let one_patch = apply(FILE, &both).unwrap();

        assert_eq!(two_passes, one_patch);
        assert_eq!(cell(&two_passes, "LOCA", 1, "LOCA_NATE"), "111.00");
        assert_eq!(cell(&two_passes, "LOCA", 2, "LOCA_GL"), "21.5");
    }

    /// The template is the worked example a reader copies, so it has to BE a
    /// projection that works — not merely one that parses.
    #[test]
    fn the_templates_worked_example_projects_a_column() {
        let patch: Patch = toml::from_str(&Patch::template()).expect("template loads");
        let out = apply(FILE, &patch.op).expect("the template must apply to a LOCA file");
        assert_eq!(unit_of(&out, "LOCA", "LOCA_GL"), "m");
        assert_eq!(declared_type(&out, "LOCA", "LOCA_GL"), "3SF");
        assert_eq!(cell(&out, "LOCA", 1, "LOCA_GL"), "21.5");
    }
}
