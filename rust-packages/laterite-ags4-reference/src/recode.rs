//! Rewriting ABBR codes on request — the one shared implementation that
//! `merge` (#1010) calls and `fix` (#1024) will call.
//!
//! Rule 16 looks codes up exactly, so `"Undisturbed"` and `"UNDISTURBED"`
//! under one heading are two codes to the validator, yet many importers key
//! the ABBR list ignoring case and reject the pair as a duplicate. A caller
//! who wants one spelling has two ways to ask for it, and only these two:
//!
//! - a [`Recode`] mapping, `{heading: {from_code: to_code}}`, which names the
//!   change outright; and
//! - [`CodeCaseMode::Standard`], which settles a set of case variants only
//!   when the edition's standard list holds exactly ONE code matching them
//!   ignoring case.
//!
//! Nothing here ever picks a spelling by input order: a set with no single
//! standard match, or with several (the standard carries case-only pairs of
//! its own), is left as written. The decision is the caller's, recorded in
//! the plan, never inferred.
//!
//! The work is split in two so each caller can apply its own policy between
//! the halves. [`resolve_mapping`] finds the case-variant sets and turns the
//! caller's request into an effective mapping; [`plan_rewrite`] turns a
//! mapping into cell edits and dropped ABBR rows, and reports any KEY
//! collision the rewrite would cause. Merge refuses a plan with a collision;
//! `fix` is meant to drop the implicated sets from the mapping and plan again.
//! Neither half mutates a [`ParsedFile`] — the parse is span-backed, so the
//! plan addresses cells by position and the caller applies it.

use std::collections::BTreeMap;

use laterite_ags4_parse::{ParsedFile, ParsedGroup};

use crate::dict::Dictionary;
use crate::keychain::key_heading_names;
use crate::union::registry;

/// Whether case variants of one ABBR code are left alone or settled on the
/// edition's standard spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeCaseMode {
    /// Leave every spelling as written. The default: a rewrite changes cell
    /// values, so it is something a caller asks for.
    #[default]
    Keep,
    /// Rewrite a set of case variants to the one standard code that matches
    /// them ignoring case. A set with no standard match, or with more than
    /// one, is left as written — choosing among them would be a guess.
    Standard,
}

impl CodeCaseMode {
    /// The wire name — the exact token every surface accepts and reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CodeCaseMode::Keep => "keep",
            CodeCaseMode::Standard => "standard",
        }
    }

    /// Every accepted token, default first. The single source for the CLI's
    /// value enum, the `.pyi` `Literal` and the TS union.
    pub const ALL: [CodeCaseMode; 2] = [CodeCaseMode::Keep, CodeCaseMode::Standard];
}

impl std::str::FromStr for CodeCaseMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CodeCaseMode::ALL
            .into_iter()
            .find(|m| m.as_str() == s.trim().to_ascii_lowercase())
            .ok_or_else(|| {
                let allowed: Vec<&str> = CodeCaseMode::ALL.iter().map(|m| m.as_str()).collect();
                format!(
                    "unknown on_code_case {s:?}; expected one of {}",
                    allowed.join(", ")
                )
            })
    }
}

/// A caller-named rewrite: `{heading: {from_code: to_code}}`. Ordered maps so
/// a plan, and any message built from it, is the same on every run.
pub type Recode = BTreeMap<String, BTreeMap<String, String>>;

/// One spelling of a case-variant set, with the inputs whose ABBR group
/// declares it (argument indices, ascending).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Spelling {
    pub code: String,
    pub inputs: Vec<usize>,
}

/// Two or more ABBR codes under one heading that are equal ignoring case but
/// differ as written, as the inputs declared them (before any rewrite).
///
/// The `Serialize` derive is the wire shape merge's `abbr_code_case` warning
/// carries, so a caller can build a [`Recode`] from it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CaseSet {
    pub heading: String,
    /// Every spelling, in first-seen order (argument order, then row order).
    pub spellings: Vec<Spelling>,
    /// The single standard code matching the set ignoring case, if there is
    /// exactly one. `None` when the standard has none, or several.
    pub suggested: Option<String>,
    /// The one spelling every member carries after the rewrite, when the
    /// rewrite settled the set; `None` when it was left (wholly or partly) as
    /// written.
    pub resolved: Option<String>,
}

/// The rewrite to apply: `{heading: {from_code: to_code}}`, every entry a
/// real change. Same shape as [`Recode`]; a separate name because it is the
/// *outcome* — the caller's recode plus whatever [`CodeCaseMode::Standard`]
/// added.
pub type Mapping = BTreeMap<String, BTreeMap<String, String>>;

