//! The typed in-memory model the emitter walks, and the **seeded varied
//! generator** that builds it.
//!
//! There is no "fixed vs varied" mode: synth is realistic by default and
//! a `seed` only controls reproducibility (same seed → byte-identical
//! output, the determinism contract the tests pin). Every borehole gets
//! its *own* generated profile — distinct id, sampled activity type and
//! ground level, and its own sample regime (count + monotonic depths +
//! sampled sample-types) — so the output is genuinely diverse, not one
//! row cloned N times.
//!
//! Realism without losing clean-by-construction: the group/heading/type
//! structure is the proven-clean v4.2 shape (asserted `RustResult::Clean`
//! across many seeds), and every value is type-correct — picklist (PA)
//! codes are sampled from the bundled dictionary's own ABBR table and the
//! `ABBR` group emits exactly the codes used (with their canonical
//! descriptions), so PA/Rule-16 stay satisfied.
//!
//! Shape: a `ProjectModel` is an ordered list of `Group`s; each carries
//! its 4-letter `code`, the ordered `headings`/`units`/`types` arrays
//! (one entry per column), and its `rows` of DATA values.

use std::collections::BTreeSet;

use laterite_ags4_emit::catalog;
use laterite_ags4_parity::Rng;
use laterite_ags4_validator::{DictVersion, Dictionary};

use super::{Scaffold, breadth, bs5930, depth, generic, geotech};

/// An emitter-level defect marker on a DATA row — a malformation the
/// structured model can't otherwise express because the emitter quotes
/// every field uniformly. Injectors (A2) attach these; the emitter
/// honours them. Most injections are *structural* (duplicate/drop a row,
/// change a value, add a group) and need no marker — this is for the few
/// that don't survive normal quoting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowFault {
    /// Emit the cell at this column index **without** the surrounding
    /// quotes (a Rule 5 "missing quotes" violation).
    Unquote(usize),
}

/// One AGS4 DATA row: the values for each heading, in column order, plus
/// any emitter-level [`RowFault`] markers (empty for a clean row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Field values, one per heading, in column order. Each is the bare
    /// value (no quotes) — the emitter quotes every field uniformly
    /// unless a [`RowFault`] says otherwise.
    pub values: Vec<String>,
    /// Emitter-level defect markers (default empty).
    pub faults: Vec<RowFault>,
}

impl Row {
    /// A clean (fault-free) row from owned field strings.
    pub fn owned(values: Vec<String>) -> Self {
        Row {
            values,
            faults: Vec::new(),
        }
    }

    /// True if the cell at `col` is marked to emit without quotes.
    pub fn is_unquoted(&self, col: usize) -> bool {
        self.faults.contains(&RowFault::Unquote(col))
    }
}

/// One AGS4 group: its code plus the parallel HEADING/UNIT/TYPE column
/// definitions and its DATA rows. `headings`, `units`, `types` are the
/// same length (one entry per column); each `Row`'s `values` matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// 4-letter group code (PROJ, LOCA, SAMP, …).
    pub code: String,
    /// Column heading names, in order (e.g. `PROJ_ID`, `PROJ_NAME`).
    pub headings: Vec<String>,
    /// UNIT row values, one per heading (empty string = no unit).
    pub units: Vec<String>,
    /// TYPE row values, one per heading (the AGS data-type code).
    pub types: Vec<String>,
    /// DATA rows, in order.
    pub rows: Vec<Row>,
}

impl Group {
    /// Build a group from a fixed column schema (`headings`/`units`/
    /// `types` are the same length) plus already-built DATA rows.
    fn new<const N: usize>(
        code: &str,
        headings: [&str; N],
        units: [&str; N],
        types: [&str; N],
        rows: Vec<Row>,
    ) -> Self {
        Group {
            code: code.to_string(),
            headings: headings
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            units: units.iter().map(std::string::ToString::to_string).collect(),
            types: types.iter().map(std::string::ToString::to_string).collect(),
            rows,
        }
    }

