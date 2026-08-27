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

use laterite_ags4_parity::{Rng, reservoir};
use laterite_ags4_validator::{DictVersion, Dictionary};

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
    /// Blank a REQUIRED (non-KEY) DATA cell → Rule 10b (the per-bad-row
    /// "Empty REQUIRED fields" reconstruction). Dictionary-driven: the site is
    /// a field whose status contains REQUIRED but *not* KEY (a KEY+REQUIRED
    /// blank would additionally trip Rule 10a).
    ///
    /// A **multi-rule** injector at volume, like [`Injection::UnquotedField`]. On a
    /// realistic scaffold the only REQUIRED-non-KEY fields are *structural* —
    /// `TRAN_AGS` (drives dictionary-edition detection → Rule 7/9/18 when
    /// blanked) and the `ABBR/UNIT/TYPE` `*_DESC` definitions — so a fileful of
    /// empty-REQUIRED faults cascades rather than isolating Rule 10b. That is a
    /// real property of AGS structure, not a fixture quirk: it makes 10b's
    /// emission inherently bounded, so this injector is a realistic "messy
    /// delivery" generator, not a single-rule probe. Rule 10b's *per-finding*
    /// cost is priced by a cascade-free micro-bench instead.
    EmptyRequired,
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
        // Appended last on purpose: mine.rs pins MINEABLE indices and its
        // C(n,2) count recomputes from the length, so appending keeps both
        // valid where an insert would shift them.
        Injection::EmptyRequired,
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
            "rule10b" | "empty-required" => Injection::EmptyRequired,
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
            Injection::EmptyRequired => "rule10b",
        }
    }

    /// The `--inject` token list, rendered for `--help` from `ALL` itself.
    ///
    /// A hand-maintained list in a doc comment is what produced #709: the help
    /// advertised seven tokens where the binary accepted ten, and a caller who
    /// trusted it never learned the other four existed. Deriving it here means
    /// the two cannot drift — adding a variant to `ALL` updates the help.
    ///
    /// Aliases and scaffold-qualified forms (`rule10a:dup-samp-key`) also parse
    /// and are named rather than enumerated: the qualified suffix is open-ended,
    /// so `catalog` stays the exhaustive source and the help says so.
    pub fn inject_help() -> &'static str {
        static HELP: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        HELP.get_or_init(|| {
            let toks: Vec<&str> = std::iter::once("none")
                .chain(Self::ALL.iter().map(|i| i.token()))
                .collect();
            format!(
                "Rule violation to inject into the clean base: `{}`. \
                 Each also accepts its alias, and a scaffold-qualified form \
                 (e.g. `rule10a:dup-samp-key`); `forge catalog` prints the \
                 authoritative injector→rule map. Repeatable.",
                toks.join("|"),
            )
        })
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
            Injection::EmptyRequired => "blank a REQUIRED (non-KEY) field in a DATA row",
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
            Injection::EmptyRequired => "AGS Format Rule 10b",
        })
    }

    /// Whether this injector needs the LOCA→SAMP scaffold. The relational
    /// injectors need a parent/child pair; `UndefinedAbbrev` needs a
    /// PA-typed cell, which only the boreholes' `LOCA_TYPE/SAMP_TYPE` carry;
    /// `EmptyRequired` needs a pure-REQUIRED field, guaranteed by the
    /// loca-samp/wide dimension groups (ABBR/UNIT/TYPE) — conservatively
    /// `true` so it is never a silent no-op on a bare `Minimal`.
    pub fn needs_relational(self) -> bool {
        matches!(
            self,
            Injection::DupSampKeyTuple
                | Injection::OrphanSampRow
                | Injection::UndefinedAbbrev
                | Injection::EmptyRequired
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
            // Blank *some* pure-REQUIRED (non-KEY) cell → Rule 10b. The site
            // set is dictionary-driven (`empty_required_sites`); a KEY+REQUIRED
            // field is excluded because blanking it would also trip Rule 10a.
            Injection::EmptyRequired => {
                let sites = empty_required_sites(model);
                if !sites.is_empty() {
                    let (gi, ri, ci) = *rng.choose(&sites);
                    model.groups[gi].rows[ri].values[ci] = String::new();
                }
            }
        }
    }
}

impl Injection {
    /// Whether a *density* (fraction of applicable sites) is meaningful for
    /// this injector — i.e. it has a per-row/per-cell site set, not one fixed
    /// site. The structural singletons (`DupSampKeyTuple`, `FiveLetterGroup`,
    /// `DropProjData`, `DropTranData`, `UndefinedType`) corrupt a single
    /// place, so `forge scale --density` rejects them at the CLI. The true-set
    /// here MUST match the arms of [`Injection::apply_dense`].
    #[must_use]
    pub fn supports_density(self) -> bool {
        matches!(
            self,
            Injection::EmptyRequired
                | Injection::OrphanSampRow
                | Injection::BadDtValue
                | Injection::UnquotedField
                | Injection::UndefinedAbbrev
        )
    }