/// What [`resolve_mapping`] found and decided.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolution {
    /// Every case-variant set in the inputs' ABBR groups, in first-seen
    /// order, whatever the mode — the warning is owed even when nothing is
    /// rewritten.
    pub case_sets: Vec<CaseSet>,
    /// The effective rewrite. Empty under [`CodeCaseMode::Keep`] with no
    /// recode.
    pub mapping: Mapping,
}

/// A recode the inputs cannot honour. Refused rather than ignored: a mapping
/// that silently did nothing would look exactly like one that worked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecodeError {
    /// The heading is not typed `PA` in any input group that carries it, so
    /// it has no codes to rewrite.
    NotPaHeading(String),
    /// `from` appears neither in the ABBR rows nor in any `PA` cell under the
    /// heading.
    AbsentCode { heading: String, code: String },
    /// A mapping to the empty string would blank the cells, not recode them.
    EmptyTarget { heading: String, code: String },
}

impl std::fmt::Display for RecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecodeError::NotPaHeading(h) => write!(
                f,
                "recode names {h:?}, which no input carries as a PA heading; only PA codes \
                 can be recoded"
            ),
            RecodeError::AbsentCode { heading, code } => write!(
                f,
                "recode maps {code:?} under {heading:?}, but no input uses that code there \
                 (codes are matched exactly, letter case included)"
            ),
            RecodeError::EmptyTarget { heading, code } => write!(
                f,
                "recode maps {code:?} under {heading:?} to an empty code; name the code to \
                 rewrite it to"
            ),
        }
    }
}

impl std::error::Error for RecodeError {}

/// One cell the rewrite changes: input `input`, group `group`, data row `row`
/// (index into `ParsedGroup::rows`), column `col` (index into its headings).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellEdit {
    pub input: usize,
    pub group: String,
    pub row: usize,
    pub col: usize,
    pub value: String,
}

/// One data row the rewrite removes — always an ABBR row whose code
/// collapsed into a row that already says the same thing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowRef {
    pub input: usize,
    pub group: String,
    pub row: usize,
}

/// Two or more rows the rewrite would give one identity that did not share
/// one before. A caller that reconciled them would lose a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCollision {
    pub group: String,
    /// The identity the rows would share after the rewrite.
    pub key: Vec<String>,
    /// Each colliding row as `(input, row, identity before the rewrite)`.
    pub rows: Vec<(usize, usize, Vec<String>)>,
    /// The mapping entries, `(heading, from_code)`, that rewrote an identity
    /// cell of those rows — what a caller drops to skip the sets involved.
    pub codes: Vec<(String, String)>,
}

/// A mapping turned into edits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rewrite {
    /// Sorted by group, then input, row and column, so a caller can find a
    /// group's share by binary search.
    pub edits: Vec<CellEdit>,
    /// Always ABBR rows, in input then row order.
    pub dropped: Vec<RowRef>,
    /// Narrowest identity first: a collision in a parent group (SAMP) repeats
    /// in every child that carries its KEY, and the parent is the one to name.
    pub collisions: Vec<KeyCollision>,
}

const ABBR: &str = "ABBR";

// The helpers below use plain vectors and linear or binary search rather than
// hash maps: the sets they hold are an ABBR group's worth of codes, and every
// distinct map type is code the browser engine downloads (the tier-1 wasm
// ceiling, `tools/release/check-wasm-tier1.mjs`, holds merge's share of it).

/// Lower-case fold matching [`Dictionary::abbr_codes_ignoring_case`], so a
/// set this module finds is a set that lookup would group.
fn fold(code: &str) -> String {
    code.to_lowercase()
}

#[inline(never)]
fn col(g: &ParsedGroup, heading: &str) -> Option<usize> {
    g.headings.iter().position(|h| h == heading)
}

#[inline(never)]
fn value(g: &ParsedGroup, row: usize, c: usize) -> &str {
    g.rows.get(row).and_then(|r| g.value_at(r, c)).unwrap_or("")
}

/// The concatenation character each input declares in `TRAN_RCON`. Read per
/// input because the inputs are separate transmissions; an input without one
/// has values that cannot be split, so they are matched whole.
#[inline(never)]
fn rcon(f: &ParsedFile) -> Option<&str> {
    let g = f.groups.get("TRAN")?;
    let c = col(g, "TRAN_RCON")?;
    Some(value(g, 0, c)).filter(|s| !s.is_empty())
}

/// Whether column `c` of `g` is declared `PA`.
fn is_pa(g: &ParsedGroup, c: usize) -> bool {
    g.types.get(c).is_some_and(|t| t.trim() == "PA")
}

