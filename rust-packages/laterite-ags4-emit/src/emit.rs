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
//!      returns it with findings, AutoFix applies the *safe* mechanical
//!      fixes (the same machinery the web "fix-all-safe" button uses) and
//!      returns the compliant-where-fixable bytes plus residual findings.
//!
//! Steps 1–3 are pure formatting; step 4 reuses the validator's shipped
//! parse / `run_all` / `compute_fixes` / `apply_fixes` — no new fix logic.

use laterite_ags4_validator::dict::Dictionary;
use laterite_ags4_validator::findings::{Findings, Severity};
use laterite_ags4_validator::fixes::{FixRisk, apply_fixes, compute_fixes};
use laterite_ags4_validator::parse::parse_bytes;
use laterite_ags4_validator::{CheckOptions, DictVersion, rules};
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

/// Emit options. `edition` selects which AGS4 standard dictionary fills
/// UNIT/TYPE and which rule set validity is judged against.
#[derive(Debug, Clone)]
pub struct EmitOpts {
    pub mode: EmitMode,
    pub edition: DictVersion,
}

impl Default for EmitOpts {
    fn default() -> Self {
        // AutoFix + 4.1.1 are the resolved project defaults (see the
        // ags4-output design page).
        EmitOpts {
            mode: EmitMode::AutoFix,
            edition: DictVersion::V4_1_1,
        }
    }
}

/// The emit result. `findings` are on the *returned* bytes (so, post-fix
/// for AutoFix) — empty means clean. `fixes_applied` is the count of safe
/// fixes AutoFix applied (0 for Strict/Report).
pub struct EmitResult {
    pub bytes: Vec<u8>,
    pub findings: Findings,
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
    // file — mint UNIT/TYPE (derived from the data), a placeholder TRAN, and
    // ABBR (when PA codes are used) for whichever are absent. PROJ is never
    // synthesized (real project identity), so a missing PROJ stays a Rule 13 finding.
    if opts.mode == EmitMode::AutoFix {
        let synth = synthesize_metadata(&owned, &dict);
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
    let found = validate(&bytes, opts.edition)?;
    match opts.mode {
        EmitMode::Report => Ok(EmitResult {
            bytes,
            findings: found,
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
                    fixes_applied: 0,
                })
            }
        }
        EmitMode::AutoFix => {
            // Re-parse for compute_fixes (it needs the ParsedFile + the
            // findings computed against it).
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let parsed = parse_bytes(&bytes, encoding_rs::UTF_8)
                .map_err(|e| EmitError::Reparse(e.to_string()))?;
            let safe: Vec<_> = compute_fixes(&parsed, &found)
                .into_iter()
                .filter(|f| f.risk == FixRisk::Safe)
                .collect();
            if safe.is_empty() {
                return Ok(EmitResult {
                    bytes,
                    findings: found,
                    fixes_applied: 0,
                });
            }
            // The emitter never writes a BOM, so has_bom = false.
            let fixed = apply_fixes(&text, false, &safe);
            let fixed_bytes = fixed.into_bytes();
            // Residual findings on the *fixed* output.
            let residual = validate(&fixed_bytes, opts.edition)?;
            Ok(EmitResult {
                bytes: fixed_bytes,
                findings: residual,
                fixes_applied: safe.len(),
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
/// AutoFix's text fixer pads/normalises, Strict rejects).
fn format_cell(value: &Value, ags_type: &str) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => ags4_str(value, ags_type),
    }
}

/// Parse bytes + run all rules at the given edition, returning findings.
fn validate(bytes: &[u8], edition: DictVersion) -> Result<Findings, EmitError> {
    let parsed =
        parse_bytes(bytes, encoding_rs::UTF_8).map_err(|e| EmitError::Reparse(e.to_string()))?;
    let dict = Dictionary::bundled(edition);
    let opts = CheckOptions {
        dict_version: Some(edition),
        ..CheckOptions::default()
    };
    let mut found = Findings::new();
    rules::run_all(&parsed, &dict, &opts, None, &mut found);
    Ok(found)
}

/// Under AutoFix, synthesize whichever mandatory metadata catalog group is
/// absent so a data-only build still yields a valid file. UNIT and TYPE are
/// pure derivations of the data; TRAN is a placeholder stub the caller
/// overwrites; ABBR is minted when (and only when) the data uses PA picklist
/// codes (Rule 16). PROJ is deliberately never synthesized — it carries real
/// project identity, not derivable metadata, so a missing PROJ stays a Rule 13
/// finding.
fn synthesize_metadata(owned: &[OwnedGroup], dict: &Dictionary) -> Vec<OwnedGroup> {
    let present: BTreeSet<&str> = owned.iter().map(|g| g.code.as_str()).collect();
    let mut synth: Vec<OwnedGroup> = Vec::new();

    // TRAN first: its DT `TRAN_DATE` introduces the `yyyy-mm-dd` unit and the
    // `DT` type that the UNIT/TYPE catalogs below must then cover.
    if !present.contains("TRAN") {
        synth.push(synth_tran(dict));
    }
    // UNIT: one row per distinct unit used across all groups (Rule 15).
    if !present.contains("UNIT") {
        let units = collect_units(owned.iter().chain(synth.iter()));
        synth.push(synth_catalog("UNIT", "UNIT_UNIT", "UNIT_DESC", &units));
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

/// A minimal valid TRAN stub. `TRAN_AGS` is the edition's expected value (no
/// unrecognised-edition warning); `TRAN_DLIM`/`TRAN_RCON` are the AGS standard
/// `"|"`/`"+"`. The REQUIRED transmission fields the build can't know
/// (producer, recipient, status) are `"TBC"` placeholders and `TRAN_DATE` a
/// fixed placeholder date — all meant to be overwritten by the caller.
fn synth_tran(dict: &Dictionary) -> OwnedGroup {
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
            "1".to_string(),
            "1900-01-01".to_string(),
            "TBC".to_string(),
            "TBC".to_string(),
            dict.tran_ags().to_string(),
            "TBC".to_string(),
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
                let heading = g.headings.get(ci).map(String::as_str).unwrap_or("");
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
                .map(str::to_string)
                .unwrap_or_else(|| c.clone());
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

    #[test]
    fn autofix_synthesizes_missing_metadata_groups() {
        // The flagship enhancement: a data-only build (PROJ + LOCA, no
        // TRAN/UNIT/TYPE) under the default AutoFix mode synthesizes the
        // missing root-metadata groups and comes back fully valid.
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
                text.contains(&format!("\"GROUP\",\"{g}\"")),
                "AutoFix should synthesize the {g} group, got:\n{text}"
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
            "synthesized build should be error-free, findings:\n{:?}",
            r.findings
        );
    }

    #[test]
    fn autofix_synthesizes_abbr_for_pa_codes() {
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
        let r = emit_ags4(&[proj(), loca], &EmitOpts::default()).unwrap();
        let text = String::from_utf8(r.bytes.clone()).unwrap();
        assert!(
            text.contains("\"GROUP\",\"ABBR\""),
            "AutoFix should synthesize ABBR, got:\n{text}"
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
}
