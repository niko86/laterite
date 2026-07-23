//! Mutation operators — the rule **injectors**, single and combined.
//!
//! A single injector synthesizes a clean base and injects exactly one
//! rule's violation, so the defect is isolable — the honesty principle the
//! parity-matrix enforces. Injection is a typed-model mutation
//! ([`Injection::apply_model`]): find the target group/column by name and
//! edit a row (or attach a [`RowFault`]), never a byte-substring rewrite —
//! so it survives the realistic varied values and streams at any scale.
//!
//! [`synth_combined`] applies *several* injectors to one base — a
//! combination candidate. Because each injector picks its site from the
//! shared placement RNG against the model's **current** state, the faults
//! interact for real (one can mask or cascade into another). So a
//! combination's true rule-set is **not** assumed to be the union of the
//! individual `target_rule`s — the caller must *validate the emitted file*
//! to read what actually tripped. That honesty is what lets the corpus-gap
//! miner trust a synthesized combination's signature.

use std::fmt;

use laterite_ags4_parity::Rng;

use crate::synth::Scaffold;
use crate::synth::emit::emit_to_string;
use crate::synth::model::{DENSE_LAB, Group, ProjectModel, Row, RowFault, varied_model_lab};

/// Decorrelates the placement RNG from the model-generation RNG (both are
/// seeded from `seed`), so *where* a fault lands isn't tied to *which*
/// values were generated. (`SplitMix64`'s golden-ratio increment.)
const PLACEMENT_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

/// Synthesize `scaffold`'s realistic base (seeded by `seed`), apply
/// `injections` to the **typed model** in order at seeded sites, and emit.
/// Injectors mutate the model (pick an applicable group/row/column, edit
/// it or attach a [`RowFault`]) rather than `replacen`-ing the emitted
/// string — so they survive the varied values, land in *varied places*,
/// and stream at any scale.
///
/// All injectors share **one** placement RNG, drawn against the model's
/// running state, so a later injector sees the earlier ones' edits (faults
/// genuinely interact). Deterministic: same `(seed, injection order)` →
/// same bytes. The result's *actual* rule-set comes from validating the
/// emitted file, not from the injectors' declared `target_rule`s.
pub fn synth_combined(scaffold: Scaffold, seed: u64, injections: &[Injection]) -> String {
    synth_combined_lab(scaffold, seed, injections, DENSE_LAB)
}

/// [`synth_combined`] with a `lab_rate` (`Wide` only): the per-sample probability
/// each lab-test result is present (`1.0` = dense = every sample every test;
/// `< 1.0` = a realistic sparse test matrix). `gen`'s `--lab-test-rate` path.
pub fn synth_combined_lab(
    scaffold: Scaffold,
    seed: u64,
    injections: &[Injection],
    lab_rate: f64,
) -> String {
    let mut model = varied_model_lab(scaffold, seed, lab_rate);
    let mut rng = Rng::seeded(seed ^ PLACEMENT_SALT);
    for inj in injections {
        inj.apply_model(&mut model, &mut rng);
    }
    emit_to_string(&model)
}

/// Single-injector convenience over [`synth_combined`] — byte-identical to
/// the one-element combination (same base, same single RNG draw).
pub fn synth_injected(scaffold: Scaffold, seed: u64, injection: Injection) -> String {
    synth_combined(scaffold, seed, std::slice::from_ref(&injection))
}

/// [`synth_injected`] + a `lab_rate` (see [`synth_combined_lab`]).
pub fn synth_injected_lab(
    scaffold: Scaffold,
    seed: u64,
    injection: Injection,
    lab_rate: f64,
) -> String {
    synth_combined_lab(scaffold, seed, std::slice::from_ref(&injection), lab_rate)
}

