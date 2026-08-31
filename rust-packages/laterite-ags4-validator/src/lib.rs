//! Clean-room AGS4 transfer-format validator.
//!
//! Implements the AGS4 numbered rules from the published specification
//! (`reports/AGS 4_1.pdf`, §4.1.1). It is **not** a translation of
//! python-ags4 (LGPL-3.0) — the AGS4 rules themselves are a functional
//! standard (not copyrightable); python-ags4 source was consulted only
//! to understand *which* rules exist and *how* the spec's ambiguous
//! wording is interpreted, never copied. See the crate README and each
//! `rules/*` module header.
//!
//! ## Status
//!
//! **Feature-complete.** All rule families are wired into the rule engine
//! (`rules::run_all`, crate-private — reach it through [`check_parsed`]). The
//! five bundled standard dictionaries (AGS
//! 4.0.3/4.0.4/4.1/4.1.1/4.2) are compiled in (no runtime parse);
//! [`check_file`] **auto-selects** the matching edition from the
//! file's `TRAN_AGS` ([`resolve_dict_version`]) unless an explicit
//! [`CheckOptions::dict_version`] is given (this resolves OBSERVATIONS
//! O-10). One scoped deferral remains: an external `--dict <path>`
//! override (see [`CheckOptions::custom_dict`], O-28).
//!
//! ## Entry points
//!
//! - [`check_file`] — parse + auto-pick the dictionary + run all rules.
//! - [`check_parsed`] — the same rule run over an already-parsed file. The
//!   **only** public way in for a caller holding bytes/text rather than a path,
//!   and the only place that can refuse a request it cannot honestly answer.
//! - [`is_clean`] — convenience boolean (zero findings), named for the
//!   "clean (0 findings)" line the surfaces print. It answers "did the run find
//!   anything", which since #321 is **not** the verdict.
//! - [`verdict::Verdict`] — the verdict, and the only producer of it. Every
//!   surface asks this rather than deriving its own, so `is_valid` and the
//!   exit code cannot disagree (#321).
//! - [`resolve_dict_version`] / [`tran_ags_of`] — exposed so callers
//!   (e.g. the corpus-QA harness) can *report* which edition a file
//!   was judged against without re-implementing the policy.

use std::path::{Path, PathBuf};

/// The validation **engine** version — a hand-bumped semver. Useful for humans,
/// **useless as a cache key**: edit a rule without bumping the crate and this
/// value is unchanged, so a certificate minted by the old engine still looks
/// current. [`ENGINE_FINGERPRINT`] is the value a cert must stamp.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The identity of the *engine that produces verdicts* — a build-time SHA-256
/// over every rule source file plus the dictionary and rules-catalogue data
/// (see `build.rs`). Change a rule, change a dictionary entry, and this changes;
/// forget to bump the crate version and this changes anyway.
///
/// This is what an `.ags.idx` certificate stamps, and what a later run compares
/// against before it trusts a recorded verdict enough to skip re-validating. The
/// question a cert answers is "would *this* engine, over *these* bytes, still say
/// clean?" — and only a fingerprint of the engine's actual inputs can answer it.
pub const ENGINE_FINGERPRINT: &str = env!("LATERITE_ENGINE_FINGERPRINT");

pub mod catalogue;
pub mod error;
pub mod findings;
pub mod fixes;
pub mod parse;
pub mod rules;
pub mod verdict;
pub mod world;

pub use world::WorldScope;

pub use catalogue::{RULE_LABELS, rule_metadata_json};
pub use dict::{DictResolution, DictVersion, Dictionary};
pub use error::ValidatorError;
pub use findings::{Finding, Findings};
pub use fixes::{
    Fix, FixKind, FixOutcome, Fixes, SpanEdit, apply_fixes, compute_fixes, fix_document,
    fix_document_selective,
};
// The phf-projected dictionary (`Dictionary`, `DictVersion`, `dictionary_dto`,
// …) moved into the reference leaf (laterite-dev#475 PR2). Re-exported as a module (not
// just the three names above) so every existing `crate::dict::…` /
// `laterite_ags4_validator::dict::…` path throughout this crate + its
// consumers (laterite-py, laterite-node, wasm) keeps resolving unchanged.
pub use laterite_ags4_reference::dict;
// The Rule 18 effective dictionary (standard ∪ the file's own DICT group) —
// the shared implementation the Rule 7/9, 10a-c and 19b families all consume
// since #777 (O-25/O-29 record the two private copies it replaced). Re-exported
// so surfaces and tests reach it as `laterite_ags4_validator::effective_dict::…`
// without a second dependency on the reference leaf.
pub use laterite_ags4_reference::effective_dict;
// The runtime custom-dictionary overlay (laterite-dev#568): `CustomDict` (parsed once at the
// surface boundary, carried on `CheckOptions::custom_dict`), `parse_dict`, and
// its `DictError`. Re-exported so surfaces build a `CustomDict` through
// `laterite_ags4_validator::overlay::…` without reaching past the validator.
pub use laterite_ags4_reference::overlay;

/// The bundled AGS4 editions joined with `sep` — the single source for every
/// surface's "expected auto|4.0.3|…" / "pass one of 4.0.3/…" message, so no
/// hand-written list can drift from `DictVersion::ALL`.
#[must_use]
pub fn editions_joined(sep: &str) -> String {
    DictVersion::ALL
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(sep)
}

