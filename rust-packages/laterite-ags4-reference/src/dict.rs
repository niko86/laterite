//! Standard AGS4 data dictionary access.
//!
//! The dictionary *content* is ©AGS published reference data (see
//! `data/PROVENANCE.md`). It's compiled into the binary at build time
//! by `build.rs` as `phf` perfect-hash maps — zero startup cost, no
//! runtime parse. This module owns the public value types the generated
//! tables instantiate, plus the version selector + lookup surface the
//! rule modules (V3+) use.
//!
//! [`Dictionary`] is a lifetime-parametric enum (#568): `Bundled` is the
//! zero-cost `&'static` phf handle used everywhere today; `Layered` overlays
//! a runtime-owned [`OwnedDelta`](crate::overlay::OwnedDelta) (a custom `--dict`)
//! on a bundled base, consulting the tiny delta first and falling through to the
//! phf base. The enum stays `Copy` (both arms are refs/statics), so every existing
//! by-value `Dictionary` site is unaffected. Lookups return small `Copy` view
//! structs ([`HeadingRef`]/[`GroupRef`]) rather than `&'static DictEntry`, so the
//! two arms share one return type.
//!
//! phf does not implement `PhfHash` for tuples, so heading keys are the
//! composite string `"GROUP\u{1f}HEADING"` (US — unit separator, never
//! valid inside an AGS4 name). Build it via [`heading_key`].

use std::borrow::Cow;

use crate::overlay::OwnedDelta;

/// One heading's dictionary definition. All `&'static str` so the phf
/// tables live entirely in the binary's read-only segment and the
/// linker dedups the many repeated `"OTHER"` / `"m"` / `""` literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictEntry {
    pub ags_type: &'static str,
    pub unit: &'static str,
    pub status: &'static str,
    pub desc: &'static str,
}

/// One group's dictionary metadata (from the DICT group's `GROUP` rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupMeta {
    /// Parent group code; `""` for a root group (e.g. PROJ).
    pub parent: &'static str,
    pub desc: &'static str,
}

/// A borrowed view of one heading's definition, shared by the bundled
/// (`&'static`) and layered (owned-delta) arms of [`Dictionary`]. Field names
/// mirror [`DictEntry`], so every `e.ags_type` / `e.status.contains("KEY")`
/// reader is source-compatible. `Copy` and cheap — four `&str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadingRef<'a> {
    pub ags_type: &'a str,
    pub unit: &'a str,
    pub status: &'a str,
    pub desc: &'a str,
}

/// A borrowed view of one group's metadata (mirrors [`GroupMeta`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupRef<'a> {
    pub parent: &'a str,
    pub desc: &'a str,
}

// `DictVersion` (the bundled-edition enum) + `as_str` / `ALL` / `from_edition` /
// `tables` / `FALLBACK` are GENERATED from ags_dictionary.json by build.rs — see
// the `include!` below. Add an edition to the official source dictionaries +
// regenerate the union, never here.

/// *How* `resolve_dict_version` arrived at the edition — so a caller
/// (the corpus-QA harness) can REPORT it without re-deriving the
/// policy. The dogfood blind spot this fixes: a genuine `TRAN_AGS`
/// edition and the O-30 fallback both resolve to `4.1.1`, so a plain
/// version string can't tell "294 real 4.1.1 files" from "294 files
/// with no parseable `TRAN_AGS`". Cross-ref O-30.
///
/// Serde-able because an `.ags.idx` certificate records it: a cert that vouched for a
/// verdict has to be able to say which dictionary reached it AND how that dictionary was
/// chosen, without re-parsing the file — re-parsing being the cost the certificate
/// exists to avoid. The wire tokens are the `as_str` tokens; `serialized_names_match_as_str`
/// pins that, so the JSON and the reported string can never drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DictResolution {
    /// Explicit `CheckOptions::dict_version` override.
    #[serde(rename = "forced")]
    Forced,
    /// `TRAN_AGS` exactly matched a bundled edition (python-parity).
    #[serde(rename = "exact")]
    ExactTranAgs,
    /// Newest bundled patch of the `TRAN_AGS` major.minor (O-30).
    #[serde(rename = "guessed")]
    GuessedPatch,
    /// Missing / unparsable / unrecognised 4.x / future → FALLBACK.
    #[serde(rename = "fallback")]
    Fallback,
    /// A custom `--dict` overlay whose base edition was detected structurally
    /// from the dictionary itself (#568 §2 `detect_base`).
    #[serde(rename = "structural")]
    StructuralBase,
    /// A custom `--dict --dict-replace` full replacement — no base edition
    /// contributes (#568 §2).
    #[serde(rename = "replacement")]
    Replacement,
}

