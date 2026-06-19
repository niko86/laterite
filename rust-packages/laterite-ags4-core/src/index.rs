//! Byte-offset index over an AGS4 file: *where* each group's section lives, so a
//! single group can be parsed from its own slice instead of re-parsing the whole
//! file.
//!
//! Raw `.ags` has no footer/index, so locating a group means a scan — but a
//! *cheap* one: this records only the byte range of each group's section, not its
//! typed rows. A host that wants just `LOCA` from a 50 MB file then parses the
//! `LOCA` slice (O(group)) rather than the whole file (O(size)); a host
//! materialising every group parses each slice once in a single pass over the
//! bytes (O(size) total, not O(groups × size)). Persisted as a sidecar it also
//! lets a remote reader range-GET one group's bytes.
//!
//! **Slice re-parse agrees with the whole-file parse by construction.** The scan
//! drives the *same* `csv::Reader` configuration as
//! [`crate::ags4_codec::parse_reader`] and reads each record's byte offset via
//! `csv::StringRecord::position()`, so a section's start is exactly the byte
//! offset of its `"GROUP",…` record and re-parsing the slice reproduces the
//! whole-file group exactly — the `slice_parity` test guards the two CSV configs
//! against drift.
//!
//! This is a *locator*, not a validator: it inspects only `"GROUP"` records, so it
//! does not reproduce the parser's structural checks (it won't reject, e.g., a
//! HEADING/DATA row before the first GROUP, which `parse_reader` errors on), and
//! for a (rare, non-conforming) file that splits one group across two sections the
//! first section wins and the later rows are not in the slice. So the lazy
//! single-group path is for **well-formed files the parser already accepts**,
//! while the eager whole-file parse stays the validating, always-correct default.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ags4_codec::{AgsGroup, read_ags4_bytes};
use crate::error::CliError;

/// Half-open byte range `[start, end)` of a group's section in the source bytes.
pub type Range = (u64, u64);

/// Where each group's section lives in an AGS4 file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupIndex {
    /// Group code → its section's byte range.
    pub groups: HashMap<String, Range>,
    /// Section order as they appear in the file (matches `ParsedAgs4::order`).
    pub order: Vec<String>,
}

impl GroupIndex {
    /// The byte range of `code`'s section, if present.
    pub fn range(&self, code: &str) -> Option<Range> {
        self.groups.get(code).copied()
    }
}

/// Scan `bytes` for group-section boundaries. One pass with the parser's CSV
/// config; each `"GROUP",…` record's byte offset starts a section, which runs to
/// the next `"GROUP"` (or EOF).
pub fn index_ags4_bytes(bytes: &[u8]) -> Result<GroupIndex, CliError> {
    // MUST match `ags4_codec::parse_reader`'s builder so record boundaries — and
    // thus each GROUP record's byte offset — line up with the parser. The
    // `slice_parity` test fails loudly if this drifts.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::None)
        .from_reader(bytes);

    // (code, start byte) for each GROUP marker, in file order.
    let mut starts: Vec<(String, u64)> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| CliError::Schema(format!("AGS4 CSV: {e}")))?;
        if record.get(0).unwrap_or("").trim() == "GROUP" {
            let code = record
                .get(1)
                .ok_or_else(|| CliError::Schema("GROUP row missing group code".into()))?
                .trim()
                .to_string();
            // `position()` is the byte offset of the record's start — populated by
            // default; only `None` if position tracking were disabled.
            let start = record
                .position()
                .map(|p| p.byte())
                .ok_or_else(|| CliError::Schema("csv position unavailable for GROUP row".into()))?;
            starts.push((code, start));
        }
    }

    let total = bytes.len() as u64;
    let mut groups: HashMap<String, Range> = HashMap::with_capacity(starts.len());
    let mut order: Vec<String> = Vec::with_capacity(starts.len());
    for (i, (code, start)) in starts.iter().enumerate() {
        // A section ends where the next one begins, or at EOF for the last.
        let end = starts.get(i + 1).map(|(_, s)| *s).unwrap_or(total);
        // First section wins for a (non-conforming) repeated group — see the
        // module note.
        if !groups.contains_key(code) {
            order.push(code.clone());
            groups.insert(code.clone(), (*start, end));
        }
    }
    Ok(GroupIndex { groups, order })
}