/// Validation options. `Default` = **auto-detect** the dictionary from
/// the file's `TRAN_AGS`, errors only.
#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// `None` ⇒ auto-select the bundled edition from the file's
    /// `TRAN_AGS` (the default — see [`resolve_dict_version`]).
    /// `Some(v)` ⇒ force edition `v`, ignoring `TRAN_AGS`.
    pub dict_version: Option<DictVersion>,
    /// Override the bundled dictionary with a runtime custom dictionary
    /// (laterite-dev#568, O-28). Parsed ONCE at the surface boundary into a
    /// [`overlay::CustomDict`] — a base-resolved sparse overlay (or full
    /// replacement) — so a batch run pays the parse cost once, reused across
    /// every file. `None` ⇒ the bundled path (`dict_version` / `TRAN_AGS`
    /// auto-detect). A malformed dict fails at that boundary
    /// ([`ValidatorError::BadDict`]) before any file is read.
    pub custom_dict: Option<overlay::CustomDict>,
    /// Include WARNING-severity findings (malformed DICT, nonstandard
    /// abbreviations, unrecognised `TRAN_AGS` edition, …). On by default at
    /// the binding layer; `--no-warnings` drops to errors-only.
    pub include_warnings: bool,
    /// Include FYI-severity findings.
    pub include_fyi: bool,
    /// Opt-in: also run Rule 20's **on-disk** half — the sidecar
    /// `FILE/<FILE_FSET>/<FILE_NAME>` tree must exist next to the
    /// `.ags`. **Default `false`**: the data-level Rule 20 is
    /// path-independent and deterministic (what a library validator
    /// and `db-to-ags4 --validate` need); the filesystem stat is a
    /// packaging/QA concern, enabled explicitly (`lat validate
    /// --check-files`) and by the corpus-qa dogfood so it matches
    /// python-ags4's always-on behaviour. std-only — no new dep.
    pub check_files: bool,
    /// Source-file encoding for byte→text decode. Default `UTF_8`
    /// matches `String::from_utf8_lossy`'s historical behaviour.
    /// Set this (or pass `--encoding` to the CLI / `encoding=` to
    /// `compat.check_file`) for files saved in `cp1252`, `latin1`,
    /// etc. — common for legacy delivery files with `°` / `±` /
    /// extended-ASCII in description fields. Decoded with
    /// `encoding_rs`'s lossy mode (undefined bytes → U+FFFD,
    /// matching python-ags4's `errors='replace'`).
    pub encoding: &'static encoding_rs::Encoding,
}

impl Default for CheckOptions {
    fn default() -> Self {
        CheckOptions {
            dict_version: None, // auto-detect from TRAN_AGS
            custom_dict: None,
            include_warnings: false,
            include_fyi: false,
            check_files: false, // path-independent by default (O-27)
            encoding: encoding_rs::UTF_8,
        }
    }
}

/// The file's declared `TRAN_AGS` value (TRAN group, first DATA row,
/// `TRAN_AGS` column), trimmed; `None` if absent/blank. Same
/// resolve-column-by-name + first-DATA-row pattern Rule 11 uses.
#[must_use]
pub fn tran_ags_of(parsed: &parse::ParsedFile) -> Option<String> {
    let tran = parsed.groups.get("TRAN")?;
    let ci = tran.headings.iter().position(|h| h == "TRAN_AGS")?;
    let v = tran.rows.first()?.values.get(ci)?;
    let t = v.slice(tran.text()).trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// Pick the bundled dictionary edition.
///
/// * an explicit `override` always wins;
/// * else exact `TRAN_AGS` match wins (`4.0.3`/`4.0.4`/`4.1`/`4.1.1`/
///   `4.2`);
/// * else the newest bundled patch of the same `major.minor`
///   (`4.0`→4.0.4, `4.1.5`→4.1.1, `4.2.7`→4.2);
/// * else (missing / unparsable / unrecognised 4.x / bare `4`) →
///   [`dict::FALLBACK`] (4.1.1, matched to python-ags4's
///   `LATEST_DICT_VERSION` so dogfood parity reflects real defects);
/// * **AGS 3.x → [`ValidatorError::UnsupportedEdition`]** (we refuse
///   rather than silently validate it against an AGS4 schema; python
///   silently defaults it to 4.1.1 — a deliberate divergence, O-30).
pub fn resolve_dict_version(
    over: Option<DictVersion>,
    tran_ags: Option<&str>,
) -> Result<(DictVersion, DictResolution), ValidatorError> {
    use DictResolution as K;
    use DictVersion::{V4_0_3, V4_0_4, V4_1, V4_1_1, V4_2};
    if let Some(v) = over {
        return Ok((v, K::Forced));
    }
    let Some(t) = tran_ags.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok((dict::FALLBACK, K::Fallback));
    };
    // Exact bundled-edition string wins (keeps python-parity for
    // 4.1/4.1.1/4.2/4.0.3/4.0.4).
    for v in [V4_0_3, V4_0_4, V4_1, V4_1_1, V4_2] {
        if v.as_str() == t {
            return Ok((v, K::ExactTranAgs));
        }
    }
    let mut parts = t.split('.');
    let Some(major) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return Ok((dict::FALLBACK, K::Fallback)); // non-numeric ("banana", "")
    };
    if major == 3 {
        return Err(ValidatorError::UnsupportedEdition {
            found: t.to_string(),
        });
    }
    if major != 4 {
        return Ok((dict::FALLBACK, K::Fallback)); // future 5.x → python-style
    }
    // major == 4, no exact match → newest bundled patch of major.minor.
    // Bare "4" / "4."/"4.x" (major-4, no usable minor) means just
    // "AGS4" — best served by the original/most-common 4.0 line
    // (4.0.4), consistent with how "4.0" resolves. (O-30 #3: 41% of a
    // 12.5k real corpus is bare "4" from producers whose "4.0" files
    // already → 4.0.4; deliberate divergence from python's →4.1.1.)
    // An *explicit* unbundled numeric minor (4.3/4.9) is a real
    // unknown edition → the python-matched 4.1.1 fallback.
    match parts.next() {
        None => Ok((V4_0_4, K::GuessedPatch)), // bare "4"
        Some(m) => match m.parse::<u32>().ok() {
            Some(0) | None => Ok((V4_0_4, K::GuessedPatch)), // 4.0/4.0.x, or "4."/"4.x" junk
            Some(1) => Ok((V4_1_1, K::GuessedPatch)),        // 4.1.x (exact above)
            Some(2) => Ok((V4_2, K::GuessedPatch)),          // 4.2.x
            Some(_) => Ok((dict::FALLBACK, K::Fallback)),    // 4.3 / 4.9 …
        },
    }
}

