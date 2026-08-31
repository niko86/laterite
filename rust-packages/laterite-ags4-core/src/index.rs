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
//! HEADING/DATA row before the first GROUP, which `parse_reader` errors on). So the
//! lazy single-group path is for **well-formed files the parser already accepts**,
//! while the eager whole-file parse stays the validating, always-correct default.
//!
//! A file that splits one group across two sections used to be handled by keeping
//! the FIRST section and dropping the rest — so a sliced read returned a strict
//! subset of the whole-file parse's rows, silently. That was a locator stating a
//! location it did not have. The index now records **every** span of every code
//! ([`GroupIndex::spans`]), and [`GroupIndex::range`] returns `None` rather than
//! guess when a code is ambiguous — a caller that cannot be told where a group is
//! must re-parse the file, not read part of it and believe it read all of it.

use std::collections::HashMap;

use laterite_ags4_parse::{ParseOptions, ParsedFile, parse_bytes_opts};
use laterite_ags4_reference::effective_dict::FileDict;
// The certificate records HOW the edition was chosen, not just which one — a cert that
// said "exact" for a file whose edition was actually guessed (O-42) would misreport the
// one thing it exists to vouch for. The reference leaf owns the enum; core is already a
// consumer of it.
use laterite_ags4_reference::dict::DictResolution;
use serde::{Deserialize, Serialize};

use crate::ags4_codec::{AgsGroup, ReadOptions, read_ags4_bytes_with};
use crate::error::CliError;

/// Half-open byte range `[start, end)` of a group's section in the source bytes.
pub type Range = (u64, u64);

/// Where each group's section lives in an AGS4 file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupIndex {
    /// Group code → EVERY section it occupies, in source order.
    ///
    /// A `Vec`, not a `Range`, because a code can legally-in-bytes appear twice.
    /// The old single-`Range` map silently kept the FIRST section, so slicing a
    /// redeclared group returned a strict SUBSET of the rows a whole-file parse
    /// sees — with no error and no warning. A locator that cannot state where a
    /// group is must say so, not guess.
    pub groups: HashMap<String, Vec<Range>>,
    /// Section order as they appear in the file (de-duplicated; matches
    /// `ParsedAgs4::order`).
    pub order: Vec<String>,
}

impl GroupIndex {
    /// Every byte range `code` occupies, in source order. Empty if absent.
    pub fn spans(&self, code: &str) -> &[Range] {
        self.groups.get(code).map_or(&[], Vec::as_slice)
    }

    /// The byte range of `code`'s section — **only when it is unambiguous**.
    ///
    /// `None` for a redeclared group, deliberately: there is no single range, and
    /// returning the first one is the truncation this type exists to prevent. A
    /// caller that gets `None` must fall back to the whole-file parse.
    #[must_use]
    pub fn range(&self, code: &str) -> Option<Range> {
        match self.spans(code) {
            [only] => Some(*only),
            _ => None,
        }
    }

    /// Is `code` present exactly once? (A caller deciding whether it may trust a
    /// sliced read.)
    #[must_use]
    pub fn is_unambiguous(&self, code: &str) -> bool {
        self.spans(code).len() == 1
    }
}

/// Build the group-section byte index from the shared parse leaf's source-true
/// byte walk (#168 Phase 4). Each `"GROUP",…` record's `group_byte` starts a
/// section, which runs to the next group's start (or EOF). The leaf's offsets are
/// the real line-starts — the csv reader this replaced recorded the preceding
/// `\n` for CRLF groups and absorbed leading blank lines (see O-40).
/// The groups the file's own `DICT` declares, measured from the same bytes the
/// index locates (#768). Derived from `(bytes, index)` rather than a caller's
/// parse ON PURPOSE: `index_ags4_bytes` parses `locate_only` (rows dropped),
/// and a caller-supplied parse could be the same shape — reading DICT rows out
/// of either would silently yield "declares nothing" for a file that declares
/// plenty. Slicing the DICT spans the index already located and re-parsing
/// just those (DICT is small) is profile-independent and cannot be lied to.
///
/// Every span, not the first: a redeclared DICT gets the union, matching the
/// per-occurrence spans the v2 index exists to record. The reader is
/// [`FileDict`] — the one shared DICT implementation (#777), not a third copy.
fn file_defines(bytes: &[u8], index: &GroupIndex) -> Result<Vec<String>, CliError> {
    let mut out = std::collections::BTreeSet::new();
    for &(start, end) in index.spans("DICT") {
        // Byte offsets are u64; every shipped target is 64-bit (usize == u64),
        // so this is a no-op there. Bounds are still checked below via `.get()`.
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = (start as usize, end as usize);
        let slice = bytes
            .get(from..to)
            .ok_or_else(|| CliError::Schema("DICT span exceeds the file".into()))?;
        // Lean but NOT locate_only — this read needs the rows.
        let parsed = parse_bytes_opts(slice, ParseOptions::lean())
            .map_err(crate::ags4_codec::map_parse_err)?;
        let fd = FileDict::from_parsed(&parsed);
        out.extend(fd.groups().into_iter().map(str::to_string));
    }
    Ok(out.into_iter().collect())
}

pub fn index_ags4_bytes(bytes: &[u8]) -> Result<GroupIndex, CliError> {
    // Lean profile: reject invalid UTF-8 loudly — mirroring the csv reader
    // this replaced (which also failed on non-UTF-8). `lean()` never
    // substitutes bytes, so the offsets index the original buffer.
    // Locate-only: this function reads `group_records`, `group_order` and
    // `total_bytes`, and nothing else. Under the plain lean profile the walk
    // still tokenised every line into owned Strings and materialised every DATA
    // row, all of which was dropped on return — a full parse to keep ~123
    // records. Same walk, same guards, minus the model no one here reads.
    let opts = ParseOptions {
        locate_only: true,
        ..ParseOptions::lean()
    };
    let parsed = parse_bytes_opts(bytes, opts).map_err(crate::ags4_codec::map_parse_err)?;
    group_index_from_parsed(&parsed)
}

/// Build the group-section index from an ALREADY-PARSED file, reusing its
/// source-true byte offsets instead of re-walking `bytes`. `mint` parses the
/// file once to validate it and then re-derived these same offsets with a second
/// `index_ags4_bytes` walk (~45 ms on a 25 MB file, ~14% of a mint); handing that
/// parse here removes the second walk (#5). The `group_records`/`group_order`/
/// `total_bytes`/`byte_offsets_source_true` this reads are profile-independent —
/// a `locate_only` lean parse and a full `validating` parse record identical
/// GROUP offsets — so the index is byte-identical to `index_ags4_bytes`'s.
///
/// Guards source-truth exactly as `index_ags4_bytes` did: a cert whose offsets
/// don't index the original bytes is a lie. A caller whose parse was NOT
/// source-true (a lossy-replaced non-UTF-8 file) must NOT reach here with it —
/// `Sidecar::assemble_from_parsed` falls back to the lean re-walk in that case,
/// preserving the original rejection.
pub fn group_index_from_parsed(parsed: &ParsedFile) -> Result<GroupIndex, CliError> {
    if !parsed.byte_offsets_source_true {
        return Err(CliError::Schema(
            "byte offsets are not source-true (encoding substitution shifted a record start)"
                .into(),
        ));
    }

    let total = parsed.total_bytes;
    let mut groups: HashMap<String, Vec<Range>> = HashMap::with_capacity(parsed.group_order.len());

    // Walk EVERY `"GROUP"` record, not the de-duplicated `group_order`. A section
    // runs from its own record to the start of the NEXT record in source order —
    // which for a redeclared code means it gets one span per occurrence rather than
    // a single span silently covering only the first.
    //
    // Reading `group_order` here was the bug: it is first-seen-wins, so a file
    // declaring LOCA twice indexed only the first LOCA section. A sliced read of
    // that range returned a strict subset of the whole-file parse's rows, with no
    // error — and the DuckDB extension consumes exactly this index.
    for (i, rec) in parsed.group_records.iter().enumerate() {
        // A GROUP record with no code can't be located or sliced — reject it,
        // matching the retired csv index (and `ags4_codec::parse_reader`, which
        // still errors). The leaf yields an empty code for a bare `"GROUP"` row.
        if rec.code.is_empty() {
            return Err(CliError::Schema("GROUP row missing group code".into()));
        }
        let end = parsed
            .group_records
            .get(i + 1)
            .map_or(total, |next| next.byte_offset);
        groups
            .entry(rec.code.clone())
            .or_default()
            .push((rec.byte_offset, end));
    }

    Ok(GroupIndex {
        groups,
        order: parsed.group_order.clone(),
    })
}