    /// Apply this injector to a deterministic `density` fraction of its
    /// applicable sites (all of them at `density >= 1.0`), returning the count
    /// mutated. The scaled counterpart of [`Injection::apply_model`]: one
    /// seeded site becomes many, so a size-scaled fixture carries a
    /// controllable *fault density* — what lets the validator's
    /// error-emission path be priced at scale (T5).
    ///
    /// Deterministic. It seeds its own placement RNG from `seed` (never the
    /// model-generation RNG, which would shift the clean base bytes and desync
    /// the dirty fixture from its clean twin), and every mutation lands on a
    /// *distinct* site, so the emitted bytes are independent of the reservoir's
    /// internal selection order: same `(seed, density)` → byte-identical file.
    ///
    /// Only the density-capable injectors have an arm ([`Injection::supports_density`]);
    /// the CLI rejects the rest, so the fallback is unreachable in practice.
    #[must_use]
    pub fn apply_dense(self, model: &mut ProjectModel, seed: u64, density: f64) -> usize {
        let mut rng = Rng::seeded(seed ^ PLACEMENT_SALT);
        match self {
            // Pure-REQUIRED (non-KEY) cells, dictionary-driven → Rule 10b.
            Injection::EmptyRequired => {
                let chosen = pick_dense(empty_required_sites(model), density, &mut rng);
                let n = chosen.len();
                for (gi, ri, ci) in chosen {
                    model.groups[gi].rows[ri].values[ci] = String::new();
                }
                n
            }
            // Every chosen SAMP row repointed at a DISTINCT missing LOCA →
            // Rule 10c. The unique bogus id keeps SAMP's KEY tuple unique, so
            // many orphans do not accidentally trip Rule 10a.
            Injection::OrphanSampRow => {
                let Some(gi) = model.groups.iter().position(|g| g.code == "SAMP") else {
                    return 0;
                };
                let Some(col) = model.groups[gi].col("LOCA_ID") else {
                    return 0;
                };
                let rows: Vec<usize> = (0..model.groups[gi].rows.len()).collect();
                let chosen = pick_dense(rows, density, &mut rng);
                let n = chosen.len();
                for ri in chosen {
                    model.groups[gi].rows[ri].values[col] = format!("ZZ_ORPHAN_{ri}");
                }
                n
            }
            // Non-date into DT-typed cells → Rule 8.
            Injection::BadDtValue => {
                let chosen = pick_dense(typed_cell_sites(model, "DT"), density, &mut rng);
                let n = chosen.len();
                for (gi, ri, ci) in chosen {
                    model.groups[gi].rows[ri].values[ci] = "not-a-date".to_string();
                }
                n
            }
            // Non-empty DATA cells emitted unquoted → Rule 5.
            Injection::UnquotedField => {
                let chosen = pick_dense(nonempty_cell_sites(model), density, &mut rng);
                let n = chosen.len();
                for (gi, ri, ci) in chosen {
                    model.groups[gi].rows[ri].faults.push(RowFault::Unquote(ci));
                }
                n
            }
            // Undefined code into PA-typed cells → Rule 16.
            Injection::UndefinedAbbrev => {
                let chosen = pick_dense(typed_cell_sites(model, "PA"), density, &mut rng);
                let n = chosen.len();
                for (gi, ri, ci) in chosen {
                    model.groups[gi].rows[ri].values[ci] = "ZZ".to_string();
                }
                n
            }
            // Structural singletons have no per-site density; the CLI rejects
            // them before this point.
            _ => {
                debug_assert!(false, "apply_dense on a non-density injector: {self}");
                0
            }
        }
    }
}

/// A field whose dictionary status marks it REQUIRED but *not* KEY — the clean
/// Rule 10b site (a KEY+REQUIRED blank would also trip Rule 10a). Statuses are
/// `+`-combined (e.g. `"KEY+REQUIRED"`), so read them part-wise, matching the
/// reference crate's `is_key`.
fn is_pure_required(status: &str) -> bool {
    let mut required = false;
    for part in status.split('+') {
        let part = part.trim();
        if part.eq_ignore_ascii_case("KEY") {
            return false;
        }
        if part.eq_ignore_ascii_case("REQUIRED") {
            required = true;
        }
    }
    required
}

/// Every non-empty cell of a pure-REQUIRED (non-KEY) heading, in file order
/// (groups → headings → rows). Dictionary-driven so it tracks the real
/// REQUIRED set `rule_10b` checks rather than a hardcoded column; ordered Vecs
/// only, so the site order — and the reservoir's pick from it — is deterministic.
fn empty_required_sites(model: &ProjectModel) -> Vec<(usize, usize, usize)> {
    let dict = Dictionary::bundled(DictVersion::V4_2);
    let mut sites = Vec::new();
    for (gi, g) in model.groups.iter().enumerate() {
        for (ci, h) in g.headings.iter().enumerate() {
            let pure_required = dict
                .heading(&g.code, h)
                .is_some_and(|hr| is_pure_required(hr.status));
            if !pure_required {
                continue;
            }
            for (ri, row) in g.rows.iter().enumerate() {
                if row.values.get(ci).is_some_and(|v| !v.is_empty()) {
                    sites.push((gi, ri, ci));
                }
            }
        }
    }
    sites
}

