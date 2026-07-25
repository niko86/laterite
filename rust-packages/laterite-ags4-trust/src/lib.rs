//! The one door: validate an AGS4 file, optionally with a certificate, and mint a
//! certificate for one that passes.
//!
//! # Why a crate for this
//!
//! "Can I trust this `.ags.idx` enough to skip re-validating?" was being answered in
//! five places — the `lat` binary, `laterite-py`'s Rust half, `laterite-py`'s *Python*
//! half, `laterite-node`'s TypeScript, and the wasm surface — each with its own
//! hand-written conjunction of freshness, engine-identity and profile checks. They did
//! not agree, and four of the five would report a file clean that was not:
//!
//! * a cert minted with `--check-files`, then the `FILE/` tree deleted → still trusted,
//!   still "clean", because the certified bytes hadn't moved;
//! * a cert whose `warnings: 0` was never measured → a `--show-warnings` request read
//!   the zero and skipped the engine;
//! * a rule edited without a version bump → every cert from the old engine still
//!   claimed to be current.
//!
//! One door, one decision, one place to get it right.
//!
//! # The model
//!
//! Every check is one of two kinds, and the difference is the whole design:
//!
//! * **CONTENT** — a pure function of the certified bytes. Same bytes in, same findings
//!   out, forever. A certificate may stand in for this, because a SHA-256 of the bytes
//!   is a complete statement about it.
//! * **WORLD** — reads state the bytes do not contain (today: Rule 20's sibling `FILE/`
//!   tree). It can change without the file changing, so **no certificate may ever speak
//!   for it**, and it is re-run on every call.
//!
//! That partition is enforced in three places, in descending strength:
//!
//! 1. **No field to lie with.** `ValidationStamp` has no `check_files`, no FILE-tree
//!    hash, no world snapshot of any kind. There is nothing for a future predicate to
//!    read and conclude "the world is still as it was".
//! 2. **No parameter to ask it with.** [`WorldScope::OnDisk`] cannot be constructed
//!    without a path. A bytes/text caller cannot request a world check; if they set
//!    `check_files` anyway they get [`ValidatorError::WorldCheckRequiresSource`], not a
//!    clean Rule 20.
//! 3. **No route around it.** In [`check`], `run_world` sits *outside* the
//!    certificate branch. A vouched cert short-circuits the content engine and nothing
//!    else.
//!
//! And when someone adds a new knob to `CheckOptions`, [`split_options`] stops
//! compiling until they say which kind it is. That compile error is the point: an
//! unclassified knob is a future false clean.

use std::fmt::Write as _;

use laterite_ags4_core::error::CliError;
// Re-exported so a surface can name a decision, a reason or a tier without also taking
// a direct dependency on core's index module — the trust vocabulary lives at ONE import.
pub use laterite_ags4_core::index::{
    CustomDictRef, Decision, ENGINE_IDENTITY, EditionInput, EngineId, Question, RevalidateReason,
    Sidecar, Tier, TierCoverage, ValidationStamp,
};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::{
    CheckOptions, DictResolution, DictVersion, Findings, ValidatorError, WorldScope,
    check_parsed_with_dict, findings, overlay, world,
};

/// The engine asking the question: this build's rules, dictionary and compat profile.
///
/// `compat` is `Some(v)` only for the python-ags4 drop-in shim, whose behaviour
/// deliberately differs from the native engine — so a compat-minted certificate cannot
/// answer for a native request, nor the reverse.
#[must_use]
pub fn engine_id(compat: Option<String>) -> EngineId {
    EngineId {
        validator: ENGINE_IDENTITY.to_string(),
        fingerprint: laterite_ags4_validator::ENGINE_FINGERPRINT.to_string(),
        compat,
    }
}

/// Everything a caller is asking for. `bytes` are the file's bytes whatever door they
/// came through — a path is read into them, and `text` is its UTF-8 encoding — because
/// a certificate is a statement about bytes and nothing else.
pub struct Request<'a> {
    /// The file's bytes. The certificate's SHA-256 is over exactly these.
    pub bytes: &'a [u8],
    /// The validation knobs. [`split_options`] classifies each one CONTENT or WORLD.
    pub opts: &'a CheckOptions,
    /// A certificate offered for these bytes, if the caller named one. Never
    /// auto-discovered: an `.ags.idx` sitting beside a file is not consent to trust it.
    pub cert: Option<&'a Sidecar>,
    /// What the caller may look at beyond the bytes. `None` for every bytes/text/browser
    /// caller — they have no world — and `OnDisk(path)` only for a real file on disk.
    pub world: WorldScope,
    /// The python-ags4 compat version, if this is a compat-shim call.
    pub compat: Option<String>,
}

