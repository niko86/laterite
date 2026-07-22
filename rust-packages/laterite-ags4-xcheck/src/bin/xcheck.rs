//! `xcheck` — the COMPARATOR of the cross-surface OUTPUT-VALUE gate (plan
//! `output/output-value-gate-plan.md` §2).
//!
//! Loads the case manifest + one observation file per leg and runs, in order:
//!   (i)   N-way equality across the legs a case declares;
//!   (ii)  LEAF AUTHORITY — the `rust-leaf` column is the reference, so a
//!         one-surface divergence still fails against it (drift #1b);
//!   (iii) absolute spec invariants (`emit_reparses`) — reparse each leg's
//!         emitted bytes with the SAME parse leaf every surface wraps.
//!
//! It does ZERO normalisation: JSON value equality, full stop. Every host-idiom
//! transform happens at the emitter edge, in that surface's own language, where
//! a reviewer can see it. A normaliser inside the comparator is where bugs hide
//! (strip a trailing CRLF "for robustness" and you have blinded the gate to the
//! very drift it exists to catch). Exits non-zero on any unexplained split.
//!
//!   xcheck --check <out-dir> --allow <file> [--cases <dir>] [--repo-root <dir>]
//!          [--require-legs all|present]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[path = "../xcheck_shared.rs"]
mod shared;
use shared::{AUTHORITY, LegObservations, Observation, canonical, load_manifests};

// --- the value allowlist (xcheck-allow.json) -------------------------

#[derive(Deserialize, Debug, Default)]
struct Allowlist {
    #[allow(dead_code)]
    format_version: u32,
    #[serde(default)]
    known_bug_budget: u32,
    #[serde(default)]
    entries: Vec<AllowEntry>,
}

#[derive(Deserialize, Debug)]
struct AllowEntry {
    case: String,
    leg: String,
    /// `by-design` | `host-idiom` | `oracle-parity` | `known-bug`.
    verdict: String,
    /// The FULL observation this entry pins (`{"ok": …}` / `{"err": …}`) — the
    /// leg is still COMPARED, just to this second value instead of the authority.
    /// An entry can therefore never hide a *new* wrong value.
    #[serde(default)]
    value: serde_json::Value,
    #[serde(default)]
    #[allow(dead_code)]
    issue: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    reason: Option<String>,
}

impl Allowlist {
    fn load(path: &Path) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
    }
    fn find(&self, case: &str, leg: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.case == case && e.leg == leg)
    }
}

// --- failures --------------------------------------------------------

struct Failure {
    case: String,
    kind: &'static str,
    detail: String,
}

fn obs_json(o: &Observation) -> serde_json::Value {
    serde_json::to_value(o).expect("observation serialises")
}

/// The char index where two serialised observations first differ (their common
/// length when one is a prefix of the other).
fn first_divergence(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| a.chars().count().min(b.chars().count()))
}

/// A ~160-char window over `s` positioned so `at` is inside it, elided with `…`
/// on whichever side is cut.
///
/// Deliberately NOT a head-truncation. Two long observations that agree for their
/// first 160 chars — a `read --json` render whose only drift is a non-ASCII cell
/// halfway down — print as two IDENTICAL prefixes under a head-cut, directly
/// beneath the word "split". A failure message that shows the matching part is
/// worse than none: it reads as a false alarm and costs the reader the trust the
/// gate exists to earn.
fn window(s: &str, at: usize) -> String {
    const WIDTH: usize = 160;
    const LEAD: usize = 40; // context BEFORE the divergence; the rest trails it
    let n = s.chars().count();
    if n <= WIDTH {
        return s.to_string();
    }
    let start = at.saturating_sub(LEAD).min(n.saturating_sub(WIDTH));
    let body: String = s.chars().skip(start).take(WIDTH).collect();
    let head = if start > 0 { "…" } else { "" };
    let tail = if start + WIDTH < n { "…" } else { "" };
    format!("{head}{body}{tail}")
}

/// Render a split pair with both windows on the first differing char, so the
/// reader sees the DIFFERENCE rather than the shared prefix.
fn split_detail(leg: &str, obs: &Observation, ref_leg: &str, ref_obs: &Observation) -> String {
    let (a, b) = (obs_json(obs).to_string(), obs_json(ref_obs).to_string());
    let at = first_divergence(&a, &b);
    format!(
        "first differ at char {at}: {leg} = {} but {ref_leg} = {}",
        window(&a, at),
        window(&b, at)
    )
}

