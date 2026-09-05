//! The AGS4 surface: read, validate, fix, build, write, diff, merge — and,
//! behind the `excel` feature, XLSX conversion in both directions.
//!
//! Everything format-specific lives under this module and not at the crate
//! root, so a future format is a sibling rather than a rename.

mod build;
mod cert;
mod diff;
mod document;
// Feature-gated at the MODULE, not inside it, so a default build never
// compiles the XLSX machinery — the weight the crate split keeps off
// consumers who never touch Excel (dec-facade-parity decision 4).
#[cfg(feature = "excel")]
mod excel;
mod fix;
mod merge;
mod report;

use std::path::{Path, PathBuf};

use laterite_ags4_core::ags4_codec::{
    DuplicateHeadings, ExcessFields, ReadOptions, read_ags4_bytes_with,
};
use laterite_ags4_emit::{EmitMode, EmitOpts, GroupInput, TranStamp, emit_ags4};
use laterite_ags4_reference::dict::DictVersion;
use laterite_ags4_validator::parse::parse_bytes;
use laterite_ags4_validator::{CheckOptions, WorldScope, check_file, check_parsed_with_dict};

pub use build::{
    Build, BuildSaved, BuildUnchecked, Cell, GroupData, build, build_document, build_unchecked,
    build_unchecked_document,
};
pub use cert::Certify;
pub use diff::{
    CellChange, Change, Delta, Diff, GroupChange, RowChange, diff, diff_bytes, diff_documents,
};
pub use document::{Document, Group, Row, Rows};
#[cfg(feature = "excel")]
pub use excel::{
    Converted, FromExcel, ToExcel, Workbook, from_excel, from_excel_bytes, to_excel, to_excel_bytes,
};
pub use fix::{Fix, Fixed, Repair, fix, fix_bytes, fix_str, fixable_rules};
pub use merge::{
    Merge, Merged, MissingTran, Note, Revision, TypeClash, merge, merge_bytes, merge_documents,
};
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

/// Map an engine error onto one of ours, from the engine's own kind token.
///
/// The token, not the variants. `ValidatorError::kind()` is documented as the
/// single producer of that domain precisely so surfaces stop re-deriving it and
/// drifting; a variant match here would be a second, competing table. One
/// function rather than one match per call site for the same reason — four
/// copies is four chances for a new token to be handled three ways.
fn validator_kind(token: &str) -> ErrorKind {
    match token {
        "io" | "not_found" => ErrorKind::Io,
        "not_ags4" => ErrorKind::NotAgs4,
        "bad_dict" | "unsupported_edition" => ErrorKind::BadDictionary,
        "world_check_requires_source" => ErrorKind::InvalidArgument,
        _ => ErrorKind::Other,
    }
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
    /// AGS4 the caller has already decoded.
    ///
    /// A distinct variant rather than `Bytes(s.into_bytes())` so that
    /// [`Read::encoding`] cannot reach it. Text is decoded by definition, and
    /// transcoding it again would corrupt exactly the non-ASCII cells — the `°`
    /// and `±` in description fields — that made someone reach for `encoding`
    /// in the first place. Structural, not a doc note.
    Text(String),
}

impl Source {
    /// How a source is named in a `Debug` rendering — its shape and size, never
    /// its contents. Factored out because three builders render it identically
    /// and a fourth spelling would be the one that drifts.
    fn describe(&self) -> String {
        match self {
            Source::Path(p) => format!("path {}", p.display()),
            Source::Bytes(b) => format!("{} bytes", b.len()),
            Source::Text(s) => format!("{} characters", s.chars().count()),
        }
    }
}

/// A pending read. Configure it, then [`Read::run`].
pub struct Read {
    source: Source,
    encoding: Option<String>,
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
    cert: Option<CertInput>,
    only: Vec<String>,
}

/// Read an AGS4 file from disk.
pub fn read(path: impl AsRef<Path>) -> Read {
    Read {
        source: Source::Path(path.as_ref().to_path_buf()),
        encoding: None,
        recover_duplicate_headings: false,
        truncate_excess_fields: false,
        cert: None,
        only: Vec::new(),
    }
}

