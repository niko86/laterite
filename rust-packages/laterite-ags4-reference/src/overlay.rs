//! Runtime-owned dictionary delta for the `--dict` override (laterite-dev#568).
//!
//! The bundled path stays `&'static` phf (zero startup, no parse). A custom
//! dictionary is expressed as a **sparse overlay**: owned data for the *delta*
//! only — the groups/headings a client adds or overrides — layered over a
//! bundled base edition detected from the dictionary itself. `Dictionary::Layered`
//! borrows an `&OwnedDelta` that lives on the caller's stack for one validation.
//!
//! **Phase 1 (this file): the owned types + their lookup helpers only.** Nothing
//! constructs an `OwnedDelta` yet — `CustomDict::build_delta` (Phase 2) parses a
//! `.ags`/JSON dictionary, detects the base, and diffs it into this shape. The
//! `Layered` arm of every `Dictionary` method is written and compiles now so the
//! wiring in Phase 2/3 is a construction site, not a type change.

use std::collections::HashMap;

use encoding_rs::Encoding;
use sha2::{Digest, Sha256};

use crate::dict::{
    BundledDict, DictResolution, DictVersion, Dictionary, GroupRef, HeadingRef, heading_key,
};
use crate::dict_read::read_ags_dict;
use crate::union::DictionaryFile;

/// One added/overridden heading, owned (parsed at runtime, not `&'static`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDictEntry {
    pub ags_type: String,
    pub unit: String,
    pub status: String,
    pub desc: String,
}

/// One added/overridden group, owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedGroupMeta {
    /// Parent group code; `""` for a root group (mirrors [`crate::dict::GroupMeta`]).
    pub parent: String,
    pub desc: String,
}

/// The sparse delta a custom dictionary contributes over its bundled base.
///
/// Keys mirror the bundled tables: heading keys are the `"GROUP\u{1f}HEADING"`
/// composite ([`crate::dict::heading_key`]); group keys are the 4-letter code.
/// Only the *differences* from the base live here — a client adding one group to
/// a 4.2-shaped dictionary yields a delta with one group, not 174.
#[derive(Debug, Clone)]
pub struct OwnedDelta {
    /// The bundled edition this delta layers over — a property of the dictionary
    /// itself (detected once at parse, not per delivery file). See laterite-dev#568 §2.
    pub base_version: DictVersion,
    /// When `false`, this is a **full replacement**: lookups never fall through to
    /// the base, so the base contributes nothing.
    pub fall_through: bool,
    /// Added/overridden group metadata, by group code.
    pub groups: HashMap<String, OwnedGroupMeta>,
    /// Added/overridden heading definitions, by `"GROUP\u{1f}HEADING"` composite.
    pub headings: HashMap<String, OwnedDictEntry>,
    /// Touched groups → the delta-added heading NAMES in dictionary order, so
    /// [`crate::dict::Dictionary::group_headings`] can append them after the base's.
    pub group_headings: HashMap<String, Vec<String>>,
    /// Custom ABBR picklists — empty in v1 (a v2 cut, laterite-dev#568 §6). Reserved so the
    /// on-disk/owned shape is stable when v2 populates it.
    pub abbrs: HashMap<String, String>,
}

impl OwnedDelta {
    /// The delta's definition for a `"GROUP\u{1f}HEADING"` key, if it overrides/adds one.
    pub(crate) fn heading(&self, key: &str) -> Option<HeadingRef<'_>> {
        self.headings.get(key).map(|e| HeadingRef {
            ags_type: &e.ags_type,
            unit: &e.unit,
            status: &e.status,
            desc: &e.desc,
        })
    }

    /// The delta's metadata for a group code, if it overrides/adds one.
    pub(crate) fn group(&self, code: &str) -> Option<GroupRef<'_>> {
        self.groups.get(code).map(|g| GroupRef {
            parent: &g.parent,
            desc: &g.desc,
        })
    }

    /// Delta-added heading names for a group, in order (empty if the group is untouched).
    pub(crate) fn added_headings(&self, code: &str) -> &[String] {
        self.group_headings.get(code).map_or(&[], Vec::as_slice)
    }
}

// ─── Custom-dictionary parsing (laterite-dev#568 Phase 2) ───────────────────────────────
//
// One dispatcher (`parse_dict`) over two accepted formats. It produces a
// base-resolved [`CustomDict`] whose `{ base_version, hash }` are precomputed
// here at the surface boundary — so the certificate can record which dictionary
// reached a verdict with no delivery bytes in hand (laterite-dev#568 §4). Nothing in the
// validator calls this yet; Phase 3 wires it.

