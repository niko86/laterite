//! Mechanical repair of a delivered AGS4 file.

use std::path::Path;

use laterite_ags4_validator::{CheckOptions, fix_document_selective, fixes::FixRisk};

use super::{Finding, Source, bad_encoding, convert, resolve_edition, validator_kind};
use crate::{Error, ErrorKind};

/// The AGS Format Rule labels whose findings the fixer can repair — the
/// vocabulary [`Fix::only`] and [`Fix::exclude`] speak.
///
/// Short forms (`"8"`, `"11a"`), matching what the other surfaces take, and read
/// from the engine rather than restated here so a new fix cannot leave this
/// list behind.
#[must_use]
pub fn fixable_rules() -> Vec<&'static str> {
    laterite_ags4_validator::fixes::FIXABLE_RULE_LABELS.to_vec()
}

/// A pending repair. Configure it, then [`Fix::run`] or [`Fix::to_path`].
pub struct Fix {
    source: Source,
    edition: Option<String>,
    encoding: Option<String>,
    risky: bool,
    only: Option<Vec<String>>,
    exclude: Vec<String>,
}

fn pending(source: Source) -> Fix {
    Fix {
        source,
        edition: None,
        encoding: None,
        risky: false,
        only: None,
        exclude: Vec::new(),
    }
}

/// Repair an AGS4 file on disk.
///
/// Non-destructive: the repaired bytes come back on the [`Fixed`] result and
/// the source file is untouched unless you ask for it with [`Fix::to_path`].
#[must_use]
pub fn fix(path: impl AsRef<Path>) -> Fix {
    pending(Source::Path(path.as_ref().to_path_buf()))
}

/// Repair AGS4 already in memory.
#[must_use]
pub fn fix_bytes(bytes: impl Into<Vec<u8>>) -> Fix {
    pending(Source::Bytes(bytes.into()))
}

/// Repair AGS4 from text already decoded.
///
/// [`Fix::encoding`] does not apply, for the same reason it does not apply to
/// [`read_str`](super::read_str): the text is decoded, so transcoding it again
/// would corrupt exactly the non-ASCII cells the option exists to rescue.
#[must_use]
pub fn fix_str(text: impl Into<String>) -> Fix {
    pending(Source::Text(text.into()))
}

impl Fix {
    /// Validate the repaired bytes against a specific AGS4 edition instead of
    /// auto-selecting from the file's `TRAN_AGS`. See [`editions`](super::editions).
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Fix {
        self.edition = Some(edition.into());
        self
    }

    /// Decode the source with this encoding instead of UTF-8 — a WHATWG label.
    ///
    /// Note that the *output* is always UTF-8 with no BOM, so repairing a
    /// `windows-1252` file also normalises its encoding. That is not a side
    /// effect to work around: a fix run rewrites the document, and emitting it
    /// back into a legacy encoding would be a second chance to lose the `°` and
    /// `±` that made the file cp1252 in the first place.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> Fix {
        self.encoding = Some(label.into());
        self
    }

    /// Also apply the fixes that guess intent.
    ///
    /// The **safe** set — CRLF, BOM, embedded CR, short-row padding, numeric
    /// reformatting, the `TRAN` delimiter and concatenator rows — is always
    /// applied, because each is unambiguous from the file alone. The risky set
    /// (duplicate-heading rename, `dd/mm` datetime canonicalisation,
    /// smart-quote to ASCII) rewrites something whose intent the file does not
    /// settle, which is why it is yours to ask for.
    ///
    /// [`Fixed::risky_available`] says how many the safe run withheld, so you
    /// can offer the choice rather than make the caller guess it exists.
    #[must_use]
    pub fn risky(mut self, yes: bool) -> Fix {
        self.risky = yes;
        self
    }

    /// Apply *only* the fixes for these rule labels — see [`fixable_rules`].
    ///
    /// The risk gate still applies first, so a rule whose only fix is risky
    /// needs [`Fix::risky`] as well as naming it here.
    #[must_use]
    pub fn only<I, S>(mut self, rules: I) -> Fix
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.only = Some(rules.into_iter().map(Into::into).collect());
        self
    }

    /// Skip the fixes for these rule labels. Combines with [`Fix::only`] —
    /// `only` narrows the set, then this removes from what is left.
    #[must_use]
    pub fn exclude<I, S>(mut self, rules: I) -> Fix
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclude = rules.into_iter().map(Into::into).collect();
        self
    }

    /// Do it, leaving the result in memory.
    pub fn run(self) -> Result<Fixed, Error> {
        let raw = match &self.source {
            Source::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            })?,
            Source::Bytes(b) => b.clone(),
            Source::Text(s) => s.clone().into_bytes(),
        };

        // Text is decoded already, so it is read back as UTF-8 whatever
        // `encoding` says — `resolve_encoding(None)` is UTF-8.
        let label = match &self.source {
            Source::Text(_) => None,
            _ => self.encoding.as_deref(),
        };
        let opts = CheckOptions {
            dict_version: self.edition.as_deref().map(resolve_edition).transpose()?,
            custom_dict: None,
            // Errors AND warnings in the residual, unconditionally — not this
            // crate's errors-only `validate` default. A fix run's residual is an
            // account of what it could not put right, and an errors-only account
            // under-reports that. The other surfaces settled here for the same
            // reason, so one file's residual reads the same wherever it is fixed.
            include_warnings: true,
            include_fyi: false,
            check_files: false,
            encoding: laterite_ags4_parse::resolve_encoding(label)
                .ok_or_else(|| bad_encoding(label.unwrap_or_default()))?,
        };

        let outcome =
            fix_document_selective(&raw, &opts, self.risky, self.only.as_deref(), &self.exclude)
                .map_err(|e| {
                    let subject = match &self.source {
                        Source::Path(p) => format!("cannot fix {}", p.display()),
                        Source::Bytes(b) => format!("cannot fix {} bytes", b.len()),
                        Source::Text(s) => format!("cannot fix {} characters", s.chars().count()),
                    };
                    Error::with_source(validator_kind(e.kind()), subject, e)
                })?;

        Ok(Fixed {
            // The engine's contract is that `fixed` is always valid UTF-8 — a
            // non-UTF-8 source is transcoded even when nothing else changed.
            // Surfaced as an error rather than unwrapped anyway: if that ever
            // stops holding it is an engine bug, and a panic in a library is a
            // poor way to report one.
            text: String::from_utf8(outcome.fixed).map_err(|e| {
                Error::with_source(
                    ErrorKind::Other,
                    "the fixer produced bytes that are not UTF-8",
                    e,
                )
            })?,
            findings: convert(outcome.residual),
            applied: outcome.applied.iter().map(Repair::from_engine).collect(),
            edition: outcome.dict_version.as_str().to_string(),
            risky_available: outcome.risky_available,
        })
    }

    /// Do it and write the repaired bytes to `path`.
    ///
    /// Pass the source path to repair in place. The result comes back either
    /// way, so what was written is also what you can inspect.
    pub fn to_path(self, path: impl AsRef<Path>) -> Result<Fixed, Error> {
        let fixed = self.run()?;
        fixed.save(path)?;
        Ok(fixed)
    }
}

