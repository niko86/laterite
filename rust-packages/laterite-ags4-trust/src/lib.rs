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
use std::sync::{Arc, Weak};

use laterite_ags4_core::error::CliError;
// Re-exported so a surface can name a decision, a reason or a tier without also taking
// a direct dependency on core's index module — the trust vocabulary lives at ONE import.
pub use laterite_ags4_core::index::{
    CustomDictRef, Decision, ENGINE_IDENTITY, EditionInput, EngineId, Question, RevalidateReason,
    Sidecar, Tier, TierCoverage, ValidationStamp,
};
use laterite_ags4_parse::{ParsedFile, parse_bytes};
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
        // A custom `--dict` overlay (laterite-dev#568) is CONTENT: it was parsed once at the
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
            // The custom overlay this request uses (laterite-dev#568) — compared against the cert's
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

        return vouched(cert, found);
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

/// The outcome of a vouched certificate: no content findings, and the edition the cert
/// was judged under. `found` holds whatever the world check found, if one ran.
fn vouched(cert: &Sidecar, found: Findings) -> Result<Outcome, ValidatorError> {
    let (dv, res) = edition_of(cert)?;
    Ok(Outcome {
        findings: found,
        dict_version: dv,
        resolution: res,
        certified: true,
        revalidate_reason: None,
    })
}

/// [`Request`] for a file that has not been read yet: a path's door (#1097).
///
/// [`check`] takes the bytes, so a path's caller used to read the whole file first,
/// even when the certificate vouched and nothing would look at the bytes but a hash.
/// Here the door reads the file itself, and only as far as the decision needs.
pub struct ReaderRequest<'a, F> {
    /// The file. It is measured by seeking to its end and read from its start.
    pub file: F,
    /// As [`Request::opts`].
    pub opts: &'a CheckOptions,
    /// As [`Request::cert`].
    pub cert: Option<&'a Sidecar>,
    /// As [`Request::world`].
    pub world: WorldScope,
    /// As [`Request::compat`].
    pub compat: Option<String>,
    /// The caller named this certificate as the one for this file, so a cert whose
    /// size or SHA-256 does not match is an error ([`ReaderError::StaleCert`]), and is
    /// raised before the engine runs. Otherwise it is only a hint, and a stale one
    /// revalidates as [`check`] does.
    pub named: bool,
}

