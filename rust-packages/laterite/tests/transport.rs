//! `laterite::transport` — the compress/encrypt envelope.
//!
//! These drive the crate the way a consumer does: through the public API only,
//! never `unstable-engine`. What is worth asserting here is not that zstd works
//! — the engine's own suite covers that — but the properties the FACADE
//! promises and could silently break while still compiling:
//!
//! - the file forms and the `_bytes` forms produce the same envelope, so a blob
//!   sealed one way opens the other,
//! - a passphrase never reaches a `Debug` rendering,
//! - a wrong passphrase is an error rather than garbage,
//! - the errors carry the coarse kind the surface documents.

use std::fs;
use std::time::Instant;

use laterite::transport;

/// A deliberately cheap scrypt factor for tests that are not about the factor.
///
/// The real default is 18 and costs about a second per seal by design — that is
/// the point of a work factor. Paying it in every test bought nothing and made
/// this file the slowest in the crate, so only the canonical file round-trip
/// below runs at the shipped default; the rest assert their own property.
const FAST: u8 = 10;

/// A delivery-shaped payload: text, repetitive, the case zstd is chosen for.
fn payload() -> Vec<u8> {
    use std::fmt::Write as _;
    let mut s = String::from("\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n");
    for i in 0..500 {
        let _ = writeln!(s, "\"DATA\",\"BH{i:04}\"\r");
    }
    s.into_bytes()
}

/// A payload whose compressed form actually depends on the level.
///
/// The repetitive [`payload`] above does not: zstd finds all of its redundancy
/// well before level 9, so levels 9 and 10 emit byte-identical output (397 bytes
/// each, measured). A test comparing two forms on that payload therefore passes
/// whether or not they agree on the level — which is exactly what happened here.
/// Mutating `pack_bytes` to use `DEFAULT_LEVEL + 1` left the interop test green.
///
/// Varied cells give zstd something to decide about, so the comparison below can
/// fail when the two forms disagree. Deterministic (fixed-seed LCG) — a flaky
/// interop test would be worse than none.
fn varied_payload() -> Vec<u8> {
    use std::fmt::Write as _;
    let mut s = String::from("\"GROUP\",\"LOCA\"\r\n");
    let mut x: u64 = 12_345;
    for i in 0..2_000u64 {
        x = x
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let _ = writeln!(
            s,
            "\"DATA\",\"BH{:05}\",\"{}\",\"{:.3}\"\r",
            i,
            x % 9_973,
            (x % 100_000) as f64 / 1_000.0
        );
    }
    s.into_bytes()
}

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "laterite-transport-{}-{}",
        std::process::id(),
        name
    ));
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn pack_then_unpack_returns_the_original_bytes() {
    let src = tmp("roundtrip.ags");
    let packed = tmp("roundtrip.ags.zst");
    let back = tmp("roundtrip.out.ags");
    let data = payload();
    fs::write(&src, &data).unwrap();

    let stats = transport::pack(&src, &packed).run().unwrap();
    assert!(stats.bytes() > 0, "packed file is empty");
    assert!(
        stats.ratio() < 1.0,
        "repetitive text should compress, got ratio {}",
        stats.ratio()
    );

    let opened = transport::unpack(&packed, &back).unwrap();
    assert_eq!(usize::try_from(opened.bytes()).unwrap(), data.len());
    assert_eq!(fs::read(&back).unwrap(), data);
}

#[test]
fn the_reported_statistics_are_the_real_ones() {
    // What each accessor IS, not merely that it is plausible. The round-trip
    // above proves the envelope works and asserts only `bytes() > 0` and
    // `ratio() < 1.0`, both of which a constant satisfies: `bytes -> 1` and
    // `ratio -> 0.0` survived a mutation sweep, and so did `ratio -> -1.0`
    // (#273). `ratio` is the one that matters most to a caller — it is the
    // number someone decides whether compression was worth it by.
    let src = tmp("stats.ags");
    let packed = tmp("stats.ags.zst");
    let back = tmp("stats.out.ags");
    let data = payload();
    fs::write(&src, &data).unwrap();

    let clock = Instant::now();
    let stats = transport::pack(&src, &packed).run().unwrap();
    let pack_wall = clock.elapsed().as_secs_f64();
    let on_disk = fs::metadata(&packed).unwrap().len();

    assert_eq!(stats.bytes(), on_disk, "bytes() is not the file it wrote");
    let expected = on_disk as f64 / data.len() as f64;
    assert!(
        (stats.ratio() - expected).abs() < 1e-9,
        "ratio() {} is not output/input ({expected})",
        stats.ratio()
    );
    // The payload is repetitive enough that the ratio is far below 1, so this
    // could not pass on a payload the envelope failed to compress.
    assert!(
        stats.ratio() < 0.5,
        "ratio {} is not a real one",
        stats.ratio()
    );

    // Elapsed gets an INTERVAL, not a property. "Positive and finite" was the
    // first attempt and the sweep walked straight through it — `elapsed_secs`
    // replaced by the constant `1.0` satisfies both, and survived. Timing it
    // from outside gives a real upper bound: the work cannot have taken longer
    // than the call did. That is exact on any machine, so it cannot go flaky
    // the way a hand-picked ceiling would, and it leaves no constant standing.
    assert!(
        stats.elapsed_secs() > 0.0 && stats.elapsed_secs() <= pack_wall,
        "pack claims {}s, but the call itself took {pack_wall}s",
        stats.elapsed_secs()
    );

    let clock = Instant::now();
    let opened = transport::unpack(&packed, &back).unwrap();
    let unpack_wall = clock.elapsed().as_secs_f64();
    assert_eq!(
        opened.bytes(),
        fs::metadata(&back).unwrap().len(),
        "Unpacked::bytes() is not the file it wrote"
    );
    // `Unpacked::elapsed_secs` is its own function, and survived `1.0` for its
    // own reasons; it needs the bound in its own right.
    assert!(
        opened.elapsed_secs() > 0.0 && opened.elapsed_secs() <= unpack_wall,
        "unpack claims {}s, but the call itself took {unpack_wall}s",
        opened.elapsed_secs()
    );
}