/// Validate an AGS4 file. Parses it, **auto-selects** the bundled
/// dictionary from its `TRAN_AGS` (unless `opts.dict_version` forces
/// one), then runs every enabled rule. Rule *violations* come back as
/// [`Findings`]; only un-validatable inputs (missing, non-UTF-8, not
/// AGS4, unsupported edition) return `Err`.
pub fn check_file(path: &Path, opts: &CheckOptions) -> Result<Findings, ValidatorError> {
    check_file_with_dict(path, opts).map(|(f, _, _)| f)
}

/// Run the rules over an already-parsed file — **the one door into the engine**.
///
/// Two halves, and the split is the whole point:
///
/// * CONTENT (`rules::run_all`) — a pure function of `parsed`. Cacheable; an
///   `.ags.idx` certificate may stand in for it.
/// * WORLD ([`world::run`]) — reads state outside the bytes (today: Rule 20's
///   sibling `FILE/` tree). Never cacheable, and here it is run *unconditionally*,
///   outside any branch a future certificate-skip could hide behind.
///
/// `opts.check_files` is the request; `world` is the *ability* to honour it. When
/// a caller asks for the on-disk check and supplies no path — every bytes/text
/// read, and wasm always — the honest answer is
/// [`ValidatorError::WorldCheckRequiresSource`], not a clean Rule 20. The engine
/// used to give the clean one.
pub fn check_parsed(
    parsed: &parse::ParsedFile,
    dict: &Dictionary,
    opts: &CheckOptions,
    world: &WorldScope,
) -> Result<Findings, ValidatorError> {
    if opts.check_files && matches!(world, WorldScope::None) {
        return Err(ValidatorError::WorldCheckRequiresSource);
    }
    let mut found: Findings = Findings::new();
    rules::run_all(parsed, dict, opts, &mut found);
    world::run(parsed, world, &mut found);
    Ok(found)
}

/// The world a caller with `source` and `opts` is entitled to look at: the
/// on-disk tree iff they asked for it *and* there is a path to look beside.
/// A caller with no path and `check_files` set gets `Err` — see [`check_parsed`].
fn world_for(opts: &CheckOptions, source: Option<&Path>) -> Result<WorldScope, ValidatorError> {
    match (opts.check_files, source) {
        (true, Some(p)) => Ok(WorldScope::OnDisk(p.to_path_buf())),
        (true, None) => Err(ValidatorError::WorldCheckRequiresSource),
        (false, _) => Ok(WorldScope::None),
    }
}

/// Headings introduced in AGS 4.0.4 that did **not** exist in 4.0.3. The two
/// editions are otherwise identical — same 124 groups, no headings removed;
/// the only other delta is PMTL's parent re-pointing PMTD→PMTG. So a file in
/// the 4.0 line that carries any of these is the one *deterministic* signal
/// that it is really ≥4.0.4. Sourced from the official
/// `Standard_dictionary_v4_0_3`↔`v4_0_4` diff; both editions are frozen, so
/// this list never drifts. (laterite#222 / OBSERVATIONS O-42.)
const HEADINGS_NEW_IN_4_0_4: &[(&str, &str)] = &[
    ("GCHM", "GCHM_DLM"),
    ("GCHM", "GCHM_RTXT"),
    ("LOCA", "LOCA_NATD"),
    ("LOCA", "LOCA_ORCO"),
    ("LOCA", "LOCA_ORID"),
    ("LOCA", "LOCA_ORJO"),
    ("RDEN", "RDEN_IDEN"),
    ("SAMP", "SAMP_RECL"),
];

/// The first heading the file uses that exists only from 4.0.4, if any — the
/// signal that a 4.0-line file should be judged against 4.0.4, not 4.0.3.
fn first_post_4_0_3_heading(parsed: &parse::ParsedFile) -> Option<&'static str> {
    HEADINGS_NEW_IN_4_0_4.iter().find_map(|(grp, hdng)| {
        parsed
            .groups
            .get(*grp)
            .filter(|g| g.headings.iter().any(|h| h == hdng))
            .map(|_| *hdng)
    })
}

/// Content-aware defence on the 4.0 ambiguity (#222 / O-42): if auto-resolution
/// landed on **4.0.3** but the file uses a heading introduced in 4.0.4, judge it
/// against **4.0.4** instead — its newer vocabulary is then not false-flagged as
/// non-standard (Rule 9) and PMTL uses its 4.0.4 parent (PMTG, not PMTD). The
/// ambiguous `"4.0"`/`"4"` already guess 4.0.4 (O-30); this additionally catches
/// a file *mislabeled* `"4.0.3"`. An explicit `--dict-version` (`Forced`) is
/// never overridden — that edition is the caller's deliberate choice. The third
/// return value is the triggering heading (so the caller can emit a transparency
/// FYI), `Some` iff an upgrade happened.
fn guard_4_0_4(
    dv: DictVersion,
    kind: DictResolution,
    parsed: &parse::ParsedFile,
) -> (DictVersion, DictResolution, Option<&'static str>) {
    if dv == DictVersion::V4_0_3 && kind != DictResolution::Forced {
        if let Some(h) = first_post_4_0_3_heading(parsed) {
            return (DictVersion::V4_0_4, DictResolution::GuessedPatch, Some(h));
        }
    }
    (dv, kind, None)
}