/// A targeted, single-rule violation injected into a clean synthetic
/// base. Each variant names the AGS4 rule it is *designed* to trip;
/// whether Rust and python agree on it is exactly what forge measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Injection {
    /// No-op — emit the clean base (dual-validate the baseline itself).
    None,
    /// Duplicate the SAMP DATA row → identical KEY tuple → Rule 10a.
    DupSampKeyTuple,
    /// Drop the SAMP `LOCA_ID` value's parent (rename LOCA's id) so the
    /// SAMP row is orphaned → Rule 10c.
    OrphanSampRow,
    /// Non-date in the DT-typed `TRAN_DATE` → Rule 8.
    BadDtValue,
    /// Unquote a DATA field → Rule 5 (Rust) / cascade (python).
    UnquotedField,
    /// Append a throwaway 5-letter GROUP → Rule 19.
    FiveLetterGroup,
    /// Remove the PROJ DATA row → Rule 13 (symmetric Rule 2).
    DropProjData,
    /// Remove the TRAN DATA row → Rule 14 (the mandatory-group twin of
    /// `DropProjData`/Rule 13; clearing rather than duplicating keeps it
    /// single-rule — a duplicate row also trips Rule 10a on TRAN's KEY).
    DropTranData,
    /// Put an undefined code in a PA-typed cell (not in the file's ABBR
    /// group) → Rule 16.
    UndefinedAbbrev,
    /// Point a heading's TYPE at a code the file's TYPE group doesn't
    /// define → Rule 17.
    UndefinedType,
}

impl Injection {
    /// Every injector (the no-op baseline excluded). The catalog and the
    /// validator-regression test iterate this, so a new variant can't be
    /// silently forgotten in either.
    pub const ALL: &'static [Injection] = &[
        Injection::DupSampKeyTuple,
        Injection::OrphanSampRow,
        Injection::BadDtValue,
        Injection::UnquotedField,
        Injection::FiveLetterGroup,
        Injection::DropProjData,
        Injection::DropTranData,
        Injection::UndefinedAbbrev,
        Injection::UndefinedType,
    ];
}