#[test]
fn pack_bytes_and_pack_agree_so_either_can_open_the_other() {
    // The interop claim in the module docs. If these diverge, a service that
    // seals in memory produces files its own CLI cannot open — and every
    // individual round-trip test would still pass.
    let src = tmp("agree.ags");
    let packed = tmp("agree.ags.zst");
    let data = varied_payload();
    fs::write(&src, &data).unwrap();

    transport::pack(&src, &packed).run().unwrap();
    let in_memory = transport::pack_bytes(data.clone()).run().unwrap();

    assert_eq!(
        fs::read(&packed).unwrap(),
        in_memory,
        "the file form and the bytes form produced different envelopes"
    );
    assert_eq!(transport::unpack_bytes(&in_memory).unwrap(), data);
}

#[test]
fn lock_then_unlock_returns_the_original_bytes() {
    let src = tmp("sealed.ags");
    let sealed = tmp("sealed.ags.zst.age");
    let back = tmp("sealed.out.ags");
    let data = payload();
    fs::write(&src, &data).unwrap();

    let clock = Instant::now();
    let stats = transport::lock(&src, &sealed, "hunter2").run().unwrap();
    let wall = clock.elapsed().as_secs_f64();
    let opened = transport::unlock(&sealed, &back, "hunter2").unwrap();

    assert_eq!(usize::try_from(opened.bytes()).unwrap(), data.len());
    assert_eq!(fs::read(&back).unwrap(), data);

    // The sealing path reports too. Asserted inside this test rather than in one
    // of its own because it runs at `transport::DEFAULT_WORK_FACTOR` — scrypt at
    // the shipped factor is deliberately expensive, and there is no reason to
    // pay for it twice.
    assert_eq!(stats.bytes(), fs::metadata(&sealed).unwrap().len());
    assert!(
        stats.elapsed_secs() > 0.0 && stats.elapsed_secs() <= wall,
        "lock claims {}s, but the call itself took {wall}s",
        stats.elapsed_secs()
    );
}

#[test]
fn a_sealed_file_opens_from_bytes_and_the_reverse() {
    // Cross-form, not just same-form: seal to a file, open in memory; seal in
    // memory, open from the file. Encryption is nondeterministic (a fresh salt
    // per seal), so unlike `pack` these cannot be compared byte-for-byte — the
    // property is that each opens the other.
    let src = tmp("cross.ags");
    let sealed = tmp("cross.ags.zst.age");
    let data = payload();
    fs::write(&src, &data).unwrap();

    transport::lock(&src, &sealed, "pw")
        .work_factor(FAST)
        .run()
        .unwrap();
    let from_file = fs::read(&sealed).unwrap();
    assert_eq!(transport::unlock_bytes(&from_file, "pw").unwrap(), data);

    let in_memory = transport::lock_bytes(data.clone(), "pw")
        .work_factor(FAST)
        .run()
        .unwrap();
    let sealed2 = tmp("cross2.ags.zst.age");
    let back = tmp("cross2.out.ags");
    fs::write(&sealed2, &in_memory).unwrap();
    transport::unlock(&sealed2, &back, "pw").unwrap();
    assert_eq!(fs::read(&back).unwrap(), data);
}

#[test]
fn the_wrong_passphrase_fails_rather_than_returning_garbage() {
    let sealed = transport::lock_bytes(payload(), "right")
        .work_factor(FAST)
        .run()
        .unwrap();
    let err = transport::unlock_bytes(&sealed, "wrong").unwrap_err();
    assert_eq!(err.kind_str(), "error");
}