/// Read AGS4 from bytes already in memory — an upload, a database blob.
pub fn read_bytes(bytes: impl Into<Vec<u8>>) -> Read {
    Read {
        source: Source::Bytes(bytes.into()),
        encoding: None,
        recover_duplicate_headings: false,
        truncate_excess_fields: false,
        cert: None,
        only: Vec::new(),
    }
}

/// Read AGS4 from text already decoded — a string literal, a template, a column
/// out of a database driver that hands back `String`.
///
/// `read_bytes(s.as_bytes())` reaches the same place, and that is the point: it
/// is the workaround, not the door. Python and Node both offer this form, and a
/// caller who has a `String` should not have to know that the engine wants
/// bytes.
///
/// [`Read::encoding`] does not apply here and cannot — the text is decoded
/// already. That matches the Python surface, whose `encoding` is documented as
/// governing bytes and path input only.
pub fn read_str(text: impl Into<String>) -> Read {
    Read {
        source: Source::Text(text.into()),
        encoding: None,
        recover_duplicate_headings: false,
        truncate_excess_fields: false,
        cert: None,
        only: Vec::new(),
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

    /// Discard the extra fields on a DATA row that split into more of them than
    /// its group declares headings, instead of refusing the file.
    ///
    /// Off by default, and for the same reason as
    /// [`Read::recover_duplicate_headings`] — except that here nothing can be
    /// renamed to rescue the value, because the extra field belongs to no
    /// heading at all. The usual cause is a value containing a comma whose
    /// quotes were lost (AGS4 Rule 5), and no reader can say which side of the
    /// comma the heading wanted. Turning this on shortens such a row silently,
    /// which is what every read did before #776. Use it to salvage a file you
    /// cannot repair at source; never to round-trip or certify one.
    #[must_use]
    pub fn truncate_excess_fields(mut self, yes: bool) -> Read {
        self.truncate_excess_fields = yes;
        self
    }

    /// Read only these groups.
    ///
    /// On its own this is a filter — the file is parsed and the rest discarded.
    /// Combined with [`Read::index`] it becomes a *slice*: the named sections are
    /// parsed straight out of their byte ranges and the rest of the file is never
    /// looked at. [`Document::sliced`] reports which of the two happened.
    #[must_use]
    pub fn only<I, S>(mut self, codes: I) -> Read
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.only = codes.into_iter().map(Into::into).collect();
        self
    }

    /// Offer a certificate, so [`Read::only`] can slice instead of parsing.
    ///
    /// A certificate carries a byte index — group code to byte range — which is
    /// what turns "read one group out of a large delivery" into a read of that
    /// group's bytes rather than of the file.
    ///
    /// Purely an optimisation, and it declines itself whenever it cannot be sure:
    /// a certificate that does not match these bytes, a group the index places in
    /// more than one section, or an `encoding` override (which means the bytes
    /// being parsed are not the bytes the index was built over) all fall back to
    /// the whole-file parse. The document is the same either way.
    #[must_use]
    pub fn index(mut self, path: impl AsRef<Path>) -> Read {
        self.cert = Some(CertInput::Path(path.as_ref().to_path_buf()));
        self
    }

    /// Offer certificate bytes already in memory — see [`Read::index`].
    #[must_use]
    pub fn index_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Read {
        self.cert = Some(CertInput::Bytes(bytes.into()));
        self
    }

    /// Do it.
    pub fn run(self) -> Result<Document, Error> {
        let raw = match &self.source {
            Source::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            })?,
            Source::Bytes(b) => b.clone(),
            // Already decoded — hand the engine its UTF-8 and skip the transcode
            // below entirely. `encoding` is unreachable on this variant.
            Source::Text(s) => s.clone().into_bytes(),
        };

        // Decode here rather than pushing an encoding down the engine: the core
        // reader's entry point takes bytes it assumes are UTF-8, so transcoding
        // first is both correct and keeps `encoding_rs` out of our API.
        //
        // Matched on the SOURCE as well as the label. Text is decoded already,
        // so transcoding it is not a no-op — it re-reads UTF-8 as cp1252 and
        // turns `°` into `Â°`, corrupting precisely the cells the option exists
        // to rescue. Keying only on `encoding` looked right and did exactly
        // that; `encoding_cannot_corrupt_text` is the test that says so.
        let raw_for_cert = raw.clone();
        let bytes = match (&self.source, &self.encoding) {
            (Source::Text(_), _) | (_, None) => raw,
            (_, Some(label)) => {
                let enc = laterite_ags4_parse::resolve_encoding(Some(label))
                    .ok_or_else(|| bad_encoding(label))?;
                enc.decode(&raw).0.into_owned().into_bytes()
            }
        };

        let opts = read_options(self.recover_duplicate_headings, self.truncate_excess_fields);
        // The sliced path, when a certificate can vouch for the byte index AND
        // the caller asked for specific groups. Every guard below is a reason to
        // fall back rather than to fail: the whole-file parse is always correct,
        // so an index that cannot be trusted costs time, never accuracy.
        //
        // Nested rather than a `let ... && ...` chain: let chains are stable in
        // 1.88 and every crate here declares `rust-version = "1.85"`. That is a
        // promise to a stranger's toolchain, and the publish gate builds on 1.85
        // to keep it honest.
        //
        // The index's offsets are into the ORIGINAL bytes. If an `encoding`
        // override transcoded them, the bytes being parsed are not the bytes the
        // index describes, and slicing would read from the wrong offsets — hence
        // the `raw_for_cert == bytes` guard.
        let can_slice = self.cert.is_some() && !self.only.is_empty() && raw_for_cert == bytes;
        if can_slice {
            let input = self
                .cert
                .as_ref()
                .expect("can_slice is false when there is no certificate");
            let cert_bytes = match input {
                CertInput::Path(p) => std::fs::read(p).map_err(|e| {
                    Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
                })?,
                CertInput::Bytes(b) => b.clone(),
            };
            let sidecar = cert::parse_cert(&cert_bytes)?;
            let index = sidecar.index();

            // `range` is None for a group the index places in more than one
            // section. That is the truncation guard: slicing the first section of
            // a redeclared group returns a strict SUBSET of its rows, silently.
            let ranges: Option<Vec<_>> = self
                .only
                .iter()
                .map(|code| index.range(code).map(|r| (code.clone(), r)))
                .collect();

            // Against the ORIGINAL bytes, which is what the certificate hashed.
            // Checking the transcoded buffer instead happens to work while the
            // guard above holds them equal — and hides the fact that the guard
            // is what makes slicing safe. Mutation testing found exactly that.
            if let (true, Some(ranges)) = (sidecar.is_fresh_for(&raw_for_cert), ranges) {
                let mut groups = Vec::with_capacity(ranges.len());
                for (code, range) in ranges {
                    let group = laterite_ags4_core::index::parse_group_slice_with(
                        &bytes, range, &code, opts,
                    )
                    .map_err(|e| {
                        Error::with_source(
                            ErrorKind::NotAgs4,
                            format!("cannot read group `{code}` from its byte range"),
                            e,
                        )
                    })?;
                    groups.push(group);
                }
                let parsed = laterite_ags4_core::ags4_codec::ParsedAgs4::from_groups(groups);
                let mut doc = Document::new(parsed, raw_for_cert, self.encoding.clone());
                doc.sliced = true;
                return Ok(doc);
            }
        }

        read_ags4_bytes_with(&bytes, opts)
            // `raw`, not `bytes`: a certificate minted from this handle must hash
            // what the file actually holds, not what we decoded it into. See
            // `Document::source_bytes`.
            .map(|parsed| Document::new(parsed, raw_for_cert, self.encoding.clone()))
            .map(|mut doc| {
                // `only` without a usable index is still a filter — the caller
                // asked for these groups and gets these groups, just not cheaply.
                if !self.only.is_empty() {
                    doc.retain_only(&self.only);
                }
                doc
            })
            .map_err(|e| Error::with_source(ErrorKind::NotAgs4, "cannot read as AGS4", e))
    }
}

