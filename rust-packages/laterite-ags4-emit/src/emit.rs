//! The host-agnostic AGS4 emit orchestrator.
//!
//! Turns per-group cell data (typed from frames/Arrow, or strings from
//! browser JSON) into valid AGS4 bytes:
//!
//!   1. resolve each heading's UNIT/TYPE — **hybrid**: the caller's
//!      explicit value wins, else the per-edition standard dictionary fills,
//!      else `""` / `"X"`;
//!   2. format each cell — typed values via `laterite_types::ags4_str` (the
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

use laterite_ags4_validator::dict::Dictionary;
use laterite_ags4_validator::findings::{Findings, Severity};
use laterite_ags4_validator::fixes::{Fix, FixRisk, apply_fixes, compute_fixes};
use laterite_ags4_validator::parse::{ParsedFile, parse_bytes};
use laterite_ags4_validator::{CheckOptions, DictVersion, WorldScope, check_parsed};
use laterite_types::ags4_str;
use serde_json::Value;
use std::collections::BTreeSet;

use crate::error::EmitError;
use crate::writer::{EmitGroup, write_ags4};

/// One group's data to emit. `units` / `types` are optional per-heading
/// overrides — `None` (or a blank entry) means "fill from the dictionary".
/// `rows` cells are JSON values: typed (Number / Bool / Null) from frames
/// or Arrow, or strings from browser JSON.
pub struct GroupInput {
    pub code: String,
    pub headings: Vec<String>,
    pub units: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub rows: Vec<Vec<Value>>,
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

/// Caller-supplied metadata for a synthesised TRAN row.
///
/// Lives here, the lowest crate that needs it, so `laterite-ags4-merge` and the
/// emit path describe a transmission with ONE type rather than two that drift.
/// `merge` re-exports it, so its public API is unchanged.
///
/// Every field is the caller's to state. The engine can derive `ags` from the
/// edition, but nothing else here is derivable — which is precisely why an
/// absent stamp means "emit no TRAN" rather than "emit a guess".
#[derive(Debug, Clone, Default)]
pub struct TranStamp {
    pub isno: String,
    pub date: String,
    pub prod: String,
    pub recv: String,
    pub stat: String,
    pub ags: String,
}

impl TranStamp {
    /// Fold a surface's five optional TRAN arguments into a stamp, or `None`.
    ///
    /// **One rule, one place.** A stamp is minted only when BOTH an issue number
    /// and a date are supplied — the two fields that make a transmission
    /// identifiable at all, and both REQUIRED by the dictionary. Anything less
    /// would produce a TRAN missing its own mandatory fields, which is the class
    /// of half-true record this whole change exists to stop.
    ///
    /// This lived as a private helper on each surface, and they had already
    /// drifted: `merge` required issue+date while the browser's `build_ags4`
    /// accepted any one of the five. Every surface now folds its arguments the
    /// same way, so "what counts as enough to stamp a TRAN" cannot answer
    /// differently depending on which door you came through.
    #[must_use]
    pub fn from_parts(
        isno: Option<String>,
        date: Option<String>,
        prod: Option<String>,
        recv: Option<String>,
        stat: Option<String>,
        ags: String,
    ) -> Option<TranStamp> {
        match (isno, date) {
            (Some(isno), Some(date)) => Some(TranStamp {
                isno,
                date,
                prod: prod.unwrap_or_default(),
                recv: recv.unwrap_or_default(),
                stat: stat.unwrap_or_default(),
                ags,
            }),
            _ => None,
        }
    }
}

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

