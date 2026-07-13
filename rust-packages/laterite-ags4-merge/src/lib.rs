//! AGS4 merge — reconcile N deliveries of one project into a single file.
//!
//! Real geotechnical delivery is incremental: each AGS4 file carries only the
//! groups/rows captured that round. Merge folds them, **in caller-given argument
//! order**, into one file. The three load-bearing decisions:
//!
//! - **Union, never intersection.** A row (or group) absent from a later file is
//!   silence, not a deletion — a producer simply expressed no opinion on it this
//!   round. So merge only ever *adds* to the accumulated state; nothing a later
//!   file omits is dropped. (There is deliberately no delete/supersede primitive;
//!   a KEY-value correction therefore reads as a new row, not a revision — an
//!   inherent limit of KEY-based identity, documented, not silently handled.)
//! - **Argument order is authority; TRAN_DATE only cross-checks.** When two files
//!   carry the same KEY with different content, the later-argument file wins. If
//!   its file-level `TRAN_DATE` predates an earlier file's, that contradiction is
//!   *warned* (advisory), never blocking — because `TRAN_ISNO`/`TRAN_STAT` are
//!   free text (`X`) and only `TRAN_DATE` (`DT`) is machine-orderable, and even
//!   that is file-level, blind to a per-row regression inside an overall-newer
//!   file.
//! - **Type disagreement widens to `X`.** `X` is the top of the AGS type lattice
//!   (raw text holds any value faithfully), so a heading two files typed
//!   differently resolves to `X` under [`TypeMismatchMode::Lenient`] (lossless at
//!   the byte level) or errors under [`TypeMismatchMode::Strict`] (the default).
//!   Widening is emission-only: it changes the merged TYPE row, never how rows
//!   were matched (per-file `parse_value`) nor any content-addressed key.
//!
//! Row identity comes from the ONE shared definition
//! ([`laterite_ags4_reference::keychain::key_heading_names`]) that
//! `laterite-ags4-diff` also consumes — merge never re-derives "what identifies a
//! row". The merged bytes are written through [`laterite_ags4_emit`], the same
//! byte-faithful writer every other surface uses.

use std::collections::{BTreeSet, HashMap};

use laterite_ags4_emit::{EmitMode, EmitOpts, GroupInput, emit_ags4};
use laterite_ags4_parse::{ParsedFile, ParsedGroup};
use laterite_ags4_reference::dict::DictVersion;
use laterite_ags4_reference::keychain::key_heading_names;
use laterite_ags4_reference::union::registry;
use laterite_types::parse_value;
use serde_json::Value;

/// What to do when two files declare a different AGS TYPE for the same heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypeMismatchMode {
    /// Error on any type disagreement. Reconciling two independent producers'
    /// declared types is high-stakes and less reversible than a single-file
    /// fixup, so a lossy widen must be opted into — hence this is the default.
    #[default]
    Strict,
    /// Widen the merged column's TYPE to `X` (keeping every value's raw bytes)
    /// and carry on. Typed-vs-`X` widens silently (`X` trivially absorbs a typed
    /// value); two *different* non-`X` types warn (a genuine disagreement).
    Lenient,
}

/// Caller-supplied metadata for the merged file's own TRAN row. The merged file
/// genuinely *is* a new transmission, so it gets a fresh TRAN describing that
/// transmission (not a copy of any input's) — see the module note on TRAN.
#[derive(Debug, Clone, Default)]
pub struct TranStamp {
    pub isno: String,
    pub date: String,
    pub prod: String,
    pub recv: String,
    pub stat: String,
    pub ags: String,
}

/// Merge options.
#[derive(Debug, Clone)]
pub struct MergeOpts {
    pub type_mismatch: TypeMismatchMode,
    pub edition: DictVersion,
    pub emit_mode: EmitMode,
    /// When `Some`, the merged output carries one synthesised TRAN row from this
    /// stamp (with input ISNOs/dates recorded in `TRAN_REM` for provenance).
    /// When `None`, TRAN is reconciled like any other group (newest wins) and a
    /// warning notes no merge-transmission stamp was supplied.
    pub tran: Option<TranStamp>,
}