/// Parse a single group from its indexed byte range, reusing the whole-file
/// parser on just that slice. The slice begins at a `"GROUP",…` record, so the
/// parser sees a self-contained one-group file.
pub fn parse_group_slice(bytes: &[u8], range: Range, code: &str) -> Result<AgsGroup, CliError> {
    parse_group_slice_with(bytes, range, code, ReadOptions::default())
}

/// [`parse_group_slice`] with explicit [`ReadOptions`].
///
/// The sliced read and the whole-file read must reach the same verdict on the
/// same bytes — an index is a shortcut to the answer, never a different answer —
/// so the caller's read policy has to reach here too. Without it, a handle
/// configured to tolerate something tolerated it only when the slice path
/// happened not to be taken, which depends on whether a certificate was fresh.
pub fn parse_group_slice_with(
    bytes: &[u8],
    range: Range,
    code: &str,
    opts: ReadOptions,
) -> Result<AgsGroup, CliError> {
    let (start, end) = range;
    // Byte offsets are u64; every shipped target is 64-bit (usize == u64),
    // so this is a no-op there. Bounds are still checked below via `.get()`.
    #[allow(clippy::cast_possible_truncation)]
    let (from, to) = (start as usize, end as usize);
    let slice = bytes.get(from..to).ok_or_else(|| {
        CliError::Schema(format!(
            "index range {start}..{end} out of bounds for {} bytes",
            bytes.len()
        ))
    })?;
    let mut parsed = read_ags4_bytes_with(slice, opts).map_err(|e| rebase_line(e, bytes, from))?;
    parsed
        .groups
        .remove(code)
        .ok_or_else(|| CliError::Schema(format!("group '{code}' not found in its indexed slice")))
}

/// Restate a slice-relative line number in whole-file terms.
///
/// The slice parser counts from the `"GROUP"` record it was handed, so it would
/// name line 3 of a group that starts at line 4000. A reader cannot act on that,
/// and the number looks plausible enough not to be questioned. Counting the
/// newlines before the slice is O(prefix) and only ever runs on the error path,
/// so the fast read the index exists to enable is untouched.
fn rebase_line(err: CliError, bytes: &[u8], start: usize) -> CliError {
    match err {
        CliError::ExcessFields {
            group,
            line,
            found,
            declared,
        } => {
            let before = u32::try_from(newlines_before(bytes, start)).unwrap_or(0);
            CliError::ExcessFields {
                group,
                line: line + before,
                found,
                declared,
            }
        }
        other => other,
    }
}

/// Newlines in `bytes[..start]` — the slice's own first line is line 1, so this
/// is exactly the offset to add.
// clippy wants the `bytecount` crate. This runs only when a read has already
// failed, and core is a leaf whose dependency list is a deliberate promise to
// the wasm build — a crate earning its place on an error path is a bad trade.
#[allow(clippy::naive_bytecount)]
fn newlines_before(bytes: &[u8], start: usize) -> usize {
    bytes
        .get(..start)
        .map_or(0, |prefix| prefix.iter().filter(|&&b| b == b'\n').count())
}

/// Format version of the `.ags.idx` sidecar.
///
/// **2**: two changes, both retiring a way the format could lie.
///
/// * `groups` maps a code to EVERY span it occupies (`Vec<Range>`), not one — a v1
///   cert recorded only a redeclared group's first section.
/// * the [`ValidationStamp`] records, per severity tier, whether the tier was
///   actually **measured** ([`TierCoverage`]) — v1 wrote `warnings: 0` whether it had
///   looked or not — carries the `EngineFingerprint` of the engine that produced the
///   verdict rather than a hand-bumped semver, and has **no `check_files` field at
///   all**, because the on-disk `FILE/` tree is not a property of the certified bytes
///   and no certificate may speak for it.
///
/// A v1 cert is refused, not migrated: every one of those fields is precisely the
/// untruth this version exists to retire, so there is nothing in it worth carrying
/// forward. Certificates are a regenerable cache and none are deployed, so an old one
/// simply falls back to a full validation.
pub const SIDECAR_VERSION: u32 = 2;

/// The shared **engine identity** every minting surface stamps into a
/// certificate's [`ValidationStamp::validator`]. They all run the same
/// `laterite_ags4_validator` rule engine, so a clean verdict is portable: a cert
/// minted by one door (Python, Node, the CLI, wasm) is trusted by another. The
/// *string* is trust-inert provenance — real trust gates on the
/// `EngineFingerprint` and the tier coverage — so unifying it removes an
/// accidental per-binding silo without weakening any real trust boundary.
///
/// The DuckDB extension is **not** in that list: it is read-only and mints
/// nothing (its own `cert.rs` says so — "minting lives *outside* this read-only
/// extension"). It is a pure consumer, and a narrower one than the doors above —
/// it uses the sidecar's byte-offset index for a sliced read and gates on **size**
/// alone, never reaching the fingerprint comparison, because re-hashing a remote
/// object to read one group would mean downloading it and defeating the point.
pub const ENGINE_IDENTITY: &str = "laterite_ags4";

/// What a validation run actually **measured** for one severity tier.
///
/// Deliberately not a bare `u32`. "I looked for warnings and found none" and "I never
/// looked for warnings" are different facts, and v1 could only express the first: its
/// `warnings: u32` defaulted to `0`, so a mint that never ran the warning rules
/// recorded a confident zero. Every certificate `laterite-py` ever produced said
/// `warnings: 0` without having measured anything, and a later `--show-warnings`
/// request read that zero and skipped the engine.
///
/// With this type, an unmeasured tier cannot be written as a clean one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TierCoverage {
    /// The rules for this tier were not run. The certificate has nothing to say about
    /// it, and a request that wants it must re-validate.
    NotMeasured,
    /// The rules for this tier ran and found `count` findings.
    Measured { count: u32 },
}

impl TierCoverage {
    /// Did this tier run AND come back empty? The only state in which a certificate
    /// can stand in for the engine — a cert stores counts, not findings, so it can
    /// only reproduce a report that has nothing in it.
    #[must_use]
    pub fn is_measured_clean(self) -> bool {
        matches!(self, TierCoverage::Measured { count: 0 })
    }
}

/// Which severity tier a [`RevalidateReason`] is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Errors,
    Warnings,
    Fyi,
}

/// How the edition that judged the file was arrived at.
///
/// One indivisible fact, where v1 had two fields that could disagree: `FileMeta.edition`
/// (a string) and `ValidationStamp.edition_forced` (a bool). A forced run and an
/// auto-resolved run can land on the same edition string having applied *different*
/// dictionaries — so the pair had to be compared together, and the old
/// `profile_covers` compared them apart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum EditionInput {
    /// Resolved from the file's own `TRAN_AGS` (with the O-42 content guard). A later
    /// run that also auto-resolves lands on the same edition — same bytes, same engine,
    /// same policy — so these two fields are provenance, not part of the trust test.
    Auto {
        /// The edition the rules ran against.
        resolved: String,
        /// *How* it was arrived at — exact `TRAN_AGS` match, a guessed patch (O-30 /
        /// O-42), or the fallback. Recorded because a surface reports it, and a cert
        /// that had to re-derive it would have to re-parse — which is the cost the
        /// certificate exists to avoid.
        resolution: DictResolution,
    },
    /// The caller overrode the file's declared edition (`--dict-version X`). Only a
    /// request forcing the SAME edition may be answered from this cert.
    Forced { edition: String },
}

impl EditionInput {
    /// The edition string the rules actually ran against, either way.
    #[must_use]
    pub fn edition(&self) -> &str {
        match self {
            EditionInput::Auto { resolved, .. } => resolved,
            EditionInput::Forced { edition } => edition,
        }
    }