impl Injection {
    /// Parse a CLI `--inject` token.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "none" => Injection::None,
            "rule10a" | "dup-key" => Injection::DupSampKeyTuple,
            "rule10c" | "orphan" => Injection::OrphanSampRow,
            "rule8" | "bad-dt" => Injection::BadDtValue,
            "rule5" | "unquoted" => Injection::UnquotedField,
            "rule19" | "five-letter-group" => Injection::FiveLetterGroup,
            "rule13" | "drop-proj-data" => Injection::DropProjData,
            "rule14" | "drop-tran-data" => Injection::DropTranData,
            "rule16" | "undefined-abbrev" => Injection::UndefinedAbbrev,
            "rule17" | "undefined-type" => Injection::UndefinedType,
            _ => return None,
        })
    }

    /// The canonical short token (`rule10a`, `none`, …) — the `--inject`
    /// spelling, and the building block of a combination label
    /// (`rule10a+rule8`). The `Display` head before `:`.
    pub fn token(self) -> &'static str {
        match self {
            Injection::None => "none",
            Injection::DupSampKeyTuple => "rule10a",
            Injection::OrphanSampRow => "rule10c",
            Injection::BadDtValue => "rule8",
            Injection::UnquotedField => "rule5",
            Injection::FiveLetterGroup => "rule19",
            Injection::DropProjData => "rule13",
            Injection::DropTranData => "rule14",
            Injection::UndefinedAbbrev => "rule16",
            Injection::UndefinedType => "rule17",
        }
    }

    /// One-line description of the mutation (for `forge catalog`).
    pub fn description(self) -> &'static str {
        match self {
            Injection::None => "no-op — emit the clean base (dual-validate the baseline)",
            Injection::DupSampKeyTuple => "duplicate a SAMP DATA row → identical KEY tuple",
            Injection::OrphanSampRow => "point a SAMP row at a LOCA that doesn't exist",
            Injection::BadDtValue => "write a non-date into a DT-typed cell",
            Injection::UnquotedField => "emit a DATA field without its surrounding quotes",
            Injection::FiveLetterGroup => "append a throwaway 5-letter GROUP",
            Injection::DropProjData => "remove PROJ's DATA row (mandatory group empty)",
            Injection::DropTranData => "remove TRAN's DATA row (mandatory group empty)",
            Injection::UndefinedAbbrev => {
                "put an undefined code in a PA-typed cell (absent from ABBR)"
            }
            Injection::UndefinedType => {
                "point a heading's TYPE at a code absent from the TYPE group"
            }
        }
    }

    /// The rule label this injector targets (for the report / strategy
    /// traceability). `None` for the clean baseline.
    pub fn target_rule(self) -> Option<&'static str> {
        Some(match self {
            Injection::None => return None,
            Injection::DupSampKeyTuple => "AGS Format Rule 10a",
            Injection::OrphanSampRow => "AGS Format Rule 10c",
            Injection::BadDtValue => "AGS Format Rule 8",
            Injection::UnquotedField => "AGS Format Rule 5",
            Injection::FiveLetterGroup => "AGS Format Rule 19",
            Injection::DropProjData => "AGS Format Rule 13",
            Injection::DropTranData => "AGS Format Rule 14",
            Injection::UndefinedAbbrev => "AGS Format Rule 16",
            Injection::UndefinedType => "AGS Format Rule 17",
        })
    }

    /// Whether this injector needs the LOCA→SAMP scaffold. The relational
    /// injectors need a parent/child pair; `UndefinedAbbrev` needs a
    /// PA-typed cell, which only the boreholes' `LOCA_TYPE/SAMP_TYPE` carry.
    pub fn needs_relational(self) -> bool {
        matches!(
            self,
            Injection::DupSampKeyTuple | Injection::OrphanSampRow | Injection::UndefinedAbbrev
        )
    }

    /// Apply the injection by **mutating the typed model** at a seeded
    /// *applicable site* drawn from `rng` — pick a group/row/column the
    /// fault can legally land in, then edit it (or attach a [`RowFault`]).
    /// Placement is part of the search space (the same rule break in a
    /// different row/group can flip validator behaviour — cascades, rule
    /// context). A no-op if the scaffold offers no applicable site (a
    /// relational injector on `Minimal` is rejected upstream by
    /// [`Injection::needs_relational`]).
    // Every `rng.below(x.len() as u64) as usize` below is safe by
    // construction: `below(n)` returns a value < n (`next_u64() % n`), and n
    // is always some `Vec::len()` widened to u64, so the result narrows back
    // to usize losslessly.
    #[allow(clippy::cast_possible_truncation)]
    pub fn apply_model(self, model: &mut ProjectModel, rng: &mut Rng) {
        match self {
            Injection::None => {}
            // Duplicate *a* SAMP DATA row → a second identical KEY tuple
            // (LOCA_ID,SAMP_TOP,SAMP_REF,SAMP_TYPE,SAMP_ID) → Rule 10a.
            Injection::DupSampKeyTuple => {
                if let Some(samp) = model.group_mut("SAMP") {
                    if !samp.rows.is_empty() {
                        let i = rng.below(samp.rows.len() as u64) as usize;
                        samp.rows.push(samp.rows[i].clone());
                    }
                }
            }
            // Point *a* SAMP row at a LOCA that doesn't exist → that child
            // has no parent record (Rule 10c orphan). SAMP's own KEY stays
            // unique (the bogus id is unused elsewhere).
            Injection::OrphanSampRow => {
                if let Some(samp) = model.group_mut("SAMP") {
                    if let Some(col) = samp.col("LOCA_ID") {
                        if !samp.rows.is_empty() {
                            let i = rng.below(samp.rows.len() as u64) as usize;
                            samp.rows[i].values[col] = "ZZ_ORPHAN".to_string();
                        }
                    }
                }
            }
            // Non-date into *some* DT-typed cell (any group/row) → Rule 8.
            Injection::BadDtValue => {
                let sites: Vec<(usize, usize)> = model
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| !g.rows.is_empty())
                    .flat_map(|(gi, g)| {
                        g.types
                            .iter()
                            .enumerate()
                            .filter(|(_, t)| t.as_str() == "DT")
                            .map(move |(ci, _)| (gi, ci))
                    })
                    .collect();
                if !sites.is_empty() {
                    let (gi, ci) = *rng.choose(&sites);
                    let g = &mut model.groups[gi];
                    let ri = rng.below(g.rows.len() as u64) as usize;
                    g.rows[ri].values[ci] = "not-a-date".to_string();
                }
            }
            // Emit *some* non-empty DATA cell (any group/row/column)
            // without quotes → Rule 5.
            Injection::UnquotedField => {
                let sites: Vec<(usize, usize, usize)> = model
                    .groups
                    .iter()
                    .enumerate()
                    .flat_map(|(gi, g)| {
                        g.rows.iter().enumerate().flat_map(move |(ri, row)| {
                            row.values
                                .iter()
                                .enumerate()
                                .filter(|(_, v)| !v.is_empty())
                                .map(move |(ci, _)| (gi, ri, ci))
                        })
                    })
                    .collect();
                if !sites.is_empty() {
                    let (gi, ri, ci) = *rng.choose(&sites);
                    model.groups[gi].rows[ri].faults.push(RowFault::Unquote(ci));
                }
            }
            // Append a throwaway 5-letter GROUP → Rule 19.
            Injection::FiveLetterGroup => {
                model.groups.push(Group {
                    code: "ABCDE".to_string(),
                    headings: vec!["ABCDE_ID".to_string()],
                    units: vec![String::new()],
                    types: vec!["ID".to_string()],
                    rows: vec![Row {
                        values: vec!["x".to_string()],
                        faults: Vec::new(),
                    }],
                });
            }
            // Remove PROJ's DATA rows → Rule 13 (symmetric Rule 2).
            Injection::DropProjData => {
                if let Some(proj) = model.group_mut("PROJ") {
                    proj.rows.clear();
                }
            }
            // Remove TRAN's DATA rows → the mandatory group is empty →
            // Rule 14 (placement-less; the empty-group twin of Rule 13).
            Injection::DropTranData => {
                if let Some(tran) = model.group_mut("TRAN") {
                    tran.rows.clear();
                }
            }
            // Put an undefined code into *some* PA-typed cell (any
            // group/row). The file's ABBR group lists only the codes the
            // base actually uses, so a sentinel code is absent from it →
            // Rule 16.
            Injection::UndefinedAbbrev => {
                let sites: Vec<(usize, usize, usize)> = model
                    .groups
                    .iter()
                    .enumerate()
                    .flat_map(|(gi, g)| {
                        let pa: Vec<usize> = g
                            .types
                            .iter()
                            .enumerate()
                            .filter(|(_, t)| t.as_str() == "PA")
                            .map(|(ci, _)| ci)
                            .collect();
                        g.rows.iter().enumerate().flat_map(move |(ri, _)| {
                            pa.clone().into_iter().map(move |ci| (gi, ri, ci))
                        })
                    })
                    .collect();
                if !sites.is_empty() {
                    let (gi, ri, ci) = *rng.choose(&sites);
                    model.groups[gi].rows[ri].values[ci] = "ZZ".to_string();
                }
            }
            // Repoint *some* heading's TYPE at a code the TYPE group never
            // defines → Rule 17 (any group/column).
            Injection::UndefinedType => {
                let sites: Vec<(usize, usize)> = model
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| !g.rows.is_empty())
                    .flat_map(|(gi, g)| (0..g.types.len()).map(move |ci| (gi, ci)))
                    .collect();
                if !sites.is_empty() {
                    let (gi, ci) = *rng.choose(&sites);
                    model.groups[gi].types[ci] = "ZZ".to_string();
                }
            }
        }
    }
}