impl Default for MergeOpts {
    fn default() -> Self {
        MergeOpts {
            type_mismatch: TypeMismatchMode::Strict,
            edition: DictVersion::V4_1_1,
            emit_mode: EmitMode::AutoFix,
            tran: None,
        }
    }
}

/// A non-fatal note about something merge resolved (recency contradiction, a
/// non-`X` type widen, a missing merge-TRAN stamp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeWarning {
    pub kind: &'static str,
    pub group: Option<String>,
    pub heading: Option<String>,
    pub message: String,
}

/// A fatal merge failure.
#[derive(Debug)]
pub enum MergeError {
    /// Strict mode hit a heading two files typed differently.
    TypeConflict {
        group: String,
        heading: String,
        types: Vec<String>,
    },
    /// The byte-emission stage failed.
    Emit(String),
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeError::TypeConflict {
                group,
                heading,
                types,
            } => write!(
                f,
                "type conflict in {group}.{heading}: files declared {types:?} (STRICT mode)"
            ),
            MergeError::Emit(e) => write!(f, "emit failed: {e}"),
        }
    }
}

impl std::error::Error for MergeError {}

/// One row whose content a later file revised. Records what changed so a merge
/// is auditable — the closest we can get to per-row staleness detection, since
/// AGS4 carries no per-row timestamp (a stale row inside an overall-newer file
/// can't be auto-detected, only surfaced here for a human to review).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionNote {
    pub group: String,
    /// The KEY tuple identifying the revised row.
    pub key: Vec<String>,
    /// Heading names whose value the winning file changed (typed comparison, so a
    /// formatting-only difference is not reported).
    pub changed: Vec<String>,
    /// Argument index of the file that supplied the winning content.
    pub winner_file: usize,
}

/// The merged file plus any advisory warnings and the per-row revision report.
#[derive(Debug)]
pub struct MergeResult {
    pub bytes: Vec<u8>,
    pub warnings: Vec<MergeWarning>,
    pub revisions: Vec<RevisionNote>,
}

