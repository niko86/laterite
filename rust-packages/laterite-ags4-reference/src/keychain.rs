//! Deterministic content-addressed keys for AGS rows.
//!
//! Every AGS row gets an `id` derived *purely from its identifying keys* —
//! a UUIDv8 over the SHA-256 of the row's spec key-chain. A child's
//! `parent_id` is the same function applied to the **parent's** key-chain,
//! which the child carries denormalised (an AGS4 child row repeats every
//! ancestor KEY — `SAMP` carries `LOCA_ID`, `TREG` carries the whole
//! `LOCA`/`SAMP` key tuple, …). So `child.parent_id == parent.id` **by
//! construction**, with no shared state: two independent reads of the same
//! file agree on every id, which is exactly what lets a DuckDB table
//! function join `read_ags(f,'SAMP')` to `read_ags(f,'LOCA')` across
//! separate calls — and what would let two separately-written database
//! files merge with no reconciliation (contrast a writer using random
//! UUID7 + lookup-table keys, which *cannot* agree across
//! calls because the keys are minted from the clock + RNG).
//!
//! **Why hash the raw string, never the parsed value.** The parent stores a
//! `2DP` `SAMP_TOP` as the bytes `"1.50"`; the child denormalises the same
//! bytes. Hashing the parsed `f64` would be non-deterministic across
//! formatters and could split identity (`1.5` vs `1.50`). The key is the
//! producer's bytes, trimmed — matching `convert.rs::encode_shared_tuple`.
//!
//! This module is pure (registry + strings + `sha2` + `uuid`) — no DuckDB,
//! no clock, no RNG — so it is exhaustively unit-testable and shareable by
//! every host.

use std::collections::HashMap;
use std::hash::BuildHasher;

use laterite_ags4_types::parse_value;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::union::{GroupDescriptor, Registry};

/// The KEY heading names of `g`, in declaration order — the ONE definition of
/// "what identifies a row", so the content-addressed `_id` (via
/// [`key_chain_values`]) and `laterite-ags4-diff`'s row matcher read the same
/// list instead of each re-deriving it. Row-identity lives here, once.
#[must_use]
pub fn key_heading_names(g: &GroupDescriptor) -> Vec<&str> {
    g.key_headings().map(|h| h.name.as_str()).collect()
}

/// The row's own key-chain: `(heading, raw value)` for each KEY heading of
/// `g`, in declaration order. `g.key_headings()` is already denormalised
/// (ancestor KEYs first, then own — e.g. `SAMP` →
/// `[LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID]`), so this *is* the
/// full identifying tuple. Missing values resolve to `""` so a partial-key
/// row can't silently alias a complete one.
#[must_use]
pub fn key_chain_values<S: BuildHasher>(
    g: &GroupDescriptor,
    row: &HashMap<String, String, S>,
) -> Vec<(String, String)> {
    g.key_headings()
        .map(|h| (h.name.clone(), value_of(row, &h.name)))
        .collect()
}

/// The *parent's* key-chain, reconstructed from the child's denormalised
/// row. `None` for a root group (no parent). Each parent KEY value is read
/// from the child row **by name** — valid because the child repeats every
/// ancestor KEY. For the rare key-drift groups (e.g. `MOND` keys on
/// `MOND_REF` where its parent `MONG` keys on `PIPE_REF`) a parent KEY name
/// is absent from the child and resolves to `""`; such a `parent_id` will
/// not match and is surfaced (see [`shared_keys`]) rather than fabricated.
#[must_use]
pub fn parent_chain_values<S: BuildHasher>(
    reg: &Registry,
    g: &GroupDescriptor,
    row: &HashMap<String, String, S>,
) -> Option<Vec<(String, String)>> {
    let parent = reg.get(g.parent.as_deref()?)?;
    Some(
        parent
            .key_headings()
            .map(|h| (h.name.clone(), value_of(row, &h.name)))
            .collect(),
    )
}

/// The KEY heading names `g` shares with its parent (the intersection that
/// links a child to its parent). Empty ⇒ the relationship is unresolvable
/// from the data (key drift with no shared name); the extension surfaces
/// this in `ags_relationships` rather than emitting a dangling `parent_id`.
#[must_use]
pub fn shared_keys(reg: &Registry, g: &GroupDescriptor) -> Vec<String> {
    let Some(parent) = g.parent.as_deref().and_then(|p| reg.get(p)) else {
        return Vec::new();
    };
    let child: std::collections::HashSet<&str> =
        g.key_headings().map(|h| h.name.as_str()).collect();
    parent
        .key_headings()
        .filter(|h| child.contains(h.name.as_str()))
        .map(|h| h.name.clone())
        .collect()
}

/// Both ids for a row in one pass: `(id, parent_id)`. `parent_id` is `None`
/// for a root group.
#[must_use]
pub fn row_ids<S: BuildHasher>(
    reg: &Registry,
    g: &GroupDescriptor,
    row: &HashMap<String, String, S>,
) -> (Uuid, Option<Uuid>) {
    let id = content_id(&g.code, &key_chain_values(g, row));
    let parent_id = match (g.parent.as_deref(), parent_chain_values(reg, g, row)) {
        (Some(parent_code), Some(chain)) => Some(content_id(parent_code, &chain)),
        _ => None,
    };
    (id, parent_id)
}

