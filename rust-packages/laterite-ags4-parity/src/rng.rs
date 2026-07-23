//! Seedable PRNG + reservoir sampling — moved verbatim from
//! `laterite-ags4-corpus-qa/src/crawl.rs` so deterministic sampling is shared
//! by `laterite-ags4-corpus-qa` (the `--seed` parity/crawl sample) and
//! `laterite-ags4-forge` (reproducible candidate selection / oracle gating).
//!
//! `reservoir` is generalised to any `T` (was `PathBuf`-specialised);
//! every existing `PathBuf` call site still type-infers unchanged —
//! a strict, behaviour-neutral generalisation.

/// Minimal seedable PRNG (`SplitMix64`) — no `rand` dep. Deterministic
/// when seeded, so reservoir sampling is unit-testable.
pub struct Rng(u64);

impl Rng {
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Rng(seed)
    }
    #[must_use]
    pub fn from_time() -> Self {
        // Entropy for a PRNG seed, not a value anyone reads back: any
        // truncation (only possible once nanos-since-epoch exceeds u64,
        // around the year 2554) still yields a perfectly good seed.
        #[allow(clippy::cast_possible_truncation)]
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64);
        Rng(n ^ 0xD1B5_4A32_D192_ED03)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in `0..n` (n > 0).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// Inclusive uniform integer in `lo..=hi` (`lo <= hi`).
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(lo <= hi);
        lo + self.below((hi - lo + 1) as u64) as i64
    }

    /// A uniformly-chosen reference into `items` (panics on empty —
    /// callers sample from non-empty picklists/word-lists).
    // `below(n)` returns a value < n by construction (`next_u64() % n`), and
    // n is `items.len()` widened to u64, so the result narrows back to
    // usize losslessly.
    #[allow(clippy::cast_possible_truncation)]
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// Algorithm R reservoir sampling — keep `k` items from a stream of
/// unknown length without materialising it.
pub fn reservoir<T, I: Iterator<Item = T>>(iter: I, k: usize, rng: &mut Rng) -> Vec<T> {
    if k == 0 {
        return Vec::new();
    }
    let mut res: Vec<T> = Vec::with_capacity(k);
    for (i, item) in iter.enumerate() {
        if i < k {
            res.push(item);
        } else {
            // `below(n)` returns a value < n by construction, and n is
            // `i + 1` (a usize loop counter) widened to u64, so the result
            // narrows back to usize losslessly.
            #[allow(clippy::cast_possible_truncation)]
            let j = rng.below((i + 1) as u64) as usize;
            if j < k {
                res[j] = item;
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("f{i}")).collect()
    }

    #[test]
    fn reservoir_keeps_min_k_total_and_is_seed_deterministic() {
        let src = items(1000);
        let mut r1 = Rng::seeded(42);
        let a = reservoir(src.clone().into_iter(), 50, &mut r1);
        let mut r2 = Rng::seeded(42);
        let b = reservoir(src.clone().into_iter(), 50, &mut r2);
        assert_eq!(a.len(), 50);
        assert_eq!(a, b, "same seed → same sample");

        // k >= total → keep everything.
        let mut r3 = Rng::seeded(1);
        let all = reservoir(items(10).into_iter(), 50, &mut r3);
        assert_eq!(all.len(), 10);

        // k == 0 → empty.
        let mut r4 = Rng::seeded(1);
        assert!(reservoir(src.into_iter(), 0, &mut r4).is_empty());
    }

    #[test]
    fn reservoir_only_yields_input_items() {
        let src = items(200);
        let mut rng = Rng::seeded(7);
        for p in reservoir(src.clone().into_iter(), 20, &mut rng) {
            assert!(src.contains(&p));
        }
    }

    #[test]
    fn seeded_rng_is_reproducible_and_from_time_differs() {
        let mut a = Rng::seeded(12345);
        let mut b = Rng::seeded(12345);
        let xs: Vec<u64> = (0..8).map(|_| a.below(1_000_000)).collect();
        let ys: Vec<u64> = (0..8).map(|_| b.below(1_000_000)).collect();
        assert_eq!(xs, ys, "same seed ⇒ identical stream");
    }
}