    // --- steps 1–2: resolve UNIT/TYPE (hybrid) + format cells ---------
    // `OwnedGroup` holds Strings; `EmitGroup` borrows them for the write.
    let mut owned: Vec<OwnedGroup> = groups
        .iter()
        .map(|g| {
            let units: Vec<String> = (0..g.headings.len())
                .map(|i| {
                    resolve_meta(g.units.as_ref(), i, || {
                        dict_unit(&dict, &g.code, &g.headings[i])
                    })
                })
                .collect();
            let types: Vec<String> = (0..g.headings.len())
                .map(|i| {
                    resolve_meta(g.types.as_ref(), i, || {
                        dict_type(&dict, &g.code, &g.headings[i])
                    })
                })
                .collect();
            let rows: Vec<Vec<String>> = g
                .rows
                .iter()
                .map(|row| {
                    row.iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            format_cell(cell, types.get(i).map_or("X", String::as_str))
                        })
                        .collect()
                })
                .collect();
            OwnedGroup {
                code: g.code.clone(),
                headings: g.headings.clone(),
                units,
                types,
                rows,
            }
        })
        .collect();

    // --- step 2.5: synthesize missing mandatory metadata groups -------
    // AutoFix only: a data-only build (notably a typed PROJ graph, which
    // can't reach the parentless root-metadata groups) still yields a valid
    // file — mint UNIT/TYPE (derived from the data) and ABBR (when PA codes are
    // used) for whichever are absent, plus TRAN when the caller stamped one.
    // PROJ is never synthesized (real project identity), so a missing PROJ stays
    // a Rule 13 finding.
    if opts.mode == EmitMode::AutoFix && opts.synthesise_metadata {
        let synth = synthesise_metadata(&owned, &dict, opts.tran.as_ref());
        owned.extend(synth);
    }

    // --- step 3: write the sections -----------------------------------
    let views: Vec<EmitGroup<'_>> = owned
        .iter()
        .map(|g| EmitGroup {
            code: &g.code,
            headings: g.headings.iter().map(String::as_str).collect(),
            units: g.units.iter().map(String::as_str).collect(),
            types: g.types.iter().map(String::as_str).collect(),
            rows: g.rows.clone(),
        })
        .collect();
    let mut bytes: Vec<u8> = Vec::new();
    write_ags4(&mut bytes, &views)?;

    // --- step 4: apply the validity mode ------------------------------
    // Keep the ParsedFile: `AutoFix` needs it for `compute_fixes`, and it used
    // to re-parse the SAME bytes to get a second copy of it.
    let (parsed, found) = validate(&bytes, opts.edition)?;
    match opts.mode {
        EmitMode::Report => Ok(EmitResult {
            bytes,
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
                    bytes,
                    findings: found,
                    applied: Vec::new(),
                    fixes_applied: 0,
                })
            }
        }
        EmitMode::AutoFix => {
            // `parsed` comes from step 4's validate — these are the very bytes
            // it just parsed, so re-parsing them produced an identical
            // ParsedFile at full cost. Borrowed, not owned: `apply_fixes` wants
            // `&str`, so an all-ASCII emit (the normal case) does not copy the
            // whole output either.
            let text = String::from_utf8_lossy(&bytes);
            let safe: Vec<_> = compute_fixes(&parsed, &found)
                .into_iter()
                .filter(|f| f.risk == FixRisk::Safe)
                .collect();
            if safe.is_empty() {
                return Ok(EmitResult {
                    bytes,
                    findings: found,
                    applied: Vec::new(),
                    fixes_applied: 0,
                });
            }
            // The emitter never writes a BOM, so has_bom = false.
            let fixed = apply_fixes(&text, false, &safe);
            let fixed_bytes = fixed.into_bytes();
            // Residual findings on the *fixed* output — a genuinely different
            // document, so this parse is real work, not a repeat.
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

/// Owned mirror of an emit group — `EmitGroup` borrows `&str`s, so we
/// build owned Strings first then borrow them for the write.
struct OwnedGroup {
    code: String,
    headings: Vec<String>,
    units: Vec<String>,
    types: Vec<String>,
    rows: Vec<Vec<String>>,
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
fn format_cell(value: &Value, ags_type: &str) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => ags4_str(value, ags_type),
    }
}

/// Parse bytes + run all rules at the given edition, returning the parse AND
/// the findings. Handing back the `ParsedFile` is the point: it is the exact
/// input the rules ran against, so a caller that needs it (`AutoFix`, for
/// `compute_fixes`) reuses it instead of re-deriving it from the same bytes.
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

/// Under `AutoFix`, synthesize whichever mandatory metadata catalog group is
/// absent so a data-only build still yields a valid file. UNIT and TYPE are
/// pure derivations of the data; ABBR is minted when (and only when) the data
/// uses PA picklist codes (Rule 16); TRAN is written only from a caller-supplied
/// `TranStamp`, never invented. PROJ is deliberately never synthesized — it
/// carries real project identity, not derivable metadata, so a missing PROJ
/// stays a Rule 13 finding.
fn synthesise_metadata(
    owned: &[OwnedGroup],
    dict: &Dictionary,
    tran: Option<&TranStamp>,
) -> Vec<OwnedGroup> {
    let present: BTreeSet<&str> = owned.iter().map(|g| g.code.as_str()).collect();
    let mut synth: Vec<OwnedGroup> = Vec::new();

    // TRAN first: its DT `TRAN_DATE` introduces the `yyyy-mm-dd` unit and the
    // `DT` type that the UNIT/TYPE catalogs below must then cover.
    //
    // Only when the caller supplied one. Without a stamp we emit NO TRAN and let
    // Rule 14 report it — see `EmitOpts::tran` for why a placeholder is worse
    // than an absence.
    if !present.contains("TRAN")
        && let Some(t) = tran
    {
        synth.push(synth_tran(dict, t));
    }
    // UNIT: one row per distinct unit used across all groups (Rule 15).
    //
    // Skipped when nothing uses a unit. An empty catalog is not a neutral
    // no-op — a group with no DATA rows is itself a Rule 2 error, so minting
    // one would trade a Rule 15 finding for a Rule 2 finding and call it
    // synthesis. This became reachable when TRAN stopped being minted
    // unconditionally: its `yyyy-mm-dd` on TRAN_DATE was quietly the thing
    // guaranteeing at least one unit existed.
    if !present.contains("UNIT") {
        let units = collect_units(owned.iter().chain(synth.iter()));
        if !units.is_empty() {
            synth.push(synth_catalog("UNIT", "UNIT_UNIT", "UNIT_DESC", &units));
        }
    }
    // TYPE: one row per distinct type code used (Rule 17). Force `X` — every
    // synthesized metadata group is all-`X`, so the catalog must self-cover.
    if !present.contains("TYPE") {
        let mut types = collect_types(owned.iter().chain(synth.iter()));
        types.insert("X".to_string());
        synth.push(synth_catalog("TYPE", "TYPE_TYPE", "TYPE_DESC", &types));
    }
    // ABBR: only when the data uses PA picklist codes (Rule 16) — one row per
    // distinct (heading, code), description from the standard ABBR table
    // (fallback: the code itself). PA values are split on the concatenator the
    // same way Rule 16 reads it (the file's `TRAN_RCON`, else the AGS `+`).
    if !present.contains("ABBR") {
        let abbrs = collect_abbreviations(owned, dict, &concatenator(owned));
        if !abbrs.is_empty() {
            synth.push(synth_abbr(abbrs));
        }
    }
    synth
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
    OwnedGroup {
        code: "TRAN".to_string(),
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
        units: ["", "yyyy-mm-dd", "", "", "", "", "", ""]
            .map(String::from)
            .to_vec(),
        types: ["X", "DT", "X", "X", "X", "X", "X", "X"]
            .map(String::from)
            .to_vec(),
        rows: vec![vec![
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
        ]],
    }
}

