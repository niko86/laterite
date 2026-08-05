//! Comparing two revisions of an AGS4 file.

use std::path::{Path, PathBuf};

use laterite_ags4_reference::dict::Dictionary;

use super::{Document, resolve_edition, validator_kind};
use crate::{Error, ErrorKind};

/// What happened to one row.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Present only in the revision.
    Added,
    /// Present only in the baseline.
    Removed,
    /// Matched by KEY, and at least one cell genuinely differs.
    Changed,
}

/// One cell that differs between the two revisions.
///
/// `baseline` and `revision` are `None` when that side's row is shorter than the
/// heading list — a real state in a file with ragged rows, and distinct from a
/// cell that is present and empty.
#[derive(Debug, Clone)]
pub struct CellChange {
    heading: String,
    ags_type: String,
    baseline: Option<String>,
    revision: Option<String>,
}

impl CellChange {
    /// The heading whose value changed.
    #[must_use]
    pub fn heading(&self) -> &str {
        &self.heading
    }

    /// The heading's declared AGS TYPE, which is what the comparison was made
    /// through — see [`diff`].
    #[must_use]
    pub fn ags_type(&self) -> &str {
        &self.ags_type
    }

    /// The value in the baseline.
    #[must_use]
    pub fn baseline(&self) -> Option<&str> {
        self.baseline.as_deref()
    }

    /// The value in the revision.
    #[must_use]
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

/// One row's verdict.
#[derive(Debug, Clone)]
pub struct RowChange {
    change: Change,
    key: Vec<String>,
    line_baseline: Option<u32>,
    line_revision: Option<u32>,
    cells: Vec<CellChange>,
}

impl RowChange {
    /// Added, removed, or changed.
    #[must_use]
    pub fn change(&self) -> Change {
        self.change
    }

    /// The KEY values identifying this row — or the whole-row tuple when the
    /// group has no KEY headings. See [`GroupChange::keyed`].
    #[must_use]
    pub fn key(&self) -> &[String] {
        &self.key
    }

    /// The line this row sat on in the baseline, if it was there.
    #[must_use]
    pub fn line_baseline(&self) -> Option<u32> {
        self.line_baseline
    }

    /// The line this row sits on in the revision, if it is there.
    #[must_use]
    pub fn line_revision(&self) -> Option<u32> {
        self.line_revision
    }

    /// The cells that differ — populated for [`Change::Changed`] and empty
    /// otherwise, since a whole added or removed row has no per-cell verdict.
    #[must_use]
    pub fn cells(&self) -> &[CellChange] {
        &self.cells
    }
}

/// What changed in one group.
#[derive(Debug, Clone)]
pub struct GroupChange {
    code: String,
    added: usize,
    removed: usize,
    changed: usize,
    headings_added: Vec<String>,
    headings_removed: Vec<String>,
    keyed: bool,
    key_headings: Vec<String>,
    rows: Vec<RowChange>,
}

impl GroupChange {
    /// The group's four-letter code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// How many rows the revision adds. A **true total**, unaffected by
    /// [`Diff::max_rows_per_group`] — the cap limits the detail in
    /// [`GroupChange::rows`], never the counts.
    #[must_use]
    pub fn added(&self) -> usize {
        self.added
    }

    /// How many rows the revision drops — a true total, as above.
    #[must_use]
    pub fn removed(&self) -> usize {
        self.removed
    }

    /// How many matched rows differ — a true total, as above.
    #[must_use]
    pub fn changed(&self) -> usize {
        self.changed
    }

    /// Headings present only in the revision — a structural change, reported
    /// here rather than as a per-cell one.
    #[must_use]
    pub fn headings_added(&self) -> &[String] {
        &self.headings_added
    }

    /// Headings present only in the baseline.
    #[must_use]
    pub fn headings_removed(&self) -> &[String] {
        &self.headings_removed
    }

    /// Whether rows were matched by dictionary KEY headings.
    ///
    /// `false` means the group had no KEY headings present in both files — a
    /// custom or passthrough group — and rows were matched on the whole tuple
    /// instead, so an edited row shows up as a remove and an add rather than a
    /// change. Worth surfacing: it changes how the result reads.
    #[must_use]
    pub fn keyed(&self) -> bool {
        self.keyed
    }

    /// The KEY headings rows were matched on.
    #[must_use]
    pub fn key_headings(&self) -> &[String] {
        &self.key_headings
    }

    /// The per-row detail, capped by [`Diff::max_rows_per_group`].
    #[must_use]
    pub fn rows(&self) -> &[RowChange] {
        &self.rows
    }
}

/// What changed between two revisions.
#[derive(Debug, Clone)]
pub struct Delta {
    groups: Vec<GroupChange>,
    groups_added: Vec<String>,
    groups_removed: Vec<String>,
    added: usize,
    removed: usize,
    changed: usize,
}

impl Delta {
    /// The groups with at least one change, in the revision's file order and
    /// then the baseline-only ones.
    #[must_use]
    pub fn groups(&self) -> &[GroupChange] {
        &self.groups
    }

