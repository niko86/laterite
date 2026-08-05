//! Minting and consuming `.ags.idx` validity certificates.
//!
//! Public API only, like the rest of this crate's suite. The properties worth
//! nailing down are the ones a certificate could get *quietly* wrong:
//!
//! - a certificate is over the ORIGINAL bytes, so one minted from a transcoded
//!   read still matches the file on disk,
//! - a stale or foreign certificate is refused and the engine runs anyway —
//!   never a wrong verdict, only a slower one,
//! - nothing is auto-discovered: a certificate sitting beside a file does
//!   nothing unless it is named,
//! - a file with errors cannot be certified at all.

use std::fs;

use laterite::ags4;

/// Clean AGS4 — it has to validate error-free or it cannot be certified.
/// Genuinely clean AGS4 — error-free, or it cannot be certified at all.
///
/// Bigger than it looks because "valid" is a real bar: PROJ alone fails Rules
/// 14, 15 and 17 for the missing TRAN, UNIT and TYPE groups, and TRAN itself
/// has six REQUIRED headings that must appear in dictionary order.
const CLEAN: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Certificate test\"\r\n",
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2026-08-05\",\"Producer\",\"Draft\",\"4.1.1\",\"Recipient\"\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"Date\"\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date\"\r\n",
);

/// The same file with one cell changed — same shape, different bytes, so a
/// certificate for `CLEAN` must refuse it.
const CLEAN_EDITED: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Certificate test edited\"\r\n",
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2026-08-05\",\"Producer\",\"Draft\",\"4.1.1\",\"Recipient\"\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"Date\"\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date\"\r\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-cert-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_certificate_lets_validate_skip_the_engine() {
    let dir = scratch("happy");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();

    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let report = ags4::validate(&ags).index(&idx).run().unwrap();
    assert!(report.certified(), "a fresh certificate was not used");
    assert_eq!(report.revalidate_reason(), None);
    assert!(report.is_valid());
}

#[test]
fn without_naming_it_the_certificate_is_ignored() {
    // Auto-discovery is refused by design: an `.ags.idx` beside a file is not
    // consent to trust it. The file below sits right next to its certificate and
    // must still be validated from scratch.
    let dir = scratch("nodiscover");
    let ags = dir.join("d.ags");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(dir.join("d.ags.idx"))
        .unwrap();

    let report = ags4::validate(&ags).run().unwrap();
    assert!(
        !report.certified(),
        "a certificate was picked up without being named"
    );
}

#[test]
fn an_edited_file_refuses_its_old_certificate_and_revalidates() {
    // The core safety property. A certificate that no longer matches must not
    // vouch — and the run must still produce a verdict, not an error.
    let dir = scratch("stale");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    fs::write(&ags, CLEAN_EDITED).unwrap();
    let report = ags4::validate(&ags).index(&idx).run().unwrap();

    assert!(!report.certified(), "a stale certificate was trusted");
    assert!(
        report.revalidate_reason().is_some(),
        "the report does not say why the certificate was refused"
    );
    assert!(report.is_valid(), "the edited file is still valid AGS4");
}

#[test]
fn the_bytes_form_and_the_file_form_are_interchangeable() {
    let dir = scratch("bytesform");
    let ags = dir.join("d.ags");
    fs::write(&ags, CLEAN).unwrap();

    let cert = ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_bytes()
        .unwrap();
    let report = ags4::validate(&ags).index_bytes(cert).run().unwrap();
    assert!(report.certified());
}

#[test]
fn a_certificate_travels_with_the_bytes_not_the_path() {
    // Minted from a path, offered against the same bytes read into memory. The
    // certificate is a statement about content, so this has to hold.
    let dir = scratch("travel");
    let ags = dir.join("d.ags");
    fs::write(&ags, CLEAN).unwrap();
    let cert = ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_bytes()
        .unwrap();

    let report = ags4::validate_bytes(CLEAN.as_bytes().to_vec())
        .index_bytes(cert)
        .run()
        .unwrap();
    assert!(report.certified());
}