/// Like [`check_file`] but also returns the bundled edition the file
/// was judged against **and how it was resolved** (forced / exact
/// `TRAN_AGS` / guessed-patch / fallback). Single parse — used by the
/// corpus-QA harness to record *why* each file was checked against a
/// given schema (and to tell a genuine edition from the O-30
/// fallback), without re-parsing (matters on 420 MB deliveries).
pub fn check_file_with_dict(
    path: &Path,
    opts: &CheckOptions,
) -> Result<(Findings, DictVersion, DictResolution), ValidatorError> {
    let parsed = parse::parse_file_with_encoding(path, opts.encoding)?;
    // We have a path, so `--check-files` is answerable: Rule 20's on-disk half
    // can locate the sibling `FILE/` tree.
    let world = world_for(opts, Some(path))?;
    check_parsed_with_dict(&parsed, opts, &world)
}

/// **Pick the dictionary, then judge the file against it** — the whole of what
/// `check_file_with_dict` does once the bytes are parsed, and the door every
/// modality goes through.
///
/// It exists because "resolve the edition, then run the rules" is not two steps a
/// caller can be trusted to assemble. It is four: resolve `TRAN_AGS`, apply the
/// 4.0.3→4.0.4 content guard (`guard_4_0_4`, O-42), run the rules, and emit the
/// transparency FYI that says the file's declared edition and its actual vocabulary
/// disagree. Every caller that hand-assembled it got the same two right and the same
/// two wrong: `laterite-py`, `laterite-node` and the wasm surface each resolved and
/// ran, and each skipped the guard. So a file whose `TRAN_AGS` said 4.0.3 while it
/// used a 4.0.4-only heading was judged against **4.0.4 from a path and 4.0.3 from
/// bytes** — same file, same flags, two dictionaries, and two phantom Rule 9
/// findings on the bytes side. Not a knob that disagreed; an answer that did.
pub fn check_parsed_with_dict(
    parsed: &parse::ParsedFile,
    opts: &CheckOptions,
    world: &WorldScope,
) -> Result<(Findings, DictVersion, DictResolution), ValidatorError> {
    // The custom-dictionary path (laterite-dev#568): the base + delta are already fixed on
    // the `CustomDict` (parsed once at the surface boundary), so there is no
    // `TRAN_AGS` resolution and no 4.0.3→4.0.4 content guard — the caller chose
    // the base deliberately. `build_delta` re-derives the stack-local overlay the
    // layered `Dictionary` borrows for this one validation.
    if let Some(custom) = &opts.custom_dict {
        let delta = custom.build_delta().map_err(|e| ValidatorError::BadDict {
            path: PathBuf::from(&custom.name),
            reason: e.to_string(),
        })?;
        let dict = Dictionary::layered(&delta);
        let mut found = check_parsed(parsed, &dict, opts, world)?;
        // Honour + warn (laterite-dev#568 §3): the overlay takes effect, but every override
        // of a STANDARD group/heading is surfaced, so a bespoke dictionary can
        // never silently reshape the standard schema. Which TIER each override
        // lands in is the #321 surprise test — see the function.
        emit_override_findings(custom, &delta, opts, &mut found);
        return Ok((found, custom.base_version, custom.resolution));
    }

    let (dv, kind) = resolve_dict_version(opts.dict_version, tran_ags_of(parsed).as_deref())?;
    let (dv, kind, upgraded_from_4_0_3) = guard_4_0_4(dv, kind, parsed);
    let dict = Dictionary::bundled(dv);

    let mut found = check_parsed(parsed, &dict, opts, world)?;
    // Transparency FYI (#222 / O-42): if we resolved UP from the file's
    // declared 4.0.3 to 4.0.4 because it uses 4.0.4-only vocabulary, say so —
    // TRAN_AGS and the file's content disagree, and the user may want to bump
    // TRAN_AGS (or drop the heading). Shown only with `--show-fyi`.
    if let Some(h) = upgraded_from_4_0_3 {
        findings::add_at(
            &mut found,
            "FYI",
            None,
            "TRAN",
            format!(
                "TRAN_AGS declares 4.0.3 but the file uses {h:?}, a heading \
                 introduced in 4.0.4 — validated against 4.0.4. Set TRAN_AGS to \
                 4.0.4 (or remove the heading) to make the file self-consistent."
            ),
            findings::Location::default(),
            findings::Severity::Fyi,
        );
    }
    Ok((found, dv, kind))
}

