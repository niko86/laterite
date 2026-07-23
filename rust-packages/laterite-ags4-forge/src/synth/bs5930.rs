//! A constraint-valid **BS 5930:2015+A1:2020 Section 6** soil-description
//! generator — the realistic `GEOL_DESC` engine.
//!
//! The open vocabularies (strength/consistency, relative density, colour,
//! particle angularity, size sub-bands) are parsed from the vendored skill
//! data under `data/bs5930/` — see that dir's `PROVENANCE.md` — so they
//! stay synced to the owner's `solmek-field-app` source. The *constraint
//! engine* (the secondary-constituent proportion bands of Tables 16/17,
//! the cumulative-≤100% rule, the silty/clayey mutual exclusion, the
//! coarse-then-fine word order, the colour lightness·chroma·hue order) is
//! encoded here in Rust, citing `proportion-rules.json`.
//!
//! The variety the owner asked for is **combinatorial**, not a finite list
//! of canned strings: principal × strength/density × (lightness × chroma ×
//! hue) × secondary constituents × proportion bands gives millions of
//! distinct, *standard-compliant* descriptions. The percentages are drawn
//! first, respecting the cumulative rule **by construction**, then mapped
//! to their band qualifier — so a generated description can never say the
//! impossible "very sandy very gravelly CLAY".
//!
//! Scope (v1): natural inorganic coarse (SAND/GRAVEL) and fine (SILT/CLAY)
//! soils — the two lanes that share the standard word order. Very coarse
//! (BOULDERS/COBBLES), peat/organic and anthropogenic ground use different
//! apportionment/word orders and are left for later (as the source skill
//! itself stages them).

use laterite_ags4_parity::Rng;
use serde::Deserialize;

// Vendored skill data (data/bs5930/PROVENANCE.md). Parsed once into `Vocab`.
const TERMS_JSON: &str = include_str!("../../data/bs5930/terms.json");
const PARTICLE_SIZES_JSON: &str = include_str!("../../data/bs5930/particle-sizes.json");

/// Hues with no chroma form (BS 5930 Table 7 note): a chroma word like
/// "reddish" must never precede these.
const ACHROMATIC_HUES: &[&str] = &["white", "grey", "black", "cream"];

/// A `terms.json` entry — only the fields the generator reads (serde
/// ignores the rest).
#[derive(Deserialize)]
struct Term {
    #[serde(rename = "term")]
    text: String,
    #[serde(default)]
    group: Option<String>,
    #[serde(default, rename = "cuKpa")]
    cu_kpa: Option<String>,
}

#[derive(Deserialize)]
struct Terms {
    #[serde(rename = "strengthConsistency")]
    strength_consistency: Vec<Term>,
    #[serde(rename = "relativeDensity")]
    relative_density: Vec<Term>,
    colour: Vec<Term>,
    angularity: Vec<Term>,
}

#[derive(Deserialize)]
struct Subdiv {
    name: String,
}
#[derive(Deserialize)]
struct Fraction {
    name: String,
    #[serde(default)]
    subdivisions: Vec<Subdiv>,
}
#[derive(Deserialize)]
struct ParticleSizes {
    fractions: Vec<Fraction>,
}

/// The parsed BS 5930 vocabularies (built once via [`Vocab::load`]).
pub struct Vocab {
    /// Table-8 hand-test consistency terms (Very soft … Very stiff) — the
    /// leading word for a fine soil. (The cu-banded Table-9 terms are
    /// excluded.)
    consistency: Vec<String>,
    /// Relative-density terms (Very loose … Very dense) — the leading word
    /// for a coarse soil.
    density: Vec<String>,
    lightness: Vec<String>,
    chroma: Vec<String>,
    hue: Vec<String>,
    /// Particle angularity (subangular …) — used in the grading sentence.
    angularity: Vec<String>,
    /// SAND/GRAVEL size sub-bands (fine/medium/coarse).
    grading: Vec<String>,
}

