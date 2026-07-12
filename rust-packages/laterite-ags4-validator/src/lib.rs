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
//! **Feature-complete.** All rule families are wired into
//! [`rules::run_all`]. The five bundled standard dictionaries (AGS
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
//! - [`is_valid`] — convenience boolean (zero findings).
//! - [`resolve_dict_version`] / [`tran_ags_of`] — exposed so callers
//!   (e.g. the corpus-QA harness) can *report* which edition a file
//!   was judged against without re-implementing the policy.

use std::path::{Path, PathBuf};

/// The validation **engine** version — the identity a `.ags.idx` certificate
/// stamps so a clean verdict is only trusted (validation skipped) when the same
/// engine would produce it. Distinct from any binding's crate version (the Python
/// wheel, the DuckDB extension) so a cert minted on one surface is comparable on
/// another: all bindings record THIS, not their own version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod catalogue;
pub mod error;
pub mod findings;
pub mod fixes;
pub mod parse;
pub mod rules;

pub use catalogue::{RULE_LABELS, rule_metadata_json};
pub use dict::{DictResolution, DictVersion, Dictionary};
pub use error::ValidatorError;
pub use findings::{Finding, Findings};
pub use fixes::{
    Fix, FixKind, FixOutcome, Fixes, SpanEdit, apply_fixes, compute_fixes, fix_document,
    fix_document_selective,
};
// The phf-projected dictionary (`Dictionary`, `DictVersion`, `dictionary_dto`,
// …) moved into the reference leaf (#475 PR2). Re-exported as a module (not
// just the three names above) so every existing `crate::dict::…` /
// `laterite_ags4_validator::dict::…` path throughout this crate + its
// consumers (laterite-py, laterite-node, wasm) keeps resolving unchanged.
pub use laterite_ags4_reference::dict;

/// The bundled AGS4 editions joined with `sep` — the single source for every
/// surface's "expected auto|4.0.3|…" / "pass one of 4.0.3/…" message, so no
/// hand-written list can drift from `DictVersion::ALL`.
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
    /// Override the bundled dictionary with an external `.ags` standard
    /// dictionary. **Deliberately deferred** (O-28): supporting a
    /// runtime-parsed dictionary means abstracting the `'static`
    /// phf-backed [`Dictionary`] over an owned variant, which ripples a
    /// non-`'static` lifetime through every rule module. Setting this
    /// returns [`ValidatorError::BadDict`] (a clear error, never
    /// silent).
    pub custom_dict: Option<PathBuf>,
    /// Include WARNING-severity findings (malformed DICT, nonstandard
    /// abbreviations, unrecognised TRAN_AGS edition, …). On by default at
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
pub fn tran_ags_of(parsed: &parse::ParsedFile) -> Option<String> {
    let tran = parsed.groups.get("TRAN")?;
    let ci = tran.headings.iter().position(|h| h == "TRAN_AGS")?;
    let v = tran.rows.first()?.values.get(ci)?;
    let t = v.trim();
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
    use DictVersion::*;
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
            Some(0) => Ok((V4_0_4, K::GuessedPatch)),     // 4.0 / 4.0.x
            Some(1) => Ok((V4_1_1, K::GuessedPatch)),     // 4.1.x (exact above)
            Some(2) => Ok((V4_2, K::GuessedPatch)),       // 4.2.x
            Some(_) => Ok((dict::FALLBACK, K::Fallback)), // 4.3 / 4.9 …
            None => Ok((V4_0_4, K::GuessedPatch)),        // "4." / "4.x" junk
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
    if let Some(custom) = &opts.custom_dict {
        return Err(ValidatorError::BadDict {
            path: custom.clone(),
            reason: "external --dict override is not implemented; use a bundled \
                     --dict-version (4.0.3/4.0.4/4.1/4.1.1/4.2) or omit it for \
                     TRAN_AGS auto-detection"
                .to_string(),
        });
    }

    let parsed = parse::parse_file_with_encoding(path, opts.encoding)?;
    let (dv, kind) = resolve_dict_version(opts.dict_version, tran_ags_of(&parsed).as_deref())?;
    let (dv, kind, upgraded_from_4_0_3) = guard_4_0_4(dv, kind, &parsed);
    let dict = Dictionary::bundled(dv);

    let mut found: Findings = Findings::new();
    // `path` is owned for this whole fn — pass it so Rule 20's opt-in
    // on-disk half can locate the sibling `FILE/` tree (no lifetime
    // ripple: the borrow is strictly shorter than `path`).
    rules::run_all(&parsed, &dict, opts, Some(path), &mut found);
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

/// `true` iff `check_file` produced zero findings. What the CLI exit
/// code and `ags5db db-to-ags4 --validate` key off.
pub fn is_valid(path: &Path, opts: &CheckOptions) -> Result<bool, ValidatorError> {
    Ok(findings::count(&check_file(path, opts)?) == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use DictVersion::*;

    #[test]
    fn custom_dict_is_rejected() {
        let opts = CheckOptions {
            custom_dict: Some(PathBuf::from("whatever.ags")),
            ..Default::default()
        };
        let err = check_file(Path::new("unused"), &opts).unwrap_err();
        assert!(matches!(err, ValidatorError::BadDict { .. }));
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
}