// --- validate -----------------------------------------------------------

/// Where an offered certificate comes from. Held unparsed so the engine's
/// `Sidecar` stays behind the boundary until [`Validate::run`] needs it.
enum CertInput {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// A pending validation. Configure it, then [`Validate::run`].
pub struct Validate {
    cert: Option<CertInput>,
    source: Source,
    warnings: bool,
    fyi: bool,
    edition: Option<String>,
    encoding: Option<String>,
    check_files: bool,
}

/// Validate an AGS4 file on disk against the numbered rules.
///
/// This is the only form that can answer Rule 20's on-disk half — see
/// [`Validate::check_files`]. For AGS4 that never touches a filesystem, use
/// [`validate_bytes`].
pub fn validate(path: impl AsRef<Path>) -> Validate {
    Validate {
        cert: None,
        source: Source::Path(path.as_ref().to_path_buf()),
        warnings: false,
        fyi: false,
        edition: None,
        encoding: None,
        check_files: false,
    }
}

/// Validate AGS4 already in memory — an upload, a queue message, a database
/// blob, a document you just wrote with [`write()`].
///
/// **Every rule runs except Rule 20's on-disk half**, which asks whether the
/// sibling `FILE/` tree really holds the attachments the file references. Bytes
/// have no sibling anything, so that half is not run — and asking for it anyway
/// via [`Validate::check_files`] is an error rather than a clean result. A
/// service that validates uploads without exposing a filesystem is the case this
/// exists for; it is also what the browser build has always done.
///
/// Everything else is identical to [`validate`], deliberately: both go through
/// the engine's single door, so the edition resolved from `TRAN_AGS` — and the
/// 4.0.3→4.0.4 content guard that goes with it — cannot come out differently for
/// the same file read two ways.
pub fn validate_bytes(bytes: impl Into<Vec<u8>>) -> Validate {
    Validate {
        cert: None,
        source: Source::Bytes(bytes.into()),
        warnings: false,
        fyi: false,
        edition: None,
        encoding: None,
        check_files: false,
    }
}

/// Validate AGS4 from text already decoded.
///
/// Identical to [`validate_bytes`] in every respect that reaches a finding —
/// same engine door, same edition resolution, same Rule 20 restriction — and
/// offered for the same reason as [`read_str`]: a caller holding a `String`
/// should not have to convert it to satisfy a signature.
///
/// [`Validate::encoding`] does not apply, as with [`read_str`].
pub fn validate_str(text: impl Into<String>) -> Validate {
    Validate {
        cert: None,
        source: Source::Text(text.into()),
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

    /// Offer a certificate, so a matching one skips the rule engine.
    ///
    /// Never auto-discovered: an `.ags.idx` sitting beside a file is not consent
    /// to trust it, so naming one here is how you assert that this certificate
    /// is for these bytes.
    ///
    /// A certificate that does not match cannot produce a wrong verdict — the
    /// engine simply runs, and [`Report::revalidate_reason`] says why it had to.
    /// The file is read by [`Validate::run`], so a missing one fails there.
    #[must_use]
    pub fn index(mut self, path: impl AsRef<Path>) -> Validate {
        self.cert = Some(CertInput::Path(path.as_ref().to_path_buf()));
        self
    }

    /// Offer certificate bytes already in memory — one minted by
    /// [`Certify::to_bytes`], or fetched from a store rather than a disk.
    #[must_use]
    pub fn index_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Validate {
        self.cert = Some(CertInput::Bytes(bytes.into()));
        self
    }

    /// Also run Rule 20's on-disk half: the `FILE/` tree beside the `.ags` must
    /// actually contain what the file references.
    ///
    /// Off by default, because it makes the answer depend on the directory
    /// rather than on the bytes — two runs over the same file can then disagree.
    ///
    /// Requires [`validate`]. Setting it on [`validate_bytes`] fails the run with
    /// [`ErrorKind::InvalidArgument`] instead of reporting Rule 20 clean, because
    /// "I looked and found nothing" and "I could not look" are different answers
    /// and only one of them is true.
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
        // Text is decoded already, so its bytes must be read back as UTF-8
        // whatever `encoding` says. `resolve_encoding(None)` is UTF-8 — the same
        // fact the comment above relies on.
        let utf8 =
            laterite_ags4_parse::resolve_encoding(None).ok_or_else(|| bad_encoding("utf-8"))?;

        // A certificate routes through the trust engine instead. That is not a
        // second implementation of the check: `trust::check` ends at the same
        // `check_parsed_with_dict` when the cert cannot answer, so the edition
        // guard below is resolved in one place either way.
        if let Some(input) = &self.cert {
            let raw = match input {
                CertInput::Path(p) => &std::fs::read(p).map_err(|e| {
                    Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
                })?,
                CertInput::Bytes(b) => b,
            };
            let sidecar = cert::parse_cert(raw)?;

            // The bytes the certificate is judged against, and the world it may
            // look at. Only a real path has a world — the same rule the engine
            // applies, restated here because the source is ours to classify.
            let (bytes, world) = match &self.source {
                Source::Path(p) => (
                    std::fs::read(p).map_err(|e| {
                        Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
                    })?,
                    WorldScope::OnDisk(p.clone()),
                ),
                Source::Bytes(b) => (b.clone(), WorldScope::None),
                Source::Text(s) => (s.clone().into_bytes(), WorldScope::None),
            };

            let outcome = laterite_ags4_trust::check(laterite_ags4_trust::Request {
                bytes: &bytes,
                opts: &opts,
                cert: Some(&sidecar),
                world,
                compat: None,
            })
            .map_err(|e| Error::with_source(validator_kind(e.kind()), "cannot validate", e))?;

            return Ok(Report {
                findings: convert(outcome.findings),
                certified: outcome.certified,
                revalidate_reason: outcome.revalidate_reason.map(|r| r.as_str().to_string()),
            });
        }

        // Both arms end at `check_parsed_with_dict` — `check_file` reaches it too.
        // That is not incidental: resolving `TRAN_AGS` and applying the 4.0.3→4.0.4
        // content guard is four steps, and the engine records that every surface
        // which hand-assembled them got the guard wrong, judging one file against
        // two dictionaries depending on whether it arrived as a path or as bytes.
        // Whatever this does, it must not become a fifth place that gets it wrong.
        let result = match &self.source {
            Source::Path(p) => check_file(p, &opts),
            Source::Bytes(b) => parse_bytes(b, opts.encoding).and_then(|parsed| {
                // `WorldScope::None` is the honest scope for bytes, and it is what
                // turns `check_files` into an error inside the engine rather than a
                // silent pass here.
                check_parsed_with_dict(&parsed, &opts, &WorldScope::None).map(|(f, _, _)| f)
            }),
            // UTF-8 explicitly, not `opts.encoding`: the text is decoded, so the
            // only faithful reading of its bytes is the one that round-trips them.
            Source::Text(s) => parse_bytes(s.as_bytes(), utf8).and_then(|parsed| {
                check_parsed_with_dict(&parsed, &opts, &WorldScope::None).map(|(f, _, _)| f)
            }),
        };

        let findings = result.map_err(|e| {
            let kind = validator_kind(e.kind());
            let subject = match &self.source {
                Source::Path(p) => format!("cannot validate {}", p.display()),
                Source::Bytes(b) => format!("cannot validate {} bytes", b.len()),
                Source::Text(s) => format!("cannot validate {} characters", s.chars().count()),
            };
            // One token gets its own sentence, because it is the one a caller
            // causes rather than receives: `check_files` on bytes. "cannot
            // validate 210 bytes" describes the size of the input and nothing
            // about the mistake, and it is the first thing a service validating
            // uploads runs into.
            //
            // No source attached in that arm, deliberately. The engine's own
            // wording says the same thing, so keeping it made `{:#}` — and every
            // `anyhow` chain — print the explanation twice in a row. A cause is
            // worth carrying when it adds what the message lacks; here it does
            // not. Every other arm keeps the terse subject and its real cause.
            if e.kind() == "world_check_requires_source" {
                Error::new(
                    kind,
                    format!(
                        "{subject}: the on-disk file check (Rule 20) needs a path to look \
                         beside, so `check_files` works with `validate` and not with \
                         `validate_bytes` — drop it, or validate the file from disk"
                    ),
                )
            } else {
                Error::with_source(kind, subject, e)
            }
        })?;
        Ok(Report {
            findings: convert(findings),
            certified: false,
            revalidate_reason: None,
        })
    }
}

impl Document {
    /// Mint this file's `.ags.idx` validity certificate.
    ///
    /// The certificate is over the bytes this document was READ from, before any
    /// transcode — see [`Certify`] for what one is and why it is never
    /// auto-discovered.
    ///
    /// Minting validates; it is not told a verdict. There is deliberately no
    /// parameter through which a caller can assert one, because that is exactly
    /// what earlier versions of this library got wrong — every certificate it
    /// produced recorded zero warnings because zero was the default argument.
    #[must_use]
    pub fn certify(&self) -> Certify<'_> {
        Certify {
            doc: self,
            edition: None,
        }
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
                                // `Text` goes out verbatim — a document's cells
                                // were already formatted when its file was
                                // written (#790: no more `Value` wrapper around
                                // an already-owned string).
                                .map(|h| {
                                    laterite_ags4_emit::Cell::Text(
                                        r.cell(h).unwrap_or("").to_string(),
                                    )
                                })
                                .collect()
                        })
                        .collect(),
                }
            })
            .collect();

        emit_groups(
            &groups,
            self.mode,
            self.edition.as_deref(),
            self.synthesise_metadata,
            self.tran,
        )
    }

    /// Produce the AGS4 bytes.
    pub fn to_bytes(self) -> Result<Written, Error> {
        self.emit()
    }

    /// Produce the AGS4 bytes and write them to `path`.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<Written, Error> {
        let path = path.as_ref();
        let written = self.emit()?;
        std::fs::write(path, written.bytes()).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })?;
        Ok(written)
    }
}

