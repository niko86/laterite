//! The host-agnostic AGS4 emit orchestrator.
//!
//! Turns per-group cell data (typed from frames/Arrow, or strings from
//! browser JSON) into valid AGS4 bytes:
//!
//!   1. resolve each heading's UNIT/TYPE — **hybrid**: the caller's
//!      explicit value wins, else the per-edition standard dictionary fills,
//!      else `""` / `"X"`;
//!   2. format each cell — typed values via `laterite_ags4_types::ags4_str` (the
//!      canonical AGS4 string per type), string values verbatim (so the
//!      *mode* below is the single owner of any canonicalisation);
//!   3. `write_ags4` the sections;
//!   4. apply the chosen [`EmitMode`] — Strict rejects bad output, Report
//!      returns it with findings, `AutoFix` applies the *safe* mechanical
//!      fixes (the same machinery the web "fix-all-safe" button uses) and
//!      returns the compliant-where-fixable bytes plus residual findings.
//!
//! Steps 1–3 are pure formatting; step 4 reuses the validator's shipped
//! parse / `run_all` / `compute_fixes` / `apply_fixes` — no new fix logic.

use laterite_ags4_parse::builder::ParsedFileBuilder;
use laterite_ags4_types::{Cell, ags4_str};
use laterite_ags4_validator::dict::Dictionary;
use laterite_ags4_validator::findings::{Findings, Severity};
use laterite_ags4_validator::fixes::{Fix, FixRisk, apply_fixes, compute_fixes};
use laterite_ags4_validator::parse::{ParsedFile, parse_bytes};
use laterite_ags4_validator::{
    CheckOptions, DictVersion, ValidatorError, WorldScope, check_parsed,
};
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::error::EmitError;
use crate::writer::{EmitGroup, write_section_recorded};

/// One group's data to emit. `units` / `types` are optional per-heading
/// overrides — `None` (or a blank entry) means "fill from the dictionary".
/// `rows` cells are [`Cell`]s: typed (`Int` / `Float` / `Bool` / `Null`)
/// from frames or Arrow, or `Text` from browser JSON — deliberately not
/// `serde_json::Value`, whose feature-unified footprint dominated
/// `build_ags4`'s peak (#790; `ags-wiki/design/dec-emit-cell-representation.md`).
pub struct GroupInput {
    pub code: String,
    pub headings: Vec<String>,
    pub units: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub rows: Vec<Vec<Cell>>,
}

/// What to do about the validity of the generated output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmitMode {
    /// Reject (`Err(EmitError::Invalid)`) if the output violates any
    /// error-severity AGS4 rule. A hard gate — nothing is emitted.
    Strict,
    /// Emit unmodified; return the findings for the caller to act on.
    Report,
    /// Emit, then apply the *safe* mechanical fixes and return the
    /// compliant-where-fixable bytes + whatever findings remained. The
    /// default — fits "just give me valid AGS4 from my data".
    #[default]
    AutoFix,
}

/// The transmission a file represents — the caller's half of a synthesised TRAN.
///
/// Lives here, the lowest crate that needs it, so `laterite-ags4-merge` and the
/// emit path describe a transmission with ONE type rather than two that drift.
/// `merge` re-exports it.
///
/// **The dictionary splits TRAN's headings three ways, and this type encodes
/// that split rather than documenting it.**
///
/// * `TRAN_ISNO` (KEY), `TRAN_DATE`, `TRAN_PROD`, `TRAN_RECV`, `TRAN_STAT` are
///   REQUIRED *and* authorial — no engine can know who sent what to whom. They
///   are constructor arguments, so a stamp missing any of them cannot be built.
///   That matters: they are REQUIRED headings, so a partial stamp doesn't just
///   look thin, it trips Rule 10b on every cell it leaves empty. The predecessor
///   of this type took five `Option`s, demanded only two, and let the other three
///   default to `""` — shipping exactly that half-true record.
/// * `TRAN_AGS`, `TRAN_DLIM`, `TRAN_RCON` are REQUIRED-or-OTHER but *derivable*:
///   they describe the syntax of the file the emitter is writing. They are absent
///   from this type on purpose. A caller-supplied value could only contradict the
///   bytes, and "the delimiter is `|`" is not a fact anyone should have to repeat
///   to the thing that chose it. `synth_tran` fills them.
/// * `TRAN_DESC` and `TRAN_REM` are OTHER and authorial — genuinely optional, so
///   they are builder methods rather than constructor arguments.
///
/// `FILE_FSET` is deliberately NOT exposed. It references an associated file set
/// (Rule 20), and offering it without the `FILE` group machinery would let a
/// caller mint a reference to nothing — inventing again, one heading further out.
#[derive(Debug, Clone, Default)]
pub struct TranStamp {
    pub isno: String,
    pub date: String,
    pub prod: String,
    pub recv: String,
    pub stat: String,
    /// The AGS edition. Engine-derived: left empty by `new`, filled by
    /// `synth_tran` from the dictionary in force. `merge` sets it directly
    /// because it resolves the edition from the newest input file.
    pub ags: String,
    /// `TRAN_DESC` — optional, authorial.
    pub desc: Option<String>,
    /// `TRAN_REM` — optional, authorial.
    pub rem: Option<String>,
}

impl TranStamp {
    /// State a transmission. All five are REQUIRED headings, so all five are
    /// required arguments — the type makes a half-stamp unconstructible rather
    /// than leaving it to a rule to report after the fact.
    pub fn new(
        isno: impl Into<String>,
        date: impl Into<String>,
        prod: impl Into<String>,
        recv: impl Into<String>,
        stat: impl Into<String>,
    ) -> TranStamp {
        TranStamp {
            isno: isno.into(),
            date: date.into(),
            prod: prod.into(),
            recv: recv.into(),
            stat: stat.into(),
            ags: String::new(),
            desc: None,
            rem: None,
        }
    }

    /// `TRAN_DESC` — what was transferred.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> TranStamp {
        self.desc = Some(desc.into());
        self
    }

    /// `TRAN_REM` — free remarks.
    #[must_use]
    pub fn with_remarks(mut self, rem: impl Into<String>) -> TranStamp {
        self.rem = Some(rem.into());
        self
    }

    /// Fold a surface's five loose optional values into a stamp, or `None`.
    ///
    /// **The boundary adapter, not a constructor.** Public because the binding
    /// crates need it: CLI flags, Python kwargs and JSON objects all arrive as
    /// independent optionals whatever the type says. Every surface funnels
    /// through here so exactly one place decides what counts as enough — and it
    /// is `all five or nothing`, matching `new`. It used to be issue+date, which
    /// is how the three REQUIRED cells came to be silently empty.
    ///
    /// Prefer `new` wherever the values are already known to be present; reach
    /// for this only at a boundary that genuinely holds five optionals.
    ///
    /// `Err` carries the names of the missing fields: a caller who supplied four
    /// of five made a mistake and should be told which, not handed a silent
    /// `None` that reads identically to "I meant not to stamp one".
    ///
    /// `desc` and `rem` are deliberately OUTSIDE the all-five-or-none rule.
    /// That rule exists because `TRAN_ISNO`/`DATE`/`PROD`/`RECV`/`STAT` are
    /// REQUIRED headings; `TRAN_DESC` and `TRAN_REM` are OTHER, so a stamp with
    /// five parts and no description is complete, and folding them in would
    /// make the optional mandatory. They do not get to be silent either: a
    /// caller who states only a description has stated a partial transmission,
    /// and gets the same `Err` naming the five it lacks rather than a `None`
    /// that discards what they wrote.
    pub fn from_parts(
        isno: Option<String>,
        date: Option<String>,
        prod: Option<String>,
        recv: Option<String>,
        stat: Option<String>,
        desc: Option<String>,
        rem: Option<String>,
    ) -> Result<Option<TranStamp>, TranStampError> {
        let parts = [
            ("issue", &isno),
            ("date", &date),
            ("producer", &prod),
            ("recipient", &recv),
            ("status", &stat),
        ];
        let missing: Vec<&str> = parts
            .iter()
            .filter(|(_, v)| v.as_ref().is_none_or(|s| s.trim().is_empty()))
            .map(|(n, _)| *n)
            .collect();
        let stated = |v: &Option<String>| v.as_ref().is_some_and(|s| !s.trim().is_empty());
        let optionals_stated = stated(&desc) || stated(&rem);
        match missing.len() {
            // Nothing stated at all: no TRAN, and Rule 14 reports the gap. But a
            // description or remark with no transmission behind it is a partial
            // stamp, not an absent one — returning `None` there would drop what
            // the caller wrote and look identical to not asking.
            5 if !optionals_stated => Ok(None),
            0 => {
                let mut stamp = TranStamp::new(
                    isno.unwrap(),
                    date.unwrap(),
                    prod.unwrap(),
                    recv.unwrap(),
                    stat.unwrap(),
                );
                if stated(&desc) {
                    stamp = stamp.with_description(desc.unwrap());
                }
                if stated(&rem) {
                    stamp = stamp.with_remarks(rem.unwrap());
                }
                Ok(Some(stamp))
            }
            _ => Err(TranStampError {
                missing: missing.iter().map(|s| (*s).to_string()).collect(),
            }),
        }
    }
}

/// A partially-stated transmission — some TRAN fields given, others not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranStampError {
    pub missing: Vec<String>,
}

