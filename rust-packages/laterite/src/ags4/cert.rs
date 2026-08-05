//! Validity certificates — the `.ags.idx` sidecar.
//!
//! A certificate is a signed-by-construction statement that *these exact bytes*
//! validated clean, together with a byte index into them. Its point is that a
//! later reader can skip the rule engine entirely, having checked only a hash.
//!
//! ```no_run
//! use laterite::ags4;
//!
//! // Mint one over a file that validates clean.
//! let doc = ags4::read("delivery.ags").run()?;
//! doc.certify().to_path("delivery.ags.idx")?;
//!
//! // Offer it back, and the rule engine does not run.
//! let report = ags4::validate("delivery.ags")
//!     .index("delivery.ags.idx")
//!     .run()?;
//! assert!(report.certified());
//! # Ok::<(), laterite::Error>(())
//! ```
//!
//! # A certificate is never auto-discovered
//!
//! An `.ags.idx` sitting next to a file is not consent to trust it. You name it
//! or it is not used — the same rule the Python and Node surfaces keep, and the
//! reason is that the alternative silently converts "I read a file" into "I
//! trusted a file someone else put beside it".
//!
//! # It can only ever be an optimisation
//!
//! If the certificate does not match — different bytes, a different engine, or
//! a question it did not measure — the engine simply runs and
//! [`Report::revalidate_reason`] says why. A stale certificate cannot produce a
//! wrong verdict, only a slower one.
//!
//! [`Report::revalidate_reason`]: crate::ags4::Report::revalidate_reason

use std::path::Path;

use laterite_ags4_core::index::Sidecar;
use laterite_ags4_validator::CheckOptions;

use super::{Document, resolve_edition};
use crate::{Error, ErrorKind};

/// A pending [`Document::certify`]. Configure it, then choose an output.
pub struct Certify<'a> {
    pub(crate) doc: &'a Document,
    pub(crate) edition: Option<String>,
}

impl Certify<'_> {
    /// Judge against this edition rather than resolving one from `TRAN_AGS`.
    ///
    /// Recorded in the certificate as *forced*, which means only a later request
    /// forcing the same edition can be answered from it.
    #[must_use]
    pub fn edition(mut self, edition: impl Into<String>) -> Self {
        self.edition = Some(edition.into());
        self
    }

    fn mint(&self) -> Result<Sidecar, Error> {
        let opts = CheckOptions {
            dict_version: self.edition.as_deref().map(resolve_edition).transpose()?,
            custom_dict: None,
            // Whatever these are set to, `mint` forces both tiers on. Minting a
            // certificate weaker than the one we can measure would only ever
            // narrow the set of questions it can later answer.
            include_warnings: true,
            include_fyi: true,
            check_files: false,
            encoding: laterite_ags4_parse::resolve_encoding(self.doc.encoding.as_deref())
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "unknown encoding {:?}",
                            self.doc.encoding.as_deref().unwrap_or_default()
                        ),
                    )
                })?,
        };

        laterite_ags4_trust::mint(
            &self.doc.source_bytes,
            &opts,
            chrono::Utc::now().to_rfc3339(),
            None,
        )
        .map_err(|e| {
            // `NotCertifiable` is the interesting one, and it is a caller error
            // rather than an engine failure: a certificate asserts an error-clean
            // validation, so a file with errors has nothing to certify. Saying
            // that with `InvalidArgument` lets a caller branch on it without
            // reading the message.
            let kind = match &e {
                laterite_ags4_trust::MintError::NotCertifiable { .. } => ErrorKind::InvalidArgument,
                _ => ErrorKind::Other,
            };
            Error::with_source(kind, "cannot certify", e)
        })
    }

    /// Mint the certificate and return its bytes.
    ///
    /// The in-memory analogue of [`Certify::to_path`] — same certificate bar the
    /// mint timestamp, so a service can hand one back without a scratch file.
    ///
    /// # Errors
    /// [`ErrorKind::InvalidArgument`] if the file has error-severity findings and
    /// so cannot be certified at all; [`ErrorKind::Other`] if validation or
    /// indexing fails.
    pub fn to_bytes(self) -> Result<Vec<u8>, Error> {
        self.mint()?
            .to_json()
            .map_err(|e| Error::with_source(ErrorKind::Other, "cannot serialise certificate", e))
    }

    /// Mint the certificate and write it.
    ///
    /// # Errors
    /// As [`Certify::to_bytes`], plus [`ErrorKind::Io`] if the write fails.
    pub fn to_path(self, dest: impl AsRef<Path>) -> Result<(), Error> {
        let dest = dest.as_ref();
        let bytes = self.to_bytes()?;
        std::fs::write(dest, bytes).map_err(|e| {
            Error::with_source(ErrorKind::Io, format!("cannot write {}", dest.display()), e)
        })
    }
}

impl std::fmt::Debug for Certify<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Certify")
            .field("edition", &self.edition)
            .finish_non_exhaustive()
    }
}

/// Parse certificate bytes, mapping the failure onto the facade's kinds.
pub(crate) fn parse_cert(bytes: &[u8]) -> Result<Sidecar, Error> {
    Sidecar::from_json(bytes).map_err(|e| {
        Error::with_source(ErrorKind::InvalidArgument, "cannot read the certificate", e)
    })
}