    /// Column index of `heading`, if present — so injectors target a
    /// field by name (robust to value/scaffold changes) instead of by
    /// position or a literal byte match.
    pub fn col(&self, heading: &str) -> Option<usize> {
        self.headings.iter().position(|h| h == heading)
    }
}

/// The whole synthetic file: an ordered list of groups (vector order is
/// file order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModel {
    pub groups: Vec<Group>,
}

impl ProjectModel {
    /// First group with the given 4-letter `code`, mutably — so an
    /// injector can edit it by name (no-op `None` if the scaffold doesn't
    /// carry that group).
    pub fn group_mut(&mut self, code: &str) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| g.code == code)
    }
}

/// Build a realistic, clean-by-construction model for `scaffold`, seeded
/// by `seed` (same seed → identical model). `Minimal` = PROJ + TRAN (+
/// the UNIT/TYPE those need); `LocaSamp` inserts a varied LOCA→SAMP set
/// of boreholes + the ABBR/UNIT/TYPE definitions they introduce.
pub fn varied_model(scaffold: Scaffold, seed: u64) -> ProjectModel {
    build(scaffold, seed, None, None, DENSE_LAB)
}

/// Like [`varied_model`] but with an explicit borehole count `n_loca`
/// (`None` ⇒ a random 3–7, the normal file). The LOCA-id width tracks the
/// count's digits. `n_loca` is the `scale` command's knob: more boreholes →
/// proportionally more SAMP/GEOL/breadth rows → a bigger file.
pub fn varied_model_n(scaffold: Scaffold, seed: u64, n_loca: Option<usize>) -> ProjectModel {
    build(scaffold, seed, n_loca, None, DENSE_LAB)
}

/// Like [`varied_model_n`] but with an explicit LOCA-id width — so the
/// `scale` calibration can measure tiny samples at the *target's* id width
/// and stay exact at any size.
pub fn varied_model_sized(
    scaffold: Scaffold,
    seed: u64,
    n_loca: usize,
    id_width: usize,
) -> ProjectModel {
    build(scaffold, seed, Some(n_loca), Some(id_width), DENSE_LAB)
}

/// The dense (every-sample-every-test) lab-test rate — the default that keeps
/// `Wide` byte-identical to before the `--lab-test-rate` knob existed.
pub const DENSE_LAB: f64 = 1.0;

/// Like [`varied_model`] but with a `lab_rate` (`Wide` only): the per-sample
/// probability each lab-test result is present (`1.0` = dense). `< 1.0` gives a
/// realistic sparse test matrix (seeded → deterministic). `gen`'s rate path.
pub fn varied_model_lab(scaffold: Scaffold, seed: u64, lab_rate: f64) -> ProjectModel {
    build(scaffold, seed, None, None, lab_rate)
}

/// The id width for a borehole count — its digit count, never below 3 (so
/// a normal 3–7 file keeps `BH001`).
pub fn id_width_for(n_loca: usize) -> usize {
    n_loca.to_string().len().max(3)
}

