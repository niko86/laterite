//! The trust rule, and the four false cleans it retires.
//!
//! Each test in the first block corresponds to a way the old model would have reported
//! a file clean when it was not. They are written as "the certificate is offered, and
//! the engine runs anyway" — because the whole content of the fix is that the fast path
//! is no longer taken.

use super::*;
use laterite_ags4_validator::findings::Severity;

/// Content-clean at 4.2, and it declares one FILE attachment — so Rule 20's CONTENT
/// half is satisfied (FS1 *is* defined in the FILE group) and its WORLD half has
/// something left to say: is `FILE/FS1/photo.jpg` really beside the .ags?
const CLEAN_WITH_ATTACHMENT: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Clean minimal AGS4 fixture\"\r\n\r\n",
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2020-08-18\",\"ACME Drilling Ltd\",\"Draft\",\"4.2\",\"ACME Consulting\",\"|\",\"+\"\r\n\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"year month day\"\r\n\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date and time\"\r\n\r\n",
    "\"GROUP\",\"FILE\"\r\n",
    "\"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"FS1\",\"photo.jpg\"\r\n",
);

/// Error-clean, but it declares `TRAN_AGS` 4.9 — an edition nobody bundles. The engine
/// falls back to 4.1.1 (O-30) and says so as a WARNING. Exactly the shape a certificate
/// has to be honest about: nothing wrong with the file, but something to say about it.
const CLEAN_WITH_A_WARNING: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Clean minimal AGS4 fixture\"\r\n\r\n",
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2020-08-18\",\"ACME Drilling Ltd\",\"Draft\",\"4.9\",\"ACME Consulting\",\"|\",\"+\"\r\n\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"year month day\"\r\n\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date and time\"\r\n",
);

/// The same shape of file, error-clean, but with the Greek capital omega in `PROJ_NAME`.
///
/// Its UTF-8 bytes are `CE A9`. Read as UTF-8 that is ONE code point, 937 — above the
/// extended-ASCII range Rule 1 tolerates, so it is a Rule 1 **ERROR**. Read as
/// windows-1252 the very same two bytes are TWO code points, 206 and 169 — both inside
/// that range, so it is only an **FYI**. One file, two decoders, two different verdicts
/// about errors.
const OMEGA_IN_PROJ_NAME: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"\u{3a9} site\"\r\n\r\n",
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2020-08-18\",\"ACME Drilling Ltd\",\"Draft\",\"4.2\",\"ACME Consulting\",\"|\",\"+\"\r\n\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"year month day\"\r\n\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date and time\"\r\n",
);

const AT: &str = "2026-01-01T00:00:00Z";
const RULE_20: &str = "AGS Format Rule 20";
const RULE_1: &str = "AGS Format Rule 1";

/// `CheckOptions` reading the bytes through a named decoder.
fn read_as(label: &str) -> CheckOptions {
    CheckOptions {
        encoding: laterite_ags4_parse::resolve_encoding(Some(label)).expect("a known label"),
        ..CheckOptions::default()
    }
}

fn errors_only() -> CheckOptions {
    CheckOptions::default()
}

fn with_tiers(warnings: bool, fyi: bool) -> CheckOptions {
    CheckOptions {
        include_warnings: warnings,
        include_fyi: fyi,
        ..CheckOptions::default()
    }
}

/// A file, its cert, and a temp dir to put a `FILE/` tree in.
fn minted(text: &str) -> (Vec<u8>, Sidecar) {
    let bytes = text.as_bytes().to_vec();
    let cert = mint(&bytes, &errors_only(), AT.to_string(), None).expect("mints");
    (bytes, cert)
}

fn count(f: &Findings) -> usize {
    f.values().map(Vec::len).sum()
}

// --- the mint tells the truth ----------------------------------------------

#[test]
fn the_mint_measures_every_tier_rather_than_asserting_zero() {
    // THE BUG. laterite-py's cert factory took `warnings=0, fyi=0` as DEFAULT
    // ARGUMENTS and nothing ever passed them, so every cert it minted claimed to have
    // measured zero warnings without having looked. `mint` takes no counts at all —
    // there is no parameter through which a caller could assert that again.
    let (_bytes, cert) = minted(CLEAN_WITH_ATTACHMENT);
    let v = &cert.validation;

    assert_eq!(v.errors, TierCoverage::Measured { count: 0 });
    // Measured, whatever the answer — never NotMeasured, never an unlooked-at zero.
    assert!(
        matches!(v.warnings, TierCoverage::Measured { .. }),
        "the mint runs both tiers: {:?}",
        v.warnings
    );
    assert!(
        matches!(v.fyi, TierCoverage::Measured { .. }),
        "{:?}",
        v.fyi
    );
}

