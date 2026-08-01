//! The AGS4 surface: read, validate, write.
//!
//! Everything format-specific lives under this module and not at the crate
//! root, so a future format is a sibling rather than a rename.

mod document;
mod report;

use std::path::{Path, PathBuf};

use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ReadOptions, read_ags4_bytes_with};
use laterite_ags4_emit::{EmitMode, EmitOpts, GroupInput, TranStamp, emit_ags4};
use laterite_ags4_reference::dict::DictVersion;
use laterite_ags4_validator::{CheckOptions, check_file};

pub use document::{Document, Group, Row, Rows};
pub use report::{Finding, Report, Severity};

use crate::{Error, ErrorKind};

/// The AGS4 editions this build understands, as edition strings (`"4.0.3"`,
/// `"4.1"`, …) suitable for [`Validate::edition`] and [`Write::edition`].
#[must_use]
pub fn editions() -> Vec<&'static str> {
    DictVersion::ALL.iter().map(|v| v.as_str()).collect()
}

fn resolve_edition(label: &str) -> Result<DictVersion, Error> {
    DictVersion::from_edition(label).ok_or_else(|| {
        Error::new(
            ErrorKind::BadDictionary,
            format!(
                "unknown AGS4 edition `{label}` — this build has {}",
                editions().join(", ")
            ),
        )
    })
}

/// Only the ERROR is factored out, not the lookup.
///
/// A helper returning the encoding would have to name `encoding_rs::Encoding` in
/// its signature, which would put `encoding_rs` in this crate's manifest.
/// Resolving inline lets inference carry the type, so the dependency stays
/// genuinely absent rather than merely absent from the public API.
fn bad_encoding(label: &str) -> Error {
    Error::new(
        ErrorKind::InvalidArgument,
        format!("unknown encoding label `{label}` (WHATWG names, e.g. `windows-1252`)"),
    )
}

/// Findings from the engine, flattened into our own shape.
///
/// The engine keys them by rule label in a `BTreeMap`, which is what keeps two
/// runs diffable; flattening in key order preserves that while keeping its map
/// type out of our signatures.
fn convert(findings: laterite_ags4_validator::Findings) -> Vec<Finding> {
    findings
        .into_iter()
        .flat_map(|(rule, group_findings)| {
            group_findings.into_iter().map(move |f| Finding {
                rule: rule.clone(),
                group: f.group,
                description: f.desc,
                line: f.line,
                // Exhaustive on purpose: the engine's `Severity` is not
                // `#[non_exhaustive]`, so a new level there SHOULD stop this
                // compiling until someone decides what it means here.
                severity: match f.severity {
                    laterite_ags4_validator::findings::Severity::Error => Severity::Error,
                    laterite_ags4_validator::findings::Severity::Warning => Severity::Warning,
                    laterite_ags4_validator::findings::Severity::Fyi => Severity::Fyi,
                },
            })
        })
        .collect()
}

// --- read ---------------------------------------------------------------

enum Source {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// A pending read. Configure it, then [`Read::run`].
pub struct Read {
    source: Source,
    encoding: Option<String>,
    recover_duplicate_headings: bool,
}

/// Read an AGS4 file from disk.
pub fn read(path: impl AsRef<Path>) -> Read {
    Read {
        source: Source::Path(path.as_ref().to_path_buf()),
        encoding: None,
        recover_duplicate_headings: false,
    }
}

/// Read AGS4 from bytes already in memory — an upload, a database blob.
pub fn read_bytes(bytes: impl Into<Vec<u8>>) -> Read {
    Read {
        source: Source::Bytes(bytes.into()),
        encoding: None,
        recover_duplicate_headings: false,
    }
}

impl Read {
    /// Decode with this encoding instead of UTF-8.
    ///
    /// A WHATWG label — `"windows-1252"`, `"latin1"`, `"utf-8"` — not an
    /// encoding object, so no encoding library's version can reach this
    /// signature. Legacy delivery files are frequently cp1252 because of `°`
    /// and `±` in description fields.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> Read {
        self.encoding = Some(label.into());
        self
    }

    /// Recover from a group that declares the same heading twice, instead of
    /// refusing the file.
    ///
    /// Off by default, and that default is the careful one. AGS4 forbids the
    /// duplicate, and rows are keyed by heading name — so read naively, the
    /// second column silently overwrites the first and you get a column that
    /// looks fully populated and is not. Turning this on renames the later
    /// occurrences so nothing is lost, at the cost of a document that is
    /// deliberately no longer valid AGS4. Use it to rescue data, not to
    /// round-trip.
    #[must_use]
    pub fn recover_duplicate_headings(mut self, yes: bool) -> Read {
        self.recover_duplicate_headings = yes;
        self
    }