#[test]
fn a_certificate_is_over_the_original_bytes_not_the_decoded_ones() {
    // The trap: `read` transcodes before parsing, so the bytes the parser saw
    // are not the bytes on disk. Minting over the decoded form would produce a
    // certificate that never matches its own file — and every ASCII test would
    // still pass, because for ASCII the two forms are identical.
    //
    // A UTF-8 `°` (0xC2 0xB0) read AS cp1252 decodes to "Â°" (0xC3 0x82 0xC2
    // 0xB0). Different bytes, both valid UTF-8, so the mint is reachable either
    // way and only the choice of which bytes to hash decides this test.
    let dir = scratch("encoding");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN.replace("Certificate test", "Dip 45\u{b0}")).unwrap();

    ags4::read(&ags)
        .encoding("windows-1252")
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let report = ags4::validate(&ags)
        .encoding("windows-1252")
        .index(&idx)
        .run()
        .unwrap();
    assert!(
        report.certified(),
        "the certificate did not match the file it was minted from — it was \
         almost certainly minted over the decoded bytes rather than the original \
         (reason: {:?})",
        report.revalidate_reason()
    );
}

#[test]
fn bytes_that_are_not_utf8_cannot_be_certified() {
    // A documented engine limit worth pinning from out here: a certificate
    // carries byte offsets into the source, and the engine refuses to record
    // offsets into bytes it cannot address. So a genuinely cp1252-encoded file
    // is readable and validatable but NOT certifiable — on every surface, since
    // they all mint over the original bytes.
    let dir = scratch("notutf8");
    let ags = dir.join("d.ags");
    let (head, tail) = CLEAN.split_once("Certificate test").unwrap();
    let mut cp1252 = Vec::new();
    cp1252.extend_from_slice(head.as_bytes());
    cp1252.extend_from_slice(b"Dip 45");
    cp1252.push(0xB0); // `°` in cp1252 — a lone high byte, not valid UTF-8
    cp1252.extend_from_slice(tail.as_bytes());
    fs::write(&ags, &cp1252).unwrap();

    // It reads fine, which is what makes the refusal worth documenting.
    let doc = ags4::read(&ags).encoding("windows-1252").run().unwrap();
    let err = doc.certify().to_bytes().unwrap_err();
    assert_eq!(err.kind_str(), "error");
    assert!(
        format!("{err:#}").contains("index"),
        "expected the indexing limit to be named, got: {err:#}"
    );
}

#[test]
fn a_file_with_errors_cannot_be_certified() {
    // A certificate asserts an error-clean validation, so there is nothing to
    // mint here. It must fail loudly rather than produce a certificate that
    // vouches for a broken file.
    let broken = "\"GROUP\",\"NOPE\"\r\n\"HEADING\",\"NOPE_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"x\"\r\n";
    let err = ags4::read_str(broken)
        .run()
        .unwrap()
        .certify()
        .to_bytes()
        .unwrap_err();
    assert_eq!(err.kind_str(), "invalid_argument");
}

#[test]
fn certificate_bytes_that_are_not_a_certificate_are_rejected() {
    let err = ags4::validate_str(CLEAN)
        .index_bytes(b"{\"not\":\"a certificate\"}".to_vec())
        .run()
        .unwrap_err();
    assert_eq!(err.kind_str(), "invalid_argument");
}

#[test]
fn a_missing_certificate_file_is_an_io_error() {
    let dir = scratch("missing");
    let ags = dir.join("d.ags");
    fs::write(&ags, CLEAN).unwrap();
    let err = ags4::validate(&ags)
        .index(dir.join("absent.ags.idx"))
        .run()
        .unwrap_err();
    assert_eq!(err.kind_str(), "io");
}

#[test]
fn a_report_without_a_certificate_says_so() {
    let report = ags4::validate_str(CLEAN).run().unwrap();
    assert!(!report.certified());
    assert_eq!(report.revalidate_reason(), None);
}

// --- the byte index: reading groups without parsing the file ----------------

#[test]
fn a_certificate_lets_read_slice_named_groups() {
    let dir = scratch("slice");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let doc = ags4::read(&ags).index(&idx).only(["PROJ"]).run().unwrap();
    assert!(doc.sliced(), "the byte index was not used");
    assert_eq!(doc.codes(), ["PROJ"]);
    assert_eq!(doc.group("PROJ").unwrap().len(), 1);
}

#[test]
fn slicing_and_parsing_produce_the_same_group() {
    // The optimisation must be invisible in the result. If these ever disagree,
    // the fast path is not reading what the slow path reads.
    let dir = scratch("sameresult");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let sliced = ags4::read(&ags).index(&idx).only(["TRAN"]).run().unwrap();
    let parsed = ags4::read(&ags).only(["TRAN"]).run().unwrap();
    assert!(sliced.sliced() && !parsed.sliced());

    let a = sliced.group("TRAN").unwrap();
    let b = parsed.group("TRAN").unwrap();
    assert_eq!(a.headings(), b.headings());
    assert_eq!(a.units(), b.units());
    assert_eq!(a.types(), b.types());
    assert_eq!(a.len(), b.len());
}

