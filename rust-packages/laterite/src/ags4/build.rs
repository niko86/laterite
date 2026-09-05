//! Constructing AGS4 from data you hold, rather than from a file you read.

use std::path::{Path, PathBuf};

use laterite_ags4_emit::GroupInput;

use super::{Document, Finding, WriteMode, Written, emit_groups};
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
    build(document_groups(doc))
}

/// A document's groups as build input — shared by the two handle doors
/// ([`build_document`] and [`build_unchecked_document`]) so they cannot come
/// to read the same document differently. Cells carry across as text, because
/// that is what a document holds: they were already formatted when the file
/// was written.
fn document_groups(doc: &Document) -> Vec<GroupData> {
    doc.groups()
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
        .collect()
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

    /// Do it and write the judged document to `path`.
    ///
    /// The verdict comes first: under [`WriteMode::Strict`] a refusal returns
    /// the error with **nothing written**. What does get written is staged to
    /// a temporary file in the destination's own directory and renamed into
    /// place — atomic on one filesystem — so `path` never holds a partial or
    /// unjudged document.
    ///
    /// The result carries the verdict and the path, deliberately **not** the
    /// bytes: this door exists for the caller who wants the document on disk
    /// rather than resident, and a result quietly holding it anyway would
    /// defeat that. Want both? [`Build::run`] then [`Written::bytes`].
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<BuildSaved, Error> {
        let dest = path.as_ref();
        let written = self.run()?;
        staged_write(dest, written.bytes())?;
        Ok(BuildSaved {
            path: dest.to_path_buf(),
            findings: written.findings,
            fixes_applied: written.fixes_applied,
        })
    }
}

/// What [`Build::to_path`] produced: the verdict on a document already on disk.
///
/// The to-disk twin of [`Written`], minus the bytes — see
/// [`Build::to_path`] for why they are withheld.
pub struct BuildSaved {
    path: PathBuf,
    findings: Vec<Finding>,
    fixes_applied: usize,
}

impl BuildSaved {
    /// Where the judged AGS4 document was written.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Anything still wrong with the written document after the chosen
    /// [`WriteMode`] ran — see [`Written::findings`].
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// How many mechanical fixes were applied on the way out.
    #[must_use]
    pub fn fixes_applied(&self) -> usize {
        self.fixes_applied
    }
}