// `rng.range(3, 7) as usize` below always returns a value in `3..=7`,
// comfortably inside usize.
#[allow(clippy::cast_possible_truncation)]
fn build(
    scaffold: Scaffold,
    seed: u64,
    n_loca: Option<usize>,
    id_width: Option<usize>,
    lab_rate: f64,
) -> ProjectModel {
    let mut rng = Rng::seeded(seed);
    let dict = Dictionary::bundled(DictVersion::V4_2);

    let mut groups = vec![proj(&mut rng), tran(&mut rng)];
    match scaffold {
        Scaffold::Minimal => {
            groups.push(unit_group(false));
            groups.push(type_group(false));
        }
        Scaffold::LocaSamp => {
            let n = n_loca.unwrap_or_else(|| rng.range(3, 7) as usize);
            let w = id_width.unwrap_or_else(|| id_width_for(n));
            let (loca, samp, geol, abbr) = boreholes(&mut rng, &dict, n, w);
            groups.push(loca);
            groups.push(samp);
            groups.push(geol);
            groups.push(abbr);
            groups.push(unit_group(true));
            groups.push(type_group(true));
        }
        Scaffold::Wide => {
            // The borehole core + every safe LOCA-child group + the depth
            // (SAMP-child lab-test results + LBSG/LBST schedule), then ABBR/
            // UNIT/TYPE scanned from whatever those groups actually used (so
            // the wide set's varied picklists/units/types stay covered —
            // Rule 15/16/17 clean by construction).
            let n = n_loca.unwrap_or_else(|| rng.range(3, 7) as usize);
            let w = id_width.unwrap_or_else(|| id_width_for(n));
            let (loca, samp, geol, _abbr) = boreholes(&mut rng, &dict, n, w);
            let loca_ids: Vec<String> = loca.rows.iter().map(|r| r.values[0].clone()).collect();
            // SAMP rows are [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID]
            // — the parent five-key the lab-test/schedule depth links to.
            let samp_keys: Vec<Vec<String>> = samp.rows.iter().map(|r| r.values.clone()).collect();
            groups.push(loca);
            groups.push(samp);
            groups.push(geol);
            for code in breadth::safe_loca_children(&dict) {
                groups.push(breadth::generate(&dict, code, &loca_ids, &mut rng));
            }
            // Depth: lab-test result groups deepen the file below each sample
            // (2nd level), then their own children deepen it once more (3rd
            // level — TREG→TRET etc.); LBSG/LBST schedule named tests against
            // the same samples. All clean-by-construction (Rule 10c links).
            // `lab_rate` < 1.0 makes the per-sample test matrix sparse.
            groups.extend(depth::generate_lab_depth(
                &dict, &samp_keys, lab_rate, &mut rng,
            ));
            let (lbsg, lbst) = depth::schedule(&dict, &samp_keys, &mut rng);
            groups.push(lbsg);
            groups.push(lbst);
            let abbr = collect_abbr(&groups, &dict);
            let unit = collect_unit(&groups);
            let types = collect_type(&groups);
            groups.push(abbr);
            groups.push(unit);
            groups.push(types);
        }
    }
    ProjectModel { groups }
}

/// Build the `ABBR` group covering every (heading, code) used in a PA-typed
/// cell across `groups` — so Rule 16 holds for the whole (wide) file. The
/// description is the dictionary's canonical one (FYI), falling back to the
/// code itself if absent.
fn collect_abbr(groups: &[Group], dict: &Dictionary<'static>) -> Group {
    let mut used: BTreeSet<(String, String)> = BTreeSet::new();
    for g in groups {
        for (ci, t) in g.types.iter().enumerate() {
            if t == "PA" {
                for r in &g.rows {
                    let v = &r.values[ci];
                    if !v.is_empty() {
                        used.insert((g.headings[ci].clone(), v.clone()));
                    }
                }
            }
        }
    }
    let rows = used
        .iter()
        .map(|(h, c)| {
            let desc = dict.abbr_desc(h, c).unwrap_or(c.as_str());
            Row::owned(vec![h.clone(), c.clone(), desc.to_string()])
        })
        .collect();
    Group::new(
        "ABBR",
        ["ABBR_HDNG", "ABBR_CODE", "ABBR_DESC"],
        ["", "", ""],
        ["X", "X", "X"],
        rows,
    )
}

// The collection rule — trim, the header-literal exclusions, the PU
// harvest — is `laterite_ags4_emit::catalog` (#924), shared with the
// shipped emitter's catalog synthesis. Reliquary row 14 kept the two
// collectors separate "until they drift"; they drifted (forge kept padded
// units the emitter trimmed, and never learned the PU harvest, so the
// dogfood corpus could not manufacture the picklist-of-units case the
// emitter handles). This adapter is forge's half of the seam.
impl catalog::GroupView for Group {
    fn units(&self) -> &[String] {
        &self.units
    }
    fn types(&self) -> &[String] {
        &self.types
    }
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn cell(&self, row: usize, col: usize) -> Option<&str> {
        self.rows.get(row)?.values.get(col).map(String::as_str)
    }
}