impl fmt::Display for Injection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Injection::None => "none",
            Injection::DupSampKeyTuple => "rule10a:dup-samp-key",
            Injection::OrphanSampRow => "rule10c:orphan-samp",
            Injection::BadDtValue => "rule8:bad-dt",
            Injection::UnquotedField => "rule5:unquoted",
            Injection::FiveLetterGroup => "rule19:five-letter-group",
            Injection::DropProjData => "rule13:drop-proj-data",
            Injection::DropTranData => "rule14:drop-tran-data",
            Injection::UndefinedAbbrev => "rule16:undefined-abbrev",
            Injection::UndefinedType => "rule17:undefined-type",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::model::varied_model;
    use crate::synth::{Scaffold, synth};

    const SEED: u64 = 7;

    /// Build a model and apply `inj` at the same seeded site
    /// `synth_injected` uses, so model- and string-level checks agree.
    fn injected_model(scaffold: Scaffold, seed: u64, inj: Injection) -> ProjectModel {
        let mut m = varied_model(scaffold, seed);
        let mut rng = Rng::seeded(seed ^ PLACEMENT_SALT);
        inj.apply_model(&mut m, &mut rng);
        m
    }

    fn group<'a>(m: &'a ProjectModel, code: &str) -> &'a Group {
        m.groups.iter().find(|g| g.code == code).unwrap()
    }

    /// Every `(group, row, col)` cell marked `Unquote`, in file order.
    fn unquoted_sites(m: &ProjectModel) -> Vec<(usize, usize, usize)> {
        let mut out = Vec::new();
        for (gi, g) in m.groups.iter().enumerate() {
            for (ri, r) in g.rows.iter().enumerate() {
                for f in &r.faults {
                    let RowFault::Unquote(ci) = f;
                    out.push((gi, ri, *ci));
                }
            }
        }
        out
    }

    /// The catalog/injector contract: every injector in `ALL` has a target
    /// rule, a non-empty description, and a token that round-trips through
    /// `parse` and is unique. Guards `forge catalog` (and the `--inject`
    /// surface) against a half-wired new variant.
    #[test]
    fn all_injectors_are_well_formed_and_unique() {
        let mut tokens = std::collections::HashSet::new();
        for &inj in Injection::ALL {
            assert!(
                inj.target_rule().is_some(),
                "{inj} must declare a target rule"
            );
            assert!(!inj.description().is_empty(), "{inj} needs a description");
            let tok = inj.token();
            assert_eq!(Injection::parse(tok), Some(inj), "{tok} must round-trip");
            assert!(tokens.insert(tok), "duplicate token {tok}");
        }
        // None is the only non-ALL variant and parses, but isn't a target.
        assert_eq!(Injection::parse("none"), Some(Injection::None));
        assert!(Injection::None.target_rule().is_none());
    }

    #[test]
    fn none_is_identity() {
        assert_eq!(
            synth_injected(Scaffold::LocaSamp, SEED, Injection::None),
            synth(Scaffold::LocaSamp, SEED)
        );
    }

    #[test]
    fn dup_key_adds_one_duplicate_samp_row() {
        let n_clean = group(&varied_model(Scaffold::LocaSamp, SEED), "SAMP")
            .rows
            .len();
        let dirty = injected_model(Scaffold::LocaSamp, SEED, Injection::DupSampKeyTuple);
        let samp = group(&dirty, "SAMP");
        assert_eq!(samp.rows.len(), n_clean + 1, "dup adds one SAMP row");
        let last = samp.rows.last().unwrap();
        assert!(
            samp.rows[..samp.rows.len() - 1].contains(last),
            "the appended row must duplicate an existing SAMP row"
        );
    }

    #[test]
    fn orphan_repoints_a_samp_at_a_missing_loca() {
        let m = injected_model(Scaffold::LocaSamp, SEED, Injection::OrphanSampRow);
        let samp = group(&m, "SAMP");
        let col = samp.col("LOCA_ID").unwrap();
        assert!(
            samp.rows.iter().any(|r| r.values[col] == "ZZ_ORPHAN"),
            "some SAMP row must reference the missing LOCA"
        );
        assert!(
            !group(&m, "LOCA")
                .rows
                .iter()
                .any(|r| r.values[0] == "ZZ_ORPHAN"),
            "no LOCA carries the orphan id"
        );
    }

    #[test]
    fn drop_proj_data_clears_proj_rows() {
        let m = injected_model(Scaffold::Minimal, SEED, Injection::DropProjData);
        assert!(
            group(&m, "PROJ").rows.is_empty(),
            "PROJ DATA rows must be gone"
        );
    }

    /// `UnquotedField` marks exactly one DATA cell (at a seeded site, not
    /// necessarily PROJ) to emit raw; the marked value appears unquoted.
    #[test]
    fn unquoted_field_emits_one_raw_cell() {
        let m = injected_model(Scaffold::LocaSamp, SEED, Injection::UnquotedField);
        let sites = unquoted_sites(&m);
        assert_eq!(sites.len(), 1, "exactly one cell marked unquoted");
        let (gi, ri, ci) = sites[0];
        let val = &m.groups[gi].rows[ri].values[ci];
        let s = synth_injected(Scaffold::LocaSamp, SEED, Injection::UnquotedField);
        assert!(
            s.contains(&format!(",{val},")) || s.contains(&format!(",{val}\r\n")),
            "the marked value {val:?} must appear unquoted:\n{s}"
        );
    }

    #[test]
    fn injectors_actually_change_the_output() {
        let clean = synth(Scaffold::LocaSamp, SEED);
        for &inj in Injection::ALL {
            assert_ne!(
                synth_injected(Scaffold::LocaSamp, SEED, inj),
                clean,
                "{inj} should mutate the output"
            );
        }
    }

    /// The delegation invariant: a one-element combination is byte-for-byte
    /// the single-injector path (same base, same single RNG draw), for
    /// every injector and a spread of seeds. Guards the `synth_injected`
    /// refactor — and the whole single-rule test suite leans on it.
    #[test]
    fn single_combination_equals_synth_injected() {
        for seed in [0u64, 7, 13, 42] {
            for inj in std::iter::once(Injection::None).chain(Injection::ALL.iter().copied()) {
                assert_eq!(
                    synth_combined(Scaffold::LocaSamp, seed, &[inj]),
                    synth_injected(Scaffold::LocaSamp, seed, inj),
                    "{inj} seed {seed}: one-element combine must equal single inject"
                );
            }
        }
    }

    /// A combination applies *every* listed injector to one model — here a
    /// dup SAMP row (Rule 10a) AND an appended 5-letter group (Rule 19)
    /// both land in the same file. (Pipeline-level proof that the validator
    /// then sees both rules is in `pipeline::tests`.)
    #[test]
    fn combination_applies_all_injectors() {
        let mut m = varied_model(Scaffold::LocaSamp, SEED);
        let mut rng = Rng::seeded(SEED ^ PLACEMENT_SALT);
        let n_samp = group(&m, "SAMP").rows.len();
        for inj in [Injection::DupSampKeyTuple, Injection::FiveLetterGroup] {
            inj.apply_model(&mut m, &mut rng);
        }
        assert_eq!(
            group(&m, "SAMP").rows.len(),
            n_samp + 1,
            "the dup injector still added a SAMP row in the combination"
        );
        assert!(
            m.groups.iter().any(|g| g.code == "ABCDE"),
            "the 5-letter-group injector still appended its group"
        );
    }

    /// Placement is part of the search space: the same injector lands in
    /// different sites across seeds (not pinned to one location).
    #[test]
    fn unquoted_placement_varies_across_seeds() {
        let sites: std::collections::HashSet<_> = (0..20u64)
            .map(|s| {
                unquoted_sites(&injected_model(
                    Scaffold::LocaSamp,
                    s,
                    Injection::UnquotedField,
                ))[0]
            })
            .collect();
        assert!(
            sites.len() > 1,
            "fault placement must vary across seeds, got {sites:?}"
        );
    }
}