impl std::fmt::Display for TranStampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "incomplete TRAN: missing {}. All five are REQUIRED headings \
             (TRAN_ISNO/DATE/PROD/RECV/STAT), so a partial stamp would emit a \
             TRAN that fails Rule 10b. Supply all five, or none to omit TRAN \
             and let Rule 14 report the gap.",
            self.missing.join(", ")
        )
    }
}

impl std::error::Error for TranStampError {}

/// Emit options. `edition` selects which AGS4 standard dictionary fills
/// UNIT/TYPE and which rule set validity is judged against.
#[derive(Debug, Clone)]
pub struct EmitOpts {
    pub mode: EmitMode,
    pub edition: DictVersion,
    /// The TRAN row to stamp when synthesis is on and the input carries no TRAN.
    ///
    /// `None` means **emit no TRAN at all** — deliberately, and this is the
    /// point. The engine cannot know who produced a file, for whom, on what
    /// date, at what status; a stub asserting `TBC`/`1900-01-01` still SATISFIES
    /// Rule 14, so a recipient has no way to tell an invented transmission
    /// record from a real one and nothing downstream flags it. A missing TRAN
    /// that reports Rule 14 is strictly more honest than a present one that
    /// lies. Same reasoning already applied to PROJ and DICT: never invent what
    /// only the caller can know.
    pub tran: Option<TranStamp>,
    /// Mint the mandatory metadata catalogs (UNIT / TYPE / TRAN / ABBR) the
    /// input doesn't carry. `AutoFix` only — `Strict`/`Report` always show or
    /// reject the gaps rather than filling them.
    ///
    /// **Off by default, and opt-in by design.** Synthesis adds whole GROUPS
    /// the caller never wrote; that is the sort of unexpected magic a caller
    /// should ask for rather than discover. Turning it on is a statement that
    /// derived catalogs are wanted.
    ///
    /// Only *derivable* metadata is ever minted: UNIT and TYPE are pure
    /// functions of the data, ABBR comes from the standard table and only when
    /// PA codes are used. PROJ, DICT and TRAN are never synthesised — a project
    /// identity, a schema extension and a record of transmission are authorial
    /// facts. Inventing a DICT parent would turn a loud Rule 18 error into a
    /// silent false statement that Rule 10's relational checks then trust; TRAN
    /// is stamped from `EmitOpts::tran` or omitted, never guessed.
    ///
    /// Separated from `mode` so it can be MEASURED: it is a distinct stage
    /// (step 2.5), and folding it into `AutoFix` meant the only observable
    /// number was the whole mode. `benches/emit.rs` walks write → Report →
    /// AutoFix-without → AutoFix-with to price each stage; the stage costs
    /// ~0.3% of an export, so this flag is not a performance knob.
    pub synthesise_metadata: bool,
}

impl Default for EmitOpts {
    fn default() -> Self {
        // AutoFix + 4.1.1 are the resolved project defaults (see the
        // ags4-output design page, decided 2026-06-12).
        //
        // `synthesise_metadata: false` (2026-07-24). Minting whole GROUPS the
        // caller never asked for is the kind of unexpected magic the caller
        // should opt INTO — the decision is about agency, not cost. The staged
        // benches priced it at ~0.13 ms (0.3% of an export), so there is no
        // performance argument on either side of that choice; it is free to
        // leave off and free to turn on.
        //
        // The consequence is deliberate and must stay documented: a data-only
        // build now reports Rule 14/15/17 rather than silently filling them.
        // See the ags4-output design page.
        EmitOpts {
            mode: EmitMode::AutoFix,
            edition: DictVersion::V4_1_1,
            tran: None,
            synthesise_metadata: false,
        }
    }
}

/// The emit result. `findings` are on the *returned* bytes (so, post-fix
/// for `AutoFix`) — empty means clean. `applied` is the list of safe fixes
/// `AutoFix` made (empty for Strict/Report); `fixes_applied` is its length,
/// kept as a convenience so a caller can report a count without the detail.
pub struct EmitResult {
    pub bytes: Vec<u8>,
    pub findings: Findings,
    pub applied: Vec<Fix>,
    pub fixes_applied: usize,
}

/// Build valid AGS4 bytes from typed/string group data per `opts`.
pub fn emit_ags4(groups: &[GroupInput], opts: &EmitOpts) -> Result<EmitResult, EmitError> {
    let dict = Dictionary::bundled(opts.edition);
    let mut stream = EmitStream::new(opts, &dict);
    for g in groups {
        // Per-group: the formatted copy is written and dropped before the
        // next group formats — no whole-file OwnedGroup slab exists
        // (dec-emit-streamed-verdict).
        stream.push(owned_group(g, &dict))?;
    }
    stream.finish()
}

/// Like [`emit_ags4`], but consuming its input, group by group.
///
/// Behaviourally identical; the difference is the peak. Under the borrowed
/// entry the caller's groups stay live behind the borrow until the emit
/// returns, co-resident with the formatted copy, the written bytes and the
/// validating re-parse. Here each group's input drops the moment its cells
/// are formatted, so the input copy and the re-parse never peak together.
/// Measured with `examples/heap_profile.rs` (#789, then #790 — which also
/// shrank the cells themselves from `serde_json::Value` to [`Cell`]): the
/// input copy was the single largest slice of the emit's live-at-peak bytes.
pub fn emit_ags4_owned(groups: Vec<GroupInput>, opts: &EmitOpts) -> Result<EmitResult, EmitError> {
    let dict = Dictionary::bundled(opts.edition);
    let mut stream = EmitStream::new(opts, &dict);
    for g in groups {
        // `g` is consumed per iteration: its cell rows free here, not at
        // return — and the formatted copy drops inside `push`, so input,
        // formatted copy and verdict never peak together.
        stream.push(owned_group_consuming(g, &dict))?;
    }
    stream.finish()
}

/// Steps 1–2 for one group: resolve UNIT/TYPE (hybrid) + format every cell.
/// `OwnedGroup` holds Strings; the writer borrows them for the write.
fn owned_group(g: &GroupInput, dict: &Dictionary) -> OwnedGroup {
    let (units, types) = resolved_meta(g, dict);
    let rows: Vec<Vec<String>> = g.rows.iter().map(|row| format_row(row, &types)).collect();
    OwnedGroup {
        code: g.code.clone(),
        headings: g.headings.clone(),
        units,
        types,
        rows,
    }
}

/// [`owned_group`], consuming: each input row's cells free as soon as that
/// row is formatted, so a large group's input copy and its formatted copy
/// never fully coexist — the live set trades one for the other row by row.
/// (Per-GROUP consumption alone does not get that: the measured TREL is one
/// group of 2.4M rows, and its whole input was still live at the moment its
/// last row formatted.)
fn owned_group_consuming(g: GroupInput, dict: &Dictionary) -> OwnedGroup {
    let (units, types) = resolved_meta(&g, dict);
    let rows: Vec<Vec<String>> = g
        .rows
        .into_iter()
        .map(|row| format_row(&row, &types))
        .collect();
    OwnedGroup {
        code: g.code,
        headings: g.headings,
        units,
        types,
        rows,
    }
}

fn resolved_meta(g: &GroupInput, dict: &Dictionary) -> (Vec<String>, Vec<String>) {
    resolved_meta_parts(
        &g.code,
        &g.headings,
        g.units.as_ref(),
        g.types.as_ref(),
        dict,
    )
}

/// [`resolved_meta`] over loose parts — shared with the Arrow door
/// (`arrow_in`), whose group is not a [`GroupInput`], so the hybrid
/// caller-wins/dictionary-fills resolution stays in exactly one place.
pub(crate) fn resolved_meta_parts(
    code: &str,
    headings: &[String],
    units: Option<&Vec<String>>,
    types: Option<&Vec<String>>,
    dict: &Dictionary,
) -> (Vec<String>, Vec<String>) {
    let resolved_units: Vec<String> = (0..headings.len())
        .map(|i| resolve_meta(units, i, || dict_unit(dict, code, &headings[i])))
        .collect();
    let resolved_types: Vec<String> = (0..headings.len())
        .map(|i| resolve_meta(types, i, || dict_type(dict, code, &headings[i])))
        .collect();
    (resolved_units, resolved_types)
}

fn format_row(row: &[Cell], types: &[String]) -> Vec<String> {
    row.iter()
        .enumerate()
        .map(|(i, cell)| format_cell(cell, types.get(i).map_or("X", String::as_str)))
        .collect()
}

/// Steps 2.5–4 as a STREAM: each formatted group is written (and its
/// verdict structure recorded) the moment it arrives, then dropped — for
/// both doors, so no whole-file `OwnedGroup` slab ever exists — and the
/// verdict is assembled from the writer's own records instead of a
/// `parse_bytes` of the bytes it just wrote (`dec-emit-streamed-verdict`).
/// The old pipeline held the parse hold TWICE at the peak: the caller's
/// retained `ParsedFile` plus the validating parse-back, co-peak shoulders
/// that backed each other up (#848 — only fixing both moved the peak).
pub(crate) struct EmitStream<'a> {
    builder: ParsedFileBuilder,
    opts: &'a EmitOpts,
    dict: &'a Dictionary<'a>,
    /// step 2.5's catalogues, filled as the groups stream — `Some` only when
    /// synthesis is on (`AutoFix` + opt-in), exactly the old batch condition.
    synth: Option<SynthAccumulator>,
    wrote_any: bool,
}