impl DictResolution {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DictResolution::Forced => "forced",
            DictResolution::ExactTranAgs => "exact",
            DictResolution::GuessedPatch => "guessed",
            DictResolution::Fallback => "fallback",
            DictResolution::StructuralBase => "structural",
            DictResolution::Replacement => "replacement",
        }
    }
}

#[cfg(test)]
mod resolution_tests {
    use super::DictResolution as R;

    #[test]
    fn serialized_names_match_as_str() {
        // One value domain, two producers (serde and `as_str`). A rename on one side
        // that isn't mirrored on the other would put a different token in the cert JSON
        // than the surfaces report — and the cert would then name a resolution no
        // consumer recognises.
        for r in [
            R::Forced,
            R::ExactTranAgs,
            R::GuessedPatch,
            R::Fallback,
            R::StructuralBase,
            R::Replacement,
        ] {
            let json = serde_json::to_string(&r).expect("serialises");
            assert_eq!(json, format!("\"{}\"", r.as_str()));
            let back: R = serde_json::from_str(&json).expect("round-trips");
            assert_eq!(back, r);
        }
    }
}

// Generated by build.rs into OUT_DIR from ags_dictionary.json. Defines:
//   pub enum DictVersion { … } + as_str / ALL / from_edition / tables
//   pub const FALLBACK: DictVersion   (the python-parity auto-select fallback,
//                                       sourced from the union's fallback_edition)
//   per version: DICT_<v>_HEADINGS / _GROUPS / _GROUP_HEADINGS / _ABBRS / _TRAN_AGS
//
// An outer `#[allow(clippy::pedantic)]` directly on the `include!` is a no-op
// (clippy still flags the generated items — `include!` isn't attribute-macro-aware,
// so rustc reports the attribute itself as unused). Wrapping in a private module
// scopes the allow correctly; fixes would vanish on the next build.rs regen anyway
// (chore/clippy-pedantic). `pub use` (not `pub(crate)`) preserves the existing
// `dict::DictVersion` / `dict::FALLBACK` paths other crates (validator, laterite-py,
// laterite-node, …) depend on.
#[allow(clippy::pedantic)]
mod dict_data {
    // The generated statics instantiate `DictEntry`/`GroupMeta`, defined
    // in the parent module.
    use super::{DictEntry, GroupMeta};

    include!(concat!(env!("OUT_DIR"), "/dict_data.rs"));
}
pub use dict_data::*;

/// Composite key for a heading lookup. Group + heading joined by U+001F.
#[must_use]
pub fn heading_key(group: &str, heading: &str) -> String {
    let mut k = String::with_capacity(group.len() + 1 + heading.len());
    k.push_str(group);
    k.push('\u{1f}');
    k.push_str(heading);
    k
}

/// Read-only handle onto one bundled standard dictionary. Cheap to copy
/// (just static map refs + a str). This is today's `Dictionary` verbatim,
/// renamed (#568): the hot path only ever touches this, and `build.rs` /
/// the phf codegen are unchanged.
#[derive(Debug, Clone, Copy)]
pub struct BundledDict {
    /// This edition's bit in an [`EdMask`]. The four tables are SHARED across
    /// editions — one entry per key, carrying one variant per distinct value —
    /// so selecting an edition is masking, not choosing a table.
    bit: EdMask,
    version: DictVersion,
}