/// Why [`check_reader`] has no [`Outcome`].
#[derive(Debug)]
pub enum ReaderError {
    /// The file could not be measured or read.
    Io(std::io::Error),
    /// [`ReaderRequest::named`] was set, and the certificate is not for these bytes.
    StaleCert,
    /// As [`check`]'s error.
    Validate(ValidatorError),
}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaderError::Io(e) => write!(f, "{e}"),
            ReaderError::StaleCert => {
                f.write_str("the certificate does not match the file (size / SHA-256 differ)")
            }
            ReaderError::Validate(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ReaderError {}

impl From<std::io::Error> for ReaderError {
    fn from(e: std::io::Error) -> Self {
        ReaderError::Io(e)
    }
}

impl From<ValidatorError> for ReaderError {
    fn from(e: ValidatorError) -> Self {
        ReaderError::Validate(e)
    }
}

/// [`check`], reading the file only as far as the decision needs (#1097).
///
/// Same verdict, same `revalidate_reason`, as [`check`] over the file's bytes. What
/// changes is what is read and what is held:
///
/// * a certificate of another size is settled by the file's length, before a byte is
///   read;
/// * a certificate that would vouch for matching bytes, with no world check asked, is
///   checked by hashing the file as it streams. If it matches, the bytes are never
///   held, so the memory a vouched check costs does not grow with the file;
/// * anything else needs the engine, which needs the bytes, so they are read whole,
///   once, and handed to [`check`].
///
/// The only file read twice is a stale one offered as a hint: hashed as a stream,
/// found not to match, then read again for the engine.
///
/// A file that cannot seek (a pipe) is read whole, as [`check`]'s caller always did.
pub fn check_reader<F: std::io::Read + std::io::Seek>(
    req: ReaderRequest<'_, F>,
) -> Result<Outcome, ReaderError> {
    let ReaderRequest {
        mut file,
        opts,
        cert,
        world,
        compat,
        named,
    } = req;

    let size = match file.seek(std::io::SeekFrom::End(0)) {
        Ok(n) => {
            file.rewind()?;
            Some(n)
        }
        Err(_) => None,
    };

    if let (Some(c), Some(size)) = (cert, size) {
        // `decide`'s first two gates, format and size, read nothing.
        let sized = c.version == laterite_ags4_core::index::SIDECAR_VERSION && c.size_matches(size);
        if named && !sized {
            return Err(ReaderError::StaleCert);
        }
        if sized && only_the_hash_is_left(c, opts, &world, compat.as_deref()) {
            if c.is_fresh_for_reader(&mut file)? {
                return Ok(vouched(c, Findings::new())?);
            }
            if named {
                return Err(ReaderError::StaleCert);
            }
            file.rewind()?;
        }
    }

    let mut bytes = Vec::with_capacity(size.and_then(|n| usize::try_from(n).ok()).unwrap_or(0));
    file.read_to_end(&mut bytes)?;
    if named && cert.is_some_and(|c| !c.is_fresh_for(&bytes)) {
        return Err(ReaderError::StaleCert);
    }
    Ok(check(Request {
        bytes: &bytes,
        opts,
        cert,
        world,
        compat,
    })?)
}

/// Would `cert` vouch for this request, given bytes that match it? Only then is a
/// streamed hash the whole of the work. A world check needs the parse, and any other
/// answer needs the engine; both need the bytes.
fn only_the_hash_is_left(
    cert: &Sidecar,
    opts: &CheckOptions,
    world: &WorldScope,
    compat: Option<&str>,
) -> bool {
    match split_options(opts, world.clone()) {
        Ok((question, WorldScope::None)) => {
            cert.answers(&question, &engine_id(compat.map(str::to_string))) == Decision::Vouched
        }
        _ => false,
    }
}

/// [`check`] for a caller that already holds the parse — a read handle validating
/// what it read, which would otherwise pay for a second parse of the same bytes.
///
/// There is no certificate parameter, and that is the point: a certificate is a
/// statement about bytes, and a caller holding only a parse has no bytes to show
/// it. A caller with a cert to offer goes through [`check`].
///
/// `parsed` must be `parse_bytes(bytes, opts.encoding)` of the file being judged;
/// a parse under another decoder answers a different question.
pub fn check_parsed(
    parsed: &ParsedFile,
    opts: &CheckOptions,
    world: WorldScope,
) -> Result<Outcome, ValidatorError> {
    check_parsed_measured(parsed, opts, world).map(|(outcome, _)| outcome)
}

/// [`check_parsed`], also returning the run as a [`Measurement`] when it asked everything
/// a mint asks: both tiers on, and no world check. A later [`mint_measured`] over the
/// same parse can then mint from it instead of running the engine a second time (#1096).
///
/// `None` for any narrower run. Such a run did not measure what a certificate records,
/// so it has nothing for one to reuse.
pub fn check_parsed_measured(
    parsed: &ParsedFile,
    opts: &CheckOptions,
    world: WorldScope,
) -> Result<(Outcome, Option<Measurement>), ValidatorError> {
    let (question, world) = split_options(opts, world)?;
    let (found, dv, res) = check_parsed_with_dict(parsed, opts, &world)?;
    // A world check's findings sit in the same map as the content ones, and a
    // certificate is a statement about content alone. So a run that looked at the
    // world is not something a certificate can be minted from.
    let measurement = (question.want_warnings
        && question.want_fyi
        && matches!(world, WorldScope::None))
    .then(|| Measurement {
        question,
        engine: engine_id(None),
        judged: Arc::downgrade(&parsed.text),
        tally: Tally::of(&found, dv, res),
    });
    let outcome = Outcome {
        findings: found,
        dict_version: dv,
        resolution: res,
        certified: false,
        revalidate_reason: None,
    };
    Ok((outcome, measurement))
}

/// A verdict the rule engine reached on one parse, under a mint's full question.
///
/// It exists so that validate-then-certify runs the engine once (#1096). Its fields are
/// private and only [`check_parsed_measured`] builds one, which it does by running the
/// engine. So holding a `Measurement` means the engine ran, and [`mint_measured`] taking
/// one does not reopen what [`mint`] closed: a caller still cannot hand a mint a count
/// the engine did not produce.
///
/// It names the parse it judged by identity, not by content: a weak reference to that
/// parse's text buffer. So it does not keep the buffer alive, and a parse that merely
/// landed at the same address does not match. (A `ParsedFile` clone shares the buffer, so
/// this stops a measurement being offered for the wrong parse by accident, not by a Rust
/// author set on it. The same limit applies to every guard in this crate.)
#[derive(Debug, Clone)]
pub struct Measurement {
    /// The question that run answered: tiers, edition, decoder, custom dictionary.
    question: Question,
    /// The engine that answered it.
    engine: EngineId,
    /// The parse it judged, as a weak handle on its [`ParsedFile::text`].
    judged: Weak<String>,
    tally: Tally,
}

/// The counts a certificate records, and the edition they were reached under.
#[derive(Debug, Clone, Copy)]
struct Tally {
    dict_version: DictVersion,
    resolution: DictResolution,
    errors: u32,
    warnings: u32,
    fyi: u32,
}

impl Tally {
    fn of(found: &Findings, dict_version: DictVersion, resolution: DictResolution) -> Self {
        Tally {
            dict_version,
            resolution,
            errors: count_of(found, findings::Severity::Error),
            warnings: count_of(found, findings::Severity::Warning),
            fyi: count_of(found, findings::Severity::Fyi),
        }
    }
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
    let full = mint_options(opts);
    let parsed = parse_bytes(bytes, full.encoding).map_err(ValidatorError::from)?;
    let tally = run_engine(&parsed, &full)?;
    seal(bytes, &parsed, &full, tally, checked_at, compat)
}

/// What [`mint_measured`] did: the certificate or the reason there is none, and whether
/// the rule engine ran to reach that verdict.
#[derive(Debug)]
pub struct Minted {
    /// What [`mint`] would have returned for the same bytes and options.
    pub result: Result<Sidecar, MintError>,
    /// `false` when the measurement stood in for the engine, or when the mint failed
    /// before reaching a verdict at all.
    pub engine_ran: bool,
}

/// [`mint`], reusing `measurement` instead of running the engine when it already answers
/// this mint's question for these bytes (#1096).
///
/// `judged` is the parse the caller ran [`check_parsed_measured`] on. The measurement
/// stands in for the engine only if all of these hold:
///
/// * it answered the question this mint asks: both tiers, and the same edition, decoder
///   and custom dictionary;
/// * it came from the engine this mint would stamp;
/// * it was taken on `judged` itself, not on some other parse;
/// * `bytes` parse to a `ParsedFile` equal to `judged`.
///
/// The last condition is what makes the verdict true of `bytes`. A verdict is a function
/// of the parse and the options alone, so an equal parse under the same question gets the
/// same verdict. It also catches a file that changed on disk after it was read. Whenever
/// a condition fails, this mints exactly as [`mint`] does. Either way the stamp is built
/// by the same code from the same counts, so the certificate is the one [`mint`] makes.
pub fn mint_measured(
    bytes: &[u8],
    opts: &CheckOptions,
    checked_at: String,
    compat: Option<String>,
    judged: &ParsedFile,
    measurement: &Measurement,
) -> Minted {
    let full = mint_options(opts);
    let parsed = match parse_bytes(bytes, full.encoding) {
        Ok(p) => p,
        Err(e) => {
            return Minted {
                result: Err(ValidatorError::from(e).into()),
                engine_ran: false,
            };
        }
    };
    let reusable = match split_options(&full, WorldScope::None) {
        Ok((question, _)) => {
            measurement.question == question
                && measurement.engine == engine_id(compat.clone())
                && measurement
                    .judged
                    .upgrade()
                    .is_some_and(|text| Arc::ptr_eq(&text, &judged.text))
                && same_parse(&parsed, judged)
        }
        Err(_) => false,
    };
    let tally = if reusable {
        Ok(measurement.tally)
    } else {
        run_engine(&parsed, &full)
    };
    match tally {
        Ok(tally) => Minted {
            result: seal(bytes, &parsed, &full, tally, checked_at, compat),
            engine_ran: !reusable,
        },
        Err(e) => Minted {
            result: Err(e),
            engine_ran: !reusable,
        },
    }
}

/// The options a mint validates under, whatever the caller asked for.
fn mint_options(opts: &CheckOptions) -> CheckOptions {
    // Both tiers, always. A cert that measured only errors can still be used — it just
    // can't answer a `--show-warnings` request (`TierNotMeasured`) — but there is no
    // reason to mint a weaker cert than we can, and a caller cannot ask us to.
    CheckOptions {
        include_warnings: true,
        include_fyi: true,
        // The mint is CONTENT-only by construction. Even if the caller was doing an
        // on-disk check, its result has no business in a certificate.
        check_files: false,
        ..opts.clone()
    }
}

fn run_engine(parsed: &ParsedFile, full: &CheckOptions) -> Result<Tally, MintError> {
    let (found, dv, res) = check_parsed_with_dict(parsed, full, &WorldScope::None)?;
    Ok(Tally::of(&found, dv, res))
}

/// Do two parses say the same thing? The destructure is exhaustive on purpose, as in
/// [`split_options`]: a field added to `ParsedFile` stops this compiling until someone
/// decides whether it is compared. A field left out here would be one a rule could read
/// and a reused verdict would not cover.
fn same_parse(a: &ParsedFile, b: &ParsedFile) -> bool {
    let ParsedFile {
        text,
        groups,
        group_order,
        group_records,
        raw_lines,
        line_byte_offsets,
        total_lines,
        has_bom,
        total_bytes,
        byte_offsets_source_true,
    } = a;
    *total_bytes == b.total_bytes
        && *total_lines == b.total_lines
        && *has_bom == b.has_bom
        && *byte_offsets_source_true == b.byte_offsets_source_true
        && *group_order == b.group_order
        && *group_records == b.group_records
        && *raw_lines == b.raw_lines
        && *line_byte_offsets == b.line_byte_offsets
        && *groups == b.groups
        && **text == *b.text
}

/// Build the certificate from a verdict. [`mint`] and [`mint_measured`] both end here,
/// which is why a reused verdict mints the same certificate as a fresh one.
fn seal(
    bytes: &[u8],
    parsed: &ParsedFile,
    full: &CheckOptions,
    tally: Tally,
    checked_at: String,
    compat: Option<String>,
) -> Result<Sidecar, MintError> {
    let Tally {
        dict_version: dv,
        resolution: res,
        errors,
        warnings,
        fyi,
    } = tally;
    if errors > 0 {
        return Err(MintError::NotCertifiable { errors });
    }

    let edition = match full.dict_version {
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
        // The custom overlay this verdict was reached against (laterite-dev#568), so a later
        // request supplying a different (or no) dict revalidates rather than trusting a
        // cert minted under another dictionary.
        custom_dict: full.custom_dict.as_ref().map(custom_dict_record),
        // Measured, all three, because the engine ran all three: just now, or in the
        // run a `Measurement` records. This is the only place a ValidationStamp is
        // built from a real engine result.
        errors: TierCoverage::Measured { count: errors },
        warnings: TierCoverage::Measured { count: warnings },
        fyi: TierCoverage::Measured { count: fyi },
    };
    // Reuse the parse we already did to validate — its source-true byte offsets
    // are exactly what the index needs, so certifying does not walk the file a
    // second time (#5, ~14% of a mint on a 25 MB file).
    Sidecar::assemble_from_parsed(bytes, parsed, stamp)
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

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
