//! Generic (non-geotech) field generators. Names are **composed** from
//! word-parts so the variety is combinatorial (hundreds–thousands of
//! distinct outputs), not a short fixed list; dates are sampled in a
//! plausible range. All draws go through the shared deterministic
//! [`Rng`](laterite_ags4_parity::Rng), so a seed → byte-identical output.

use laterite_ags4_parity::Rng;

const PLACES: &[&str] = &[
    "Ashford",
    "Barnwell",
    "Carlisle",
    "Dunsfold",
    "Elmstead",
    "Fenwick",
    "Greenhithe",
    "Hartcliffe",
    "Ilkeston",
    "Kelvedon",
    "Langley",
    "Marsden",
    "Northwich",
    "Oakworth",
    "Penistone",
    "Redmarley",
    "Stanford",
    "Thurnby",
    "Uxbridge",
    "Wexham",
];
const FEATURES: &[&str] = &[
    "Bridge",
    "Interchange",
    "Reservoir",
    "Viaduct",
    "Embankment",
    "Tunnel",
    "Cutting",
    "Quay",
    "Sidings",
    "Distributor Road",
    "Flood Wall",
    "Pumping Station",
    "Substation",
];
const SCHEMES: &[&str] = &[
    "Highway Improvement",
    "Flood Alleviation",
    "Site Investigation",
    "Ground Investigation",
    "Redevelopment",
    "Capacity Upgrade",
    "Geotechnical Appraisal",
    "Phase 2 Investigation",
];
const PRODUCERS: &[&str] = &[
    "Geoterra",
    "Stratum",
    "Subsoil",
    "Terrafirma",
    "Boreline",
    "Geosense",
    "Probe",
    "Coreworks",
    "Deepground",
    "Sondex",
];
const SUFFIXES: &[&str] = &[
    "Ltd",
    "Geotechnical",
    "Geosciences",
    "Engineering",
    "Consulting",
];

/// A composed project name, e.g. `"Ashford Bridge Highway Improvement"`.
pub fn project_name(rng: &mut Rng) -> String {
    format!(
        "{} {} {}",
        rng.choose(PLACES),
        rng.choose(FEATURES),
        rng.choose(SCHEMES)
    )
}

/// A composed producer/company name, e.g. `"Geoterra Geotechnical"`.
pub fn producer(rng: &mut Rng) -> String {
    format!("{} {}", rng.choose(PRODUCERS), rng.choose(SUFFIXES))
}

/// A valid ISO `yyyy-mm-dd` date (day capped at 28 so every draw is a
/// real calendar date → always a clean AGS4 `DT` value).
pub fn iso_date(rng: &mut Rng) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        rng.range(2015, 2024),
        rng.range(1, 12),
        rng.range(1, 28)
    )
}
