//! Relational rules: AGS4.1/4.2 Rules 10a, 10b, 10c, 11 (→ 11a/11b)
//! and 11c.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn *which* relational checks it performs and *how* it
//! interprets the prose (facts about the AGS standard, not
//! copyrightable). No code, structure, or wording was copied.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, pp.155–156 — AGS 4.1 identical):
//!
//! * **Rule 10**  — "HEADINGs are defined as KEY, REQUIRED or OTHER. …"
//! * **Rule 10a** — "In every GROUP, certain HEADINGs are defined as
//!   KEY. There shall not be more than one row of data in each GROUP
//!   with the same combination of KEY field entries. KEY fields must
//!   appear in each GROUP, but may contain null data (see Rule 12)."
//! * **Rule 10b** — "Some HEADINGs are marked as REQUIRED. REQUIRED
//!   fields must appear in the data GROUPs where they are indicated in
//!   the AGS FORMAT DATA DICTIONARY. These fields require data entry
//!   and cannot be null (i.e. left blank or empty)."
//! * **Rule 10c** — "Links are made between data rows in GROUPs by the
//!   KEY fields. Every entry made in the KEY fields in any GROUP must
//!   have an equivalent entry in its PARENT GROUP. The PARENT GROUP
//!   must be included within the data file."
//! * **Rule 11**  — "HEADINGs defined as a data TYPE of 'Record Link'
//!   (RL) can be used to link data rows to entries in GROUPs outside
//!   of the defined hierarchy (Rule 10c) or DICT group for user
//!   defined GROUPs. The GROUP name followed by the KEY FIELDs
//!   defining the cross-referenced data row, in the order presented in
//!   the AGS4 DATA DICTIONARY."
//! * **Rule 11a** — "Each GROUP/KEY FIELD shall be separated by a
//!   delimiter character … defined in `TRAN_DLIM`. The default being
//!   '|' (ASCII 124)."
//! * **Rule 11b** — "… more than one combination … separated by a
//!   defined concatenation character … defined in `TRAN_RCON`. The
//!   default being '+' (ASCII 43)."
//! * **Rule 11c** — "Any heading of data TYPE 'Record Link' included
//!   in a data file shall cross-reference to the KEY FIELDs of data
//!   rows in the GROUP referred to by the heading contents."
//!
//! The KEY/REQUIRED status and parent of each group come from an
//! *effective* dictionary = the bundled standard dictionary with the
//! file's own DICT group overlaid (Rule 9/18 territory; consumed here,
//! validated in V6). Rule 10c skips a hardcoded set of parentless /
//! implicitly-linked groups exactly as python-ags4 does — see
//! OBSERVATIONS O-21/O-22/O-23/O-24.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::CheckOptions;
use crate::dict::Dictionary;
use crate::effective_dict::EffectiveDict;
use crate::findings::{Findings, Location, Severity, Target, add, add_at};
use crate::parse::{ParsedFile, ParsedGroup};

const RULE_10A: &str = "AGS Format Rule 10a";
const RULE_10B: &str = "AGS Format Rule 10b";
const RULE_10C: &str = "AGS Format Rule 10c";
/// The tier a reader actually meets (warnings are on by default where FYI
/// is not) for the parentage check this rule DECLINES to make — see the
/// all-empty-key arm below.
const RULE_10C_WARN: &str = "Warning (Related to Rule 10c)";
/// The advisory tier for a link 10c cannot ask about at all — a KEY heading
/// owned by a group off the declared parent chain (#759). Separate from the
/// warning above because that one is a row that DECLINED a check this rule
/// could have made; this one is a check the rule was never able to make.
const RULE_10C_FYI: &str = "FYI (Related to Rule 10c)";
const RULE_11A: &str = "AGS Format Rule 11a";
const RULE_11B: &str = "AGS Format Rule 11b";
const RULE_11C: &str = "AGS Format Rule 11c";

/// Groups with no checkable parent linkage. python-ags4 hardcodes this
/// exact list: roots (PROJ/TRAN/…) plus groups whose link to PROJ/LOCA
/// is implicit, not a repeated-KEY relation (LOCA's `DICT_PGRP` is
/// `PROJ`, yet a LOCA row carries no PROJ key — so 10c can't be run on
/// it). Replicated verbatim for parity + correctness (O-21).
const PARENTLESS: &[&str] = &[
    "PROJ", "TRAN", "ABBR", "DICT", "UNIT", "TYPE", "LOCA", "FILE", "LBSG", "PREM", "STND",
];

/// The published 3-argument entry point, kept because `rules` is public API
/// and crates.io freezes what ships: breaking it would force a version bump
/// across every crate in the workspace to change one finding's tier.
///
/// It means exactly what it always meant. `CheckOptions::default()` is
/// errors-only, and before #656 this family emitted nothing above that tier —
/// so a caller on the frozen signature gets byte-identical findings to the
/// ones it got before the declined-parentage warning existed. [`check_with`]
/// is the tier-aware form, and the two should converge on the next breaking
/// bump (`groups::check` and `line_format::check` already take options).
pub fn check(parsed: &ParsedFile, dict: &Dictionary<'_>, found: &mut Findings) {
    check_with(parsed, dict, &CheckOptions::default(), found);
}

/// Rules 10a-10c and 11a-11c, honouring the tier flags.
pub fn check_with(
    parsed: &ParsedFile,
    dict: &Dictionary<'_>,
    opts: &CheckOptions,
    found: &mut Findings,
) {
    let eff = EffectiveDict::build(parsed, *dict);

    // A parent's KEY-tuple set depends only on the parent group, never on which
    // child is asking, so it is memoised by parent code across the whole file.
    // Rule 10c runs once per group and many groups share a parent — SAMP alone
    // parents the entire lab-test family — so rebuilding the set per child was
    // the bulk of this rule's cost (see [[perf-campaign]] T1). The tuples borrow
    // from `parsed`, which outlives the loop.
    let mut parent_tuples: HashMap<String, HashSet<Vec<&str>>> = HashMap::new();

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        rule_10a(g, code, &eff, found);
        rule_10b(g, code, &eff, found);
        rule_10c(parsed, g, code, &eff, opts, found, &mut parent_tuples);
        rule_10c_unchecked_link(g, code, &eff, opts, found);
    }

    rule_11(parsed, found);
}

/// Column index of `name` in a group's HEADING row.
fn col(g: &ParsedGroup, name: &str) -> Option<usize> {
    g.headings.iter().position(|h| h == name)
}