/// Input format of a `--dict` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictFormat {
    /// AGS4 file carrying a DICT group.
    Ags,
    /// JSON in `ags_dictionary.json`'s `{ groups: { CODE: {...} } }` shape.
    Json,
    /// Sniff: first non-whitespace byte `{` (after any BOM) ⇒ JSON, else AGS.
    Auto,
}

/// How the base edition for an overlay is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseSpec {
    /// Detect structurally from the dictionary itself ([`detect_base`]).
    Auto,
    /// Caller forced a base edition (`--dict-version`).
    Force(DictVersion),
    /// No base — the dict fully replaces the standard (`--dict-replace`).
    Replace,
}

/// Why a `--dict` file could not be turned into a usable dictionary. Every
/// variant names the DICTIONARY's own problem (never the delivery file), so a
/// surface reports it before any data rule runs (laterite-dev#568 §4). `line` is the
/// 1-indexed source line for the `.ags` path, `0` for the JSON path (serde owns
/// JSON syntax locations via [`DictError::MalformedJson`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictError {
    /// The file parsed but declared no groups/headings.
    Empty,
    /// An `.ags` dict with no readable DICT group.
    NotADictionary,
    /// The `.ags` tokenizer rejected the bytes.
    Parse(String),
    /// A DICT HEADING row with no heading name.
    BadGroupRowArity { line: u32 },
    /// A status token that is not KEY / REQUIRED / OTHER.
    UnknownStatus { line: u32, token: String },
    /// A data-type token that is not a recognised AGS type.
    UnknownType { line: u32, token: String },
    /// A group whose declared parent is in neither the base nor the delta.
    UnknownParent { group: String, parent: String },
    /// A group declaring the same heading name twice.
    DuplicateHeading { group: String, heading: String },
    /// JSON that serde could not deserialise.
    MalformedJson {
        msg: String,
        line: usize,
        col: usize,
    },
}

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictError::Empty => write!(f, "dictionary declares no groups or headings"),
            DictError::NotADictionary => {
                write!(f, "no readable DICT group (not an AGS4 dictionary)")
            }
            DictError::Parse(m) => write!(f, "could not parse dictionary: {m}"),
            DictError::BadGroupRowArity { line } => {
                write!(f, "DICT HEADING row with no heading name (line {line})")
            }
            DictError::UnknownStatus { line, token } => {
                write!(
                    f,
                    "unknown status {token:?} (line {line}); expected KEY/REQUIRED/OTHER"
                )
            }
            DictError::UnknownType { line, token } => {
                write!(f, "unknown AGS data type {token:?} (line {line})")
            }
            DictError::UnknownParent { group, parent } => {
                write!(
                    f,
                    "group {group} names parent {parent:?}, which no edition or the dict itself defines"
                )
            }
            DictError::DuplicateHeading { group, heading } => {
                write!(f, "group {group} declares heading {heading:?} twice")
            }
            DictError::MalformedJson { msg, line, col } => {
                write!(
                    f,
                    "malformed JSON dictionary at line {line} col {col}: {msg}"
                )
            }
        }
    }
}

impl std::error::Error for DictError {}

/// Is `s` a recognised `DICT_STAT`? Empty is accepted (treated as OTHER downstream).
pub(crate) fn valid_status(s: &str) -> bool {
    s.is_empty()
        || s.eq_ignore_ascii_case("KEY")
        || s.eq_ignore_ascii_case("REQUIRED")
        || s.eq_ignore_ascii_case("OTHER")
}

/// Is `t` a recognised AGS data type? Empty is accepted (stored, treated as
/// String downstream). The authority is `laterite_ags4_types::canonical_type`, the
/// same one the read codec + DDL builder trust — so the dict validator and the
/// value casters agree on what a type IS.
pub(crate) fn valid_type(t: &str) -> bool {
    t.is_empty() || laterite_ags4_types::canonical_type(t).is_some()
}

/// A parsed, base-resolved custom dictionary. Built once at the surface boundary
/// (`parse_dict`) and stored on `CheckOptions`; `build_delta` diffs it into the
/// stack-local [`OwnedDelta`] a [`Dictionary::Layered`] borrows for one
/// validation. Owned + `Clone` so it can live on the options struct.
#[derive(Debug, Clone)]
pub struct CustomDict {
    file: DictionaryFile,
    /// The bundled edition this dict layers over (forced, detected, or — for a
    /// full replacement — the latest, unused since nothing falls through).
    pub base_version: DictVersion,
    /// How the base was chosen — recorded on the certificate (laterite-dev#568 §4).
    pub resolution: DictResolution,
    /// `false` only for a full replacement: lookups then never see the base.
    pub fall_through: bool,
    /// `detect_base` agreement score in `[0,1]`; `1.0` when forced/replacement.
    pub base_score: f32,
    /// Human label for the cert warning — a declared name or filename basename,
    /// never a path.
    pub name: String,
    /// SHA-256 over (normalised delta ⊕ base edition ⊕ mode). Precomputed so the
    /// cert can record it with no delivery bytes in hand (laterite-dev#568 §4).
    pub hash: [u8; 32],
}