impl Vocab {
    /// Parse the vendored skill data. Panics only if the in-repo JSON is
    /// malformed (a build-time guarantee, asserted by the tests).
    pub fn load() -> Vocab {
        let t: Terms = serde_json::from_str(TERMS_JSON).expect("vendored terms.json is valid");
        let ps: ParticleSizes = serde_json::from_str(PARTICLE_SIZES_JSON)
            .expect("vendored particle-sizes.json is valid");
        let by_group = |g: &str| -> Vec<String> {
            t.colour
                .iter()
                .filter(|c| c.group.as_deref() == Some(g))
                .map(|c| c.text.clone())
                .collect()
        };
        Vocab {
            // The Table-8 consistency terms carry no cu band.
            consistency: t
                .strength_consistency
                .iter()
                .filter(|x| x.cu_kpa.is_none())
                .map(|x| x.text.clone())
                .collect(),
            density: t.relative_density.iter().map(|x| x.text.clone()).collect(),
            lightness: by_group("lightness"),
            chroma: by_group("chroma"),
            hue: by_group("hue"),
            angularity: t.angularity.iter().map(|x| x.text.clone()).collect(),
            grading: ps
                .fractions
                .iter()
                .find(|f| f.name == "SAND")
                .map(|f| f.subdivisions.iter().map(|s| s.name.clone()).collect())
                .unwrap_or_default(),
        }
    }
}

/// Which lane of the standard word order a description follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalClass {
    /// SAND / GRAVEL — leading term is relative density.
    Coarse,
    /// SILT / CLAY — leading term is consistency.
    Fine,
}

/// One secondary constituent, with the field-estimate percentage it was
/// drawn at and the band qualifier that percentage maps to.
#[derive(Debug, Clone)]
pub struct Secondary {
    /// Lowercase fraction adjective stem (`sand`/`gravel`/`silt`/`clay`).
    pub soil: &'static str,
    /// Visual field-estimate percentage by mass (excl. cobbles/boulders);
    /// `0` for a qualitative fine-in-fine secondary.
    pub pct: u32,
    /// `slightly` / "" (plain) / `very`; empty for a qualitative fine-in-fine.
    pub qualifier: &'static str,
}

impl Secondary {
    /// Whether this is a fine fraction (silt/clay) vs a coarse one
    /// (sand/gravel) — derived from `soil`.
    pub fn is_fine(&self) -> bool {
        matches!(self.soil, "silt" | "clay")
    }

    /// The adjective as it appears in the description (`very gravelly`,
    /// `slightly clayey`, `silty`).
    fn adjective(&self) -> String {
        let stem = match self.soil {
            "sand" => "sandy",
            "gravel" => "gravelly",
            "silt" => "silty",
            "clay" => "clayey",
            other => other,
        };
        if self.qualifier.is_empty() {
            stem.to_string()
        } else {
            format!("{} {stem}", self.qualifier)
        }
    }
}

/// A generated, standard-compliant soil description: the assembled text
/// plus the structured fields the constraint tests assert on.
#[derive(Debug, Clone)]
pub struct SoilDescription {
    pub text: String,
    pub principal: &'static str,
    pub principal_class: PrincipalClass,
    /// Coarse secondaries first, then any fine secondary (file order).
    pub secondaries: Vec<Secondary>,
}

/// Tables 16/17 for a **coarse** principal (SAND/GRAVEL): a secondary —
/// coarse or fine — bands at `<5` / `5–20` / `>20`.
fn band_coarse_principal(pct: u32) -> &'static str {
    if pct < 5 {
        "slightly"
    } else if pct <= 20 {
        ""
    } else {
        "very"
    }
}

/// Table 17 'Coarse soil' column for a **fine** principal (SILT/CLAY): a
/// coarse secondary bands at `<35` / `35–65` / `>65`.
fn band_fine_principal(pct: u32) -> &'static str {
    if pct < 35 {
        "slightly"
    } else if pct <= 65 {
        ""
    } else {
        "very"
    }
}

/// The base hue a chroma word modifies (`greenish` → `green`), so a chroma
/// is never paired with its own hue ("greenish green").
fn chroma_base(chroma: &str) -> &str {
    match chroma {
        "pinkish" => "pink",
        "reddish" => "red",
        "orangeish" => "orange",
        "yellowish" => "yellow",
        "brownish" => "brown",
        "greenish" => "green",
        "bluish" => "blue",
        "greyish" => "grey",
        other => other,
    }
}