    /// Do it.
    pub fn run(self) -> Result<Document, Error> {
        let raw = match &self.source {
            Source::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            })?,
            Source::Bytes(b) => b.clone(),
        };

        // Decode here rather than pushing an encoding down the engine: the core
        // reader's entry point takes bytes it assumes are UTF-8, so transcoding
        // first is both correct and keeps `encoding_rs` out of our API.
        let bytes = match &self.encoding {
            None => raw,
            Some(label) => {
                let enc = laterite_ags4_parse::resolve_encoding(Some(label))
                    .ok_or_else(|| bad_encoding(label))?;
                enc.decode(&raw).0.into_owned().into_bytes()
            }
        };

        let opts = ReadOptions {
            duplicate_headings: if self.recover_duplicate_headings {
                DuplicateHeadings::Recover
            } else {
                DuplicateHeadings::Error
            },
        };
        read_ags4_bytes_with(&bytes, opts)
            .map(Document::new)
            .map_err(|e| Error::with_source(ErrorKind::NotAgs4, "cannot read as AGS4", e))
    }
}

// --- validate -----------------------------------------------------------

/// A pending validation. Configure it, then [`Validate::run`].
pub struct Validate {
    path: PathBuf,
    warnings: bool,
    fyi: bool,
    edition: Option<String>,
    encoding: Option<String>,
    check_files: bool,
}

/// Validate an AGS4 file against the numbered rules.
///
/// Takes a path rather than bytes because one rule — Rule 20 — is about files
/// on disk beside the `.ags`, and a bytes API could only ever answer half of it.
/// A bytes form will be added when it can say honestly which half it ran.
pub fn validate(path: impl AsRef<Path>) -> Validate {
    Validate {
        path: path.as_ref().to_path_buf(),
        warnings: false,
        fyi: false,
        edition: None,
        encoding: None,
        check_files: false,
    }
}

impl Validate {
    /// Include WARNING-severity findings.
    #[must_use]
    pub fn warnings(mut self, yes: bool) -> Validate {
        self.warnings = yes;
        self
    }

    /// Include FYI-severity findings.
    #[must_use]
    pub fn fyi(mut self, yes: bool) -> Validate {
        self.fyi = yes;
        self
    }

    /// Force an AGS4 edition instead of auto-selecting from the file's
    /// `TRAN_AGS`. See [`editions`] for what this build carries.
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Validate {
        self.edition = Some(edition.into());
        self
    }

    /// Decode with this encoding instead of UTF-8 — a WHATWG label.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> Validate {
        self.encoding = Some(label.into());
        self
    }

    /// Also run Rule 20's on-disk half: the `FILE/` tree beside the `.ags` must
    /// actually contain what the file references.
    ///
    /// Off by default, because it makes the answer depend on the directory
    /// rather than on the bytes — two runs over the same file can then disagree.
    #[must_use]
    pub fn check_files(mut self, yes: bool) -> Validate {
        self.check_files = yes;
        self
    }

    /// Do it.
    pub fn run(self) -> Result<Report, Error> {
        let opts = CheckOptions {
            dict_version: self.edition.as_deref().map(resolve_edition).transpose()?,
            custom_dict: None,
            include_warnings: self.warnings,
            include_fyi: self.fyi,
            check_files: self.check_files,
            // `resolve_encoding(None)` is UTF-8, so one call covers both arms.
            encoding: laterite_ags4_parse::resolve_encoding(self.encoding.as_deref())
                .ok_or_else(|| bad_encoding(self.encoding.as_deref().unwrap_or_default()))?,
        };
        let findings = check_file(&self.path, &opts).map_err(|e| {
            // Map from the engine's own kind token, not from its variants. Its
            // `kind()` is documented as the single producer of that domain
            // precisely so surfaces stop re-deriving it and drifting; a variant
            // match here would be a second, competing table.
            let kind = match e.kind() {
                "io" | "not_found" => ErrorKind::Io,
                "not_ags4" => ErrorKind::NotAgs4,
                "bad_dict" | "unsupported_edition" => ErrorKind::BadDictionary,
                "world_check_requires_source" => ErrorKind::InvalidArgument,
                _ => ErrorKind::Other,
            };
            Error::with_source(kind, format!("cannot validate {}", self.path.display()), e)
        })?;
        Ok(Report {
            findings: convert(findings),
        })
    }
}

// --- write --------------------------------------------------------------

/// What to do when the data would produce invalid AGS4.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteMode {
    /// Emit, then apply the safe mechanical fixes. The default — it fits
    /// "just give me valid AGS4 from my data".
    #[default]
    AutoFix,
    /// Emit unchanged and hand back the findings for you to decide about.
    Report,
    /// Refuse, writing nothing, if any error-severity rule would be broken.
    Strict,
}

/// A pending write. Configure it, then [`Write::to_bytes`] or [`Write::to_path`].
pub struct Write<'a> {
    doc: &'a Document,
    mode: WriteMode,
    edition: Option<String>,
    synthesise_metadata: bool,
    tran: Option<TranStamp>,
}

/// Write a document back out as AGS4.
#[must_use]
pub fn write(doc: &Document) -> Write<'_> {
    Write {
        doc,
        mode: WriteMode::default(),
        edition: None,
        synthesise_metadata: true,
        tran: None,
    }
}

