//! Reconciling several AGS4 deliveries of one project into one file.

use std::path::{Path, PathBuf};

use laterite_ags4_merge::{MergeError, MergeOpts, MissingTranMode, TypeClashMode, merge_parsed};

use super::{Document, WriteMode, resolve_edition, validator_kind};
use crate::{Error, ErrorKind};

/// How to settle a heading two files typed differently.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypeClash {
    /// Refuse the merge. The default, deliberately: reconciling two independent
    /// producers' declared types is high-stakes and less reversible than a
    /// single-file fixup, so an automatic resolution is something you ask for.
    #[default]
    Refuse,
    /// Fall back to `X`, free text. Lossless at the byte level — every value's
    /// raw characters survive — but it throws the type away, which is the least
    /// informative resolution available.
    Widen,
    /// Keep the column numeric where that costs no digit: when every clashing
    /// code is in the `nDP` family, take **max(n)** and zero-pad the coarser
    /// files' cells.
    ///
    /// Promote, never demote — taking the lower precision would round `10.00123`
    /// to `10.00` and destroy data. That also makes the outcome independent of
    /// the order the files were given in. Anything outside the `nDP` family
    /// falls back to [`TypeClash::Widen`].
    Promote,
}

impl TypeClash {
    fn to_engine(self) -> TypeClashMode {
        match self {
            TypeClash::Refuse => TypeClashMode::Error,
            TypeClash::Widen => TypeClashMode::Widen,
            TypeClash::Promote => TypeClashMode::Promote,
        }
    }
}

/// What to do when no transmission stamp is supplied and the sources carry
/// TRAN rows of their own.
///
/// Only consulted when you did not call [`Merge::transmission`]; supplying a
/// stamp makes this irrelevant.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissingTran {
    /// Fold TRAN like any other group and leave a [`Note`]. `TRAN_ISNO` is a KEY
    /// heading and two deliveries carry different issue numbers, so both
    /// transmissions survive — and Rule 14 allows exactly one row.
    ///
    /// The default, so that adding this option changed nothing for anyone.
    #[default]
    Reconcile,
    /// Refuse the merge before any bytes are produced.
    ///
    /// Named [`Refuse`](Self::Refuse) rather than `Error` for the same reason
    /// [`TypeClash::Refuse`] is: at this level it reads as the thing merge does,
    /// not as an error category.
    Refuse,
}

impl MissingTran {
    fn to_engine(self) -> MissingTranMode {
        match self {
            MissingTran::Reconcile => MissingTranMode::Reconcile,
            MissingTran::Refuse => MissingTranMode::Error,
        }
    }
}

/// Something the merge resolved that you may want to look at.
///
/// Advisory, not a failure: the merge completed. Each one names what it settled
/// and where, so a merge stays auditable rather than being a black box that
/// produced a file.
#[derive(Debug, Clone)]
pub struct Note {
    kind: String,
    group: Option<String>,
    heading: Option<String>,
    message: String,
}

impl Note {
    /// What sort of note this is — a recency contradiction, a widened type, a
    /// missing merge transmission.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The group it concerns, when it concerns one.
    #[must_use]
    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    /// The heading it concerns, when it concerns one.
    #[must_use]
    pub fn heading(&self) -> Option<&str> {
        self.heading.as_deref()
    }

    /// The explanation, in full.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// One row a later file revised.
///
/// AGS4 carries no per-row timestamp, so a stale row inside an
/// otherwise-newer file cannot be detected automatically. This is the closest
/// thing available: a record of what the winning file changed, for a person to
/// review.
#[derive(Debug, Clone)]
pub struct Revision {
    group: String,
    key: Vec<String>,
    changed: Vec<String>,
    winner: usize,
}

impl Revision {
    /// The group the revised row is in.
    #[must_use]
    pub fn group(&self) -> &str {
        &self.group
    }

    /// The KEY values identifying the row.
    #[must_use]
    pub fn key(&self) -> &[String] {
        &self.key
    }

    /// The headings whose value the winning file changed.
    ///
    /// A typed comparison, so a formatting-only difference is not reported here
    /// — the same rule [`diff`](super::diff) follows.
    #[must_use]
    pub fn changed(&self) -> &[String] {
        &self.changed
    }

    /// Which input supplied the winning content, by its index in the sources
    /// you passed.
    #[must_use]
    pub fn winner(&self) -> usize {
        self.winner
    }
}

/// What a merge produced.
pub struct Merged {
    text: String,
    notes: Vec<Note>,
    revisions: Vec<Revision>,
}

impl Merged {
    /// The merged AGS4 bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Take ownership of the merged bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.text.into_bytes()
    }

    /// The merged AGS4 as text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Take ownership of the merged text.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }

    /// What the merge settled along the way — see [`Note`].
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// The rows a later file revised — see [`Revision`].
    #[must_use]
    pub fn revisions(&self) -> &[Revision] {
        &self.revisions
    }

    /// Write the merged bytes to `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        std::fs::write(path, self.bytes()).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", path.display()), e)
        })
    }
}

/// One input to a merge.
enum Source<'a> {
    Path(PathBuf),
    Bytes(Vec<u8>),
    /// As with [`diff_documents`](super::diff_documents), a document is merged
    /// as it stands now — edits included — not as the file it was read from.
    Document(&'a Document),
}

impl Source<'_> {
    fn bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            Source::Path(p) => std::fs::read(p).map_err(|e| {
                Error::with_source(ErrorKind::Io, format!("cannot read {}", p.display()), e)
            }),
            Source::Bytes(b) => Ok(b.clone()),
            Source::Document(doc) => super::write(doc)
                .mode(WriteMode::Report)
                .to_bytes()
                .map(super::Written::into_bytes),
        }
    }

    fn describe(&self) -> String {
        match self {
            Source::Path(p) => format!("path {}", p.display()),
            Source::Bytes(b) => format!("{} bytes", b.len()),
            Source::Document(doc) => format!("document of {} groups", doc.len()),
        }
    }
}

/// A pending merge. Configure it, then [`Merge::run`].
pub struct Merge<'a> {
    sources: Vec<Source<'a>>,
    on_type_clash: TypeClash,
    on_missing_tran: MissingTran,
    edition: Option<String>,
    encoding: Option<String>,
    mode: WriteMode,
    tran: Option<laterite_ags4_emit::TranStamp>,
}

fn pending(sources: Vec<Source<'_>>) -> Merge<'_> {
    Merge {
        sources,
        on_type_clash: TypeClash::default(),
        on_missing_tran: MissingTran::default(),
        edition: None,
        encoding: None,
        mode: WriteMode::default(),
        tran: None,
    }
}

/// Merge AGS4 files on disk into one.
///
/// **Order is meaning**: a later file wins a KEY conflict, so the sources go
/// oldest first. Rows are identified by their group's dictionary KEY headings,
/// not by position, so a re-sorted borehole list still merges onto its prior
/// self rather than doubling it.
///
/// Fewer than two sources is an error rather than a passthrough — merging one
/// file is a question with no answer, and silently returning it would hide a
/// caller who meant to pass more.
#[must_use]
pub fn merge<I, P>(sources: I) -> Merge<'static>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    pending(
        sources
            .into_iter()
            .map(|p| Source::Path(p.as_ref().to_path_buf()))
            .collect(),
    )
}

/// Merge AGS4 documents already in memory — see [`merge`].
#[must_use]
pub fn merge_bytes<I, B>(sources: I) -> Merge<'static>
where
    I: IntoIterator<Item = B>,
    B: Into<Vec<u8>>,
{
    pending(
        sources
            .into_iter()
            .map(|b| Source::Bytes(b.into()))
            .collect(),
    )
}

/// Merge read handles — see [`merge`].
///
/// Each document is merged as it stands now, including any edit, for the same
/// reason [`diff_documents`](super::diff_documents) compares the current
/// content.
#[must_use]
pub fn merge_documents<'a, I>(sources: I) -> Merge<'a>
where
    I: IntoIterator<Item = &'a Document>,
{
    pending(sources.into_iter().map(Source::Document).collect())
}