impl<'a> EmitStream<'a> {
    pub(crate) fn new(opts: &'a EmitOpts, dict: &'a Dictionary<'a>) -> Self {
        // AutoFix only: a data-only build (notably a typed PROJ graph, which
        // can't reach the parentless root-metadata groups) still yields a
        // valid file — mint UNIT/TYPE (derived from the data) and ABBR (when
        // PA codes are used) for whichever are absent, plus TRAN when the
        // caller stamped one. PROJ is never synthesized (real project
        // identity), so a missing PROJ stays a Rule 13 finding.
        let synth = (opts.mode == EmitMode::AutoFix && opts.synthesise_metadata)
            .then(SynthAccumulator::default);
        EmitStream {
            builder: ParsedFileBuilder::new(),
            opts,
            dict,
            synth,
            wrote_any: false,
        }
    }

    /// One group's section through the recording writer: the borrow view is
    /// built HERE and only here — `push` and `assemble`'s synthesised loop
    /// carried this block verbatim twice, which is exactly the shape that
    /// drifts one field at a time (#857).
    fn write_group(&mut self, og: &OwnedGroup) -> Result<(), EmitError> {
        let view = EmitGroup {
            code: &og.code,
            headings: og.headings.iter().map(String::as_str).collect(),
            units: og.units.iter().map(String::as_str).collect(),
            types: og.types.iter().map(String::as_str).collect(),
            rows: &og.rows,
        };
        write_section_recorded(&mut self.builder, &view, !self.wrote_any)?;
        self.wrote_any = true;
        Ok(())
    }

    /// Write one formatted group's section and record its verdict structure.
    /// Consumes the group ON PURPOSE (the streamed join's whole point): its
    /// cells free when this returns, not at finish — by-reference would let a
    /// caller keep the formatted slab alive across the stream.
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn push(&mut self, og: OwnedGroup) -> Result<(), EmitError> {
        if let Some(acc) = &mut self.synth {
            acc.fold(&og);
        }
        self.write_group(&og)
    }

    /// Write the synthesised groups (last, preserving the batch pipeline's
    /// byte order) and assemble the writer-built verdict: the emitted bytes
    /// adopted as the verdict's buffer, no `parse_bytes` run.
    pub(crate) fn assemble(mut self) -> Result<(ParsedFile, usize), EmitError> {
        if let Some(acc) = self.synth.take() {
            for og in acc.synthesised(self.dict, self.opts.tran.as_ref()) {
                self.write_group(&og)?;
            }
        }
        // A zero-group build fails exactly as the parse-back used to:
        // through the walk's own NotAgs4, worded identically.
        self.builder
            .finish()
            .map_err(|e| EmitError::Reparse(ValidatorError::from(e).to_string()))
    }

    /// Assemble, judge, apply the validity mode.
    pub(crate) fn finish(self) -> Result<EmitResult, EmitError> {
        let opts = self.opts;
        let (parsed, emitted_len) = self.assemble()?;
        judge(parsed, emitted_len, opts)
    }
}

/// Step 4, over the CONSTRUCTED verdict: run the rule engine, apply the
/// validity mode. The happy path never parses — the bytes leave through
/// [`reclaim_bytes`], zero-copy out of the verdict's buffer.
fn judge(parsed: ParsedFile, emitted_len: usize, opts: &EmitOpts) -> Result<EmitResult, EmitError> {
    let dict = Dictionary::bundled(opts.edition);
    let copts = CheckOptions {
        dict_version: Some(opts.edition),
        ..CheckOptions::default()
    };
    // Content-only by construction: bytes produced in memory have no
    // directory to sit beside, so there is no world to look at.
    let found = check_parsed(&parsed, &dict, &copts, &WorldScope::None)
        .map_err(|e| EmitError::Reparse(e.to_string()))?;
    match opts.mode {
        EmitMode::Report => Ok(EmitResult {
            bytes: reclaim_bytes(parsed, emitted_len),
            findings: found,
            applied: Vec::new(),
            fixes_applied: 0,
        }),
        EmitMode::Strict => {
            let errors = found
                .values()
                .flatten()
                .filter(|f| f.severity == Severity::Error)
                .count();
            if errors > 0 {
                Err(EmitError::Invalid(found))
            } else {
                Ok(EmitResult {
                    bytes: reclaim_bytes(parsed, emitted_len),
                    findings: found,
                    applied: Vec::new(),
                    fixes_applied: 0,
                })
            }
        }
        EmitMode::AutoFix => {
            // `compute_fixes` reads exactly the inventory the constructed
            // verdict populates (#848), so no parse runs to price the fixes.
            let safe: Vec<_> = compute_fixes(&parsed, &found)
                .into_iter()
                .filter(|f| f.risk == FixRisk::Safe)
                .collect();
            if safe.is_empty() {
                return Ok(EmitResult {
                    bytes: reclaim_bytes(parsed, emitted_len),
                    findings: found,
                    applied: Vec::new(),
                    fixes_applied: 0,
                });
            }
            // The rare path — actual safe fixes exist. The verdict buffer's
            // emitted prefix IS the text (`apply_fixes` wants `&str`); the
            // emitter never writes a BOM, so has_bom = false.
            let fixed = apply_fixes(&parsed.text[..emitted_len], false, &safe);
            drop(parsed);
            let fixed_bytes = fixed.into_bytes();
            // Residual findings on the *fixed* output — a genuinely different
            // document, so this bounded re-parse is real work, not a repeat.
            let (_, residual) = validate(&fixed_bytes, opts.edition)?;
            Ok(EmitResult {
                bytes: fixed_bytes,
                findings: residual,
                fixes_applied: safe.len(),
                applied: safe,
            })
        }
    }
}

/// Take the emitted bytes back out of the verdict's buffer: drop the parse
/// (each group holds a refcount on the buffer), unwrap it at refcount 1,
/// truncate the fix-up tail. Zero-copy on the invariant path — the
/// `unwrap_or_else` copy is a defensive fallback for a leaked refcount, not
/// a code path anything is expected to take.
fn reclaim_bytes(parsed: ParsedFile, emitted_len: usize) -> Vec<u8> {
    let text = Arc::clone(&parsed.text);
    drop(parsed);
    let mut s = Arc::try_unwrap(text).unwrap_or_else(|a| (*a).clone());
    s.truncate(emitted_len);
    s.into_bytes()
}

/// Owned mirror of an emit group — `EmitGroup` borrows `&str`s, so we
/// build owned Strings first then borrow them for the write.
///
/// `pub(crate)`: this is the JOIN POINT of the two input doors (#790). The
/// JSON/document door ([`owned_group`]) and the Arrow door
/// (`arrow_in::owned_group_from_arrow`) each produce one of these, and
/// everything after — synthesis, write, validate, mode — is a single code
/// path. A differential test holds the doors equal here, so the type stays
/// crate-internal: it is a meeting place, not a surface.
pub(crate) struct OwnedGroup {
    pub(crate) code: String,
    pub(crate) headings: Vec<String>,
    pub(crate) units: Vec<String>,
    pub(crate) types: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

/// Hybrid metadata resolution: the caller's explicit non-blank value
/// wins; otherwise the dictionary fallback (`fill`) fills.
fn resolve_meta(
    overrides: Option<&Vec<String>>,
    i: usize,
    fill: impl FnOnce() -> String,
) -> String {
    match overrides.and_then(|v| v.get(i)) {
        Some(s) if !s.trim().is_empty() => s.clone(),
        _ => fill(),
    }
}

fn dict_unit(dict: &Dictionary, code: &str, heading: &str) -> String {
    dict.heading(code, heading)
        .map(|e| e.unit.to_string())
        .unwrap_or_default()
}

fn dict_type(dict: &Dictionary, code: &str, heading: &str) -> String {
    // Out-of-dictionary headings default to "X" (free text) — write_ags4
    // also defaults a blank TYPE to "X", so this is belt-and-braces.
    dict.heading(code, heading)
        .map_or_else(|| "X".to_string(), |e| e.ags_type.to_string())
}

/// Format one cell to its AGS4 string. Typed values go through the
/// canonical `ags4_str`; string values emit verbatim so the validity
/// *mode* is the single owner of canonicalisation (Report = unchanged,
/// `AutoFix`'s text fixer pads/normalises, Strict rejects).
fn format_cell(value: &Cell, ags_type: &str) -> String {
    match value {
        Cell::Text(s) => s.clone(),
        _ => ags4_str(value, ags_type),
    }
}

/// Parse bytes + run all rules at the given edition. Since the writer-built
/// verdict (`dec-emit-streamed-verdict`) this runs on ONE path only: the
/// residual validation of `apply_fixes`' output — a genuinely different
/// document from the one the writer recorded, so the parse is real work.
/// The happy path's verdict is constructed by the writer and never parses.
///
/// Content-only by construction: the emitter re-validates bytes it just produced
/// in memory, which have no directory to sit beside, so there is no world to look
/// at ([`WorldScope::None`]) and `check_files` stays default-off.
fn validate(bytes: &[u8], edition: DictVersion) -> Result<(ParsedFile, Findings), EmitError> {
    let parsed =
        parse_bytes(bytes, encoding_rs::UTF_8).map_err(|e| EmitError::Reparse(e.to_string()))?;
    let dict = Dictionary::bundled(edition);
    let opts = CheckOptions {
        dict_version: Some(edition),
        ..CheckOptions::default()
    };
    let found = check_parsed(&parsed, &dict, &opts, &WorldScope::None)
        .map_err(|e| EmitError::Reparse(e.to_string()))?;
    Ok((parsed, found))
}

/// Step 2.5's catalogues, accumulated as the groups STREAM — the batch
/// `synthesise_metadata(&[OwnedGroup], …)` walked the whole formatted set,
/// which no longer exists ([`EmitStream`] drops each group after writing
/// it). Same derivations, same order, same output (the differential test
/// pins the accumulator against the batch computation): UNIT and TYPE are
/// pure derivations of the data; ABBR is minted when (and only when) the
/// data uses PA picklist codes (Rule 16); TRAN is written only from a
/// caller-supplied `TranStamp`, never invented. PROJ is deliberately never
/// synthesized — it carries real project identity, not derivable metadata,
/// so a missing PROJ stays a Rule 13 finding.
#[derive(Default)]
struct SynthAccumulator {
    present: BTreeSet<String>,
    units: BTreeSet<String>,
    types: BTreeSet<String>,
    /// Distinct `(heading, raw cell)` pairs from PA-typed columns. RAW —
    /// the concatenator that splits them can arrive in a later group's
    /// `TRAN_RCON`, so splitting waits for `synthesised`.
    pa_cells: BTreeSet<(String, String)>,
    /// The first non-blank `TRAN_RCON` seen (Rule 16a's concatenator).
    rcon: Option<String>,
}

impl SynthAccumulator {
    /// Fold one streamed group's contribution into the catalogues, via the
    /// same collectors the batch path used one group at a time.
    fn fold(&mut self, g: &OwnedGroup) {
        self.present.insert(g.code.clone());
        self.units.extend(collect_units(std::iter::once(g)));
        self.types.extend(collect_types(std::iter::once(g)));
        collect_pa_cells(g, &mut self.pa_cells);
        if self.rcon.is_none() {
            self.rcon = tran_rcon(g);
        }
    }

