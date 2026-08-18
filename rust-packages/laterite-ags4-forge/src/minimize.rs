//! ddmin minimization — shrink a divergence-producing candidate to a
//! minimal reproducer **before** it is reported, so a finding becomes
//! a clean wiki probe (`ags-wiki/.bootstrap/probes/`, never
//! `tests/fixtures/`).
//!
//! Two granularities, in order: (1) line-level delta-debugging,
//! (2) within survivors, drop trailing DATA fields / blank values.
//! **Invariant**: a reduction is accepted only if the dual-validate
//! signature is *unchanged* — the same `Parity::tag()` **and** the
//! same rust rule-set **and** the same python rule-set (not merely
//! "still a finding") — so one divergence can't be minimized into a
//! different one. Deterministic (no RNG); re-running yields the
//! identical minimal reproducer.

use std::sync::atomic::{AtomicU64, Ordering};

use laterite_ags4_parity::PyOracle;

use crate::pipeline::{Outcome, dual_validate};

/// The signature a reduction must preserve.
pub type Sig = (String, Vec<String>, Vec<String>);

pub fn sig_of(o: &Outcome) -> Sig {
    let tag = match &o.verdict {
        Some(p) => p.tag().to_string(),
        None => "RUST_ONLY".to_string(),
    };
    let mut r = o.rust_rules();
    r.sort();
    let mut p = o.python_rules();
    p.sort();
    (tag, r, p)
}

fn scratch() -> std::path::PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("forge_min_{}_{n}.ags", std::process::id()))
}

/// Dual-validate `text` (writing a scratch file the python wrapper can
/// read) and return its signature.
fn eval(text: &str, oracle: Option<&PyOracle>) -> Sig {
    let p = scratch();
    let _ = std::fs::write(&p, text.as_bytes());
    let o = dual_validate(&p, oracle);
    let _ = std::fs::remove_file(&p);
    sig_of(&o)
}

/// Standard ddmin over the line list, preserving `target`.
fn ddmin_lines(lines: Vec<String>, target: &Sig, oracle: Option<&PyOracle>) -> Vec<String> {
    let mut cur = lines;
    let mut n = 2usize;
    while cur.len() >= 2 {
        let chunk = cur.len().div_ceil(n);
        let mut reduced = false;
        let mut i = 0;
        while i < cur.len() {
            // The complement of subset [i, i+chunk).
            let mut cand: Vec<String> = Vec::with_capacity(cur.len());
            cand.extend_from_slice(&cur[..i]);
            if i + chunk < cur.len() {
                cand.extend_from_slice(&cur[i + chunk..]);
            }
            if !cand.is_empty() && eval(&join(&cand), oracle) == *target {
                cur = cand;
                n = (n - 1).max(2);
                reduced = true;
                break;
            }
            i += chunk;
        }
        if !reduced {
            if n >= cur.len() {
                break;
            }
            n = (n * 2).min(cur.len());
        }
    }
    cur
}

/// Field pass: for each surviving DATA line, try blanking each value
/// (right-to-left) while the signature holds.
fn shrink_fields(mut lines: Vec<String>, target: &Sig, oracle: Option<&PyOracle>) -> Vec<String> {
    for li in 0..lines.len() {
        if !lines[li].starts_with("\"DATA\"") {
            continue;
        }
        loop {
            let parts: Vec<&str> = lines[li].split(',').collect();
            if parts.len() <= 2 {
                break;
            }
            // Try dropping the last field.
            let trimmed = parts[..parts.len() - 1].join(",");
            let mut probe = lines.clone();
            probe[li].clone_from(&trimmed);
            if eval(&join(&probe), oracle) == *target {
                lines[li] = trimmed;
            } else {
                break;
            }
        }
    }
    lines
}

fn join(lines: &[String]) -> String {
    let mut s = lines.join("\r\n");
    s.push_str("\r\n");
    s
}

/// Minimize `text`, preserving its current dual-validate signature.
/// Returns `(minimal_text, signature)`. If the input's signature can't
/// be re-observed (flaky oracle), returns the input unchanged.
pub fn minimize(text: &str, oracle: Option<&PyOracle>) -> (String, Sig) {
    let target = eval(text, oracle);
    // Blanks are kept — group separators (Rule 2a). The split alone
    // already preserves them, so no filter needed.
    let lines: Vec<String> = text
        .split("\r\n")
        .map(std::string::ToString::to_string)
        .collect();
    let l = ddmin_lines(lines, &target, oracle);
    let l = shrink_fields(l, &target, oracle);
    let out = join(&l);
    // Final guard: never return something off-target.
    if eval(&out, oracle) == target {
        (out, target)
    } else {
        (text.to_string(), target)
    }
}

/// Pre-fill an `insights/` + drafted-`O-NN` stub for the §12.5 flow.
/// The CLI only *drafts* (it embeds no LLM and never writes
/// `OBSERVATIONS.md`); the author reviews, fills judgement, and writes
/// the ratified O-N deliberately.
pub fn insight_stub(sig: &Sig, injection: &str, repro_rel: &str, next_obs: &str) -> String {
    let (tag, rust, py) = sig;
    format!(
        "---\ntype: insight\nstatus: hypothesis\ngap_kind: rust-vs-python\n\
         proposes_observation: true\nrules: []\n---\n\n\
         # forge finding — {injection} → {tag}\n\n\
         > [!divergence] laterite-ags4-forge surfaced this via Mode-B \
         synthesize+inject; minimized by ddmin (signature-preserving).\n\n\
         - **rust**: `{rust:?}`\n- **python**: `{py:?}`\n\
         - **verdict**: {tag}\n- **evidence**: `{repro_rel}` (run through \
         both `lat` and `tools/py_ags4_check_json.py`, cwd repo root)\n\n\
         ## Drafted OBSERVATIONS entry (review before ratifying)\n\n\
         ### {next_obs} [TAG] <title>\n\
         - **Observed / where**: forge {injection}; minimal repro `{repro_rel}`.\n\
         - **Spec (§…)**: <fill>\n\
         - **Assessment**: <is this a new gap, or the known O-35 \
         presence-only cascade / O-3-narrow? recognise before opening O-N>\n\
         - **Upstream-reportable**: <yes/no>\n\
         - **Our decision**: <reconcile arm? doc-only? validator change?>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{Injection, synth_injected};
    use crate::synth::Scaffold;

    #[test]
    fn ddmin_preserves_signature_and_is_deterministic() {
        // Rust-only predicate (oracle=None) → fully deterministic.
        let dirty = synth_injected(Scaffold::LocaSamp, 7, Injection::DupSampKeyTuple);
        let before = eval(&dirty, None);
        // Sanity: the injected file really is a Rule-10a finding.
        assert!(
            before.1.iter().any(|r| r == "AGS Format Rule 10a"),
            "precondition: rust flags 10a, got {:?}",
            before.1
        );
        let (m1, s1) = minimize(&dirty, None);
        let (m2, s2) = minimize(&dirty, None);
        assert_eq!(m1, m2, "ddmin must be deterministic");
        assert_eq!(s1, before, "signature must be preserved");
        assert_eq!(s2, before);
        assert!(
            m1.len() <= dirty.len(),
            "minimized must not grow ({} vs {})",
            m1.len(),
            dirty.len()
        );
        // The minimal repro still reproduces the exact signature.
        assert_eq!(eval(&m1, None), before);
    }
}