    /// Group codes the revision introduces.
    #[must_use]
    pub fn groups_added(&self) -> &[String] {
        &self.groups_added
    }

    /// Group codes the revision drops.
    #[must_use]
    pub fn groups_removed(&self) -> &[String] {
        &self.groups_removed
    }

    /// Rows added, summed over the groups **both revisions have**.
    ///
    /// A group the revision introduces contributes nothing here — it is
    /// reported whole, by [`Delta::groups_added`], and its rows are not counted
    /// as additions. That is the engine's contract and it is the right one: a
    /// new group is one fact, not one fact per row it happens to carry. Use
    /// [`Delta::is_unchanged`] rather than summing these three to ask whether
    /// anything changed at all.
    #[must_use]
    pub fn added(&self) -> usize {
        self.added
    }

    /// Rows removed, summed over the groups both revisions have — with the same
    /// caveat as [`Delta::added`] about a group dropped whole.
    #[must_use]
    pub fn removed(&self) -> usize {
        self.removed
    }

    /// Rows changed, summed over the groups both revisions have.
    #[must_use]
    pub fn changed(&self) -> usize {
        self.changed
    }

    /// Whether the two revisions are the same document, in AGS terms.
    ///
    /// "In AGS terms" is the whole point — see [`diff`]. Two files that differ
    /// byte-for-byte, in row order, or in how a number is spelled are identical
    /// here, and that is the answer worth having.
    #[must_use]
    pub fn is_unchanged(&self) -> bool {
        self.groups.is_empty() && self.groups_added.is_empty() && self.groups_removed.is_empty()
    }
}

/// One side of a comparison.
enum Side<'a> {
    Path(PathBuf),
    Bytes(Vec<u8>),
    /// A document, re-emitted at [`Diff::run`] rather than here.
    ///
    /// Deliberately the document's CURRENT content and not the bytes it was read
    /// from: a handle you have edited is what you meant to compare, and diffing
    /// the file on disk instead would silently ignore the edit. Python's handle
    /// door resolves to `Ags4File.bytes` — the re-emit — for the same reason.
    Document(&'a Document),
}

impl Side<'_> {
    fn bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            Side::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            }),
            Side::Bytes(b) => Ok(b.clone()),
            Side::Document(doc) => super::write(doc)
                .mode(super::WriteMode::Report)
                .to_bytes()
                .map(super::Written::into_bytes),
        }
    }

    fn describe(&self) -> String {
        match self {
            Side::Path(p) => format!("path {}", p.display()),
            Side::Bytes(b) => format!("{} bytes", b.len()),
            Side::Document(doc) => format!("document of {} groups", doc.len()),
        }
    }
}

/// A pending comparison. Configure it, then [`Diff::run`].
pub struct Diff<'a> {
    baseline: Side<'a>,
    revision: Side<'a>,
    edition: Option<String>,
    encoding: Option<String>,
    max_rows_per_group: Option<usize>,
}

fn pending<'a>(baseline: Side<'a>, revision: Side<'a>) -> Diff<'a> {
    Diff {
        baseline,
        revision,
        edition: None,
        encoding: None,
        max_rows_per_group: None,
    }
}

/// Compare two AGS4 files on disk — `baseline` against `revision`.
///
/// The comparison is in **AGS terms, not text terms**, which is what makes it
/// worth having over a line diff:
///
/// - **Rows are matched by their group's dictionary KEY headings**, not by line
///   order, so a file whose boreholes were re-sorted or re-numbered still pairs
///   each row with its own prior self.
/// - **Cells are compared through their declared TYPE**, so a formatting-only
///   change — `"1.0"` to `"1.00"`, an equivalent datetime spelling, trailing
///   whitespace — is not reported. Only a genuine change of value is.
///
/// A group with no KEY headings present in both files falls back to matching on
/// the whole row tuple, and says so via [`GroupChange::keyed`].
#[must_use]
pub fn diff(baseline: impl AsRef<Path>, revision: impl AsRef<Path>) -> Diff<'static> {
    pending(
        Side::Path(baseline.as_ref().to_path_buf()),
        Side::Path(revision.as_ref().to_path_buf()),
    )
}

/// Compare two AGS4 documents already in memory — see [`diff`].
#[must_use]
pub fn diff_bytes(baseline: impl Into<Vec<u8>>, revision: impl Into<Vec<u8>>) -> Diff<'static> {
    pending(Side::Bytes(baseline.into()), Side::Bytes(revision.into()))
}