/// Build the `UNIT` group covering every unit any group uses (Rule 15,
/// via `catalog::units_used` — UNIT-row values plus PU-column cells).
/// Description falls back to the unit symbol itself.
fn collect_unit(groups: &[Group]) -> Group {
    let rows = catalog::units_used(groups)
        .iter()
        .map(|u| Row::owned(vec![u.clone(), u.clone()]))
        .collect();
    Group::new(
        "UNIT",
        ["UNIT_UNIT", "UNIT_DESC"],
        ["", ""],
        ["X", "X"],
        rows,
    )
}

/// Build the `TYPE` group covering every type code any group declares
/// (Rule 17, via `catalog::types_used`). Description falls back to the
/// code itself.
fn collect_type(groups: &[Group]) -> Group {
    let rows = catalog::types_used(groups)
        .iter()
        .map(|t| Row::owned(vec![t.clone(), t.clone()]))
        .collect();
    Group::new(
        "TYPE",
        ["TYPE_TYPE", "TYPE_DESC"],
        ["", ""],
        ["X", "X"],
        rows,
    )
}

/// PROJ — a varied id + a composed project name.
fn proj(rng: &mut Rng) -> Group {
    Group::new(
        "PROJ",
        ["PROJ_ID", "PROJ_NAME"],
        ["", ""],
        ["ID", "X"],
        vec![Row::owned(vec![
            format!("P{:04}", rng.range(1000, 9999)),
            generic::project_name(rng),
        ])],
    )
}

/// TRAN — the transmission header; varied date + producer, AGS edition
/// pinned to the dictionary the model is clean against (4.2).
fn tran(rng: &mut Rng) -> Group {
    Group::new(
        "TRAN",
        [
            "TRAN_ISNO",
            "TRAN_DATE",
            "TRAN_PROD",
            "TRAN_STAT",
            "TRAN_AGS",
            "TRAN_RECV",
            "TRAN_DLIM",
            "TRAN_RCON",
        ],
        ["", "yyyy-mm-dd", "", "", "", "", "", ""],
        ["X", "DT", "X", "X", "X", "X", "X", "X"],
        vec![Row::owned(vec![
            "1".to_string(),
            generic::iso_date(rng),
            generic::producer(rng),
            "Draft".to_string(),
            "4.2".to_string(),
            generic::producer(rng),
            "|".to_string(),
            "+".to_string(),
        ])],
    )
}