/// What a [`check`] produced, and — honestly — how.
#[derive(Debug)]
pub struct Outcome {
    /// Content findings ∪ world findings.
    pub findings: Findings,
    /// The dictionary the file was judged against.
    pub dict_version: DictVersion,
    /// How that dictionary was chosen (forced / exact / guessed / fallback).
    pub resolution: DictResolution,
    /// `true` iff a certificate stood in for the CONTENT engine. The world half ran
    /// regardless — this flag never means "nothing was checked".
    pub certified: bool,
    /// If a certificate was offered and NOT used, why. `None` when no cert was offered,
    /// or when it was vouched. Surfaces report this so a user can see why their cert
    /// didn't help, instead of silently paying for a full validation.
    pub revalidate_reason: Option<RevalidateReason>,
}

/// Split `CheckOptions` into the question a certificate could answer (CONTENT) and the
/// world a check is allowed to look at (WORLD).
///
/// **The destructure below is exhaustive on purpose.** Add a field to `CheckOptions` and
/// this function stops compiling until the author writes it into one of the two lists.
/// That compile error is the only structural defence there is against the next
/// `check_files`: a knob that reads external state but gets filed with the pure ones is
/// a knob a certificate will happily vouch for, and that is a false clean waiting to
/// happen. Rust has no effect system; this is the closest thing to one we get.
pub fn split_options(
    opts: &CheckOptions,
    world: WorldScope,
) -> Result<(Question, WorldScope), ValidatorError> {
    let CheckOptions {
        // --- CONTENT: pure functions of the bytes. A certificate may speak for these.
        dict_version,     // which dictionary judges the bytes
        include_warnings, // which severity tiers the engine runs
        include_fyi,      //   "
        encoding,         // how the bytes decode to text
        // A custom `--dict` overlay (#568) is CONTENT: it was parsed once at the
        // surface boundary into an owned `CustomDict`, and its identity (base + delta,
        // hashed) is a pure function of that dictionary — nothing on disk is read per
        // file. So the cert can and must speak for it: which dictionary produced the
        // verdict is recorded on the Question below and compared in `Sidecar::decide`.
        custom_dict,
        // --- WORLD: reads state the bytes do not contain. No certificate may speak
        //     for these; they run live, every call.
        check_files, // Rule 20's on-disk FILE/ tree
    } = opts;

    // The request and the ability to honour it must agree. Asking for the on-disk check
    // with no disk to look at is not a question anyone can answer, and answering it
    // "clean" — which is what the engine used to do — is a lie.
    if *check_files && matches!(world, WorldScope::None) {
        return Err(ValidatorError::WorldCheckRequiresSource);
    }
    // ...and a world we were handed but not asked to look at, we do not look at.
    let world = if *check_files {
        world
    } else {
        WorldScope::None
    };

    Ok((
        Question {
            want_warnings: *include_warnings,
            want_fyi: *include_fyi,
            forced_edition: dict_version.map(|d| d.as_str().to_string()),
            // The decoder is part of the question, not just of the parse. The findings are
            // a function of bytes AND decoder, and the certificate seals only the bytes.
            encoding: encoding.name().to_string(),
            // The custom overlay this request uses (#568) — compared against the cert's
            // own record. `custom_dict` here is the destructured `&Option<CustomDict>`.
            custom_dict: custom_dict.as_ref().map(custom_dict_record),
        },
        world,
    ))
}

/// The cert's portable record of a custom `--dict` overlay: its advisory name +
/// the hex of its precomputed identity hash. Both the Question (what this request
/// uses) and the `ValidationStamp` (what the verdict used) carry it, so
/// [`Sidecar::decide`] can compare like for like.
fn custom_dict_record(cd: &overlay::CustomDict) -> CustomDictRef {
    CustomDictRef {
        name: cd.name.clone(),
        hash: cd.hash.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        }),
    }
}