    /// The synthesised groups, in the batch pipeline's order — each folded
    /// into the catalogues the NEXT one derives from, reproducing the batch
    /// `chain(owned, synth)` reads.
    fn synthesised(mut self, dict: &Dictionary, tran: Option<&TranStamp>) -> Vec<OwnedGroup> {
        let mut synth: Vec<OwnedGroup> = Vec::new();

        // TRAN first: its DT `TRAN_DATE` introduces the `yyyy-mm-dd` unit and
        // the `DT` type that the UNIT/TYPE catalogs below must then cover.
        //
        // Only when the caller supplied one. Without a stamp we emit NO TRAN
        // and let Rule 14 report it — see `EmitOpts::tran` for why a
        // placeholder is worse than an absence.
        //
        // `filter` rather than the `&& let` chain this obviously wants to be:
        // let chains stabilised in 1.88, and this crate's `rust-version`
        // promises 1.85. The nested-`if` alternative reads better but trips
        // clippy's `collapsible_if`, whose suggested fix is the let chain
        // again. Don't "simplify" this back — `msrv` in CI will catch it,
        // which is how it was found.
        if let Some(t) = tran.filter(|_| !self.present.contains("TRAN")) {
            let g = synth_tran(dict, t);
            self.units.extend(collect_units(std::iter::once(&g)));
            self.types.extend(collect_types(std::iter::once(&g)));
            synth.push(g);
        }
        // UNIT: one row per distinct unit used across all groups (Rule 15).
        //
        // Skipped when nothing uses a unit. An empty catalog is not a neutral
        // no-op — a group with no DATA rows is itself a Rule 2 error, so
        // minting one would trade a Rule 15 finding for a Rule 2 finding and
        // call it synthesis. This became reachable when TRAN stopped being
        // minted unconditionally: its `yyyy-mm-dd` on TRAN_DATE was quietly
        // the thing guaranteeing at least one unit existed.
        if !self.present.contains("UNIT") && !self.units.is_empty() {
            synth.push(synth_catalog("UNIT", "UNIT_UNIT", "UNIT_DESC", &self.units));
        }
        // TYPE: one row per distinct type code used (Rule 17). Force `X` —
        // every synthesized metadata group is all-`X`, so the catalog must
        // self-cover (which also covers the UNIT catalog just minted).
        if !self.present.contains("TYPE") {
            let mut types = std::mem::take(&mut self.types);
            types.insert("X".to_string());
            synth.push(synth_catalog("TYPE", "TYPE_TYPE", "TYPE_DESC", &types));
        }
        // ABBR: only when the data uses PA picklist codes (Rule 16) — one row
        // per distinct (heading, code), description from the standard ABBR
        // table (fallback: the code itself). PA values split on the
        // concatenator the same way Rule 16 reads it (the file's `TRAN_RCON`,
        // else the AGS `+`) — from the CALLER's groups only, as the batch
        // computation read them.
        if !self.present.contains("ABBR") {
            let concat = self.rcon.as_deref().unwrap_or("+");
            let abbrs = abbreviations_from_pa_cells(&self.pa_cells, dict, concat);
            if !abbrs.is_empty() {
                synth.push(synth_abbr(abbrs));
            }
        }
        synth
    }
}

/// Distinct non-empty units used (Rule 15): every group's UNIT-row value plus
/// every distinct value in a `PU`-typed data column. Blanks and the literal
/// `"UNIT"` are excluded.
fn collect_units<'a>(groups: impl Iterator<Item = &'a OwnedGroup>) -> BTreeSet<String> {
    let mut units = BTreeSet::new();
    for g in groups {
        for u in &g.units {
            let u = u.trim();
            if !u.is_empty() && u != "UNIT" {
                units.insert(u.to_string());
            }
        }
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() == "PU" {
                for row in &g.rows {
                    if let Some(v) = row.get(ci).map(|s| s.trim()) {
                        if !v.is_empty() {
                            units.insert(v.to_string());
                        }
                    }
                }
            }
        }
    }
    units
}

/// Distinct non-empty type codes used (Rule 17): every group's TYPE-row value.
/// Blanks and the literal `"TYPE"` are excluded.
fn collect_types<'a>(groups: impl Iterator<Item = &'a OwnedGroup>) -> BTreeSet<String> {
    let mut types = BTreeSet::new();
    for g in groups {
        for t in &g.types {
            let t = t.trim();
            if !t.is_empty() && t != "TYPE" {
                types.insert(t.to_string());
            }
        }
    }
    types
}

/// A two-column catalog group (UNIT or TYPE): the KEY symbol + a DESC that
/// falls back to the symbol itself (the validator only requires DESC be
/// non-empty, and the dictionary carries no unit/type description catalog).
fn synth_catalog(code: &str, key: &str, desc: &str, symbols: &BTreeSet<String>) -> OwnedGroup {
    OwnedGroup {
        code: code.to_string(),
        headings: vec![key.to_string(), desc.to_string()],
        units: vec![String::new(), String::new()],
        types: vec!["X".to_string(), "X".to_string()],
        rows: symbols.iter().map(|s| vec![s.clone(), s.clone()]).collect(),
    }
}

/// The TRAN row for a synthesised transmission, built from the CALLER's
/// [`TranStamp`].
///
/// `TRAN_DLIM`/`TRAN_RCON` are the AGS standard `"|"`/`"+"`, and an empty
/// `stamp.ags` falls back to the edition's expected value (so no
/// unrecognised-edition warning). Every other field comes from the caller.
///
/// This function used to write `"TBC"` and `"1900-01-01"` when it had nothing
/// better, which produced a file that SATISFIED Rule 14 while asserting a
/// transmission that never happened. It is now unreachable without a stamp.
fn synth_tran(dict: &Dictionary, stamp: &TranStamp) -> OwnedGroup {
    // The five authorial cells, then the three the emitter knows because it is
    // writing the file: the edition in force, and the delimiter + concatenator
    // this emitter uses. `stamp.ags` wins only for `merge`, which resolves the
    // edition from its newest input rather than from `opts.edition`.
    let mut headings = vec![
        "TRAN_ISNO",
        "TRAN_DATE",
        "TRAN_PROD",
        "TRAN_STAT",
        "TRAN_AGS",
        "TRAN_RECV",
        "TRAN_DLIM",
        "TRAN_RCON",
    ];
    let mut units = vec!["", "yyyy-mm-dd", "", "", "", "", "", ""];
    let mut row = vec![
        stamp.isno.clone(),
        stamp.date.clone(),
        stamp.prod.clone(),
        stamp.stat.clone(),
        if stamp.ags.trim().is_empty() {
            dict.tran_ags().to_string()
        } else {
            stamp.ags.clone()
        },
        stamp.recv.clone(),
        "|".to_string(),
        "+".to_string(),
    ];

    // OTHER headings, emitted only when stated. An empty TRAN_DESC column is not
    // free: every heading present must also be covered by the TYPE catalog, so a
    // column nobody filled buys a catalog row that describes nothing.
    for (name, value) in [("TRAN_DESC", &stamp.desc), ("TRAN_REM", &stamp.rem)] {
        if let Some(v) = value {
            headings.push(name);
            units.push("");
            row.push(v.clone());
        }
    }

    OwnedGroup {
        code: "TRAN".to_string(),
        types: headings
            .iter()
            .map(|h| if *h == "TRAN_DATE" { "DT" } else { "X" }.to_string())
            .collect(),
        headings: headings.into_iter().map(String::from).collect(),
        units: units.into_iter().map(String::from).collect(),
        rows: vec![row],
    }
}