    /// How the edition was chosen, as the surfaces report it.
    #[must_use]
    pub fn resolution(&self) -> DictResolution {
        match self {
            EditionInput::Auto { resolution, .. } => *resolution,
            EditionInput::Forced { .. } => DictResolution::Forced,
        }
    }

    /// Was it forced? (Provenance for the surfaces that report it.)
    #[must_use]
    pub fn is_forced(&self) -> bool {
        matches!(self, EditionInput::Forced { .. })
    }
}

/// Who is asking, and with what engine — the identity a certificate must match before
/// its verdict is worth anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineId {
    /// [`ENGINE_IDENTITY`].
    pub validator: String,
    /// `laterite_ags4_validator::ENGINE_FINGERPRINT` — a SHA-256 over the rule sources
    /// and the bundled dictionary. Not the crate's semver: a rule can change without
    /// the semver moving, and then a stale cert still looks current.
    pub fingerprint: String,
    /// `Some(v)` when validating through the python-ags4 compat shim, whose behaviour
    /// deliberately differs from the native engine. A compat verdict is not a native
    /// verdict; neither may answer for the other.
    pub compat: Option<String>,
}

/// The custom dictionary (laterite-dev#568 `--dict`) a verdict was reached against, recorded
/// on the certificate. It is a RECORD, not a contract: a mismatch — different
/// content, or a cert that names a dict the request doesn't supply (or vice
/// versa) — means "revalidate", never "hard-fail". The index vouches for what
/// happened; it does not bind the caller (cert-trust-v2, O-48). `name` is a human
/// label, advisory only; `hash` is the authority on identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDictRef {
    /// Advisory label — a declared name or the dict filename's basename, never a path.
    pub name: String,
    /// Hex SHA-256 over (normalised delta ⊕ base edition ⊕ mode): the identity that
    /// decides whether two requests used the same effective dictionary.
    pub hash: String,
}

/// The question a caller is asking of the file. A certificate may answer it only if it
/// can answer it **completely** — see [`Sidecar::decide`].
///
/// The world (Rule 20's on-disk `FILE/` tree) is deliberately absent. It is not a
/// question a certificate can be asked, because it is not a fact about the certified
/// bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The caller wants WARNING-severity findings too.
    pub want_warnings: bool,
    /// The caller wants FYI-severity findings too.
    pub want_fyi: bool,
    /// `Some(ed)` iff the caller is forcing an edition; `None` means auto-resolve.
    pub forced_edition: Option<String>,
    /// The decoder the caller is reading the bytes through (`encoding_rs` canonical
    /// name: `"UTF-8"`, `"windows-1252"`, …).
    ///
    /// It is here because the findings are a function of bytes **and decoder**, not
    /// bytes alone — and a certificate seals only the bytes. The same UTF-8 file
    /// containing `Ω` (bytes `CE A9`) is a Rule 1 **error** read as UTF-8 (one code
    /// point, 937 — above the extended-ASCII range the rule tolerates) and merely an
    /// **FYI** read as windows-1252 (two code points, 206 and 169 — both inside it).
    /// Mint under the lenient decoder, then read under the strict one, and a cert that
    /// compared everything *but* this would vouch for an error-clean file that has an
    /// error in it.
    pub encoding: String,
    /// The custom `--dict` overlay (laterite-dev#568) this request supplies, or `None` for the
    /// bundled path. Compared against the cert's own record in [`Sidecar::decide`]:
    /// a difference revalidates (never hard-fails).
    pub custom_dict: Option<CustomDictRef>,
}

/// May the certificate stand in for the engine?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Yes — and because a vouched cert is by definition one whose every asked-for
    /// tier was measured and found empty, the content findings it stands in for are
    /// **none**. There is nothing to reconstruct.
    Vouched,
    /// No. Run the engine. The reason is carried so a surface can say *why* it did not
    /// take the fast path (and so a test can assert which guard fired).
    Revalidate(RevalidateReason),
}

/// Why a certificate could not answer the question asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevalidateReason {
    /// Not a v2 cert.
    FormatVersion,
    /// The file is a different length than the one certified.
    SizeChanged,
    /// Same length, different bytes.
    ContentChanged,
    /// A different validator (or the compat shim vs the native engine).
    DifferentValidator,
    /// Same validator, different rules or dictionary — the fingerprint moved.
    DifferentEngine,
    /// The cert judged the file against a different dictionary than this request asks
    /// for (forced-vs-auto, or forced to a different edition).
    EditionDiffers,
    /// The cert read the bytes through a different decoder than this request asks for.
    /// The bytes are identical — the TEXT they become is not, and the rules judge text.
    EncodingDiffers,
    /// The caller asked about a tier the cert never ran. The old format could not even
    /// represent this state, so it never fired — it silently answered `0`.
    TierNotMeasured(Tier),
    /// The cert measured the tier and found findings. It stores counts, not findings,
    /// so it knows there is something to say but not what — the engine must speak.
    TierNotClean(Tier),
    /// The cert and this request name a different custom `--dict` overlay (laterite-dev#568) —
    /// one supplies a dict the other doesn't, or the same-named dict has different
    /// content. The effective dictionary changed, so the verdict may differ.
    DictionaryChanged,
}

impl RevalidateReason {
    /// A stable machine token for this reason — the single source the bindings surface
    /// (`report.revalidate_reason` on py/node/wasm) when a cert could not answer. The
    /// CLI's `why()` renders human prose for the same variants; this is its terse twin,
    /// so a caller can branch on the reason without parsing a sentence. Tokens are
    /// `snake_case` and match the tier suffix `Tier` serialises to.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RevalidateReason::FormatVersion => "format_version",
            RevalidateReason::SizeChanged => "size_changed",
            RevalidateReason::ContentChanged => "content_changed",
            RevalidateReason::DifferentValidator => "different_validator",
            RevalidateReason::DifferentEngine => "different_engine",
            RevalidateReason::EditionDiffers => "edition_differs",
            RevalidateReason::EncodingDiffers => "encoding_differs",
            RevalidateReason::TierNotMeasured(t) => match t {
                Tier::Errors => "tier_not_measured_errors",
                Tier::Warnings => "tier_not_measured_warnings",
                Tier::Fyi => "tier_not_measured_fyi",
            },
            RevalidateReason::TierNotClean(t) => match t {
                Tier::Errors => "tier_not_clean_errors",
                Tier::Warnings => "tier_not_clean_warnings",
                Tier::Fyi => "tier_not_clean_fyi",
            },
            RevalidateReason::DictionaryChanged => "dictionary_changed",
        }
    }
}

/// The source file a [`Sidecar`] certifies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    /// Source byte length.
    pub size: u64,
    /// Hex-encoded SHA-256 of the source bytes — the strong, portable,
    /// origin-independent staleness fingerprint, and the ground truth any cheaper
    /// check falls back to. Always present; never superseded by the optional
    /// transport validators below (a local file has no `ETag`, and an `ETag` is only
    /// meaningful relative to the endpoint that issued it).
    pub sha256: String,
    // NOTE: the AGS edition used to live here. It is a property of the VALIDATION —
    // which dictionary judged the file — not of the file's bytes, and it is
    // meaningless apart from whether it was forced or auto-resolved. Both facts now
    // live together in `ValidationStamp::edition` as one `EditionInput`, so they
    // cannot be compared apart (which is how `profile_covers` came to trust a forced
    // cert for an auto request).
    /// The remote origin's HTTP `ETag` observed at mint time, verbatim (`W/`
    /// weak prefix preserved), when minted from a remote (http/s3) source — else
    /// `None`. A *cheap* freshness shortcut: a HEAD whose `ETag` matches proves the
    /// object is byte-identical, so a remote reader can trust the SHA + byte
    /// offsets WITHOUT re-downloading to re-hash. Only ever grants trust on a
    /// match; absence/mismatch downgrades to the SHA path (see
    /// [`Sidecar::is_fresh_for_remote`]).
    #[serde(default)]
    pub etag: Option<String>,
    /// The remote origin's HTTP `Last-Modified` at mint time, when known — the
    /// weak fallback (paired with `size`) for stores that return no usable `ETag`.
    /// Weaker than the `ETag` (second granularity), so it gates the cheap ranged
    /// read but never the strong verdict on its own.
    #[serde(default)]
    pub last_modified: Option<String>,
}