/// The ABBR rows of one input, as `(row, heading, code)`.
#[inline(never)]
fn abbr_rows(f: &ParsedFile) -> Vec<(usize, &str, &str)> {
    let Some(g) = f.groups.get(ABBR) else {
        return Vec::new();
    };
    let (Some(hc), Some(cc)) = (col(g, "ABBR_HDNG"), col(g, "ABBR_CODE")) else {
        return Vec::new();
    };
    (0..g.rows.len())
        .map(|r| (r, value(g, r, hc), value(g, r, cc)))
        .filter(|(_, h, c)| !h.is_empty() && !c.is_empty())
        .collect()
}

/// Split a PA cell into its codes on the input's concatenation character.
#[inline(never)]
fn parts<'a>(v: &'a str, rcon: Option<&str>) -> Vec<&'a str> {
    match rcon {
        Some(sep) if v.contains(sep) => v.split(sep).collect(),
        _ => vec![v],
    }
}

/// `to` in `map`, or `from` itself when it is not mapped.
fn image<'a>(map: Option<&'a BTreeMap<String, String>>, from: &'a str) -> &'a str {
    map.and_then(|m| m.get(from)).map_or(from, String::as_str)
}

/// One case set as walked: `(heading, fold, [(source spelling, spelling)])`.
type Grouped = (String, String, Vec<(String, Spelling)>);

/// Every ABBR spelling in the inputs as `(heading, fold, [(spelling, inputs)])`
/// in first-seen order (argument order, then row order), each spelling's
/// first appearance kept. `recode` re-spells each code before it is folded and
/// grouped, so the same walk serves the sets as declared and as they stand
/// once a recode is applied.
#[inline(never)]
fn group_spellings(files: &[ParsedFile], recode: Option<&Recode>) -> Vec<Grouped> {
    let mut sets: Vec<Grouped> = Vec::new();
    for (i, f) in files.iter().enumerate() {
        for (_, h, c) in abbr_rows(f) {
            let img = image(recode.and_then(|r| r.get(h)), c);
            let k = fold(img);
            let at = if let Some(p) = sets.iter().position(|s| s.0 == h && s.1 == k) {
                p
            } else {
                sets.push((h.to_string(), k, Vec::new()));
                sets.len() - 1
            };
            let members = &mut sets[at].2;
            match members.iter_mut().find(|(o, _)| o == c) {
                Some((_, s)) if s.inputs.last() != Some(&i) => s.inputs.push(i),
                Some(_) => {}
                None => members.push((
                    c.to_string(),
                    Spelling {
                        code: img.to_string(),
                        inputs: vec![i],
                    },
                )),
            }
        }
    }
    sets
}

/// The one standard code matching `code` ignoring case, if exactly one does.
#[inline(never)]
fn single_standard(dict: &Dictionary<'_>, heading: &str, code: &str) -> Option<String> {
    match dict.abbr_codes_ignoring_case(heading, code).as_slice() {
        [one] => Some((*one).to_string()),
        _ => None,
    }
}

/// Record `c` as used when the recode names it — only those are ever asked
/// about, so the list stays the size of the recode, not of the data.
fn note<'a>(codes: &BTreeMap<String, String>, c: &'a str, used: &mut Vec<&'a str>) {
    if codes.contains_key(c) && !used.contains(&c) {
        used.push(c);
    }
}