/// Merge `files` (in argument order — later wins a KEY conflict) into one AGS4 file.
pub fn merge_parsed(files: &[ParsedFile], opts: &MergeOpts) -> Result<MergeResult, MergeError> {
    let mut warnings = Vec::new();
    let mut revisions = Vec::new();

    // --- recency cross-check: warn if argument order contradicts TRAN_DATE ----
    recency_warnings(files, &mut warnings);

    // --- group processing order: PROJ, then TRAN, then the rest sorted --------
    let mut codes: BTreeSet<&str> = BTreeSet::new();
    for f in files {
        for c in f.groups.keys() {
            codes.insert(c.as_str());
        }
    }
    let mut ordered: Vec<&str> = Vec::new();
    for lead in ["PROJ", "TRAN"] {
        if codes.remove(lead) {
            ordered.push(lead);
        }
    }
    // A supplied merge-TRAN stamp always yields a TRAN group, even when no input
    // carried one — otherwise the stamp is silently dropped and emit injects its
    // own generic placeholder TRAN instead. Slot it right after PROJ (AGS order).
    if opts.tran.is_some() && !ordered.contains(&"TRAN") {
        let pos = usize::from(ordered.first() == Some(&"PROJ"));
        ordered.insert(pos, "TRAN");
    }
    ordered.extend(codes.iter().copied());

    // --- reconcile each group into a GroupInput --------------------------------
    let mut inputs: Vec<GroupInput> = Vec::with_capacity(ordered.len());
    for code in ordered {
        // TRAN is special: the merged file is a NEW transmission, so synthesise
        // its TRAN rather than reconcile the inputs' (see module note). With no
        // stamp, fall through to ordinary reconciliation (newest wins) + a warning.
        if code == "TRAN" {
            match &opts.tran {
                Some(stamp) => {
                    inputs.push(synthesise_tran(files, stamp));
                    continue;
                }
                None => warnings.push(MergeWarning {
                    kind: "tran_not_stamped",
                    group: Some("TRAN".into()),
                    heading: None,
                    message: "no merge TRAN stamp supplied; the merged file kept the \
                              newest input's TRAN as a fallback"
                        .into(),
                }),
            }
        }
        inputs.push(reconcile_group(
            code,
            files,
            opts,
            &mut warnings,
            &mut revisions,
        )?);
    }

    // --- coarse parent-child consistency flag ---------------------------------
    // Merge reconciles each group independently, so a revised parent (e.g. a LOCA
    // whose GL changed) can leave its children (SAMP, …) referencing the old
    // assumption if they weren't re-supplied. AGS4 has no cross-group consistency
    // check and we can't invent one, but we CAN flag it: one advisory per revised
    // parent that has child groups in this merge, so a human verifies them.
    let present_codes: BTreeSet<&str> = files
        .iter()
        .flat_map(|f| f.groups.keys().map(String::as_str))
        .collect();
    let revised: BTreeSet<&str> = revisions.iter().map(|r| r.group.as_str()).collect();
    for parent in &revised {
        let children: Vec<String> = registry()
            .iter()
            .filter(|g| {
                g.parent.as_deref() == Some(parent) && present_codes.contains(g.code.as_str())
            })
            .map(|g| g.code.clone())
            .collect();
        if !children.is_empty() {
            warnings.push(MergeWarning {
                kind: "parent_revised_check_children",
                group: Some(parent.to_string()),
                heading: None,
                message: format!(
                    "{parent} was revised; verify child group(s) {children:?} are still \
                     consistent (merge performs no cross-group consistency check)"
                ),
            });
        }
    }

    // --- emit through the byte-faithful writer --------------------------------
    let emit_opts = EmitOpts {
        mode: opts.emit_mode,
        edition: opts.edition,
    };
    let out = emit_ags4(&inputs, &emit_opts).map_err(|e| MergeError::Emit(e.to_string()))?;
    Ok(MergeResult {
        bytes: out.bytes,
        warnings,
        revisions,
    })
}

/// The `(heading -> column index)` map for a parsed group, for cell lookup.
fn heading_index(headings: &[String]) -> HashMap<&str, usize> {
    headings
        .iter()
        .enumerate()
        .map(|(i, h)| (h.as_str(), i))
        .collect()
}

/// A cell of a parsed group's row by heading name; `None` if the group doesn't
/// carry that heading, `Some("")` for a short/ragged row (carried-but-empty).
fn cell<'a>(idx: &HashMap<&str, usize>, row: &'a [String], h: &str) -> Option<&'a str> {
    idx.get(h)
        .map(|&i| row.get(i).map(String::as_str).unwrap_or(""))
}

/// Warn when a file later in argument order carries an earlier `TRAN_DATE` than
/// one before it — argument order still wins, this is advisory only.
fn recency_warnings(files: &[ParsedFile], warnings: &mut Vec<MergeWarning>) {
    let mut running_max: Option<(usize, String)> = None; // (file index, normalised date)
    for (i, f) in files.iter().enumerate() {
        let Some(date) = tran_field(f, "TRAN_DATE") else {
            continue;
        };
        // parse_value normalises DT to a zero-padded "%Y-%m-%d %H:%M:%S", so
        // lexical order == chronological order. TRAN_DATE's declared unit is
        // yyyy-mm-dd, so compare the DATE portion only — a same-day AM/PM re-issue
        // (a producer who informally wrote a full timestamp) must not false-warn.
        let day = match parse_value(Some(&date), "DT") {
            Value::String(s) => s.get(..10).map(str::to_string).unwrap_or(s),
            _ => continue, // unparseable date → can't cross-check this file
        };
        match &running_max {
            Some((prev_i, prev)) if day < *prev => {
                warnings.push(MergeWarning {
                    kind: "recency_contradiction",
                    group: Some("TRAN".into()),
                    heading: Some("TRAN_DATE".into()),
                    message: format!(
                        "file[{i}] is later in argument order but its TRAN_DATE ({date}) \
                         predates file[{prev_i}]'s ({prev}); argument order still wins"
                    ),
                });
            }
            _ => {}
        }
        if running_max.as_ref().map(|(_, d)| day > *d).unwrap_or(true) {
            running_max = Some((i, day));
        }
    }
}