impl<'a> Merge<'a> {
    /// How to settle a heading two files typed differently — see [`TypeClash`].
    #[must_use]
    pub fn on_type_clash(mut self, mode: TypeClash) -> Merge<'a> {
        self.on_type_clash = mode;
        self
    }

    /// What to do when no [`transmission`](Merge::transmission) stamp is
    /// supplied — see [`MissingTran`].
    #[must_use]
    pub fn on_missing_tran(mut self, mode: MissingTran) -> Merge<'a> {
        self.on_missing_tran = mode;
        self
    }

    /// Merge against a specific AGS4 edition. See [`editions`](super::editions).
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Merge<'a> {
        self.edition = Some(edition.into());
        self
    }

    /// Decode every source with this encoding instead of UTF-8 — a WHATWG label.
    #[must_use]
    pub fn encoding(mut self, label: impl Into<String>) -> Merge<'a> {
        self.encoding = Some(label.into());
        self
    }

    /// What to do about the validity of the merged output — see [`WriteMode`].
    #[must_use]
    pub fn mode(mut self, mode: WriteMode) -> Merge<'a> {
        self.mode = mode;
        self
    }

    /// State the transmission the merged file represents, producing one
    /// synthesised `TRAN` row with the inputs' issue numbers and dates recorded
    /// in `TRAN_REM` for provenance.
    ///
    /// Without it, `TRAN` is reconciled like any other group and a [`Note`]
    /// records that no merge transmission was supplied. Reconciliation keeps
    /// rows with distinct KEYs, and `TRAN_ISNO` is one, so each input's
    /// transmission normally survives and the result carries more TRAN rows
    /// than Rule 14 permits. Not inventing one is still the honest outcome: a
    /// merged file's transmission is a fact about the merge, and only you know
    /// it.
    #[must_use]
    pub fn transmission(
        mut self,
        issue_number: impl Into<String>,
        date: impl Into<String>,
        producer: impl Into<String>,
        recipient: impl Into<String>,
        status: impl Into<String>,
    ) -> Merge<'a> {
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
    pub fn run(self) -> Result<Merged, Error> {
        if self.sources.len() < 2 {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!(
                    "merge needs at least two files, and was given {}",
                    self.sources.len()
                ),
            ));
        }

        let enc = laterite_ags4_parse::resolve_encoding(self.encoding.as_deref())
            .ok_or_else(|| super::bad_encoding(self.encoding.as_deref().unwrap_or_default()))?;

        let mut parsed = Vec::with_capacity(self.sources.len());
        for (i, source) in self.sources.iter().enumerate() {
            let bytes = source.bytes()?;
            parsed.push(
                laterite_ags4_parse::parse_bytes(&bytes, enc)
                    .map_err(laterite_ags4_validator::ValidatorError::from)
                    .map_err(|e| {
                        Error::with_source(
                            validator_kind(e.kind()),
                            format!("cannot read source {i} as AGS4"),
                            e,
                        )
                    })?,
            );
        }

        let opts = MergeOpts {
            on_type_clash: self.on_type_clash.to_engine(),
            on_missing_tran: self.on_missing_tran.to_engine(),
            edition: match &self.edition {
                Some(label) => resolve_edition(label)?,
                None => laterite_ags4_reference::dict::FALLBACK,
            },
            emit_mode: match self.mode {
                WriteMode::AutoFix => laterite_ags4_emit::EmitMode::AutoFix,
                WriteMode::Report => laterite_ags4_emit::EmitMode::Report,
                WriteMode::Strict => laterite_ags4_emit::EmitMode::Strict,
            },
            tran: self.tran,
        };

        let result = merge_parsed(&parsed, &opts).map_err(|e| {
            // Kind by variant here, not by a token, because merge is the one
            // engine that does NOT publish a `kind()`. The tokens are still the
            // siblings' — `type_conflict`, `unit_conflict` — so a caller routing
            // on `kind_str()` reads the same thing whichever surface answered.
            let kind = match &e {
                MergeError::TypeConflict { .. } => ErrorKind::TypeConflict,
                MergeError::UnitConflict { .. } => ErrorKind::UnitConflict,
                MergeError::MissingTran => ErrorKind::MissingTran,
                MergeError::Emit(_) => ErrorKind::Emit,
            };
            // The engine's own message carries the remedy — which mode settles a
            // type clash, and why nothing settles a unit one — so it is the
            // message rather than a cause. Restating it here would print it
            // twice under `{:#}`.
            Error::new(kind, e.to_string())
        })?;

        Ok(Merged {
            // The merged bytes come from our own emitter, so they are UTF-8 by
            // construction — same argument as `Written::text`, surfaced as an
            // error rather than unwrapped for the same reason.
            text: String::from_utf8(result.bytes).map_err(|e| {
                Error::with_source(
                    ErrorKind::Emit,
                    "the merge produced bytes that are not UTF-8",
                    e,
                )
            })?,
            notes: result
                .warnings
                .into_iter()
                .map(|w| Note {
                    kind: w.kind.to_string(),
                    group: w.group,
                    heading: w.heading,
                    message: w.message,
                })
                .collect(),
            revisions: result
                .revisions
                .into_iter()
                .map(|r| Revision {
                    group: r.group,
                    key: r.key,
                    changed: r.changed,
                    winner: r.winner_file,
                })
                .collect(),
        })
    }
}

// --- Debug -------------------------------------------------------------
//
// Shape and settings, never cell values — see the note in `document.rs`.

impl std::fmt::Debug for Merge<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Merge")
            .field(
                "sources",
                &self
                    .sources
                    .iter()
                    .map(Source::describe)
                    .collect::<Vec<_>>(),
            )
            .field("on_type_clash", &self.on_type_clash)
            .field("on_missing_tran", &self.on_missing_tran)
            .field("edition", &self.edition)
            .field("encoding", &self.encoding)
            .field("mode", &self.mode)
            .field("transmission", &self.tran.is_some())
            .finish()
    }
}

impl std::fmt::Debug for Merged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Merged")
            .field("bytes", &self.text.len())
            .field("notes", &self.notes.len())
            .field("revisions", &self.revisions.len())
            .finish_non_exhaustive()
    }
}
