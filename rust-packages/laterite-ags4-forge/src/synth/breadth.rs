//! Dictionary-driven **generic** group generator — clean-by-construction
//! rows for arbitrary AGS groups, so the forge can synthesize *wide* files
//! (toward a real ~69-group delivery) instead of only the hand-built
//! borehole core.
//!
//! v1 covers **LOCA's direct children** (they inherit just `LOCA_ID`, so
//! linking is a single copy from a chosen LOCA row). Only KEY + REQUIRED
//! headings are emitted (OTHER omitted, as everywhere in the synth), with
//! type-correct values so the output stays `RustResult::Clean`. A group is
//! included only if every KEY/REQUIRED heading has a type we can generate
//! safely and at least one *own* KEY heading is uniquifiable — otherwise
//! it's skipped. [`safe_loca_children`] computes the set from the live
//! dictionary; the `varied_baseline` clean guard backstops it.
//!
//! Realism note: values are plausible-but-synthetic (a depth, a count, a
//! picklist code, an id stem) — the geotechnical *prose* realism lives in
//! GEOL via [`super::bs5930`]; breadth is about group *variety* for the
//! validator/perf surface, not per-field realism.

use laterite_ags4_parity::Rng;
use laterite_ags4_validator::Dictionary;

use super::generic;
use super::model::{Group, Row};

/// The base AGS type kind we can generate a value for. `None` ⇒ a type we
/// don't synthesize safely (RL record-links, U/PU units, DMS, nSF/nSCI, …)
/// → the owning group is skipped.
pub(super) enum Kind {
    Text,
    Decimal(usize),
    Integer,
    Date,
    YesNo,
    Picklist,
}

/// Classify an AGS type code. Conservative: only the codes a clean value is
/// trivially generatable for.
pub(super) fn kind(ags_type: &str) -> Option<Kind> {
    match ags_type {
        "ID" | "X" | "PT" | "MC" | "XN" => Some(Kind::Text),
        "0DP" => Some(Kind::Integer),
        "DT" => Some(Kind::Date),
        "YN" => Some(Kind::YesNo),
        "PA" => Some(Kind::Picklist),
        t => t
            .strip_suffix("DP")
            .and_then(|n| n.parse::<usize>().ok())
            .map(Kind::Decimal),
    }
}

/// A type that can carry a unique, monotonic per-row value (so it can
/// discriminate the KEY tuple). PA/YN/DT can't (few/colliding values).
pub(super) fn uniquifiable(ags_type: &str) -> bool {
    matches!(
        kind(ags_type),
        Some(Kind::Text | Kind::Integer | Kind::Decimal(_))
    )
}

/// The last `_`-separated part of a heading (`ISPT_REP` → `REP`) — a
/// readable stem for generated text/id values.
fn stem(heading: &str) -> &str {
    heading.rsplit('_').next().unwrap_or(heading)
}

/// Generate a type-correct AGS4 field string. KEY values are made unique by
/// `gidx` (the row's global index in the group) so the KEY tuple never
/// repeats; REQUIRED non-key values are plausible draws.
pub(super) fn value(
    rng: &mut Rng,
    ags_type: &str,
    dict: &Dictionary<'static>,
    heading: &str,
    gidx: u64,
    key: bool,
) -> String {
    match kind(ags_type) {
        Some(Kind::Text) => {
            if key {
                format!("{}{gidx}", stem(heading))
            } else {
                format!("{}{}", stem(heading), rng.range(1, 9999))
            }
        }
        Some(Kind::Integer) => {
            if key {
                gidx.to_string()
            } else {
                rng.range(0, 1000).to_string()
            }
        }
        Some(Kind::Decimal(prec)) => {
            let v = if key {
                gidx as f64 * 0.13 + 0.01
            } else {
                rng.range(0, 10_000) as f64 / 100.0
            };
            format!("{v:.prec$}")
        }
        Some(Kind::Date) => generic::iso_date(rng),
        Some(Kind::YesNo) => {
            if rng.below(2) == 0 {
                "Y".into()
            } else {
                "N".into()
            }
        }
        Some(Kind::Picklist) => {
            // Non-empty by the safety filter; the chosen code lands in ABBR
            // when the file's ABBR group is scanned from the PA cells.
            let codes = dict.abbr_codes(heading);
            (*rng.choose(&codes)).to_string()
        }
        None => String::new(),
    }
}