#[test]
fn the_mint_stamps_the_engine_that_produced_the_verdict() {
    // v1 stamped CARGO_PKG_VERSION — a hand-bumped semver that does not move when a
    // rule does, so a cert outlived the engine that made it and still looked current.
    let (_bytes, cert) = minted(CLEAN_WITH_ATTACHMENT);
    assert_eq!(cert.validation.validator, ENGINE_IDENTITY);
    assert_eq!(
        cert.validation.engine,
        laterite_ags4_validator::ENGINE_FINGERPRINT
    );
    assert_ne!(
        cert.validation.engine,
        laterite_ags4_validator::VERSION,
        "the engine's identity is not the crate's semver"
    );
}

#[test]
fn the_mint_refuses_a_file_with_errors_and_cannot_be_told_otherwise() {
    // Not "the caller says it's clean" — the mint runs the engine and looks.
    let bad = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\"UNIT\",\"\"\r\n\
               \"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n";
    let err = mint(bad.as_bytes(), &errors_only(), AT.to_string(), None)
        .expect_err("a file with errors must not be certifiable");
    assert!(err.to_string().contains("cannot certify"), "{err}");
}

#[test]
fn the_mint_still_rejects_a_non_utf8_file() {
    // #5 made `mint` reuse its validating parse for the byte index instead of
    // walking the file a second time. That must NOT open a door for a non-UTF-8
    // file: `assemble_from_parsed` falls back to the lean/`Reject` walk when the
    // parse is not source-true, and a lossy-replaced byte is a Rule 1 error
    // regardless — either way the file is rejected, exactly as before. (Closes the
    // coverage gap the campaign flagged: nothing minted a non-UTF-8 file.)
    let bad: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                       \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH\xff1\"\r\n";
    mint(bad, &errors_only(), AT.to_string(), None)
        .expect_err("a non-UTF-8 file must not be certifiable");
}

#[test]
fn the_mint_records_a_warning_instead_of_refusing_it() {
    // A cert asserts ERROR-cleanliness, not perfection. A file may legitimately carry
    // warnings and still be a valid delivery — so they are recorded, not fatal. (The
    // uvx and npx `certify` shims used to refuse such a file while the native binary
    // minted it: three launchers, two behaviours, for the same bytes.)
    let bytes = CLEAN_WITH_A_WARNING.as_bytes();
    let both = with_tiers(true, true);
    let parsed = parse_bytes(bytes, both.encoding).expect("parses");
    let Ok((found, _, _)) = check_parsed_with_dict(&parsed, &both, &WorldScope::None) else {
        panic!("fixture must validate")
    };
    let errs = count_of(&found, Severity::Error);
    let warns = count_of(&found, Severity::Warning);
    assert_eq!(errs, 0, "fixture must be error-clean: {found:?}");
    assert!(warns > 0, "fixture must carry a warning: {found:?}");

    let cert = mint(bytes, &errors_only(), AT.to_string(), None)
        .expect("error-clean file with a warning is certifiable");
    assert_eq!(cert.validation.errors, TierCoverage::Measured { count: 0 });
    assert_eq!(
        cert.validation.warnings,
        TierCoverage::Measured { count: warns },
        "the warning is RECORDED, honestly, not suppressed"
    );
}

// --- the trust rule ---------------------------------------------------------