/// **Validate.** The only way in.
///
/// ```text
/// content = cert.decide(bytes, question, engine) == Vouched
///             ? {}                        // vouched ⇒ every asked tier was clean
///             : run_content_engine(bytes)
/// world   = run_world(scope)              // <- NOT inside that branch. Ever.
/// findings = content ∪ world
/// ```
///
/// The shape of those four lines is the design. A certificate can only ever remove the
/// content run; the world run is not something it is in a position to remove, so it is
/// not written anywhere it could.
pub fn check(req: Request<'_>) -> Result<Outcome, ValidatorError> {
    let (question, world) = split_options(req.opts, req.world)?;
    let engine = engine_id(req.compat.clone());

    // Ask the certificate — if one was offered — whether it can answer THIS question.
    let decision = req.cert.map(|c| c.decide(req.bytes, &question, &engine));

    if let Some(Decision::Vouched) = decision {
        let cert = req.cert.expect("Vouched implies a cert");
        let mut found = findings::Findings::new();

        // A vouched cert means: every tier the caller asked about was measured, and
        // every one came back empty. So the content findings ARE empty — nothing to
        // reconstruct, and nothing the cert has to remember beyond the counts.
        //
        // The world, meanwhile, gets looked at now. It always does.
        if !matches!(world, WorldScope::None) {
            // The on-disk half needs the FILE group, so the bytes are parsed even here.
            // Still far cheaper than the rule engine, and the alternative — trusting a
            // cert for the state of a directory — is not on the table.
            let parsed = parse_bytes(req.bytes, req.opts.encoding).map_err(ValidatorError::from)?;
            world::run(&parsed, &world, &mut found);
        }

        let (dv, res) = edition_of(cert)?;
        return Ok(Outcome {
            findings: found,
            dict_version: dv,
            resolution: res,
            certified: true,
            revalidate_reason: None,
        });
    }

    // No usable cert: the engine runs. Content and world both, through the validator's
    // own door, which puts them in the same union.
    let parsed = parse_bytes(req.bytes, req.opts.encoding).map_err(ValidatorError::from)?;
    let (found, dv, res) = check_parsed_with_dict(&parsed, req.opts, &world)?;
    Ok(Outcome {
        findings: found,
        dict_version: dv,
        resolution: res,
        certified: false,
        revalidate_reason: match decision {
            Some(Decision::Revalidate(r)) => Some(r),
            _ => None,
        },
    })
}

/// **Mint a certificate** for `bytes` — but only after validating them here.
///
/// Note what this function does NOT take: any finding count, any verdict, any claim
/// about the file. The caller cannot tell it that the file is clean, or that there were
/// no warnings, because the caller is exactly who kept getting that wrong. Every
/// certificate `laterite-py` ever produced recorded `warnings: 0` and `fyi: 0` — not
/// because it had measured them, but because those were the default arguments to a
/// factory function that let the caller assert them.
///
/// So: both tiers are forced on, the engine runs, and the stamp is built from what it
/// actually returned. The mint refuses iff there are **errors** — warnings and FYI are
/// recorded, not fatal, because a file may legitimately carry them and still be a valid
/// AGS4 delivery.
///
/// There is no `world` parameter either. A certificate is a statement about bytes, and
/// the on-disk `FILE/` tree is not part of them.
pub fn mint(
    bytes: &[u8],
    opts: &CheckOptions,
    checked_at: String,
    compat: Option<String>,
) -> Result<Sidecar, MintError> {
    // Both tiers, always. A cert that measured only errors can still be used — it just
    // can't answer a `--show-warnings` request (`TierNotMeasured`) — but there is no
    // reason to mint a weaker cert than we can, and a caller cannot ask us to.
    let full = CheckOptions {
        include_warnings: true,
        include_fyi: true,
        // The mint is CONTENT-only by construction. Even if the caller was doing an
        // on-disk check, its result has no business in a certificate.
        check_files: false,
        ..opts.clone()
    };
    let parsed = parse_bytes(bytes, full.encoding).map_err(ValidatorError::from)?;
    let (found, dv, res) = check_parsed_with_dict(&parsed, &full, &WorldScope::None)?;

    let errors = count_of(&found, findings::Severity::Error);
    if errors > 0 {
        return Err(MintError::NotCertifiable { errors });
    }

    let edition = match opts.dict_version {
        // Forced: only a request forcing the same edition may be answered from this.
        Some(d) => EditionInput::Forced {
            edition: d.as_str().to_string(),
        },
        // Auto: record what it resolved to AND how (provenance), but the trust test is
        // simply that the request also auto-resolves — same bytes + same engine ⇒ the
        // same answer, so the values need not be compared.
        None => EditionInput::Auto {
            resolved: dv.as_str().to_string(),
            resolution: res,
        },
    };

    let id = engine_id(compat);
    let stamp = ValidationStamp {
        validator: id.validator,
        engine: id.fingerprint,
        compat: id.compat,
        checked_at,
        edition,
        // What the bytes were READ as. `Sidecar::decide` refuses a request made through a
        // different decoder: same bytes, different text, different question.
        encoding: full.encoding.name().to_string(),
        // The custom overlay this verdict was reached against (#568), so a later
        // request supplying a different (or no) dict revalidates rather than trusting a
        // cert minted under another dictionary.
        custom_dict: opts.custom_dict.as_ref().map(custom_dict_record),
        // Measured, all three, because we just ran all three. This is the only place a
        // ValidationStamp is built from a real engine result.
        errors: TierCoverage::Measured { count: errors },
        warnings: TierCoverage::Measured {
            count: count_of(&found, findings::Severity::Warning),
        },
        fyi: TierCoverage::Measured {
            count: count_of(&found, findings::Severity::Fyi),
        },
    };
    // Reuse the parse we already did to validate — its source-true byte offsets
    // are exactly what the index needs, so certifying does not walk the file a
    // second time (#5, ~14% of a mint on a 25 MB file).
    Sidecar::assemble_from_parsed(bytes, &parsed, stamp)
        .map_err(|e: CliError| MintError::NotIndexable(e.to_string()))
}