/// Parse a single group from its indexed byte range, reusing the whole-file
/// parser on just that slice. The slice begins at a `"GROUP",…` record, so the
/// parser sees a self-contained one-group file.
pub fn parse_group_slice(bytes: &[u8], range: Range, code: &str) -> Result<AgsGroup, CliError> {
    let (start, end) = range;
    let slice = bytes.get(start as usize..end as usize).ok_or_else(|| {
        CliError::Schema(format!(
            "index range {start}..{end} out of bounds for {} bytes",
            bytes.len()
        ))
    })?;
    let mut parsed = read_ags4_bytes(slice)?;
    parsed
        .groups
        .remove(code)
        .ok_or_else(|| CliError::Schema(format!("group '{code}' not found in its indexed slice")))
}

/// Format version of the `.ags.idx` sidecar.
pub const SIDECAR_VERSION: u32 = 1;

/// The source file a [`Sidecar`] certifies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    /// Source byte length.
    pub size: u64,
    /// Hex-encoded SHA-256 of the source bytes — the strong staleness check.
    pub sha256: String,
    /// AGS edition resolved from the file's `TRAN_AGS` at validation time
    /// (e.g. "4.1"); empty if the minting layer didn't resolve one.
    pub edition: String,
}

/// The validation a [`Sidecar`] registers. A sidecar is only minted for a file
/// that validated **clean** (zero error-severity findings), so this records *who*
/// validated it, *when*, and any non-blocking advisories that were present — it is
/// a provenance record, not a re-derivable computation. Core cannot mint it
/// (validation lives in the validator crate, above core).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStamp {
    /// What validated it (e.g. "lat-check", "laterite_ags4").
    pub validator: String,
    /// The validator's version string.
    pub validator_version: String,
    /// When, as an ISO-8601 / RFC-3339 UTC string, set by the minting layer.
    pub checked_at: String,
    /// Non-blocking advisory counts present at validation. Errors are 0 by
    /// construction — a sidecar is only minted for a clean file.
    pub warnings: u32,
    pub fyi: u32,
}

/// A persisted `.ags.idx`: a validity **certificate** + byte-offset index for one
/// AGS4 file. It exists only for a file that validated clean, carries the
/// provenance of that validation, and locates each group's bytes — so an index
/// that exists is a positive assertion ("this exact file was validated clean,
/// here is the proof, here is where the bytes are"), never a reference into a
/// possibly-corrupt file.
///
/// Core owns the *format* (this struct, its JSON (de)serialise, and the
/// [`Sidecar::is_fresh_for`] staleness check) and can *read* a sidecar with no
/// validator. **Minting** one — validate, and only if clean assemble + stamp +
/// write — is an **opt-in** action of a validator-aware layer (`lat-check
/// --emit-index`, the `laterite_ags4` extension's `ags_index`), never automatic;
/// core does not depend on the validator so it cannot mint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sidecar {
    /// Sidecar format version.
    pub version: u32,
    /// The source file this certifies.
    pub file: FileMeta,
    /// The validation this registers.
    pub validation: ValidationStamp,
    /// Group code → byte range of its section in the source.
    pub groups: HashMap<String, Range>,
    /// Section order as in the source file.
    pub order: Vec<String>,
}

impl Sidecar {
    /// Assemble a sidecar for an already-validated file. The CALLER must have
    /// confirmed a clean validation (zero error findings) and supply the
    /// `edition` + [`ValidationStamp`]; the byte index is computed here from
    /// `bytes`. Core cannot validate, so it cannot enforce the precondition — it
    /// trusts the caller, which is the (opt-in) validator-aware minting layer.
    pub fn assemble(
        bytes: &[u8],
        edition: String,
        validation: ValidationStamp,
    ) -> Result<Sidecar, CliError> {
        let index = index_ags4_bytes(bytes)?;
        Ok(Sidecar {
            version: SIDECAR_VERSION,
            file: FileMeta {
                size: bytes.len() as u64,
                sha256: sha256_hex(bytes),
                edition,
            },
            validation,
            groups: index.groups,
            order: index.order,
        })
    }