/// Check a caller's recode against the inputs, refusing what cannot apply.
#[inline(never)]
fn validate_recode(files: &[ParsedFile], recode: &Recode) -> Result<(), RecodeError> {
    for (heading, codes) in recode {
        let mut pa = false;
        let mut used: Vec<&str> = Vec::new();
        for f in files {
            let rc = rcon(f);
            for (code, g) in &f.groups {
                if code == ABBR {
                    continue;
                }
                let Some(c) = col(g, heading).filter(|&c| is_pa(g, c)) else {
                    continue;
                };
                pa = true;
                for r in 0..g.rows.len() {
                    for p in parts(value(g, r, c), rc) {
                        note(codes, p, &mut used);
                    }
                }
            }
            for (_, h, c) in abbr_rows(f) {
                if h == heading {
                    note(codes, c, &mut used);
                }
            }
        }
        if !pa {
            return Err(RecodeError::NotPaHeading(heading.clone()));
        }
        for (from, to) in codes {
            if !used.contains(&from.as_str()) {
                return Err(RecodeError::AbsentCode {
                    heading: heading.clone(),
                    code: from.clone(),
                });
            }
            if to.is_empty() {
                return Err(RecodeError::EmptyTarget {
                    heading: heading.clone(),
                    code: from.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Find the inputs' case-variant sets and turn the caller's request into the
/// mapping to apply.
///
/// `recode` is applied first. [`CodeCaseMode::Standard`] then looks only at
/// the sets still holding two or more spellings once the recode is applied,
/// and skips any set a recode target falls in, so a code the caller named is
/// never re-spelled behind their back. It rewrites a set only when exactly one
/// standard code matches it — even a spelling no input used.
///
/// # Errors
///
/// A [`RecodeError`] naming the first recode entry the inputs cannot honour:
/// a heading no input types `PA`, a code no input uses under it, or an empty
/// target.
pub fn resolve_mapping(
    files: &[ParsedFile],
    dict: &Dictionary<'_>,
    recode: &Recode,
    mode: CodeCaseMode,
) -> Result<Resolution, RecodeError> {
    validate_recode(files, recode)?;

    let mut mapping: Mapping = BTreeMap::new();
    let mut map = |heading: &str, from: &str, to: &str| {
        if from != to {
            mapping
                .entry(heading.to_string())
                .or_default()
                .insert(from.to_string(), to.to_string());
        }
    };
    for (heading, codes) in recode {
        for (from, to) in codes {
            map(heading, from, to);
        }
    }

    if mode == CodeCaseMode::Standard {
        // The sets as they stand once the recode is applied. One holding a
        // recode target is the caller's, so standard leaves it alone.
        for (heading, _, members) in group_spellings(files, Some(recode)) {
            let first = &members[0].1.code;
            if members.iter().all(|(_, s)| s.code == *first) {
                continue;
            }
            let targets = recode.get(&heading);
            if members.iter().any(|(o, s)| {
                targets.is_some_and(|m| m.contains_key(o) || m.values().any(|t| *t == s.code))
            }) {
                continue;
            }
            let Some(standard) = single_standard(dict, &heading, first) else {
                continue;
            };
            for (orig, _) in &members {
                map(&heading, orig, &standard);
            }
        }
    }

    let case_sets = group_spellings(files, None)
        .into_iter()
        .filter(|(_, _, m)| m.len() >= 2)
        .map(|(heading, _, members)| {
            let map = mapping.get(&heading);
            let spellings: Vec<Spelling> = members.into_iter().map(|(_, s)| s).collect();
            let end = image(map, &spellings[0].code).to_string();
            let resolved = spellings
                .iter()
                .all(|s| image(map, &s.code) == end)
                .then_some(end);
            CaseSet {
                suggested: single_standard(dict, &heading, &spellings[0].code),
                heading,
                spellings,
                resolved,
            }
        })
        .collect();

    Ok(Resolution { case_sets, mapping })
}

/// Rewrite one PA cell part by part; `None` when nothing in it is mapped.
#[inline(never)]
fn rewrite_cell(v: &str, rcon: Option<&str>, map: &BTreeMap<String, String>) -> Option<String> {
    let ps = parts(v, rcon);
    if !ps.iter().any(|p| map.contains_key(*p)) {
        return None;
    }
    let out: Vec<&str> = ps.iter().map(|p| image(Some(map), p)).collect();
    Some(out.join(rcon.unwrap_or("")))
}

/// Turn `mapping` into cell edits and dropped ABBR rows over `files`, and
/// report every KEY collision the edits would cause.
///
/// Every `PA` cell under a mapped heading, in every group, is rewritten part
/// by part on its input's `TRAN_RCON`. The ABBR rows collapse to one per
/// target code: when an input already declares the target, its row stands
/// and the mapped rows are dropped; otherwise the first mapped row (argument
/// order, then row order) becomes the target's row, taking the standard
/// description when the edition has one and keeping its own otherwise, and
/// the rest are dropped.
///
/// A collision is two rows of one group whose identity — the group's KEY
/// headings, or every heading for a group without KEYs, over the headings
/// the inputs carry — differed before the edits and is equal after. Rows that
/// already shared an identity are not a collision. The edits are returned
/// whole either way; what to do about a collision is the caller's policy.
#[must_use]
pub fn plan_rewrite(files: &[ParsedFile], dict: &Dictionary<'_>, mapping: &Mapping) -> Rewrite {
    let mut out = Rewrite::default();
    if mapping.is_empty() {
        return out;
    }
    let edit = |input: usize, group: &str, row: usize, col: usize, value: String| CellEdit {
        input,
        group: group.to_string(),
        row,
        col,
        value,
    };

    // --- PA cells, every group but ABBR --------------------------------------
    for (i, f) in files.iter().enumerate() {
        let rc = rcon(f);
        for (code, g) in &f.groups {
            if code == ABBR {
                continue;
            }
            for (c, h) in g.headings.iter().enumerate() {
                let Some(map) = mapping.get(h).filter(|_| is_pa(g, c)) else {
                    continue;
                };
                for r in 0..g.rows.len() {
                    if let Some(v) = rewrite_cell(value(g, r, c), rc, map) {
                        out.edits.push(edit(i, code, r, c, v));
                    }
                }
            }
        }
    }

    // --- ABBR rows ------------------------------------------------------------
    // Which targets an input declares in its own right — a row whose code is
    // the target and is not itself being rewritten away.
    let mut declared: Vec<(&str, &str)> = Vec::new();
    for f in files {
        for (_, h, c) in abbr_rows(f) {
            if mapping.get(h).is_none_or(|m| !m.contains_key(c)) {
                declared.push((h, c));
            }
        }
    }
    let mut kept: Vec<(&str, &str)> = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let Some(g) = f.groups.get(ABBR) else {
            continue;
        };
        let cc = col(g, "ABBR_CODE");
        let dc = col(g, "ABBR_DESC");
        for (r, h, c) in abbr_rows(f) {
            let Some(to) = mapping.get(h).and_then(|m| m.get(c)) else {
                continue;
            };
            let target = (h, to.as_str());
            if declared.contains(&target) || kept.contains(&target) {
                out.dropped.push(RowRef {
                    input: i,
                    group: ABBR.to_string(),
                    row: r,
                });
                continue;
            }
            kept.push(target);
            if let Some(cc) = cc {
                out.edits.push(edit(i, ABBR, r, cc, to.clone()));
            }
            if let (Some(dc), Some(desc)) = (dc, dict.abbr_desc(h, to)) {
                if value(g, r, dc) != desc {
                    out.edits.push(edit(i, ABBR, r, dc, desc.to_string()));
                }
            }
        }
    }

    // One order for every caller to search: by group, then position.
    out.edits.sort_unstable_by(|a, b| {
        (&a.group, a.input, a.row, a.col).cmp(&(&b.group, b.input, b.row, b.col))
    });
    out.collisions = collisions(files, mapping, &out.edits);
    out
}

/// The identity a row lands on after the rewrite, with its identity before
/// it, where it came from, and the mapping entries that rewrote it.
struct Landing {
    after: Vec<String>,
    input: usize,
    row: usize,
    before: Vec<String>,
    codes: Vec<(String, String)>,
}

/// The KEY collisions `edits` cause. ABBR is exempt: its rows collapse by
/// design, through `dropped`, never through a shared identity.
#[inline(never)]
fn collisions(files: &[ParsedFile], mapping: &Mapping, edits: &[CellEdit]) -> Vec<KeyCollision> {
    // `edits` arrive sorted by group, then position (see `plan_rewrite`).
    let mut touched: Vec<&str> = edits
        .iter()
        .map(|e| e.group.as_str())
        .filter(|g| *g != ABBR)
        .collect();
    touched.dedup();

    let mut found: Vec<(usize, KeyCollision)> = Vec::new();
    for code in touched {
        // The identity merge keys on: the KEY headings present in the union of
        // the inputs' headings, else every heading of that union.
        let mut union: Vec<&str> = Vec::new();
        for g in files.iter().filter_map(|f| f.groups.get(code)) {
            for h in &g.headings {
                if !union.contains(&h.as_str()) {
                    union.push(h);
                }
            }
        }
        let keys: Vec<&str> = registry()
            .get(code)
            .map(|g| {
                key_heading_names(g)
                    .into_iter()
                    .filter(|k| union.contains(k))
                    .collect()
            })
            .unwrap_or_default();
        let ids: Vec<&str> = if keys.is_empty() { union } else { keys };

        let mut landed: Vec<Landing> = Vec::new();
        for (input, f) in files.iter().enumerate() {
            let Some(g) = f.groups.get(code) else {
                continue;
            };
            let rc = rcon(f);
            let cols: Vec<Option<usize>> = ids.iter().map(|h| col(g, h)).collect();
            for row in 0..g.rows.len() {
                let mut l = Landing {
                    after: Vec::with_capacity(ids.len()),
                    input,
                    row,
                    before: Vec::with_capacity(ids.len()),
                    codes: Vec::new(),
                };
                for (h, c) in ids.iter().zip(&cols) {
                    let b = c.map_or("", |c| value(g, row, c));
                    let hit = c.and_then(|c| {
                        edits
                            .binary_search_by(|e| {
                                (e.group.as_str(), e.input, e.row, e.col)
                                    .cmp(&(code, input, row, c))
                            })
                            .ok()
                    });
                    l.before.push(b.to_string());
                    let Some(at) = hit else {
                        l.after.push(b.to_string());
                        continue;
                    };
                    l.after.push(edits[at].value.clone());
                    let map = mapping.get(*h);
                    for p in parts(b, rc) {
                        if map.is_some_and(|m| m.contains_key(p)) {
                            l.codes.push(((*h).to_string(), p.to_string()));
                        }
                    }
                }
                landed.push(l);
            }
        }

        // Rows landing on one identity from two different ones collide.
        landed.sort_unstable_by(|a, b| (&a.after, a.input, a.row).cmp(&(&b.after, b.input, b.row)));
        for run in landed.chunk_by(|a, b| a.after == b.after) {
            if run.iter().all(|l| l.before == run[0].before) {
                continue;
            }
            let mut codes: Vec<(String, String)> = Vec::new();
            for entry in run.iter().flat_map(|l| &l.codes) {
                if !codes.contains(entry) {
                    codes.push(entry.clone());
                }
            }
            // Narrowest identity first — see `Rewrite::collisions`.
            let at = found
                .iter()
                .position(|(w, _)| *w > ids.len())
                .unwrap_or(found.len());
            found.insert(
                at,
                (
                    ids.len(),
                    KeyCollision {
                        group: code.to_string(),
                        key: run[0].after.clone(),
                        rows: run
                            .iter()
                            .map(|l| (l.input, l.row, l.before.clone()))
                            .collect(),
                        codes,
                    },
                ),
            );
        }
    }
    found.into_iter().map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use laterite_ags4_parse::parse_str;
    use std::fmt::Write as _;

    fn dict() -> Dictionary<'static> {
        Dictionary::bundled(DictVersion::V4_1_1)
    }

    /// A small delivery: TRAN with `+` as the concatenator, an ABBR row per
    /// `(heading, code)` given, and TRIG rows carrying `conds` in `TRIG_COND`.
    fn delivery(abbr: &[(&str, &str, &str)], samp_types: &[&str], conds: &[&str]) -> ParsedFile {
        let mut s = String::from(
            "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_ISNO\",\"TRAN_RCON\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"1\",\"+\"\r\n\r\n\
             \"GROUP\",\"ABBR\"\r\n\"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n",
        );
        for (h, c, d) in abbr {
            write!(s, "\"DATA\",\"{h}\",\"{c}\",\"{d}\"\r\n").unwrap();
        }
        s.push_str(
            "\r\n\"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\
             \"SAMP_TYPE\",\"SAMP_ID\"\r\n\"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
             \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n",
        );
        for t in samp_types {
            write!(s, "\"DATA\",\"BH1\",\"1.00\",\"1\",\"{t}\",\"S1\"\r\n").unwrap();
        }
        s.push_str(
            "\r\n\"GROUP\",\"TRIG\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\
             \"SAMP_TYPE\",\"SAMP_ID\",\"SPEC_REF\",\"SPEC_DPTH\",\"TRIG_COND\"\r\n\
             \"UNIT\",\"\",\"m\",\"\",\"\",\"\",\"\",\"m\",\"\"\r\n\
             \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\",\"X\",\"2DP\",\"PA\"\r\n",
        );
        for (n, c) in conds.iter().enumerate() {
            write!(
                s,
                "\"DATA\",\"BH1\",\"1.00\",\"1\",\"U\",\"S1\",\"{n}\",\"1.00\",\"{c}\"\r\n"
            )
            .unwrap();
        }
        parse_str(&s).expect("fixture parses")
    }

    fn recode(h: &str, from: &str, to: &str) -> Recode {
        let mut m = Recode::new();
        m.entry(h.to_string())
            .or_default()
            .insert(from.to_string(), to.to_string());
        m
    }

    #[test]
    fn the_mode_vocabulary_round_trips_and_names_itself_when_refused() {
        for m in CodeCaseMode::ALL {
            assert_eq!(m.as_str().parse::<CodeCaseMode>(), Ok(m));
        }
        let e = "first".parse::<CodeCaseMode>().unwrap_err();
        assert!(e.contains("keep, standard"), "{e}");
    }

    #[test]
    fn a_case_set_is_found_across_inputs_and_named_with_its_inputs() {
        let a = delivery(
            &[("TRIG_COND", "Undisturbed", "u")],
            &["U"],
            &["Undisturbed"],
        );
        let b = delivery(
            &[("TRIG_COND", "UNDISTURBED", "U")],
            &["U"],
            &["UNDISTURBED"],
        );
        let r = resolve_mapping(&[a, b], &dict(), &Recode::new(), CodeCaseMode::Keep).unwrap();
        assert!(r.mapping.is_empty(), "keep rewrites nothing");
        assert_eq!(r.case_sets.len(), 1);
        let s = &r.case_sets[0];
        assert_eq!(s.heading, "TRIG_COND");
        assert_eq!(
            s.spellings,
            vec![
                Spelling {
                    code: "Undisturbed".into(),
                    inputs: vec![0]
                },
                Spelling {
                    code: "UNDISTURBED".into(),
                    inputs: vec![1]
                },
            ]
        );
        assert_eq!(s.suggested.as_deref(), Some("UNDISTURBED"));
        assert_eq!(s.resolved, None);
    }

    #[test]
    fn a_single_input_case_set_is_found_too() {
        let a = delivery(
            &[
                ("TRIG_COND", "Undisturbed", "u"),
                ("TRIG_COND", "UNDISTURBED", "U"),
            ],
            &["U"],
            &["Undisturbed", "UNDISTURBED"],
        );
        let r = resolve_mapping(&[a], &dict(), &Recode::new(), CodeCaseMode::Keep).unwrap();
        assert_eq!(r.case_sets.len(), 1);
        assert_eq!(r.case_sets[0].spellings[1].inputs, vec![0]);
    }

    #[test]
    fn standard_uses_a_lone_standard_match_even_when_no_input_spelled_it() {
        let a = delivery(
            &[("TRIG_COND", "Undisturbed", "u")],
            &["U"],
            &["Undisturbed"],
        );
        let b = delivery(
            &[("TRIG_COND", "undisturbed", "U")],
            &["U"],
            &["undisturbed"],
        );
        let r = resolve_mapping(&[a, b], &dict(), &Recode::new(), CodeCaseMode::Standard).unwrap();
        let m = &r.mapping["TRIG_COND"];
        assert_eq!(m["Undisturbed"], "UNDISTURBED");
        assert_eq!(m["undisturbed"], "UNDISTURBED");
        assert_eq!(r.case_sets[0].resolved.as_deref(), Some("UNDISTURBED"));
    }

    #[test]
    fn standard_leaves_a_set_with_no_standard_match_and_still_reports_it() {
        let a = delivery(&[("TRIG_COND", "Wobbly", "w")], &["U"], &["Wobbly"]);
        let b = delivery(&[("TRIG_COND", "WOBBLY", "W")], &["U"], &["WOBBLY"]);
        let r = resolve_mapping(&[a, b], &dict(), &Recode::new(), CodeCaseMode::Standard).unwrap();
        assert!(r.mapping.is_empty());
        assert_eq!(r.case_sets.len(), 1);
        assert_eq!(r.case_sets[0].suggested, None);
        assert_eq!(r.case_sets[0].resolved, None);
    }

    #[test]
    fn standard_leaves_a_set_with_several_standard_matches() {
        // The standard itself carries "CONSTANT HEAD" and "Constant Head".
        let a = delivery(&[("PTST_TYPE", "constant head", "c")], &["U"], &[]);
        let b = delivery(&[("PTST_TYPE", "CONSTANT HEAD", "C")], &["U"], &[]);
        let r = resolve_mapping(&[a, b], &dict(), &Recode::new(), CodeCaseMode::Standard).unwrap();
        assert!(r.mapping.is_empty());
        assert_eq!(r.case_sets[0].suggested, None);
    }

    #[test]
    fn recode_takes_precedence_over_standard_for_the_sets_it_names() {
        let a = delivery(
            &[("TRIG_COND", "Undisturbed", "u")],
            &["U"],
            &["Undisturbed"],
        );
        let b = delivery(
            &[("TRIG_COND", "UNDISTURBED", "U")],
            &["U"],
            &["UNDISTURBED"],
        );
        let c = delivery(
            &[("TRIG_COND", "undisturbed", "U")],
            &["U"],
            &["undisturbed"],
        );
        let rc = recode("TRIG_COND", "UNDISTURBED", "Undisturbed");
        let r = resolve_mapping(&[a, b, c], &dict(), &rc, CodeCaseMode::Standard).unwrap();
        // The set still holds "Undisturbed" and "undisturbed", but the caller
        // named "Undisturbed", so standard does not re-spell it.
        assert_eq!(r.mapping["TRIG_COND"].len(), 1);
        assert_eq!(r.mapping["TRIG_COND"]["UNDISTURBED"], "Undisturbed");
    }

    #[test]
    fn a_recode_naming_a_non_pa_heading_or_an_absent_code_is_refused_by_name() {
        let a = delivery(
            &[("TRIG_COND", "Undisturbed", "u")],
            &["U"],
            &["Undisturbed"],
        );
        let e = resolve_mapping(
            std::slice::from_ref(&a),
            &dict(),
            &recode("SAMP_REF", "1", "2"),
            CodeCaseMode::Keep,
        )
        .unwrap_err();
        assert_eq!(e, RecodeError::NotPaHeading("SAMP_REF".into()));
        assert!(e.to_string().contains("SAMP_REF"));
        let e = resolve_mapping(
            std::slice::from_ref(&a),
            &dict(),
            &recode("TRIG_COND", "UNDISTURBED", "X"),
            CodeCaseMode::Keep,
        )
        .unwrap_err();
        assert!(matches!(e, RecodeError::AbsentCode { .. }));
        assert!(e.to_string().contains("\"UNDISTURBED\""), "{e}");
    }

    #[test]
    fn the_rewrite_reaches_concatenated_parts_and_collapses_abbr() {
        let a = delivery(
            &[
                ("TRIG_COND", "Undisturbed", "mine"),
                ("TRIG_COND", "B", "b"),
            ],
            &["U"],
            &["Undisturbed+B"],
        );
        let rc = recode("TRIG_COND", "Undisturbed", "UNDISTURBED");
        let res =
            resolve_mapping(std::slice::from_ref(&a), &dict(), &rc, CodeCaseMode::Keep).unwrap();
        let w = plan_rewrite(std::slice::from_ref(&a), &dict(), &res.mapping);
        assert!(w.collisions.is_empty());
        let trig: Vec<&str> = w
            .edits
            .iter()
            .filter(|e| e.group == "TRIG")
            .map(|e| e.value.as_str())
            .collect();
        assert_eq!(trig, vec!["UNDISTURBED+B"]);
        // The target is standard, so its row takes the standard description.
        let abbr: Vec<&str> = w
            .edits
            .iter()
            .filter(|e| e.group == "ABBR")
            .map(|e| e.value.as_str())
            .collect();
        assert_eq!(abbr[0], "UNDISTURBED");
        assert_eq!(Some(abbr[1]), dict().abbr_desc("TRIG_COND", "UNDISTURBED"));
    }

    #[test]
    fn a_declared_target_keeps_its_own_row_and_the_mapped_rows_drop() {
        let a = delivery(
            &[("TRIG_COND", "Undisturbed", "u")],
            &["U"],
            &["Undisturbed"],
        );
        let b = delivery(
            &[("TRIG_COND", "UNDISTURBED", "theirs")],
            &["U"],
            &["UNDISTURBED"],
        );
        let files = [a, b];
        let rc = recode("TRIG_COND", "Undisturbed", "UNDISTURBED");
        let res = resolve_mapping(&files, &dict(), &rc, CodeCaseMode::Keep).unwrap();
        let w = plan_rewrite(&files, &dict(), &res.mapping);
        assert_eq!(
            w.dropped,
            vec![RowRef {
                input: 0,
                group: "ABBR".into(),
                row: 0
            }]
        );
        assert!(w.edits.iter().all(|e| e.group != "ABBR"));
    }

    #[test]
    fn a_non_standard_target_keeps_the_first_mapped_rows_description() {
        let a = delivery(&[("TRIG_COND", "Odd", "first")], &["U"], &["Odd"]);
        let b = delivery(&[("TRIG_COND", "ODD", "second")], &["U"], &["ODD"]);
        let files = [a, b];
        let mut rc = recode("TRIG_COND", "Odd", "Even");
        rc.get_mut("TRIG_COND")
            .unwrap()
            .insert("ODD".into(), "Even".into());
        let res = resolve_mapping(&files, &dict(), &rc, CodeCaseMode::Keep).unwrap();
        let w = plan_rewrite(&files, &dict(), &res.mapping);
        // Input 0's row becomes "Even" with its own description; input 1's drops.
        let abbr: Vec<&CellEdit> = w.edits.iter().filter(|e| e.group == "ABBR").collect();
        assert_eq!(abbr.len(), 1);
        assert_eq!((abbr[0].input, abbr[0].value.as_str()), (0, "Even"));
        assert_eq!(w.dropped.len(), 1);
        assert_eq!(w.dropped[0].input, 1);
    }

    #[test]
    fn a_rewrite_that_merges_two_keyed_rows_is_a_collision() {
        let a = delivery(
            &[("SAMP_TYPE", "U", "u"), ("SAMP_TYPE", "u", "u")],
            &["U", "u"],
            &[],
        );
        let files = [a];
        let res = resolve_mapping(&files, &dict(), &Recode::new(), CodeCaseMode::Standard).unwrap();
        assert_eq!(res.mapping["SAMP_TYPE"]["u"], "U");
        let w = plan_rewrite(&files, &dict(), &res.mapping);
        assert_eq!(w.collisions.len(), 1, "{w:?}");
        let c = &w.collisions[0];
        assert_eq!(c.group, "SAMP");
        assert_eq!(c.rows.len(), 2);
        assert_eq!(c.codes, vec![("SAMP_TYPE".to_string(), "u".to_string())]);
    }

    #[test]
    fn rows_that_already_shared_a_key_are_not_a_collision() {
        let a = delivery(&[("SAMP_TYPE", "u", "u")], &["u", "u"], &[]);
        let res = plan_rewrite(
            std::slice::from_ref(&a),
            &dict(),
            &recode("SAMP_TYPE", "u", "U"),
        );
        assert!(res.collisions.is_empty(), "{res:?}");
    }
}