#[test]
fn a_fresh_cert_vouches_for_the_question_it_measured() {
    let (bytes, cert) = minted(CLEAN_WITH_ATTACHMENT);
    let opts = errors_only();
    let out = check(Request {
        bytes: &bytes,
        opts: &opts,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");

    assert!(out.certified, "a fresh, matching cert must be trusted");
    assert_eq!(count(&out.findings), 0);
    assert_eq!(out.dict_version.as_str(), "4.2");
    assert!(out.revalidate_reason.is_none());
}

#[test]
fn a_cert_that_measured_a_tier_and_found_it_dirty_cannot_answer_for_it() {
    // The subtle half of the trust rule. The cert stores COUNTS, not findings: if it
    // measured 1 warning it knows there is something to say but not what. Asking for
    // warnings must therefore run the engine — the cert cannot produce the text.
    let bytes = CLEAN_WITH_A_WARNING.as_bytes();
    let cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");

    // Errors only → the cert answers (errors were measured, and clean).
    let errs = errors_only();
    let out = check(Request {
        bytes,
        opts: &errs,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(out.certified);
    assert_eq!(count(&out.findings), 0);

    // Ask for warnings → it measured them, and they were NOT clean. Engine.
    let warn = with_tiers(true, false);
    let out = check(Request {
        bytes,
        opts: &warn,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(
        !out.certified,
        "a dirty tier cannot be answered from a count"
    );
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::TierNotClean(Tier::Warnings))
    );
    assert!(
        count(&out.findings) > 0,
        "and the engine actually produced the warning the cert could only count"
    );
}

#[test]
fn an_unmeasured_tier_is_not_a_clean_one() {
    // The state v1 could not even represent. Hand-build a cert whose warning tier was
    // never run — the old format would have written `warnings: 0` and a
    // `--show-warnings` request would have read that zero and skipped the engine.
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let id = engine_id(None);
    let stamp = ValidationStamp {
        validator: id.validator,
        engine: id.fingerprint,
        compat: None,
        checked_at: AT.to_string(),
        edition: EditionInput::Auto {
            resolved: "4.2".to_string(),
            resolution: laterite_ags4_validator::DictResolution::ExactTranAgs,
        },
        encoding: "UTF-8".to_string(),
        custom_dict: None,
        errors: TierCoverage::Measured { count: 0 },
        warnings: TierCoverage::NotMeasured,
        fyi: TierCoverage::NotMeasured,
    };
    let cert = Sidecar::assemble(bytes, stamp).expect("assembles");

    // Errors only → fine, that tier WAS measured.
    let errs = errors_only();
    assert!(
        check(Request {
            bytes,
            opts: &errs,
            cert: Some(&cert),
            world: WorldScope::None,
            compat: None,
        })
        .expect("checks")
        .certified
    );

    // Ask about a tier it never ran → it must say so, not answer zero.
    let warn = with_tiers(true, false);
    let out = check(Request {
        bytes,
        opts: &warn,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(!out.certified);
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::TierNotMeasured(Tier::Warnings))
    );
}

#[test]
fn a_cert_from_a_different_engine_is_not_trusted() {
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let mut cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");
    cert.validation.engine = "0000000000000000".to_string(); // a rule changed under it

    let opts = errors_only();
    let out = check(Request {
        bytes,
        opts: &opts,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(!out.certified);
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::DifferentEngine)
    );
}

#[test]
fn changed_bytes_are_not_the_certified_bytes() {
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");

    // Same length, different content — the size gate passes and the SHA catches it.
    let mut tampered = bytes.to_vec();
    let i = tampered.len() - 6;
    tampered[i] = b'X';
    assert_eq!(tampered.len(), bytes.len());

    let opts = errors_only();
    let out = check(Request {
        bytes: &tampered,
        opts: &opts,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(!out.certified);
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::ContentChanged)
    );
}

#[test]
fn a_forced_edition_and_an_auto_one_do_not_answer_for_each_other() {
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let auto_cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");
    assert!(matches!(
        auto_cert.validation.edition,
        EditionInput::Auto { .. }
    ));

    // The file's TRAN_AGS says 4.2 and the cert auto-resolved to 4.2 — but a request
    // that FORCES 4.2 is a different question: forcing means "ignore TRAN_AGS", and on
    // a file whose declared edition disagreed with its content the two runs would
    // apply different dictionaries. The old `profile_covers` compared the edition
    // string and the forced flag separately and got this wrong.
    let forced = CheckOptions {
        dict_version: Some(DictVersion::V4_2),
        ..CheckOptions::default()
    };
    let out = check(Request {
        bytes,
        opts: &forced,
        cert: Some(&auto_cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("checks");
    assert!(!out.certified);
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::EditionDiffers)
    );
}

#[test]
fn a_compat_cert_does_not_answer_for_the_native_engine() {
    // The compat shim mimics python-ags4, which disagrees with the native engine on
    // real files. Neither verdict may stand in for the other.
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let compat_cert = mint(
        bytes,
        &errors_only(),
        AT.to_string(),
        Some("1.2.0".to_string()),
    )
    .expect("mints");

    let opts = errors_only();
    let out = check(Request {
        bytes,
        opts: &opts,
        cert: Some(&compat_cert),
        world: WorldScope::None,
        compat: None, // asking as the native engine
    })
    .expect("checks");
    assert!(!out.certified);
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::DifferentValidator)
    );
}

// --- the world is never vouched for ----------------------------------------

#[test]
fn a_vouched_cert_does_not_stop_the_world_check_from_running() {
    // THE BUG THIS WHOLE ARC IS FOR. Certify a file with --check-files, delete the
    // FILE/ tree, re-validate with --check-files and a fresh cert. The .ags bytes are
    // byte-identical, the SHA matches, the cert is perfectly valid — and the old model
    // skipped the whole run and reported clean, exit 0, where the truth is one finding.
    //
    // Now: the cert vouches for the CONTENT (correctly — the content IS clean), and the
    // world check runs anyway, because it sits outside the branch the cert can
    // short-circuit.
    let dir = tempfile::tempdir().expect("tempdir");
    let ags = dir.path().join("delivery.ags");
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    std::fs::write(&ags, bytes).expect("write");

    // The attachment exists → --check-files is clean.
    let leaf = dir.path().join("FILE").join("FS1");
    std::fs::create_dir_all(&leaf).expect("mkdir");
    std::fs::write(leaf.join("photo.jpg"), b"x").expect("write");

    let cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");
    let cf = CheckOptions {
        check_files: true,
        ..CheckOptions::default()
    };

    let out = check(Request {
        bytes,
        opts: &cf,
        cert: Some(&cert),
        world: WorldScope::OnDisk(ags.clone()),
        compat: None,
    })
    .expect("checks");
    assert!(out.certified, "the content is certified…");
    assert_eq!(count(&out.findings), 0, "…and the world is currently fine");

    // Now delete the tree. Not one byte of the .ags changes.
    std::fs::remove_dir_all(dir.path().join("FILE")).expect("rm");

    let out = check(Request {
        bytes,
        opts: &cf,
        cert: Some(&cert),
        world: WorldScope::OnDisk(ags),
        compat: None,
    })
    .expect("checks");
    assert!(
        out.certified,
        "the cert still vouches for the CONTENT — it is still true"
    );
    assert!(
        out.findings.contains_key(RULE_20),
        "but the world is checked LIVE and the missing tree is found: {:?}",
        out.findings
    );
    assert_eq!(
        count(&out.findings),
        1,
        "exactly the world finding, no content re-run"
    );
}

#[test]
fn a_world_check_with_no_world_is_refused_not_answered() {
    // Every bytes/text caller, and the browser always. `WorldScope::None` + check_files
    // is a question with nothing to ask it of.
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let cert = mint(bytes, &errors_only(), AT.to_string(), None).expect("mints");
    let cf = CheckOptions {
        check_files: true,
        ..CheckOptions::default()
    };
    let err = check(Request {
        bytes,
        opts: &cf,
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect_err("must refuse");
    assert!(matches!(err, ValidatorError::WorldCheckRequiresSource));
}

#[test]
fn the_mint_never_records_a_world_claim_even_when_one_was_asked_for() {
    // There is no `world` parameter on `mint`, and `check_files` is forced off inside
    // it — but assert the OUTPUT: a stamp built from an options struct that had
    // check_files set carries no trace of it, because the struct has nowhere to put it.
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let cf = CheckOptions {
        check_files: true,
        ..CheckOptions::default()
    };
    let cert = mint(bytes, &cf, AT.to_string(), None).expect("mints");

    let json = String::from_utf8(cert.to_json().expect("serialises")).expect("utf8");
    assert!(
        !json.contains("check_files"),
        "the format has no field in which to record a world claim:\n{json}"
    );
    assert!(!json.contains("FILE/"), "{json}");
}

// --- no cert at all ---------------------------------------------------------

#[test]
fn with_no_cert_the_engine_runs_and_the_world_is_still_checked() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ags = dir.path().join("delivery.ags");
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    std::fs::write(&ags, bytes).expect("write"); // no FILE/ tree beside it

    let cf = CheckOptions {
        check_files: true,
        ..CheckOptions::default()
    };
    let out = check(Request {
        bytes,
        opts: &cf,
        cert: None,
        world: WorldScope::OnDisk(ags),
        compat: None,
    })
    .expect("checks");

    assert!(!out.certified);
    assert!(out.revalidate_reason.is_none(), "no cert was offered");
    assert!(out.findings.contains_key(RULE_20), "{:?}", out.findings);
}

#[test]
fn a_world_we_were_handed_but_not_asked_to_look_at_is_not_looked_at() {
    // `check_files` off + a path available: the on-disk half must NOT run. Callers pass
    // a path for lots of reasons; the path is not the request.
    let dir = tempfile::tempdir().expect("tempdir");
    let ags = dir.path().join("delivery.ags");
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    std::fs::write(&ags, bytes).expect("write"); // no FILE/ tree

    let opts = errors_only(); // check_files: false
    let out = check(Request {
        bytes,
        opts: &opts,
        cert: None,
        world: WorldScope::OnDisk(ags),
        compat: None,
    })
    .expect("checks");
    assert_eq!(
        count(&out.findings),
        0,
        "path-independent by default (O-27): {:?}",
        out.findings
    );
}

#[test]
fn an_external_dict_is_honoured_not_refused() {
    // #568 reversed the O-28 deferral: a valid `--dict` overlay now RUNS (layered
    // over its detected base) rather than erroring BadDict at the door. The overlay
    // adds a group the file doesn't use, so the verdict is unchanged from a bundled
    // run — the point is that it is honoured, not refused.
    let bytes = CLEAN_WITH_ATTACHMENT.as_bytes();
    let dict_json = br#"{"groups":{"XTRA":{"parent":"SAMP","headings":[
        {"name":"SAMP_ID","type":"ID","status":"KEY"},
        {"name":"XTRA_VAL","type":"2DP","status":"REQUIRED"}
    ]}}}"#;
    let custom = laterite_ags4_validator::overlay::parse_dict(
        dict_json,
        laterite_ags4_validator::overlay::DictFormat::Json,
        CheckOptions::default().encoding,
        laterite_ags4_validator::overlay::BaseSpec::Auto,
        "mine.json",
    )
    .expect("custom dict parses");
    let opts = CheckOptions {
        custom_dict: Some(custom),
        ..CheckOptions::default()
    };
    check(Request {
        bytes,
        opts: &opts,
        cert: None,
        world: WorldScope::None,
        compat: None,
    })
    .expect("a valid custom dict is honoured, not refused");
}

