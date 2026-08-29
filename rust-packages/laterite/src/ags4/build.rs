//! Constructing AGS4 from data you hold, rather than from a file you read.

use laterite_ags4_emit::GroupInput;

use super::{Document, WriteMode, Written, emit_groups};
use crate::{Error, ErrorKind};

/// One cell's value, before AGS4 formatting.
///
/// Every AGS4 cell is a string on the wire, but *which* string depends on the
/// heading's declared TYPE: `1.5` under `2DP` is `1.50`, and under `X` it is
/// `1.5`. Handing the builder a typed value lets it do that formatting; handing
/// it a string is a statement that the string is already what you want written,
/// and it goes out verbatim.
///
/// This is our own enum rather than the engine's. That is the facade's
/// central rule — no non-`laterite` type in a public signature — and though
/// the engine's row type is now a near-identical scalar enum of our own
/// (`laterite_ags4_types::Cell`, since #790; it was a `serde_json::Value`,
/// which is what this type was built to keep out of our API), the gate that
/// enforces the rule (`tools/check_public_api.py`) cannot tell a benign
/// engine type from a binding one, and holding the two apart keeps the
/// engine free to reshape its cell without the facade's semver noticing.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// No value. Emitted as the empty cell AGS4 uses for absent data.
    Null,
    /// Written verbatim — see the note above about formatting.
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<&str> for Cell {
    fn from(s: &str) -> Cell {
        Cell::Text(s.to_string())
    }
}

impl From<String> for Cell {
    fn from(s: String) -> Cell {
        Cell::Text(s)
    }
}

impl From<&String> for Cell {
    fn from(s: &String) -> Cell {
        Cell::Text(s.clone())
    }
}

impl From<i64> for Cell {
    fn from(v: i64) -> Cell {
        Cell::Int(v)
    }
}

impl From<i32> for Cell {
    fn from(v: i32) -> Cell {
        Cell::Int(i64::from(v))
    }
}

impl From<f64> for Cell {
    fn from(v: f64) -> Cell {
        Cell::Float(v)
    }
}

impl From<bool> for Cell {
    fn from(v: bool) -> Cell {
        Cell::Bool(v)
    }
}

/// `None` is the absent cell — so an `Option<T>` column maps straight across
/// without the caller flattening it first.
impl<T: Into<Cell>> From<Option<T>> for Cell {
    fn from(v: Option<T>) -> Cell {
        v.map_or(Cell::Null, Into::into)
    }
}

impl Cell {
    /// Consuming on purpose: `Text` moves its `String` across the boundary
    /// instead of cloning it per cell — the same peak-shaving #788/#789
    /// applied at the emit door, applied at this one (#790). A non-finite
    /// float still crosses as the engine's `Null`, exactly as it did when
    /// the engine cell was a `serde_json::Value` (JSON cannot carry NaN, and
    /// the emitted bytes are pinned to the blank it produced).
    fn into_engine(self) -> laterite_ags4_emit::Cell {
        match self {
            Cell::Null => laterite_ags4_emit::Cell::Null,
            Cell::Text(s) => laterite_ags4_emit::Cell::Text(s),
            Cell::Int(v) => laterite_ags4_emit::Cell::Int(v),
            Cell::Float(v) => laterite_ags4_emit::Cell::from(v),
            Cell::Bool(v) => laterite_ags4_emit::Cell::Bool(v),
        }
    }
}

/// One group's worth of data to build from: its code, its headings, and its
/// rows.
///
/// `UNIT` and `TYPE` are filled from the chosen edition unless you set them —
/// which is what makes this a *data* door rather than a file-assembly one. Set
/// them for a heading the standard dictionary does not know.
#[derive(Debug, Clone)]
pub struct GroupData {
    code: String,
    headings: Vec<String>,
    units: Option<Vec<String>>,
    types: Option<Vec<String>>,
    rows: Vec<Vec<Cell>>,
}

impl GroupData {
    /// A group with these headings and no rows yet.
    pub fn new<I, S>(code: impl Into<String>, headings: I) -> GroupData
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        GroupData {
            code: code.into(),
            headings: headings.into_iter().map(Into::into).collect(),
            units: None,
            types: None,
            rows: Vec::new(),
        }
    }

    /// Set the `UNIT` row explicitly, one entry per heading.
    ///
    /// Without this the units come from the dictionary. Give it for a heading
    /// the standard dictionary has never heard of, which is otherwise emitted
    /// with an empty unit.
    #[must_use]
    pub fn units<I, S>(mut self, units: I) -> GroupData
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.units = Some(units.into_iter().map(Into::into).collect());
        self
    }

    /// Set the `TYPE` row explicitly, one entry per heading — see
    /// [`GroupData::units`].
    #[must_use]
    pub fn types<I, S>(mut self, types: I) -> GroupData
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.types = Some(types.into_iter().map(Into::into).collect());
        self
    }

    /// Add a row. One cell per heading, in heading order.
    ///
    /// Arity is checked when the build runs rather than here, so assembling a
    /// group stays infallible and the error names the group it came from.
    #[must_use]
    pub fn row<I, C>(mut self, cells: I) -> GroupData
    where
        I: IntoIterator<Item = C>,
        C: Into<Cell>,
    {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    /// The group's four-letter code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// How many rows have been added.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the group has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Check arity and hand the engine its own shape. Consuming: the rows
    /// move — see [`Cell::into_engine`].
    fn into_engine(self) -> Result<GroupInput, Error> {
        let width = self.headings.len();
        // Checked here, not by the emitter. A short row is padded by the safe
        // fix set and a long one is silently truncated by the writer, so a
        // miscounted row would come back as a *finding* about the output rather
        // than as the mistake it is — a caller who mismatched their own columns
        // is better told so than quietly corrected.
        for (i, row) in self.rows.iter().enumerate() {
            if row.len() != width {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "{}: row {i} has {} cells but the group declares {width} headings",
                        self.code,
                        row.len()
                    ),
                ));
            }
        }
        for (what, meta) in [
            ("units", self.units.as_ref()),
            ("types", self.types.as_ref()),
        ] {
            if let Some(m) = meta {
                if m.len() != width {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "{}: {what} has {} entries but the group declares {width} headings",
                            self.code,
                            m.len()
                        ),
                    ));
                }
            }
        }
        Ok(GroupInput {
            code: self.code,
            headings: self.headings,
            units: self.units,
            types: self.types,
            rows: self
                .rows
                .into_iter()
                .map(|r| r.into_iter().map(Cell::into_engine).collect())
                .collect(),
        })
    }
}