/// Surface each override a custom overlay makes to a STANDARD group/heading
/// (laterite-dev#568 §3, honour + warn). Only overlays (not full replacements — a
/// replacement is declared, not an override) and only groups/headings the base
/// actually defines.
///
/// **Two tiers, by the #321 rule: a WARNING predicts a downstream *surprise*.**
/// Re-parenting and KEY demotion both change row identity silently, so they stay
/// WARNING. A plain type/status override changes nothing the caller did not ask
/// for in the file they wrote — it is announced and honoured, with no surprise
/// left over — so it is an FYI.
///
/// Gated on the tier flags, which it did not use to be: `emit_override_warnings`
/// took no `CheckOptions` and fired unconditionally, so `--no-warnings` (whose
/// whole contract is "errors only") did not suppress these. The in-crate tests
/// asserted them under `CheckOptions::default()` — `include_warnings: false` —
/// which is what a gate looks like when it isn't there. It matters more now:
/// the FYI tier is opt-in, and an FYI nobody can turn off is not one.
fn emit_override_findings(
    custom: &overlay::CustomDict,
    delta: &overlay::OwnedDelta,
    opts: &CheckOptions,
    found: &mut Findings,
) {
    if !custom.fall_through {
        return; // a full replacement redefines the schema wholesale, by design
    }
    if !opts.include_warnings && !opts.include_fyi {
        return; // nothing this function emits is error-tier
    }
    let base = Dictionary::bundled(custom.base_version);

    // Re-parented standard groups. A SURPRISE: Rule 10c reads parentage, so a
    // row's relational identity changes without the file saying anything.
    let mut groups: Vec<&String> = delta.groups.keys().collect();
    groups.sort();
    for code in groups {
        if let Some(bg) = base.group(code) {
            let meta = &delta.groups[code];
            if meta.parent != bg.parent && opts.include_warnings {
                findings::add_at(
                    found,
                    "DICT",
                    None,
                    code,
                    format!(
                        "custom dictionary re-parents standard group {code} from \
                         {:?} to {:?} — honoured, but it reshapes the standard tree.",
                        bg.parent, meta.parent
                    ),
                    findings::Location::default(),
                    findings::Severity::Warning,
                );
            }
        }
    }

    // Overridden standard headings (type/status), KEY demotion loudest.
    let mut keys: Vec<&String> = delta.headings.keys().collect();
    keys.sort();
    for key in keys {
        let Some((group, heading)) = key.split_once('\u{1f}') else {
            continue;
        };
        let Some(bh) = base.heading(group, heading) else {
            continue; // a brand-new heading is an addition, not an override
        };
        let over = &delta.headings[key];
        let key_demotion = bh.status.contains("KEY") && !over.status.contains("KEY");
        if key_demotion {
            // A SURPRISE: KEY is what makes a row identifiable, so Rules 10a/10c
            // start answering differently about rows nobody edited.
            if opts.include_warnings {
                findings::add_at(
                    found,
                    "DICT",
                    None,
                    group,
                    format!(
                        "custom dictionary demotes standard KEY heading {group}/{heading} \
                         to status {:?} — honoured, but it changes row identity.",
                        over.status
                    ),
                    findings::Location::default(),
                    findings::Severity::Warning,
                );
            }
        } else if bh.ags_type != over.ags_type || bh.status != over.status {
            // NOT a surprise, so FYI since #321: the caller declared this
            // override in the dictionary they passed, it is honoured exactly as
            // declared, and no downstream consumer receives anything other than
            // what the file says. Worth being able to see; not worth interrupting.
            // Its own label, so the `DICT` bucket stays warning-pure — the same
            // reason `FYI (Related to Rule N)` is separate from the rule's own key.
            if opts.include_fyi {
                findings::add_at(
                    found,
                    "FYI (Related to DICT)",
                    None,
                    group,
                    format!(
                        "custom dictionary overrides standard heading {group}/{heading} \
                         ({}/{} → {}/{}) — honoured.",
                        bh.ags_type, bh.status, over.ags_type, over.status
                    ),
                    findings::Location::default(),
                    findings::Severity::Fyi,
                );
            }
        }
    }
}

/// `true` iff `check_file` produced zero findings — across whichever tiers
/// `opts` asked for, so an errors-only `opts` calls a warning-carrying file
/// clean. "Clean (0 findings)" is what every surface's report already prints;
/// this is that sentence as a boolean. What validate-on-convert callers key off.
///
/// **Not the verdict.** It was called `is_valid` until #321, when the two
/// questions came apart: a warning is now reported without failing, so this and
/// [`verdict::Verdict::is_valid`] disagree on the same file. One name for both
/// was harmless while they coincided and is a trap now — reach for `Verdict`
/// when you mean "did it pass", and this when you mean "did the run find
/// anything".
pub fn is_clean(path: &Path, opts: &CheckOptions) -> Result<bool, ValidatorError> {
    Ok(findings::count(&check_file(path, opts)?) == 0)
}

