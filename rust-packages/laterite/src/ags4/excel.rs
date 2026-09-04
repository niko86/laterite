//! AGS4 ↔ XLSX conversion — the `excel` feature's two doors.
//!
//! Same layout as every other surface produces: one worksheet per AGS4 group,
//! the `HEADING` column first, the `UNIT` / `TYPE` / `DATA` pseudo-rows
//! preserved as rows. The conversion engine underneath is shared with the
//! Python, Node, browser and `lat` surfaces, so a workbook made here is the
//! workbook they make.

use std::path::{Path, PathBuf};

use laterite_ags4_core::ags4_codec::ReadOptions;
use laterite_ags4_excel::{ExcelStats, ags4_bytes_to_xlsx_with, xlsx_bytes_to_ags4};

use super::bad_encoding;
use crate::{Error, ErrorKind};

/// Where a conversion's input comes from.
///
/// Two variants, not the module's three-variant `Source`: there is no text door
/// on either direction (an XLSX is binary, and the sibling surfaces' floor for
/// the AGS4 side is path + bytes), so carrying an unreachable `Text` arm here
/// would be a match nothing can ever exercise.
enum ExcelInput {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

impl ExcelInput {
    /// Shape and size for `Debug`, never contents — same contract as
    /// `Source::describe`.
    fn describe(&self) -> String {
        match self {
            ExcelInput::Path(p) => format!("path {}", p.display()),
            ExcelInput::Bytes(b) => format!("{} bytes", b.len()),
        }
    }

    /// The raw input bytes, reading the file if the input is a path.
    fn load(&self) -> Result<Vec<u8>, Error> {
        match self {
            ExcelInput::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            }),
            ExcelInput::Bytes(b) => Ok(b.clone()),
        }
    }
}

// --- AGS4 → XLSX --------------------------------------------------------

/// A pending AGS4 → XLSX conversion. Configure it, then [`ToExcel::run`] or
/// [`ToExcel::to_path`].
pub struct ToExcel {
    input: ExcelInput,
    encoding: Option<String>,
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
}

fn pending_to_excel(input: ExcelInput) -> ToExcel {
    ToExcel {
        input,
        encoding: None,
        recover_duplicate_headings: false,
        truncate_excess_fields: false,
    }
}

/// Convert an AGS4 file on disk to an XLSX workbook.
#[must_use]
pub fn to_excel(path: impl AsRef<Path>) -> ToExcel {
    pending_to_excel(ExcelInput::Path(path.as_ref().to_path_buf()))
}

/// Convert AGS4 bytes already in memory to an XLSX workbook.
#[must_use]
pub fn to_excel_bytes(bytes: impl Into<Vec<u8>>) -> ToExcel {
    pending_to_excel(ExcelInput::Bytes(bytes.into()))
}