/// Assemble a colour per BS 5930 Table 7 word order — `lightness? chroma?
/// hue`. A chroma word is only emitted before a hue that has a chroma form
/// (not white/grey/black/cream) and is a *different* colour family;
/// lightness is skipped for the lightness extremes themselves (no
/// "light black" / "dark white").
fn colour(vocab: &Vocab, rng: &mut Rng) -> String {
    let hue = rng.choose(&vocab.hue).clone();
    let mut parts: Vec<String> = Vec::new();
    // ~40% carry a lightness, except on black/white (contradictory).
    if !vocab.lightness.is_empty() && !matches!(hue.as_str(), "black" | "white") && rng.below(5) < 2
    {
        parts.push(rng.choose(&vocab.lightness).clone());
    }
    // Chroma only for chromatic hues, and only from a different family.
    if !ACHROMATIC_HUES.contains(&hue.as_str()) && rng.below(2) == 0 {
        let choices: Vec<String> = vocab
            .chroma
            .iter()
            .filter(|c| chroma_base(c) != hue)
            .cloned()
            .collect();
        if !choices.is_empty() {
            parts.push(rng.choose(&choices).clone());
        }
    }
    parts.push(hue);
    parts.join(" ")
}

/// `true` with probability `pct/100`.
fn chance(rng: &mut Rng, pct: u64) -> bool {
    rng.below(100) < pct
}

/// Describe a coarse soil (SAND/GRAVEL). Secondaries (the other coarse
/// fraction + an optional fine fraction) are kept to ≤45% total so the
/// principal stays the dominant fraction (cumulative ≤100% by construction).
// `rng.range(1, hi) as u32` below always returns a value in `[1, hi]`, and
// `hi` is `budget.min(40)` — so the result is always in `1..=40`, comfortably
// inside u32.
#[allow(clippy::cast_possible_truncation)]
fn describe_coarse(vocab: &Vocab, rng: &mut Rng) -> SoilDescription {
    let coarse_pair = [("SAND", "gravel"), ("GRAVEL", "sand")];
    let (principal, other_coarse) = *rng.choose(&coarse_pair);

    let mut secondaries = Vec::new();
    let mut budget = 45u32; // headroom for both secondaries
    // Coarse secondary (the other coarse fraction).
    if chance(rng, 60) {
        let pct = rng.range(1, i64::from(budget.min(40))) as u32;
        budget -= pct.min(budget);
        secondaries.push(Secondary {
            soil: other_coarse,
            pct,
            qualifier: band_coarse_principal(pct),
        });
    }
    // Fine secondary — silt OR clay, never both (mutual exclusion).
    if budget >= 1 && chance(rng, 55) {
        let fine = *rng.choose(&["silt", "clay"]);
        let pct = rng.range(1, i64::from(budget.min(40))) as u32;
        secondaries.push(Secondary {
            soil: fine,
            pct,
            qualifier: band_coarse_principal(pct),
        });
    }

    let density = rng.choose(&vocab.density).clone();
    let col = colour(vocab, rng);
    let mut words = vec![density, col];
    // Coarse-then-fine order is already the push order above.
    for s in &secondaries {
        words.push(s.adjective());
    }
    words.push(principal.to_string());
    let mut text = words.join(" ");
    text.push('.');
    // Optional grading sentence for the coarse fraction.
    if !vocab.grading.is_empty() && !vocab.angularity.is_empty() && chance(rng, 50) {
        text.push_str(&grading_sentence(principal, vocab, rng));
    }

    SoilDescription {
        text,
        principal,
        principal_class: PrincipalClass::Coarse,
        secondaries,
    }
}

/// Describe a fine soil (SILT/CLAY). The coarse secondaries (sandy and/or
/// gravelly) are assessed **separately**, each against Table 17, but their
/// combined percentage is held ≤80% so the cumulative-≤100% rule holds and
/// "very + very" is structurally impossible. A fine-in-fine secondary
/// (silty CLAY / clayey SILT) is qualitative — no percentage, no qualifier.
// `rng.range(1, budget.min(72)) as u32` below always returns a value in
// `[1, 72]` (the `budget >= 1` guard rules out an empty/invalid range),
// comfortably inside u32.
#[allow(clippy::cast_possible_truncation)]
fn describe_fine(vocab: &Vocab, rng: &mut Rng) -> SoilDescription {
    let fine_pair = [("CLAY", "silt"), ("SILT", "clay")];
    let (principal, other_fine) = *rng.choose(&fine_pair);

    // Draw the two coarse secondaries with a shared ≤80% budget.
    let mut coarse: Vec<Secondary> = Vec::new();
    let mut budget = 80i64;
    let sand_first = rng.below(2) == 0;
    let order: [&'static str; 2] = if sand_first {
        ["sand", "gravel"]
    } else {
        ["gravel", "sand"]
    };
    for soil in order {
        if budget >= 1 && chance(rng, 55) {
            let pct = rng.range(1, budget.min(72)) as u32;
            budget -= i64::from(pct);
            coarse.push(Secondary {
                soil,
                pct,
                qualifier: band_fine_principal(pct),
            });
        }
    }
    // Order the coarse secondaries by increasing proportion (word-order rule).
    coarse.sort_by_key(|s| s.pct);

    let mut secondaries = coarse;
    // Optional fine-in-fine (qualitative; mutually exclusive by construction).
    if chance(rng, 35) {
        secondaries.push(Secondary {
            soil: other_fine,
            pct: 0,
            qualifier: "",
        });
    }

    let consistency = rng.choose(&vocab.consistency).clone();
    let col = colour(vocab, rng);
    let mut words = vec![consistency, col];
    for s in &secondaries {
        words.push(s.adjective());
    }
    words.push(principal.to_string());
    let mut text = words.join(" ");
    text.push('.');

    SoilDescription {
        text,
        principal,
        principal_class: PrincipalClass::Fine,
        secondaries,
    }
}