/// One group's contribution to Rule 16a's concatenator: the first row's
/// non-blank `TRAN_RCON` of a TRAN group, else `None`. The accumulator keeps
/// the first `Some` it sees — the batch scan's first-non-blank-wins.
fn tran_rcon(g: &OwnedGroup) -> Option<String> {
    if g.code != "TRAN" {
        return None;
    }
    let ci = g.headings.iter().position(|h| h == "TRAN_RCON")?;
    let v = g.rows.first().and_then(|r| r.get(ci))?.trim();
    (!v.is_empty()).then(|| v.to_string())
}

/// One group's distinct `(heading, raw cell)` pairs from `PA`-typed columns,
/// into `seen`. RAW: the concatenator that splits a cell into codes can be
/// declared by a TRAN group that streams later, so splitting waits until
/// every group has passed ([`abbreviations_from_pa_cells`]). Distinct pairs
/// bound the hold to the picklist's vocabulary, not the row count.
fn collect_pa_cells(g: &OwnedGroup, seen: &mut BTreeSet<(String, String)>) {
    for (ci, ty) in g.types.iter().enumerate() {
        if ty.trim() == "PA" {
            let heading = g.headings.get(ci).map_or("", String::as_str);
            for row in &g.rows {
                if let Some(cell) = row.get(ci) {
                    if !cell.is_empty() {
                        seen.insert((heading.to_string(), cell.clone()));
                    }
                }
            }
        }
    }
}

/// Distinct `(heading, code, desc)` triples from the accumulated PA cells
/// (Rule 16). Each cell splits on `concat` (matching Rule 16a); the
/// description is the standard ABBR table's, falling back to the code.
fn abbreviations_from_pa_cells(
    pa_cells: &BTreeSet<(String, String)>,
    dict: &Dictionary,
    concat: &str,
) -> Vec<[String; 3]> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for (heading, cell) in pa_cells {
        for code in cell.split(concat) {
            let code = code.trim();
            if !code.is_empty() {
                seen.insert((heading.clone(), code.to_string()));
            }
        }
    }
    seen.into_iter()
        .map(|(h, c)| {
            let desc = dict
                .abbr_desc(&h, &c)
                .map_or_else(|| c.clone(), str::to_string);
            [h, c, desc]
        })
        .collect()
}