/// Write `bytes` to `dest` via a temporary file in the destination's own
/// directory + rename — atomic on one filesystem, so `dest` never holds a
/// partial write. Shared by the build doors' file forms; what each door lets
/// REACH this write is the doors' difference, not the write's. (The same
/// contract as the Python surface's `_staged_write`; `std::fs::rename`
/// replaces an existing file on Unix and Windows alike.)
///
/// A DELIBERATE copy of `laterite_ags4_hostopts::staged_write` (#923, extracted
/// to its own crate by #947), not an oversight — a wait-state copy, inventoried
/// with the why in [`crate::pending_adoptions`] (#930). Adopt only per that
/// ledger's rule.
fn staged_write(dest: &Path, bytes: &[u8]) -> Result<(), Error> {
    let io_err = |e: std::io::Error| {
        Error::with_source(ErrorKind::Io, format!("cannot write {}", dest.display()), e)
    };
    let dir = staging_dir(dest);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let tmp = dir.join(format!(
        ".laterite-build-{}-{nanos}.tmp",
        std::process::id()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
        .and_then(|()| std::fs::rename(&tmp, dest))
        .map_err(|e| {
            // Best-effort cleanup: the temp file is ours alone (create_new),
            // and leaving it behind litters the caller's output directory.
            let _ = std::fs::remove_file(&tmp);
            io_err(e)
        })
}

/// The directory the staging file goes in: the DESTINATION's own, never the
/// system temp dir — rename is only atomic within one filesystem, and that
/// atomicity is the door's whole promise. A bare filename has no parent (or an
/// empty one), and both mean the current directory.
///
/// Its own function because the property is invisible to the integration
/// tests: on one filesystem a mis-chosen directory still passes every
/// end-to-end assertion, so the choice is pinned here, where a unit test can
/// see it.
fn staging_dir(dest: &Path) -> &Path {
    match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

// --- build_unchecked ----------------------------------------------------

/// A pending unchecked build. Configure it, then [`BuildUnchecked::run`] or
/// [`BuildUnchecked::to_path`].
pub struct BuildUnchecked {
    groups: Vec<GroupData>,
    edition: Option<String>,
}

/// [`build`] without the verdict — you are choosing to ship unchecked bytes.
///
/// Builds exactly what `build(groups).mode(WriteMode::Report)` with metadata
/// synthesis off builds — the same dictionary `UNIT`/`TYPE` fills, the same
/// cell formatting, the same section order, byte for byte (a test pins the
/// identity) — and skips the validation that follows. **Nothing here confirms
/// the output satisfies any AGS4 rule, and nothing downstream will**: no
/// findings, no fixes, no strict gate. The rule engine is most of what a
/// checked build spends its time on, so this door exists for the caller who
/// validates elsewhere or has decided not to — a pipeline's inner loop whose
/// output the pipeline's end validates once, a file bound for an external
/// checker.
///
/// The judge-coupled knobs are not defaulted, they are **absent from the
/// type**: no `mode` (there is no verdict for a mode to act on), no
/// `synthesise_metadata` / `transmission` (synthesis fills gaps a report
/// would have told you about; with no report, a silently minted catalogue is
/// a statement nobody checked). [`BuildUnchecked::edition`] and the
/// `units`/`types` on [`GroupData`] remain — they shape the data, not the
/// verdict.
///
/// [`BuildUnchecked::run`] returns plain bytes, deliberately not a
/// [`Written`]: its empty findings would read as "judged clean", and nothing
/// here judged anything.
#[must_use]
pub fn build_unchecked(groups: Vec<GroupData>) -> BuildUnchecked {
    BuildUnchecked {
        groups,
        edition: None,
    }
}

/// [`build_unchecked`] fed from a [`Document`] — the same handle door as
/// [`build_document`], minus the verdict. See [`build_unchecked`] for what
/// you are choosing.
#[must_use]
pub fn build_unchecked_document(doc: &Document) -> BuildUnchecked {
    build_unchecked(document_groups(doc))
}

impl BuildUnchecked {
    /// Build against a specific AGS4 edition. See [`editions`](super::editions).
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> BuildUnchecked {
        self.edition = Some(edition.into());
        self
    }

    /// Do it. The AGS4 bytes, with no verdict attached.
    pub fn run(self) -> Result<Vec<u8>, Error> {
        let edition = super::edition_or_fallback(self.edition.as_deref())?;
        let groups = self
            .groups
            .into_iter()
            .map(GroupData::into_engine)
            .collect::<Result<Vec<_>, _>>()?;
        laterite_ags4_emit::emit_ags4_unchecked(groups, edition)
            .map_err(|e| Error::with_source(ErrorKind::Emit, "cannot write as AGS4", e))
    }

    /// Do it and write the bytes to `path`, returning the path written.
    ///
    /// The same staged temp-file + rename as [`Build::to_path`], minus the
    /// verdict gate in front of it — there is no verdict to gate on.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let dest = path.as_ref().to_path_buf();
        let bytes = self.run()?;
        staged_write(&dest, &bytes)?;
        Ok(dest)
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

impl std::fmt::Debug for BuildUnchecked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildUnchecked")
            .field(
                "groups",
                &self
                    .groups
                    .iter()
                    .map(|g| format!("{} x{}", g.code, g.rows.len()))
                    .collect::<Vec<_>>(),
            )
            .field("edition", &self.edition)
            .finish()
    }
}

impl std::fmt::Debug for BuildSaved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildSaved")
            .field("path", &self.path)
            .field("findings", &self.findings.len())
            .field("fixes_applied", &self.fixes_applied)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::staging_dir;
    use std::path::Path;

    /// See `staging_dir` — the same-filesystem property cannot fail an
    /// integration test on a single-filesystem machine, so the choice itself
    /// is the thing asserted.
    #[test]
    fn the_staging_dir_is_the_destinations_own() {
        assert_eq!(
            staging_dir(Path::new("/a/b/out.ags")),
            Path::new("/a/b"),
            "staging anywhere else forfeits rename atomicity"
        );
        assert_eq!(staging_dir(Path::new("out.ags")), Path::new("."));
        assert_eq!(staging_dir(Path::new("./out.ags")), Path::new("."));
    }
}