/// What a repair produced.
pub struct Fixed {
    /// Held as `String` with bytes derived, the same way round as
    /// [`Written`](super::Written) — see the note there. Sound for the same
    /// reason: the fixer's output is UTF-8 by contract.
    text: String,
    findings: Vec<Finding>,
    applied: Vec<Repair>,
    edition: String,
    risky_available: usize,
}

impl Fixed {
    /// The repaired AGS4, always UTF-8 with no BOM.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Take ownership of the repaired bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.text.into_bytes()
    }

    /// The repaired AGS4 as text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Take ownership of the repaired text.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }

    /// What is **still** wrong, after the fixer had its turn.
    ///
    /// The fixer re-validates its own output, so this is the complement of
    /// [`Fixed::applied`]: the issues that could not be mechanically resolved
    /// and still need a person. Empty means the file came out clean.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The ledger of repairs that were made.
    #[must_use]
    pub fn applied(&self) -> &[Repair] {
        &self.applied
    }

    /// How many repairs were made — `applied().len()`.
    #[must_use]
    pub fn fixes_applied(&self) -> usize {
        self.applied.len()
    }

    /// How many further repairs [`Fix::risky`] would have applied, and `0` when
    /// it was already on.
    ///
    /// A discoverability signal rather than a count of problems: non-zero means
    /// more of this file is mechanically repairable, without the caller having
    /// to know an opt-in tier exists.
    #[must_use]
    pub fn risky_available(&self) -> usize {
        self.risky_available
    }

    /// The AGS4 edition the repaired bytes were validated against — whether you
    /// pinned it or it was derived from `TRAN_AGS`.
    #[must_use]
    pub fn edition(&self) -> &str {
        &self.edition
    }

    /// Write the repaired bytes to `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        std::fs::write(path, self.bytes()).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })
    }
}

/// One repair the fixer made.
pub struct Repair {
    kind: String,
    label: String,
    rule: String,
    line: Option<u32>,
    risky: bool,
}

impl Repair {
    fn from_engine(fix: &laterite_ags4_validator::Fix) -> Repair {
        Repair {
            // Through the engine's own serialisation rather than a match here.
            // A match would be a second naming table for the same enum, and the
            // Python and Node ledgers are built from this one — the point of
            // the record is that all three surfaces present identical strings.
            kind: serde_json::to_value(fix.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default(),
            label: fix.label.clone(),
            rule: fix.rule.clone(),
            line: fix.line,
            risky: fix.risk == FixRisk::Risky,
        }
    }

    /// What sort of repair this was — `"normalize_crlf"`, `"pad_short_row"`, …
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// A human-readable description of the repair.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The AGS Format Rule this repair answers, in full (`"AGS Format Rule 8"`).
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    /// The line it applied at, or `None` for the whole-file repairs (BOM, CRLF).
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// Whether this was one of the intent-guessing repairs — see [`Fix::risky`].
    #[must_use]
    pub fn is_risky(&self) -> bool {
        self.risky
    }
}

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never file contents — see the note in `document.rs`.

impl std::fmt::Debug for Fix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fix")
            .field("source", &self.source.describe())
            .field("edition", &self.edition)
            .field("encoding", &self.encoding)
            .field("risky", &self.risky)
            .field("only", &self.only)
            .field("exclude", &self.exclude)
            .finish()
    }
}

impl std::fmt::Debug for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fixed")
            .field("bytes", &self.text.len())
            .field("findings", &self.findings.len())
            .field("applied", &self.applied.len())
            .field("edition", &self.edition)
            .field("risky_available", &self.risky_available)
            .finish()
    }
}

impl std::fmt::Debug for Repair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repair")
            .field("kind", &self.kind)
            .field("label", &self.label)
            .field("rule", &self.rule)
            .field("line", &self.line)
            .field("risky", &self.risky)
            .finish()
    }
}