/// Stringified `(_id, _parent_id)` for every row of a group — ready to prepend
/// as the two Arrow key columns (see
/// `laterite_ags4_types::arrow_cols::build_record_batch_with_ids`). Mirrors that
/// builder's `(headings, n_rows, cell)` interface deliberately: a host computes
/// the ids and the typed batch from the **same** inputs, so the two can never
/// misalign. `_parent_id` is `None` (→ a NULL Arrow cell) for a root group.
/// Returns an empty `Vec` when `code` is not a known group — a custom /
/// passthrough group carries no spec keys, so it gets no content-addressed ids
/// (the caller then builds an unkeyed batch).
///
/// Only KEY columns feed a row's `_id`, so rather than rebuild a per-row
/// `HashMap` over EVERY heading (the dominant read cost — a wide group clones
/// dozens of throwaway strings per row) we resolve each own- and parent-KEY
/// heading to its column index ONCE per group and read those cells positionally.
/// Byte-identical to the old `row_ids`/map path **by construction**: `rposition`
/// (last match) mirrors the `HashMap`'s last-insert-wins on a malformed duplicate
/// heading, an absent heading resolves to `""` exactly as `value_of` did, and the
/// same [`content_id`] hashes the same trimmed values — pinned by
/// `content_id_pins_the_cross_surface_golden`. (`row_ids`/`key_chain_values` stay
/// for the extension's per-row `HashMap` caller.)
pub fn group_row_ids<'a, F>(
    reg: &Registry,
    code: &str,
    headings: &[String],
    n_rows: usize,
    cell: F,
) -> Vec<(String, Option<String>)>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let Some(g) = reg.get(code) else {
        return Vec::new();
    };
    // Resolve each KEY heading to its column index once. `None` ⇒ the KEY heading
    // is absent from this file's group → its value is always "" (a partial key
    // can't alias a complete one). `rposition` = last match, matching the old
    // all-columns HashMap's last-insert-wins on a duplicate heading.
    let key_cols = |desc: &GroupDescriptor| -> Vec<(String, Option<usize>)> {
        desc.key_headings()
            .map(|h| (h.name.clone(), headings.iter().rposition(|x| x == &h.name)))
            .collect()
    };
    let own = key_cols(g);
    // (parent domain code, parent KEY column indices), reconstructed from the
    // child's denormalised row. `None` for a root or unregistered parent —
    // exactly when the old `parent_chain_values` returned `None`.
    let parent = g
        .parent
        .as_deref()
        .and_then(|pcode| reg.get(pcode).map(|p| (pcode.to_string(), key_cols(p))));

    // One hasher reused across every row and both ids (`finalize_reset`), fed the
    // KEY cells as BORROWED `&str` — no per-row chain `Vec`, no `canonical_encode`
    // `Vec<u8>`, no name clones (Step 2: a wide row drops from dozens of allocs to
    // just the two id strings it returns). Byte-identical to `content_id`: the
    // same `canonical_encode_into` over the same rposition-resolved, trimmed values.
    let mut hasher = Sha256::new();
    (0..n_rows)
        .map(|row| {
            let id = row_id_streamed(&mut hasher, &g.code, &own, row, &cell);
            let parent_id = parent
                .as_ref()
                .map(|(pcode, pspec)| row_id_streamed(&mut hasher, pcode, pspec, row, &cell));
            (id.to_string(), parent_id.map(|u| u.to_string()))
        })
        .collect()
}

/// The deterministic id of a (group, key-chain): UUIDv8 over the first 128
/// bits of `SHA-256(canonical_encode(...))`. `Uuid::new_v8` sets the version
/// nibble to 8 (RFC 9562's custom/application version — the correct choice
/// for app-defined deterministic UUIDs) and the RFC variant bits.
#[must_use]
pub fn content_id(group_code: &str, chain: &[(String, String)]) -> Uuid {
    // Stream the canonical encoding straight into the hasher — no intermediate
    // `Vec<u8>`. Byte-identical to `Sha256::digest(canonical_encode(...))`: the
    // same `canonical_encode_into` defines the bytes for both.
    let mut hasher = Sha256::new();
    #[allow(clippy::cast_possible_truncation)]
    let chain_len = chain.len() as u32;
    canonical_encode_into(
        &mut hasher,
        group_code,
        chain_len,
        chain.iter().map(|(n, v)| (n.as_str(), v.as_str())),
    );
    v8_from_digest(&hasher.finalize())
}

/// Domain tag mixed into the group code for [`content_hash`], so a value-hash
/// can never collide with an *identity* [`content_id`]: a row whose only
/// non-blank cell happens to be its single KEY would otherwise present the
/// identical chain to both functions and hash to the same UUID. The version
/// suffix is deliberate — the canonicalisation below IS the contract, so
/// changing it must invalidate every previously-computed hash rather than
/// silently reinterpret one. Bumped `CONTENT1` → `CONTENT2` because V2
/// additionally folds the cell's UNIT into the hash (V1 did not, so a `10.0
/// m` and a `10.0 ft` cell used to alias) — old and new hashes must never be
/// conflated, hence the bump rather than a silent behaviour change.
const CONTENT_HASH_DOMAIN: &str = "\u{1f}CONTENT2";

