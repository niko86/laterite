//! Dictionary-driven **depth** generator — the lab-test result groups that
//! hang *below* a sample, plus the LBSG/LBST testing schedule.
//!
//! Where [`super::breadth`] widens a file across LOCA's direct children
//! (one inherited `LOCA_ID`), depth deepens it: the ~50 laboratory-test
//! groups are SAMP's *direct children* — they inherit SAMP's whole
//! five-part KEY (`LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID`) and
//! add their own specimen KEY (`SPEC_REF`, `SPEC_DPTH`), so a clean row is
//! a copy of a real SAMP row's keys + a uniquified specimen. This is the
//! deeper relational level the Rule-10c chain (LOCA→SAMP→lab-test) and the
//! divergence-mining campaign want more group context from (#172).
//!
//! Clean-by-construction, exactly as breadth: only KEY + REQUIRED headings
//! are emitted, every value is type-correct (reusing breadth's [`value`]),
//! and a group is included only if every required type is generatable —
//! the few lab groups with a required `PU` unit-typed result (ELRG, ERES,
//! GCHM) are skipped, just as breadth skips unsafe LOCA children. The
//! `varied_baseline` clean guard backstops the set.
//!
//! Scheduling: **LBSG** is a root "Testing Schedule" (its own `LBSG_REF`
//! KEY); **LBST** is "Testing Schedule Details" — its parent is LBSG but
//! each row also carries the SAMP five-key, so it links a named test
//! (`LBST_TEST`) to a real sample. LBST therefore deepens the file the
//! same way the lab-test results do: a schedule entry per (sample, test).
//!
//! Scope note (deliberate, owner-review): LBST schedules *plausible* test
//! names, NOT a cross-reference to the lab-result groups actually emitted.
//! No AGS rule couples the schedule to the results (Rule 10c only checks
//! LBST→LBSG and the SAMP-key link), so we don't fabricate a coupling the
//! validator can't check. See the PR for the realism trade-off.

use laterite_ags4_parity::Rng;
use laterite_ags4_validator::Dictionary;

use super::breadth::{kind, value};
use super::model::{Group, Row};

/// SAMP's full KEY, in dictionary order — the keys every lab-test child
/// inherits and every clean child row must copy from a real sample. A
/// SAMP row's `values` are exactly these five, in this order (see
/// `model::boreholes`).
pub const SAMP_KEY: &[&str] = &["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"];

/// Lab-test groups whose detail rows the borehole core / breadth already
/// build, or that we deliberately leave to a later slice — kept empty for
/// now (depth owns the whole SAMP-child set).
const HAND_BUILT: &[&str] = &[];

/// Is `code` a clean-generatable child of `parent` whose inherited KEY is
/// `inherited_keys`? Every KEY/REQUIRED heading must be a safe type, any
/// required PA must have a picklist, and the group must own at least one
/// uniquifiable, non-inherited KEY heading (so its KEY tuple can be made
/// unique even when many rows share one parent).
fn child_is_safe(dict: &Dictionary<'static>, code: &str, inherited_keys: &[&str]) -> bool {
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
            return false; // a required type we can't synthesize (e.g. PU)
        }
        if e.ags_type == "PA" && dict.abbr_codes(h).is_empty() {
            return false; // a required picklist with no codes
        }
        if is_key && !inherited_keys.contains(&h) && super::breadth::uniquifiable(e.ags_type) {
            own_key_uniquifiable = true;
        }
    }
    own_key_uniquifiable
}

/// The safe generatable children of `parent_code` given the KEY they inherit,
/// sorted for deterministic file order. `HAND_BUILT` groups are excluded; a
/// child is included only if [`child_is_safe`].
pub fn safe_children_of(
    dict: &Dictionary<'static>,
    parent_code: &str,
    inherited_keys: &[&str],
) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = dict
        .group_codes()
        .filter(|&c| dict.group(c).map(|g| g.parent) == Some(parent_code))
        .filter(|&c| !HAND_BUILT.contains(&c))
        .filter(|&c| child_is_safe(dict, c, inherited_keys))
        .collect();
    v.sort_unstable();
    v
}