/// Every cell whose heading has AGS type `ty` (e.g. `"DT"`, `"PA"`), in file
/// order — the `BadDtValue`/`UndefinedAbbrev` dense sites.
fn typed_cell_sites(model: &ProjectModel, ty: &str) -> Vec<(usize, usize, usize)> {
    let mut sites = Vec::new();
    for (gi, g) in model.groups.iter().enumerate() {
        for (ci, t) in g.types.iter().enumerate() {
            if t == ty {
                for ri in 0..g.rows.len() {
                    sites.push((gi, ri, ci));
                }
            }
        }
    }
    sites
}

/// Every non-empty DATA cell, in file order — the `UnquotedField` dense sites.
fn nonempty_cell_sites(model: &ProjectModel) -> Vec<(usize, usize, usize)> {
    let mut sites = Vec::new();
    for (gi, g) in model.groups.iter().enumerate() {
        for (ri, row) in g.rows.iter().enumerate() {
            for (ci, v) in row.values.iter().enumerate() {
                if !v.is_empty() {
                    sites.push((gi, ri, ci));
                }
            }
        }
    }
    sites
}

/// Pick a deterministic `density` fraction of `sites`: all of them at
/// `density >= 1.0` (no sampling — the fixture's common case, trivially
/// reproducible), else `ceil(density * n)` reservoir-sampled with `rng`.
fn pick_dense<T>(sites: Vec<T>, density: f64, rng: &mut Rng) -> Vec<T> {
    let n = sites.len();
    if n == 0 || density >= 1.0 {
        return sites;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k = ((density * n as f64).ceil() as usize).clamp(1, n);
    reservoir(sites.into_iter(), k, rng)
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
            Injection::EmptyRequired => "rule10b:empty-required",
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

    /// `EmptyRequired`'s single-site form blanks exactly one non-empty
    /// pure-REQUIRED cell (`empty_required_sites` returns only non-empty ones,
    /// so its count drops by one).
    #[test]
    fn empty_required_blanks_one_pure_required_cell() {
        let mut m = varied_model(Scaffold::LocaSamp, SEED);
        let before = empty_required_sites(&m).len();
        assert!(before > 0, "loca-samp carries pure-REQUIRED cells");
        let mut rng = Rng::seeded(SEED ^ PLACEMENT_SALT);
        Injection::EmptyRequired.apply_model(&mut m, &mut rng);
        assert_eq!(
            empty_required_sites(&m).len(),
            before - 1,
            "exactly one required cell blanked"
        );
    }

    /// `apply_dense` corrupts exactly `k = ceil(density * n)` sites (all `n` at
    /// density 1.0), and returns that count.
    #[test]
    fn dense_10b_count_matches_density() {
        let n = empty_required_sites(&varied_model(Scaffold::Wide, SEED)).len();
        assert!(n > 0, "wide carries pure-REQUIRED cells");

        let mut full = varied_model(Scaffold::Wide, SEED);
        assert_eq!(
            Injection::EmptyRequired.apply_dense(&mut full, SEED, 1.0),
            n
        );
        assert_eq!(
            empty_required_sites(&full).len(),
            0,
            "density 1.0 blanks every site"
        );

        let mut half = varied_model(Scaffold::Wide, SEED);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected = ((0.5 * n as f64).ceil() as usize).clamp(1, n);
        assert_eq!(
            Injection::EmptyRequired.apply_dense(&mut half, SEED, 0.5),
            expected
        );
    }

    /// Same `(seed, density)` → byte-identical file, even at a partial density
    /// where the reservoir samples (distinct sites + fixed emit order make the
    /// bytes independent of selection order).
    #[test]
    fn dense_is_byte_deterministic() {
        let build = |density| {
            let mut m = varied_model(Scaffold::Wide, SEED);
            let _ = Injection::EmptyRequired.apply_dense(&mut m, SEED, density);
            emit_to_string(&m)
        };
        assert_eq!(build(0.5), build(0.5), "partial density is reproducible");
        assert_eq!(build(1.0), build(1.0), "full density is reproducible");
    }

    /// The density-capable set is exactly the per-row/per-cell injectors — and
    /// it must match `apply_dense`'s arms. Forces a deliberate call for any
    /// future variant.
    #[test]
    fn supports_density_partitions_injectors() {
        for &inj in Injection::ALL {
            let expected = matches!(
                inj,
                Injection::EmptyRequired
                    | Injection::OrphanSampRow
                    | Injection::BadDtValue
                    | Injection::UnquotedField
                    | Injection::UndefinedAbbrev
            );
            assert_eq!(inj.supports_density(), expected, "{inj}");
        }
        assert!(!Injection::None.supports_density());
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