/// The variant of `variants` that belongs to `bit`, or `None` if the key is not
/// in this edition at all.
///
/// The scan is over a slice that is length 1 for 89% of heading keys and never
/// exceeds 3 in the bundled data, so this is a couple of predictable compares —
/// cheaper than the cache miss the five separate tables cost by being five times
/// the size. Masks for one key are disjoint (each edition contributes exactly
/// one value), so the first match is the only match.
#[inline]
fn pick<T>(variants: &'static [(EdMask, T)], bit: EdMask) -> Option<&'static T> {
    variants
        .iter()
        .find_map(|(mask, v)| (mask & bit != 0).then_some(v))
}

/// Whether `key` exists in this edition of `table` — the masked equivalent of
/// `phf::Map::contains_key`.
#[inline]
fn has<T>(
    table: &'static phf::Map<&'static str, &'static [(EdMask, T)]>,
    key: &str,
    bit: EdMask,
) -> bool {
    table.get(key).is_some_and(|v| pick(v, bit).is_some())
}

impl BundledDict {
    #[must_use]
    pub fn bundled(version: DictVersion) -> Self {
        BundledDict {
            bit: version.bit(),
            version,
        }
    }

    /// Keys of `table` present in this edition. The shared tables hold the UNION
    /// of every edition's keys, so an unmasked `.keys()` would report headings
    /// and groups an older edition never had.
    fn keys_in_edition<T: 'static>(
        self,
        table: &'static phf::Map<&'static str, &'static [(EdMask, T)]>,
    ) -> impl Iterator<Item = &'static str> {
        let bit = self.bit;
        table
            .entries()
            .filter_map(move |(k, v)| pick(v, bit).map(|_| *k))
    }

    // The bundled-arm primitives. Each returns a `'static` result (the data lives
    // in `.rodata`, not in `&self`), so callers may invoke them on a temporary
    // `BundledDict` and keep the result. The enum widens `'static` to `'a`.

    fn heading_ref(self, key: &str) -> Option<HeadingRef<'static>> {
        let e = pick(HEADINGS.get(key)?, self.bit)?;
        Some(HeadingRef {
            ags_type: e.ags_type,
            unit: e.unit,
            status: e.status,
            desc: e.desc,
        })
    }

    fn group_ref(self, code: &str) -> Option<GroupRef<'static>> {
        let g = pick(GROUPS.get(code)?, self.bit)?;
        Some(GroupRef {
            parent: g.parent,
            desc: g.desc,
        })
    }

    fn group_headings_slice(self, code: &str) -> &'static [&'static str] {
        GROUP_HEADINGS
            .get(code)
            .and_then(|v| pick(v, self.bit))
            .copied()
            .unwrap_or(&[])
    }

    fn abbr(self, key: &str) -> Option<&'static str> {
        pick(ABBRS.get(key)?, self.bit).copied()
    }

    fn tran_ags(self) -> &'static str {
        TRAN_AGS[self.version.index()]
    }

    fn group_code_keys(self) -> impl Iterator<Item = &'static str> {
        self.keys_in_edition(&GROUPS)
    }

    /// Every heading in this edition as `(name, HeadingRef)` — the composite phf
    /// key `"GROUP\u{1f}HEADING"` reduced to its heading name. `detect_base`
    /// (#568 §2) scores a custom dict's headings against an edition by NAME,
    /// ignoring which group each lives under, so it needs name + (type, status)
    /// flat. A name that appears under several groups yields one tuple each.
    pub(crate) fn iter_headings(self) -> impl Iterator<Item = (&'static str, HeadingRef<'static>)> {
        let bit = self.bit;
        HEADINGS.entries().filter_map(move |(k, variants)| {
            let e = pick(variants, bit)?;
            let name = k.rsplit('\u{1f}').next().unwrap_or(k);
            Some((
                name,
                HeadingRef {
                    ags_type: e.ags_type,
                    unit: e.unit,
                    status: e.status,
                    desc: e.desc,
                },
            ))
        })
    }

    /// How many headings / groups this edition defines. Only the tests need
    /// these; the shared tables make `.len()` an edition-blind answer, so the
    /// counts have to be masked like everything else.
    #[cfg(test)]
    fn heading_count(self) -> usize {
        self.keys_in_edition(&HEADINGS).count()
    }

    #[cfg(test)]
    fn group_count(self) -> usize {
        self.group_code_keys().count()
    }
}