/// The validation a [`Sidecar`] registers: which engine judged which bytes against
/// which dictionary, when, and — per severity tier — whether it actually looked.
///
/// A sidecar is minted only for a file with **zero error-severity findings**, so a
/// cert's existence asserts error-cleanliness. It asserts nothing else. In particular
/// it says nothing about the sibling `FILE/` tree: that is not a property of the
/// certified bytes, it can change without the file changing, and so there is
/// deliberately **no field here in which to record a claim about it**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStamp {
    /// What validated it — the shared [`ENGINE_IDENTITY`] string, so a clean cert
    /// minted by any surface is comparable by the others.
    pub validator: String,
    /// The [`EngineId::fingerprint`] of the engine that produced this verdict: a
    /// SHA-256 over the rule sources and the bundled dictionary.
    ///
    /// This replaces v1's `validator_version`, which was the validator crate's
    /// hand-bumped semver — a value that does not move when a rule's logic does. Edit a
    /// severity, fix a false positive, forget the bump, and every certificate minted by
    /// the old engine kept claiming to be current and kept being trusted.
    pub engine: String,
    /// The python-ags4 compatibility version when validated through the
    /// `laterite.compat` drop-in (whose behaviour deliberately mimics that python-ags4
    /// release), else `None` for the native engine. A compat verdict is not a native
    /// verdict — the two can disagree on a file — so neither answers for the other.
    #[serde(default)]
    pub compat: Option<String>,
    /// When, as an ISO-8601 / RFC-3339 UTC string, set by the minting layer.
    pub checked_at: String,
    /// The dictionary this verdict was reached against, and how it was chosen.
    pub edition: EditionInput,
    /// The decoder the bytes were read through (`encoding_rs` canonical name).
    ///
    /// A certificate seals the BYTES — but the rules see TEXT, and which text the bytes
    /// become is the decoder's answer, not the file's. Two decoders can reach two
    /// verdicts on one unchanged file (see [`Question::encoding`]), so a verdict that
    /// did not say which decoder produced it would be an incomplete statement about the
    /// content it claims to have checked.
    pub encoding: String,
    /// The custom `--dict` overlay (laterite-dev#568) this verdict was reached against, or `None`
    /// for the bundled path. `#[serde(default)]`: certs minted before laterite-dev#568 have no
    /// such field and correctly deserialise to `None` (a bundled verdict).
    #[serde(default)]
    pub custom_dict: Option<CustomDictRef>,
    /// Errors. A minted cert has always measured this tier (that is what it is FOR), so
    /// in practice this is `Measured { count: 0 }` — but the type does not assume it,
    /// because a type that assumes it is a type that can be lied to.
    pub errors: TierCoverage,
    /// Warnings — measured, or honestly recorded as unmeasured.
    pub warnings: TierCoverage,
    /// FYI — measured, or honestly recorded as unmeasured.
    pub fyi: TierCoverage,
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
    pub groups: HashMap<String, Vec<Range>>,
    /// Section order as in the source file.
    pub order: Vec<String>,
    /// The groups the file's own `DICT` declares (#768) — [`FileDict::groups`]
    /// semantics (touched by a `GROUP`-type row or a heading declaration),
    /// sorted. One fetch of the cert now says a file carries groups no
    /// standard dictionary has, instead of the reader fetching and parsing
    /// `DICT` to discover it; `edition.resolved` beside it is what to diff
    /// against. Names only, never the definitions — `groups["DICT"]` locates
    /// those precisely, and duplicating them here is the drift the issue
    /// weighed and declined.
    ///
    /// `Option` for the same reason [`TierCoverage`] is not a bare count:
    /// `None` means a cert minted before this field existed and MEASURED
    /// nothing, `Some(vec![])` means measured — the file declares nothing.
    /// A default would write the second meaning over the first.
    #[serde(default)]
    pub defines: Option<Vec<String>>,
}

