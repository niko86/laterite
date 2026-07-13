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
//! separate calls — and what would let two separately-written `.ags5db`
//! files merge with no reconciliation (contrast the writer's random-UUID7 +
//! lookup-table path in `laterite-ags5-db`, which *cannot* agree across
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

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::union::{GroupDescriptor, Registry};

/// The KEY heading names of `g`, in declaration order — the ONE definition of
/// "what identifies a row", so the content-addressed `_id` (via
/// [`key_chain_values`]) and `laterite-ags4-diff`'s row matcher read the same
/// list instead of each re-deriving it. Row-identity lives here, once.
pub fn key_heading_names(g: &GroupDescriptor) -> Vec<&str> {
    g.key_headings().map(|h| h.name.as_str()).collect()
}

/// The row's own key-chain: `(heading, raw value)` for each KEY heading of
/// `g`, in declaration order. `g.key_headings()` is already denormalised
/// (ancestor KEYs first, then own — e.g. `SAMP` →
/// `[LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID]`), so this *is* the
/// full identifying tuple. Missing values resolve to `""` so a partial-key
/// row can't silently alias a complete one.
pub fn key_chain_values(
    g: &GroupDescriptor,
    row: &HashMap<String, String>,
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
pub fn parent_chain_values(
    reg: &Registry,
    g: &GroupDescriptor,
    row: &HashMap<String, String>,
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
pub fn shared_keys(reg: &Registry, g: &GroupDescriptor) -> Vec<String> {
    let parent = match g.parent.as_deref().and_then(|p| reg.get(p)) {
        Some(p) => p,
        None => return Vec::new(),
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
pub fn row_ids(
    reg: &Registry,
    g: &GroupDescriptor,
    row: &HashMap<String, String>,
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
/// `laterite_types::arrow_cols::build_record_batch_with_ids`). Mirrors that
/// builder's `(headings, n_rows, cell)` interface deliberately: a host computes
/// the ids and the typed batch from the **same** inputs, so the two can never
/// misalign. Each pair is [`row_ids`] stringified; `_parent_id` is `None`
/// (→ a NULL Arrow cell) for a root group. Returns an empty `Vec` when `code`
/// is not a known group — a custom / passthrough group carries no spec keys, so
/// it gets no content-addressed ids (the caller then builds an unkeyed batch).
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
    (0..n_rows)
        .map(|row| {
            // row_ids reads KEY values BY NAME, so reconstruct this row's
            // heading→value map. A short/ragged row leaves a heading absent →
            // `value_of` resolves it to "" (so a partial key can't alias).
            let map: HashMap<String, String> = headings
                .iter()
                .enumerate()
                .map(|(col, h)| (h.clone(), cell(col, row).unwrap_or("").to_string()))
                .collect();
            let (id, parent) = row_ids(reg, g, &map);
            (id.to_string(), parent.map(|u| u.to_string()))
        })
        .collect()
}

/// The deterministic id of a (group, key-chain): UUIDv8 over the first 128
/// bits of `SHA-256(canonical_encode(...))`. `Uuid::new_v8` sets the version
/// nibble to 8 (RFC 9562's custom/application version — the correct choice
/// for app-defined deterministic UUIDs) and the RFC variant bits.
pub fn content_id(group_code: &str, chain: &[(String, String)]) -> Uuid {
    let digest = Sha256::digest(canonical_encode(group_code, chain));
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::new_v8(bytes)
}

/// Injective, collision-proof encoding of a (group, key-chain). Every string
/// is length-prefixed (`u32` LE byte length + UTF-8 bytes), so no field's
/// content — NUL, newline, commas, whatever a hostile file carries — can be
/// misread as a separator. The group code is hashed too (so the same key
/// tuple under two groups can never collide), and the chain length is fixed
/// in (so a trailing empty key can't alias a shorter chain). This is the
/// reason a plain `"\n\0"` join (the legacy `encode_shared_tuple`) is *not*
/// reused: that join is not injective for arbitrary content.
pub fn canonical_encode(group_code: &str, chain: &[(String, String)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + group_code.len() + chain.len() * 16);
    put_lp(&mut out, group_code);
    out.extend_from_slice(&(chain.len() as u32).to_le_bytes());
    for (name, value) in chain {
        put_lp(&mut out, name);
        put_lp(&mut out, value);
    }
    out
}

/// Read a KEY value from a row by name, trimmed; absent ⇒ `""`. The codec
/// already trims on parse; trimming here keeps the key stable regardless of
/// how the caller built the row.
fn value_of(row: &HashMap<String, String>, name: &str) -> String {
    row.get(name)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// Append a length-prefixed string: `u32` LE byte length, then the bytes.
fn put_lp(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
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
            .zip(loca_rows[0].iter().map(|s| s.to_string()))
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
}