/// The ABBR group from collected `(heading, code, desc)` triples.
fn synth_abbr(rows: Vec<[String; 3]>) -> OwnedGroup {
    OwnedGroup {
        code: "ABBR".to_string(),
        headings: vec![
            "ABBR_HDNG".to_string(),
            "ABBR_CODE".to_string(),
            "ABBR_DESC".to_string(),
        ],
        units: vec![String::new(), String::new(), String::new()],
        types: vec!["X".to_string(), "X".to_string(), "X".to_string()],
        rows: rows.into_iter().map(|r| r.to_vec()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cell shorthand — the role `json!` played before #790.
    fn c(v: impl Into<Cell>) -> Cell {
        v.into()
    }

    /// THE EQUIVALENCE CONTRACT (dec-emit-streamed-verdict): the writer-built
    /// verdict must be indistinguishable from a real `parse_bytes` of the
    /// same bytes, held with BOTH definitions —
    ///
    /// 1. **findings equality**: `check_parsed` over the constructed object
    ///    and over the re-parse yields identical findings;
    /// 2. **field equality on the read-inventory** (`groups`, `group_order`,
    ///    `raw_lines`, `has_bom`, line text, descriptor rows/lines, per-row
    ///    line numbers and values — plus the group-level coordinates), which
    ///    localises a failure when the first definition fires.
    ///
    /// This test is the contract's ENFORCEMENT, not scaffolding — it stays
    /// for as long as the constructed verdict does. Span COORDINATES are
    /// deliberately not compared: the two buffers lay out differently
    /// (terminators, fix-up placement) and only resolved values are claims.
    fn assert_constructed_verdict_equals_reparse(groups: &[GroupInput], opts: &EmitOpts) {
        let dict = Dictionary::bundled(opts.edition);
        let mut stream = EmitStream::new(opts, &dict);
        for g in groups {
            stream.push(owned_group(g, &dict)).expect("push");
        }
        let (built, emitted_len) = stream.assemble().expect("assemble");
        let bytes = &built.text.as_bytes()[..emitted_len];
        let reparsed = parse_bytes(bytes, encoding_rs::UTF_8).expect("re-parse");

        // Definition 2 first: it localises.
        assert_eq!(built.group_order, reparsed.group_order);
        assert_eq!(built.total_lines, reparsed.total_lines);
        assert_eq!(built.has_bom, reparsed.has_bom);
        assert_eq!(built.total_bytes, reparsed.total_bytes);
        assert_eq!(built.group_records, reparsed.group_records);
        assert_eq!(
            built.byte_offsets_source_true,
            reparsed.byte_offsets_source_true
        );
        assert_eq!(built.raw_lines.len(), reparsed.raw_lines.len());
        for (bl, pl) in built.raw_lines.iter().zip(&reparsed.raw_lines) {
            assert_eq!(bl.number, pl.number);
            assert_eq!(bl.had_crlf, pl.had_crlf, "line {}", pl.number);
            assert_eq!(
                built.line_text(bl),
                reparsed.line_text(pl),
                "line {}",
                pl.number
            );
        }
        for code in &reparsed.group_order {
            let bg = &built.groups[code];
            let pg = &reparsed.groups[code];
            assert_eq!(bg.group_line, pg.group_line, "{code}");
            assert_eq!(bg.group_byte, pg.group_byte, "{code}");
            assert_eq!(bg.heading_line, pg.heading_line, "{code}");
            assert_eq!(bg.unit_line, pg.unit_line, "{code}");
            assert_eq!(bg.type_line, pg.type_line, "{code}");
            assert_eq!(bg.headings, pg.headings, "{code}");
            assert_eq!(bg.units, pg.units, "{code}");
            assert_eq!(bg.types, pg.types, "{code}");
            assert_eq!(bg.rows.len(), pg.rows.len(), "{code}");
            for (br, pr) in bg.rows.iter().zip(&pg.rows) {
                assert_eq!(br.line, pr.line, "{code}");
                assert_eq!(br.n_values(), pr.n_values(), "{code} line {}", pr.line);
                for i in 0..pr.n_values() {
                    assert_eq!(
                        bg.value_at(br, i),
                        pg.value_at(pr, i),
                        "{code} line {} col {i}",
                        pr.line
                    );
                }
            }
        }

        // Definition 1: the findings.
        let copts = CheckOptions {
            dict_version: Some(opts.edition),
            ..CheckOptions::default()
        };
        let f_built = check_parsed(&built, &dict, &copts, &WorldScope::None).expect("check built");
        let f_reparsed =
            check_parsed(&reparsed, &dict, &copts, &WorldScope::None).expect("check reparse");
        assert_eq!(f_built, f_reparsed, "findings must be identical");
    }

    /// The differential over the awkward shapes the writer can produce: a
    /// clean typed build; escaped quotes and non-ASCII; ragged rows both
    /// short AND long of the headings; a duplicate group code (first-seen-
    /// wins on the verdict side); an empty group; blank cells.
    #[test]
    fn constructed_verdict_matches_a_real_parse_across_awkward_shapes() {
        let opts = EmitOpts {
            mode: EmitMode::Report,
            ..EmitOpts::default()
        };
        // Clean typed build.
        assert_constructed_verdict_equals_reparse(
            &[
                proj(),
                GroupInput {
                    code: "LOCA".into(),
                    headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
                    units: None,
                    types: None,
                    rows: vec![vec![c("BH01"), c(12.3)], vec![c("BH02"), Cell::Null]],
                },
            ],
            &opts,
        );
        // Escapes, non-ASCII, blanks, ragged short + long rows.
        assert_constructed_verdict_equals_reparse(
            &[
                proj(),
                GroupInput {
                    code: "LOCA".into(),
                    headings: vec!["LOCA_ID".into(), "LOCA_REM".into()],
                    units: None,
                    types: None,
                    rows: vec![
                        vec![c("BH01"), c("say \"hi\" — 10° ✓")],
                        vec![c("BH02")],
                        vec![c("BH03"), c(""), c("extra beyond the headings")],
                        vec![c("\"\"")],
                    ],
                },
            ],
            &opts,
        );
        // A duplicate group code: the verdict must keep ONE entry whose
        // descriptor rows are the LAST section's and whose data rows are
        // both sections' — the walk's first-seen-wins semantics.
        assert_constructed_verdict_equals_reparse(
            &[
                proj(),
                GroupInput {
                    code: "LOCA".into(),
                    headings: vec!["LOCA_ID".into()],
                    units: None,
                    types: None,
                    rows: vec![vec![c("BH01")]],
                },
                GroupInput {
                    code: "LOCA".into(),
                    headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
                    units: None,
                    types: None,
                    rows: vec![vec![c("BH02"), c(1.5)]],
                },
            ],
            &opts,
        );
        // An empty group (no headings, no rows) beside a real one.
        assert_constructed_verdict_equals_reparse(
            &[
                proj(),
                GroupInput {
                    code: "ZZZZ".into(),
                    headings: vec![],
                    units: None,
                    types: None,
                    rows: vec![],
                },
            ],
            &opts,
        );
    }

    /// The differential under SYNTHESIS: the accumulator-derived catalogues
    /// stream out as sections whose re-parse agrees with the constructed
    /// verdict — TRAN stamp, UNIT/TYPE catalogues, PA-driven ABBR included.
    #[test]
    fn constructed_verdict_matches_a_real_parse_under_synthesis() {
        let opts = EmitOpts {
            synthesise_metadata: true,
            tran: Some(stamp()),
            ..EmitOpts::default()
        };
        assert_constructed_verdict_equals_reparse(
            &[
                proj(),
                GroupInput {
                    code: "LOCA".into(),
                    headings: vec!["LOCA_ID".into(), "LOCA_TYPE".into()],
                    units: None,
                    types: None,
                    rows: vec![vec![c("BH01"), c("TP")]],
                },
            ],
            &opts,
        );
    }

    /// The streamed accumulator against the batch computation it replaced:
    /// folding group-at-a-time yields the same synthesised groups as the
    /// whole-set reads (`chain(owned, synth)`) the batch pipeline made —
    /// pinned over a shape that exercises every catalogue (units from UNIT
    /// rows and PU columns, types, PA cells split on a caller `TRAN_RCON`).
    #[test]
    fn the_accumulator_reproduces_the_batch_synthesis() {
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let groups = [
            OwnedGroup {
                code: "TRAN".into(),
                headings: vec!["TRAN_RCON".into()],
                units: vec![String::new()],
                types: vec!["X".into()],
                rows: vec![vec!["~".into()]],
            },
            OwnedGroup {
                code: "LOCA".into(),
                headings: vec!["LOCA_ID".into(), "LOCA_TYPE".into(), "LOCA_XTRA".into()],
                units: vec![String::new(), String::new(), "m".into()],
                types: vec!["ID".into(), "PA".into(), "PU".into()],
                rows: vec![
                    vec!["BH01".into(), "TP~CP".into(), "kPa".into()],
                    vec!["BH02".into(), "TP".into(), String::new()],
                ],
            },
        ];
        let mut acc = SynthAccumulator::default();
        for g in &groups {
            acc.fold(g);
        }
        let synth = acc.synthesised(&dict, Some(&stamp()));

        // The batch reads, restated inline (the shape synthesise_metadata
        // had): TRAN present -> no TRAN synth; UNIT from all groups' unit
        // rows + PU cells; TYPE from all types + forced X; ABBR from PA
        // cells split on the caller's TRAN_RCON.
        let codes: Vec<&str> = synth.iter().map(|g| g.code.as_str()).collect();
        assert_eq!(codes, ["UNIT", "TYPE", "ABBR"], "TRAN present, not minted");
        let unit_rows: Vec<&str> = synth[0].rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(unit_rows, ["kPa", "m"], "UNIT row + PU column, blanks out");
        let type_rows: Vec<&str> = synth[1].rows.iter().map(|r| r[0].as_str()).collect();
        assert_eq!(type_rows, ["ID", "PA", "PU", "X"], "all types + forced X");
        let abbr_rows: Vec<(String, String)> = synth[2]
            .rows
            .iter()
            .map(|r| (r[0].clone(), r[1].clone()))
            .collect();
        assert_eq!(
            abbr_rows,
            [
                ("LOCA_TYPE".to_string(), "CP".to_string()),
                ("LOCA_TYPE".to_string(), "TP".to_string()),
            ],
            "PA cells split on the caller's ~ concatenator, deduped"
        );
    }

    fn proj() -> GroupInput {
        GroupInput {
            code: "PROJ".into(),
            headings: vec!["PROJ_ID".into(), "PROJ_NAME".into()],
            units: None,
            types: None,
            rows: vec![vec![c("P1"), c("Demo project")]],
        }
    }

    #[test]
    fn typed_numeric_is_canonical_by_construction() {
        // A typed Float under a 2DP heading formats to "12.30" with no
        // fixing needed.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c(12.3)]],
        };
        let r = emit_ags4(&[proj(), loca], &EmitOpts::default()).unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        assert!(
            text.contains("\"12.30\""),
            "expected canonical 2DP, got:\n{text}"
        );
    }

    #[test]
    fn dict_fills_unit_and_type() {
        // LOCA_GL is a 2DP heading with unit "m" in the standard dict;
        // we supply neither, so the dict fills both.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c(12.3)]],
        };
        let r = emit_ags4(
            &[proj(), loca],
            &EmitOpts {
                mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        assert!(
            text.contains("\"TYPE\",\"ID\",\"2DP\""),
            "dict TYPE fill, got:\n{text}"
        );
        assert!(
            text.contains("\"UNIT\",\"\",\"m\""),
            "dict UNIT fill, got:\n{text}"
        );
    }

    #[test]
    fn report_emits_string_verbatim() {
        // A string "12.3" under 2DP stays verbatim in Report mode.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c("12.3")]],
        };
        let r = emit_ags4(
            &[proj(), loca],
            &EmitOpts {
                mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        assert!(
            text.contains("\"12.3\""),
            "Report should not pad, got:\n{text}"
        );
        assert_eq!(r.fixes_applied, 0);
    }

    #[test]
    fn autofix_pads_a_string_numeric() {
        // The flagship default-mode behaviour: a string "12.3" under a
        // 2DP heading is non-compliant (Rule 8), so AutoFix's safe
        // ReformatNumeric fix pads it to "12.30" and reports the fix.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c("12.3")]],
        };
        let r = emit_ags4(&[proj(), loca], &EmitOpts::default()).unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        assert!(r.fixes_applied >= 1, "AutoFix should apply >=1 safe fix");
        assert!(
            text.contains("\"12.30\""),
            "AutoFix should pad to 2DP, got:\n{text}"
        );
        assert!(
            !text.contains("\"12.3\""),
            "the un-padded value should be gone, got:\n{text}"
        );
    }

    #[test]
    fn round_trips_through_the_parser() {
        // Build -> parse -> the groups + rows survive.
        let r = emit_ags4(
            &[proj()],
            &EmitOpts {
                mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .unwrap();
        let parsed = parse_bytes(&r.bytes, encoding_rs::UTF_8).unwrap();
        assert!(
            parsed.groups.contains_key("PROJ"),
            "PROJ group should survive the round trip"
        );
    }

    fn loca() -> GroupInput {
        GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c(12.3)]],
        }
    }

    /// Four of five is an ERROR naming the gaps, and zero of five is `None`.
    ///
    /// These are different answers to different questions and they used to be
    /// the same one: the old rule minted a stamp on issue+date and let the other
    /// three default to `""`, so "I forgot the producer" and "I meant not to
    /// stamp a TRAN" both produced a file, one of them silently wrong.
    #[test]
    fn a_partial_stamp_is_an_error_but_an_empty_one_is_simply_no_tran() {
        let none = TranStamp::from_parts(None, None, None, None, None, None, None).unwrap();
        assert!(none.is_none(), "nothing stated means no TRAN, not an error");

        let err = TranStamp::from_parts(
            Some("1".into()),
            Some("2026-07-30".into()),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err.missing, ["producer", "recipient", "status"]);
        // The message must name the gaps: a caller four-fifths of the way there
        // needs to know WHICH, not merely that something is wrong.
        let msg = err.to_string();
        for m in ["producer", "recipient", "status"] {
            assert!(msg.contains(m), "message should name {m}: {msg}");
        }

        // Whitespace is not a value — " " in a REQUIRED cell is still a Rule 10b
        // failure, so it counts as missing rather than sneaking past the check.
        let blank = TranStamp::from_parts(
            Some("1".into()),
            Some("2026-07-30".into()),
            Some("   ".into()),
            Some("r".into()),
            Some("s".into()),
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(blank.missing, ["producer"]);
    }

    /// `desc`/`rem` reach the stamp through the boundary adapter, stay outside
    /// the all-five rule, and cannot be stated on their own.
    ///
    /// They had no parameter here at all, so every surface that did not repeat
    /// the `with_description`/`with_remarks` trick after the fold dropped them
    /// silently — which was Python on both merge and build, and the CLI, which
    /// had no flags for them either (#730). The seam claimed to be the single
    /// owner of stamp policy; it now is.
    #[test]
    fn description_and_remarks_cross_the_boundary_without_joining_the_five() {
        let five = || {
            (
                Some("1".to_string()),
                Some("2026-07-30".to_string()),
                Some("p".to_string()),
                Some("r".to_string()),
                Some("s".to_string()),
            )
        };
        let (i, d, p, r, st) = five();
        let stamp = TranStamp::from_parts(i, d, p, r, st, Some("DESC".into()), Some("REM".into()))
            .unwrap()
            .expect("five parts is a stamp");
        assert_eq!(stamp.desc.as_deref(), Some("DESC"));
        assert_eq!(stamp.rem.as_deref(), Some("REM"));

        // Optional means optional: five parts and neither of these is complete.
        let (i, d, p, r, st) = five();
        let bare = TranStamp::from_parts(i, d, p, r, st, None, None)
            .unwrap()
            .expect("five parts is still a stamp without them");
        assert_eq!(bare.desc, None);
        assert_eq!(bare.rem, None);

        // Whitespace is not a value here either — consistent with the five.
        let (i, d, p, r, st) = five();
        let blankish =
            TranStamp::from_parts(i, d, p, r, st, Some("  ".into()), Some(String::new()))
                .unwrap()
                .unwrap();
        assert_eq!(blankish.desc, None, "whitespace is not a description");
        assert_eq!(blankish.rem, None, "empty is not a remark");

        // A description with no transmission behind it is a PARTIAL stamp, not
        // an absent one. Returning None would discard what the caller wrote and
        // read identically to not asking for a TRAN at all.
        let err = TranStamp::from_parts(None, None, None, None, None, Some("DESC".into()), None)
            .unwrap_err();
        assert_eq!(
            err.missing,
            ["issue", "date", "producer", "recipient", "status"]
        );

        // ...but nothing stated anywhere is still simply no TRAN.
        assert!(
            TranStamp::from_parts(None, None, None, None, None, None, None)
                .unwrap()
                .is_none()
        );
    }

    /// Every distinct value lands in its own heading, and the three derivable
    /// cells are filled by the emitter.
    ///
    /// Five same-typed fields in a row are a transposition waiting to happen —
    /// swap two and nothing fails to compile. Distinct sentinel values make a
    /// swap visible. The `TRAN_AGS`/`DLIM`/`RCON` assertions pin the other half
    /// of the contract: the caller CANNOT state them, so the engine must.
    #[test]
    fn each_tran_value_lands_in_its_own_heading_and_the_rest_are_derived() {
        let stamp = TranStamp::new("ISNO-1", "2026-07-30", "PROD-2", "RECV-3", "STAT-4")
            .with_description("DESC-5")
            .with_remarks("REM-6");
        let res = emit_ags4(
            &[proj(), loca()],
            &EmitOpts {
                synthesise_metadata: true,
                tran: Some(stamp),
                ..Default::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(res.bytes).unwrap();
        let tran = text
            .lines()
            .skip_while(|l| !l.contains("\"TRAN\""))
            .take(6)
            .collect::<Vec<_>>()
            .join("\n");

        let headings: Vec<&str> = tran
            .lines()
            .find(|l| l.starts_with("\"HEADING\""))
            .unwrap()
            .split(',')
            .map(|c| c.trim_matches('"'))
            .collect();
        let data: Vec<&str> = tran
            .lines()
            .find(|l| l.starts_with("\"DATA\""))
            .unwrap()
            .split(',')
            .map(|c| c.trim_matches('"'))
            .collect();
        let cell = |h: &str| {
            let i = headings.iter().position(|x| *x == h).unwrap_or_else(|| {
                panic!("TRAN should carry {h}:\n{tran}");
            });
            data[i]
        };

        assert_eq!(cell("TRAN_ISNO"), "ISNO-1");
        assert_eq!(cell("TRAN_DATE"), "2026-07-30");
        assert_eq!(cell("TRAN_PROD"), "PROD-2");
        assert_eq!(cell("TRAN_RECV"), "RECV-3");
        assert_eq!(cell("TRAN_STAT"), "STAT-4");
        assert_eq!(cell("TRAN_DESC"), "DESC-5");
        assert_eq!(cell("TRAN_REM"), "REM-6");
        // Derived, not stated: absent from TranStamp entirely.
        assert!(!cell("TRAN_AGS").is_empty(), "the edition must be filled");
        assert_eq!(cell("TRAN_DLIM"), "|");
        assert_eq!(cell("TRAN_RCON"), "+");
    }

    fn stamp() -> TranStamp {
        TranStamp::new(
            "1",
            "2026-07-30",
            "Acme Ground Engineering",
            "Client Ltd",
            "FINAL",
        )
    }

    /// Opting in WITH a stamp yields a valid file in one call. The
    /// `..Default::default()` spread is deliberate: this must keep passing if
    /// other defaults move.
    #[test]
    fn autofix_synthesises_missing_metadata_groups_when_asked() {
        let opts = EmitOpts {
            synthesise_metadata: true,
            tran: Some(stamp()),
            ..EmitOpts::default()
        };
        let r = emit_ags4(&[proj(), loca()], &opts).unwrap();
        let text = String::from_utf8(r.bytes.clone()).unwrap();
        for g in ["TRAN", "UNIT", "TYPE"] {
            assert!(
                text.contains(&format!("\"GROUP\",\"{g}\"")),
                "opted-in AutoFix should synthesise the {g} group, got:\n{text}"
            );
        }
        // The synthesized file is valid — no error-severity findings.
        let errors = r
            .findings
            .values()
            .flatten()
            .filter(|f| f.severity == Severity::Error)
            .count();
        assert_eq!(
            errors, 0,
            "synthesised build should be error-free, findings:\n{:?}",
            r.findings
        );
    }

    /// The TRAN row carries the CALLER's values, not placeholders.
    ///
    /// Guards the actual defect: a stub reading `TBC`/`1900-01-01` SATISFIES
    /// Rule 14, so a recipient cannot distinguish an invented transmission from
    /// a real one. Asserting the absence of those two literals is the point —
    /// if they ever come back, this file is lying again and passing while it
    /// does.
    #[test]
    fn synthesised_tran_carries_caller_values_not_placeholders() {
        let opts = EmitOpts {
            synthesise_metadata: true,
            tran: Some(stamp()),
            ..EmitOpts::default()
        };
        let r = emit_ags4(&[proj(), loca()], &opts).unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        for v in [
            "2026-07-30",
            "Acme Ground Engineering",
            "Client Ltd",
            "FINAL",
        ] {
            assert!(text.contains(v), "TRAN should carry {v:?}, got:\n{text}");
        }
        assert!(
            !text.contains("TBC") && !text.contains("1900-01-01"),
            "no placeholder may survive into a synthesised TRAN, got:\n{text}"
        );
    }

    /// Synthesis WITHOUT a stamp mints no TRAN and lets Rule 14 report it.
    ///
    /// The honest half of the contract: the engine cannot know who transmitted
    /// what to whom, so it declines to say. A caller who wants a clean file
    /// supplies the transmission; a caller who does not gets told what is
    /// missing rather than handed a fiction that validates.
    #[test]
    fn synthesis_without_a_stamp_omits_tran_and_reports_rule_14() {
        let opts = EmitOpts {
            synthesise_metadata: true,
            tran: None,
            ..EmitOpts::default()
        };
        let r = emit_ags4(&[proj(), loca()], &opts).unwrap();
        let text = String::from_utf8(r.bytes.clone()).unwrap();
        assert!(
            !text.contains("\"GROUP\",\"TRAN\""),
            "no TRAN may be invented without a stamp, got:\n{text}"
        );
        assert!(
            r.findings.contains_key("AGS Format Rule 14"),
            "the missing TRAN must be REPORTED, not silently absent: {:?}",
            r.findings
        );
        // UNIT/TYPE are still derivable, so they are still minted.
        for g in ["UNIT", "TYPE"] {
            assert!(
                text.contains(&format!("\"GROUP\",\"{g}\"")),
                "{g} is derivable and should still be synthesised, got:\n{text}"
            );
        }
    }

    /// The NEW default: `AutoFix` still fixes what the caller wrote, but does not
    /// invent groups they didn't. This is the behaviour change, so it is pinned
    /// directly rather than inferred from the opt-in test's absence.
    #[test]
    fn autofix_does_not_synthesise_by_default() {
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c(12.3)]],
        };
        let r = emit_ags4(&[proj(), loca], &EmitOpts::default()).unwrap();
        let text = String::from_utf8(r.bytes.clone()).unwrap();
        for g in ["TRAN", "UNIT", "TYPE"] {
            assert!(
                !text.contains(&format!("\"GROUP\",\"{g}\"")),
                "default AutoFix must not mint {g}, got:\n{text}"
            );
        }
        // And the gaps are REPORTED rather than silently filled — the whole
        // point of opting in is that the caller can see what they declined.
        let labels: Vec<&str> = r.findings.keys().map(String::as_str).collect();
        assert!(
            labels.iter().any(|l| l.contains("Rule 14"))
                && labels.iter().any(|l| l.contains("Rule 15"))
                && labels.iter().any(|l| l.contains("Rule 17")),
            "missing catalogs should surface as findings, got: {labels:?}"
        );
    }

    #[test]
    fn autofix_synthesises_abbr_for_pa_codes_when_asked() {
        // A PA-coded value (LOCA_TYPE is a PA heading) needs an ABBR definition
        // (Rule 16); AutoFix mints ABBR covering exactly the codes used, and the
        // file is valid.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_TYPE".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c("TP")]],
        };
        let opts = EmitOpts {
            synthesise_metadata: true,
            // Stamped, so the file can reach error-free — this test is about
            // ABBR, and an unstamped build would fail on Rule 14 for unrelated
            // reasons (see `synthesis_without_a_stamp_omits_tran_...`).
            tran: Some(stamp()),
            ..EmitOpts::default()
        };
        let r = emit_ags4(&[proj(), loca], &opts).unwrap();
        let text = String::from_utf8(r.bytes.clone()).unwrap();
        assert!(
            text.contains("\"GROUP\",\"ABBR\""),
            "opted-in AutoFix should synthesise ABBR, got:\n{text}"
        );
        assert!(
            text.contains("\"DATA\",\"LOCA_TYPE\",\"TP\""),
            "ABBR should define LOCA_TYPE/TP, got:\n{text}"
        );
        let errors = r
            .findings
            .values()
            .flatten()
            .filter(|f| f.severity == Severity::Error)
            .count();
        assert_eq!(
            errors, 0,
            "PA-coded build should be error-free, findings:\n{:?}",
            r.findings
        );
    }

    #[test]
    fn report_mode_does_not_synthesize() {
        // Report mode leaves the file as given — a missing TRAN is not minted.
        let r = emit_ags4(
            &[proj()],
            &EmitOpts {
                mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(r.bytes).unwrap();
        assert!(
            !text.contains("\"GROUP\",\"TRAN\""),
            "Report must not synthesize TRAN, got:\n{text}"
        );
    }

    #[test]
    fn strict_mode_rejects_error_findings_and_accepts_a_clean_file() {
        // Pins the Strict gate: `count(severity == Error)` and `errors > 0`.
        // Strict does NOT synthesize the mandatory metadata groups (AutoFix
        // does), so a genuinely clean fixture must carry PROJ + TRAN + the
        // UNIT/TYPE catalogs itself (mirroring `synthesise_metadata`).
        let strict = EmitOpts {
            mode: EmitMode::Strict,
            ..Default::default()
        };
        let cat = |code: &str, key: &str, desc: &str, syms: &[&str]| GroupInput {
            code: code.into(),
            headings: vec![key.into(), desc.into()],
            units: Some(vec![String::new(), String::new()]),
            types: Some(vec!["X".into(), "X".into()]),
            rows: syms.iter().map(|s| vec![c(*s), c(*s)]).collect(),
        };
        let tran = || GroupInput {
            code: "TRAN".into(),
            headings: [
                "TRAN_ISNO",
                "TRAN_DATE",
                "TRAN_PROD",
                "TRAN_STAT",
                "TRAN_AGS",
                "TRAN_RECV",
                "TRAN_DLIM",
                "TRAN_RCON",
            ]
            .map(String::from)
            .to_vec(),
            units: Some(
                ["", "yyyy-mm-dd", "", "", "", "", "", ""]
                    .map(String::from)
                    .to_vec(),
            ),
            types: Some(
                ["X", "DT", "X", "X", "X", "X", "X", "X"]
                    .map(String::from)
                    .to_vec(),
            ),
            rows: vec![vec![
                c("1"),
                c("1900-01-01"),
                c("TBC"),
                c("TBC"),
                c("4.1.1"),
                c("TBC"),
                c("|"),
                c("+"),
            ]],
        };
        let clean_file = || {
            vec![
                proj(),
                tran(),
                cat("UNIT", "UNIT_UNIT", "UNIT_DESC", &["yyyy-mm-dd"]),
                cat("TYPE", "TYPE_TYPE", "TYPE_DESC", &["DT", "ID", "X"]),
            ]
        };
        let clean = emit_ags4(&clean_file(), &strict);
        assert!(
            clean.is_ok(),
            "a complete file passes Strict, err: {:?}",
            clean.as_ref().err()
        );

        // The same file plus a duplicate-KEY LOCA (Rule 10a error) → Strict Err.
        let mut with_dup = clean_file();
        with_dup.push(GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into()],
            units: None,
            types: None,
            rows: vec![vec![c("BH01")], vec![c("BH01")]],
        });
        let bad = emit_ags4(&with_dup, &strict);
        assert!(
            matches!(bad, Err(EmitError::Invalid(_))),
            "error findings ⇒ Strict Err (was_ok={})",
            bad.is_ok()
        );
    }

    #[test]
    fn explicit_unit_override_wins_but_a_blank_one_falls_back_to_the_dict() {
        // LOCA_GL carries UNIT "m" in the dict. An explicit non-blank UNIT
        // override must WIN; a blank ("") override must FALL BACK to the dict —
        // pinning the `Some(s) if !s.trim().is_empty()` guard both ways.
        let base = |units: Option<Vec<String>>| GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
            units,
            types: None,
            rows: vec![vec![c("BH01"), c(12.3)]],
        };
        let opts = EmitOpts {
            mode: EmitMode::Report,
            ..Default::default()
        };
        let win = emit_ags4(
            &[proj(), base(Some(vec![String::new(), "furlong".into()]))],
            &opts,
        )
        .unwrap();
        let tw = String::from_utf8(win.bytes).unwrap();
        assert!(
            tw.contains("\"UNIT\",\"\",\"furlong\""),
            "explicit override wins, got:\n{tw}"
        );
        let fallback = emit_ags4(
            &[proj(), base(Some(vec![String::new(), String::new()]))],
            &opts,
        )
        .unwrap();
        let tf = String::from_utf8(fallback.bytes).unwrap();
        assert!(
            tf.contains("\"UNIT\",\"\",\"m\""),
            "blank override falls back to the dict, got:\n{tf}"
        );
    }

    #[test]
    fn report_mode_emits_a_dt_string_verbatim_not_through_ags4_str() {
        // A DT-typed STRING with a `T00:00:00` tail: `ags4_str` would drop the
        // midnight time, but format_cell's `Value::String` arm emits strings
        // verbatim so the validity MODE owns canonicalisation. Report keeps it.
        let loca = GroupInput {
            code: "LOCA".into(),
            headings: vec!["LOCA_ID".into(), "LOCA_STAR".into()], // LOCA_STAR is DT
            units: None,
            types: None,
            rows: vec![vec![c("BH01"), c("2023-02-22T00:00:00")]],
        };
        let r = emit_ags4(
            &[proj(), loca],
            &EmitOpts {
                mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .unwrap();
        let t = String::from_utf8(r.bytes).unwrap();
        assert!(
            t.contains("\"2023-02-22T00:00:00\""),
            "DT string stays verbatim in Report, got:\n{t}"
        );
    }

    #[test]
    fn collect_units_gathers_unit_rows_and_pu_columns_only() {
        // 379: only PU-typed columns contribute data-cell units (`== "PU"`).
        // 382: blank PU cells are skipped (`!v.is_empty()`).
        let g = OwnedGroup {
            code: "X".into(),
            headings: vec!["A".into(), "B".into()],
            units: vec!["m".into(), "UNIT".into()], // "m" kept; literal "UNIT" excluded
            types: vec!["PU".into(), "X".into()],   // col A is PU, col B is not
            rows: vec![
                vec!["kPa".into(), "notpu".into()], // kPa from the PU col; B ignored
                vec![String::new(), "blankpu".into()], // blank PU cell skipped
            ],
        };
        let units = collect_units(std::iter::once(&g));
        assert!(units.contains("m"), "UNIT-row value: {units:?}");
        assert!(units.contains("kPa"), "PU-column value: {units:?}");
        assert!(
            !units.contains("notpu"),
            "non-PU column must not contribute: {units:?}"
        );
        assert!(
            !units.contains(""),
            "blank PU cell must be skipped: {units:?}"
        );
        assert!(
            !units.contains("UNIT"),
            "the literal UNIT header excluded: {units:?}"
        );
    }

    #[test]
    fn collect_types_excludes_blanks_and_the_literal_type() {
        // 400: `!t.is_empty() && t != "TYPE"` — a blank or the literal "TYPE"
        // is dropped; flipping `&&` to `||` would admit them.
        let g = OwnedGroup {
            code: "X".into(),
            headings: vec!["A".into(), "B".into(), "C".into()],
            units: vec![String::new(), String::new(), String::new()],
            types: vec!["ID".into(), String::new(), "TYPE".into()],
            rows: vec![],
        };
        let types: Vec<String> = collect_types(std::iter::once(&g)).into_iter().collect();
        assert_eq!(types, vec!["ID".to_string()], "only the real type survives");
    }

    #[test]
    fn tran_rcon_reads_the_concatenator_else_none() {
        // Not a TRAN group → no contribution (the accumulator then defaults
        // to the AGS standard "+").
        let loca = OwnedGroup {
            code: "LOCA".into(),
            headings: vec![],
            units: vec![],
            types: vec![],
            rows: vec![],
        };
        assert_eq!(tran_rcon(&loca), None);
        // A TRAN group with an explicit non-blank TRAN_RCON wins — pins the
        // code=="TRAN", heading=="TRAN_RCON", first-row and non-blank guards
        // all at once.
        let tran = OwnedGroup {
            code: "TRAN".into(),
            headings: vec!["TRAN_DLIM".into(), "TRAN_RCON".into()],
            units: vec![String::new(), String::new()],
            types: vec!["X".into(), "X".into()],
            rows: vec![vec!["|".into(), "~".into()]],
        };
        assert_eq!(tran_rcon(&tran).as_deref(), Some("~"));
        // A blank TRAN_RCON is no contribution either — the accumulator must
        // keep looking rather than adopt an empty concatenator.
        let blank = OwnedGroup {
            code: "TRAN".into(),
            headings: vec!["TRAN_RCON".into()],
            units: vec![String::new()],
            types: vec!["X".into()],
            rows: vec![vec!["  ".into()]],
        };
        assert_eq!(tran_rcon(&blank), None);
    }
}
