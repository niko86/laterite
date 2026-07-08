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
//! **Slice re-parse agrees with the whole-file parse by construction.** Section
//! starts come from the shared parse leaf's source-true byte walk
//! ([`laterite_ags4_parse`]): each section begins at the exact byte offset of its
//! `"GROUP",…` record, so re-parsing the slice reproduces the whole-file group
//! exactly. The in-module consistency tests guard that those offsets slice
//! correctly for [`crate::ags4_codec`]'s reparse. (Before #168 Phase 4 the scan
//! used the csv reader's record positions, which were off-by-one for CRLF and
//! absorbed leading blank lines — see O-40.)
//!
//! This is a *locator*, not a validator: it inspects only `"GROUP"` records, so it
//! does not reproduce the parser's structural checks (it won't reject, e.g., a
//! HEADING/DATA row before the first GROUP, which `parse_reader` errors on), and
//! for a (rare, non-conforming) file that splits one group across two sections the
//! first section wins and the later rows are not in the slice. So the lazy
//! single-group path is for **well-formed files the parser already accepts**,
//! while the eager whole-file parse stays the validating, always-correct default.

use std::collections::HashMap;

use laterite_ags4_parse::{ParseOptions, parse_bytes_opts};
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

/// Build the group-section byte index from the shared parse leaf's source-true
/// byte walk (#168 Phase 4). Each `"GROUP",…` record's `group_byte` starts a
/// section, which runs to the next group's start (or EOF). The leaf's offsets are
/// the real line-starts — the csv reader this replaced recorded the preceding
/// `\n` for CRLF groups and absorbed leading blank lines (see O-40).
pub fn index_ags4_bytes(bytes: &[u8]) -> Result<GroupIndex, CliError> {
    // Lean profile: no raw-line retention, and reject invalid UTF-8 loudly —
    // mirroring the csv reader this replaced (which also failed on non-UTF-8).
    // `lean()` never substitutes bytes, so the offsets index the original buffer.
    let parsed =
        parse_bytes_opts(bytes, ParseOptions::lean()).map_err(crate::ags4_codec::map_parse_err)?;
    // Defence in depth: a cert whose offsets don't index the original bytes is a
    // lie. `lean()` rejects rather than substitutes, so this never fires today —
    // but it must stay true if the profile ever changes.
    if !parsed.byte_offsets_source_true {
        return Err(CliError::Schema(
            "byte offsets are not source-true (encoding substitution shifted a record start)"
                .into(),
        ));
    }

    let total = parsed.total_bytes;
    let mut groups: HashMap<String, Range> = HashMap::with_capacity(parsed.group_order.len());
    let mut order: Vec<String> = Vec::with_capacity(parsed.group_order.len());
    // `group_order` is already de-duplicated (the leaf keeps the first-seen GROUP
    // for a repeated code), so each code resolves in `groups` exactly once.
    for (i, code) in parsed.group_order.iter().enumerate() {
        // A GROUP record with no code can't be located or sliced — reject it,
        // matching the retired csv index (and `ags4_codec::parse_reader`, which
        // still errors). The leaf yields an empty code for a bare `"GROUP"` row.
        if code.is_empty() {
            return Err(CliError::Schema("GROUP row missing group code".into()));
        }
        let start = parsed.groups[code].group_byte;
        // A section ends where the next one begins, or at EOF for the last.
        let end = parsed
            .group_order
            .get(i + 1)
            .map(|next| parsed.groups[next].group_byte)
            .unwrap_or(total);
        order.push(code.clone());
        groups.insert(code.clone(), (start, end));
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

/// The shared **engine identity** every surface stamps into a certificate's
/// [`ValidationStamp::validator`]. All surfaces run the same
/// `laterite_ags4_validator` rule engine, so a clean verdict is portable: a cert
/// minted by one door (Python, Node, the CLI, wasm, the DuckDB extension) is
/// trusted by another. The *string* is trust-inert provenance — real trust gates
/// on `validator_version` + `compat` + the check profile ([`Sidecar::checker_matches`]
/// / [`Sidecar::profile_covers`]) — so unifying it removes an accidental per-binding
/// silo without weakening any real trust boundary.
pub const ENGINE_IDENTITY: &str = "laterite_ags4";

/// The source file a [`Sidecar`] certifies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    /// Source byte length.
    pub size: u64,
    /// Hex-encoded SHA-256 of the source bytes — the strong, portable,
    /// origin-independent staleness fingerprint, and the ground truth any cheaper
    /// check falls back to. Always present; never superseded by the optional
    /// transport validators below (a local file has no ETag, and an ETag is only
    /// meaningful relative to the endpoint that issued it).
    pub sha256: String,
    /// AGS edition resolved from the file's `TRAN_AGS` at validation time
    /// (e.g. "4.1"); empty if the minting layer didn't resolve one.
    pub edition: String,
    /// The remote origin's HTTP `ETag` observed at mint time, verbatim (`W/`
    /// weak prefix preserved), when minted from a remote (http/s3) source — else
    /// `None`. A *cheap* freshness shortcut: a HEAD whose ETag matches proves the
    /// object is byte-identical, so a remote reader can trust the SHA + byte
    /// offsets WITHOUT re-downloading to re-hash. Only ever grants trust on a
    /// match; absence/mismatch downgrades to the SHA path (see
    /// [`Sidecar::is_fresh_for_remote`]).
    #[serde(default)]
    pub etag: Option<String>,
    /// The remote origin's HTTP `Last-Modified` at mint time, when known — the
    /// weak fallback (paired with `size`) for stores that return no usable ETag.
    /// Weaker than the ETag (second granularity), so it gates the cheap ranged
    /// read but never the strong verdict on its own.
    #[serde(default)]
    pub last_modified: Option<String>,
}

/// The validation a [`Sidecar`] registers. A sidecar is only minted for a file
/// that validated **clean** (zero error-severity findings), so this records *who*
/// validated it, *when*, and any non-blocking advisories that were present — it is
/// a provenance record, not a re-derivable computation. Core cannot mint it
/// (validation lives in the validator crate, above core).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStamp {
    /// What validated it — the shared [`ENGINE_IDENTITY`] engine string, so a
    /// clean cert minted by any surface is trusted by the others (trust also
    /// gates on `validator_version` + `compat` + profile).
    pub validator: String,
    /// The validation **engine** version (`laterite_ags4_validator::VERSION`),
    /// NOT the minting binding's crate version — so a cert is comparable across
    /// surfaces (the Python wheel and the independently-versioned DuckDB extension
    /// both stamp the engine version). A consumer trusts the clean verdict (skips
    /// re-validation) only when this still matches its own engine.
    pub validator_version: String,
    /// The python-ags4 compatibility version when validated through the
    /// `laterite.compat` drop-in (whose behaviour mimics that python-ags4
    /// release), else `None` for the native validator. Part of the checker
    /// identity: a compat-minted clean verdict isn't trusted by the native
    /// validator (and vice versa), since the two can disagree on a file.
    #[serde(default)]
    pub compat: Option<String>,
    /// Whether the validation ran Rule 20's **on-disk** half (the sibling `FILE/`
    /// tree must exist) — `lat validate --check-files`. Part of the check PROFILE: a
    /// missing on-disk file is an error, so a cert minted *without* this must not
    /// be trusted to skip a request that *wants* it ([`Sidecar::profile_covers`]).
    #[serde(default)]
    pub check_files: bool,
    /// Whether the edition was **forced** (`--dict-version X`) rather than
    /// auto-resolved from `TRAN_AGS`. A forced cert and an auto cert can record the
    /// same `edition` string yet have run different dictionaries when the file's
    /// `TRAN_AGS` disagrees — so the skip only trusts a same-forcing request.
    #[serde(default)]
    pub edition_forced: bool,
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
/// write — is an **opt-in** action of a validator-aware layer (`lat
/// certify`, the `laterite_ags4` extension's `ags_index`), never automatic;
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

/// Verdict of a cheap, I/O-free remote freshness check ([`Sidecar::is_fresh_for_remote`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreshness {
    /// A strong validator (ETag) matched — the object is byte-identical; trust the
    /// SHA + byte offsets without re-downloading.
    Trusted,
    /// Only weak signals (size + Last-Modified) matched — probably fresh; fine to
    /// gate the cheap ranged read, but a verdict-strict caller may still re-hash.
    ProbablyFresh,
    /// No usable cheap match — download + re-hash ([`Sidecar::is_fresh_for`]) or
    /// regenerate the cert.
    MustRehash,
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
                // Local mint by default; a remote-aware minting layer records the
                // origin's HTTP validators via `with_origin`.
                etag: None,
                last_modified: None,
            },
            validation,
            groups: index.groups,
            order: index.order,
        })
    }

    /// Record the remote origin's HTTP validators (`ETag` / `Last-Modified`)
    /// observed at mint time, so a remote consumer can confirm freshness with a
    /// HEAD instead of re-downloading to re-hash. Builder; a local mint leaves both
    /// `None` (and the SHA stays the authoritative check regardless).
    pub fn with_origin(mut self, etag: Option<String>, last_modified: Option<String>) -> Self {
        self.file.etag = etag;
        self.file.last_modified = last_modified;
        self
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

    /// Was this cert minted by the given checker identity? `is_fresh_for` proves
    /// the *bytes* are unchanged; this proves the *checker* is the same — both
    /// must hold before a consumer trusts the clean verdict and **skips**
    /// re-validation. A cert from a different/older engine (or a different compat
    /// profile) is byte-fresh but checker-stale: re-validate rather than trust a
    /// verdict today's rules might not reproduce.
    pub fn checker_matches(
        &self,
        validator: &str,
        validator_version: &str,
        compat: Option<&str>,
    ) -> bool {
        self.validation.validator == validator
            && self.validation.validator_version == validator_version
            && self.validation.compat.as_deref() == compat
    }

    /// Does this cert's check **profile** cover a request's? A clean verdict is
    /// only trustworthy for a request the cert validated *at least as strictly*:
    ///
    /// - it ran the on-disk file check if the request wants it
    ///   (`cert.check_files >= want_check_files` — a stronger cert covers a weaker
    ///   request, never the reverse), and
    /// - its edition forcing matches: a *forced* request (`want_forced_edition =
    ///   Some(ed)`) is covered only by a cert forced to the same edition; an *auto*
    ///   request (`None`) only by an auto cert — because a forced and an
    ///   auto-resolved run can apply different dictionaries to the same bytes.
    ///
    /// Pair with [`Sidecar::checker_matches`] (engine identity) and freshness
    /// before skipping re-validation.
    pub fn profile_covers(
        &self,
        want_check_files: bool,
        want_forced_edition: Option<&str>,
    ) -> bool {
        let check_ok = self.validation.check_files >= want_check_files;
        let edition_ok = match want_forced_edition {
            Some(ed) => self.validation.edition_forced && self.file.edition == ed,
            None => !self.validation.edition_forced,
        };
        check_ok && edition_ok
    }

    /// Cheap, **I/O-free** remote freshness check against a live HEAD's observed
    /// `(size, etag, last_modified)`. Core never does the network I/O — the caller
    /// (the DuckDB VFS / httpfs / a remote reader) performs the HEAD and passes the
    /// observed values in. The optional transport validators can only ever GRANT
    /// trust on a match; absence or mismatch downgrades toward the strong SHA path
    /// ([`Sidecar::is_fresh_for`]), never the reverse — so they can never make a
    /// stale cert look fresh.
    pub fn is_fresh_for_remote(
        &self,
        observed_size: u64,
        observed_etag: Option<&str>,
        observed_last_modified: Option<&str>,
    ) -> RemoteFreshness {
        // Wrong format version or a size change → the cheap path can't help.
        if self.version != SIDECAR_VERSION || self.file.size != observed_size {
            return RemoteFreshness::MustRehash;
        }
        // Strong: a stored ETag we can compare to the live one.
        if let (Some(cert), Some(live)) = (self.file.etag.as_deref(), observed_etag) {
            return if cert == live {
                RemoteFreshness::Trusted
            } else {
                RemoteFreshness::MustRehash // ETag changed ⇒ object changed
            };
        }
        // Weak: size already matched; require Last-Modified to agree too.
        if let (Some(cert), Some(live)) =
            (self.file.last_modified.as_deref(), observed_last_modified)
        {
            return if cert == live {
                RemoteFreshness::ProbablyFresh
            } else {
                RemoteFreshness::MustRehash
            };
        }
        // size matched but no usable transport validator → can't cheaply confirm.
        RemoteFreshness::MustRehash
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

        // For a file that BEGINS with a GROUP (no leading blanks/BOM), ranges tile
        // [0, len): first at byte 0, last at EOF, contiguous. Files with leading
        // content start the first section at the true offset instead (a gap before
        // it) — see `leading_blank_lines_shift_the_first_offset` and O-40.
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

    /// #168 Phase 4 (O-40): a file with leading blank lines locates its first
    /// section at the TRUE offset (after the blanks), NOT byte 0 — the ratified
    /// tightening. The leading bytes belong to no section, yet every group still
    /// slice-reparses identically. (The retired csv index absorbed the blanks,
    /// recording the first GROUP at 0.)
    #[test]
    fn leading_blank_lines_shift_the_first_offset() {
        let content = format!("\n\n{TWO}"); // two blank lines before the first GROUP
        let bytes = content.as_bytes();
        let whole = read_ags4_bytes(bytes).unwrap();
        let idx = index_ags4_bytes(bytes).unwrap();

        // The first section now starts at byte 2 (after the two `\n`), not 0 — so
        // bytes [0, 2) deliberately belong to no section.
        assert_eq!(
            idx.range(&idx.order[0]).unwrap().0,
            2,
            "first GROUP is located after the leading blank lines"
        );
        // Slice parity still holds for every group despite the leading gap.
        for code in &whole.order {
            let g = whole.get(code).unwrap();
            let s = parse_group_slice(bytes, idx.range(code).unwrap(), code).unwrap();
            assert_eq!(s.headings, g.headings, "{code} headings");
            assert_eq!(s.rows, g.rows, "{code} rows");
        }
    }

    /// A bare `"GROUP"` row with no code can't be located or sliced. The
    /// leaf yields an empty code for it; `index_ags4_bytes` rejects it, preserving
    /// the retired csv index's behaviour (and matching `ags4_codec`).
    #[test]
    fn group_row_without_a_code_is_rejected() {
        let err = index_ags4_bytes(b"\"GROUP\"\n\"HEADING\",\"X\"\n\"UNIT\",\"\"\n").unwrap_err();
        assert!(
            matches!(err, CliError::Schema(ref m) if m.contains("missing group code")),
            "expected a 'missing group code' schema error, got {err:?}"
        );
    }

    /// The shared engine identity is the load-bearing cross-surface contract:
    /// Python/Node/wasm (and the DuckDB extension) all stamp THIS exact string,
    /// so a clean cert minted by one door is trusted by another. Changing it
    /// silently re-silos the surfaces — pin it. (#430 PR 1a)
    #[test]
    fn engine_identity_is_the_shared_cross_surface_string() {
        assert_eq!(ENGINE_IDENTITY, "laterite_ags4");
    }

    fn stamp() -> ValidationStamp {
        ValidationStamp {
            validator: "test".into(),
            validator_version: "0.0.0".into(),
            compat: None,
            check_files: false,
            edition_forced: false,
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
    fn checker_matches_is_exact_on_identity() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        // stamp() is the native shape (validator "test", version "0.0.0", no compat)
        assert!(
            sc.checker_matches("test", "0.0.0", None),
            "same checker trusted"
        );
        assert!(
            !sc.checker_matches("test", "0.0.1", None),
            "a newer engine version is NOT trusted (re-validate)"
        );
        assert!(
            !sc.checker_matches("other", "0.0.0", None),
            "a different validator is NOT trusted"
        );
        assert!(
            !sc.checker_matches("test", "0.0.0", Some("python-ags4-0.5")),
            "a compat consumer does NOT trust a native-minted cert"
        );
    }

    #[test]
    fn compat_provenance_round_trips_and_defaults() {
        let bytes = TWO.as_bytes();
        let mut st = stamp();
        st.compat = Some("python-ags4-0.5.0".into());
        let sc = Sidecar::assemble(bytes, "4.1".into(), st).unwrap();
        let back = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(back.validation.compat.as_deref(), Some("python-ags4-0.5.0"));
        assert!(back.checker_matches("test", "0.0.0", Some("python-ags4-0.5.0")));
        // a legacy cert JSON without the field deserialises with compat = None
        let legacy = r#"{"version":1,"file":{"size":1,"sha256":"x","edition":"4.1"},
            "validation":{"validator":"v","validator_version":"1","checked_at":"t","warnings":0,"fyi":0},
            "groups":{},"order":[]}"#;
        assert_eq!(
            Sidecar::from_json(legacy.as_bytes())
                .unwrap()
                .validation
                .compat,
            None
        );
    }

    #[test]
    fn profile_covers_is_directional() {
        let bytes = TWO.as_bytes();
        // default cert: check_files=false, edition_forced=false, edition "4.1"
        let def = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        assert!(
            def.profile_covers(false, None),
            "default cert covers a default request"
        );
        assert!(
            !def.profile_covers(true, None),
            "a default cert does NOT cover a --check-files request"
        );
        assert!(
            !def.profile_covers(false, Some("4.1")),
            "an auto cert does NOT cover a forced-edition request (different dictionaries possible)"
        );

        // a stronger cert: ran the on-disk file check
        let mut s = stamp();
        s.check_files = true;
        let strong = Sidecar::assemble(bytes, "4.1".into(), s).unwrap();
        assert!(
            strong.profile_covers(true, None),
            "covers a --check-files request"
        );
        assert!(
            strong.profile_covers(false, None),
            "and still covers a weaker default request"
        );

        // a forced-edition cert covers only the SAME forced edition
        let mut f = stamp();
        f.edition_forced = true;
        let forced = Sidecar::assemble(bytes, "4.0.4".into(), f).unwrap();
        assert!(
            forced.profile_covers(false, Some("4.0.4")),
            "covers the same forced edition"
        );
        assert!(
            !forced.profile_covers(false, Some("4.1")),
            "not a different forced edition"
        );
        assert!(!forced.profile_covers(false, None), "not an auto request");
    }

    #[test]
    fn remote_freshness_grants_only_on_a_validator_match() {
        let bytes = TWO.as_bytes();
        let size = bytes.len() as u64;
        let sc = Sidecar::assemble(bytes, "4.1".into(), stamp())
            .unwrap()
            .with_origin(
                Some("\"abc123\"".into()),
                Some("Wed, 19 Jun 2026 00:00:00 GMT".into()),
            );
        // round-trips
        let sc = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(sc.file.etag.as_deref(), Some("\"abc123\""));

        // strong: a matching ETag is trusted (no re-hash needed)
        assert_eq!(
            sc.is_fresh_for_remote(size, Some("\"abc123\""), None),
            RemoteFreshness::Trusted
        );
        // a changed ETag means changed bytes — must re-hash, NOT fall to weak
        assert_eq!(
            sc.is_fresh_for_remote(
                size,
                Some("\"different\""),
                Some("Wed, 19 Jun 2026 00:00:00 GMT")
            ),
            RemoteFreshness::MustRehash
        );
        // no live ETag → weak path: size + Last-Modified match ⇒ probably fresh
        assert_eq!(
            sc.is_fresh_for_remote(size, None, Some("Wed, 19 Jun 2026 00:00:00 GMT")),
            RemoteFreshness::ProbablyFresh
        );
        // a size change is never cheaply fresh
        assert_eq!(
            sc.is_fresh_for_remote(size + 1, Some("\"abc123\""), None),
            RemoteFreshness::MustRehash
        );
        // a purely-local cert (no transport validators) can't be cheaply confirmed
        let local = Sidecar::assemble(bytes, "4.1".into(), stamp()).unwrap();
        assert_eq!(
            local.is_fresh_for_remote(size, None, None),
            RemoteFreshness::MustRehash
        );
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
