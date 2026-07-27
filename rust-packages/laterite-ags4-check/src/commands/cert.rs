//! Certificate I/O for `lat` — load one, write one, and say what happened.
//!
//! **The trust decision is not here.** It used to be: `try_certified_skip` was this
//! file's own conjunction of freshness + engine identity + "profile covers", and it was
//! one of five such conjunctions across the surfaces, no two alike. It is now
//! `laterite_ags4_trust::check`, and this module is reduced to what a CLI should own:
//! reading a file, writing a file, and printing a line.

use std::path::{Path, PathBuf};

use laterite_ags4_core::index::Sidecar;
use laterite_ags4_trust::{RevalidateReason, Tier};
use laterite_cliutil::write_atomic;

use crate::commands::common::default_index_path;

/// Read a `.ags.idx`. A cert that can't be read is not an error — it is simply not
/// usable, and the caller re-validates. (`--index` on a missing cert should cost you a
/// validation, not a crash.)
pub fn load(cert_path: &Path) -> Result<Sidecar, String> {
    let bytes = std::fs::read(cert_path)
        .map_err(|e| format!("cannot read certificate {}: {e}", cert_path.display()))?;
    Sidecar::from_json(&bytes).map_err(|e| {
        format!(
            "{} is not a valid .ags.idx certificate: {e}",
            cert_path.display()
        )
    })
}

/// Write a minted certificate to `out`, else `<path>.ags.idx`.
pub fn write(sidecar: &Sidecar, path: &Path, out: Option<&Path>) -> Result<PathBuf, String> {
    let dest = out.map_or_else(|| default_index_path(path), PathBuf::from);
    let json = sidecar.to_json().map_err(|e| e.to_string())?;
    write_atomic(&dest, &json).map_err(|e| format!("write {}: {e}", dest.display()))?;
    Ok(dest)
}

/// Say why a certificate could not answer the question — in the user's terms, not the
/// type system's. A cert that doesn't help should say so; silently paying for a full
/// validation when you asked for a fast one is its own small lie.
pub fn why(reason: RevalidateReason) -> &'static str {
    match reason {
        RevalidateReason::FormatVersion => {
            "the certificate is an older format — rebuild it with `lat certify`"
        }
        RevalidateReason::SizeChanged | RevalidateReason::ContentChanged => {
            "the certificate is stale — the file changed since it was minted"
        }
        RevalidateReason::DifferentValidator => {
            "the certificate was minted by a different validator (or the python-ags4 \
             compat shim), whose verdict this one cannot inherit"
        }
        RevalidateReason::DifferentEngine => {
            "the certificate was minted by a different rule engine — the rules or the \
             dictionary have changed since, so its verdict may no longer hold"
        }
        RevalidateReason::EditionDiffers => {
            "the certificate judged the file against a different dictionary than this \
             request asks for (--dict-version)"
        }
        RevalidateReason::DictionaryChanged => {
            "the certificate was minted with a different custom dictionary than this \
             request supplies (--dict) — the effective dictionary changed, so its \
             verdict may no longer hold"
        }
        RevalidateReason::EncodingDiffers => {
            "the certificate read the file through a different decoder than this request \
             asks for (--encoding) — the bytes are the same, but the text they become is \
             not, and the rules judge the text"
        }
        // Deliberately distinct messages: "I never looked" and "I looked and there was
        // something there" are different facts, and the old format could only say the
        // first by pretending it was the second.
        RevalidateReason::TierNotMeasured(t) => match t {
            Tier::Errors => "the certificate never measured errors",
            Tier::Warnings => "the certificate never measured warnings",
            Tier::Fyi => "the certificate never measured FYI findings",
        },
        RevalidateReason::TierNotClean(t) => match t {
            Tier::Errors => "the certificate recorded errors",
            Tier::Warnings => {
                "the certificate recorded warnings — it counted them but \
                              does not carry their text, so they must be re-derived"
            }
            Tier::Fyi => {
                "the certificate recorded FYI findings — it counted them but \
                          does not carry their text, so they must be re-derived"
            }
        },
    }
}

/// The provenance note for a run whose CONTENT came from a certificate.
///
/// Note the wording: the rule ENGINE was skipped, not "the file was not checked". If a
/// world check ran (Rule 20's on-disk half), it ran for real — a certificate can never
/// stand in for that, and the note must not imply it did.
pub fn report_certified_skip(sidecar: &Sidecar, world_checked: bool) {
    let v = &sidecar.validation;
    let world = if world_checked {
        " (the on-disk FILE/ check still ran — a certificate cannot vouch for it)"
    } else {
        ""
    };
    eprintln!(
        "note: certified clean by {} at {} — rule engine skipped{world}",
        v.validator, v.checked_at
    );
}

#[cfg(test)]
mod tests {
    use super::why;
    use laterite_ags4_trust::{RevalidateReason, Tier};

    /// `why` is the user-facing reason table, and its doc contract is that every
    /// reason gets a **distinct** message naming its own cause. The end-to-end
    /// certificate tests only exercise the "stale" arm, and cargo-mutants mutates
    /// whole-function returns, not individual match-arm literals — so without this
    /// the other twelve arms are unverified. Assert a discriminating phrase per
    /// reason (a swapped or emptied arm then fails) and full pairwise distinctness.
    #[test]
    fn why_names_the_cause_of_every_revalidation_reason() {
        use RevalidateReason::*;
        // each reason's message names its own cause (a swapped/emptied arm fails)
        assert!(why(FormatVersion).contains("older format"));
        assert!(why(SizeChanged).contains("stale"));
        assert!(why(ContentChanged).contains("stale"));
        assert!(why(DifferentValidator).contains("different validator"));
        assert!(why(DifferentEngine).contains("different rule engine"));
        assert!(why(EditionDiffers).contains("--dict-version"));
        assert!(why(DictionaryChanged).contains("custom dictionary"));
        assert!(why(EncodingDiffers).contains("--encoding"));
        assert!(why(TierNotMeasured(Tier::Errors)).contains("never measured errors"));
        assert!(why(TierNotMeasured(Tier::Warnings)).contains("never measured warnings"));
        assert!(why(TierNotMeasured(Tier::Fyi)).contains("never measured FYI"));
        assert!(why(TierNotClean(Tier::Errors)).contains("recorded errors"));
        assert!(why(TierNotClean(Tier::Warnings)).contains("recorded warnings"));
        assert!(why(TierNotClean(Tier::Fyi)).contains("recorded FYI"));

        // distinct facts → distinct messages (the doc's explicit contract): only
        // SizeChanged/ContentChanged deliberately share, so the 13 non-shared
        // reasons must yield 13 different texts.
        let mut texts = [
            why(FormatVersion),
            why(SizeChanged),
            why(DifferentValidator),
            why(DifferentEngine),
            why(EditionDiffers),
            why(DictionaryChanged),
            why(EncodingDiffers),
            why(TierNotMeasured(Tier::Errors)),
            why(TierNotMeasured(Tier::Warnings)),
            why(TierNotMeasured(Tier::Fyi)),
            why(TierNotClean(Tier::Errors)),
            why(TierNotClean(Tier::Warnings)),
            why(TierNotClean(Tier::Fyi)),
        ];
        let count = texts.len();
        texts.sort_unstable();
        let mut uniq = texts.to_vec();
        uniq.dedup();
        assert_eq!(uniq.len(), count, "reason messages are not distinct");
    }
}