impl<'a> Write<'a> {
    /// What to do about validity — see [`WriteMode`].
    #[must_use]
    pub fn mode(mut self, mode: WriteMode) -> Write<'a> {
        self.mode = mode;
        self
    }

    /// Write against a specific AGS4 edition. See [`editions`].
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Write<'a> {
        self.edition = Some(edition.into());
        self
    }

    /// Derive the `UNIT` and `TYPE` catalogue groups from the data. On by
    /// default: AGS4 requires them to cover everything used elsewhere in the
    /// file, and deriving them is more reliable than assembling them by hand.
    #[must_use]
    pub fn synthesise_metadata(mut self, yes: bool) -> Write<'a> {
        self.synthesise_metadata = yes;
        self
    }

    /// State the transmission this file represents, producing a `TRAN` group.
    ///
    /// All five are required AGS4 headings, so all five are required arguments.
    /// **Without this call no `TRAN` group is written at all** and validation
    /// will say so — which is the honest outcome. A synthesised placeholder
    /// would be a claim about who transferred what, to whom, and when; that is
    /// not something a writer can invent on your behalf.
    ///
    /// `date` is an ISO `yyyy-mm-dd` string.
    #[must_use]
    pub fn transmission(
        mut self,
        issue_number: impl Into<String>,
        date: impl Into<String>,
        producer: impl Into<String>,
        recipient: impl Into<String>,
        status: impl Into<String>,
    ) -> Write<'a> {
        self.tran = Some(TranStamp::new(
            issue_number,
            date,
            producer,
            recipient,
            status,
        ));
        self
    }

    fn emit(self) -> Result<Written, Error> {
        let groups: Vec<GroupInput> = self
            .doc
            .groups()
            .iter()
            .map(|g| {
                let headings = g.headings();
                GroupInput {
                    code: g.code().to_string(),
                    headings: headings.iter().map(|h| (*h).to_string()).collect(),
                    units: Some(g.units().iter().map(|u| (*u).to_string()).collect()),
                    types: Some(g.types().iter().map(|t| (*t).to_string()).collect()),
                    rows: g
                        .rows()
                        .map(|r| {
                            headings
                                .iter()
                                .map(|h| serde_json::Value::from(r.cell(h).unwrap_or("")))
                                .collect()
                        })
                        .collect(),
                }
            })
            .collect();

        let edition = match &self.edition {
            Some(label) => resolve_edition(label)?,
            None => laterite_ags4_reference::dict::FALLBACK,
        };
        let opts = EmitOpts {
            mode: match self.mode {
                WriteMode::AutoFix => EmitMode::AutoFix,
                WriteMode::Report => EmitMode::Report,
                WriteMode::Strict => EmitMode::Strict,
            },
            edition,
            tran: self.tran,
            synthesise_metadata: self.synthesise_metadata,
        };
        let result = emit_ags4(&groups, &opts)
            .map_err(|e| Error::with_source(ErrorKind::Emit, "cannot write as AGS4", e))?;
        Ok(Written {
            fixes_applied: result.fixes_applied,
            findings: convert(result.findings),
            bytes: result.bytes,
        })
    }

    /// Produce the AGS4 bytes.
    pub fn to_bytes(self) -> Result<Written, Error> {
        self.emit()
    }

    /// Produce the AGS4 bytes and write them to `path`.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<Written, Error> {
        let path = path.as_ref();
        let written = self.emit()?;
        std::fs::write(path, &written.bytes).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })?;
        Ok(written)
    }
}

/// What a write produced.
pub struct Written {
    bytes: Vec<u8>,
    findings: Vec<Finding>,
    fixes_applied: usize,
}

impl Written {
    /// The AGS4 bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Take ownership of the bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Anything still wrong with the output after the chosen [`WriteMode`] ran.
    ///
    /// Non-empty is not necessarily a failure: under [`WriteMode::AutoFix`] the
    /// safe fixes have already been applied, and what remains is what could not
    /// be fixed mechanically.
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

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never file contents — see the note in `document.rs`.

impl std::fmt::Debug for Read {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Read")
            .field(
                "source",
                &match &self.source {
                    Source::Path(p) => format!("path {}", p.display()),
                    Source::Bytes(b) => format!("{} bytes", b.len()),
                },
            )
            .field("encoding", &self.encoding)
            .field(
                "recover_duplicate_headings",
                &self.recover_duplicate_headings,
            )
            .finish()
    }
}

impl std::fmt::Debug for Validate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Validate")
            .field("path", &self.path.display().to_string())
            .field("warnings", &self.warnings)
            .field("fyi", &self.fyi)
            .field("edition", &self.edition)
            .field("encoding", &self.encoding)
            .field("check_files", &self.check_files)
            .finish()
    }
}

impl std::fmt::Debug for Write<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Write")
            .field("mode", &self.mode)
            .field("edition", &self.edition)
            .field("synthesise_metadata", &self.synthesise_metadata)
            .field("transmission", &self.tran.is_some())
            .finish()
    }
}

impl std::fmt::Debug for Written {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Written")
            .field("bytes", &self.bytes.len())
            .field("findings", &self.findings.len())
            .field("fixes_applied", &self.fixes_applied)
            .finish()
    }
}