/// The varied LOCA→SAMP→GEOL boreholes (`n_loca` of them) + the ABBR group
/// covering exactly the PA codes they use. Each borehole has its own
/// sampled activity type, ground level, 1–3 samples at monotonically
/// increasing depths, and 2–5 contiguous geological strata each carrying a
/// constraint-valid BS 5930 description (the [`bs5930`] engine). `n_loca`
/// is the scale knob (the caller draws 3–7 for a normal file, or a
/// calibrated count for `scale`).
fn boreholes(
    rng: &mut Rng,
    dict: &Dictionary<'static>,
    n_loca: usize,
    id_width: usize,
) -> (Group, Group, Group, Group) {
    let loca_types = dict.abbr_codes("LOCA_TYPE");
    let samp_types = dict.abbr_codes("SAMP_TYPE");
    // Sorted (heading, code) so the ABBR group is deterministic per seed.
    let mut used: BTreeSet<(&'static str, &'static str)> = BTreeSet::new();
    let mut loca_rows = Vec::new();
    let mut samp_rows = Vec::new();
    let mut loca_ids = Vec::new();

    for i in 0..n_loca {
        // Fixed-width id across the whole file → constant per-borehole
        // bytes, so the `scale` calibration is exact at any size.
        let loca_id = format!("BH{:0id_width$}", i + 1);
        let lt = *rng.choose(&loca_types);
        used.insert(("LOCA_TYPE", lt));
        loca_rows.push(Row::owned(vec![
            loca_id.clone(),
            lt.to_string(),
            geotech::ground_level(rng),
        ]));

        let n_samp = rng.range(1, 3);
        let mut top = geotech::depth_step(rng);
        for j in 0..n_samp {
            let st = *rng.choose(&samp_types);
            used.insert(("SAMP_TYPE", st));
            let samp_ref = format!("S{}", j + 1);
            let samp_id = format!("{loca_id}-{samp_ref}");
            samp_rows.push(Row::owned(vec![
                loca_id.clone(),
                format!("{top:.2}"),
                samp_ref,
                st.to_string(),
                samp_id,
            ]));
            top += geotech::depth_step(rng);
        }
        loca_ids.push(loca_id);
    }

    // GEOL strata per borehole — contiguous depths from ground level down,
    // each described by the BS 5930 engine (a separate pass over the known
    // LOCA ids keeps the LOCA/SAMP byte stream stable). The (LOCA_ID,
    // GEOL_TOP) KEY is unique because tops climb within a borehole and the
    // id differs between boreholes; the parent LOCA always exists.
    let voc = bs5930::vocab();
    let mut geol_rows = Vec::new();
    for loca_id in &loca_ids {
        let n_strata = rng.range(2, 5);
        let mut top = 0.0_f64;
        for _ in 0..n_strata {
            let base = top + geotech::depth_step(rng);
            let desc = bs5930::describe(voc, rng).text;
            geol_rows.push(Row::owned(vec![
                loca_id.clone(),
                format!("{top:.2}"),
                format!("{base:.2}"),
                desc,
            ]));
            top = base;
        }
    }

    let loca = Group::new(
        "LOCA",
        ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"],
        ["", "", "m"],
        ["ID", "PA", "2DP"],
        loca_rows,
    );
    let samp = Group::new(
        "SAMP",
        ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"],
        ["", "m", "", "", ""],
        ["ID", "2DP", "X", "PA", "ID"],
        samp_rows,
    );
    // GEOL — KEY (LOCA_ID, GEOL_TOP), REQUIRED (GEOL_BASE, GEOL_DESC), in
    // dictionary heading order.
    let geol = Group::new(
        "GEOL",
        ["LOCA_ID", "GEOL_TOP", "GEOL_BASE", "GEOL_DESC"],
        ["", "m", "m", ""],
        ["ID", "2DP", "2DP", "X"],
        geol_rows,
    );
    // One ABBR row per used (heading, code), with the dictionary's
    // canonical description (so Rule 16's FYI compare passes).
    let abbr_rows = used
        .iter()
        .map(|(h, c)| {
            let desc = dict
                .abbr_desc(h, c)
                .expect("sampled code came from abbr_codes, so abbr_desc resolves");
            Row::owned(vec![h.to_string(), c.to_string(), desc.to_string()])
        })
        .collect();
    let abbr = Group::new(
        "ABBR",
        ["ABBR_HDNG", "ABBR_CODE", "ABBR_DESC"],
        ["", "", ""],
        ["X", "X", "X"],
        abbr_rows,
    );
    (loca, samp, geol, abbr)
}

/// UNIT definitions covering every unit the data groups use (Rule 15):
/// `yyyy-mm-dd` always; `m` once boreholes (`LOCA_GL/SAMP_TOP`) appear.
fn unit_group(loca_samp: bool) -> Group {
    let mut rows = vec![Row::owned(vec![
        "yyyy-mm-dd".to_string(),
        "year month day".to_string(),
    ])];
    if loca_samp {
        rows.push(Row::owned(vec!["m".to_string(), "metre".to_string()]));
    }
    Group::new(
        "UNIT",
        ["UNIT_UNIT", "UNIT_DESC"],
        ["", ""],
        ["X", "X"],
        rows,
    )
}