/// A **typed, blank-insensitive** fingerprint of a row's whole *value* — the
/// counterpart to [`content_id`], which fingerprints a row's *identity*.
///
/// The two answer different questions and must not be conflated: two deliveries
/// of `LOCA BH01` with a corrected `LOCA_GL` share an `id` (same borehole) and
/// differ in their `content_hash` (the data changed). That is precisely the
/// distinction the DuckDB cookbook once got wrong.
///
/// **Why this hashes the PARSED value, where [`content_id`] hashes raw bytes.**
/// The module note above explains why identity must hash the producer's bytes:
/// a child denormalises the parent's `"1.50"` verbatim, so parsing would risk
/// splitting identity across formatters. A *value* hash has the opposite need —
/// a producer re-emitting `1.0` as `1.00` has changed nothing, and reporting it
/// as a revision is a false positive. So values go through
/// [`laterite_ags4_types::parse_value`], the same canonicaliser
/// `laterite-ags4-merge` uses to decide a cell actually changed and
/// `laterite-ags4-diff` uses to ignore formatting-only edits. One authority for
/// "are these two cells the same", by construction rather than by coincidence.
///
/// The rules, in full:
/// - Every heading is hashed, not just KEYs (so a changed non-key cell is
///   visible), including custom/passthrough headings — whose unknown AGS type
///   falls through `parse_value` to string, so they still count.
/// - **A cell that canonicalises to `Null` is OMITTED.** `parse_value` maps an
///   empty cell to `Value::Null`, so *blank ≡ absent* falls out of the existing
///   canonicaliser rather than needing a special case. This is what lets two
///   deliveries with different heading sets still dedup on the columns they
///   share. The cost, stated plainly: a column that is absent and a column that
///   is present-but-blank are indistinguishable to this hash.
/// - Pairs are sorted by heading name, so column ORDER does not affect the hash.
/// - The group code (domain-tagged) is hashed in, so identical values under two
///   different groups can never collide.
/// - **The UNIT is folded into the hash.** So a `10.0 m` and a `10.0 ft` cell
///   never dedup — `laterite-ags4-merge` refuses that pair as irreconcilable
///   (laterite-dev#501), and collapsing it here would be silent data loss. A blank unit
///   means "unspecified": it trims to `""`, the same constant for every blank
///   cell, matching `merged_unit`'s `filter(|u| !u.is_empty())` — so two
///   blank-unit cells still dedup among themselves, but never against a
///   stated unit (the conservative direction: failing to dedup is safe, a
///   false dedup is not).
///
/// **The sharp edge.** The hash is computed from ONE file, using THAT file's
/// declared TYPE row. Two deliveries that disagree on a column's TYPE can
/// canonicalise the same raw bytes differently (`"10.00"` as a number under
/// `2DP` vs as a string under `X`) and therefore NOT dedup. This is inherent to
/// a per-row column — it cannot know what the other file declared — and it is
/// exactly the disagreement `laterite-ags4-merge` exists to reconcile.
#[must_use]
pub fn content_hash(group_code: &str, cells: &[(&str, &str, &str, &str)]) -> Uuid {
    let mut triples: Vec<(String, String, String)> = cells
        .iter()
        .filter_map(|(heading, unit, ags_type, raw)| {
            match parse_value(Some(raw), ags_type) {
                // Blank ≡ absent. Dropping the cell (rather than hashing an empty
                // string) is what makes a blank and an absent column hash alike.
                Value::Null => None,
                v => Some((
                    (*heading).to_string(),
                    // UNIT folded in so `10.0 m` and `10.0 ft` never dedup — merge
                    // refuses that pair as irreconcilable (laterite-dev#501); collapsing it
                    // would be silent data loss. A blank unit is "unspecified" and
                    // trims to "" (constant for every blank), matching
                    // `merged_unit`'s `filter(|u| !u.is_empty())`: blanks dedup
                    // among themselves and never with a stated unit.
                    (*unit).trim().to_string(),
                    v.to_string(),
                )),
            }
        })
        .collect();
    triples.sort_by(|a, b| a.0.cmp(&b.0));

    let domain = format!("{group_code}{CONTENT_HASH_DOMAIN}");
    let digest = Sha256::digest(content_encode(&domain, &triples));
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::new_v8(bytes)
}

/// Stringified `_content_hash` for every row of a group — the value-side twin of
/// [`group_row_ids`], and deliberately the same `(headings, n_rows, cell)`
/// interface so a host computes ids, hashes and the typed batch from the **same**
/// inputs and they cannot misalign.
///
/// Takes the file's own `TYPE` row (`types`, parallel to `headings`) because
/// canonicalisation is per-file — see [`content_hash`]'s "sharp edge" — and its
/// own `UNIT` row (`units`, also parallel to `headings`), because the UNIT is
/// folded into the hash (see [`content_hash`]'s UNIT rule).
///
/// Unlike [`group_row_ids`] this needs **no `Registry`**: it hashes every
/// heading rather than the spec key-chain, so an unknown custom/passthrough
/// group still gets a usable hash where it would get no `_id` at all.
pub fn group_content_hashes<'a, F>(
    code: &str,
    headings: &[String],
    units: &[String],
    types: &[String],
    n_rows: usize,
    cell: F,
) -> Vec<String>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    (0..n_rows)
        .map(|row| {
            let cells: Vec<(&str, &str, &str, &str)> = headings
                .iter()
                .enumerate()
                .map(|(col, h)| {
                    // A ragged/short row leaves a cell absent → "" → Null →
                    // omitted, which is the same outcome as a blank cell. That
                    // is the intended equivalence, not an accident.
                    let unit = units.get(col).map_or("", String::as_str);
                    let ty = types.get(col).map_or("", String::as_str);
                    (h.as_str(), unit, ty, cell(col, row).unwrap_or(""))
                })
                .collect();
            content_hash(code, &cells).to_string()
        })
        .collect()
}