impl ToExcel {
    /// Decode the AGS4 source with this encoding instead of UTF-8 — a WHATWG
    /// label, exactly as on [`read`](super::read). An XLSX export is a common
    /// way out of a legacy delivery, and legacy deliveries are frequently
    /// cp1252 for the `°` and `±` in description fields.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> ToExcel {
        self.encoding = Some(label.into());
        self
    }

    /// Recover from a duplicated heading instead of refusing the file — see
    /// [`Read::recover_duplicate_headings`](super::Read::recover_duplicate_headings)
    /// for why the default is the careful one. It matters most on this door:
    /// an XLSX export is the usual way someone tries to get data OUT of a file
    /// that will not validate.
    #[must_use]
    pub fn recover_duplicate_headings(mut self, yes: bool) -> ToExcel {
        self.recover_duplicate_headings = yes;
        self
    }

    /// Discard the extra fields on an over-long DATA row instead of refusing
    /// the file — see [`Read::truncate_excess_fields`](super::Read::truncate_excess_fields).
    #[must_use]
    pub fn truncate_excess_fields(mut self, yes: bool) -> ToExcel {
        self.truncate_excess_fields = yes;
        self
    }

    /// Do it, leaving the workbook in memory.
    pub fn run(self) -> Result<Workbook, Error> {
        let raw = self.input.load()?;
        // Decode here rather than pushing an encoding down the engine — the
        // same boundary choice as `Read::run`, for the same reason: the
        // conversion engine takes bytes it assumes are UTF-8, and transcoding
        // first keeps every encoding type out of our API.
        let bytes = match &self.encoding {
            None => raw,
            Some(label) => {
                let enc = laterite_ags4_parse::resolve_encoding(Some(label))
                    .ok_or_else(|| bad_encoding(label))?;
                enc.decode(&raw).0.into_owned().into_bytes()
            }
        };
        let opts =
            ReadOptions::from_flags(self.recover_duplicate_headings, self.truncate_excess_fields);
        // `None` sheet order: the workbook keeps the AGS4 source order, which
        // is what every sibling surface's default does.
        let (xlsx, stats) = ags4_bytes_to_xlsx_with(&bytes, None, opts).map_err(|e| {
            Error::with_source(
                ErrorKind::NotAgs4,
                format!("cannot convert {} to XLSX", self.input.describe()),
                e,
            )
        })?;
        Ok(Workbook {
            bytes: xlsx,
            stats: Stats::from_engine(stats),
        })
    }

    /// Do it and write the workbook to `path`.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<Workbook, Error> {
        let workbook = self.run()?;
        workbook.save(path)?;
        Ok(workbook)
    }
}

/// The XLSX workbook a conversion produced.
pub struct Workbook {
    bytes: Vec<u8>,
    stats: Stats,
}

impl Workbook {
    /// The workbook, as XLSX bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Take ownership of the workbook bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// How many worksheets were written — one per AGS4 group.
    #[must_use]
    pub fn sheets_written(&self) -> usize {
        self.stats.sheets
    }

    /// How many DATA rows were written across all worksheets.
    #[must_use]
    pub fn rows_written(&self) -> usize {
        self.stats.rows
    }

    /// What the conversion could not place, named — an empty sheet skipped, a
    /// column whose name is no AGS4 heading. Dropping silently is the failure
    /// mode this list exists to prevent.
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.stats.warnings
    }

    /// Write the workbook to `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        std::fs::write(path, &self.bytes).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })
    }
}

// --- XLSX → AGS4 --------------------------------------------------------

/// A pending XLSX → AGS4 conversion. Configure it, then [`FromExcel::run`] or
/// [`FromExcel::to_path`].
pub struct FromExcel {
    input: ExcelInput,
    format_numeric_columns: bool,
}

/// Convert an XLSX workbook on disk back to AGS4.
///
/// Each worksheet with a `HEADING` column becomes one group; what cannot be
/// placed — a sheet with no `HEADING`, a column named outside the AGS4 heading
/// grammar, a row that is none of `UNIT`/`TYPE`/`DATA` — is dropped and named
/// in [`Converted::warnings`].
#[must_use]
pub fn from_excel(path: impl AsRef<Path>) -> FromExcel {
    FromExcel {
        input: ExcelInput::Path(path.as_ref().to_path_buf()),
        format_numeric_columns: true,
    }
}

/// Convert XLSX bytes already in memory back to AGS4 — an upload needn't hit
/// disk first.
#[must_use]
pub fn from_excel_bytes(bytes: impl Into<Vec<u8>>) -> FromExcel {
    FromExcel {
        input: ExcelInput::Bytes(bytes.into()),
        format_numeric_columns: true,
    }
}

impl FromExcel {
    /// Re-render each DATA cell to its column's declared AGS TYPE (on by
    /// default, matching every sibling surface).
    ///
    /// A spreadsheet holds numbers as floats, so an edited `2DP` cell comes
    /// back as `523145.1` where AGS4 canonically writes `523145.10`. Leaving
    /// this on restores the declared formatting; turn it off only to keep the
    /// values exactly as the spreadsheet held them.
    #[must_use]
    pub fn format_numeric_columns(mut self, yes: bool) -> FromExcel {
        self.format_numeric_columns = yes;
        self
    }

