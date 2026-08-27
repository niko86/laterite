//! Geotech numeric generators — type-correct AGS4 field strings sampled
//! from curated *continuous* ranges (sourced from
//! `examples/benchmark_scale.py`) through the shared deterministic
//! [`Rng`], so a seed → byte-identical output and the
//! variety is generated (a real range), never a small fixed list.

use laterite_ags4_parity::Rng;

/// A `2DP` value uniformly in `[lo, hi]`, formatted as the AGS4 field
/// string (e.g. `"42.17"`). `lo`/`hi` are inclusive to the centimetre.
// The only caller (`ground_level`) passes hardcoded 10.0..100.0; `*100.0`
// rounds well within i64 for any value in that realistic range.
#[allow(clippy::cast_possible_truncation)]
pub fn dp2(rng: &mut Rng, lo: f64, hi: f64) -> String {
    let v = rng.range((lo * 100.0).round() as i64, (hi * 100.0).round() as i64);
    format!("{:.2}", v as f64 / 100.0)
}

/// Ground level (mOD) — a plausible 10–100 m (`benchmark_scale` `loca_gl`).
pub fn ground_level(rng: &mut Rng) -> String {
    dp2(rng, 10.0, 100.0)
}

/// A sample/strata depth increment (m) — 0.25–2.50, so a borehole's
/// successive depths climb monotonically by realistic steps.
pub fn depth_step(rng: &mut Rng) -> f64 {
    rng.range(25, 250) as f64 / 100.0
}