    /// Serialise to pretty JSON — the on-disk `.ags.idx` form.
    pub fn to_json(&self) -> Result<Vec<u8>, CliError> {
        serde_json::to_vec_pretty(self)
            .map_err(|e| CliError::Schema(format!("sidecar serialize: {e}")))
    }

    /// Parse a sidecar from its JSON bytes, rejecting an unknown format version.
    pub fn from_json(bytes: &[u8]) -> Result<Sidecar, CliError> {
        let s: Sidecar = serde_json::from_slice(bytes)
            .map_err(|e| CliError::Schema(format!("sidecar parse: {e}")))?;
        if s.version != SIDECAR_VERSION {
            return Err(CliError::Schema(format!(
                "sidecar version {} unsupported (expected {SIDECAR_VERSION}) — rebuild it",
                s.version
            )));
        }
        Ok(s)
    }

    /// Is this sidecar still current for `bytes`? Strong check: version + size +
    /// SHA-256. A mismatch means the source changed under the sidecar — its byte
    /// offsets are now lies, so rebuild rather than trust them (`.ags.idx` is a
    /// pure cache: stale ⇒ ignore + regenerate).
    pub fn is_fresh_for(&self, bytes: &[u8]) -> bool {
        self.version == SIDECAR_VERSION
            && self.file.size == bytes.len() as u64
            && self.file.sha256 == sha256_hex(bytes)
    }

    /// Cheap size-only freshness pre-check — for a remote source where re-hashing
    /// would mean re-downloading. Necessary but not sufficient; pair it with an
    /// ETag/Last-Modified check at the call site.
    pub fn size_matches(&self, size: u64) -> bool {
        self.file.size == size
    }

    /// The byte-offset index view (for locating / slicing groups).
    pub fn index(&self) -> GroupIndex {
        GroupIndex {
            groups: self.groups.clone(),
            order: self.order.clone(),
        }
    }
}

/// Hex-encode the SHA-256 of `bytes` — the sidecar's staleness fingerprint.
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // A normal two-group file (LF line endings, a blank-line separator).
    const TWO: &str = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","ID","X"