/// Findings of one severity.
// Bounded by the number of validator findings in a file, which can't exceed
// the file's cell count — far below u32::MAX for any real AGS4 file.
#[allow(clippy::cast_possible_truncation)]
fn count_of(found: &Findings, sev: findings::Severity) -> u32 {
    found
        .values()
        .flatten()
        .filter(|f| f.severity == sev)
        .count() as u32
}

/// The edition a vouched certificate was judged against, as the surfaces report it.
/// Read back from the stamp rather than re-derived: re-deriving means re-parsing, which
/// is the cost the certificate exists to avoid — and a cert that reported "exact" for a
/// file whose edition was actually *guessed* (O-42) would be lying about the one thing
/// it is for.
fn edition_of(cert: &Sidecar) -> Result<(DictVersion, DictResolution), ValidatorError> {
    let e = &cert.validation.edition;
    let dv = DictVersion::from_edition(e.edition()).ok_or_else(|| {
        ValidatorError::NotAgs4(format!(
            "certificate names an edition this build does not bundle: {:?}",
            e.edition()
        ))
    })?;
    Ok((dv, e.resolution()))
}

/// Why a mint refused.
///
/// Its own type, not a `ValidatorError`, because two of the three reasons are not
/// validation failures at all — and squeezing them into `NotAgs4` produced the sentence
/// "not a parseable AGS4 file: cannot certify: 3 error-severity finding(s)", which is
/// false in its first clause about a file that parsed perfectly well.
#[derive(Debug)]
pub enum MintError {
    /// The file could not be validated at all (unreadable edition, bad dictionary, …).
    Validate(ValidatorError),
    /// The file validated, and it has errors. A certificate asserts an error-clean
    /// validation, so there is nothing to certify. (Warnings and FYI findings are
    /// recorded, not refused.)
    NotCertifiable { errors: u32 },
    /// The file validated clean, but its bytes cannot be INDEXED — today only because
    /// they are not UTF-8. The certificate carries byte offsets into the source, and an
    /// offset into bytes we cannot address is not a fact we can write down.
    NotIndexable(String),
}

impl std::fmt::Display for MintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MintError::Validate(e) => write!(f, "{e}"),
            MintError::NotCertifiable { errors } => write!(
                f,
                "cannot certify: {errors} error-severity finding(s) — a certificate \
                 asserts an error-clean validation"
            ),
            MintError::NotIndexable(why) => write!(f, "cannot index the source: {why}"),
        }
    }
}

impl std::error::Error for MintError {}

impl From<ValidatorError> for MintError {
    fn from(e: ValidatorError) -> Self {
        MintError::Validate(e)
    }
}

impl MintError {
    /// The process exit code, on the `lat` contract: a refused-but-checkable file is 1
    /// (findings), an unusable one is whatever the validator says, and un-indexable
    /// bytes are 4 (input we cannot work with).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            MintError::Validate(e) => e.exit_code(),
            MintError::NotCertifiable { .. } => 1,
            MintError::NotIndexable(_) => 4,
        }
    }
}

#[cfg(test)]
mod tests;