/// A trailing grading sentence, e.g. " Gravel is subangular fine to medium."
// `below(n)` returns a value < n by construction, and every n here is a
// `Vec::len()` widened to u64, so each narrowing back to usize is lossless.
#[allow(clippy::cast_possible_truncation)]
fn grading_sentence(principal: &str, vocab: &Vocab, rng: &mut Rng) -> String {
    let ang = rng.choose(&vocab.angularity).clone();
    // "fine", "fine to medium", or "fine to coarse" style spans.
    let lo = rng.below(vocab.grading.len() as u64) as usize;
    let hi =
        (lo + rng.below((vocab.grading.len() - lo) as u64) as usize).min(vocab.grading.len() - 1);
    let sizes = if lo == hi {
        vocab.grading[lo].clone()
    } else {
        format!("{} to {}", vocab.grading[lo], vocab.grading[hi])
    };
    let fraction = if principal == "GRAVEL" {
        "Gravel"
    } else {
        "Sand"
    };
    format!(" {fraction} is {ang} {sizes}.")
}

/// The process-wide vocabulary, parsed once from the vendored JSON — so a
/// strata-heavy synthesis (many `describe` calls) pays the parse cost a
/// single time.
pub fn vocab() -> &'static Vocab {
    static V: std::sync::OnceLock<Vocab> = std::sync::OnceLock::new();
    V.get_or_init(Vocab::load)
}