/// Is `code` a LOCA-child group we can generate cleanly? Every KEY/REQUIRED
/// heading must be a safe type; any PA among them must have a picklist; and
/// the group's *own* (non-inherited) KEY must include a uniquifiable
/// heading so the KEY tuple can be made unique.
fn group_is_safe(dict: &Dictionary<'static>, code: &str) -> bool {
    let headings = dict.group_headings(code);
    if headings.is_empty() {
        return false;
    }
    let mut own_key_uniquifiable = false;
    for &h in headings.iter() {
        let Some(e) = dict.heading(code, h) else {
            return false;
        };
        let is_key = e.status.contains("KEY");
        let is_req = e.status.contains("REQUIRED");
        if !(is_key || is_req) {
            continue; // OTHER headings are omitted, never generated
        }
        if kind(e.ags_type).is_none() {
            return false; // a required type we can't synthesize
        }
        if e.ags_type == "PA" && dict.abbr_codes(h).is_empty() {
            return false; // a required picklist with no codes
        }
        // "own" KEY = not the inherited LOCA_ID.
        if is_key && h != "LOCA_ID" && uniquifiable(e.ags_type) {
            own_key_uniquifiable = true;
        }
    }
    own_key_uniquifiable
}

/// LOCA-child groups the borehole core already builds by hand — excluded
/// from breadth so a group is never emitted twice.
const HAND_BUILT: &[&str] = &["SAMP", "GEOL"];

/// The LOCA-direct-child groups the breadth generator can synthesize
/// cleanly, sorted for deterministic file order. Excludes the hand-built
/// borehole groups.
pub fn safe_loca_children(dict: &Dictionary<'static>) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = dict
        .group_codes()
        .filter(|&c| dict.group(c).map(|g| g.parent) == Some("LOCA"))
        .filter(|&c| !HAND_BUILT.contains(&c))
        .filter(|&c| group_is_safe(dict, c))
        .collect();
    v.sort_unstable();
    v
}

/// Generate group `code` as a clean child of LOCA: one row per LOCA id,
/// each copying its parent `LOCA_ID` and filling its own KEY/REQUIRED
/// headings with type-correct values (KEY values uniquified by row index).
pub fn generate(
    dict: &Dictionary<'static>,
    code: &'static str,
    loca_ids: &[String],
    rng: &mut Rng,
) -> Group {
    let headings = dict.group_headings(code);
    // Emit only KEY + REQUIRED columns (the clean minimal shape).
    let cols: Vec<&'static str> = headings
        .iter()
        .copied()
        .filter(|&h| {
            dict.heading(code, h)
                .is_some_and(|e| e.status.contains("KEY") || e.status.contains("REQUIRED"))
        })
        .collect();
    let units: Vec<String> = cols
        .iter()
        .map(|&h| {
            dict.heading(code, h)
                .map(|e| e.unit.to_string())
                .unwrap_or_default()
        })
        .collect();
    let types: Vec<String> = cols
        .iter()
        .map(|&h| {
            dict.heading(code, h)
                .map(|e| e.ags_type.to_string())
                .unwrap_or_default()
        })
        .collect();

    let mut rows = Vec::new();
    for (gidx, loca_id) in loca_ids.iter().enumerate() {
        let row = cols
            .iter()
            .map(|&h| {
                if h == "LOCA_ID" {
                    return loca_id.clone();
                }
                let e = dict.heading(code, h).expect("heading exists");
                value(
                    rng,
                    e.ags_type,
                    dict,
                    h,
                    gidx as u64,
                    e.status.contains("KEY"),
                )
            })
            .collect();
        rows.push(Row::owned(row));
    }

    Group {
        code: code.to_string(),
        headings: cols.iter().map(std::string::ToString::to_string).collect(),
        units,
        types,
        rows,
    }
}