// --- the decoder is part of the question ------------------------------------

/// The bytes are sealed by SHA-256. The DECODER is not part of them — and the rules
/// judge the TEXT a decoder produces, not the bytes themselves.
///
/// This is the fifth false clean, and the one the CONTENT/WORLD partition did not by
/// itself close. `encoding` IS content (the text is a pure function of the bytes and the
/// label), so it belongs on the fast path — but the certificate did not record WHICH
/// decoder produced its verdict, so a cert minted under a lenient one answered a request
/// made under a strict one. Probed on the built wheel before this gate existed: the file
/// below validated as 1 ERROR under UTF-8, certified error-clean under windows-1252, and
/// then read back with that cert as `count = 0, certified = true, is_valid = true`.
#[test]
fn a_cert_minted_through_another_decoder_does_not_answer() {
    let bytes = OMEGA_IN_PROJ_NAME.as_bytes().to_vec();

    // The premise, asserted rather than assumed: same bytes, two decoders, two verdicts —
    // differing in the ERROR tier, which is the one a certificate asserts.
    let utf8 = check(Request {
        bytes: &bytes,
        opts: &read_as("utf-8"),
        cert: None,
        world: WorldScope::None,
        compat: None,
    })
    .expect("validates");
    assert_eq!(
        utf8.findings.get(RULE_1).map(Vec::len),
        Some(1),
        "read as UTF-8 the omega is one code point (937) — a Rule 1 error"
    );

    // So it can be certified under the lenient decoder: cp1252 sees only extended ASCII.
    let cert = mint(&bytes, &read_as("windows-1252"), AT.to_string(), None)
        .expect("error-clean under windows-1252, so it mints");
    assert_eq!(cert.validation.encoding, "windows-1252");
    assert_eq!(cert.validation.errors, TierCoverage::Measured { count: 0 });

    // And now the question that used to come back a lie: the SAME bytes, read as UTF-8,
    // offering that certificate.
    let out = check(Request {
        bytes: &bytes,
        opts: &read_as("utf-8"),
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("validates");

    assert!(
        !out.certified,
        "a windows-1252 verdict cannot answer a UTF-8 question"
    );
    assert_eq!(
        out.revalidate_reason,
        Some(RevalidateReason::EncodingDiffers)
    );
    assert_eq!(
        out.findings.get(RULE_1).map(Vec::len),
        Some(1),
        "and because the engine ran, the error is reported"
    );

    // The decoder it WAS minted under still answers — this is a match, not a ban.
    let same = check(Request {
        bytes: &bytes,
        opts: &read_as("windows-1252"),
        cert: Some(&cert),
        world: WorldScope::None,
        compat: None,
    })
    .expect("validates");
    assert!(same.certified, "same bytes, same decoder, same question");
}