/// The PA concatenator (Rule 16a) — the file's `TRAN_RCON` if a TRAN group was
/// supplied, else the AGS standard `+` (also what `synth_tran` writes).
fn concatenator(owned: &[OwnedGroup]) -> String {
    for g in owned {
        if g.code == "TRAN" {
            if let Some(ci) = g.headings.iter().position(|h| h == "TRAN_RCON") {
                if let Some(v) = g.rows.first().and_then(|r| r.get(ci)) {
                    let v = v.trim();
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
    }
    "+".to_string()
}

/// Distinct `(heading, code, desc)` triples for every abbreviation used in a
/// `PA`-typed column (Rule 16). Each cell is split on `concat` (matching Rule
/// 16a); the description is the standard ABBR table's, falling back to the code.
fn collect_abbreviations(
    groups: &[OwnedGroup],
    dict: &Dictionary,
    concat: &str,
) -> Vec<[String; 3]> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for g in groups {
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() == "PA" {
                let heading = g.headings.get(ci).map_or("", String::as_str);
                for row in &g.rows {
                    if let Some(cell) = row.get(ci) {
                        for code in cell.split(concat) {
                            let code = code.trim();
                            if !code.is_empty() {
                                seen.insert((heading.to_string(), code.to_string()));
                            }
                        }
                    }
                }
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
    use serde_json::json;

    fn proj() -> GroupInput {
        GroupInput {
            code: "PROJ".into(),
            headings: vec!["PROJ_ID".into(), "PROJ_NAME".into()],
            units: None,
            types: None,
            rows: vec![vec![json!("P1"), json!("Demo project")]],
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
            rows: vec![vec![json!("BH01"), json!(12.3)]],
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
            rows: vec![vec![json!("BH01"), json!(12.3)]],
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
            rows: vec![vec![json!("BH01"), json!("12.3")]],
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
            rows: vec![vec![json!("BH01"), json!("12.3")]],
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
            rows: vec![vec![json!("BH01"), json!(12.3)]],
        }
    }

    fn stamp() -> TranStamp {
        TranStamp {
            isno: "1".into(),
            date: "2026-07-30".into(),
            prod: "Acme Ground Engineering".into(),
            recv: "Client Ltd".into(),
            stat: "FINAL".into(),
            ags: String::new(),
        }
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
            rows: vec![vec![json!("BH01"), json!(12.3)]],
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
            rows: vec![vec![json!("BH01"), json!("TP")]],
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
            rows: syms.iter().map(|s| vec![json!(s), json!(s)]).collect(),
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
                json!("1"),
                json!("1900-01-01"),
                json!("TBC"),
                json!("TBC"),
                json!("4.1.1"),
                json!("TBC"),
                json!("|"),
                json!("+"),
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
            rows: vec![vec![json!("BH01")], vec![json!("BH01")]],
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
            rows: vec![vec![json!("BH01"), json!(12.3)]],
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
            rows: vec![vec![json!("BH01"), json!("2023-02-22T00:00:00")]],
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
    fn concatenator_reads_tran_rcon_else_defaults_to_plus() {
        // No TRAN group → the AGS standard "+".
        let loca = OwnedGroup {
            code: "LOCA".into(),
            headings: vec![],
            units: vec![],
            types: vec![],
            rows: vec![],
        };
        assert_eq!(concatenator(&[loca]), "+");
        // A TRAN group with an explicit non-blank TRAN_RCON wins — pins the
        // code=="TRAN" (464), heading=="TRAN_RCON" (465), non-blank (468) and
        // whole-return (463) all at once.
        let tran = OwnedGroup {
            code: "TRAN".into(),
            headings: vec!["TRAN_DLIM".into(), "TRAN_RCON".into()],
            units: vec![String::new(), String::new()],
            types: vec!["X".into(), "X".into()],
            rows: vec![vec!["|".into(), "~".into()]],
        };
        assert_eq!(concatenator(&[tran]), "~", "explicit TRAN_RCON wins");
    }
}