/// Renamed to [`is_clean`] — see it for why.
///
/// Kept as a delegating alias rather than deleted so the rename reaches a
/// crates.io consumer as a **warning naming its replacement**, not as a build
/// failure they have to diagnose. That matters more than usual here: the two
/// candidate replacements answer different questions now, so a consumer must
/// choose, and a hard break gives them nothing to choose from. Inventoried in
/// the reliquary for deletion in the release that bumps the engine tier.
#[deprecated(
    since = "0.10.0",
    note = "renamed: use `is_clean` for \"did the run find anything\", or \
            `verdict::Verdict::is_valid` for the verdict — since #321 they differ"
)]
pub fn is_valid(path: &Path, opts: &CheckOptions) -> Result<bool, ValidatorError> {
    is_clean(path, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use DictVersion::*;

    #[test]
    fn custom_dict_is_honoured_not_rejected() {
        // The laterite-dev#568 reversal of the old refusal: a valid `--dict` overlay resolves
        // to its detected base and runs the rules against the layered dictionary,
        // rather than short-circuiting to BadDict.
        let dict_json = br#"{"groups":{"TEST":{"parent":"SAMP","headings":[
            {"name":"SAMP_ID","type":"ID","status":"KEY"},
            {"name":"TEST_VAL","type":"2DP","status":"REQUIRED"}
        ]}}}"#;
        let custom = overlay::parse_dict(
            dict_json,
            overlay::DictFormat::Json,
            encoding_rs::UTF_8,
            overlay::BaseSpec::Auto,
            "test.json",
        )
        .expect("custom dict parses");
        let opts = CheckOptions {
            custom_dict: Some(custom),
            ..Default::default()
        };
        let pf = parse::parse_str(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        )
        .expect("parses");
        let (_found, dv, res) =
            check_parsed_with_dict(&pf, &opts, &WorldScope::None).expect("custom dict runs");
        assert_eq!(dv, V4_2, "additive dict detects the latest base");
        assert_eq!(res, DictResolution::StructuralBase);
    }

    #[test]
    fn editions_joined_lists_the_bundled_editions() {
        let s = editions_joined("|");
        assert!(s.contains("4.0.3") && s.contains("4.2"), "{s}");
        assert!(s.contains('|'), "{s}");
        assert_eq!(s.split('|').count(), DictVersion::ALL.len());
    }

    /// Both tiers on — what a `lat validate --dict … --show-fyi` run asks for.
    /// The tests below used `Default` (errors only) and still saw these
    /// findings, which is how the missing tier gate stayed invisible; asking
    /// explicitly is what makes the gate's own test below mean something.
    fn overlay_opts(custom: overlay::CustomDict) -> CheckOptions {
        CheckOptions {
            custom_dict: Some(custom),
            include_warnings: true,
            include_fyi: true,
            ..Default::default()
        }
    }

    /// An overlay that re-parents a standard group and demotes two standard KEYs
    /// WARNs on each (honour + warn, laterite-dev#568) — both are downstream surprises, which
    /// is what keeps them in the warning tier under #321.
    ///
    /// This used to claim `SAMP_TOP` was "a standard non-KEY heading" being
    /// retyped, and asserted only that some finding mentioned it. Base `SAMP_TOP`
    /// is `2DP/KEY`, so it is a second KEY DEMOTION and the assertion was passing
    /// on the demotion message — the retype branch was never reached here at all.
    /// `override_warnings_distinguish_demote_from_type_and_status_change` below is
    /// what actually covers it, on LOCA.
    #[test]
    fn override_warnings_flag_reparent_and_two_key_demotions() {
        let dict_json = br#"{"groups":{"SAMP":{"parent":"PROJ","headings":[
            {"name":"SAMP_ID","type":"ID","status":"REQUIRED"},
            {"name":"SAMP_TOP","type":"X","status":"REQUIRED"}
        ]}}}"#;
        let custom = overlay::parse_dict(
            dict_json,
            overlay::DictFormat::Json,
            encoding_rs::UTF_8,
            overlay::BaseSpec::Auto,
            "test.json",
        )
        .expect("custom dict parses");
        let opts = overlay_opts(custom);
        let pf = parse::parse_str(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        )
        .expect("parses");
        let (found, _dv, _res) =
            check_parsed_with_dict(&pf, &opts, &WorldScope::None).expect("runs");
        let dict = found.get("DICT").expect("DICT override warnings");
        assert!(
            dict.iter()
                .any(|f| f.desc.contains("re-parents") && f.desc.contains("SAMP")),
            "re-parent warning missing: {dict:?}"
        );
        // BOTH headings are KEY in the base, so both demote. Named separately so
        // a regression that drops one is not hidden by the other.
        for hd in ["SAMP_ID", "SAMP_TOP"] {
            assert!(
                dict.iter()
                    .any(|f| f.desc.contains("demotes") && f.desc.contains(hd)),
                "KEY-demotion warning missing for {hd}: {dict:?}"
            );
        }
        // Every finding here is a surprise, so the bucket is warning-pure and can
        // be asserted whole — a demotion mis-tiered into FYI fails this.
        assert!(
            dict.iter()
                .all(|f| f.severity == findings::Severity::Warning),
            "the DICT bucket must be warning-pure: {dict:?}"
        );
        assert!(
            !found.contains_key("FYI (Related to DICT)"),
            "nothing here is a plain retype, so the FYI bucket must be absent: {:?}",
            found.get("FYI (Related to DICT)")
        );
    }

    /// The tier flags reach these findings at all — which they did not until
    /// #321. `--no-warnings` promises errors only, and an opt-in FYI nobody can
    /// turn off is not opt-in. Asserted at three settings because the interesting
    /// failure is a gate that is present but reads the wrong flag.
    #[test]
    fn override_findings_honour_the_tier_flags() {
        // One overlay producing BOTH tiers, so each flag can be shown to move its
        // own tier and only its own: SAMP_ID KEY→REQUIRED is a demotion
        // (WARNING), LOCA_TYPE PA/OTHER→X/OTHER is a plain retype of a non-KEY
        // heading (FYI). A single-tier fixture would let a gate reading the wrong
        // flag pass half the assertions.
        let dict_json = br#"{"groups":{
            "SAMP":{"parent":"LOCA","headings":[
                {"name":"SAMP_ID","type":"ID","status":"REQUIRED"}
            ]},
            "LOCA":{"parent":"PROJ","headings":[
                {"name":"LOCA_TYPE","type":"X","status":"OTHER"}
            ]}
        }}"#;
        let pf = parse::parse_str(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        )
        .expect("parses");
        let run = |warnings: bool, fyi: bool| {
            let custom = overlay::parse_dict(
                dict_json,
                overlay::DictFormat::Json,
                encoding_rs::UTF_8,
                overlay::BaseSpec::Auto,
                "test.json",
            )
            .expect("custom dict parses");
            let opts = CheckOptions {
                custom_dict: Some(custom),
                include_warnings: warnings,
                include_fyi: fyi,
                ..Default::default()
            };
            let (found, _dv, _res) =
                check_parsed_with_dict(&pf, &opts, &WorldScope::None).expect("runs");
            (
                found.contains_key("DICT"),
                found.contains_key("FYI (Related to DICT)"),
            )
        };

        // Errors only — the `--no-warnings` contract. Neither tier may appear.
        assert_eq!(
            run(false, false),
            (false, false),
            "errors-only leaked a tier"
        );
        // The default: warnings shown, FYI still opt-in.
        assert_eq!(run(true, false), (true, false), "default tier set wrong");
        // `--show-fyi` alone, which is also what compat runs.
        assert_eq!(run(false, true), (false, true), "fyi requested alone");
    }

    #[test]
    fn override_warnings_distinguish_demote_from_type_and_status_change() {
        // Re-declare three standard LOCA headings, each a DIFFERENT single-facet
        // change, so every branch of the override-warning selector is pinned:
        //   LOCA_ID   KEY/ID  → KEY/X      : KEY retained + type changed ⇒
        //                                     "overrides", NOT "demotes".
        //   LOCA_TYPE OTHER/PA → OTHER/X   : type-only change  ⇒ "overrides".
        //   LOCA_REM  OTHER/X  → REQUIRED/X: status-only change ⇒ "overrides".
        let dict_json = br#"{"groups":{"LOCA":{"parent":"PROJ","headings":[
            {"name":"LOCA_ID","type":"X","status":"KEY"},
            {"name":"LOCA_TYPE","type":"X","status":"OTHER"},
            {"name":"LOCA_REM","type":"X","status":"REQUIRED"}
        ]}}}"#;
        let custom = overlay::parse_dict(
            dict_json,
            overlay::DictFormat::Json,
            encoding_rs::UTF_8,
            overlay::BaseSpec::Auto,
            "test.json",
        )
        .expect("custom dict parses");
        let opts = overlay_opts(custom);
        let pf = parse::parse_str(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        )
        .expect("parses");
        let (found, _dv, _res) =
            check_parsed_with_dict(&pf, &opts, &WorldScope::None).expect("runs");
        // Every branch here is an "overrides" line, so all three land in the FYI
        // bucket now — and the WARNING bucket must be absent entirely, which is
        // the assertion that would catch a demotion applied to the wrong branch.
        assert!(
            !found.contains_key("DICT"),
            "no surprise here, so nothing may warn: {:?}",
            found.get("DICT")
        );
        let fyi = found
            .get("FYI (Related to DICT)")
            .expect("the override FYI bucket");
        let has = |needle: &str, hd: &str| {
            fyi.iter()
                .any(|f| f.desc.contains(needle) && f.desc.contains(hd))
        };
        // KEY retained ⇒ an "overrides" line, and explicitly NOT a "demotes" one.
        assert!(
            has("overrides", "LOCA_ID"),
            "LOCA_ID type-change override: {fyi:?}"
        );
        assert!(
            !has("demotes", "LOCA_ID"),
            "LOCA_ID must not read as a demotion: {fyi:?}"
        );
        // Single-facet type-only and status-only changes are each still reported.
        assert!(
            has("overrides", "LOCA_TYPE"),
            "LOCA_TYPE type-only override: {fyi:?}"
        );
        assert!(
            has("overrides", "LOCA_REM"),
            "LOCA_REM status-only override: {fyi:?}"
        );
    }

    fn r(t: &str) -> DictVersion {
        resolve_dict_version(None, Some(t)).expect("ok").0
    }

    #[test]
    fn resolve_exact_editions_are_python_parity() {
        assert_eq!(r("4.0.3"), V4_0_3);
        assert_eq!(r("4.0.4"), V4_0_4);
        assert_eq!(r("4.1"), V4_1); // exact — NOT 4.1.1 (python-parity)
        assert_eq!(r("4.1.1"), V4_1_1);
        assert_eq!(r("4.2"), V4_2);
    }

    #[test]
    fn resolve_guesses_newest_patch_of_major_minor() {
        // Deliberate divergence from python ("4.0" → 4.0.3 there): we
        // pick the newest bundled 4.0 patch (O-30).
        assert_eq!(r("4.0"), V4_0_4);
        assert_eq!(r("4.0.9"), V4_0_4);
        assert_eq!(r("4.1.5"), V4_1_1);
        assert_eq!(r("4.2.7"), V4_2);
        // O-30 #3: bare "4" / "4."/"4.x" (no usable minor) → the 4.0
        // line, NOT the 4.1.1 fallback (python → 4.1.1).
        assert_eq!(r("4"), V4_0_4);
        assert_eq!(r("4."), V4_0_4);
        assert_eq!(r("4.x"), V4_0_4);
    }

    #[test]
    fn guard_upgrades_4_0_3_to_4_0_4_on_a_4_0_4_only_heading() {
        // SAMP_RECL is new in 4.0.4 (absent from 4.0.3) — its presence means
        // the file is really ≥4.0.4 (#222 / O-42).
        let with = parse::parse_str(
            "\"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_RECL\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"BH1\",\"0.50\"\r\n",
        )
        .unwrap();
        assert_eq!(first_post_4_0_3_heading(&with), Some("SAMP_RECL"));
        // an exact "4.0.3" auto-resolution is upgraded to 4.0.4, naming the heading
        let (dv, _, trig) = guard_4_0_4(V4_0_3, DictResolution::ExactTranAgs, &with);
        assert_eq!(dv, V4_0_4);
        assert_eq!(trig, Some("SAMP_RECL"));
        // but a *forced* 4.0.3 (--dict-version) is the caller's choice — kept
        let (dvf, _, trigf) = guard_4_0_4(V4_0_3, DictResolution::Forced, &with);
        assert_eq!(dvf, V4_0_3);
        assert!(trigf.is_none());
    }

    #[test]
    fn guard_leaves_editions_alone_without_a_4_0_4_heading() {
        let plain = parse::parse_str(
            "\"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\"\r\n\
             \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\"DATA\",\"BH1\",\"0.50\"\r\n",
        )
        .unwrap();
        assert!(first_post_4_0_3_heading(&plain).is_none());
        // 4.0.3 with no 4.0.4 vocabulary stays 4.0.3 (no upgrade, no FYI)
        assert_eq!(
            guard_4_0_4(V4_0_3, DictResolution::ExactTranAgs, &plain),
            (V4_0_3, DictResolution::ExactTranAgs, None)
        );
        // a higher edition is never touched, heading or not
        let (dv2, _, _) = guard_4_0_4(V4_2, DictResolution::ExactTranAgs, &plain);
        assert_eq!(dv2, V4_2);
    }

    #[test]
    fn resolve_falls_back_to_python_latest_4_1_1() {
        // NB: bare "4" is NOT here — it now resolves to the 4.0 line
        // (O-30 #3). Only truly unknown editions fall back.
        for t in ["", "   ", "banana", "4.3", "4.9", "5.0", "v4.2"] {
            assert_eq!(r(t), dict::FALLBACK, "input {t:?}");
        }
        assert_eq!(resolve_dict_version(None, None).unwrap().0, dict::FALLBACK);
        assert_eq!(dict::FALLBACK, V4_1_1);
    }

    #[test]
    fn resolve_ags3_is_hard_error() {
        for t in ["3", "3.1", "3.1.1"] {
            assert!(matches!(
                resolve_dict_version(None, Some(t)),
                Err(ValidatorError::UnsupportedEdition { .. })
            ));
        }
    }

    #[test]
    fn explicit_override_always_wins() {
        // Even an AGS3 / nonsense TRAN_AGS is ignored when forced.
        assert_eq!(
            resolve_dict_version(Some(V4_2), Some("3.1")).unwrap().0,
            V4_2
        );
        assert_eq!(
            resolve_dict_version(Some(V4_0_3), Some("4.2")).unwrap().0,
            V4_0_3
        );
    }

    #[test]
    fn resolution_kind_is_reported() {
        use DictResolution::*;
        // (version, how-resolved) — lets the harness tell a genuine
        // edition from the O-30 fallback without re-deriving policy.
        assert_eq!(
            resolve_dict_version(None, Some("4.2")).unwrap(),
            (V4_2, ExactTranAgs)
        );
        assert_eq!(
            resolve_dict_version(None, Some("4.1.5")).unwrap(),
            (V4_1_1, GuessedPatch)
        );
        assert_eq!(
            resolve_dict_version(None, Some("4")).unwrap(),
            (V4_0_4, GuessedPatch)
        );
        assert_eq!(
            resolve_dict_version(None, None).unwrap(),
            (dict::FALLBACK, Fallback)
        );
        assert_eq!(
            resolve_dict_version(Some(V4_0_3), Some("4.2")).unwrap(),
            (V4_0_3, Forced)
        );
    }

    #[test]
    fn tran_ags_of_reads_the_file_value() {
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_ISNO\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"1\",\"4.0\"\r\n";
        let pf = parse::parse_str(src).unwrap();
        assert_eq!(tran_ags_of(&pf).as_deref(), Some("4.0"));
        // No TRAN group → None (→ FALLBACK at resolve time).
        let pf2 = parse::parse_str(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        )
        .unwrap();
        assert_eq!(tran_ags_of(&pf2), None);
    }

    /// The AGS4 file the pathless tests validate: one FILE row declaring an
    /// attachment. Rule 20's CONTENT half is satisfied (FS1 *is* defined in the
    /// FILE group); only its WORLD half — does `FILE/FS1/photo.jpg` exist? —
    /// has anything left to say.
    const WITH_ATTACHMENT: &str = "\"GROUP\",\"LOCA\"\r\n\
         \"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
         \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
         \"DATA\",\"BH1\",\"FS1\"\r\n\r\n\
         \"GROUP\",\"FILE\"\r\n\
         \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
         \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
         \"DATA\",\"FS1\",\"photo.jpg\"\r\n";

    #[test]
    fn check_files_without_a_path_refuses_instead_of_reporting_clean() {
        // THE BUG, pinned by its output value. A caller asks for the on-disk FILE/
        // check but holds bytes, not a path — so there is no directory to look
        // beside. The old engine dropped the request and returned Ok with ZERO
        // Rule 20 findings: a clean bill of health for a check it never ran, on
        // every bytes/text read, and in the browser always. No certificate needed.
        let pf = parse::parse_str(WITH_ATTACHMENT).expect("parses");
        let dict = Dictionary::bundled(V4_2);
        let opts = CheckOptions {
            check_files: true,
            ..Default::default()
        };

        let err = check_parsed(&pf, &dict, &opts, &WorldScope::None)
            .expect_err("a world check with no world must not succeed");
        assert!(matches!(err, ValidatorError::WorldCheckRequiresSource));
        assert_eq!(err.kind(), "world_check_requires_source");
        assert_eq!(err.exit_code(), 5);
    }

    #[test]
    fn the_same_request_with_a_path_actually_runs_the_world_check() {
        // The other half of the proof: the refusal above is not the engine being
        // unable to do the check — hand it a path and Rule 20 fires. What changed
        // is that "I can't answer" no longer looks identical to "nothing wrong".
        let pf = parse::parse_str(WITH_ATTACHMENT).expect("parses");
        let dict = Dictionary::bundled(V4_2);
        let opts = CheckOptions {
            check_files: true,
            ..Default::default()
        };
        let tmp = tempfile::tempdir().expect("tempdir");
        let ags = tmp.path().join("site.ags"); // no FILE/ tree beside it

        let found = check_parsed(&pf, &dict, &opts, &WorldScope::OnDisk(ags))
            .expect("a path makes the question answerable");
        assert!(
            found.contains_key("AGS Format Rule 20"),
            "missing FILE/ tree must flag Rule 20: {found:?}"
        );
    }

    #[test]
    fn content_only_runs_are_unaffected() {
        // Default opts (check_files off) + no world: the everyday library call.
        // Rule 20's CONTENT half is happy, and nothing touches the filesystem.
        let pf = parse::parse_str(WITH_ATTACHMENT).expect("parses");
        let dict = Dictionary::bundled(V4_2);
        let found = check_parsed(&pf, &dict, &CheckOptions::default(), &WorldScope::None)
            .expect("content-only always answerable");
        assert!(
            !found.contains_key("AGS Format Rule 20"),
            "content-only must stay path-independent: {found:?}"
        );
    }

    #[test]
    fn the_engine_fingerprint_identifies_the_engine_not_the_crate_version() {
        // What a certificate stamps. 64 bits of the SHA-256 over the rule sources
        // + the bundled dictionary — so a rule edit that forgets to bump the crate
        // version still invalidates every cert minted by the old engine.
        assert_eq!(ENGINE_FINGERPRINT.len(), 16, "{ENGINE_FINGERPRINT}");
        assert!(
            ENGINE_FINGERPRINT
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "must be lowercase hex: {ENGINE_FINGERPRINT}"
        );
        assert_ne!(
            ENGINE_FINGERPRINT, VERSION,
            "the engine's identity is not the crate's semver"
        );
    }
}