/// Verdict of a cheap, I/O-free remote freshness check ([`Sidecar::is_fresh_for_remote`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreshness {
    /// A strong validator (`ETag`) matched — the object is byte-identical; trust the
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
    /// Assemble a sidecar for an already-validated file: hash the bytes, build the
    /// byte index, and attach the caller's [`ValidationStamp`].
    ///
    /// Core cannot validate — the rule engine sits above it — so it cannot check that
    /// the stamp is true. That is exactly why this is not a public minting door.
    /// `laterite_ags4_trust::mint` is: it runs the engine itself, builds the stamp from
    /// what the engine actually returned, and refuses a file with errors. Reach for
    /// that. This stays public only for the byte-index consumers (and the tests) that
    /// need a `Sidecar` without a verdict to trust.
    pub fn assemble(bytes: &[u8], validation: ValidationStamp) -> Result<Sidecar, CliError> {
        let index = index_ags4_bytes(bytes)?;
        Self::from_index(bytes, index, validation)
    }

    /// Like [`Sidecar::assemble`], but reuses a parse the caller already did
    /// rather than walking `bytes` a second time to rebuild the byte index (#5).
    /// `mint` validates the file by parsing it, then certifies it — this hands
    /// that parse straight in, removing the redundant ~14%-of-mint index walk.
    ///
    /// The reuse is sound only when `parsed`'s offsets index the ORIGINAL bytes
    /// (`byte_offsets_source_true`) — true for the clean UTF-8 files that certify.
    /// A lossy-replaced non-UTF-8 parse shifts offsets, so it falls back to
    /// [`index_ags4_bytes`], whose `Reject` profile produces the byte-identical
    /// rejection `assemble` gave before. The hash is over `bytes` either way.
    pub fn assemble_from_parsed(
        bytes: &[u8],
        parsed: &ParsedFile,
        validation: ValidationStamp,
    ) -> Result<Sidecar, CliError> {
        let index = if parsed.byte_offsets_source_true {
            group_index_from_parsed(parsed)?
        } else {
            index_ags4_bytes(bytes)?
        };
        Self::from_index(bytes, index, validation)
    }

    /// Assemble a `Sidecar` from a built `GroupIndex` + the caller's stamp,
    /// hashing `bytes` for the freshness check. Shared by [`Sidecar::assemble`]
    /// (which walks to build the index) and [`Sidecar::assemble_from_parsed`]
    /// (which reuses a parse), so the two differ ONLY in how the index is built.
    fn from_index(
        bytes: &[u8],
        index: GroupIndex,
        validation: ValidationStamp,
    ) -> Result<Sidecar, CliError> {
        let defines = file_defines(bytes, &index)?;
        Ok(Sidecar {
            version: SIDECAR_VERSION,
            file: FileMeta {
                size: bytes.len() as u64,
                sha256: sha256_hex(bytes),
                // Local mint by default; a remote-aware minting layer records the
                // origin's HTTP validators via `with_origin`.
                etag: None,
                last_modified: None,
            },
            validation,
            groups: index.groups,
            order: index.order,
            defines: Some(defines),
        })
    }

    /// Record the remote origin's HTTP validators (`ETag` / `Last-Modified`)
    /// observed at mint time, so a remote consumer can confirm freshness with a
    /// HEAD instead of re-downloading to re-hash. Builder; a local mint leaves both
    /// `None` (and the SHA stays the authoritative check regardless).
    #[must_use]
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
    #[must_use]
    pub fn is_fresh_for(&self, bytes: &[u8]) -> bool {
        self.version == SIDECAR_VERSION
            && self.file.size == bytes.len() as u64
            && self.file.sha256 == sha256_hex(bytes)
    }

    /// Cheap size-only freshness pre-check — for a remote source where re-hashing
    /// would mean re-downloading. Necessary but not sufficient; pair it with an
    /// ETag/Last-Modified check at the call site.
    #[must_use]
    pub fn size_matches(&self, size: u64) -> bool {
        self.file.size == size
    }

    /// **The trust rule.** May this certificate stand in for a run of the engine?
    ///
    /// > A certificate may answer a question only if it can answer it **completely**.
    ///
    /// That is the whole of it, and every clause below is a way of asking whether it
    /// can. A cert records *counts*, not findings — so the only report it can
    /// reproduce is an empty one, and the only question it can answer is one where
    /// every tier the caller asked about was measured and came back clean. Anything
    /// else and the engine runs.
    ///
    /// Six gates, cheapest first:
    ///
    /// 1. **format** — a v1 cert is not read (its fields are the untruths v2 retired).
    /// 2. **size**, then 3. **content** — the certified bytes must be *these* bytes.
    /// 4. **checker** — same validator, same compat profile, and the same
    ///    [`EngineId::fingerprint`]: not the crate's semver but a hash of the rules and
    ///    the dictionary, so a rule edit invalidates every cert that predates it even
    ///    if nobody bumped a version.
    /// 5. **edition** — the cert must have judged the file against the dictionary this
    ///    request asks for. Auto answers only auto (same bytes + same engine ⇒ the same
    ///    resolution, so its `resolved` value need not be compared); forced answers only
    ///    the same forcing.
    /// 6. **tiers** — for errors, and for warnings/FYI iff asked: measured, and clean.
    ///
    /// Note what is **not** here. There is no clause about Rule 20's on-disk `FILE/`
    /// tree, because there is no field about it, because a certificate cannot speak for
    /// the state of a directory it does not hash. World checks run live, every time,
    /// outside this decision entirely — see `laterite_ags4_trust::check`.
    #[must_use]
    pub fn decide(&self, bytes: &[u8], q: &Question, engine: &EngineId) -> Decision {
        use RevalidateReason as R;
        let v = &self.validation;

        if self.version != SIDECAR_VERSION {
            return Decision::Revalidate(R::FormatVersion);
        }
        if self.file.size != bytes.len() as u64 {
            return Decision::Revalidate(R::SizeChanged);
        }
        if self.file.sha256 != sha256_hex(bytes) {
            return Decision::Revalidate(R::ContentChanged);
        }
        if v.validator != engine.validator || v.compat != engine.compat {
            return Decision::Revalidate(R::DifferentValidator);
        }
        if v.engine != engine.fingerprint {
            return Decision::Revalidate(R::DifferentEngine);
        }
        // The bytes are sealed by the SHA above; the DECODER is not part of them. A cert
        // minted through one decoder cannot answer a request made through another, because
        // they are questions about two different texts.
        if v.encoding != q.encoding {
            return Decision::Revalidate(R::EncodingDiffers);
        }
        match (&v.edition, q.forced_edition.as_deref()) {
            // Auto for auto: a later auto-resolve over the same bytes with the same
            // engine reaches the same edition, so there is nothing to compare.
            (EditionInput::Auto { .. }, None) => {}
            (EditionInput::Forced { edition }, Some(want)) if edition == want => {}
            _ => return Decision::Revalidate(R::EditionDiffers),
        }
        // The custom `--dict` overlay (laterite-dev#568). A difference — different content, or one
        // side present and the other absent — changes the effective dictionary, so the
        // cert answers a different question. This match is hand-written, not
        // exhaustiveness-checked, so this arm is deliberate: warn-and-revalidate, NOT a
        // hard error, because the index is a record of what happened, not a contract the
        // caller must honour (O-48).
        if v.custom_dict != q.custom_dict {
            return Decision::Revalidate(R::DictionaryChanged);
        }
        // Errors are always asked about — a report always reports them.
        for (tier, coverage) in [
            (Tier::Errors, v.errors),
            (Tier::Warnings, v.warnings),
            (Tier::Fyi, v.fyi),
        ] {
            let asked = match tier {
                Tier::Errors => true,
                Tier::Warnings => q.want_warnings,
                Tier::Fyi => q.want_fyi,
            };
            if !asked {
                continue;
            }
            match coverage {
                TierCoverage::NotMeasured => return Decision::Revalidate(R::TierNotMeasured(tier)),
                TierCoverage::Measured { count: 0 } => {}
                TierCoverage::Measured { .. } => {
                    return Decision::Revalidate(R::TierNotClean(tier));
                }
            }
        }
        Decision::Vouched
    }

    /// Cheap, **I/O-free** remote freshness check against a live HEAD's observed
    /// `(size, etag, last_modified)`. Core never does the network I/O — the caller
    /// (the DuckDB VFS / httpfs / a remote reader) performs the HEAD and passes the
    /// observed values in. The optional transport validators can only ever GRANT
    /// trust on a match; absence or mismatch downgrades toward the strong SHA path
    /// ([`Sidecar::is_fresh_for`]), never the reverse — so they can never make a
    /// stale cert look fresh.
    #[must_use]
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
    #[must_use]
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
    use std::fmt::Write as _;

    use super::*;
    use crate::ags4_codec::read_ags4_bytes;

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

    /// A code declared TWICE, with another group between the two sections. The
    /// whole-file parse merges both sections' rows into one group; the index must
    /// therefore be able to SAY there are two places to look.
    const REDECLARED: &str = r#""GROUP","PROJ"
"HEADING","PROJ_ID"
"UNIT",""
"TYPE","ID"
"DATA","P1"

"GROUP","LOCA"
"HEADING","LOCA_ID"
"UNIT",""
"TYPE","ID"
"DATA","BH01"

"GROUP","ABBR"
"HEADING","ABBR_CODE"
"UNIT",""
"TYPE","X"
"DATA","CP"

