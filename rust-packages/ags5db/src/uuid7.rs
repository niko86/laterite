//! UUID7 minting — port of `ags5_db._uuid`.
//!
//! Every writer row gets a UUID7 surrogate primary key. UUID7 has a
//! millisecond time prefix followed by random bits, so:
//!
//!   * IDs are sortable by creation time (writes append in order).
//!   * Cross-write merge stays cheap — the same AGS KEY tuple resolves
//!     to the *same* UUID via the dedup index (this module just mints
//!     fresh IDs; dedup lives in `writer.rs`).
//!
//! Python uses the `uuid-utils` Rust binding under the hood; here we
//! pull the same algorithm from the `uuid` crate's `v7` feature.

use uuid::Uuid;

/// Mint a fresh UUID7. Monotonic within a millisecond is *not* guaranteed
/// across separate calls — but the random tail (62 bits) makes within-
/// millisecond collisions astronomically unlikely.
pub fn mint() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_uuids_differ() {
        let a = mint();
        let b = mint();
        assert_ne!(a, b);
    }

    #[test]
    fn uuid7_is_sortable_by_time() {
        // UUID7's first 48 bits are unix-millis big-endian, so
        // chronologically-ordered mints sort lexicographically.
        let earlier = mint();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let later = mint();
        assert!(
            earlier < later,
            "later UUID should sort after earlier: {earlier} vs {later}",
        );
    }

    #[test]
    fn version_is_7() {
        let id = mint();
        // The version nibble is bits 48..52. uuid crate exposes it
        // directly.
        assert_eq!(id.get_version_num(), 7);
    }
}