/// Compare two read handles — see [`diff`].
///
/// Each document is compared as it stands *now*, including any edit made with
/// [`Document::set_cell`] or [`Document::push_row`], not as the file it came
/// from. Comparing the file on disk instead would quietly ignore the edit, which
/// is the one thing a caller holding an edited handle cannot want.
#[must_use]
pub fn diff_documents<'a>(baseline: &'a Document, revision: &'a Document) -> Diff<'a> {
    pending(Side::Document(baseline), Side::Document(revision))
}

impl<'a> Diff<'a> {
    /// Compare against a specific AGS4 edition instead of taking it from the
    /// revision's `TRAN_AGS`.
    ///
    /// The edition matters here because it supplies the KEY headings that decide
    /// which rows are the same row. See [`editions`](super::editions).
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Diff<'a> {
        self.edition = Some(edition.into());
        self
    }

    /// Decode both sides with this encoding instead of UTF-8 — a WHATWG label.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> Diff<'a> {
        self.encoding = Some(label.into());
        self
    }

    /// Cap how many per-row changes each group reports.
    ///
    /// The counts on [`GroupChange`] stay the true totals — this bounds the
    /// detail, not the answer, so a group with 40 000 changed rows can be
    /// summarised without materialising 40 000 records.
    #[must_use]
    pub fn max_rows_per_group(mut self, rows: usize) -> Diff<'a> {
        self.max_rows_per_group = Some(rows);
        self
    }

    /// Do it.
    pub fn run(self) -> Result<Delta, Error> {
        let a = self.baseline.bytes()?;
        let b = self.revision.bytes()?;

        let enc = laterite_ags4_parse::resolve_encoding(self.encoding.as_deref())
            .ok_or_else(|| super::bad_encoding(self.encoding.as_deref().unwrap_or_default()))?;
        let parse = |bytes: &[u8], which: &str| {
            laterite_ags4_parse::parse_bytes(bytes, enc)
                .map_err(laterite_ags4_validator::ValidatorError::from)
                .map_err(|e| {
                    Error::with_source(
                        validator_kind(e.kind()),
                        format!("cannot read the {which} as AGS4"),
                        e,
                    )
                })
        };
        let parsed_a = parse(&a, "baseline")?;
        let parsed_b = parse(&b, "revision")?;

        // The edition comes from the REVISION's TRAN_AGS, not the baseline's:
        // the KEY headings that decide which rows are the same row should be the
        // ones the newer file was written against. Every other surface resolves
        // it the same way, so one pair of files does not diff differently
        // depending on which binding asked.
        let forced = self.edition.as_deref().map(resolve_edition).transpose()?;
        let tran = laterite_ags4_validator::tran_ags_of(&parsed_b);
        let edition = laterite_ags4_validator::resolve_dict_version(forced, tran.as_deref())
            .map_or(laterite_ags4_reference::dict::FALLBACK, |(dv, _)| dv);
        let dict = Dictionary::bundled(edition);

        let delta =
            laterite_ags4_diff::diff_parsed(&parsed_a, &parsed_b, &dict, self.max_rows_per_group);

        Ok(Delta {
            groups: delta.groups.into_iter().map(convert_group).collect(),
            groups_added: delta.groups_added,
            groups_removed: delta.groups_removed,
            added: delta.total_added,
            removed: delta.total_removed,
            changed: delta.total_changed,
        })
    }
}

fn convert_group(g: laterite_ags4_diff::GroupDelta) -> GroupChange {
    GroupChange {
        code: g.code,
        added: g.added,
        removed: g.removed,
        changed: g.changed,
        headings_added: g.headings_added,
        headings_removed: g.headings_removed,
        keyed: g.keyed,
        key_headings: g.key_headings,
        rows: g
            .rows
            .into_iter()
            .map(|r| RowChange {
                // The engine spells the verdict as a `&'static str` because its
                // whole job is to serialise. Anything it does not name is
                // `Changed`, which is the conservative reading: a row the engine
                // reported at all is a row that differs.
                change: match r.kind {
                    "added" => Change::Added,
                    "removed" => Change::Removed,
                    _ => Change::Changed,
                },
                key: r.key,
                line_baseline: r.line_a,
                line_revision: r.line_b,
                cells: r
                    .cells
                    .into_iter()
                    .map(|c| CellChange {
                        heading: c.heading,
                        ags_type: c.ags_type,
                        baseline: c.a,
                        revision: c.b,
                    })
                    .collect(),
            })
            .collect(),
    }
}

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never cell values — see the note in `document.rs`.

impl std::fmt::Debug for Diff<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diff")
            .field("baseline", &self.baseline.describe())
            .field("revision", &self.revision.describe())
            .field("edition", &self.edition)
            .field("encoding", &self.encoding)
            .field("max_rows_per_group", &self.max_rows_per_group)
            .finish()
    }
}