/// Column indices of `names`, resolved ONCE for a group.
///
/// The indices do not vary by row, but the per-row form of this
/// (`col()` inside `tuple()`) re-derived them for every row — and `col` is a
/// linear scan of the heading list, so a group of R rows with K keys and H
/// headings paid R*K*H string comparisons to answer the same K questions.
/// Hoisting it makes that R*K pointer reads. See [[core-perf-baseline]].
fn cols(g: &ParsedGroup, names: &[String]) -> Vec<Option<usize>> {
    names.iter().map(|n| col(g, n)).collect()
}

/// The row's values at `idx`, in `idx` order, BORROWED from the row.
///
/// Same contract as the earlier owned version — a missing column or a ragged
/// row yields `""` so tuples stay positional — but it no longer clones every
/// cell. These tuples exist only to be hashed and compared; the rows already
/// own the text, so the clone bought nothing.
fn tuple_at<'a>(idx: &[Option<usize>], row: &crate::parse::DataRow, buf: &'a str) -> Vec<&'a str> {
    idx.iter()
        .map(|i| i.and_then(|i| row.values.get(i)).map_or("", |s| s.slice(buf)))
        .collect()
}

/// Rule 10a — KEY fields present; no duplicate KEY combinations.
// `ri` is a DATA row's index within one AGS4 group — bounded by that
// group's actual row count, far below u32::MAX for any real AGS4 file.
#[allow(clippy::cast_possible_truncation)]
fn rule_10a(g: &ParsedGroup, code: &str, eff: &EffectiveDict<'_>, found: &mut Findings) {
    let keys = eff.key_fields(code);
    if keys.is_empty() {
        return;
    }

    let mut all_present = true;
    for k in &keys {
        if !g.headings.iter().any(|h| h == k) {
            all_present = false;
            if let Some(hl) = g.heading_line {
                add_at(
                    found,
                    RULE_10A,
                    Some(hl),
                    code,
                    format!("KEY field {k} is not present."),
                    Location {
                        target: Target::Heading,
                        heading: Some(k.clone()),
                        ..Default::default()
                    },
                    Severity::Error,
                );
            }
        }
    }
    if !all_present {
        return; // can't trust the combination check without all keys
    }

    let idx = cols(g, &keys);
    let mut counts: HashMap<Vec<&str>, usize> = HashMap::new();
    for row in &g.rows {
        *counts.entry(tuple_at(&idx, row, g.text())).or_default() += 1;
    }
    for (ri, row) in g.rows.iter().enumerate() {
        let t = tuple_at(&idx, row, g.text());
        if counts.get(&t).copied().unwrap_or(0) > 1 {
            add_at(
                found,
                RULE_10A,
                Some(row.line),
                code,
                format!("Duplicate KEY field combination: {}", t.join("|")),
                Location {
                    target: Target::Cell,
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }
}

/// Rule 10b — REQUIRED fields present and non-empty in every DATA row.
// `ri` is bounded the same way as in `rule_10a` above.
#[allow(clippy::cast_possible_truncation)]
fn rule_10b(g: &ParsedGroup, code: &str, eff: &EffectiveDict<'_>, found: &mut Findings) {
    let req = eff.required_fields(code);
    if req.is_empty() {
        return;
    }

    let mut present: Vec<&String> = Vec::new();
    for r in &req {
        if g.headings.iter().any(|h| h == r) {
            present.push(r);
        } else if let Some(hl) = g.heading_line {
            add_at(
                found,
                RULE_10B,
                Some(hl),
                code,
                format!("REQUIRED field {r} is not present."),
                Location {
                    target: Target::Heading,
                    heading: Some(r.clone()),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }

    // Map REQUIRED heading name → column index (skip those that aren't
    // present — `req_cols` carries only resolvable REQUIRED columns).
    let req_cols: Vec<(usize, &str)> = present
        .iter()
        .filter_map(|r| col(g, r).map(|i| (i, r.as_str())))
        .collect();

    for (ri, row) in g.rows.iter().enumerate() {
        let any_empty = req_cols
            .iter()
            .any(|(i, _)| {
                row.values
                    .get(*i)
                    .is_none_or(|v| v.slice(g.text()).trim().is_empty())
            });
        if !any_empty {
            continue;
        }
        // Reconstruct the DATA row with `|`-separator, substituting
        // `??<FIELD_NAME>??` where a REQUIRED column is empty —
        // python-ags4's `rule_10b` row format. Leading `DATA|` matches
        // the row tag; trailing empty columns stay as bare `|`s.
        let empty_at: std::collections::HashMap<usize, &str> = req_cols
            .iter()
            .copied()
            .filter(|(i, _)| {
                row.values
                    .get(*i)
                    .is_none_or(|v| v.slice(g.text()).trim().is_empty())
            })
            .collect();
        let mut parts: Vec<String> = Vec::with_capacity(g.headings.len() + 1);
        parts.push("DATA".to_string());
        for (i, _) in g.headings.iter().enumerate() {
            let v = row.values.get(i).map_or("", |s| s.slice(g.text()));
            if let Some(name) = empty_at.get(&i) {
                parts.push(format!("??{name}??"));
            } else {
                parts.push(v.to_string());
            }
        }
        add_at(
            found,
            RULE_10B,
            Some(row.line),
            code,
            format!("Empty REQUIRED fields: {}", parts.join("|")),
            Location {
                target: Target::Cell,
                data_row: Some(ri as u32 + 1),
                ..Default::default()
            },
            Severity::Error,
        );
    }
}

/// Rule 10c — every child row must have a matching parent row, keyed
/// by the parent's KEY fields.
// `ri` is bounded the same way as in `rule_10a` above.
#[allow(clippy::cast_possible_truncation)]
fn rule_10c<'p>(
    parsed: &'p ParsedFile,
    g: &'p ParsedGroup,
    code: &str,
    eff: &EffectiveDict<'_>,
    opts: &CheckOptions,
    found: &mut Findings,
    parent_tuples: &mut HashMap<String, HashSet<Vec<&'p str>>>,
) {
    if PARENTLESS.contains(&code) {
        return;
    }
    let Some(parent) = eff.parent(code) else {
        add(
            found,
            RULE_10C,
            None,
            code,
            "Could not check parent entries: group not defined in the standard dictionary or DICT group."
                .to_string(),
        );
        return;
    };
    if parent.is_empty() {
        add(
            found,
            RULE_10C,
            None,
            code,
            "Parent group is left blank in the dictionary.".to_string(),
        );
        return;
    }
    let Some(pg) = parsed.groups.get(parent) else {
        add(
            found,
            RULE_10C,
            None,
            code,
            format!("Parent group {parent} is not in the file."),
        );
        return;
    };

    let pkeys = eff.key_fields(parent);
    if pkeys.is_empty() {
        add(
            found,
            RULE_10C,
            None,
            code,
            format!("No KEY fields are defined in the parent group ({parent})."),
        );
        return;
    }
    let ckeys = eff.key_fields(code);
    let missing: Vec<&String> = pkeys.iter().filter(|p| !ckeys.contains(p)).collect();
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        add(
            found,
            RULE_10C,
            None,
            code,
            format!(
                "{} defined as KEY field(s) in the parent group ({parent}) but not in the child group.",
                names.join(", ")
            ),
        );
        return;
    }

    let in_child = pkeys.iter().all(|k| col(g, k).is_some());
    let in_parent = pkeys.iter().all(|k| col(pg, k).is_some());
    if !(in_child && in_parent) {
        if let Some(hl) = g.heading_line {
            add(
                found,
                RULE_10C,
                Some(hl),
                code,
                format!(
                    "Could not check parent entries: KEY fields missing in {code} or {parent} \
                     (see Rule 10a)."
                ),
            );
        }
        return;
    }

    // Child indices are per-child, so resolved here; the parent tuple SET is
    // per-parent, so built once and cached (see `check`). `entry` clones the
    // parent code once per child — cheap beside the row scan it replaces.
    let cidx = cols(g, &pkeys);
    let ptuples = parent_tuples.entry(parent.to_string()).or_insert_with(|| {
        let pidx = cols(pg, &pkeys);
        pg.rows.iter().map(|r| tuple_at(&pidx, r, pg.text())).collect()
    });
    for (ri, row) in g.rows.iter().enumerate() {
        let t = tuple_at(&cidx, row, g.text());
        // O-39: a child row whose parent KEY cells are ALL empty is
        // "standalone" by the file's own design (a lab-control SAMP with
        // no LOCA borehole, an off-site sample), so the link requirement
        // is read as applying to entries MADE — and an empty cell is not
        // an entry. A row with even one non-empty parent KEY field IS
        // claiming a parent and gets the usual check.
        //
        // Declining to check is not the same as checking and finding
        // nothing, and the reader could not tell which they got: the row
        // simply produced no finding. A standalone row and a row whose key
        // was dropped or typo'd to blank look identical here, and only the
        // author knows which they meant. So the skip SAYS it happened
        // (#656), at the warning tier — the one shown by default, unlike
        // FYI. It never changes the verdict.
        if t.iter().all(|s| s.trim().is_empty()) {
            if !opts.include_warnings {
                continue; // errors-only: `--no-warnings` means what it says
            }
            add_at(
                found,
                RULE_10C_WARN,
                Some(row.line),
                code,
                format!(
                    "Parentage not checked: every {parent} key field on this \
                     row ({}) is empty, so the row claims no parent; a \
                     standalone record and a blanked key look the same here.",
                    pkeys.join(", ")
                ),
                Location {
                    target: Target::Cell,
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                },
                Severity::Warning,
            );
            continue;
        }
        if !ptuples.contains(&t) {
            add_at(
                found,
                RULE_10C,
                Some(row.line),
                code,
                format!(
                    "No parent entry in {parent} for KEY combination: {}",
                    t.join("|")
                ),
                Location {
                    target: Target::Cell,
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }
}

/// Rule 10c FYI — a KEY heading owned by a group that is NOT on the declared
/// parent chain, where that group could have been the declared parent.
///
/// 10c asks exactly one question per group: does this row's copy of the
/// PARENT's KEY tuple exist in the parent? A KEY heading belonging to any other
/// group is not in that tuple, so the link it stands for is never asked about —
/// a child can name a row that does not exist and the file validates clean
/// (#759). AGS4 gives a group one `DICT_PGRP`, so this is not a defect in the
/// file or in the rule: it is a link the format cannot express and the
/// validator therefore cannot check. Hence the advisory tier — it never moves
/// the verdict, and it is off unless the caller asks for FYI.
///
/// Reported ONLY where the child's KEY tuple contains the owner's WHOLE KEY
/// tuple. Without that test the finding is a false positive, because a group
/// that could not have been the parent is not a missed parent: `DISC` is the
/// shipped counter-example — it keys on `FRAC_SET` but not on FRAC's
/// `FRAC_FROM`/`FRAC_TO`, so FRAC was never a candidate and DISC stays silent.
///
/// Two classes are dropped, both on purpose:
///
/// * the [`PARENTLESS`] groups, where 10c checks NO link at all (O-21) — an
///   advisory naming one of them would imply the rest were checked;
/// * a KEY heading whose prefix names no group holding a KEY tuple of its own.
///   The containment test does this one on its way past — a group with no
///   tuple has nothing to be contained in the child's — rather than any check
///   on the prefix itself. It is not an edge case: `SPEC_REF`/`SPEC_DPTH` are
///   KEY across the whole lab-test family and there has never been a SPEC
///   group, specimen identity living on the test groups themselves. Every one
///   of those groups would otherwise carry a permanent advisory about a parent
///   that does not exist.
fn rule_10c_unchecked_link(
    g: &ParsedGroup,
    code: &str,
    eff: &EffectiveDict<'_>,
    opts: &CheckOptions,
    found: &mut Findings,
) {
    if !opts.include_fyi || PARENTLESS.contains(&code) {
        return;
    }
    // A missing or blank parent is 10c's own error to report, and leaves no
    // declared chain for a heading to be outside of.
    let Some(parent) = eff.parent(code).filter(|p| !p.is_empty()) else {
        return;
    };
    let ckeys = eff.key_fields(code);
    if ckeys.is_empty() {
        return;
    }
    let chain = eff.ancestry(code);

    // A heading names its owner in the prefix Rule 19b's naming law gives it, so
    // that is where the owner is read from. Whether the prefix is WELL-formed is
    // not asked here: 19b reports a malformed one already, and the containment
    // filter below needs the owner to be a group carrying a KEY tuple, which no
    // malformed prefix is. A length test here would only be 19b's rule restated
    // in bytes where 19b counts chars, and would disagree with it on both sides.
    let mut by_owner: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for k in &ckeys {
        let Some((owner, _)) = k.split_once('_') else {
            continue;
        };
        if !chain.contains(owner) {
            by_owner.entry(owner).or_default().push(k.as_str());
        }
    }

    let ckeyset: HashSet<&str> = ckeys.iter().map(String::as_str).collect();
    let candidates: Vec<(&str, Vec<&str>)> = by_owner
        .into_iter()
        .filter(|(owner, _)| {
            let okeys = eff.key_fields(owner);
            !okeys.is_empty() && okeys.iter().all(|o| ckeyset.contains(o.as_str()))
        })
        .collect();
    let ancestries: Vec<HashSet<String>> =
        candidates.iter().map(|(o, _)| eff.ancestry(o)).collect();

    for (i, (owner, hdngs)) in candidates.iter().enumerate() {
        // A candidate that is an ANCESTOR of another candidate says nothing the
        // more specific one doesn't: LBST's unchecked LOCA link is a consequence
        // of its unchecked SAMP link, and naming both reports one gap twice.
        if ancestries
            .iter()
            .enumerate()
            .any(|(j, a)| j != i && a.contains(*owner))
        {
            continue;
        }
        add_at(
            found,
            RULE_10C_FYI,
            g.heading_line,
            code,
            format!(
                "{code} keys on {}, owned by {owner}, while declaring {parent} as its \
                 parent. Every KEY field of {owner} is present in {code}, so those \
                 cells identify an {owner} row too — and Rule 10c checks the declared \
                 {parent} link only, so nothing verifies that row exists.",
                hdngs.join(", ")
            ),
            Location {
                target: Target::Group,
                ..Default::default()
            },
            Severity::Fyi,
        );
    }
}

/// Rule 11 — read `TRAN_DLIM` / `TRAN_RCON`, dispatch 11a/11b/11c.
fn rule_11(parsed: &ParsedFile, found: &mut Findings) {
    let Some(tran) = parsed.groups.get("TRAN") else {
        return; // TRAN missing → Rule 14 reports it
    };
    let Some(data) = tran.rows.first() else {
        return; // no DATA row → Rule 14
    };
    // Absent columns: python-ags4 swallows the KeyError → no finding.
    // We mirror that (TRAN_DLIM/RCON are OTHER, not REQUIRED). (O-23)
    let (Some(di), Some(ci)) = (col(tran, "TRAN_DLIM"), col(tran, "TRAN_RCON")) else {
        return;
    };
    let delim = data.values.get(di).map_or("", |s| s.slice(tran.text()));
    let concat = data.values.get(ci).map_or("", |s| s.slice(tran.text()));

    let mut blocked = false;
    if delim.is_empty() {
        add(
            found,
            RULE_11A,
            Some(data.line),
            "TRAN",
            "TRAN_DLIM is missing.",
        );
        blocked = true;
    }
    if concat.is_empty() {
        add(
            found,
            RULE_11B,
            Some(data.line),
            "TRAN",
            "TRAN_RCON is missing.",
        );
        blocked = true;
    }
    if blocked {
        return; // 11c needs a usable delimiter + concatenator
    }
    rule_11c(parsed, delim, concat, found);
}

/// Rule 11c — every Record-Link value must resolve to exactly one row
/// in the referenced GROUP (positional match against its leading
/// columns, the AGS4 "GROUP|key1|key2…" form).
// `ci`/`ri` are a column/row index within one AGS4 group — both bounded
// far below u32::MAX for any real AGS4 file (see `rule_10a` above).
#[allow(clippy::cast_possible_truncation)]
fn rule_11c(parsed: &ParsedFile, delim: &str, concat: &str, found: &mut Findings) {
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() != "RL" {
                continue;
            }
            for (ri, row) in g.rows.iter().enumerate() {
                let Some(rl) = row.values.get(ci).map(|s| s.slice(g.text())) else {
                    continue;
                };
                if rl.is_empty() {
                    continue;
                }
                let loc = || Location {
                    target: Target::Cell,
                    field_index: Some(ci as u32),
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                };
                if !rl.contains(delim) {
                    add_at(
                        found,
                        RULE_11C,
                        Some(row.line),
                        code,
                        format!(
                            "Invalid Record Link {rl:?}: {delim:?} must separate the GROUP \
                             and KEY fields."
                        ),
                        loc(),
                        Severity::Error,
                    );
                    continue;
                }
                for link in rl.split(concat) {
                    let parts: Vec<&str> = link.split(delim).collect();
                    match fetch_count(parsed, &parts) {
                        0 => add_at(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: no such record."),
                            loc(),
                            Severity::Error,
                        ),
                        1 => {}
                        _ => add_at(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: matches more than one record."),
                            loc(),
                            Severity::Error,
                        ),
                    }
                }
            }
        }
    }
}

/// How many DATA rows in `parts[0]` match `parts[1..]` positionally
/// against the group's leading heading columns (python-ags4's
/// `fetch_record` is positional, not KEY-aware — O-24).
fn fetch_count(parsed: &ParsedFile, parts: &[&str]) -> usize {
    let Some((grp, keys)) = parts.split_first() else {
        return 0;
    };
    let Some(g) = parsed.groups.get(*grp) else {
        return 0;
    };
    if keys.is_empty() || keys.len() > g.headings.len() {
        return 0;
    }
    g.rows
        .iter()
        .filter(|r| {
            keys.iter()
                .enumerate()
                .all(|(i, k)| r.values.get(i).map_or("", |s| s.slice(g.text())) == *k)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        run_v(src, DictVersion::V4_2)
    }

    fn run_v(src: &str, v: DictVersion) -> Findings {
        run_opts(src, v, &CheckOptions::default())
    }

    /// The tier flags are honoured at each emission site, so a test that only
    /// ever runs at one tier cannot see a finding that ignores them — which is
    /// exactly how the declined-parentage warning reached `laterite.compat`.
    fn run_opts(src: &str, v: DictVersion, opts: &CheckOptions) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let d = Dictionary::bundled(v);
        let mut f = Findings::new();
        check_with(&pf, &d, opts, &mut f);
        f
    }

    /// Synthetic repro of the #222 / O-42 corpus file (no real data): PMTL's
    /// parent is **PMTD** in 4.0.3 (KEY includes `PMTD_SEQ`) but **PMTG** in
    /// 4.0.4+. This PMTL row has a blank `PMTD_SEQ`, so under 4.0.3 it orphans
    /// against PMTD (no PMTD row with that blank SEQ) while under 4.0.4 it
    /// matches its PMTG parent on the shorter key. PMTL is the only group in
    /// the dictionary with an edition-varying parent.
    const PMTL_EDITION_FIXTURE: &str = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
        \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
        \"GROUP\",\"PMTG\"\r\n\"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\"\r\n\
        \"UNIT\",\"\",\"m\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\"\r\n\r\n\
        \"GROUP\",\"PMTD\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\",\"PMTD_SEQ\"\r\n\
        \"UNIT\",\"\",\"m\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\",\"1\"\r\n\r\n\
        \"GROUP\",\"PMTL\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\",\"PMTL_LNO\",\"PMTD_SEQ\"\r\n\
        \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\",\"ID\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\",\"1\",\"\"\r\n";

    #[test]
    fn fields_with_status_dedups_a_std_heading_redeclared_in_file_dict() {
        // A file DICT overlay re-declares the standard KEY heading LOCA_ID with
        // the SAME status, so it appears in BOTH passes of fields_with_status
        // (standard headings first, then the file_hdng overlay). The dedup guard
        // must return it exactly once — a flipped guard double-pushes it, which
        // would corrupt the KEY tuple the relational rules build from this.
        let src = "\"GROUP\",\"DICT\"\r\n\
            \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
            \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
            \"DATA\",\"HEADING\",\"LOCA\",\"LOCA_ID\",\"KEY\",\"\"\r\n";
        let parsed = parse_str(src).expect("fixture parses");
        let dict = Dictionary::bundled(DictVersion::V4_2);
        let eff = EffectiveDict::build(&parsed, dict);
        let keys = eff.key_fields("LOCA");
        assert_eq!(
            keys.iter().filter(|h| h.as_str() == "LOCA_ID").count(),
            1,
            "LOCA_ID must appear once, not duplicated: {keys:?}"
        );
    }

    #[test]
    fn rule_10c_pmtl_parent_is_edition_dependent() {
        // #222 / O-42: the parent PMTL is checked against differs by edition.
        let pmtl_orphans = |v| {
            run_v(PMTL_EDITION_FIXTURE, v).get(RULE_10C).map_or(0, |f| {
                f.iter()
                    .filter(|x| x.group == "PMTL" && x.desc.contains("No parent entry"))
                    .count()
            })
        };
        // 4.0.3: PMTL→PMTD on a key incl. the (blank) PMTD_SEQ → orphan.
        assert_eq!(
            pmtl_orphans(DictVersion::V4_0_3),
            1,
            "4.0.3 should orphan PMTL→PMTD"
        );
        // 4.0.4+: PMTL→PMTG on the shorter key → matches, no orphan. python-ags4's
        // stale "4.0"→4.0.3 alias is what makes it over-report these (false positives).
        assert_eq!(
            pmtl_orphans(DictVersion::V4_0_4),
            0,
            "4.0.4 should match PMTL→PMTG"
        );
        assert_eq!(
            pmtl_orphans(DictVersion::V4_2),
            0,
            "4.2 keeps the 4.0.4 PMTL→PMTG parent"
        );
    }

    fn run_fyi(src: &str, v: DictVersion) -> Findings {
        run_opts(
            src,
            v,
            &CheckOptions {
                include_fyi: true,
                ..CheckOptions::default()
            },
        )
    }

    /// The AGS-L working group's TRIL shape (#759), rebuilt from its published
    /// parentage and KEY list — no corpus data. TRIL declares TRIG, and keys on
    /// `TRIT_TESN`, which TRIG's KEY tuple does not contain: the test-number
    /// link is one Rule 10c can never ask about. TRIL is not a standard group,
    /// so the file's own DICT is what defines it — the path a user-defined
    /// group actually arrives by.
    const TRIL_FOREIGN_KEY_FIXTURE: &str = "\"GROUP\",\"DICT\"\r\n\
        \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
        \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
        \"DATA\",\"GROUP\",\"TRIL\",\"\",\"\",\"TRIG\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"LOCA_ID\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SAMP_TOP\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SAMP_REF\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SAMP_TYPE\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SAMP_ID\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SPEC_REF\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"SPEC_DPTH\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"TRIT_TESN\",\"KEY\",\"\"\r\n\
        \"DATA\",\"HEADING\",\"TRIL\",\"TRIL_MNUM\",\"KEY\",\"\"\r\n\r\n\
        \"GROUP\",\"TRIL\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\",\
\"SPEC_REF\",\"SPEC_DPTH\",\"TRIT_TESN\",\"TRIL_MNUM\"\r\n\
        \"UNIT\",\"\",\"m\",\"\",\"\",\"\",\"\",\"m\",\"\",\"\"\r\n\
        \"TYPE\",\"ID\",\"2DP\",\"ID\",\"PA\",\"ID\",\"ID\",\"2DP\",\"ID\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"1\",\"SP1\",\"1.00\",\"T1\",\"1\"\r\n";

    #[test]
    fn rule_10c_fyi_names_the_owner_that_could_have_been_the_parent() {
        let f = run_fyi(TRIL_FOREIGN_KEY_FIXTURE, DictVersion::V4_2);
        let fyi = f.get(RULE_10C_FYI).expect("the unchecked-link FYI");
        assert_eq!(fyi.len(), 1, "one owner, one finding: {fyi:?}");
        assert_eq!(fyi[0].group, "TRIL");
        assert_eq!(fyi[0].severity, Severity::Fyi);
        assert_eq!(fyi[0].location.target, Target::Group);
        assert!(
            fyi[0].desc.contains("TRIT_TESN") && fyi[0].desc.contains("TRIG"),
            "must name both the foreign KEY and the declared parent: {}",
            fyi[0].desc
        );
        // SPEC_REF/SPEC_DPTH are KEY here and SPEC is not a group in any
        // edition, so the prefix filter has to swallow them — otherwise every
        // lab-test group in the dictionary carries this finding forever.
        assert!(!fyi[0].desc.contains("SPEC"), "{}", fyi[0].desc);
    }

    #[test]
    fn rule_10c_fyi_is_silent_at_the_default_tier() {
        // Advisory means advisory: `CheckOptions::default()` is errors-only, and
        // this must not reach a caller who never asked for the FYI tier.
        assert!(
            !run(TRIL_FOREIGN_KEY_FIXTURE).contains_key(RULE_10C_FYI),
            "the FYI fired without include_fyi"
        );
    }

    #[test]
    fn rule_10c_fyi_needs_the_whole_owner_tuple_to_be_contained() {
        // DISC keys on FRAC_SET but not on FRAC's FRAC_FROM/FRAC_TO, so FRAC
        // could NOT have been DISC's parent — the containment test is the whole
        // difference between an advisory and a false positive.
        let src = "\"GROUP\",\"DISC\"\r\n\
            \"HEADING\",\"LOCA_ID\",\"DISC_TOP\",\"DISC_BASE\",\"FRAC_SET\",\"DISC_NUMB\"\r\n\
            \"UNIT\",\"\",\"m\",\"m\",\"\",\"\"\r\n\
            \"TYPE\",\"ID\",\"2DP\",\"2DP\",\"ID\",\"ID\"\r\n\
            \"DATA\",\"BH1\",\"1.00\",\"2.00\",\"S1\",\"1\"\r\n";
        let f = run_fyi(src, DictVersion::V4_2);
        assert!(
            !f.contains_key(RULE_10C_FYI),
            "FRAC is not a candidate parent for DISC: {:?}",
            f.get(RULE_10C_FYI)
        );
    }

    #[test]
    fn rule_10c_fyi_names_only_the_most_specific_owner() {
        // LBST keys on LOCA_ID and the whole SAMP tuple while declaring LBSG,
        // so BOTH LOCA and SAMP pass containment. LOCA is SAMP's ancestor, so
        // reporting it too would report one gap twice.
        let src = "\"GROUP\",\"LBST\"\r\n\
            \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\",\
\"LBSG_REF\",\"LBST_TEST\"\r\n\
            \"UNIT\",\"\",\"m\",\"\",\"\",\"\",\"\",\"\"\r\n\
            \"TYPE\",\"ID\",\"2DP\",\"ID\",\"PA\",\"ID\",\"ID\",\"PA\"\r\n\
            \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"1\",\"SCH1\",\"MC\"\r\n";
        let fyi = run_fyi(src, DictVersion::V4_2)
            .get(RULE_10C_FYI)
            .cloned()
            .expect("LBST's unchecked SAMP link");
        assert_eq!(fyi.len(), 1, "LOCA is subsumed by SAMP: {fyi:?}");
        assert!(fyi[0].desc.contains("SAMP"), "{}", fyi[0].desc);
        assert!(
            !fyi[0].desc.starts_with("LBST keys on LOCA_ID,"),
            "the LOCA finding survived: {}",
            fyi[0].desc
        );
    }

    #[test]
    fn rule_10c_fyi_follows_the_edition_that_moved_pmtl() {
        // The same three editions O-42 splits PMTL across, seen from this rule:
        // 4.0.3 declares PMTD, so PMTD_SEQ is ON the chain and there is nothing
        // to say; 4.1 keeps PMTD_SEQ as KEY but moves the parent to PMTG, which
        // is exactly the shape this advisory exists for; 4.1.1 drops PMTD_SEQ
        // from PMTL's KEY tuple, and the shape goes away with it.
        let pmtd_advisories = |v| {
            run_fyi(PMTL_EDITION_FIXTURE, v)
                .get(RULE_10C_FYI)
                .map_or(0, |f| {
                    f.iter()
                        .filter(|x| x.group == "PMTL" && x.desc.contains("PMTD"))
                        .count()
                })
        };
        assert_eq!(
            pmtd_advisories(DictVersion::V4_0_3),
            0,
            "4.0.3 declares PMTD"
        );
        assert_eq!(pmtd_advisories(DictVersion::V4_1), 1, "4.1 declares PMTG");
        assert_eq!(
            pmtd_advisories(DictVersion::V4_2),
            0,
            "4.2 dropped PMTD_SEQ from PMTL's KEY tuple"
        );
    }

    #[test]
    fn rule_10c_fyi_survives_a_dict_declaring_a_parent_cycle() {
        // The parent chain is half file-authored, so it can be a cycle. The walk
        // must terminate rather than hang the validator on a malformed DICT.
        let src = "\"GROUP\",\"DICT\"\r\n\
            \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
            \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
            \"DATA\",\"GROUP\",\"ZZZA\",\"\",\"\",\"ZZZB\"\r\n\
            \"DATA\",\"GROUP\",\"ZZZB\",\"\",\"\",\"ZZZA\"\r\n\
            \"DATA\",\"HEADING\",\"ZZZA\",\"ZZZA_ID\",\"KEY\",\"\"\r\n\r\n\
            \"GROUP\",\"ZZZA\"\r\n\"HEADING\",\"ZZZA_ID\"\r\n\"UNIT\",\"\"\r\n\
            \"TYPE\",\"ID\"\r\n\"DATA\",\"1\"\r\n";
        // Reaching this assertion at all is the test: an unguarded walk spins.
        assert!(!run_fyi(src, DictVersion::V4_2).contains_key(RULE_10C_FYI));
    }

    #[test]
    fn rule_10a_flags_duplicate_key_and_missing_key() {
        // LOCA KEY = LOCA_ID. Two rows with the same LOCA_ID.
        let dup = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH1\",\"2.00\"\r\n";
        let r = run(dup);
        let r10a = r.get(RULE_10A).expect("Rule 10a");
        assert_eq!(
            r10a.iter().filter(|x| x.desc.contains("Duplicate")).count(),
            2
        );

        // LOCA without its LOCA_ID KEY column.
        let miss = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_FDEP\"\r\n\
                    \"UNIT\",\"m\"\r\n\"TYPE\",\"2DP\"\r\n\"DATA\",\"1.00\"\r\n";
        assert!(run(miss).get(RULE_10A).is_some_and(|v| {
            v.iter()
                .any(|x| x.desc.contains("LOCA_ID") && x.desc.contains("not present"))
        }));
    }

    #[test]
    fn rule_10b_flags_missing_and_empty_required() {
        // PROJ_ID is KEY+REQUIRED; blank in the only DATA row.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"\",\"Site\"\r\n";
        let r10b = run(src);
        let v = r10b.get(RULE_10B).expect("Rule 10b");
        assert!(
            v.iter()
                .any(|x| x.desc.contains("Empty REQUIRED") && x.line == Some(5)),
            "{v:?}"
        );
    }

    #[test]
    fn rule_10c_flags_orphan_child_row() {
        // SAMP (parent LOCA, key LOCA_ID) references a LOCA that exists,
        // plus one orphan (BH9 not in LOCA).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\
                   \"DATA\",\"BH9\",\"2.00\",\"S2\",\"B\",\"BH9S2\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.desc.contains("BH9") && x.line == Some(12)),
            "expected orphan BH9 flagged: {r10c:?}"
        );
        assert!(
            !r10c.iter().any(|x| x.desc.contains("BH1")),
            "BH1 has a parent: {r10c:?}"
        );
    }

    #[test]
    fn rule_10c_reuses_the_parent_tuple_set_across_two_children_correctly() {
        // Two children of the SAME parent LOCA. The first (SAMP) builds and
        // caches LOCA's KEY-tuple set; the second (a file-DICT child QKID, also
        // parent LOCA) must reuse that cached set — flagging its own orphan while
        // accepting its own valid reference. If the memoisation keyed or reused
        // the set wrongly, QKID would either miss BH9 or wrongly flag BH1.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"BH1\"\r\n\"DATA\",\"BH2\"\r\n\r\n\
                   \"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QKID\",\"\",\"\",\"LOCA\"\r\n\
                   \"DATA\",\"HEADING\",\"QKID\",\"LOCA_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QKID\",\"QKID_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\r\n\
                   \"GROUP\",\"QKID\"\r\n\"HEADING\",\"LOCA_ID\",\"QKID_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"K1\"\r\n\"DATA\",\"BH9\",\"K2\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        // QKID's orphan BH9 flagged from the cached LOCA set...
        assert!(
            r10c.iter()
                .any(|x| x.group == "QKID" && x.desc.contains("BH9")),
            "QKID orphan BH9 should be flagged from the cached parent set: {r10c:?}"
        );
        // ...and QKID's genuinely-valid BH1 not flagged (the cache is LOCA's, not
        // empty or SAMP's).
        assert!(
            !r10c
                .iter()
                .any(|x| x.group == "QKID" && x.desc.contains("BH1")),
            "QKID's BH1 has a real LOCA parent: {r10c:?}"
        );
        // SAMP (which populated the cache) is itself clean.
        assert!(
            !r10c.iter().any(|x| x.group == "SAMP"),
            "SAMP's only row references BH1 which exists: {r10c:?}"
        );
    }

    #[test]
    fn rule_10c_skips_rows_with_all_empty_parent_keys() {
        // O-39: a SAMP row with empty LOCA_ID is "standalone" — no
        // parent claim. Don't flag (matches python-ags4's permissive
        // spec reading; "every entry made in KEY fields" = non-empty
        // entries only). A row with non-empty LOCA_ID that has no
        // match still flags as before.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"\",\"1.00\",\"S1\",\"B\",\"LAB_CTRL\"\r\n\
                   \"DATA\",\"BH1\",\"2.00\",\"S2\",\"B\",\"BH1S2\"\r\n";
        let r = run(src);
        // Standalone row (LOCA_ID="") → no Rule 10c.
        assert!(
            r.get(RULE_10C).is_none_or(std::vec::Vec::is_empty),
            "standalone row must not flag: {:?}",
            r.get(RULE_10C)
        );
    }

    #[test]
    fn rule_11a_11b_flag_blank_tran_delims() {
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"\",\"\"\r\n";
        let r = run(src);
        assert!(r.contains_key(RULE_11A), "{r:?}");
        assert!(r.contains_key(RULE_11B), "{r:?}");
    }

    #[test]
    fn rule_11c_flags_bad_record_link() {
        // SAMP.SAMP_LINK is RL pointing at LOCA|BH404 (no such LOCA).
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH404\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        assert!(
            r11c.iter().any(|x| x.desc.contains("no such record")),
            "{r11c:?}"
        );

        // A link with no delimiter at all.
        let nodelim = "\"GROUP\",\"TRAN\"\r\n\
                       \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                       \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                       \"DATA\",\"BH1\",\"LOCABH1\"\r\n";
        assert!(
            run(nodelim)
                .get(RULE_11C)
                .is_some_and(|v| v.iter().any(|x| x.desc.contains("must separate")))
        );
    }

    #[test]
    fn rule_11c_flags_link_matching_more_than_one_record() {
        // Two LOCA rows share leading column "BH1" (LOCA's ID column
        // isn't KEY-deduped here — fetch_record is positional, O-24), so
        // a link "LOCA|BH1" resolves to 2 rows → the `_ => matches more
        // than one` arm of rule_11c.
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"BH1\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH1\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        assert!(
            r11c.iter()
                .any(|x| x.desc.contains("matches more than one record")),
            "{r11c:?}"
        );
    }

    // ---- mutation-sweep additions: finding LOCATIONS + fetch_count bounds ----
    //
    // The existing tests assert a finding *fired* and its message; these pin the
    // Location it carries (target / heading / field_index / 1-based data_row) and
    // the exact fetch_count guards — the coordinates every surface renders.

    /// Rule 10a: a missing KEY points at the HEADING row naming the field; a
    /// duplicate KEY points at the offending 1-based data row.
    #[test]
    fn rule_10a_finding_locations() {
        let dup = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH1\",\"2.00\"\r\n";
        let r = run(dup);
        let dupf = r
            .get(RULE_10A)
            .unwrap()
            .iter()
            .find(|x| x.desc.contains("Duplicate") && x.location.data_row == Some(2))
            .expect("2nd duplicate row flagged at data_row 2");
        assert_eq!(dupf.location.target, Target::Cell);

        let miss = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_FDEP\"\r\n\
                    \"UNIT\",\"m\"\r\n\"TYPE\",\"2DP\"\r\n\"DATA\",\"1.00\"\r\n";
        let missf = run(miss)
            .get(RULE_10A)
            .unwrap()
            .iter()
            .find(|x| x.desc.contains("not present"))
            .cloned()
            .expect("missing KEY flagged");
        assert_eq!(missf.location.target, Target::Heading);
        assert_eq!(missf.location.heading.as_deref(), Some("LOCA_ID"));
    }

    /// Rule 10b: a missing REQUIRED field is flagged at the HEADING row (naming
    /// it — the present-check must actually compare names); an empty REQUIRED
    /// cell is flagged at its 1-based data row.
    #[test]
    fn rule_10b_finding_locations() {
        let miss = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_NAME\"\r\n\
                    \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"Site\"\r\n";
        let missf = run(miss)
            .get(RULE_10B)
            .expect("missing REQUIRED fires")
            .iter()
            .find(|x| x.desc.contains("PROJ_ID") && x.desc.contains("not present"))
            .cloned()
            .expect("PROJ_ID missing flagged");
        assert_eq!(missf.location.target, Target::Heading);
        assert_eq!(missf.location.heading.as_deref(), Some("PROJ_ID"));

        let empty = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                     \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"\",\"Site\"\r\n";
        let r = run(empty);
        let emptyf = &r.get(RULE_10B).expect("empty REQUIRED fires")[0];
        assert_eq!(emptyf.location.target, Target::Cell);
        assert_eq!(emptyf.location.data_row, Some(1));
    }

    /// Rule 10c orphan finding points at the offending child row (1-based).
    #[test]
    fn rule_10c_orphan_finding_location() {
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\
                   \"DATA\",\"BH9\",\"2.00\",\"S2\",\"B\",\"BH9S2\"\r\n";
        let orphan = run(src)
            .get(RULE_10C)
            .expect("orphan fires")
            .iter()
            .find(|x| x.desc.contains("BH9"))
            .cloned()
            .expect("BH9 orphan");
        assert_eq!(orphan.location.target, Target::Cell);
        assert_eq!(orphan.location.data_row, Some(2)); // BH9 is SAMP's 2nd row
    }

    /// Rule 11c: a bad record link points at the RL cell (column index + 1-based
    /// row); a link resolving to EXACTLY one record is not flagged.
    #[test]
    fn rule_11c_finding_location_and_valid_link_passes() {
        let bad = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH404\"\r\n";
        let rbad = run(bad);
        let f = &rbad.get(RULE_11C).expect("bad link fires")[0];
        assert_eq!(f.location.target, Target::Cell);
        assert_eq!(f.location.field_index, Some(1)); // SAMP_LINK is column 1
        assert_eq!(f.location.data_row, Some(1)); // the only SAMP row

        let good = "\"GROUP\",\"TRAN\"\r\n\
                    \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                    \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                    \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                    \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                    \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                    \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                    \"DATA\",\"BH1\",\"LOCA|BH1\"\r\n";
        assert!(
            !run(good).contains_key(RULE_11C),
            "a link matching exactly one record must not be flagged"
        );
    }

    /// `fetch_count` guards: empty keys must NOT match every row, and fewer keys
    /// than columns is the normal (matching) case.
    #[test]
    fn fetch_count_key_guards() {
        let pf = parse_str(
            "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
             \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
             \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH2\",\"2.00\"\r\n",
        )
        .unwrap();
        assert_eq!(fetch_count(&pf, &["LOCA"]), 0); // no keys → not "all rows"
        assert_eq!(fetch_count(&pf, &["LOCA", "BH1"]), 1); // fewer keys than cols
        assert_eq!(fetch_count(&pf, &["NOPE", "x"]), 0); // unknown group
    }

    #[test]
    fn rule_11_silent_when_tran_has_no_data_row() {
        // TRAN group with the delimiter headings but zero DATA rows →
        // rule_11's `tran.rows.first()` None early return (no 11a/11b/c).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n";
        let r = run(src);
        assert!(!r.contains_key(RULE_11A), "{r:?}");
        assert!(!r.contains_key(RULE_11B), "{r:?}");
        assert!(!r.contains_key(RULE_11C), "{r:?}");
    }

    #[test]
    fn rule_11_silent_when_tran_lacks_delim_columns() {
        // TRAN present, has a DATA row, but no TRAN_DLIM/TRAN_RCON
        // columns → the `(Some, Some)` column-resolve None arm.
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"4.2\"\r\n";
        let r = run(src);
        assert!(!r.contains_key(RULE_11A), "{r:?}");
        assert!(!r.contains_key(RULE_11B), "{r:?}");
    }

    // ---- Rule 10c error branches (no/blank/absent parent, key mismatch) ----

    #[test]
    fn rule_10c_flags_parent_group_absent_from_file() {
        // SAMP's parent LOCA is in the standard dictionary but not in the
        // file → the "Parent group … is not in the file" arm.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "SAMP" && x.desc.contains("is not in the file")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_group_undefined_anywhere() {
        // A custom group "QZZZ" with no standard- or DICT-defined parent
        // → the `eff.parent(code) = None` "group not defined" arm. It's
        // not in PARENTLESS, so 10c runs and can't find a parent at all.
        let src = "\"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"X1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("not defined")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_uses_file_dict_to_define_a_custom_parent_chain() {
        // A file DICT defines QZZZ (parent PROJ, KEY=QZZZ_ID) and child
        // QCHD (parent QZZZ, KEY borrowing QZZZ_ID). QCHD's data row
        // references a QZZZ_ID that doesn't exist → orphan flagged via
        // the file-DICT overlay path (EffectiveDict::build with a DICT
        // group, exercising file_parent + file_hdng + fields_with_status
        // extras). PROJ is the QZZZ parent (PARENTLESS-exempt LOCA-style
        // not used).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"PROJ\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QZZZ\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"Z1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QZZZ_ID\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"Z9\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        // QCHD row references QZZZ_ID "Z9" which has no QZZZ parent (Z1).
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("Z9")),
            "file-DICT-defined parent chain must drive the orphan check: {r10c:?}"
        );
    }

    #[test]
    fn rule_10b_uses_file_dict_required_status() {
        // A file DICT marks QZZZ_REF as REQUIRED for a custom group;
        // an empty cell there → Rule 10b fires via the file_hdng overlay
        // (fields_with_status extras branch).
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_REF\",\"REQUIRED\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\",\"QZZZ_REF\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"Z1\",\"\"\r\n";
        let r = run(src);
        let r10b = r.get(RULE_10B).expect("Rule 10b");
        assert!(
            r10b.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("QZZZ_REF")),
            "file-DICT REQUIRED status must drive Rule 10b: {r10b:?}"
        );
    }

    #[test]
    fn rule_10c_flags_parent_with_no_key_fields() {
        // File-DICT: parent QPAR has NO KEY heading; child QCHD names
        // QPAR as parent. pkeys is empty → the "No KEY fields are defined
        // in the parent group" arm (lines 393-401).
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_VAL\",\"OTHER\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_VAL\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("No KEY fields")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_parent_key_not_declared_in_child() {
        // File-DICT: parent QPAR has KEY=QPAR_ID; child QCHD names QPAR
        // as parent but does NOT declare QPAR_ID as one of its KEYs. So
        // a parent KEY is "missing from the child" → lines 404-417.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter().any(|x| x.group == "QCHD"
                && x.desc.contains("QPAR_ID")
                && x.desc.contains("not in the child group")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_key_column_absent_from_heading_row() {
        // File-DICT: both QPAR and QCHD declare QPAR_ID as KEY (so the
        // effective-dict KEY sets line up and `missing` is empty), but
        // the child's actual HEADING row OMITS the QPAR_ID column → the
        // `!(in_child && in_parent)` arm (lines 420-435), reported at the
        // child's HEADING line.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("KEY fields missing")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_11c_ignores_empty_record_link_cell() {
        // An RL-typed column whose value is empty must be skipped (the
        // `rl.is_empty()` continue in rule_11c) — no Rule 11c finding.
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"\"\r\n";
        assert!(!run(src).contains_key(RULE_11C), "{:?}", run(src));
    }

    #[test]
    fn rule_11c_unknown_group_in_link_is_no_such_record() {
        // Link references group "NOPE" that isn't in the file →
        // fetch_count's `parsed.groups.get == None` returns 0 → "no such
        // record". Also covers a link whose key count exceeds the target
        // group's heading count (fetch_count's `keys.len() > headings`).
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"NOPE|x\"\r\n\"DATA\",\"BH1\",\"LOCA|a|b|c\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        // Both bad links resolve to 0 records → "no such record".
        assert!(
            r11c.iter()
                .filter(|x| x.desc.contains("no such record"))
                .count()
                >= 2,
            "{r11c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_blank_parent_in_file_dict() {
        // A file-DICT GROUP row whose DICT_PGRP is blank ("") for a
        // non-PARENTLESS custom group → eff.parent returns Some("") →
        // the "Parent group is left blank" arm.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"Z1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("left blank")),
            "{r10c:?}"
        );
    }
}