/// Read a single-value field from a file's (single-row) TRAN group.
fn tran_field(f: &ParsedFile, heading: &str) -> Option<String> {
    let g = f.groups.get("TRAN")?;
    let idx = heading_index(&g.headings);
    let row = g.rows.first()?;
    cell(&idx, &row.values, heading).map(str::to_string)
}

/// Resolve the merged TYPE for one heading across the files that declare it.
/// Returns the merged type string and pushes a warning for a non-`X`-vs-non-`X`
/// widen; errors in Strict mode.
fn merged_type(
    code: &str,
    heading: &str,
    declared: &[String],
    opts: &MergeOpts,
    warnings: &mut Vec<MergeWarning>,
) -> Result<String, MergeError> {
    let distinct: BTreeSet<&str> = declared
        .iter()
        .map(String::as_str)
        .filter(|t| !t.is_empty())
        .collect();
    match distinct.len() {
        0 => Ok(String::new()), // no declared type anywhere → let emit fill from dict
        1 => Ok(distinct.iter().next().unwrap().to_string()),
        _ => {
            // A genuine disagreement.
            if opts.type_mismatch == TypeMismatchMode::Strict {
                return Err(MergeError::TypeConflict {
                    group: code.to_string(),
                    heading: heading.to_string(),
                    types: distinct.iter().map(|s| s.to_string()).collect(),
                });
            }
            // Lenient: widen to X. Silent when the only disagreement is typed-vs-X
            // (X absorbs anything); warn when two *different* non-X types clash.
            let non_x = distinct.iter().filter(|t| **t != "X").count();
            if non_x >= 2 {
                warnings.push(MergeWarning {
                    kind: "type_widened",
                    group: Some(code.to_string()),
                    heading: Some(heading.to_string()),
                    message: format!(
                        "files disagree on TYPE {distinct:?}; widened to X (values kept as raw text)"
                    ),
                });
            }
            Ok("X".to_string())
        }
    }
}