"GROUP","LOCA"
"HEADING","LOCA_ID"
"UNIT",""
"TYPE","ID"
"DATA","BH02"
"#;

    #[test]
    fn two_groups_lf() {
        assert_consistent(TWO);
    }

    /// The index records EVERY span of a redeclared code — not just the first.
    ///
    /// This is the locator lie: `groups` was `HashMap<String, Range>`, so a code
    /// appearing twice kept only its first section. The DuckDB extension slices from
    /// exactly this index, so it re-parsed the first section and returned BH01 alone
    /// while the whole-file parse sees BH01 *and* BH02 — a silently truncated read.
    #[test]
    fn a_redeclared_group_records_every_span() {
        let bytes = REDECLARED.as_bytes();
        let idx = index_ags4_bytes(bytes).unwrap();

        assert_eq!(idx.spans("LOCA").len(), 2, "LOCA occupies two sections");
        assert_eq!(idx.spans("PROJ").len(), 1);
        assert_eq!(idx.spans("ABBR").len(), 1);

        // The two LOCA spans are disjoint and in source order, and the ABBR section
        // sits between them — i.e. they are genuinely two places, not one range.
        let loca = idx.spans("LOCA").to_vec();
        assert!(loca[0].1 <= loca[1].0, "spans are disjoint and ordered");
        let abbr = idx.range("ABBR").unwrap();
        assert!(
            loca[0].1 <= abbr.0 && abbr.1 <= loca[1].0,
            "ABBR lies between the two LOCA sections — a single range could not span them"
        );

        // Each span, sliced and re-parsed, yields the row that section declared.
        let first = parse_group_slice(bytes, loca[0], "LOCA").unwrap();
        let second = parse_group_slice(bytes, loca[1], "LOCA").unwrap();
        assert_eq!(first.rows[0]["LOCA_ID"], "BH01");
        assert_eq!(second.rows[0]["LOCA_ID"], "BH02");
    }

    /// `range()` REFUSES an ambiguous code rather than handing back the first span.
    ///
    /// Returning the first is what made the truncation silent. A caller that cannot
    /// be told where a group is must re-parse the file — so it is told `None`, not a
    /// half-truth it has no way to detect.
    #[test]
    fn range_refuses_to_guess_for_a_redeclared_group() {
        let idx = index_ags4_bytes(REDECLARED.as_bytes()).unwrap();

        assert_eq!(idx.range("LOCA"), None, "ambiguous — must not pick one");
        assert!(!idx.is_unambiguous("LOCA"));

        // The unambiguous codes still resolve, so the refusal is targeted, not blunt.
        assert!(idx.range("PROJ").is_some());
        assert!(idx.is_unambiguous("PROJ"));
    }

    /// The whole-file parse merges a redeclared group's rows. Pinned here because it
    /// is the reason the index must record both spans: the parse sees BH01+BH02, so
    /// an index that can only point at BH01 disagrees with the parser about what the
    /// file contains.
    #[test]
    fn the_whole_file_parse_sees_both_sections_rows() {
        let whole = read_ags4_bytes(REDECLARED.as_bytes()).unwrap();
        let loca = whole.get("LOCA").unwrap();
        let ids: Vec<&str> = loca.rows.iter().map(|r| r["LOCA_ID"].as_str()).collect();
        assert_eq!(ids, vec!["BH01", "BH02"]);
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

    /// The sliced read is a shortcut to the same answer, never a different one —
    /// so #776's refusal has to reach it too. LOCA is the SECOND group here on
    /// purpose: the slice parser counts lines from the `"GROUP"` record it was
    /// handed, so an unrebased line number would come back as 3 and look
    /// entirely plausible.
    #[test]
    fn a_sliced_read_refuses_excess_fields_and_names_the_whole_file_line() {
        let f = "\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\"\n\
\"UNIT\",\"\"\n\
\"TYPE\",\"ID\"\n\
\"DATA\",\"P1\"\n\
\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_LOCX\"\n\
\"DATA\",\"BH01\",Acme, Bloggs\n";
        let bytes = f.as_bytes();
        let idx = index_ags4_bytes(bytes).unwrap();
        let range = idx.range("LOCA").unwrap();

        let err = parse_group_slice(bytes, range, "LOCA").unwrap_err();
        let CliError::ExcessFields { line, .. } = err else {
            panic!("expected ExcessFields, got {err:?}");
        };
        assert_eq!(line, 8, "the line must be the file's, not the slice's");

        // And the caller's opt-out reaches the slice path too, or a handle would
        // tolerate the row only when no certificate happened to be fresh.
        let opts = ReadOptions {
            excess_fields: crate::ags4_codec::ExcessFields::Truncate,
            ..ReadOptions::default()
        };
        let loca = parse_group_slice_with(bytes, range, "LOCA", opts).expect("truncates");
        assert_eq!(loca.rows[0]["LOCA_LOCX"], "Acme");
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
                    let _ = writeln!(s, "\"GROUP\",\"G{g:03}\"");
                    s.push_str("\"HEADING\",\"A_ID\",\"A_VAL\"\n");
                    s.push_str("\"UNIT\",\"\",\"\"\n");
                    s.push_str("\"TYPE\",\"ID\",\"X\"\n");
                    // vary row counts per group so ranges differ in size
                    for r in 0..(g * row_step) {
                        let _ = writeln!(s, "\"DATA\",\"K{g}_{r}\",\"v{r}\"");
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

    /// The small `TierCoverage`/`EditionInput` accessors `decide` is built on.
    /// They read clean under mutation because every caller routes through
    /// `decide`, which has many other reasons to revalidate — so a wrong
    /// accessor is masked. Pin them directly.
    #[test]
    fn tier_and_edition_accessors_read_their_own_state() {
        // is_measured_clean: TRUE only for a measured, zero-finding tier.
        assert!(TierCoverage::Measured { count: 0 }.is_measured_clean());
        assert!(!TierCoverage::Measured { count: 1 }.is_measured_clean());
        assert!(!TierCoverage::NotMeasured.is_measured_clean());

        // edition(): the resolved string for Auto, the forced string for Forced.
        let auto = EditionInput::Auto {
            resolved: "4.1".into(),
            resolution: DictResolution::ExactTranAgs,
        };
        let forced = EditionInput::Forced {
            edition: "4.0.4".into(),
        };
        assert_eq!(auto.edition(), "4.1");
        assert_eq!(forced.edition(), "4.0.4");

        // is_forced(): only Forced is forced.
        assert!(!auto.is_forced());
        assert!(forced.is_forced());
    }

    fn stamp() -> ValidationStamp {
        ValidationStamp {
            validator: "test".into(),
            engine: "0000000000000000".into(),
            compat: None,
            checked_at: "2026-06-19T00:00:00Z".into(),
            edition: EditionInput::Auto {
                resolved: "4.1".into(),
                resolution: DictResolution::ExactTranAgs,
            },
            encoding: "UTF-8".into(),
            custom_dict: None,
            errors: TierCoverage::Measured { count: 0 },
            warnings: TierCoverage::Measured { count: 0 },
            fyi: TierCoverage::Measured { count: 1 },
        }
    }

    /// The engine that minted `stamp()` — the identity `decide` compares against.
    fn asking_engine() -> EngineId {
        EngineId {
            validator: "test".into(),
            fingerprint: "0000000000000000".into(),
            compat: None,
        }
    }

    fn errors_only() -> Question {
        Question {
            want_warnings: false,
            want_fyi: false,
            forced_edition: None,
            encoding: "UTF-8".into(),
            custom_dict: None,
        }
    }

    /// #5: `assemble_from_parsed` (the reuse path `mint` takes) must produce the
    /// byte-identical `Sidecar` that `assemble` (the second walk) produced — same
    /// index, same order, same hash. Parses with `validating()`, exactly the
    /// profile `mint` hands in, so this pins the real reuse, not a lean twin.
    #[test]
    fn assemble_from_parsed_matches_the_walk() {
        let bytes = TWO.as_bytes();
        let walked = Sidecar::assemble(bytes, stamp()).unwrap();

        let parsed = parse_bytes_opts(bytes, ParseOptions::validating()).unwrap();
        assert!(
            parsed.byte_offsets_source_true,
            "a clean UTF-8 file parses source-true"
        );
        let reused = Sidecar::assemble_from_parsed(bytes, &parsed, stamp()).unwrap();

        assert_eq!(reused.groups, walked.groups, "index groups identical");
        assert_eq!(reused.order, walked.order, "order identical");
        assert_eq!(reused.file.sha256, walked.file.sha256, "hash identical");
        assert_eq!(reused.file.size, walked.file.size, "size identical");
    }

    /// #5: a non-UTF-8 file parsed under `validating()`'s lossy profile is NOT
    /// source-true (its offsets shifted), so the reuse path must fall back to the
    /// lean re-walk and reject it EXACTLY as `assemble` did — the non-UTF-8 mint
    /// semantics must not change silently.
    #[test]
    fn assemble_from_parsed_falls_back_when_not_source_true() {
        // A structurally-valid one-group file with an invalid byte (0xFF) in a
        // DATA cell: `validating()` lossy-replaces it (parse succeeds, not
        // source-true); `lean()`/`Reject` (index_ags4_bytes) rejects it.
        let bytes: &[u8] =
            b"\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"BH\xff01\"\n";

        let parsed = parse_bytes_opts(bytes, ParseOptions::validating()).unwrap();
        assert!(
            !parsed.byte_offsets_source_true,
            "a lossy-replaced byte makes offsets non-source-true"
        );

        let walked = Sidecar::assemble(bytes, stamp());
        let reused = Sidecar::assemble_from_parsed(bytes, &parsed, stamp());
        assert!(walked.is_err(), "the walk rejects non-UTF-8");
        assert!(reused.is_err(), "the reuse path falls back and rejects too");
        assert_eq!(
            format!("{:?}", walked.unwrap_err()),
            format!("{:?}", reused.unwrap_err()),
            "the fallback yields the byte-identical rejection"
        );
    }

    /// The bytes are sealed. The DECODER is not part of them — and the rules judge the
    /// TEXT the decoder produces, not the bytes. A cert minted through one decoder must
    /// not answer a question asked through another.
    ///
    /// This is not hypothetical: a UTF-8 file containing `Ω` (bytes `CE A9`) is a Rule 1
    /// ERROR read as UTF-8 (one code point, 937) and only an FYI read as windows-1252
    /// (two code points, 206 and 169). Certify it under the lenient decoder and, before
    /// this gate, a default read of the very same bytes came back clean and certified.
    #[test]
    fn a_cert_minted_through_another_decoder_cannot_answer() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap(); // stamped UTF-8
        let cp1252 = Question {
            encoding: "windows-1252".into(),
            ..errors_only()
        };
        assert_eq!(
            sc.decide(bytes, &cp1252, &asking_engine()),
            Decision::Revalidate(RevalidateReason::EncodingDiffers),
            "same bytes, different decoder — a different text, so a different question"
        );
        assert_eq!(
            sc.decide(bytes, &errors_only(), &asking_engine()),
            Decision::Vouched,
            "the decoder it was minted under still answers"
        );
    }

    /// The custom-dict comparison arm in `decide` is hand-written, NOT
    /// exhaustiveness-checked (`decide` is a sequence of `if`s), so a forgotten
    /// arm would silently reopen O-48. This pins all four cases — and pins that a
    /// difference REVALIDATES, never hard-fails (the index is a record, laterite-dev#568 §4).
    #[test]
    fn a_custom_dict_difference_revalidates_rather_than_hard_fails() {
        use RevalidateReason as R;
        let bytes = TWO.as_bytes();
        let e = asking_engine();
        let dref = |hash: &str| CustomDictRef {
            name: "mine".into(),
            hash: hash.into(),
        };

        // A cert minted WITH a custom dict.
        let mut minted_with = stamp();
        minted_with.custom_dict = Some(dref("aaaa"));
        let sc = Sidecar::assemble(bytes, minted_with).unwrap();

        // Same dict → the mismatch arm does not fire (and the cert is otherwise
        // clean, so it vouches).
        let same = Question {
            custom_dict: Some(dref("aaaa")),
            ..errors_only()
        };
        assert_eq!(sc.decide(bytes, &same, &e), Decision::Vouched);

        // Different content → revalidate.
        let other = Question {
            custom_dict: Some(dref("bbbb")),
            ..errors_only()
        };
        assert_eq!(
            sc.decide(bytes, &other, &e),
            Decision::Revalidate(R::DictionaryChanged),
            "a different custom dict is a different question"
        );

        // Cert has a dict, request supplies none → revalidate (NOT a hard error).
        let none = Question {
            custom_dict: None,
            ..errors_only()
        };
        assert_eq!(
            sc.decide(bytes, &none, &e),
            Decision::Revalidate(R::DictionaryChanged),
            "a bare request cannot inherit a custom-dict verdict — but it revalidates, \
             it is not refused"
        );

        // Symmetric: a bundled cert cannot answer a custom-dict request.
        let bundled = Sidecar::assemble(bytes, stamp()).unwrap(); // stamp() has no dict
        let wants_dict = Question {
            custom_dict: Some(dref("aaaa")),
            ..errors_only()
        };
        assert_eq!(
            bundled.decide(bytes, &wants_dict, &e),
            Decision::Revalidate(R::DictionaryChanged)
        );
    }

    #[test]
    fn revalidate_reason_tokens_are_stable() {
        use RevalidateReason as R;
        // The bindings (py/node/wasm) surface these tokens verbatim, so a rename here is
        // an API break — this pins the ones a caller is most likely to branch on. The
        // match in `as_str` is exhaustive, so a NEW variant fails to compile until it is
        // given a token; this guards the spelling of the ones already shipped.
        assert_eq!(R::DictionaryChanged.as_str(), "dictionary_changed");
        assert_eq!(R::EditionDiffers.as_str(), "edition_differs");
        assert_eq!(R::ContentChanged.as_str(), "content_changed");
        assert_eq!(
            R::TierNotMeasured(Tier::Errors).as_str(),
            "tier_not_measured_errors"
        );
        assert_eq!(R::TierNotClean(Tier::Fyi).as_str(), "tier_not_clean_fyi");
    }

    #[test]
    fn sidecar_json_round_trips() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap();
        let back = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(sc, back, "sidecar survives a JSON round-trip");
        // the embedded index matches a direct scan of the same bytes
        assert_eq!(back.index(), index_ags4_bytes(bytes).unwrap());
        assert_eq!(back.file.sha256.len(), 64, "sha256 is 64 hex chars");
    }

    #[test]
    fn sidecar_freshness_tracks_the_source() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap();
        assert!(
            sc.is_fresh_for(bytes),
            "fresh for the bytes it was built from"
        );
        assert!(sc.size_matches(bytes.len() as u64));
        assert!(
            !sc.size_matches(bytes.len() as u64 + 1),
            "a different size does not match"
        );
        // any change to the source busts it (the sha differs)
        let mut changed = bytes.to_vec();
        changed.push(b'\n');
        assert!(!sc.is_fresh_for(&changed), "a changed file is not fresh");
        // SAME size, different content — the size clause passes, so only the sha
        // clause can reject it. This is the case that separates the `size && sha`
        // from a `size || sha`: a cert must not vouch for a same-length edit.
        let mut same_len = bytes.to_vec();
        let mid = same_len.len() / 2;
        same_len[mid] ^= 0xFF;
        assert_eq!(same_len.len(), bytes.len(), "the edit kept the length");
        assert!(
            !sc.is_fresh_for(&same_len),
            "same size but different bytes is not fresh"
        );
    }

    #[test]
    fn the_format_has_no_field_in_which_to_claim_a_world_check() {
        // The strongest of the three defences: v1 had `check_files: bool`, and a
        // consumer read it to conclude that a cert covered a `--check-files` request —
        // for a `FILE/` tree that could have been deleted since. There is now nothing
        // to read. Asserted on the SERIALISED form, because that is what a future
        // consumer would parse.
        let json = String::from_utf8(
            Sidecar::assemble(TWO.as_bytes(), stamp())
                .unwrap()
                .to_json()
                .unwrap(),
        )
        .unwrap();
        assert!(!json.contains("check_files"), "{json}");
        assert!(!json.contains("edition_forced"), "{json}");
        // ...and the tier counts cannot be written as bare zeros either.
        assert!(json.contains("measured"), "{json}");
    }

    #[test]
    fn decide_vouches_only_for_a_question_it_can_fully_answer() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap();
        let e = asking_engine();

        // errors-only: measured, clean → vouched.
        assert_eq!(sc.decide(bytes, &errors_only(), &e), Decision::Vouched);

        // warnings: measured (0) → still vouched.
        let want_warn = Question {
            want_warnings: true,
            ..errors_only()
        };
        assert_eq!(sc.decide(bytes, &want_warn, &e), Decision::Vouched);

        // fyi: measured, but there IS one (stamp() records fyi = 1). The cert stores a
        // COUNT, not the finding — it knows there is something to say and cannot say
        // it. So the engine must run.
        let want_fyi = Question {
            want_fyi: true,
            ..errors_only()
        };
        assert_eq!(
            sc.decide(bytes, &want_fyi, &e),
            Decision::Revalidate(RevalidateReason::TierNotClean(Tier::Fyi))
        );
    }

    #[test]
    fn decide_refuses_an_unmeasured_tier_rather_than_reading_a_zero() {
        // The state v1 could not represent: it wrote `warnings: 0` whether or not the
        // warning rules had run, so a `--show-warnings` request read that zero and
        // skipped the engine.
        let bytes = TWO.as_bytes();
        let mut st = stamp();
        st.warnings = TierCoverage::NotMeasured;
        let sc = Sidecar::assemble(bytes, st).unwrap();
        let e = asking_engine();

        assert_eq!(sc.decide(bytes, &errors_only(), &e), Decision::Vouched);
        assert_eq!(
            sc.decide(
                bytes,
                &Question {
                    want_warnings: true,
                    ..errors_only()
                },
                &e
            ),
            Decision::Revalidate(RevalidateReason::TierNotMeasured(Tier::Warnings))
        );
    }

    #[test]
    fn decide_gates_on_the_engine_that_produced_the_verdict() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap();

        // A rule changed under the cert: same validator, different fingerprint.
        let moved = EngineId {
            fingerprint: "ffffffffffffffff".into(),
            ..asking_engine()
        };
        assert_eq!(
            sc.decide(bytes, &errors_only(), &moved),
            Decision::Revalidate(RevalidateReason::DifferentEngine)
        );

        // A compat consumer does not trust a native-minted cert, nor the reverse.
        let compat = EngineId {
            compat: Some("python-ags4-0.5".into()),
            ..asking_engine()
        };
        assert_eq!(
            sc.decide(bytes, &errors_only(), &compat),
            Decision::Revalidate(RevalidateReason::DifferentValidator)
        );
    }

    #[test]
    fn decide_gates_on_the_bytes() {
        let bytes = TWO.as_bytes();
        let sc = Sidecar::assemble(bytes, stamp()).unwrap();
        let e = asking_engine();

        let mut longer = bytes.to_vec();
        longer.push(b'\n');
        assert_eq!(
            sc.decide(&longer, &errors_only(), &e),
            Decision::Revalidate(RevalidateReason::SizeChanged)
        );

        // Same length, different content — the SHA is what catches this.
        let mut same_len = bytes.to_vec();
        let i = same_len.len() - 3;
        same_len[i] = b'Z';
        assert_eq!(
            sc.decide(&same_len, &errors_only(), &e),
            Decision::Revalidate(RevalidateReason::ContentChanged)
        );
    }

    #[test]
    fn an_auto_cert_and_a_forced_request_are_different_questions() {
        // v1 kept the edition string and the forced flag in SEPARATE structs and
        // compared them apart. They are one fact: a forced run ignores TRAN_AGS, so on
        // a file whose declared edition disagrees with its content the two runs apply
        // different dictionaries — even when the edition STRING is the same.
        let bytes = TWO.as_bytes();
        let auto = Sidecar::assemble(bytes, stamp()).unwrap();
        let e = asking_engine();

        assert_eq!(
            auto.decide(
                bytes,
                &Question {
                    forced_edition: Some("4.1".into()),
                    ..errors_only()
                },
                &e
            ),
            Decision::Revalidate(RevalidateReason::EditionDiffers),
            "same edition string, different question"
        );

        let mut st = stamp();
        st.edition = EditionInput::Forced {
            edition: "4.0.4".into(),
        };
        let forced = Sidecar::assemble(bytes, st).unwrap();
        assert_eq!(
            forced.decide(
                bytes,
                &Question {
                    forced_edition: Some("4.0.4".into()),
                    ..errors_only()
                },
                &e
            ),
            Decision::Vouched,
            "the same forcing IS the same question"
        );
        assert_eq!(
            forced.decide(bytes, &errors_only(), &e),
            Decision::Revalidate(RevalidateReason::EditionDiffers),
            "a forced cert does not answer an auto request"
        );
        // Forced cert vs a request forcing a DIFFERENT edition. The guard is
        // `edition == want`; without this case a cert forced at 4.0.4 would appear
        // to answer a `--dict-version 4.2` request.
        assert_eq!(
            forced.decide(
                bytes,
                &Question {
                    forced_edition: Some("4.2".into()),
                    ..errors_only()
                },
                &e
            ),
            Decision::Revalidate(RevalidateReason::EditionDiffers),
            "forcing a different edition is a different question",
        );
    }

    #[test]
    fn compat_provenance_round_trips() {
        let bytes = TWO.as_bytes();
        let mut st = stamp();
        st.compat = Some("python-ags4-0.5.0".into());
        let sc = Sidecar::assemble(bytes, st).unwrap();
        let back = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(back.validation.compat.as_deref(), Some("python-ags4-0.5.0"));
        assert_eq!(
            back.decide(
                bytes,
                &errors_only(),
                &EngineId {
                    compat: Some("python-ags4-0.5.0".into()),
                    ..asking_engine()
                }
            ),
            Decision::Vouched
        );
    }

    #[test]
    fn a_v1_cert_is_refused_not_migrated() {
        // Every field v1 carried is one of the untruths v2 exists to retire, so there is
        // nothing in it worth reading. It is a regenerable cache; refusing costs a
        // re-validation and nothing else.
        let v1 = br#"{"version":1,"file":{"size":1,"sha256":"x","edition":"4.1"},
            "validation":{"validator":"v","validator_version":"1","checked_at":"t",
            "check_files":true,"warnings":0,"fyi":0},"groups":{},"order":[]}"#;
        assert!(
            Sidecar::from_json(v1).is_err(),
            "a v1 cert is not read as a v2"
        );
    }

    #[test]
    fn remote_freshness_grants_only_on_a_validator_match() {
        let bytes = TWO.as_bytes();
        let size = bytes.len() as u64;
        let sc = Sidecar::assemble(bytes, stamp()).unwrap().with_origin(
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
        let local = Sidecar::assemble(bytes, stamp()).unwrap();
        assert_eq!(
            local.is_fresh_for_remote(size, None, None),
            RemoteFreshness::MustRehash
        );
    }

    #[test]
    fn sidecar_rejects_unknown_version() {
        let bytes = TWO.as_bytes();
        let mut sc = Sidecar::assemble(bytes, stamp()).unwrap();
        sc.version = SIDECAR_VERSION + 1;
        assert!(
            Sidecar::from_json(&sc.to_json().unwrap()).is_err(),
            "an unknown sidecar version is rejected, not silently trusted"
        );
    }

    // --- #768: the cert says what the file defines -------------------------

    /// TWO, plus a DICT declaring one bespoke group and extending LOCA with a
    /// heading — the two ways a file-declared group enters `FileDict::groups`.
    const WITH_DICT: &str = r#""GROUP","PROJ"
"HEADING","PROJ_ID"
"UNIT",""
"TYPE","ID"
"DATA","P1"

"GROUP","DICT"
"HEADING","DICT_TYPE","DICT_GRP","DICT_HDNG","DICT_STAT","DICT_DTYP","DICT_UNIT","DICT_DESC","DICT_PGRP"
"UNIT","","","","","","","",""
"TYPE","PA","X","X","X","PT","PU","X","X"
"DATA","GROUP","MONG","","","","","Monitoring bespoke","PROJ"
"DATA","HEADING","MONG","MONG_ID","KEY","ID","","Monitoring id",""
"DATA","HEADING","LOCA","LOCA_CUST","OTHER","X","","Custom remark",""

"GROUP","MONG"
"HEADING","MONG_ID"
"UNIT",""
"TYPE","ID"
"DATA","M1"
"#;

    #[test]
    fn a_minted_cert_names_the_groups_the_dict_declares() {
        let sc = Sidecar::assemble(WITH_DICT.as_bytes(), stamp()).unwrap();
        assert_eq!(
            sc.defines.as_deref(),
            Some(&["LOCA".to_string(), "MONG".to_string()][..]),
            "sorted union of GROUP-type declarations and heading extensions"
        );
    }

    #[test]
    fn a_file_with_no_dict_measures_an_empty_defines_not_an_absent_one() {
        // "Looked and found none" — Some(vec![]) — must survive a JSON round
        // trip distinct from None, or the field re-learns v1's confident-zero
        // lie one level up.
        let sc = Sidecar::assemble(TWO.as_bytes(), stamp()).unwrap();
        assert_eq!(sc.defines.as_deref(), Some(&[][..]));
        let back = Sidecar::from_json(&sc.to_json().unwrap()).unwrap();
        assert_eq!(back.defines.as_deref(), Some(&[][..]));
    }

    #[test]
    fn a_cert_minted_before_the_field_reads_as_unmeasured() {
        // A pre-#768 v2 cert has no `defines` key at all. It must come back as
        // None — "nothing was measured" — never as a measured empty.
        let sc = Sidecar::assemble(WITH_DICT.as_bytes(), stamp()).unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&sc.to_json().unwrap()).unwrap();
        json.as_object_mut().unwrap().remove("defines");
        let old = Sidecar::from_json(&serde_json::to_vec(&json).unwrap()).unwrap();
        assert_eq!(old.defines, None);
    }

    #[test]
    fn a_redeclared_dict_contributes_every_section() {
        // The v2 index exists because a redeclared group's later sections were
        // silently dropped; `defines` must not repeat that with DICT itself.
        let two_dicts = concat!(
            "\"GROUP\",\"DICT\"\n",
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\n",
            "\"UNIT\",\"\",\"\",\"\"\n",
            "\"TYPE\",\"PA\",\"X\",\"X\"\n",
            "\"DATA\",\"GROUP\",\"AAAA\",\"\"\n",
            "\n",
            "\"GROUP\",\"PROJ\"\n",
            "\"HEADING\",\"PROJ_ID\"\n",
            "\"UNIT\",\"\"\n",
            "\"TYPE\",\"ID\"\n",
            "\"DATA\",\"P1\"\n",
            "\n",
            "\"GROUP\",\"DICT\"\n",
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\n",
            "\"UNIT\",\"\",\"\",\"\"\n",
            "\"TYPE\",\"PA\",\"X\",\"X\"\n",
            "\"DATA\",\"GROUP\",\"BBBB\",\"\"\n",
        );
        let sc = Sidecar::assemble(two_dicts.as_bytes(), stamp()).unwrap();
        assert_eq!(
            sc.defines.as_deref(),
            Some(&["AAAA".to_string(), "BBBB".to_string()][..])
        );
    }
}