"DATA","P1","Demo project"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","100.50"
"DATA","BH02","TP","98.00"
"#;

    /// Assert the index is internally consistent and slice-parses identically to
    /// the whole-file parse — the core property, reused across fixtures.
    fn assert_consistent(content: &str) {
        let bytes = content.as_bytes();
        let whole = read_ags4_bytes(bytes).unwrap();
        let idx = index_ags4_bytes(bytes).unwrap();

        // Order matches the parser.
        assert_eq!(idx.order, whole.order, "index order must match parse order");

        // Ranges tile [0, len): start at byte 0, end at EOF, contiguous.
        let ranges: Vec<Range> = idx.order.iter().map(|c| idx.range(c).unwrap()).collect();
        assert_eq!(
            ranges.first().unwrap().0,
            0,
            "first section starts at byte 0"
        );
        assert_eq!(
            ranges.last().unwrap().1,
            bytes.len() as u64,
            "last section ends at EOF"
        );
        for w in ranges.windows(2) {
            assert_eq!(
                w[0].1, w[1].0,
                "sections must be contiguous (no gap/overlap)"
            );
        }

        // Slice parity: each group from its slice == the whole-file group.
        for code in &whole.order {
            let g = whole.get(code).unwrap();
            let s = parse_group_slice(bytes, idx.range(code).unwrap(), code).unwrap();
            assert_eq!(s.headings, g.headings, "{code} headings");
            assert_eq!(s.units, g.units, "{code} units");
            assert_eq!(s.types, g.types, "{code} types");
            assert_eq!(s.rows, g.rows, "{code} rows");
        }
    }

    #[test]
    fn two_groups_lf() {
        assert_consistent(TWO);
    }

    #[test]
    fn crlf_line_endings() {
        assert_consistent(&TWO.replace('\n', "\r\n"));
    }

    #[test]
    fn quoted_comma_and_escaped_quote_survive_slicing() {
        // A DATA value with an embedded comma and an escaped quote ("") — the
        // byte offsets + slice re-parse must preserve them (proves we honour the
        // csv quoting, not a naive split).
        let f = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","ID","X"
"DATA","P1","Acme, Inc. ""HQ"""

"GROUP","LOCA"
"HEADING","LOCA_ID"
"UNIT",""
"TYPE","ID"
"DATA","BH01"
"#;
        assert_consistent(f);
        // and spot-check the value really did round-trip through the slice
        let bytes = f.as_bytes();
        let idx = index_ags4_bytes(bytes).unwrap();
        let proj = parse_group_slice(bytes, idx.range("PROJ").unwrap(), "PROJ").unwrap();
        assert_eq!(proj.rows[0]["PROJ_NAME"], r#"Acme, Inc. "HQ""#);
    }

    #[test]
    fn group_with_zero_data_rows() {
        // PROJ has no DATA row — still indexed, still slice-parses (empty rows).
        let f = r#""GROUP","PROJ"
"HEADING","PROJ_ID"
"UNIT",""
"TYPE","ID"

"GROUP","LOCA"
"HEADING","LOCA_ID"
"UNIT",""
"TYPE","ID"
"DATA","BH01"
"#;
        assert_consistent(f);
        let bytes = f.as_bytes();
        let idx = index_ags4_bytes(bytes).unwrap();
        let proj = parse_group_slice(bytes, idx.range("PROJ").unwrap(), "PROJ").unwrap();
        assert!(proj.rows.is_empty());
    }

    /// Deterministic stand-in for a property test: many synthetic files varying
    /// group count and per-group row count, all must stay consistent. Covers the
    /// "synthetic multi-group" property without pulling in a proptest dep.
    #[test]
    fn many_synthetic_files_are_consistent() {
        for n_groups in [1usize, 2, 5, 12] {
            for row_step in [0usize, 1, 3] {
                let mut s = String::new();
                for g in 0..n_groups {
                    // arbitrary 4-char codes G000..; the index is registry-free.
                    s.push_str(&format!("\"GROUP\",\"G{g:03}\"\n"));
                    s.push_str("\"HEADING\",\"A_ID\",\"A_VAL\"\n");
                    s.push_str("\"UNIT\",\"\",\"\"\n");
                    s.push_str("\"TYPE\",\"ID\",\"X\"\n");
                    // vary row counts per group so ranges differ in size
                    for r in 0..(g * row_step) {
                        s.push_str(&format!("\"DATA\",\"K{g}_{r}\",\"v{r}\"\n"));
                    }
                    s.push('\n'); // blank separator
                }
                assert_consistent(&s);
            }
        }
    }

    fn stamp() -> ValidationStamp {
        ValidationStamp {
            validator: "test".into(),
            validator_version: "0.0.0".into(),
            checked_at: "2026-06-19T00:00:00Z".into(),
            warnings: 0,
            fyi: 1,
        }
    }

    #[test]
    fn sidecar_json_round_trips() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        let back = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(sc, back, "sidecar survives a JSON round-trip");
        // the embedded index matches a direct scan of the same bytes
        assert_eq!(back.index(), index_ags4_bytes(bytes).unwrap());
        assert_eq!(back.file.sha256.len(), 64, "sha256 is 64 hex chars");
    }

    #[test]
    fn sidecar_freshness_tracks_the_source() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        assert!(
            sc.is_fresh_for(bytes),
            "fresh for the bytes it was built from"
        );
        assert!(sc.size_matches(bytes.len() as u64));
        // any change to the source busts it (the sha differs)
        let mut changed = bytes.to_vec();
        changed.push(b'\n');
        assert!(!sc.is_fresh_for(&changed), "a changed file is not fresh");
    }

    #[test]
    fn sidecar_rejects_unknown_version() {
        let bytes = TWO.as_bytes();
        let mut sc = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        sc.version = SIDECAR_VERSION + 1;
        assert!(
            Sidecar::from_json(&sc.to_json().unwrap()).is_err(),
            "an unknown sidecar version is rejected, not silently trusted"
        );
    }
}