/// Reconcile one group across all files into a single emit-ready `GroupInput`,
/// appending any per-row revisions to `revisions`.
fn reconcile_group(
    code: &str,
    files: &[ParsedFile],
    opts: &MergeOpts,
    warnings: &mut Vec<MergeWarning>,
    revisions: &mut Vec<RevisionNote>,
) -> Result<GroupInput, MergeError> {
    // The files that carry this group, paired with their argument index.
    let present: Vec<(usize, &ParsedGroup)> = files
        .iter()
        .enumerate()
        .filter_map(|(i, f)| f.groups.get(code).map(|g| (i, g)))
        .collect();

    // Union headings: first file's order, then any new heading appended in
    // first-seen order (a later file only ever widens the schema).
    let mut union_h: Vec<String> = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (_, g) in &present {
        for h in &g.headings {
            if seen.insert(h.as_str()) {
                union_h.push(h.clone());
            }
        }
    }
    let union_idx: HashMap<&str, usize> = union_h
        .iter()
        .enumerate()
        .map(|(i, h)| (h.as_str(), i))
        .collect();

    // Merged UNIT (first non-empty per heading) and TYPE (agree, else widen/err).
    let mut units: Vec<String> = vec![String::new(); union_h.len()];
    let mut types: Vec<String> = vec![String::new(); union_h.len()];
    for (ui, h) in union_h.iter().enumerate() {
        let mut declared_types: Vec<String> = Vec::new();
        for (_, g) in &present {
            let hidx = heading_index(&g.headings);
            if let Some(&ci) = hidx.get(h.as_str()) {
                if units[ui].is_empty() {
                    if let Some(u) = g.units.get(ci) {
                        if !u.is_empty() {
                            units[ui] = u.clone();
                        }
                    }
                }
                if let Some(t) = g.types.get(ci) {
                    declared_types.push(t.clone());
                }
            }
        }
        types[ui] = merged_type(code, h, &declared_types, opts, warnings)?;
    }

    // Row identity: the ONE shared definition (`key_heading_names`), filtered to
    // the KEY headings present in this group's union schema (a child that doesn't
    // carry an ancestor KEY — e.g. LOCA without PROJ_ID — keys on what it has).
    // This is the same source `laterite-ags4-diff` consumes; merge never
    // re-derives it. A registered group with no KEY present, or a custom /
    // passthrough group, has no spec identity → it dedups only rows identical
    // across the WHOLE union schema (safe: never merges distinct rows, never loses
    // data; exact cross-delivery re-sends collapse, everything else is kept).
    let (id_headings, keyed): (Vec<&str>, bool) = match registry().get(code) {
        Some(g) => {
            let ks: Vec<&str> = key_heading_names(g)
                .into_iter()
                .filter(|k| union_idx.contains_key(k))
                .collect();
            if ks.is_empty() {
                (union_h.iter().map(String::as_str).collect(), false)
            } else {
                (ks, true)
            }
        }
        None => (union_h.iter().map(String::as_str).collect(), false),
    };

    let rows = reconcile_rows(
        code,
        &present,
        &union_h,
        &union_idx,
        &id_headings,
        keyed,
        warnings,
        revisions,
    );

    Ok(GroupInput {
        code: code.to_string(),
        headings: union_h,
        units: Some(units),
        types: Some(types),
        rows,
    })
}

/// A resolved cell: its raw value plus the AGS type the winning file declared for
/// it, kept so a later overwrite is compared TYPED (via `parse_value`), not by raw
/// bytes — so `"1.0"` → `"1.00"` is a byte-level win but not a reported revision.
#[derive(Clone)]
struct MCell {
    value: String,
    ty: String,
}