/// The winning base edition and why, from [`detect_base`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct BaseMatch {
    pub version: DictVersion,
    pub score: f32,
    pub signal: usize,
}

/// Parse a custom dictionary and resolve its base edition — the one entry point
/// every surface funnels bytes through.
pub fn parse_dict(
    bytes: &[u8],
    fmt: DictFormat,
    enc: &'static Encoding,
    base: BaseSpec,
    name: &str,
) -> Result<CustomDict, DictError> {
    let resolved = match fmt {
        DictFormat::Auto if sniff_json(bytes) => DictFormat::Json,
        DictFormat::Auto => DictFormat::Ags,
        other => other,
    };
    let file = match resolved {
        DictFormat::Json => parse_json_dict(bytes)?,
        DictFormat::Ags => read_ags_dict(bytes, enc)?,
        DictFormat::Auto => unreachable!("Auto resolved above"),
    };
    if file.groups.is_empty() {
        return Err(DictError::Empty);
    }

    let latest = *DictVersion::ALL
        .last()
        .expect("at least one bundled edition");
    let (base_version, fall_through, resolution, base_score) = match base {
        BaseSpec::Replace => (latest, false, DictResolution::Replacement, 1.0),
        BaseSpec::Force(v) => (v, true, DictResolution::Forced, 1.0),
        BaseSpec::Auto => {
            let m = detect_base(&file);
            (m.version, true, DictResolution::StructuralBase, m.score)
        }
    };

    let mut custom = CustomDict {
        file,
        base_version,
        resolution,
        fall_through,
        base_score,
        name: name.to_string(),
        hash: [0u8; 32],
    };
    // Compute the delta once — it both validates parent existence (the one check
    // that needs the base) and gives us the bytes to fingerprint for the cert.
    let delta = custom.build_delta()?;
    custom.hash = hash_delta(&delta);
    Ok(custom)
}

/// First non-whitespace byte (after an optional UTF-8 BOM) is `{` ⇒ JSON.
fn sniff_json(bytes: &[u8]) -> bool {
    let start = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    start
        .iter()
        .find(|b| !b.is_ascii_whitespace())
        .is_some_and(|b| *b == b'{')
}

/// Deserialise a JSON dict into the shared [`DictionaryFile`] and run the
/// semantic checks serde's syntax layer can't (status/type tokens, duplicates).
fn parse_json_dict(bytes: &[u8]) -> Result<DictionaryFile, DictError> {
    let file: DictionaryFile =
        serde_json::from_slice(bytes).map_err(|e| DictError::MalformedJson {
            msg: e.to_string(),
            line: e.line(),
            col: e.column(),
        })?;
    for (code, g) in &file.groups {
        let mut seen: Vec<&str> = Vec::new();
        for h in &g.headings {
            if !valid_status(&h.status) {
                return Err(DictError::UnknownStatus {
                    line: 0,
                    token: h.status.clone(),
                });
            }
            if !valid_type(&h.ags_type) {
                return Err(DictError::UnknownType {
                    line: 0,
                    token: h.ags_type.clone(),
                });
            }
            if seen.contains(&h.name.as_str()) {
                return Err(DictError::DuplicateHeading {
                    group: code.clone(),
                    heading: h.name.clone(),
                });
            }
            seen.push(&h.name);
        }
    }
    Ok(file)
}