/// Injective, collision-proof encoding of a (group, key-chain). Every string
/// is length-prefixed (`u32` LE byte length + UTF-8 bytes), so no field's
/// content — NUL, newline, commas, whatever a hostile file carries — can be
/// misread as a separator. The group code is hashed too (so the same key
/// tuple under two groups can never collide), and the chain length is fixed
/// in (so a trailing empty key can't alias a shorter chain). This is the
/// reason a plain `"\n\0"` join (the legacy `encode_shared_tuple`) is *not*
/// reused: that join is not injective for arbitrary content.
#[must_use]
pub fn canonical_encode(group_code: &str, chain: &[(String, String)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + group_code.len() + chain.len() * 16);
    // `chain` is one AGS4 group's KEY-heading tuple (a handful of entries by
    // dictionary construction), nowhere near u32::MAX.
    #[allow(clippy::cast_possible_truncation)]
    let chain_len = chain.len() as u32;
    canonical_encode_into(
        &mut out,
        group_code,
        chain_len,
        chain.iter().map(|(n, v)| (n.as_str(), v.as_str())),
    );
    out
}

/// Injective encoding for the value hash's `(heading, unit, value)` triples —
/// the twin of [`canonical_encode`]. The UNIT is length-prefixed as its own
/// field alongside the value, so `10.0 m` and `10.0 ft` can never encode alike.
/// Kept separate from `canonical_encode` so a change to the value hash's shape
/// can never perturb the identity hash.
fn content_encode(domain: &str, triples: &[(String, String, String)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + domain.len() + triples.len() * 24);
    put_lp(&mut out, domain);
    // `triples` is one row's (heading, unit, value) tuples for a single AGS4
    // group — bounded by that group's heading count (dictionary-bounded, a
    // few dozen at most), nowhere near u32::MAX.
    #[allow(clippy::cast_possible_truncation)]
    let triples_len = triples.len() as u32;
    out.extend_from_slice(&triples_len.to_le_bytes());
    for (name, unit, value) in triples {
        put_lp(&mut out, name);
        put_lp(&mut out, unit);
        put_lp(&mut out, value);
    }
    out
}