/// Generate one constraint-valid BS 5930 soil description for `rng`.
pub fn describe(vocab: &Vocab, rng: &mut Rng) -> SoilDescription {
    if rng.below(2) == 0 {
        describe_coarse(vocab, rng)
    } else {
        describe_fine(vocab, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab() -> Vocab {
        Vocab::load()
    }

    /// The vendored JSON parses and yields non-empty vocabularies — guards
    /// the data files + the serde shapes.
    #[test]
    fn vocab_loads_non_empty() {
        let v = vocab();
        assert!(!v.consistency.is_empty(), "consistency terms");
        assert!(!v.density.is_empty(), "density terms");
        assert!(!v.hue.is_empty() && !v.chroma.is_empty() && !v.lightness.is_empty());
        assert!(!v.angularity.is_empty() && !v.grading.is_empty());
        // The cu-banded Table-9 strength terms are excluded.
        assert!(v.consistency.iter().any(|t| t == "Firm"));
        assert!(!v.consistency.iter().any(|t| t.contains("strength")));
    }

    /// THE constraint: secondary fractions never sum past 100% (the
    /// cumulative rule, clause 33.4.4.5 NOTE 2) — across many seeds.
    #[test]
    fn cumulative_fractions_never_exceed_100() {
        let v = vocab();
        for seed in 0..500u64 {
            let mut rng = Rng::seeded(seed);
            let d = describe(&v, &mut rng);
            let sum: u32 = d.secondaries.iter().map(|s| s.pct).sum();
            assert!(
                sum <= 100,
                "seed {seed}: secondaries sum {sum}% > 100: {d:?}"
            );
        }
    }

    /// silty and clayey are mutually exclusive — never both present.
    #[test]
    fn fine_secondaries_are_mutually_exclusive() {
        let v = vocab();
        for seed in 0..500u64 {
            let mut rng = Rng::seeded(seed);
            let d = describe(&v, &mut rng);
            let fines = d.secondaries.iter().filter(|s| s.is_fine()).count();
            assert!(fines <= 1, "seed {seed}: {fines} fine secondaries: {d:?}");
        }
    }

    /// Every secondary's qualifier is exactly the band its percentage maps
    /// to for the principal's class — the percentage-first invariant.
    #[test]
    fn qualifier_matches_band() {
        let v = vocab();
        for seed in 0..500u64 {
            let mut rng = Rng::seeded(seed);
            let d = describe(&v, &mut rng);
            for s in &d.secondaries {
                if s.pct == 0 {
                    // qualitative fine-in-fine
                    assert!(s.qualifier.is_empty() && s.is_fine());
                    continue;
                }
                let expected = match d.principal_class {
                    // A fine principal bands its coarse secondaries on 35/65;
                    // a coarse principal bands everything on 5/20.
                    PrincipalClass::Fine if !s.is_fine() => band_fine_principal(s.pct),
                    _ => band_coarse_principal(s.pct),
                };
                assert_eq!(s.qualifier, expected, "seed {seed} {s:?} in {d:?}");
            }
        }
    }

    /// The impossible "very sandy very gravelly CLAY" can never be drawn
    /// (two coarse 'very' secondaries on a fine principal need >130%).
    #[test]
    fn no_double_very_coarse_on_a_fine_principal() {
        let v = vocab();
        for seed in 0..1000u64 {
            let mut rng = Rng::seeded(seed);
            let d = describe(&v, &mut rng);
            if d.principal_class == PrincipalClass::Fine {
                let verys = d
                    .secondaries
                    .iter()
                    .filter(|s| !s.is_fine() && s.qualifier == "very")
                    .count();
                assert!(
                    verys <= 1,
                    "seed {seed}: two 'very' coarse secondaries: {d:?}"
                );
            }
        }
    }

    /// Word order: coarse secondaries precede the fine secondary, and all
    /// secondaries precede the (uppercase) principal.
    #[test]
    fn word_order_coarse_then_fine_then_principal() {
        let v = vocab();
        for seed in 0..300u64 {
            let mut rng = Rng::seeded(seed);
            let d = describe(&v, &mut rng);
            assert_eq!(d.principal, d.principal.to_uppercase());
            let p = d.text.find(d.principal).expect("principal in text");
            let mut last_coarse = 0usize;
            let mut first_fine = usize::MAX;
            for s in &d.secondaries {
                let stem = match s.soil {
                    "sand" => "sandy",
                    "gravel" => "gravelly",
                    "silt" => "silty",
                    "clay" => "clayey",
                    o => o,
                };
                let at = d.text.find(stem).expect("secondary adjective in text");
                assert!(at < p, "secondary '{stem}' must precede principal: {d:?}");
                if s.is_fine() {
                    first_fine = first_fine.min(at);
                } else {
                    last_coarse = last_coarse.max(at);
                }
            }
            if first_fine != usize::MAX {
                assert!(
                    last_coarse <= first_fine,
                    "coarse secondaries must precede the fine one: {d:?}"
                );
            }
        }
    }

    /// Colour never emits a chroma word before an achromatic hue, never a
    /// redundant same-family chroma ("greenish green"), and never lightness
    /// on the lightness extremes ("light black" / "dark white").
    #[test]
    fn colour_respects_achromatic_hues() {
        let v = vocab();
        for seed in 0..2000u64 {
            let mut rng = Rng::seeded(seed);
            let c = colour(&v, &mut rng);
            for h in ACHROMATIC_HUES {
                for ch in &v.chroma {
                    assert!(
                        !c.contains(&format!("{ch} {h}")),
                        "chroma '{ch}' before achromatic '{h}': {c:?}"
                    );
                }
            }
            for ch in &v.chroma {
                assert!(
                    !c.contains(&format!("{ch} {}", chroma_base(ch))),
                    "redundant same-family chroma: {c:?}"
                );
            }
            assert!(
                !c.contains("light black")
                    && !c.contains("dark black")
                    && !c.contains("light white")
                    && !c.contains("dark white"),
                "contradictory lightness on black/white: {c:?}"
            );
        }
    }

    /// Determinism: a seed pins the bytes; different seeds genuinely vary.
    #[test]
    fn deterministic_and_varied() {
        let v = vocab();
        let one = |s: u64| describe(&v, &mut Rng::seeded(s)).text;
        assert_eq!(one(42), one(42), "same seed → identical");
        let distinct: std::collections::HashSet<_> = (0..50).map(one).collect();
        assert!(
            distinct.len() > 30,
            "50 seeds should give many distinct texts"
        );
    }
}