/// The SAMP-direct-child lab-test groups we can synthesize cleanly (the depth
/// (lab-test result) groups the borehole core doesn't build by hand).
pub fn safe_samp_children(dict: &Dictionary<'static>) -> Vec<&'static str> {
    safe_children_of(dict, "SAMP", SAMP_KEY)
}

/// The KEY/REQUIRED column schema for `code` (headings, units, types) — the
/// clean minimal shape the emitter writes, in dictionary order.
fn schema(dict: &Dictionary<'static>, code: &str) -> (Vec<&'static str>, Vec<String>, Vec<String>) {
    let cols: Vec<&'static str> = dict
        .group_headings(code)
        .iter()
        .copied()
        .filter(|&h| {
            dict.heading(code, h)
                .is_some_and(|e| e.status.contains("KEY") || e.status.contains("REQUIRED"))
        })
        .collect();
    let units = cols
        .iter()
        .map(|&h| {
            dict.heading(code, h)
                .map(|e| e.unit.to_string())
                .unwrap_or_default()
        })
        .collect();
    let types = cols
        .iter()
        .map(|&h| {
            dict.heading(code, h)
                .map(|e| e.ags_type.to_string())
                .unwrap_or_default()
        })
        .collect();
    (cols, units, types)
}

/// Generate `code` as a clean child of a parent whose KEY columns are
/// `inherited_keys` (dictionary order) and whose per-row KEY values are
/// `parent_key_rows` (parallel to `inherited_keys`). Each parent row yields at
/// most one child row: at `rate = 1.0` every parent row gets one (dense — the
/// original behaviour, and NO rng is drawn, so the byte output is unchanged);
/// at `rate < 1.0` each parent row is drawn with probability `rate` (seeded →
/// deterministic), giving a sparse test matrix. Inherited KEY values are copied
/// verbatim; the child's own KEY/REQUIRED headings are filled type-correct, its
/// own KEY uniquified by the parent-row index so the KEY tuple is unique. `None`
/// if no row was drawn — the group is then omitted rather than emitted empty.
fn generate_child(
    dict: &Dictionary<'static>,
    code: &'static str,
    inherited_keys: &[&str],
    parent_key_rows: &[Vec<String>],
    rate: f64,
    rng: &mut Rng,
) -> Option<Group> {
    let (cols, units, types) = schema(dict, code);
    let mut rows = Vec::new();
    for (gidx, keys) in parent_key_rows.iter().enumerate() {
        // Presence draw. rate >= 1.0 always includes AND draws no rng, so dense
        // output stays byte-identical to before this parameter existed.
        if rate < 1.0 && (rng.below(1000) as f64) >= rate * 1000.0 {
            continue;
        }
        let row = cols
            .iter()
            .map(|&h| {
                if let Some(pos) = inherited_keys.iter().position(|&k| k == h) {
                    return keys[pos].clone(); // inherited KEY → copy the parent's
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
    if rows.is_empty() {
        return None;
    }
    Some(Group {
        code: code.to_string(),
        headings: cols.iter().map(std::string::ToString::to_string).collect(),
        units,
        types,
        rows,
    })
}

/// A generated group's KEY columns + per-row KEY values — the parent view a
/// deeper (3rd-level) child inherits (the SAMP five-key + this group's own
/// specimen KEY).
fn key_view(
    dict: &Dictionary<'static>,
    code: &str,
    group: &Group,
) -> (Vec<&'static str>, Vec<Vec<String>>) {
    let key_headings: Vec<&'static str> = dict
        .group_headings(code)
        .iter()
        .copied()
        .filter(|&h| {
            dict.heading(code, h)
                .is_some_and(|e| e.status.contains("KEY"))
        })
        .collect();
    let positions: Vec<usize> = key_headings
        .iter()
        .map(|&kh| {
            group
                .headings
                .iter()
                .position(|gh| gh == kh)
                .expect("KEY column is emitted (KEY ⊂ the KEY+REQUIRED schema)")
        })
        .collect();
    let key_rows = group
        .rows
        .iter()
        .map(|r| positions.iter().map(|&p| r.values[p].clone()).collect())
        .collect();
    (key_headings, key_rows)
}

/// The full lab-test depth below the file's samples: each safe SAMP-child (the
/// 2nd relational level) and then any safe children of THOSE (the 3rd level —
/// e.g. `TREG → TRET`, `GRAG → GRAT`, `CONG → CONS`), every row carrying its
/// parent's whole KEY so the `LOCA → SAMP → test → spec` chain joins. `rate` is
/// the per-parent-row presence probability (1.0 = dense = every sample gets
/// every test); groups that draw no rows are omitted. Deterministic for a given
/// (seed, rate); a 2nd-level group is emitted immediately before its 3rd-level
/// children.
pub fn generate_lab_depth(
    dict: &Dictionary<'static>,
    samp_keys: &[Vec<String>],
    rate: f64,
    rng: &mut Rng,
) -> Vec<Group> {
    let mut out = Vec::new();
    for code in safe_samp_children(dict) {
        let Some(g) = generate_child(dict, code, SAMP_KEY, samp_keys, rate, rng) else {
            continue;
        };
        // 3rd level: this lab-test group's own safe children inherit its full KEY.
        let (kh, kr) = key_view(dict, code, &g);
        let grandkids = safe_children_of(dict, code, &kh);
        out.push(g);
        for gc in grandkids {
            if let Some(gg) = generate_child(dict, gc, &kh, &kr, rate, rng) {
                out.push(gg);
            }
        }
    }
    out
}

/// A short, plausible synthetic test name (the `LBST_TEST` KEY) — a
/// composed abbreviation so the variety is combinatorial, not a fixed
/// list, while staying a valid free-text `X` value.
const TEST_STEMS: &[&str] = &[
    "MC", "ATT", "PSD", "BD", "PD", "TXU", "TXCU", "CONS", "SHB", "CBR", "UCS", "PERM", "pH", "SO4",
];

/// The LBSG/LBST testing schedule for the file's samples.
///
/// LBSG is a single root schedule row (its `LBSG_REF` KEY); LBST schedules
/// 1–2 named tests against each sample, every row carrying that schedule
/// ref + the sample's five-key + a unique `LBST_TEST`. Both are
/// clean-by-construction (KEY tuples unique, parent links real).
pub fn schedule(
    dict: &Dictionary<'static>,
    samp_keys: &[Vec<String>],
    rng: &mut Rng,
) -> (Group, Group) {
    let schedule_ref = format!("SCH{:04}", rng.range(1, 9999));

    let lbsg = Group {
        code: "LBSG".to_string(),
        headings: vec!["LBSG_REF".to_string()],
        units: vec![String::new()],
        types: vec!["X".to_string()],
        rows: vec![Row::owned(vec![schedule_ref.clone()])],
    };

    // LBST's KEY = SAMP five-key + LBSG_REF + LBST_TEST. We emit exactly
    // those KEY columns (all REQUIRED-or-KEY) so the row is clean and the
    // (sample, schedule, test) KEY tuple is unique by construction: a
    // given sample never schedules the same test twice.
    let (cols, units, types) = schema(dict, "LBST");
    let mut rows = Vec::new();
    for keys in samp_keys {
        // `range(1, 2)` returns 1 or 2; `below(n)` returns a value < n by
        // construction, and n is `TEST_STEMS.len()` widened to u64 — both
        // narrow back to usize losslessly.
        #[allow(clippy::cast_possible_truncation)]
        let n_tests = rng.range(1, 2) as usize;
        // Distinct tests per sample → unique (…, LBST_TEST) tuples.
        #[allow(clippy::cast_possible_truncation)]
        let start = rng.below(TEST_STEMS.len() as u64) as usize;
        for t in 0..n_tests {
            let test = TEST_STEMS[(start + t) % TEST_STEMS.len()];
            let row = cols
                .iter()
                .map(|&h| {
                    if let Some(pos) = SAMP_KEY.iter().position(|&k| k == h) {
                        return keys[pos].clone();
                    }
                    match h {
                        "LBSG_REF" => schedule_ref.clone(),
                        "LBST_TEST" => test.to_string(),
                        // Any other KEY/REQUIRED (none in the standard LBST,
                        // but stay robust to edition drift): type-correct.
                        _ => {
                            let e = dict.heading("LBST", h).expect("heading exists");
                            value(rng, e.ags_type, dict, h, 0, false)
                        }
                    }
                })
                .collect();
            rows.push(Row::owned(row));
        }
    }
    let lbst = Group {
        code: "LBST".to_string(),
        headings: cols.iter().map(std::string::ToString::to_string).collect(),
        units,
        types,
        rows,
    };
    (lbsg, lbst)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use laterite_ags4_validator::{DictVersion, Dictionary};

    use super::*;
    use crate::synth::Scaffold;
    use crate::synth::model::{Group, ProjectModel, varied_model, varied_model_lab};

    fn group<'a>(m: &'a ProjectModel, code: &str) -> Option<&'a Group> {
        m.groups.iter().find(|g| g.code == code)
    }

    fn samp_keys(m: &ProjectModel) -> HashSet<Vec<String>> {
        group(m, "SAMP")
            .unwrap()
            .rows
            .iter()
            .map(|r| r.values.clone())
            .collect()
    }

    /// The safe SAMP-child set is the real lab-test groups, sorted and
    /// deduped, and excludes the three with a required `PU`-typed result
    /// (ELRG/ERES/GCHM) we can't synthesize cleanly — exactly the breadth
    /// safety filter, applied to SAMP's children.
    #[test]
    fn safe_samp_children_are_the_clean_lab_test_groups() {
        let dict = Dictionary::bundled(DictVersion::V4_2);
        let kids = safe_samp_children(&dict);
        assert!(
            kids.len() > 40,
            "expected the full lab-test set, got {}",
            kids.len()
        );
        let set: HashSet<_> = kids.iter().collect();
        assert_eq!(set.len(), kids.len(), "no duplicate group code");
        assert!(
            kids.windows(2).all(|w| w[0] < w[1]),
            "sorted for deterministic order"
        );
        // Representative results are present…
        for present in ["LNMC", "LLPL", "GRAG", "LDEN", "TRIG"] {
            assert!(kids.contains(&present), "{present} should be generated");
        }
        // …and the PU-result groups are filtered out (unsafe type).
        for skipped in ["ELRG", "ERES", "GCHM"] {
            assert!(!kids.contains(&skipped), "{skipped} has a PU result → skip");
        }
        // Every one really is a SAMP child.
        for &c in &kids {
            assert_eq!(dict.group(c).map(|g| g.parent), Some("SAMP"));
        }
    }

    /// Every lab-test (SAMP-child) row links to a REAL sample — the five
    /// inherited KEY cells are copied verbatim from a SAMP row — and the
    /// group's own KEY tuple is unique (no row repeats), so Rule 10a/10c
    /// hold by construction. Across seeds, so it's the varied base.
    #[test]
    fn lab_test_rows_link_real_samples_with_unique_keys() {
        for seed in 0..20u64 {
            let m = varied_model(Scaffold::Wide, seed);
            let keys = samp_keys(&m);
            let dict = Dictionary::bundled(DictVersion::V4_2);
            for code in safe_samp_children(&dict) {
                let g = group(&m, code).unwrap_or_else(|| panic!("seed {seed}: {code} present"));
                // Inherited SAMP five-key column positions in this group.
                let kcols: Vec<usize> = SAMP_KEY
                    .iter()
                    .map(|&k| g.col(k).unwrap_or_else(|| panic!("{code} carries {k}")))
                    .collect();
                let mut seen = HashSet::new();
                for r in &g.rows {
                    let parent: Vec<String> = kcols.iter().map(|&i| r.values[i].clone()).collect();
                    assert!(
                        keys.contains(&parent),
                        "seed {seed}: {code} row links a missing sample {parent:?}"
                    );
                    // Full KEY tuple unique within the group (Rule 10a).
                    let key: Vec<String> = g
                        .headings
                        .iter()
                        .enumerate()
                        .filter(|(_, h)| {
                            dict.heading(code, h)
                                .is_some_and(|e| e.status.contains("KEY"))
                        })
                        .map(|(i, _)| r.values[i].clone())
                        .collect();
                    assert!(seen.insert(key), "seed {seed}: {code} duplicate KEY tuple");
                }
            }
        }
    }

    /// The LBSG/LBST schedule is coherent: one LBSG schedule whose `LBSG_REF`
    /// every LBST row references, every LBST row links a real sample, and
    /// the (sample, schedule, test) KEY tuple is unique (a sample never
    /// schedules the same test twice).
    #[test]
    fn schedule_is_coherent_across_seeds() {
        for seed in 0..20u64 {
            let m = varied_model(Scaffold::Wide, seed);
            let keys = samp_keys(&m);
            let lbsg = group(&m, "LBSG").unwrap_or_else(|| panic!("seed {seed}: LBSG present"));
            let lbst = group(&m, "LBST").unwrap_or_else(|| panic!("seed {seed}: LBST present"));
            assert_eq!(lbsg.rows.len(), 1, "seed {seed}: one schedule row");
            let sched_ref = &lbsg.rows[0].values[group_col(lbsg, "LBSG_REF")];
            assert!(!sched_ref.is_empty(), "seed {seed}: LBSG_REF non-empty");
            assert!(!lbst.rows.is_empty(), "seed {seed}: LBST has entries");

            let kcols: Vec<usize> = SAMP_KEY.iter().map(|&k| group_col(lbst, k)).collect();
            let ref_col = group_col(lbst, "LBSG_REF");
            let mut seen = HashSet::new();
            for r in &lbst.rows {
                assert_eq!(
                    &r.values[ref_col], sched_ref,
                    "seed {seed}: LBST refs the schedule"
                );
                let parent: Vec<String> = kcols.iter().map(|&i| r.values[i].clone()).collect();
                assert!(
                    keys.contains(&parent),
                    "seed {seed}: LBST links a missing sample"
                );
                let key: Vec<String> = (0..lbst.headings.len())
                    .map(|i| r.values[i].clone())
                    .collect();
                assert!(seen.insert(key), "seed {seed}: LBST duplicate KEY tuple");
            }
        }
    }

    fn group_col(g: &Group, h: &str) -> usize {
        g.col(h).unwrap_or_else(|| panic!("{} carries {h}", g.code))
    }

    /// Depth genuinely deepens the file: Wide carries the lab-test groups
    /// + LBSG/LBST that the narrower scaffolds don't.
    #[test]
    fn wide_carries_depth_groups() {
        let m = varied_model(Scaffold::Wide, 5);
        for d in ["LNMC", "LLPL", "GRAG", "LBSG", "LBST"] {
            assert!(group(&m, d).is_some(), "Wide should carry {d}");
        }
        // LocaSamp does not.
        let ls = varied_model(Scaffold::LocaSamp, 5);
        for d in ["LNMC", "LBSG", "LBST"] {
            assert!(group(&ls, d).is_none(), "LocaSamp must not carry {d}");
        }
    }

    /// The 3rd relational level: a lab-test group's own safe children (e.g.
    /// TREG→TRET, CONG→CONS) are generated, each row inheriting its parent's
    /// full KEY — so LOCA→SAMP→test→spec joins, the deepest chain the corpus
    /// never reaches. The exact grandchild set is edition-driven; assert the
    /// mechanism via any generated SAMP-grandchild + its five-key link.
    #[test]
    fn depth_reaches_the_third_relational_level() {
        let dict = Dictionary::bundled(DictVersion::V4_2);
        let m = varied_model(Scaffold::Wide, 1);
        // A group whose parent's parent is SAMP (a lab-test grandchild).
        let g = m
            .groups
            .iter()
            .find(|g| {
                dict.group(&g.code)
                    .map(|gg| gg.parent)
                    .and_then(|p| dict.group(p))
                    .map(|pp| pp.parent)
                    == Some("SAMP")
            })
            .expect("Wide generates at least one SAMP-grandchild (3rd level)");
        // Its 2nd-level parent is present too — the chain is complete.
        let parent = dict.group(&g.code).map(|gg| gg.parent).unwrap();
        assert!(
            group(&m, parent).is_some(),
            "3rd-level {}'s parent {parent} must be present",
            g.code
        );
        // Every 3rd-level row links a real sample (carries the SAMP five-key).
        let keys = samp_keys(&m);
        let kcols: Vec<usize> = SAMP_KEY.iter().map(|&k| group_col(g, k)).collect();
        for r in &g.rows {
            let samp: Vec<String> = kcols.iter().map(|&i| r.values[i].clone()).collect();
            assert!(
                keys.contains(&samp),
                "{} 3rd-level row links a missing sample",
                g.code
            );
        }
    }

    /// `--lab-test-rate` sparsifies the per-sample test matrix; the default
    /// (1.0) stays dense (every sample every test). Sparse rows still link real
    /// samples — sparsity never breaks the KEY chain.
    #[test]
    fn lab_test_rate_sparsifies_but_dense_is_full() {
        let dense = varied_model(Scaffold::Wide, 1);
        let n_samp = group(&dense, "SAMP").unwrap().rows.len();
        assert_eq!(
            group(&dense, "LLPL").unwrap().rows.len(),
            n_samp,
            "dense = one LLPL per sample"
        );
        // Sparse (seed 1, rate 0.4 → deterministic): fewer rows than samples.
        let sparse = varied_model_lab(Scaffold::Wide, 1, 0.4);
        let llpl = group(&sparse, "LLPL").map_or(0, |g| g.rows.len());
        assert!(llpl < n_samp, "sparse LLPL ({llpl}) < samples ({n_samp})");
        let keys = samp_keys(&sparse);
        if let Some(g) = group(&sparse, "LLPL") {
            let kcols: Vec<usize> = SAMP_KEY.iter().map(|&k| group_col(g, k)).collect();
            for r in &g.rows {
                let samp: Vec<String> = kcols.iter().map(|&i| r.values[i].clone()).collect();
                assert!(keys.contains(&samp), "sparse LLPL links a missing sample");
            }
        }
    }
}