/// Read a KEY value from a row by name, trimmed; absent ⇒ `""`. The codec
/// already trims on parse; trimming here keeps the key stable regardless of
/// how the caller built the row.
fn value_of<S: BuildHasher>(row: &HashMap<String, String, S>, name: &str) -> String {
    row.get(name)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// A byte sink for the canonical key-chain encoding — implemented for both a
/// `Vec<u8>` (the buffered public [`canonical_encode`]) and a streaming
/// [`Sha256`] (the per-row hot path in [`group_row_ids`]). A single
/// [`canonical_encode_into`] defines the byte layout over this trait, so the
/// buffered and streamed encodings can never diverge.
trait ByteSink {
    fn put(&mut self, bytes: &[u8]);
}
impl ByteSink for Vec<u8> {
    fn put(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
impl ByteSink for Sha256 {
    fn put(&mut self, bytes: &[u8]) {
        Digest::update(self, bytes);
    }
}

/// Append a length-prefixed string: `u32` LE byte length, then the bytes.
// `s` is a group code, a heading name, or a single AGS4 field's cell value —
// bounded by that field's own line, which (per the parse leaf's tokenizer)
// cannot realistically reach u32::MAX bytes for real geotechnical data.
#[allow(clippy::cast_possible_truncation)]
fn put_lp<S: ByteSink>(sink: &mut S, s: &str) {
    sink.put(&(s.len() as u32).to_le_bytes());
    sink.put(s.as_bytes());
}

/// The one definition of the (group, key-chain) byte layout: the group code
/// (length-prefixed), the chain length (`u32` LE — fixed in so a trailing empty
/// key can't alias a shorter chain), then each `(name, value)` length-prefixed.
/// `chain` yields already-trimmed pairs; the sink is a `Vec` (buffered) or a
/// `Sha256` (streamed), the same bytes either way. `name`/`value` are taken as
/// `AsRef<str>` so the two can carry independent borrow lifetimes.
fn canonical_encode_into<S, I, N, V>(sink: &mut S, group_code: &str, chain_len: u32, chain: I)
where
    S: ByteSink,
    I: IntoIterator<Item = (N, V)>,
    N: AsRef<str>,
    V: AsRef<str>,
{
    put_lp(sink, group_code);
    sink.put(&chain_len.to_le_bytes());
    for (name, value) in chain {
        put_lp(sink, name.as_ref());
        put_lp(sink, value.as_ref());
    }
}

/// The first 128 bits of a finished SHA-256 as a UUIDv8 (RFC 9562's app-defined
/// version) — shared by [`content_id`] and the reused-hasher path so the id
/// derivation is defined exactly once.
fn v8_from_digest(digest: &[u8]) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::new_v8(bytes)
}

/// One row's `_id` (or `_parent_id`), streamed into a REUSED `hasher`
/// (`finalize_reset`) with each KEY column read positionally and trimmed — the
/// allocation-free twin of `content_id(dom, &chain)` for the batch
/// [`group_row_ids`] hot path. An absent column or short/ragged row resolves to
/// `""`, exactly as `value_of` did.
fn row_id_streamed<'a, F>(
    hasher: &mut Sha256,
    dom: &str,
    spec: &[(String, Option<usize>)],
    row: usize,
    cell: &F,
) -> Uuid
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    #[allow(clippy::cast_possible_truncation)]
    let chain_len = spec.len() as u32;
    canonical_encode_into(
        hasher,
        dom,
        chain_len,
        spec.iter().map(|(name, idx)| {
            (
                name.as_str(),
                idx.and_then(|i| cell(i, row)).unwrap_or("").trim(),
            )
        }),
    );
    v8_from_digest(&hasher.finalize_reset())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::union::registry;

    fn chain(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// A decoder used only to *prove* `canonical_encode` is injective:
    /// round-tripping recovers the exact input, which is impossible unless
    /// the encoding is lossless and unambiguous.
    fn decode(bytes: &[u8]) -> (String, Vec<(String, String)>) {
        let mut i = 0;
        let take_lp = |bytes: &[u8], i: &mut usize| -> String {
            let len = u32::from_le_bytes(bytes[*i..*i + 4].try_into().unwrap()) as usize;
            *i += 4;
            let s = String::from_utf8(bytes[*i..*i + len].to_vec()).unwrap();
            *i += len;
            s
        };
        let code = take_lp(bytes, &mut i);
        let n = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        i += 4;
        let mut chain = Vec::with_capacity(n);
        for _ in 0..n {
            let name = take_lp(bytes, &mut i);
            let value = take_lp(bytes, &mut i);
            chain.push((name, value));
        }
        assert_eq!(i, bytes.len(), "decoder must consume every byte");
        (code, chain)
    }

    #[test]
    fn id_is_deterministic() {
        let c = chain(&[("LOCA_ID", "BH01")]);
        assert_eq!(content_id("LOCA", &c), content_id("LOCA", &c));
    }

    #[test]
    fn id_differs_on_any_change() {
        let base = content_id("LOCA", &chain(&[("LOCA_ID", "BH01")]));
        assert_ne!(base, content_id("LOCA", &chain(&[("LOCA_ID", "BH02")]))); // value
        assert_ne!(base, content_id("SAMP", &chain(&[("LOCA_ID", "BH01")]))); // group
        assert_ne!(base, content_id("LOCA", &chain(&[("LOCA_NM", "BH01")]))); // name
    }

    #[test]
    fn id_is_uuidv8() {
        let id = content_id("LOCA", &chain(&[("LOCA_ID", "BH01")]));
        assert_eq!(id.get_version_num(), 8);
        assert_eq!(id.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn key_heading_names_lists_the_denormalised_key_chain() {
        // SAMP's identifying chain carries its own SAMP_ID and the inherited
        // LOCA_ID — a blanked or bogus list would drop them.
        let samp = registry().get("SAMP").expect("SAMP");
        let names = key_heading_names(samp);
        assert!(names.contains(&"SAMP_ID"), "own KEY missing: {names:?}");
        assert!(
            names.contains(&"LOCA_ID"),
            "inherited KEY missing: {names:?}"
        );
    }

    #[test]
    fn shared_keys_are_the_link_to_the_parent() {
        // SAMP links to LOCA through the LOCA_ID it repeats; a root group shares
        // nothing, so its list is empty.
        let reg = registry();
        let samp = reg.get("SAMP").expect("SAMP");
        assert!(shared_keys(reg, samp).contains(&"LOCA_ID".to_string()));
        assert!(
            shared_keys(reg, reg.get("PROJ").expect("PROJ")).is_empty(),
            "a root group shares no keys"
        );
    }

    #[test]
    fn content_id_pins_the_cross_surface_golden() {
        // The SINGLE SOURCE of the golden UUIDv8s that the Python / Node / wasm
        // surface tests all assert — `test_content_keys.py`, `p3-content-keys.test.ts`,
        // and `arrow_ipc_keys_match_the_shared_golden_and_default_strips`. Every host
        // reads its `_id`/`_parent_id` through THIS `row_ids`, so pinning the exact
        // strings here proves cross-surface parity by construction: change the id
        // maths and every surface's test fails together. Fixture (shared): a root
        // PROJ keyed PROJ_ID=P1, and a LOCA child carrying PROJ_ID=P1 + LOCA_ID=BH1.
        // (#303 Phase 6)
        let reg = registry();
        let map = |pairs: &[(&str, &str)]| -> HashMap<String, String> {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        let (proj_id, proj_parent) =
            row_ids(reg, reg.get("PROJ").unwrap(), &map(&[("PROJ_ID", "P1")]));
        assert_eq!(proj_id.to_string(), "ac30a95d-e0ca-85f9-83c8-37a64af2762b");
        assert!(proj_parent.is_none(), "PROJ is root");

        let (loca_id, loca_parent) = row_ids(
            reg,
            reg.get("LOCA").unwrap(),
            &map(&[("PROJ_ID", "P1"), ("LOCA_ID", "BH1")]),
        );
        assert_eq!(loca_id.to_string(), "a7025a6f-d9b8-83b6-8fad-81c0c744edbc");
        assert_eq!(loca_parent, Some(proj_id), "LOCA._parent_id == PROJ._id");
    }

    #[test]
    fn encoding_round_trips_so_it_is_injective() {
        // Cases that a naive separator-join would confuse: a value that
        // contains the legacy "\n\0" sentinel, empty values, a key whose
        // bytes equal another's name+value boundary.
        for (code, c) in [
            ("LOCA", chain(&[("LOCA_ID", "BH01")])),
            ("SAMP", chain(&[("LOCA_ID", "BH\n\0X"), ("SAMP_REF", "")])),
            ("X", chain(&[("a", "bc"), ("ab", "c")])),
            ("X", chain(&[("a", ""), ("", "bc")])),
            ("PROJ", chain(&[])),
        ] {
            let encoded = canonical_encode(code, &c);
            assert_eq!(decode(&encoded), (code.to_string(), c));
        }
    }

    #[test]
    fn ambiguous_inputs_do_not_collide() {
        // ("ab","c") vs ("a","bc") would join to the same string under a
        // naive concatenation; length-prefixing keeps them distinct.
        assert_ne!(
            canonical_encode("X", &chain(&[("ab", "c")])),
            canonical_encode("X", &chain(&[("a", "bc")])),
        );
    }

    #[test]
    fn samp_key_chain_is_denormalised_and_ordered() {
        let reg = registry();
        let samp = reg.get("SAMP").unwrap();
        let mut row = HashMap::new();
        for (k, v) in [
            ("LOCA_ID", "BH01"),
            ("SAMP_TOP", "1.50"),
            ("SAMP_REF", "S1"),
            ("SAMP_TYPE", "U"),
            ("SAMP_ID", "SA1"),
        ] {
            row.insert(k.to_string(), v.to_string());
        }
        let names: Vec<String> = key_chain_values(samp, &row)
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(
            names,
            ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"]
        );
    }

    /// The load-bearing property: for every non-drift parent/child pair in
    /// the registry, a child row carrying the shared key values produces a
    /// `parent_id` byte-identical to the parent row's own `id`. This is what
    /// makes cross-group joins work with no shared state.
    #[test]
    fn child_parent_id_equals_parent_id_by_construction() {
        let reg = registry();
        let mut checked = 0;
        let mut drift: Vec<String> = Vec::new();
        for g in reg.iter() {
            let Some(parent) = g.parent.as_deref().and_then(|p| reg.get(p)) else {
                continue; // root
            };
            // Drift = a parent KEY name the child does not carry by name.
            let child_keys: std::collections::HashSet<&str> =
                g.key_headings().map(|h| h.name.as_str()).collect();
            if !parent
                .key_headings()
                .all(|h| child_keys.contains(h.name.as_str()))
            {
                drift.push(g.code.clone());
                continue;
            }
            // One shared value table drives both rows, so by construction the
            // child carries exactly what the parent was keyed on.
            let mut values: HashMap<String, String> = HashMap::new();
            for (i, h) in g.key_headings().enumerate() {
                values.insert(h.name.clone(), format!("v{i}"));
            }
            let (_, parent_id) = row_ids(reg, g, &values);
            let (parent_own_id, _) = row_ids(reg, parent, &values);
            assert_eq!(
                parent_id,
                Some(parent_own_id),
                "{} -> {} parent_id must equal parent.id",
                g.code,
                parent.code,
            );
            checked += 1;
        }
        assert!(
            checked > 50,
            "expected to check most of the 92 groups, got {checked}"
        );
        // Document (don't fail on) the known key-drift groups; they're
        // handled separately and surfaced via `shared_keys`.
        for code in &drift {
            assert!(
                shared_keys(reg, reg.get(code).unwrap()).len() < {
                    let g = reg.get(code).unwrap();
                    reg.get(g.parent.as_deref().unwrap())
                        .unwrap()
                        .key_headings()
                        .count()
                },
                "{code} classified as drift but shares all parent keys",
            );
        }
    }

    #[test]
    fn root_group_has_no_parent_id() {
        let reg = registry();
        let proj = reg.get("PROJ").unwrap();
        let (_, parent_id) = row_ids(reg, proj, &HashMap::new());
        assert!(parent_id.is_none());
    }

    #[test]
    fn group_row_ids_wraps_row_ids_and_links_child_to_parent() {
        let reg = registry();

        // PROJ (root): one row keyed PROJ_ID=P1 → _parent_id NULL.
        let proj_h = ["PROJ_ID".to_string()];
        let proj_rows = [["P1"]];
        let proj_ids = group_row_ids(reg, "PROJ", &proj_h, proj_rows.len(), |c, r| {
            proj_rows.get(r).and_then(|row| row.get(c)).copied()
        });
        assert_eq!(proj_ids.len(), 1);
        assert!(proj_ids[0].1.is_none(), "PROJ is root → _parent_id is None");

        // LOCA (child of PROJ): carries PROJ_ID=P1 plus its own key LOCA_ID.
        let loca_h = ["PROJ_ID".to_string(), "LOCA_ID".to_string()];
        let loca_rows = [["P1", "BH01"]];
        let loca_ids = group_row_ids(reg, "LOCA", &loca_h, loca_rows.len(), |c, r| {
            loca_rows.get(r).and_then(|row| row.get(c)).copied()
        });
        assert_eq!(loca_ids.len(), 1);

        // (1) Faithful wrapper: identical to a direct row_ids over the same map.
        let map: HashMap<String, String> = loca_h
            .iter()
            .cloned()
            .zip(loca_rows[0].iter().map(std::string::ToString::to_string))
            .collect();
        let (id, parent) = row_ids(reg, reg.get("LOCA").unwrap(), &map);
        assert_eq!(loca_ids[0].0, id.to_string());
        assert_eq!(loca_ids[0].1, parent.map(|u| u.to_string()));

        // (2) The cross-group join property THROUGH the helper: a LOCA row's
        // _parent_id is byte-identical to the PROJ row's own _id.
        assert_eq!(
            loca_ids[0].1.as_deref(),
            Some(proj_ids[0].0.as_str()),
            "LOCA._parent_id must equal PROJ._id by construction",
        );
    }

    #[test]
    fn group_row_ids_empty_for_unknown_group() {
        let reg = registry();
        let ids = group_row_ids(reg, "ZZZZ", &["ZZZZ_ID".to_string()], 1, |_, _| Some("x"));
        assert!(
            ids.is_empty(),
            "a custom/passthrough group carries no spec keys → no content ids"
        );
    }

    // ---- content_hash: the value-side twin. Each test pins one clause of the
    // contract the feature was specified against; a failure here is a contract
    // breach, not a style nit.

    /// `(heading, unit, type, value)` 4-tuples, for readability in the tests
    /// below. No existing test here is *about* units, so every call site below
    /// passes `""` — [`content_hash_folds_unit`] is the one that isn't.
    fn cells<'a>(
        v: &'a [(&'a str, &'a str, &'a str, &'a str)],
    ) -> Vec<(&'a str, &'a str, &'a str, &'a str)> {
        v.to_vec()
    }

    #[test]
    fn identical_rows_hash_identically_and_a_changed_cell_does_not() {
        let a = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
            ]),
        );
        let same = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
            ]),
        );
        let revised = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "11.50"),
            ]),
        );
        assert_eq!(a, same, "same values → same hash");
        assert_ne!(a, revised, "a changed non-key value MUST change the hash");
    }

    #[test]
    fn blank_equals_absent_so_differing_heading_sets_still_dedup() {
        // File A has no LOCA_REM column at all. File B has one, left blank.
        // The contract says these are the SAME row.
        let file_a = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
            ]),
        );
        let file_b = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
                ("LOCA_REM", "", "X", ""),
            ]),
        );
        assert_eq!(
            file_a, file_b,
            "blank ≡ absent — else two deliveries with different heading sets never dedup"
        );

        // But a POPULATED extra column is a real difference.
        let file_b_populated = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
                ("LOCA_REM", "", "X", "re-survey"),
            ]),
        );
        assert_ne!(
            file_a, file_b_populated,
            "a populated extra column IS a value difference"
        );
    }

    #[test]
    fn formatting_only_change_does_not_alter_the_hash() {
        let a = content_hash("LOCA", &cells(&[("LOCA_GL", "", "2DP", "10.0")]));
        let b = content_hash("LOCA", &cells(&[("LOCA_GL", "", "2DP", "10.00")]));
        assert_eq!(
            a, b,
            "`10.0` and `10.00` under 2DP are the same value — parse_value canonicalises"
        );
    }

    #[test]
    fn column_order_does_not_alter_the_hash() {
        let a = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
            ]),
        );
        let b = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_GL", "", "2DP", "10.00"),
                ("LOCA_ID", "", "ID", "BH01"),
            ]),
        );
        assert_eq!(a, b, "pairs are sorted by heading name before encoding");
    }

    #[test]
    fn a_string_value_cannot_alias_the_same_text_as_a_number() {
        // serde_json's compact form is type-tagged (a String renders quoted),
        // so the TEXT "10.0" and the NUMBER 10.0 must not collide.
        let as_text = content_hash("LOCA", &cells(&[("LOCA_GL", "", "X", "10.0")]));
        let as_number = content_hash("LOCA", &cells(&[("LOCA_GL", "", "2DP", "10.0")]));
        assert_ne!(
            as_text, as_number,
            "type must be part of the fingerprint, not just the rendered text"
        );
    }

    #[test]
    fn the_same_values_under_two_groups_do_not_collide() {
        let a = content_hash("LOCA", &cells(&[("X", "", "X", "v")]));
        let b = content_hash("SAMP", &cells(&[("X", "", "X", "v")]));
        assert_ne!(a, b, "the group code is hashed in");
    }

    #[test]
    fn a_value_hash_never_collides_with_an_identity_id() {
        // The collision this guards: a row whose ONLY non-blank cell is its
        // single KEY presents the same (heading, value) chain to both
        // functions. The domain tag is what keeps them apart.
        let chain = vec![("PROJ_ID".to_string(), "\"P1\"".to_string())];
        let id = content_id("PROJ", &chain);
        let hash = content_hash("PROJ", &cells(&[("PROJ_ID", "", "ID", "P1")]));
        assert_ne!(
            id.to_string(),
            hash.to_string(),
            "identity and value hashes must live in separate domains"
        );
    }

    #[test]
    fn unknown_custom_group_still_hashes_even_though_it_has_no_id() {
        // group_row_ids returns EMPTY for an unknown group (no spec keys).
        // content_hash needs no Registry, so a passthrough group still gets a
        // usable value fingerprint — a real capability difference, asserted.
        let units = vec![String::new()];
        let h = group_content_hashes(
            "ZZZZ",
            &["ZZZZ_VAL".to_string()],
            &units,
            &["X".to_string()],
            2,
            |_, row| Some(if row == 0 { "a" } else { "b" }),
        );
        assert_eq!(h.len(), 2);
        assert_ne!(h[0], h[1], "distinct values → distinct hashes");
    }

    #[test]
    fn group_content_hashes_is_deterministic_and_row_aligned() {
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let units = vec![String::new(); headings.len()];
        let types = vec!["ID".to_string(), "2DP".to_string()];
        let data = [["BH01", "10.00"], ["BH02", "12.00"], ["BH01", "10.00"]];
        let run = || {
            group_content_hashes("LOCA", &headings, &units, &types, 3, |col, row| {
                Some(data[row][col])
            })
        };
        let first = run();
        assert_eq!(first, run(), "two independent computations must agree");
        assert_eq!(
            first[0], first[2],
            "rows 0 and 2 carry identical values → identical hash (this IS the dedup)"
        );
        assert_ne!(first[0], first[1]);
    }

    /// **The value-hash contract, pinned to literal UUIDs.** The behavioural
    /// tests above prove the RELATIONSHIPS (equal values → equal hash, a changed
    /// cell → a changed hash); this pins the ABSOLUTE output, so a change to the
    /// canonicalisation, the domain tag (`CONTENT_HASH_DOMAIN`), or the encoding
    /// is caught even when it preserves every relationship. If these literals
    /// move — because you bumped the domain, or `parse_value` changed (see the
    /// twin pin-table `parse_value_canonical_form_is_pinned_for_the_content_hash_contract`
    /// in `laterite-ags4-types`) — updating them here is the deliberate, reviewable
    /// record that every previously-computed hash just changed.
    #[test]
    fn content_hash_golden_literals() {
        // A single row's value-hash …
        let one = content_hash(
            "LOCA",
            &cells(&[
                ("LOCA_ID", "", "ID", "BH01"),
                ("LOCA_GL", "", "2DP", "10.00"),
            ]),
        )
        .to_string();
        assert_eq!(one, "60eed2bb-66b4-8ffd-b920-1adc741badc7");

        // … equals row 0 of the same values hashed through the group entry
        // point (same inputs → same hash, by construction), and row 1 differs.
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let units = vec![String::new(); headings.len()];
        let types = vec!["ID".to_string(), "2DP".to_string()];
        let data = [["BH01", "10.00"], ["BH02", "12.34"]];
        let hashes = group_content_hashes("LOCA", &headings, &units, &types, 2, |col, row| {
            Some(data[row][col])
        });
        assert_eq!(
            hashes,
            vec![
                "60eed2bb-66b4-8ffd-b920-1adc741badc7".to_string(),
                "38fbe590-7453-8ad7-a9dd-07a01e96f064".to_string()
            ],
        );
    }

    /// The UNIT rule, pinned: a stated unit distinguishes, a blank unit is
    /// "unspecified" and dedups only against other blanks, and re-emitting
    /// the SAME unit still dedups. See [`content_hash`]'s UNIT rule and laterite-dev#501.
    #[test]
    fn content_hash_folds_unit() {
        let metres = content_hash("LOCA", &cells(&[("LOCA_GL", "m", "2DP", "10.00")]));
        let feet = content_hash("LOCA", &cells(&[("LOCA_GL", "ft", "2DP", "10.00")]));
        assert_ne!(
            metres, feet,
            "same value, different non-blank unit — must NOT dedup (laterite-dev#501)"
        );

        let blank = content_hash("LOCA", &cells(&[("LOCA_GL", "", "2DP", "10.00")]));
        assert_ne!(
            blank, metres,
            "a blank unit ('unspecified') must not dedup against a stated unit"
        );

        let blank_again = content_hash("LOCA", &cells(&[("LOCA_GL", "", "2DP", "10.00")]));
        assert_eq!(
            blank, blank_again,
            "two blank-unit cells with the same value must still dedup among themselves"
        );

        let metres_reemit = content_hash("LOCA", &cells(&[("LOCA_GL", "m", "2DP", "10.0")]));
        assert_eq!(
            metres, metres_reemit,
            "a re-emit within the SAME unit is a formatting change, not a unit change"
        );
    }

    /// The TYPE-pair behaviour I promised to REPORT rather than assume. This
    /// asserts what actually happens when two files declare the same column
    /// differently — the feature's sharpest edge. If a future `parse_value`
    /// change moves any of these, this test fails loudly and the documented
    /// contract must be updated with it.
    #[test]
    fn type_pair_behaviour_table() {
        let h = |ty: &str, raw: &str| content_hash("LOCA", &cells(&[("LOCA_GL", "", ty, raw)]));

        // Same canonical class, different declared precision → SAME hash.
        assert_eq!(
            h("2DP", "10.00"),
            h("3DP", "10.000"),
            "2DP/3DP both canonicalise to the same number → dedup works"
        );
        // Numeric vs free-text → DIFFERENT hash for identical bytes. This is
        // the documented sharp edge: TYPE disagreement defeats dedup.
        assert_ne!(
            h("2DP", "10.00"),
            h("X", "10.00"),
            "typed-vs-X on identical bytes does NOT dedup — use `lat merge` for that case"
        );
    }
}