/// Fold rows by their identity tuple, later file wins a conflicting cell,
/// union-not-intersection (an id-tuple only an earlier file carried survives).
/// For keyed groups, records TYPED revisions and warns on within-file duplicate
/// KEYs; for unkeyed groups (`keyed == false`), `id_headings` is the whole union,
/// so only exact-identical rows collapse and no revision/dup semantics apply.
#[allow(clippy::too_many_arguments)]
fn reconcile_rows(
    code: &str,
    present: &[(usize, &ParsedGroup)],
    union_h: &[String],
    union_idx: &HashMap<&str, usize>,
    id_headings: &[&str],
    keyed: bool,
    warnings: &mut Vec<MergeWarning>,
    revisions: &mut Vec<RevisionNote>,
) -> Vec<Vec<Value>> {
    let mut order: Vec<Vec<Option<MCell>>> = Vec::new();
    let mut created_by: Vec<usize> = Vec::new();
    let mut pos: HashMap<Vec<String>, usize> = HashMap::new();
    let no_identity = id_headings.is_empty();
    let mut dup_warned = false;

    for (fi, g) in present {
        let idx = heading_index(&g.headings);
        let type_of = |h: &str| -> String {
            idx.get(h)
                .and_then(|&ci| g.types.get(ci))
                .cloned()
                .unwrap_or_default()
        };
        let mut seen_this_file: std::collections::HashSet<Vec<String>> =
            std::collections::HashSet::new();

        for row in &g.rows {
            let row_key = || -> Vec<String> {
                id_headings
                    .iter()
                    .map(|k| cell(&idx, &row.values, k).unwrap_or("").to_string())
                    .collect()
            };

            // Locate (or create) this row's slot.
            let (slot, from_earlier_file) = if no_identity {
                order.push(vec![None; union_h.len()]);
                created_by.push(*fi);
                (order.len() - 1, false)
            } else {
                let key = row_key();
                if keyed && !seen_this_file.insert(key.clone()) && !dup_warned {
                    warnings.push(MergeWarning {
                        kind: "duplicate_key_in_file",
                        group: Some(code.to_string()),
                        heading: None,
                        message: format!(
                            "file[{fi}] has more than one {code} row with the same KEY \
                             {key:?} (a data-quality error); later wins, but review the source"
                        ),
                    });
                    dup_warned = true;
                }
                match pos.get(&key) {
                    Some(&p) => (p, created_by[p] != *fi),
                    None => {
                        order.push(vec![None; union_h.len()]);
                        created_by.push(*fi);
                        let p = order.len() - 1;
                        pos.insert(key, p);
                        (p, false)
                    }
                }
            };

            // Overwrite every cell THIS file carries (later wins); leave the rest.
            let mut changed: Vec<String> = Vec::new();
            for h in &g.headings {
                let (Some(&ui), Some(v)) = (union_idx.get(h.as_str()), cell(&idx, &row.values, h))
                else {
                    continue;
                };
                let nt = type_of(h);
                // A revision = a cell an EARLIER file set whose RAW value changed
                // AND the change isn't formatting-only. Identical raw bytes are
                // never a revision, even when the column's TYPE widened around them
                // (e.g. 2DP→X) — the type change would otherwise make equal bytes
                // compare unequal across the type boundary.
                if keyed && from_earlier_file {
                    if let Some(old) = &order[slot][ui] {
                        if old.value != v
                            && parse_value(Some(&old.value), &old.ty) != parse_value(Some(v), &nt)
                        {
                            changed.push(h.clone());
                        }
                    }
                }
                order[slot][ui] = Some(MCell {
                    value: v.to_string(),
                    ty: nt,
                });
            }
            if from_earlier_file && !changed.is_empty() {
                revisions.push(RevisionNote {
                    group: code.to_string(),
                    key: row_key(),
                    changed,
                    winner_file: *fi,
                });
            }
        }
    }

    // `Option<MCell>` cells → emit `Value`s: a carried cell is raw text (byte
    // fidelity — merge never reformats), a not-carried cell is Null (→ blank).
    order
        .into_iter()
        .map(|cells| {
            cells
                .into_iter()
                .map(|c| c.map(|m| Value::String(m.value)).unwrap_or(Value::Null))
                .collect()
        })
        .collect()
}

/// Build the merged file's own single-row TRAN from the caller's stamp, recording
/// the input files' ISNO/DATE in TRAN_REM so the merge is self-documenting.
fn synthesise_tran(files: &[ParsedFile], stamp: &TranStamp) -> GroupInput {
    let provenance = {
        let parts: Vec<String> = files
            .iter()
            .filter_map(|f| {
                let isno = tran_field(f, "TRAN_ISNO").unwrap_or_default();
                let date = tran_field(f, "TRAN_DATE").unwrap_or_default();
                if isno.is_empty() && date.is_empty() {
                    None
                } else {
                    Some(format!("ISNO={isno} ({date})"))
                }
            })
            .collect();
        format!(
            "Merged from {} deliveries: {}",
            parts.len(),
            parts.join("; ")
        )
    };

    let headings = vec![
        "TRAN_ISNO".to_string(),
        "TRAN_DATE".to_string(),
        "TRAN_PROD".to_string(),
        "TRAN_STAT".to_string(),
        "TRAN_AGS".to_string(),
        "TRAN_RECV".to_string(),
        "TRAN_REM".to_string(),
    ];
    let row = vec![
        Value::String(stamp.isno.clone()),
        Value::String(stamp.date.clone()),
        Value::String(stamp.prod.clone()),
        Value::String(stamp.stat.clone()),
        Value::String(stamp.ags.clone()),
        Value::String(stamp.recv.clone()),
        Value::String(provenance),
    ];
    GroupInput {
        code: "TRAN".to_string(),
        headings,
        units: None, // let emit fill UNIT/TYPE from the dictionary
        types: None,
        rows: vec![row],
    }
}