/// Read-only dictionary handle. `Bundled` is the zero-cost `&'static` phf edition
/// used everywhere; `Layered` overlays a runtime-owned custom-dict delta (#568).
/// `Copy` (both arms are refs/statics), so every by-value `dict: Dictionary` site
/// is unaffected by the enum change.
#[derive(Debug, Clone, Copy)]
pub enum Dictionary<'a> {
    Bundled(BundledDict),
    Layered {
        base: BundledDict,
        delta: &'a OwnedDelta,
    },
}

impl<'a> Dictionary<'a> {
    /// A bundled standard edition — the constructor every existing surface uses.
    /// Returns `Dictionary<'static>` (borrows nothing), so `resolve_dict_version`,
    /// wasm/node/py and `fixes.rs` are unchanged.
    #[must_use]
    pub fn bundled(version: DictVersion) -> Dictionary<'static> {
        Dictionary::Bundled(BundledDict::bundled(version))
    }

    /// Overlay a runtime custom-dict `delta` on its (already-detected) base edition.
    /// Nothing constructs a delta before #568 Phase 2; the arm compiles now.
    #[must_use]
    pub fn layered(delta: &'a OwnedDelta) -> Dictionary<'a> {
        Dictionary::Layered {
            base: BundledDict::bundled(delta.base_version),
            delta,
        }
    }

    /// The base bundled edition (Copy handle) under either arm.
    fn base(&self) -> BundledDict {
        match self {
            Dictionary::Bundled(b) => *b,
            Dictionary::Layered { base, .. } => *base,
        }
    }

    /// The overlay delta, if this is a layered dictionary.
    fn delta(&self) -> Option<&'a OwnedDelta> {
        match self {
            Dictionary::Bundled(_) => None,
            Dictionary::Layered { delta, .. } => Some(delta),
        }
    }

    /// Whether lookups fall through to the base (true for bundled + overlay;
    /// false only for a full-replacement custom dict).
    fn falls_through(&self) -> bool {
        self.delta().is_none_or(|d| d.fall_through)
    }

    /// Canonical description for an abbreviation in the standard ABBR table.
    /// v1: always the base's picklist — custom ABBR is a v2 cut (#568 §6).
    #[must_use]
    pub fn abbr_desc(&self, heading: &str, code: &str) -> Option<&'a str> {
        let mut k = String::with_capacity(heading.len() + 1 + code.len());
        k.push_str(heading);
        k.push('\u{1f}');
        k.push_str(code);
        self.base().abbr(&k)
    }

    /// Every `ABBR_CODE` the standard ABBR table lists for `heading` (the
    /// picklist), in the map's iteration order; empty if none. v1: base only.
    #[must_use]
    pub fn abbr_codes(&self, heading: &str) -> Vec<&'a str> {
        let base = self.base();
        base.keys_in_edition(&ABBRS)
            .filter_map(|k| {
                let (h, code) = k.split_once('\u{1f}')?;
                (h == heading).then_some(code)
            })
            .collect()
    }

    #[must_use]
    pub fn version(&self) -> DictVersion {
        self.base().version
    }

    /// The `TRAN_AGS` value this dictionary edition expects (Rule 14). v1: the
    /// base's — custom `TRAN_AGS` is a v2 cut (#568 §6).
    #[must_use]
    pub fn tran_ags(self) -> &'a str {
        self.base().tran_ags()
    }

    /// Definition for `GROUP.HEADING`, or `None` if not in the effective
    /// dictionary. Layered: the delta's override first, then the base.
    #[must_use]
    pub fn heading(&self, group: &str, heading: &str) -> Option<HeadingRef<'a>> {
        let key = heading_key(group, heading);
        if let Some(d) = self.delta() {
            if let Some(h) = d.heading(&key) {
                return Some(h);
            }
            if !d.fall_through {
                return None;
            }
        }
        self.base().heading_ref(&key)
    }

    /// Metadata for a group code, or `None` if not a standard/overlaid group.
    #[must_use]
    pub fn group(&self, code: &str) -> Option<GroupRef<'a>> {
        if let Some(d) = self.delta() {
            if let Some(g) = d.group(code) {
                return Some(g);
            }
            if !d.fall_through {
                return None;
            }
        }
        self.base().group_ref(code)
    }

    /// Every group code in the effective dictionary (unordered). Callers that
    /// need a stable order should sort.
    pub fn group_codes(&self) -> impl Iterator<Item = &'a str> + 'a {
        let base = self.base();
        let delta = self.delta();
        let include_base = self.falls_through();
        let base_codes = base.group_code_keys();
        let base_iter = include_base
            .then_some(base_codes)
            .into_iter()
            .flatten()
            .map(|s| -> &'a str { s });
        let delta_iter = delta.into_iter().flat_map(move |d| {
            // Replacement: every delta group. Overlay: only brand-new codes
            // (overrides of a base group are already listed by base_iter).
            // `has` is the masked contains_key: a group the base table carries
            // for a LATER edition must not suppress the delta's own here.
            d.groups
                .keys()
                .filter(move |c| !d.fall_through || !has(&GROUPS, c.as_str(), base.bit))
                .map(|s| -> &'a str { s.as_str() })
        });
        base_iter.chain(delta_iter)
    }

    /// This group's headings in canonical dictionary order (Rule 7). Empty for
    /// an unknown group. Layered: the base's order with the delta's added names
    /// appended for a touched group; a new group returns the delta's alone.
    /// `Cow::Borrowed` (zero-cost) for an untouched/bundled group.
    pub fn group_headings(&self, code: &str) -> Cow<'a, [&'a str]> {
        let base = self.base();
        let base_slice = base.group_headings_slice(code);
        match self.delta() {
            None => Cow::Borrowed(base_slice),
            Some(d) => {
                let added = d.added_headings(code);
                if d.fall_through && added.is_empty() {
                    Cow::Borrowed(base_slice)
                } else if !d.fall_through && added.is_empty() {
                    Cow::Owned(Vec::new())
                } else {
                    let mut v: Vec<&'a str> = Vec::new();
                    if d.fall_through {
                        v.extend(base_slice.iter().copied().map(|s| -> &'a str { s }));
                    }
                    v.extend(added.iter().map(String::as_str));
                    Cow::Owned(v)
                }
            }
        }
    }

    /// Every heading name defined anywhere in the effective dictionary (across
    /// all groups, cross-group borrows repeated). Rule `19b_3` (V8) needs "is this
    /// heading defined under *some* group".
    pub fn all_heading_names(&self) -> impl Iterator<Item = &'a str> + 'a {
        let base = self.base();
        let delta = self.delta();
        let include_base = self.falls_through();
        let bit = base.bit;
        let base_names = GROUP_HEADINGS
            .values()
            .filter_map(move |v| pick(v, bit))
            .flat_map(|hs| hs.iter().copied());
        let base_iter = include_base
            .then_some(base_names)
            .into_iter()
            .flatten()
            .map(|s| -> &'a str { s });
        let delta_iter = delta.into_iter().flat_map(|d| {
            d.group_headings
                .values()
                .flat_map(|hs| hs.iter().map(String::as_str))
        });
        base_iter.chain(delta_iter)
    }
}

// --- Serialisable dictionary snapshot (the `dictionary(edition)` accessor) ----
// One shared `{ags_edition, groups:[{code, contents, parent, headings:[…]}]}`
// view of a bundled edition, so the browser (wasm), `laterite.registry.
// dictionary()` (PyO3) and Node's `registry.dictionary()` render it identically
// from ONE builder instead of three copies (#294 F#6). Serialize only — each
// surface does its own JSON (serde_json / serde_wasm_bindgen).

#[derive(serde::Serialize)]
pub struct DictHeadingDto {
    pub name: String,
    pub status: String,
    #[serde(rename = "type")]
    pub ags_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct DictGroupDto {
    pub code: String,
    /// The group's standard description (its "contents"/name).
    pub contents: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub headings: Vec<DictHeadingDto>,
}

#[derive(serde::Serialize)]
pub struct DictionaryDto {
    pub ags_edition: String,
    pub groups: Vec<DictGroupDto>,
}

/// Build the serialisable snapshot of one bundled standard-dictionary edition:
/// groups sorted by code, each group's headings in canonical dictionary order.
/// The single source the wasm / PyO3 / Node `dictionary(edition)` accessors share.
#[must_use]
pub fn dictionary_dto(version: DictVersion) -> DictionaryDto {
    let d = Dictionary::bundled(version);
    let mut codes: Vec<&'static str> = d.group_codes().collect();
    codes.sort_unstable();
    let groups = codes
        .into_iter()
        .map(|code| {
            let gm = d.group(code);
            let headings = d
                .group_headings(code)
                .iter()
                .map(|&h| {
                    let e = d.heading(code, h);
                    DictHeadingDto {
                        name: h.to_string(),
                        status: e.map_or("", |x| x.status).to_string(),
                        ags_type: e.map_or("", |x| x.ags_type).to_string(),
                        unit: e
                            .map(|x| x.unit)
                            .filter(|u| !u.is_empty())
                            .map(str::to_string),
                        description: e.map_or("", |x| x.desc).to_string(),
                    }
                })
                .collect();
            DictGroupDto {
                code: code.to_string(),
                contents: gm.map_or("", |m| m.desc).to_string(),
                parent: gm
                    .map(|m| m.parent)
                    .filter(|p| !p.is_empty())
                    .map(str::to_string),
                headings,
            }
        })
        .collect();
    DictionaryDto {
        ags_edition: version.as_str().to_string(),
        groups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_dicts_load_and_have_plausible_size() {
        for v in [
            DictVersion::V4_0_3,
            DictVersion::V4_0_4,
            DictVersion::V4_1,
            DictVersion::V4_1_1,
            DictVersion::V4_2,
        ] {
            // The private phf tables live on BundledDict; poke it directly.
            let b = BundledDict::bundled(v);
            // Every bundled AGS4 edition has >50 groups and >1000
            // headings; assert a floor so an empty/broken codegen
            // fails loudly.
            assert!(
                b.group_count() > 50,
                "{:?}: only {} groups",
                v,
                b.group_count()
            );
            assert!(
                b.heading_count() > 1000,
                "{:?}: only {} headings",
                v,
                b.heading_count()
            );
        }
    }

    #[test]
    fn known_headings_resolve() {
        let d = Dictionary::bundled(DictVersion::V4_2);
        let proj_id = d.heading("PROJ", "PROJ_ID").expect("PROJ.PROJ_ID");
        assert_eq!(proj_id.ags_type, "ID");
        assert!(proj_id.status.contains("KEY"));
        // LOCA is a real group with a parent in the dictionary.
        assert!(d.group("LOCA").is_some());
        // A heading that doesn't exist returns None.
        assert!(d.heading("PROJ", "NOT_A_HEADING").is_none());
    }

    #[test]
    fn tran_ags_matches_version() {
        for v in [
            DictVersion::V4_0_3,
            DictVersion::V4_0_4,
            DictVersion::V4_1,
            DictVersion::V4_1_1,
            DictVersion::V4_2,
        ] {
            // The dictionary's own embedded TRAN_AGS must equal the
            // variant's canonical string (Rule 14 / auto-selection).
            assert_eq!(Dictionary::bundled(v).tran_ags(), v.as_str(), "{v:?}");
        }
    }

    #[test]
    fn group_headings_are_in_dictionary_order() {
        let d = Dictionary::bundled(DictVersion::V4_2);
        let proj = d.group_headings("PROJ");
        // PROJ_ID is the KEY and must precede PROJ_NAME in the dict.
        let id = proj.iter().position(|h| *h == "PROJ_ID").expect("PROJ_ID");
        let name = proj
            .iter()
            .position(|h| *h == "PROJ_NAME")
            .expect("PROJ_NAME");
        assert!(id < name, "PROJ order wrong: {proj:?}");
        // Every ordered name must also resolve in the membership map
        // (the two tables are built from the same rows).
        for h in proj.iter() {
            assert!(
                d.heading("PROJ", h).is_some(),
                "{h} missing from headings map"
            );
        }
        // Unknown group → empty slice, never a panic.
        assert!(d.group_headings("ZZZZ").is_empty());
    }

    #[test]
    fn dict_resolution_as_str_round_trips() {
        // The harness serialises the resolution kind; every variant must
        // map to its stable token.
        assert_eq!(DictResolution::Forced.as_str(), "forced");
        assert_eq!(DictResolution::ExactTranAgs.as_str(), "exact");
        assert_eq!(DictResolution::GuessedPatch.as_str(), "guessed");
        assert_eq!(DictResolution::Fallback.as_str(), "fallback");
    }

    #[test]
    fn version_and_group_codes_expose_the_edition() {
        let d = Dictionary::bundled(DictVersion::V4_1);
        assert_eq!(d.version(), DictVersion::V4_1);
        // group_codes() is the unordered key view the web reference UI
        // serialises; it must include the well-known groups and agree in
        // length with the group count.
        let codes: Vec<&str> = d.group_codes().collect();
        let b = BundledDict::bundled(DictVersion::V4_1);
        assert_eq!(codes.len(), b.group_count());
        assert!(codes.contains(&"PROJ"));
        assert!(codes.contains(&"LOCA"));
    }

    #[test]
    fn all_heading_names_includes_cross_group_borrows() {
        // Rule 19b_3 keys off this: a borrowed heading (LOCA_ID under
        // SAMP) must appear because it's "defined somewhere".
        let d = Dictionary::bundled(DictVersion::V4_2);
        let names: std::collections::HashSet<&str> = d.all_heading_names().collect();
        assert!(names.contains("LOCA_ID"));
        assert!(names.contains("PROJ_ID"));
    }

    #[test]
    fn abbr_desc_resolves_a_standard_abbreviation() {
        // The FYI-16 path looks up canonical ABBR descriptions keyed by
        // (ABBR_HDNG, ABBR_CODE). `ARTW_TYPE/DRY → "Dry test"` is a
        // verbatim row in the bundled 4.2 ABBR group (data/
        // Standard_dictionary_v4_2.ags). An unknown pair → None.
        let d = Dictionary::bundled(DictVersion::V4_2);
        assert_eq!(d.abbr_desc("ARTW_TYPE", "DRY"), Some("Dry test"));
        assert!(d.abbr_desc("NOPE_HDNG", "ZZ").is_none());
    }

    #[test]
    fn abbr_codes_enumerates_a_picklist() {
        let d = Dictionary::bundled(DictVersion::V4_2);
        // SAMP_TYPE is a substantial picklist; every enumerated code must
        // round-trip through abbr_desc (same composite key), and "U"
        // (Undisturbed) is a verbatim row.
        let codes = d.abbr_codes("SAMP_TYPE");
        assert!(codes.len() > 1, "SAMP_TYPE should have several codes");
        assert!(
            codes.contains(&"U"),
            "SAMP_TYPE should include U, got {codes:?}"
        );
        for c in &codes {
            assert!(
                d.abbr_desc("SAMP_TYPE", c).is_some(),
                "{c} must resolve a desc"
            );
        }
        assert!(d.abbr_codes("NOPE_HDNG").is_empty());
    }

    // ---- layered (custom-dict overlay) paths ----
    // The bundled tests above never build a `Dictionary::Layered`, so the delta
    // arms of heading/group/group_codes/group_headings went unexercised at the
    // reference level (they are covered downstream by the validator, which this
    // crate's own sweep does not run). Build a delta directly — its fields are
    // public — and pin both the overlay (fall-through) and replacement forms.
    use crate::overlay::{OwnedDelta, OwnedDictEntry, OwnedGroupMeta};
    use std::collections::HashMap;

    fn entry(ags_type: &str, status: &str, unit: &str) -> OwnedDictEntry {
        OwnedDictEntry {
            ags_type: ags_type.into(),
            status: status.into(),
            unit: unit.into(),
            desc: String::new(),
        }
    }

    // A delta over 4.2: a new XTRA group, an override of LOCA's metadata and
    // LOCA_ID's type, and a heading appended to LOCA — enough to reach every
    // layered arm. `fall_through` selects overlay vs full replacement.
    fn sample_delta(fall_through: bool) -> OwnedDelta {
        OwnedDelta {
            base_version: DictVersion::V4_2,
            fall_through,
            groups: HashMap::from([
                (
                    "XTRA".to_string(),
                    OwnedGroupMeta {
                        parent: "LOCA".into(),
                        desc: "Extra".into(),
                    },
                ),
                (
                    "LOCA".to_string(),
                    OwnedGroupMeta {
                        parent: "PROJ".into(),
                        desc: "Overridden".into(),
                    },
                ),
            ]),
            headings: HashMap::from([
                (heading_key("XTRA", "XTRA_ID"), entry("ID", "KEY", "")),
                (heading_key("LOCA", "LOCA_ID"), entry("XOVR", "KEY", "")),
                (heading_key("LOCA", "LOCA_EXTRA"), entry("X", "OTHER", "m")),
            ]),
            group_headings: HashMap::from([
                ("XTRA".to_string(), vec!["XTRA_ID".to_string()]),
                ("LOCA".to_string(), vec!["LOCA_EXTRA".to_string()]),
            ]),
            abbrs: HashMap::new(),
        }
    }

    #[test]
    fn layered_overlay_adds_new_groups_overrides_and_falls_through() {
        let delta = sample_delta(true);
        let d = Dictionary::layered(&delta);
        // a brand-new group and heading resolve through the delta
        assert_eq!(d.group("XTRA").map(|g| g.parent), Some("LOCA"));
        assert_eq!(d.heading("XTRA", "XTRA_ID").map(|h| h.ags_type), Some("ID"));
        // the delta OVERRIDES a base heading (delta wins over the base)
        assert_eq!(
            d.heading("LOCA", "LOCA_ID").map(|h| h.ags_type),
            Some("XOVR")
        );
        // fall-through keeps the base groups visible, and an overridden base group
        // is listed once (by the base), not duplicated by the delta
        let codes: Vec<&str> = d.group_codes().collect();
        assert!(codes.contains(&"XTRA"), "new code missing: {codes:?}");
        assert!(codes.contains(&"PROJ"), "base code missing (fall-through)");
        assert_eq!(
            codes.iter().filter(|c| **c == "LOCA").count(),
            1,
            "override duplicated LOCA: {codes:?}"
        );
        // a touched base group keeps its base headings and appends the delta's
        let loca_h = d.group_headings("LOCA");
        assert!(loca_h.contains(&"LOCA_ID"), "base heading lost: {loca_h:?}");
        assert!(
            loca_h.contains(&"LOCA_EXTRA"),
            "delta heading not appended: {loca_h:?}"
        );
    }

    #[test]
    fn layered_replacement_hides_the_base() {
        let repl = sample_delta(false);
        let d = Dictionary::layered(&repl);
        // replacement: base-only lookups return None, base codes vanish
        assert!(
            d.heading("PROJ", "PROJ_ID").is_none(),
            "base heading leaked"
        );
        assert!(d.group("PROJ").is_none(), "base group leaked");
        let codes: Vec<&str> = d.group_codes().collect();
        assert!(codes.contains(&"XTRA"));
        assert!(
            !codes.contains(&"PROJ"),
            "replacement must hide the base: {codes:?}"
        );
        // a delta group's headings are the delta's alone (no base prefix)
        assert_eq!(d.group_headings("XTRA").as_ref(), &["XTRA_ID"]);
        // an untouched base group is empty under replacement
        assert!(d.group_headings("SAMP").is_empty());
    }

    #[test]
    fn dictionary_dto_carries_units_and_parents() {
        let dto = dictionary_dto(DictVersion::V4_2);
        let loca = dto
            .groups
            .iter()
            .find(|g| g.code == "LOCA")
            .expect("LOCA in the DTO");
        // LOCA hangs off PROJ — a non-empty parent survives as Some
        assert_eq!(loca.parent.as_deref(), Some("PROJ"));
        // at least one LOCA heading keeps its unit (units must not be dropped)
        assert!(
            loca.headings
                .iter()
                .any(|h| h.unit.as_deref().is_some_and(|u| !u.is_empty())),
            "no LOCA heading kept its unit"
        );
    }
}