fn main() {
    let mut out_dir = PathBuf::from("output/xcheck");
    let mut allow_path = PathBuf::from("xcheck-allow.json");
    let mut cases_dir = PathBuf::from("rust-packages/laterite-ags4-xcheck/cases");
    let mut repo_root = PathBuf::from(".");
    let mut require_all = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--check" => out_dir = args.next().map(PathBuf::from).expect("--check <dir>"),
            "--allow" => allow_path = args.next().map(PathBuf::from).expect("--allow <file>"),
            "--cases" => cases_dir = args.next().map(PathBuf::from).expect("--cases <dir>"),
            "--repo-root" => repo_root = args.next().map(PathBuf::from).expect("--repo-root <dir>"),
            "--require-legs" => {
                require_all = args.next().as_deref() == Some("all");
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }

    let fail = |msg: String| -> ! {
        eprintln!("xcheck: {msg}");
        std::process::exit(2);
    };
    let cases = load_manifests(&cases_dir).unwrap_or_else(|e| fail(e));
    let allow = Allowlist::load(&allow_path).unwrap_or_else(|e| fail(e));

    // Load every leg's observation file that exists. A leg's file is looked up
    // once, by the set of leg names any case declares.
    let all_legs: BTreeSet<String> = cases.iter().flat_map(|c| c.legs.iter().cloned()).collect();
    let mut legs: BTreeMap<String, LegObservations> = BTreeMap::new();
    for leg in &all_legs {
        let p = out_dir.join(format!("{leg}.json"));
        if let Ok(text) = std::fs::read_to_string(&p) {
            match serde_json::from_str::<LegObservations>(&text) {
                Ok(o) => {
                    legs.insert(leg.clone(), o);
                }
                Err(e) => fail(format!("parse {}: {e}", p.display())),
            }
        } else { /* absent file: handled per-case below */
        }
    }

    let mut failures: Vec<Failure> = Vec::new();
    let mut used_entries: BTreeSet<usize> = BTreeSet::new();
    // Each case's reference observation, kept for the cross-path equivalence pass
    // (`equivalent_to`): two DIFFERENT ops that should produce the same bytes
    // (e.g. `build_ags4` vs an Excel round-trip).
    let mut case_ref: BTreeMap<String, Observation> = BTreeMap::new();

    // known-bug budget ratchet.
    // Count of a hand-maintained allowlist file's entries — nowhere near u32::MAX.
    #[allow(clippy::cast_possible_truncation)]
    let known_bugs = allow
        .entries
        .iter()
        .filter(|e| e.verdict == "known-bug")
        .count() as u32;
    if known_bugs > allow.known_bug_budget {
        failures.push(Failure {
            case: "(allowlist)".into(),
            kind: "budget",
            detail: format!(
                "{known_bugs} known-bug entries exceed known_bug_budget={}",
                allow.known_bug_budget
            ),
        });
    }

    for case in &cases {
        // Gather each declared leg's observation (if the leg's file is present
        // and it recorded this case).
        let mut present: BTreeMap<&str, &Observation> = BTreeMap::new();
        for leg in &case.legs {
            match legs.get(leg).and_then(|o| o.cases.get(&case.id)) {
                Some(obs) => {
                    present.insert(leg.as_str(), obs);
                }
                None => {
                    // Under --require-legs all a declared leg that produced no
                    // observation is a HARD failure — that is what stops a leg
                    // silently self-skipping in CI (the ags4-perf fate).
                    if require_all {
                        failures.push(Failure {
                            case: case.id.clone(),
                            kind: "missing-leg",
                            detail: format!("leg {leg:?} produced no observation"),
                        });
                    }
                }
            }
        }
        if present.is_empty() {
            continue;
        }

        // (i)+(ii) leaf authority (subsumes N-way when the authority is present:
        // every leg == rust-leaf ⇒ every leg == every leg). Fall back to the
        // first present leg as the reference if the authority didn't run.
        let (ref_leg, ref_obs) = if let Some(o) = present.get(AUTHORITY) {
            (AUTHORITY, *o)
        } else {
            let (l, o) = present.iter().next().unwrap();
            (*l, *o)
        };
        case_ref.insert(case.id.clone(), ref_obs.clone());
        for (leg, obs) in &present {
            if *leg == ref_leg || *obs == ref_obs {
                continue;
            }
            // A split. Is it a PINNED divergence in the allowlist?
            match allow.find(&case.id, leg) {
                Some(i) if allow.entries[i].value == obs_json(obs) => {
                    used_entries.insert(i);
                }
                _ => failures.push(Failure {
                    case: case.id.clone(),
                    kind: "split",
                    detail: split_detail(leg, obs, ref_leg, ref_obs),
                }),
            }
        }

        // (iii) absolute invariants. Not allowlistable in this cut.
        for inv in &case.invariants {
            if inv == "emit_reparses" {
                let Some(fixture) = case.input.fixture.as_ref() else {
                    continue;
                };
                let input_bytes = std::fs::read(repo_root.join(fixture)).unwrap_or_default();
                let input_text = String::from_utf8_lossy(&input_bytes);
                let expected = match canonical(&input_text) {
                    Ok(c) => c,
                    Err(e) => {
                        failures.push(Failure {
                            case: case.id.clone(),
                            kind: "invariant",
                            detail: format!("emit_reparses: input {fixture} did not parse: {e}"),
                        });
                        continue;
                    }
                };
                for (leg, obs) in &present {
                    let Observation::Ok(serde_json::Value::String(emitted)) = obs else {
                        continue; // err/absent/non-string: nothing to re-parse
                    };
                    match canonical(emitted) {
                        Ok(got) if got == expected => {}
                        Ok(_) => failures.push(Failure {
                            case: case.id.clone(),
                            kind: "invariant",
                            detail: format!(
                                "emit_reparses: {leg}'s output re-parses to a different group/row shape than the input"
                            ),
                        }),
                        Err(e) => failures.push(Failure {
                            case: case.id.clone(),
                            kind: "invariant",
                            detail: format!("emit_reparses: {leg}'s output did not re-parse: {e}"),
                        }),
                    }
                }
            }
        }
    }

    // (iv) declared cross-path equivalence. A case may assert its bytes equal a
    // SIBLING case's — two different ops that should agree (`build_ags4` bytes vs
    // an Excel round-trip). This is the one bug class N-way surface equality is
    // structurally blind to: each formatter is one shared Rust fn called
    // identically by every surface, so every leg agrees with itself forever while
    // the two PATHS disagree. A divergence is pinned with the sentinel leg
    // `@equivalent_to` (verdict `by-design` — the Excel round-trip's lossy 3SF
    // formatting is accepted, gated at the formatter level by #517).
    for case in &cases {
        let Some(target) = &case.equivalent_to else {
            continue;
        };
        let (Some(b), Some(a)) = (case_ref.get(&case.id), case_ref.get(target)) else {
            // A side is absent; the missing-leg check already accounts for it.
            continue;
        };
        if a == b {
            continue;
        }
        match allow.find(&case.id, "@equivalent_to") {
            Some(i) if allow.entries[i].value == obs_json(b) => {
                used_entries.insert(i);
            }
            _ => failures.push(Failure {
                case: case.id.clone(),
                kind: "equivalence",
                detail: format!(
                    "output differs from `{target}` (declared equivalent): {}",
                    split_detail("this", b, target, a)
                ),
            }),
        }
    }

    // Staleness: an allowlist entry that never pinned a real divergence is dead
    // weight — a fix PR is FORCED to delete its own entry (a visible burn-down).
    for (i, e) in allow.entries.iter().enumerate() {
        if !used_entries.contains(&i) {
            failures.push(Failure {
                case: e.case.clone(),
                kind: "stale-allowlist",
                detail: format!(
                    "entry (leg {:?}, verdict {:?}) did not reproduce — delete it or fix its value",
                    e.leg, e.verdict
                ),
            });
        }
    }

    if failures.is_empty() {
        let n_cases = cases.len();
        let n_legs = legs.len();
        eprintln!("xcheck: OK — {n_cases} case(s) across {n_legs} leg(s), no unexplained split");
        return;
    }

    eprintln!("xcheck: {} FAILURE(S)", failures.len());
    for f in &failures {
        eprintln!("  [{}] {}: {}", f.kind, f.case, f.detail);
    }
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_divergence_finds_the_differing_char_and_the_prefix_boundary() {
        assert_eq!(first_divergence("abcX", "abcY"), 3);
        // One a prefix of the other: the divergence is where the shorter ends.
        assert_eq!(first_divergence("abc", "abcdef"), 3);
        assert_eq!(first_divergence("same", "same"), 4);
    }

    #[test]
    fn a_split_beyond_the_window_still_shows_the_difference() {
        // The regression this guards: two renders agreeing for 300 chars and
        // differing after. A head-truncation printed two IDENTICAL excerpts.
        let a = format!("{}CAFÉ{}", "x".repeat(300), "y".repeat(300));
        let b = format!("{}CAF?{}", "x".repeat(300), "y".repeat(300));
        let at = first_divergence(&a, &b);
        let (wa, wb) = (window(&a, at), window(&b, at));
        assert_ne!(wa, wb, "the two windows must not print identically");
        assert!(wa.contains('É') && wb.contains('?'));
    }

    #[test]
    fn a_short_observation_is_shown_whole_and_unelided() {
        let s = "{\"ok\":\"BH01\"}";
        assert_eq!(window(s, 0), s);
    }

    #[test]
    fn a_divergence_at_the_very_end_is_not_windowed_off_the_edge() {
        // `start` is clamped so the window never runs past the string: a drift in
        // the LAST char (a missing trailing newline) must still be visible.
        let a = format!("{}A", "x".repeat(400));
        let b = format!("{}B", "x".repeat(400));
        let at = first_divergence(&a, &b);
        assert!(window(&a, at).contains('A') && window(&b, at).contains('B'));
    }
}
