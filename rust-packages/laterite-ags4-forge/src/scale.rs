//! `forge scale` — calibrated byte-size synthesis.
//!
//! The perf/compliance matrix wants valid AGS4 files at a *target size*
//! (500 KB … 1 GB), not a fixed borehole count. File size scales linearly
//! with the borehole count `n_loca` (each LOCA adds a fixed bundle of
//! SAMP/GEOL/breadth rows), so a cheap two-point measurement on tiny
//! samples extrapolates straight to the `n_loca` that lands near the
//! target — no need to generate the big file twice. Bytes are *counted*
//! through the streaming emitter (no output buffer) during calibration.
//!
//! Memory note: the final file is built as one in-memory `ProjectModel`
//! then streamed out, so peak RAM ≈ the model (a few × the file size). The
//! multi-hundred-MB tiers are comfortable; a true 1 GB tier wants a
//! row-streaming generator (a documented follow-up).

use std::io::{self, Write};

use crate::synth::Scaffold;
use crate::synth::emit;
use crate::synth::model::{id_width_for, varied_model_sized};

/// Parse a size like `500KB`, `1.5MB`, `1GB`, or a raw byte count `1048576`
/// (decimal K/M/G; a trailing `B` is optional).
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    let s = s.strip_suffix('b').unwrap_or(&s);
    let (num, mult) = if let Some(n) = s.strip_suffix('k') {
        (n, 1_000u64)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 1_000_000)
    } else if let Some(n) = s.strip_suffix('g') {
        (n, 1_000_000_000)
    } else {
        (s, 1)
    };
    let v: f64 = num.trim().parse().ok()?;
    if v < 0.0 {
        return None;
    }
    // Rust's float→int cast saturates (defined since 1.45, never UB/wraps);
    // an absurd size string just clamps to u64::MAX rather than corrupting,
    // and every real caller passes a benchmark target in 500KB..1GB.
    #[allow(clippy::cast_possible_truncation)]
    let bytes = (v * mult as f64).round() as u64;
    Some(bytes)
}

/// An `io::Write` that discards its input and counts the bytes — lets the
/// streaming emitter measure a model's size without allocating it.
struct Counter(u64);

impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len() as u64;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Bytes the emitter writes for `(scaffold, seed, n_loca)` at a given
/// LOCA-id width — counted, not buffered.
pub fn emit_bytes_w(scaffold: Scaffold, seed: u64, n_loca: usize, id_width: usize) -> u64 {
    let model = varied_model_sized(scaffold, seed, n_loca, id_width);
    let mut c = Counter(0);
    emit::emit(&model, &mut c).expect("counting writer never errors");
    c.0
}

/// The calibration outcome: the borehole count, and the size the linear fit
/// predicts for it.
pub struct Calibration {
    pub n_loca: usize,
    pub predicted_bytes: u64,
}

/// Two-point linear calibration: `bytes ≈ overhead + k·n_loca`, solved from
/// tiny samples, then inverted for `target`. Two passes — the first gets a
/// rough count to fix the LOCA-id width, the second measures the samples at
/// *that* width so the per-borehole bytes match the final file exactly
/// (the id repeats across ~50 rows per borehole, so its width matters).
/// Errors when the scaffold doesn't scale with boreholes (e.g. `Minimal`).
pub fn calibrate(scaffold: Scaffold, seed: u64, target: u64) -> Result<Calibration, String> {
    let (n1, n2) = (16usize, 64usize);
    let solve = |width: usize| -> Result<(f64, f64), String> {
        let b1 = emit_bytes_w(scaffold, seed, n1, width) as f64;
        let b2 = emit_bytes_w(scaffold, seed, n2, width) as f64;
        let k = (b2 - b1) / (n2 - n1) as f64;
        if k <= 0.0 {
            return Err(format!(
                "scaffold {scaffold:?} does not scale with borehole count \
                 (use loca-samp or wide)"
            ));
        }
        Ok((k, b1 - k * n1 as f64))
    };
    // The inverted borehole count for a byte target `500KB..1GB` (this
    // module's documented range, see the file header): a few thousand at
    // most, so the `i64`/`usize` narrowing below never truncates in
    // practice, and the saturating float→int cast (defined since Rust 1.45)
    // is never UB even for a wildly out-of-range `target`.
    #[allow(clippy::cast_possible_truncation)]
    let invert =
        |k: f64, overhead: f64| (((target as f64 - overhead) / k).round() as i64).max(1) as usize;

    // Pass 1: a rough count at the samples' own width → the target width.
    let (k0, oh0) = solve(id_width_for(n1))?;
    let width = id_width_for(invert(k0, oh0));
    // Pass 2: re-measure at that width for an exact fit.
    let (k, overhead) = solve(width)?;
    let n_loca = invert(k, overhead);
    // Same bound as above: byte counts for the documented 500KB..1GB range
    // land far under u64::MAX, and `.max(0.0)` already rules out the
    // negative-cast case.
    #[allow(clippy::cast_possible_truncation)]
    let predicted_bytes = (overhead + k * n_loca as f64).round().max(0.0) as u64;
    Ok(Calibration {
        n_loca,
        predicted_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_handles_suffixes_and_raw() {
        assert_eq!(parse_size("500KB"), Some(500_000));
        assert_eq!(parse_size("1.5MB"), Some(1_500_000));
        assert_eq!(parse_size("1GB"), Some(1_000_000_000));
        assert_eq!(parse_size("2048"), Some(2048));
        assert_eq!(parse_size("1024b"), Some(1024));
        assert_eq!(parse_size("10mb"), Some(10_000_000));
        assert_eq!(parse_size("bogus"), None);
        assert_eq!(parse_size("kb"), None);
    }

    #[test]
    fn minimal_scaffold_does_not_scale() {
        assert!(calibrate(Scaffold::Minimal, 0, 100_000).is_err());
    }

    /// Calibration lands near the target across a range of sizes
    /// (small → low-MB), and is deterministic per (size, seed). The fit is
    /// approximate (id/index widths grow with the count), so the band is
    /// generous — a perf tier wants "~50MB", not exactly 50MB.
    #[test]
    fn calibrated_size_is_near_target_and_deterministic() {
        for target in [200_000u64, 1_000_000, 5_000_000] {
            let c = calibrate(Scaffold::Wide, 7, target).unwrap();
            let c2 = calibrate(Scaffold::Wide, 7, target).unwrap();
            assert_eq!(c.n_loca, c2.n_loca, "calibration is deterministic");
            let actual = emit_bytes_w(Scaffold::Wide, 7, c.n_loca, id_width_for(c.n_loca));
            let ratio = actual as f64 / target as f64;
            assert!(
                (0.9..=1.12).contains(&ratio),
                "target {target}: actual {actual} ratio {ratio:.3} (n_loca {})",
                c.n_loca
            );
        }
    }
}