#[test]
fn only_without_a_certificate_still_filters() {
    let doc = ags4::read_str(CLEAN).only(["PROJ", "TRAN"]).run().unwrap();
    assert!(!doc.sliced(), "there was no index to slice with");
    let mut codes = doc.codes();
    codes.sort_unstable();
    assert_eq!(codes, ["PROJ", "TRAN"]);
}

#[test]
fn a_stale_certificate_falls_back_instead_of_slicing_wrong_bytes() {
    // The offsets in a stale certificate point into a file that no longer
    // exists. Reading them would return whatever now sits at those bytes — the
    // one failure mode of a byte index, and the reason freshness is checked
    // before the ranges are touched rather than after.
    let dir = scratch("staleslice");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    fs::write(&ags, CLEAN_EDITED).unwrap();
    let doc = ags4::read(&ags).index(&idx).only(["PROJ"]).run().unwrap();

    assert!(!doc.sliced(), "a stale index was used to slice");
    assert_eq!(doc.codes(), ["PROJ"]);
    // And the content is the CURRENT file's, not the certified one's.
    assert_eq!(
        doc.group("PROJ").unwrap().row(0).unwrap().cell("PROJ_NAME"),
        Some("Certificate test edited")
    );
}

#[test]
fn an_encoding_override_that_changes_the_bytes_declines_the_index() {
    // The index's offsets are into the ORIGINAL bytes. If a transcode moved
    // them, slicing would read from the wrong places — so the guard is byte
    // equality, not "was an encoding named". A UTF-8 `°` read as cp1252 becomes
    // "Â°", which is a different length, so every offset after it shifts.
    let dir = scratch("encslice");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN.replace("Certificate test", "Dip 45\u{b0}")).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let doc = ags4::read(&ags)
        .encoding("windows-1252")
        .index(&idx)
        .only(["PROJ"])
        .run()
        .unwrap();
    assert!(!doc.sliced(), "sliced using offsets into different bytes");
    assert_eq!(doc.codes(), ["PROJ"]);
}

#[test]
fn an_encoding_that_changes_nothing_still_slices() {
    // The other half of the guard being byte equality rather than a flag: this
    // file is ASCII, so decoding it as cp1252 is a genuine no-op and the offsets
    // still describe the bytes being parsed. Declining here would be a needless
    // slow path.
    let dir = scratch("encnoop");
    let ags = dir.join("d.ags");
    let idx = dir.join("d.ags.idx");
    fs::write(&ags, CLEAN).unwrap();
    ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_path(&idx)
        .unwrap();

    let doc = ags4::read(&ags)
        .encoding("windows-1252")
        .index(&idx)
        .only(["PROJ"])
        .run()
        .unwrap();
    assert!(
        doc.sliced(),
        "an identity transcode should not decline the index"
    );
}

#[test]
fn an_index_placing_a_group_in_two_sections_declines_to_slice() {
    // The silent-truncation guard, and it IS reachable from out here: a caller
    // can hand `index_bytes` any certificate, so one whose index puts a group in
    // two sections is a supported input. Slicing the first of them would return
    // a strict SUBSET of that group's rows with no error and no warning — the
    // worst failure a byte index can have, because the result looks complete.
    //
    // The certificate stays FRESH (the hash is over the file, which is
    // untouched), so freshness cannot be what rejects it. Only the ambiguity can.
    let dir = scratch("tworanges");
    let ags = dir.join("d.ags");
    fs::write(&ags, CLEAN).unwrap();
    let cert = ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_bytes()
        .unwrap();

    let mut json: serde_json::Value = serde_json::from_slice(&cert).unwrap();
    let spans = json["groups"]["PROJ"].as_array().unwrap().clone();
    assert_eq!(spans.len(), 1, "PROJ should occupy exactly one section");
    json["groups"]["PROJ"] = serde_json::Value::Array(vec![spans[0].clone(), spans[0].clone()]);
    let doctored = serde_json::to_vec(&json).unwrap();

    let doc = ags4::read(&ags)
        .index_bytes(doctored)
        .only(["PROJ"])
        .run()
        .unwrap();

    assert!(!doc.sliced(), "sliced a group the index could not place");
    // And the fallback still answers the question that was asked.
    assert_eq!(doc.codes(), ["PROJ"]);
    assert_eq!(doc.group("PROJ").unwrap().len(), 1);
}