    /// Do it, leaving the AGS4 in memory.
    pub fn run(self) -> Result<Converted, Error> {
        let raw = self.input.load()?;
        // `Other`, not a classified kind: the failures here are the workbook's
        // (not an XLSX at all, no convertible sheet), a domain this crate's
        // ErrorKind vocabulary — shared with the Python, Node and `lat`
        // surfaces — does not name. The engine's own message rides along.
        let (ags4, stats) = xlsx_bytes_to_ags4(&raw, self.format_numeric_columns).map_err(|e| {
            Error::with_source(
                ErrorKind::Other,
                format!("cannot convert {} to AGS4", self.input.describe()),
                e,
            )
        })?;
        Ok(Converted {
            // The emitter's output is UTF-8 by contract; surfaced as an error
            // rather than unwrapped for the same reason as `Fixed` — if that
            // ever stops holding it is an engine bug, and a panic in a library
            // is a poor way to report one.
            text: String::from_utf8(ags4).map_err(|e| {
                Error::with_source(
                    ErrorKind::Other,
                    "the conversion produced bytes that are not UTF-8",
                    e,
                )
            })?,
            stats: Stats::from_engine(stats),
        })
    }

    /// Do it and write the AGS4 to `path`.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<Converted, Error> {
        let converted = self.run()?;
        converted.save(path)?;
        Ok(converted)
    }
}

/// The AGS4 a workbook converted back to.
pub struct Converted {
    /// Held as `String` with bytes derived, the same way round as
    /// [`Fixed`](super::Fixed) — the conversion's output is UTF-8 by contract.
    text: String,
    stats: Stats,
}

impl Converted {
    /// The AGS4, always UTF-8 with no BOM.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Take ownership of the AGS4 bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.text.into_bytes()
    }

    /// The AGS4 as text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Take ownership of the AGS4 text.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }

    /// How many worksheets converted — one per AGS4 group.
    #[must_use]
    pub fn sheets_written(&self) -> usize {
        self.stats.sheets
    }

    /// How many DATA rows the AGS4 holds.
    #[must_use]
    pub fn rows_written(&self) -> usize {
        self.stats.rows
    }

    /// What the conversion could not place, named — see [`from_excel`].
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.stats.warnings
    }

    /// Write the AGS4 to `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        std::fs::write(path, self.bytes()).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })
    }
}

/// The engine's counters, in our own shape so [`ExcelStats`] — an engine type
/// that reshapes with the engine — never reaches a public signature.
struct Stats {
    sheets: usize,
    rows: usize,
    warnings: Vec<String>,
}

impl Stats {
    fn from_engine(stats: ExcelStats) -> Stats {
        Stats {
            sheets: stats.sheets_written,
            rows: stats.rows_written,
            warnings: stats.warnings,
        }
    }
}

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never file contents — see the note in `document.rs`.

impl std::fmt::Debug for ToExcel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToExcel")
            .field("input", &self.input.describe())
            .field("encoding", &self.encoding)
            .field(
                "recover_duplicate_headings",
                &self.recover_duplicate_headings,
            )
            .field("truncate_excess_fields", &self.truncate_excess_fields)
            .finish()
    }
}

impl std::fmt::Debug for Workbook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workbook")
            .field("bytes", &self.bytes.len())
            .field("sheets", &self.stats.sheets)
            .field("rows", &self.stats.rows)
            .field("warnings", &self.stats.warnings.len())
            .finish()
    }
}

impl std::fmt::Debug for FromExcel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromExcel")
            .field("input", &self.input.describe())
            .field("format_numeric_columns", &self.format_numeric_columns)
            .finish()
    }
}

impl std::fmt::Debug for Converted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Converted")
            .field("bytes", &self.text.len())
            .field("sheets", &self.stats.sheets)
            .field("rows", &self.stats.rows)
            .field("warnings", &self.stats.warnings.len())
            .finish()
    }
}