#[test]
fn a_missing_source_is_an_io_error() {
    let err = transport::pack(tmp("nope").join("missing.ags"), tmp("out.zst"))
        .run()
        .unwrap_err();
    assert_eq!(err.kind_str(), "io");
    assert_eq!(err.exit_code(), 2);
}

#[test]
fn plain_bytes_are_not_a_zstd_frame() {
    let err = transport::unpack_bytes(b"not compressed at all").unwrap_err();
    assert_eq!(err.kind_str(), "error");
}

#[test]
fn the_level_knob_changes_the_output() {
    let data = payload();
    let low = transport::pack_bytes(data.clone()).level(1).run().unwrap();
    let high = transport::pack_bytes(data.clone()).level(19).run().unwrap();
    assert_ne!(low, high, "level had no effect on the output");
    // Both must still open, which is the part a caller relies on.
    assert_eq!(transport::unpack_bytes(&low).unwrap(), data);
    assert_eq!(transport::unpack_bytes(&high).unwrap(), data);
}

#[test]
fn a_passphrase_never_appears_in_a_debug_rendering() {
    // The failure this prevents is a leaked credential in a log line, which no
    // round-trip test would ever notice. Both password-carrying builders, and
    // both spellings a caller might reach for.
    let secret = "correct-horse-battery-staple";

    let file_form = transport::lock("in.ags", "out.age", secret);
    let bytes_form = transport::lock_bytes(vec![1, 2, 3], secret);

    for rendered in [
        format!("{file_form:?}"),
        format!("{file_form:#?}"),
        format!("{bytes_form:?}"),
        format!("{bytes_form:#?}"),
    ] {
        assert!(
            !rendered.contains(secret),
            "the passphrase leaked into Debug output: {rendered}"
        );
        assert!(
            rendered.contains("<redacted>"),
            "expected the redaction marker, got: {rendered}"
        );
    }
}

#[test]
fn a_debug_rendering_shows_what_it_is_supposed_to_show() {
    // The four `Debug` impls the test above does NOT reach. It covers `Lock` and
    // `LockBytes`, and covers them properly — asserting the redaction MARKER is
    // present, not merely that the passphrase is absent, so a stub returning an
    // empty rendering fails it. These four carry no passphrase, so they were
    // never in its scope, and nothing else had ever read them: all four survived
    // a sweep replaced with `Ok(Default::default())` (#273).
    //
    // Nothing leaks here — that is the point. What is worth asserting about an
    // impl with no secret in it is that it is INFORMATIVE: the struct name it
    // claims to be, and the fields someone reading a log line needs.
    let pack = transport::pack("in.ags", "out.zst");
    let pack_bytes = transport::pack_bytes(vec![1, 2, 3]);

    let rendered = format!("{pack:?}");
    assert!(rendered.starts_with("Pack {"), "got: {rendered}");
    assert!(rendered.contains("in.ags"), "got: {rendered}");
    assert!(rendered.contains("out.zst"), "got: {rendered}");
    assert!(rendered.contains("level"), "got: {rendered}");

    let rendered = format!("{pack_bytes:?}");
    assert!(rendered.starts_with("PackBytes {"), "got: {rendered}");
    // The LENGTH, never the payload — a delivery in a log line is its own
    // problem, separate from the passphrase one.
    assert!(rendered.contains("bytes: 3"), "got: {rendered}");
    assert!(rendered.contains("level"), "got: {rendered}");

    // The two result types, which carry no secret and are therefore only ever
    // checked for being informative at all.
    let src = tmp("debug.ags");
    let packed = tmp("debug.ags.zst");
    let back = tmp("debug.out.ags");
    fs::write(&src, payload()).unwrap();
    let stats = transport::pack(&src, &packed).run().unwrap();
    let opened = transport::unpack(&packed, &back).unwrap();

    let rendered = format!("{stats:?}");
    assert!(rendered.starts_with("Packed {"), "got: {rendered}");
    for field in ["bytes", "ratio", "elapsed_s"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }

    let rendered = format!("{opened:?}");
    assert!(rendered.starts_with("Unpacked {"), "got: {rendered}");
    for field in ["bytes", "elapsed_s"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
}

#[test]
fn the_work_factor_is_configurable_and_still_opens() {
    // Lower than the default purely to keep the test fast; the assertion is
    // that a non-default factor round-trips, since the envelope records it.
    let data = b"secret payload".to_vec();
    let sealed = transport::lock_bytes(data.clone(), "pw")
        .work_factor(FAST)
        .run()
        .unwrap();
    assert_eq!(transport::unlock_bytes(&sealed, "pw").unwrap(), data);
}

#[test]
fn defaults_are_the_documented_ones() {
    // These two constants are load-bearing for cross-surface interop: every
    // laterite surface writes the same level and the same work factor, so a
    // file sealed by one is the same size and opens on the others.
    assert_eq!(transport::DEFAULT_LEVEL, 9);
    assert_eq!(transport::DEFAULT_WORK_FACTOR, 18);
}