/// TYPE definitions covering every type the data groups use (Rule 17):
/// `ID`/`X`/`DT` always; `2DP`/`PA` once boreholes appear.
fn type_group(loca_samp: bool) -> Group {
    let mut rows = vec![
        Row::owned(vec!["ID".to_string(), "Unique identifier".to_string()]),
        Row::owned(vec!["X".to_string(), "Text".to_string()]),
        Row::owned(vec!["DT".to_string(), "Date and time".to_string()]),
    ];
    if loca_samp {
        rows.push(Row::owned(vec![
            "2DP".to_string(),
            "2 decimal places".to_string(),
        ]));
        rows.push(Row::owned(vec![
            "PA".to_string(),
            "Abbreviation (pick list)".to_string(),
        ]));
    }
    Group::new(
        "TYPE",
        ["TYPE_TYPE", "TYPE_DESC"],
        ["", ""],
        ["X", "X"],
        rows,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group<'a>(m: &'a ProjectModel, code: &str) -> &'a Group {
        m.groups.iter().find(|g| g.code == code).unwrap()
    }

    /// The convergence pin (#924): the SAME logical input as emit's
    /// `collect_units_gathers_unit_rows_and_pu_columns_only` — a padded
    /// unit, a header-literal, a PU column with a blank cell — asserted to
    /// the same expected sets, through forge's own `Group` adapter. Before
    /// the shared rule this input produced a DIFFERENT catalog here: the
    /// padded `" m "` survived untrimmed, `"UNIT"`/`"TYPE"` were admitted,
    /// and `kPa` (the PU cell) was never harvested at all.
    #[test]
    fn catalog_collection_matches_the_emitters_semantics() {
        let g = Group::new(
            "XXXX",
            ["A", "B"],
            [" m ", "UNIT"],
            ["PU", "TYPE"],
            vec![
                Row::owned(vec!["kPa".into(), "notpu".into()]),
                Row::owned(vec!["  ".into(), "blankpu".into()]),
            ],
        );
        let unit = collect_unit(std::slice::from_ref(&g));
        let units: Vec<&str> = unit.rows.iter().map(|r| r.values[0].as_str()).collect();
        assert_eq!(units, vec!["kPa", "m"]);
        let ty = collect_type(std::slice::from_ref(&g));
        let types: Vec<&str> = ty.rows.iter().map(|r| r.values[0].as_str()).collect();
        assert_eq!(types, vec!["PU"]);
    }

    /// GEOL strata are coherent and clean-by-construction: every row's
    /// parent LOCA exists, the description is non-empty, and per borehole
    /// the strata are contiguous (start at 0.00, each top == the previous
    /// base) — so the (`LOCA_ID`, `GEOL_TOP`) KEY is unique and the depths read
    /// like a real log. (Validator-level cleanliness is pinned separately
    /// by `pipeline::varied_baseline_is_rust_clean`.)
    // Exact equality is the actual invariant here, not an approximation to
    // relax: the model writes each stratum's `top`/`base` via `format!("{x:.2}")`
    // of the SAME underlying f64 on both sides of a boundary (this row's `top` is
    // literally the previous row's `base`), and the first `top` is the literal
    // `0.0_f64` — so `format!` + `parse` round-trips deterministically to the
    // identical bit pattern every time. An epsilon here would hide the exact
    // drift this test exists to catch.
    #[allow(clippy::float_cmp)]
    #[test]
    fn geol_strata_are_coherent_per_borehole() {
        for seed in 0..30u64 {
            let m = varied_model(Scaffold::LocaSamp, seed);
            let geol = group(&m, "GEOL");
            let loca = group(&m, "LOCA");
            let loca_ids: std::collections::HashSet<&str> =
                loca.rows.iter().map(|r| r.values[0].as_str()).collect();
            assert!(!geol.rows.is_empty(), "seed {seed}: GEOL has strata");

            // Walk strata grouped by their (consecutive) LOCA_ID.
            let mut prev_id = "";
            let mut prev_base = 0.0_f64;
            for r in &geol.rows {
                let id = r.values[0].as_str();
                let top: f64 = r.values[1].parse().unwrap();
                let base: f64 = r.values[2].parse().unwrap();
                let desc = &r.values[3];
                assert!(loca_ids.contains(id), "seed {seed}: orphan GEOL {id}");
                assert!(!desc.trim().is_empty(), "seed {seed}: empty GEOL_DESC");
                assert!(base > top, "seed {seed}: {id} base {base} <= top {top}");
                if id == prev_id {
                    assert_eq!(top, prev_base, "seed {seed}: {id} strata not contiguous");
                } else {
                    assert_eq!(
                        top, 0.0,
                        "seed {seed}: {id} first stratum must start at 0.00"
                    );
                }
                prev_id = id;
                prev_base = base;
            }
        }
    }
}