/// Detect the bundled edition a custom dict best overlays — a pure function of
/// the dictionary (laterite-dev#568 §2). Signal is heading-NAME reuse across the whole
/// edition; score is `(type, status)` agreement over that signal. Distinct
/// filters, so a variant that changes a heading's type/status scores `< 1.0`.
/// A dict with no standard-named heading at all (`signal == 0`) defaults to the
/// latest edition — a clean additive overlay, never a replacement.
pub(crate) fn detect_base(file: &DictionaryFile) -> BaseMatch {
    let custom: Vec<(&str, &str, &str)> = file
        .groups
        .values()
        .flat_map(|g| {
            g.headings
                .iter()
                .map(|h| (h.name.as_str(), h.ags_type.as_str(), h.status.as_str()))
        })
        .collect();

    let mut best: Option<BaseMatch> = None;
    for &ed in DictVersion::ALL {
        // name → the (type, status) tuples that name carries in this edition.
        let mut defs: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
        for (name, h) in BundledDict::bundled(ed).iter_headings() {
            defs.entry(name).or_default().push((h.ags_type, h.status));
        }
        let (mut signal, mut agree) = (0usize, 0usize);
        for &(name, ty, st) in &custom {
            if let Some(tuples) = defs.get(name) {
                signal += 1;
                if tuples
                    .iter()
                    .any(|(t2, s2)| t2.eq_ignore_ascii_case(ty) && s2.eq_ignore_ascii_case(st))
                {
                    agree += 1;
                }
            }
        }
        let score = if signal == 0 {
            0.0
        } else {
            agree as f32 / signal as f32
        };
        let cand = BaseMatch {
            version: ed,
            score,
            signal,
        };
        // Iterating oldest → newest, `>=` lets the NEWER edition win an exact
        // score tie (§2 step 3: tie → newer). Distinct editions never share a
        // score AND an index, so the `|S|` tertiary key is belt-and-braces.
        best = match best {
            Some(b) if b.score > cand.score => Some(b),
            _ => Some(cand),
        };
    }

    let best = best.expect("DictVersion::ALL is non-empty");
    if best.signal == 0 {
        // Purely additive, no standard signal → latest edition, clean overlay.
        return BaseMatch {
            version: *DictVersion::ALL.last().expect("non-empty"),
            score: 1.0,
            signal: 0,
        };
    }
    best
}

impl CustomDict {
    /// Diff this dictionary against its (already-resolved) base edition into the
    /// sparse [`OwnedDelta`] a [`Dictionary::Layered`] borrows. Overlay: only
    /// new/overriding groups + headings land in the delta (base supplies the
    /// rest by fall-through). Replacement: everything lands, base excluded. This
    /// is also where parent existence is validated (it needs the base).
    pub fn build_delta(&self) -> Result<OwnedDelta, DictError> {
        let base = Dictionary::bundled(self.base_version);
        let mut delta = OwnedDelta {
            base_version: self.base_version,
            fall_through: self.fall_through,
            groups: HashMap::new(),
            headings: HashMap::new(),
            group_headings: HashMap::new(),
            abbrs: HashMap::new(),
        };

        for (code, g) in &self.file.groups {
            let parent = g.parent.clone().unwrap_or_default();
            if !parent.is_empty() {
                let in_file = self.file.groups.contains_key(&parent);
                let in_base = self.fall_through && base.group(&parent).is_some();
                if !in_file && !in_base {
                    return Err(DictError::UnknownParent {
                        group: code.clone(),
                        parent,
                    });
                }
            }

            // Group metadata. A NEW group (or any group under replacement) takes
            // the file's values. An EXISTING base group is overridden ONLY where
            // the file restates parent/description — so touching a group just to
            // add headings never blanks its inherited description (§3).
            let base_group = self.fall_through.then(|| base.group(code)).flatten();
            match base_group {
                None => {
                    delta.groups.insert(
                        code.clone(),
                        OwnedGroupMeta {
                            parent: parent.clone(),
                            desc: desc_of(g).to_string(),
                        },
                    );
                }
                Some(bg) => {
                    let new_parent = g.parent.as_deref();
                    let new_desc = g.description.as_deref();
                    let p_override = new_parent.is_some_and(|p| p != bg.parent);
                    let d_override = new_desc.is_some_and(|d| d != bg.desc);
                    if p_override || d_override {
                        delta.groups.insert(
                            code.clone(),
                            OwnedGroupMeta {
                                parent: new_parent.unwrap_or(bg.parent).to_string(),
                                desc: new_desc.unwrap_or(bg.desc).to_string(),
                            },
                        );
                    }
                }
            }

            // Headings. `type`/`status` are always restated (they're required in
            // both formats), but `unit`/`description` are optional: a heading that
            // BORROWS a standard name to place it in a group inherits the base's
            // unit/desc rather than blanking them (the sparse-overlay rule that
            // mirrors the group-meta treatment above). Only an entry that actually
            // differs from the base lands in the delta — so a no-op restatement of
            // a standard heading stays out of it.
            let mut added: Vec<String> = Vec::new();
            for h in &g.headings {
                let base_h = self
                    .fall_through
                    .then(|| base.heading(code, &h.name))
                    .flatten();
                let is_new = base_h.is_none();
                let entry = match base_h {
                    None => OwnedDictEntry {
                        ags_type: h.ags_type.clone(),
                        unit: h.unit.clone().unwrap_or_default(),
                        status: h.status.clone(),
                        desc: h.description.clone(),
                    },
                    Some(bh) => OwnedDictEntry {
                        ags_type: h.ags_type.clone(),
                        status: h.status.clone(),
                        unit: match h.unit.as_deref() {
                            Some(u) if !u.is_empty() => u.to_string(),
                            _ => bh.unit.to_string(),
                        },
                        desc: if h.description.is_empty() {
                            bh.desc.to_string()
                        } else {
                            h.description.clone()
                        },
                    },
                };
                let differs = match base_h {
                    None => true,
                    Some(bh) => {
                        entry.ags_type != bh.ags_type
                            || entry.status != bh.status
                            || entry.unit != bh.unit
                            || entry.desc != bh.desc
                    }
                };
                if differs {
                    delta.headings.insert(heading_key(code, &h.name), entry);
                }
                // Replacement lists every name (base order is gone); overlay
                // appends only the names the base doesn't already order.
                if !self.fall_through || is_new {
                    added.push(h.name.clone());
                }
            }
            if !added.is_empty() {
                delta.group_headings.insert(code.clone(), added);
            }
        }
        Ok(delta)
    }
}