/// [`WriteMode`] → the engine's [`EmitMode`], one arm per public mode. ONE
/// copy on purpose (#939): each door owning its own three-arm match is how a
/// fourth mode reaches one door and not another. Private on purpose too —
/// the Angle C rule keeps the engine type off the public surface, so every
/// door owes this translation, and owing it is fine as long as it is owed
/// to one function.
pub(crate) fn emit_mode(mode: WriteMode) -> EmitMode {
    match mode {
        WriteMode::AutoFix => EmitMode::AutoFix,
        WriteMode::Report => EmitMode::Report,
        WriteMode::Strict => EmitMode::Strict,
    }
}

/// The read-tolerance pair → the engine's [`ReadOptions`], ONE copy (#939).
/// Also a wait-state single point: the engine grew
/// `ags4_codec::ReadOptions::from_flags` for exactly this translation, and
/// this body becomes a call to it once the registry carries it — see
/// [`crate::pending_adoptions`] (#930). Until then, one body to delete
/// instead of a copy per door.
pub(crate) fn read_options(
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
) -> ReadOptions {
    ReadOptions {
        duplicate_headings: if recover_duplicate_headings {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
        excess_fields: if truncate_excess_fields {
            ExcessFields::Truncate
        } else {
            ExcessFields::Error
        },
    }
}

/// The edition knob → a concrete [`DictVersion`]: a label resolves (or
/// refuses, naming the accepted set), absence means the dictionary's
/// generated fallback. ONE copy (#939) — a hand-written fallback literal in
/// a door is the exact class the editions sweep retired.
pub(crate) fn edition_or_fallback(
    edition: Option<&str>,
) -> Result<laterite_ags4_reference::dict::DictVersion, Error> {
    match edition {
        Some(label) => resolve_edition(label),
        None => Ok(laterite_ags4_reference::dict::FALLBACK),
    }
}

/// The one call into the emit engine, shared by [`write`] and
/// [`build`](build::build).
///
/// Both doors differ only in where their `GroupInput` came from — a document,
/// or the caller's own data. Everything after that (edition resolution, the
/// mode mapping, the UTF-8 contract on the way out) is the same work, and a
/// second copy of it is how two doors come to disagree about the same file.
fn emit_groups(
    groups: &[GroupInput],
    mode: WriteMode,
    edition: Option<&str>,
    synthesise_metadata: bool,
    tran: Option<TranStamp>,
) -> Result<Written, Error> {
    let opts = EmitOpts {
        mode: emit_mode(mode),
        edition: edition_or_fallback(edition)?,
        tran,
        synthesise_metadata,
    };
    let result = emit_ags4(groups, &opts)
        .map_err(|e| Error::with_source(ErrorKind::Emit, "cannot write as AGS4", e))?;
    Ok(Written {
        fixes_applied: result.fixes_applied,
        findings: convert(result.findings),
        // Cannot fail — see the note on `Written::text`. Surfaced as an error
        // rather than unwrapped anyway: if the emitter ever did produce
        // non-UTF-8 that is an engine bug, and a panic in a library is a poor
        // way to report one.
        text: String::from_utf8(result.bytes).map_err(|e| {
            Error::with_source(
                ErrorKind::Emit,
                "the emitter produced bytes that are not UTF-8",
                e,
            )
        })?,
    })
}

/// What a write produced.
pub struct Written {
    /// Held as `String`, with bytes derived — the same way round as the Python
    /// surface, whose `Ags4File.text` is primary and whose `.bytes` is
    /// `text.encode("utf-8")`.
    ///
    /// Sound because this is OUR emitter's own output, not an arbitrary file:
    /// every cell reaches the writer as a Rust `String`, so the result is UTF-8
    /// by construction. The "AGS4 is not guaranteed UTF-8" caution is real, and
    /// it is about READING files other people wrote — it does not reach here.
    text: String,
    findings: Vec<Finding>,
    fixes_applied: usize,
}

impl Written {
    /// The AGS4 bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Take ownership of the bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.text.into_bytes()
    }

    /// The AGS4 as text.
    ///
    /// Python and Node both return the produced AGS4 as a string as well as
    /// bytes; this closes the gap. Free — no copy, no re-decode — because the
    /// text is what is stored.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Take ownership of the text.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
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
            .field("source", &self.source.describe())
            .field("encoding", &self.encoding)
            .field(
                "recover_duplicate_headings",
                &self.recover_duplicate_headings,
            )
            .field("truncate_excess_fields", &self.truncate_excess_fields)
            .field("only", &self.only)
            .field(
                "index",
                &match &self.cert {
                    None => "none".to_string(),
                    Some(CertInput::Path(p)) => format!("path {}", p.display()),
                    Some(CertInput::Bytes(b)) => format!("{} bytes", b.len()),
                },
            )
            .finish()
    }
}

impl std::fmt::Debug for Validate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Validate")
            .field("source", &self.source.describe())
            .field("warnings", &self.warnings)
            .field("fyi", &self.fyi)
            .field("edition", &self.edition)
            .field("encoding", &self.encoding)
            .field(
                "index",
                &match &self.cert {
                    None => "none".to_string(),
                    Some(CertInput::Path(p)) => format!("path {}", p.display()),
                    Some(CertInput::Bytes(b)) => format!("{} bytes", b.len()),
                },
            )
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
        // `finish_non_exhaustive` because omitting the produced AGS4 is the
        // point, not an oversight — same rule as the other Debug impls here.
        // Reported as a byte length (not a char count) so it stays comparable
        // with what `bytes()` hands back and with what lands on disk.
        f.debug_struct("Written")
            .field("bytes", &self.text.len())
            .field("findings", &self.findings.len())
            .field("fixes_applied", &self.fixes_applied)
            .finish_non_exhaustive()
    }
}