/// A pending build. Configure it, then [`Build::run`].
pub struct Build {
    groups: Vec<GroupData>,
    mode: WriteMode,
    edition: Option<String>,
    synthesise_metadata: bool,
    tran: Option<laterite_ags4_emit::TranStamp>,
}

/// Build AGS4 from data you hold.
///
/// The data door, where [`write`](super::write) is the document one: this takes
/// the groups, headings and rows themselves — out of a query result, a
/// spreadsheet, your own structs — and produces AGS4 from them, deriving the
/// `UNIT` and `TYPE` catalogues and validating what it wrote.
///
/// Order is preserved, and `PROJ` goes first.
///
/// ```no_run
/// use laterite::ags4::{self, Cell, GroupData};
///
/// let proj = GroupData::new("PROJ", ["PROJ_ID", "PROJ_NAME"]).row(["P1", "A site"]);
/// // A mixed row is a row of `Cell`, named: `12.5` and `"BH01"` convert to
/// // different variants, so the array's element type has to be said out loud.
/// let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"])
///     .row([Cell::from("BH01"), Cell::from(12.5)]);
///
/// let built = ags4::build(vec![proj, loca]).run()?;
/// println!("{}", built.text());
/// # Ok::<(), laterite::Error>(())
/// ```
#[must_use]
pub fn build(groups: Vec<GroupData>) -> Build {
    Build {
        groups,
        mode: WriteMode::default(),
        edition: None,
        synthesise_metadata: true,
        tran: None,
    }
}

/// Build AGS4 from a document you already hold.
///
/// The same pipeline as [`build`], fed from a [`Document`] — read one, edit it
/// with [`Document::set_cell`] and [`Document::push_row`], then build. Cells
/// carry across as text, because that is what a document holds: they were
/// already formatted when the file was written.
#[must_use]
pub fn build_document(doc: &Document) -> Build {
    let groups = doc
        .groups()
        .iter()
        .map(|g| {
            let headings = g.headings();
            let mut data = GroupData::new(g.code(), headings.iter().copied())
                .units(g.units().iter().copied())
                .types(g.types().iter().copied());
            for row in g.rows() {
                data = data.row(
                    headings
                        .iter()
                        .map(|h| Cell::Text(row.cell(h).unwrap_or("").to_string())),
                );
            }
            data
        })
        .collect();
    build(groups)
}

impl Build {
    /// What to do about validity — see [`WriteMode`].
    #[must_use]
    pub fn mode(mut self, mode: WriteMode) -> Build {
        self.mode = mode;
        self
    }

    /// Build against a specific AGS4 edition. See [`editions`](super::editions).
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Build {
        self.edition = Some(edition.into());
        self
    }

    /// Derive the `UNIT` and `TYPE` catalogue groups from the data. On by
    /// default — see [`Write::synthesise_metadata`](super::Write::synthesise_metadata).
    #[must_use]
    pub fn synthesise_metadata(mut self, yes: bool) -> Build {
        self.synthesise_metadata = yes;
        self
    }

    /// State the transmission this file represents, producing a `TRAN` group.
    ///
    /// Without it no `TRAN` is written at all and validation says so — see
    /// [`Write::transmission`](super::Write::transmission) for why an invented
    /// one would be worse.
    #[must_use]
    pub fn transmission(
        mut self,
        issue_number: impl Into<String>,
        date: impl Into<String>,
        producer: impl Into<String>,
        recipient: impl Into<String>,
        status: impl Into<String>,
    ) -> Build {
        self.tran = Some(laterite_ags4_emit::TranStamp::new(
            issue_number,
            date,
            producer,
            recipient,
            status,
        ));
        self
    }

    /// Do it.
    pub fn run(self) -> Result<Written, Error> {
        let groups = self
            .groups
            .into_iter()
            .map(GroupData::into_engine)
            .collect::<Result<Vec<_>, _>>()?;
        emit_groups(
            &groups,
            self.mode,
            self.edition.as_deref(),
            self.synthesise_metadata,
            self.tran,
        )
    }
}

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never cell values — see the note in `document.rs`.

impl std::fmt::Debug for Build {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Build")
            .field(
                "groups",
                &self
                    .groups
                    .iter()
                    .map(|g| format!("{} x{}", g.code, g.rows.len()))
                    .collect::<Vec<_>>(),
            )
            .field("mode", &self.mode)
            .field("edition", &self.edition)
            .field("synthesise_metadata", &self.synthesise_metadata)
            .field("transmission", &self.tran.is_some())
            .finish()
    }
}