/// A group's description, `""` when absent (mirrors `build.rs`'s `'-' → ""`).
fn desc_of(g: &crate::union::DictGroup) -> &str {
    g.description.as_deref().unwrap_or("")
}

/// A deterministic fingerprint of the resolved overlay: base edition, mode, and
/// the delta's groups/headings/order in sorted key order (`HashMaps` are
/// unordered, so every collection is sorted before hashing). Two dicts that
/// resolve to the same effective dictionary hash identically; any change to a
/// type, status, parent, or heading order changes the hash.
fn hash_delta(delta: &OwnedDelta) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(delta.base_version.as_str().as_bytes());
    h.update([u8::from(delta.fall_through)]);

    let mut gcodes: Vec<&String> = delta.groups.keys().collect();
    gcodes.sort();
    for c in gcodes {
        let m = &delta.groups[c];
        h.update(b"G");
        h.update(c.as_bytes());
        h.update([0x1f]);
        h.update(m.parent.as_bytes());
        h.update([0x1f]);
        h.update(m.desc.as_bytes());
        h.update([0x1e]);
    }

    let mut hkeys: Vec<&String> = delta.headings.keys().collect();
    hkeys.sort();
    for k in hkeys {
        let e = &delta.headings[k];
        h.update(b"H");
        h.update(k.as_bytes());
        h.update([0x1f]);
        h.update(e.ags_type.as_bytes());
        h.update([0x1f]);
        h.update(e.unit.as_bytes());
        h.update([0x1f]);
        h.update(e.status.as_bytes());
        h.update([0x1f]);
        h.update(e.desc.as_bytes());
        h.update([0x1e]);
    }

    let mut ocodes: Vec<&String> = delta.group_headings.keys().collect();
    ocodes.sort();
    for c in ocodes {
        h.update(b"O");
        h.update(c.as_bytes());
        h.update([0x1f]);
        for n in &delta.group_headings[c] {
            h.update(n.as_bytes());
            h.update([0x1d]);
        }
        h.update([0x1e]);
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&h.finalize());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::{DictVersion, heading_key};
    use encoding_rs::UTF_8;

    // A bespoke lab-test group hung off SAMP, borrowing SAMP_ID (a standard KEY)
    // and adding two novel headings — the AGS-L shape. Identical content to
    // `ADDITIVE_AGS` so the two formats must resolve to the same delta + hash.
    const ADDITIVE_JSON: &[u8] = br#"{
      "groups": {
        "TEST": {
          "parent": "SAMP",
          "description": "A bespoke test",
          "headings": [
            {"name": "SAMP_ID",  "type": "ID",  "status": "KEY",      "description": "Sample ID"},
            {"name": "TEST_NUM", "type": "ID",  "status": "KEY",      "description": "Test number"},
            {"name": "TEST_VAL", "type": "2DP", "status": "REQUIRED", "unit": "mm", "description": "Value"}
          ]
        }
      }
    }"#;

    const ADDITIVE_AGS: &[u8] = concat!(
        "\"GROUP\",\"DICT\"\r\n",
        "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_DTYP\",\"DICT_UNIT\",\"DICT_DESC\",\"DICT_PGRP\"\r\n",
        "\"UNIT\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n",
        "\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n",
        "\"DATA\",\"GROUP\",\"TEST\",\"\",\"\",\"\",\"\",\"A bespoke test\",\"SAMP\"\r\n",
        "\"DATA\",\"HEADING\",\"TEST\",\"SAMP_ID\",\"KEY\",\"ID\",\"\",\"Sample ID\",\"\"\r\n",
        "\"DATA\",\"HEADING\",\"TEST\",\"TEST_NUM\",\"KEY\",\"ID\",\"\",\"Test number\",\"\"\r\n",
        "\"DATA\",\"HEADING\",\"TEST\",\"TEST_VAL\",\"REQUIRED\",\"2DP\",\"mm\",\"Value\",\"\"\r\n",
    )
    .as_bytes();

    fn parse(bytes: &[u8], fmt: DictFormat, base: BaseSpec) -> CustomDict {
        parse_dict(bytes, fmt, UTF_8, base, "test").expect("parses")
    }

    #[test]
    fn additive_json_overlays_latest_without_replacement() {
        // The B2 regression guard: a purely-additive dict must NOT discard the
        // base. It detects 4.2 (SAMP_ID agrees across editions → newest wins the
        // tie) and stays an overlay (fall_through true), score 1.0.
        let d = parse(ADDITIVE_JSON, DictFormat::Auto, BaseSpec::Auto);
        assert_eq!(d.base_version, DictVersion::V4_2);
        assert_eq!(d.resolution, DictResolution::StructuralBase);
        assert!(d.fall_through, "additive dict must overlay, not replace");
        assert!((d.base_score - 1.0).abs() < 1e-6, "score {}", d.base_score);

        let delta = d.build_delta().expect("delta");
        assert!(delta.group("TEST").is_some(), "new group present");
        assert_eq!(delta.added_headings("TEST").len(), 3, "all 3 names new");
        assert!(delta.heading(&heading_key("TEST", "TEST_VAL")).is_some());
    }

    #[test]
    fn ags_and_json_formats_converge() {
        // Same content, two formats → identical base, delta hash. One dictionary,
        // two spellings.
        let j = parse(ADDITIVE_JSON, DictFormat::Auto, BaseSpec::Auto);
        let a = parse(ADDITIVE_AGS, DictFormat::Auto, BaseSpec::Auto);
        assert_eq!(a.base_version, DictVersion::V4_2);
        assert_eq!(a.resolution, DictResolution::StructuralBase);
        assert_eq!(
            j.hash, a.hash,
            "identical dicts must fingerprint identically"
        );
    }

    #[test]
    fn purely_novel_dict_defaults_to_latest() {
        // No standard-named heading at all → signal 0 → latest edition base,
        // clean overlay (never replacement).
        let novel = br#"{"groups":{"ZZQQ":{"parent":"","headings":[
            {"name":"ZZQQ_FOO","type":"X","status":"KEY"}
        ]}}}"#;
        let d = parse(novel, DictFormat::Json, BaseSpec::Auto);
        assert_eq!(d.base_version, DictVersion::V4_2);
        assert!(d.fall_through);
    }

    #[test]
    fn overriding_a_standard_type_is_detectable_and_scores_below_one() {
        // R1: signal is name-match, score is (type,status) agreement — distinct,
        // so overriding SAMP_TOP's type from 2DP→X yields a reachable mid score
        // (SAMP_ID agrees, SAMP_TOP doesn't → 1/2). The override lands in the delta.
        let override_json = br#"{"groups":{"SAMP":{"headings":[
            {"name":"SAMP_ID","type":"ID","status":"KEY"},
            {"name":"SAMP_TOP","type":"X","status":"KEY"}
        ]}}}"#;
        let d = parse(override_json, DictFormat::Json, BaseSpec::Auto);
        assert!((d.base_score - 0.5).abs() < 1e-6, "score {}", d.base_score);

        let delta = d.build_delta().expect("delta");
        let ov = delta
            .heading(&heading_key("SAMP", "SAMP_TOP"))
            .expect("override present");
        assert_eq!(ov.ags_type, "X");
        // SAMP_ID matches the base exactly → not carried in the sparse delta.
        assert!(delta.heading(&heading_key("SAMP", "SAMP_ID")).is_none());
        // Touching SAMP to override a heading must NOT blank its description.
        assert!(
            delta.group("SAMP").is_none(),
            "no gratuitous group override"
        );
    }

    #[test]
    fn replace_mode_excludes_the_base() {
        // A full replacement must be self-contained (every parent defined in the
        // dict), so a single root group with no parent.
        let replacement = br#"{"groups":{"PROJ":{"parent":"","headings":[
            {"name":"PROJ_ID","type":"ID","status":"KEY"},
            {"name":"PROJ_NAME","type":"X","status":"REQUIRED"}
        ]}}}"#;
        let d = parse(replacement, DictFormat::Json, BaseSpec::Replace);
        assert_eq!(d.resolution, DictResolution::Replacement);
        assert!(
            !d.fall_through,
            "replacement never falls through to the base"
        );
        let delta = d.build_delta().expect("delta");
        // Under replacement every listed heading is in the delta (base excluded).
        assert_eq!(delta.added_headings("PROJ").len(), 2);
        assert!(delta.heading(&heading_key("PROJ", "PROJ_ID")).is_some());
    }

    #[test]
    fn replacement_with_a_dangling_parent_is_rejected() {
        // A replacement is not allowed to lean on the base for a parent it omits.
        let err = parse_dict(
            ADDITIVE_JSON,
            DictFormat::Json,
            UTF_8,
            BaseSpec::Replace,
            "x",
        )
        .unwrap_err();
        assert!(
            matches!(err, DictError::UnknownParent { ref parent, .. } if parent == "SAMP"),
            "{err:?}"
        );
    }

    #[test]
    fn force_pins_the_base_edition() {
        let d = parse(
            ADDITIVE_JSON,
            DictFormat::Json,
            BaseSpec::Force(DictVersion::V4_0_3),
        );
        assert_eq!(d.base_version, DictVersion::V4_0_3);
        assert_eq!(d.resolution, DictResolution::Forced);
    }

    #[test]
    fn malformed_json_reports_a_locator() {
        let err =
            parse_dict(b"{ not json", DictFormat::Json, UTF_8, BaseSpec::Auto, "x").unwrap_err();
        assert!(matches!(err, DictError::MalformedJson { .. }), "{err:?}");
    }

    #[test]
    fn unknown_type_is_rejected() {
        let bad = br#"{"groups":{"TEST":{"parent":"SAMP","headings":[
            {"name":"TEST_X","type":"BANANA","status":"KEY"}
        ]}}}"#;
        let err = parse_dict(bad, DictFormat::Json, UTF_8, BaseSpec::Auto, "x").unwrap_err();
        assert!(matches!(err, DictError::UnknownType { .. }), "{err:?}");
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let bad = br#"{"groups":{"TEST":{"parent":"ZZZZ","headings":[
            {"name":"TEST_X","type":"X","status":"KEY"}
        ]}}}"#;
        let err = parse_dict(bad, DictFormat::Json, UTF_8, BaseSpec::Auto, "x").unwrap_err();
        assert!(
            matches!(err, DictError::UnknownParent { ref parent, .. } if parent == "ZZZZ"),
            "{err:?}"
        );
    }

    #[test]
    fn duplicate_heading_is_rejected() {
        let bad = br#"{"groups":{"TEST":{"parent":"SAMP","headings":[
            {"name":"DUP","type":"X","status":"KEY"},
            {"name":"DUP","type":"X","status":"OTHER"}
        ]}}}"#;
        let err = parse_dict(bad, DictFormat::Json, UTF_8, BaseSpec::Auto, "x").unwrap_err();
        assert!(matches!(err, DictError::DuplicateHeading { .. }), "{err:?}");
    }

    #[test]
    fn hash_is_deterministic_and_content_sensitive() {
        let a = parse(ADDITIVE_JSON, DictFormat::Json, BaseSpec::Auto);
        let b = parse(ADDITIVE_JSON, DictFormat::Json, BaseSpec::Auto);
        assert_eq!(a.hash, b.hash, "same input → same fingerprint");
        // A type change must move the hash.
        let changed = br#"{"groups":{"TEST":{"parent":"SAMP","headings":[
            {"name":"SAMP_ID","type":"ID","status":"KEY"},
            {"name":"TEST_VAL","type":"3DP","status":"REQUIRED"}
        ]}}}"#;
        let c = parse(changed, DictFormat::Json, BaseSpec::Auto);
        assert_ne!(a.hash, c.hash);
    }

    #[test]
    fn dict_error_messages_name_the_problem() {
        // A blanked Display would emit empty diagnostics; each variant must name
        // its own cause (the surface prints these before any data rule runs).
        assert!(format!("{}", DictError::Empty).contains("no groups"));
        let us = DictError::UnknownStatus {
            line: 7,
            token: "ZZ".into(),
        };
        let msg = format!("{us}");
        assert!(msg.contains("ZZ") && msg.contains("KEY"), "{msg}");
        assert!(
            format!(
                "{}",
                DictError::DuplicateHeading {
                    group: "SAMP".into(),
                    heading: "SAMP_ID".into()
                }
            )
            .contains("twice")
        );
    }

    #[test]
    fn valid_status_accepts_the_vocab_and_empty_only() {
        assert!(valid_status("KEY"));
        assert!(valid_status("required")); // case-insensitive
        assert!(valid_status(""), "empty is treated as OTHER downstream");
        assert!(!valid_status("BOGUS"), "an unknown status must be rejected");
    }

    #[test]
    fn desc_of_returns_the_description_or_empty() {
        use crate::union::DictGroup;
        let with = DictGroup {
            parent: None,
            description: Some("A group".into()),
            headings: Vec::new(),
        };
        let without = DictGroup {
            parent: None,
            description: None,
            headings: Vec::new(),
        };
        assert_eq!(desc_of(&with), "A group");
        assert_eq!(desc_of(&without), "");
    }

    #[test]
    fn build_delta_records_a_parent_override() {
        // Re-parenting SAMP (base parent LOCA → PROJ) must land in the delta as a
        // group override — pins that the override fires on a DIFFERING parent, and
        // that a parent OR description change is enough (not both).
        let j = br#"{"groups":{"SAMP":{"parent":"PROJ","headings":[
            {"name":"SAMP_ID","type":"ID","status":"KEY"}
        ]}}}"#;
        let delta = parse(j, DictFormat::Json, BaseSpec::Auto)
            .build_delta()
            .expect("delta");
        assert_eq!(
            delta.group("SAMP").map(|g| g.parent),
            Some("PROJ"),
            "parent override not recorded"
        );
    }

    #[test]
    fn build_delta_records_a_description_override() {
        // Restating SAMP's parent (unchanged) but a new description overrides only
        // on the description — pins the description-difference predicate.
        let j = br#"{"groups":{"SAMP":{"parent":"LOCA","description":"Bespoke samples","headings":[
            {"name":"SAMP_ID","type":"ID","status":"KEY"}
        ]}}}"#;
        let delta = parse(j, DictFormat::Json, BaseSpec::Auto)
            .build_delta()
            .expect("delta");
        assert_eq!(
            delta.group("SAMP").map(|g| g.desc),
            Some("Bespoke samples"),
            "description override not recorded"
        );
    }

    #[test]
    fn build_delta_unit_wins_when_present_and_inherits_when_empty() {
        // Override a base heading in overlay mode, mirroring the base type/status so
        // ONLY the unit differs. A non-empty custom unit wins; overriding a base
        // heading does not re-list it as a new name.
        let base = Dictionary::bundled(DictVersion::V4_2);
        let bh = base.heading("LOCA", "LOCA_NATE").expect("LOCA_NATE");
        assert!(!bh.unit.is_empty(), "test needs a heading with a base unit");

        let unit_json = format!(
            r#"{{"groups":{{"LOCA":{{"parent":"PROJ","headings":[
                {{"name":"LOCA_NATE","type":"{t}","status":"{s}","unit":"ZZ"}}
            ]}}}}}}"#,
            t = bh.ags_type,
            s = bh.status,
        );
        let delta = parse(unit_json.as_bytes(), DictFormat::Json, BaseSpec::Auto)
            .build_delta()
            .expect("delta");
        let e = delta
            .heading(&heading_key("LOCA", "LOCA_NATE"))
            .expect("a unit-only difference must land in the delta");
        assert_eq!(e.unit, "ZZ", "a non-empty custom unit must win");
        assert!(
            delta.added_headings("LOCA").is_empty(),
            "overriding a base heading must not re-list it as new: {:?}",
            delta.added_headings("LOCA")
        );

        // A present-but-EMPTY unit must inherit the base unit (not blank it). Force
        // the heading into the delta via a type change so we can read its unit.
        let inherit_json = format!(
            r#"{{"groups":{{"LOCA":{{"parent":"PROJ","headings":[
                {{"name":"LOCA_NATE","type":"X","status":"{s}","unit":""}}
            ]}}}}}}"#,
            s = bh.status,
        );
        let delta2 = parse(inherit_json.as_bytes(), DictFormat::Json, BaseSpec::Auto)
            .build_delta()
            .expect("delta");
        let e2 = delta2
            .heading(&heading_key("LOCA", "LOCA_NATE"))
            .expect("a type override must land in the delta");
        assert_eq!(
            e2.unit, bh.unit,
            "an empty custom unit must inherit the base unit"
        );
    }
}
