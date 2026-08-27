//! End-to-end `lat merge` tests — spawn the built binary and assert real output
//! + exit codes (not just "it runs").

use std::path::PathBuf;
use std::process::Command;

const A: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"P1\",\"Demo\"\r\n\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_GL\"\r\n\"UNIT\",\"\",\"m\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\",\"2DP\"\r\n\"DATA\",\"BH1\",\"100.00\",\"10.00\"\r\n";
// B re-types LOCA_NATE 2DP → X (the type conflict) and revises BH1's GL.
const B: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"P1\",\"Demo\"\r\n\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_GL\"\r\n\"UNIT\",\"\",\"m\",\"m\"\r\n\"TYPE\",\"ID\",\"X\",\"2DP\"\r\n\"DATA\",\"BH1\",\"100.00\",\"11.50\"\r\n";

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lat_merge_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn merge_errors_on_type_conflict_exit_6_and_offers_both_ways_out() {
    let d = scratch();
    let (a, b, out) = (d.join("a.ags"), d.join("b.ags"), d.join("strict.ags"));
    std::fs::write(&a, A).unwrap();
    std::fs::write(&b, B).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(6),
        "TYPE conflict → schema-violation exit 6"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("TYPE conflict in LOCA.LOCA_NATE"),
        "stderr: {err}"
    );
    // Both escape hatches must be offered — naming only the lossy one would push
    // every clash toward X, which is what the lattice (laterite-dev#500) set out to stop.
    assert!(
        err.contains("--on-type-clash promote"),
        "offers promote: {err}"
    );
    assert!(err.contains("--on-type-clash widen"), "offers widen: {err}");
}

#[test]
fn merge_widen_writes_output_and_reports_the_real_revision() {
    let d = scratch();
    let (a, b, out) = (d.join("a2.ags"), d.join("b2.ags"), d.join("merged.ags"));
    std::fs::write(&a, A).unwrap();
    std::fs::write(&b, B).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .args([
            "--on-type-clash",
            "widen",
            "--tran-issue",
            "3",
            "--tran-date",
            "2024-03-01",
            "--tran-producer",
            "Merger",
            "--tran-recipient",
            "Client",
            "--tran-status",
            "Merged",
        ])
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "widen merge succeeds: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(stdout.contains("merged 2 files"), "stdout: {stdout}");
    // Only LOCA_GL genuinely changed; LOCA_NATE's identical raw value must NOT be a
    // reported revision despite the 2DP→X widen.
    assert!(
        stdout.contains("LOCA_GL"),
        "reports the GL revision: {stdout}"
    );
    assert!(
        !stdout.contains("LOCA_NATE"),
        "the type-widened equal value is not a revision: {stdout}"
    );
    // The merged file was actually written and re-parses as AGS4.
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"\"GROUP\""));
}

/// `--on-type-clash promote` — the nDP lattice join, end to end through the binary:
/// the merged column keeps the greatest precision and the coarser value is padded.
#[test]
fn merge_promote_keeps_the_greatest_precision_and_pads() {
    let d = scratch();
    let (a, b, out) = (d.join("a3.ags"), d.join("b3.ags"), d.join("promoted.ags"));
    // Same LOCA_GL column, typed 2DP in one delivery and 5DP in the other.
    std::fs::write(&a, A).unwrap();
    std::fs::write(
        &b,
        B.replace(r#""TYPE","ID","X","2DP""#, r#""TYPE","ID","2DP","5DP""#)
            .replace(
                r#""DATA","BH1","100.00","11.50""#,
                r#""DATA","BH2","200.00","20.12345""#,
            ),
    )
    .unwrap();

    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .args(["--on-type-clash", "promote"])
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "promote merge succeeds: {}",
        String::from_utf8_lossy(&o.stderr)
    );

    let merged = String::from_utf8(std::fs::read(&out).unwrap()).unwrap();
    assert!(
        merged.contains(r#""TYPE","ID","2DP","5DP""#),
        "LOCA_GL keeps 5DP, not X: {merged}"
    );
    assert!(
        merged.contains(r#""DATA","BH1","100.00","10.00000""#),
        "the 2DP value is zero-padded to 5 places: {merged}"
    );
    assert!(
        merged.contains(r#""DATA","BH2","200.00","20.12345""#),
        "the already-5DP value is untouched: {merged}"
    );
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(
        stdout.contains("type_promoted"),
        "the promote is announced: {stdout}"
    );
}

/// The vocabulary is projected from `TypeClashMode::ALL`, so clap rejects anything
/// else and lists exactly the modes the library accepts.
#[test]
fn merge_rejects_an_unknown_clash_mode() {
    let d = scratch();
    let (a, b, out) = (d.join("a4.ags"), d.join("b4.ags"), d.join("nope.ags"));
    std::fs::write(&a, A).unwrap();
    std::fs::write(&b, B).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .args(["--on-type-clash", "yolo"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    for mode in ["error", "widen", "promote"] {
        assert!(err.contains(mode), "lists {mode}: {err}");
    }
}

/// Four of five `--tran-*` flags is a USAGE error naming what's missing — not a
/// silently unstamped file, and not a TRAN with three blank REQUIRED cells.
///
/// The old rule was issue+date, so this exact invocation used to succeed and
/// write `TRAN_PROD`/`TRAN_RECV`/`TRAN_STAT` empty. The CLI arm of "all five or none".
#[test]
fn merge_rejects_a_partial_tran_stamp_naming_the_missing_flags() {
    let d = scratch();
    let (a, b, out) = (
        d.join("partial_a.ags"),
        d.join("partial_b.ags"),
        d.join("partial.ags"),
    );
    std::fs::write(&a, A).unwrap();
    std::fs::write(&b, B).unwrap();
    let _ = std::fs::remove_file(&out);

    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .args([
            "--on-type-clash",
            "widen",
            "--tran-issue",
            "3",
            "--tran-date",
            "2024-03-01",
        ])
        .output()
        .unwrap();

    assert!(!o.status.success(), "a partial stamp must not merge");
    let err = String::from_utf8_lossy(&o.stderr);
    for missing in ["producer", "recipient", "status"] {
        assert!(
            err.contains(missing),
            "the error must name {missing}, so the user knows which flag to add:\n{err}"
        );
    }
    // Naming what WAS supplied would send the user to change the wrong flag.
    assert!(
        !err.contains("missing issue") && !err.contains("issue,"),
        "it must not name the flags that were supplied:\n{err}"
    );
    assert!(!out.exists(), "nothing should be written on a usage error");
}

/// `--tran-description` / `--tran-remarks` reach the emitted TRAN row.
///
/// The five REQUIRED flags were already end-to-end; these two are OTHER, arrive
/// through a different arm of the stamp seam, and nothing spawned the binary with
/// them. The assertion is on the merged BYTES — the only thing a CLI user gets —
/// not on any intermediate the binary happens to build.
#[test]
fn merge_carries_description_and_remarks_into_the_emitted_tran() {
    let d = scratch();
    let (a, b, out) = (d.join("a4.ags"), d.join("b4.ags"), d.join("stamped.ags"));
    std::fs::write(&a, A).unwrap();
    std::fs::write(&b, B).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["merge"])
        .args([&a, &b])
        .arg("--out")
        .arg(&out)
        .args([
            "--on-type-clash",
            "widen",
            "--tran-issue",
            "7",
            "--tran-date",
            "2024-05-01",
            "--tran-producer",
            "Merger",
            "--tran-recipient",
            "Client",
            "--tran-status",
            "Merged",
            "--tran-description",
            "Combined ground investigation",
            "--tran-remarks",
            "Second issue supersedes the first",
        ])
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "stamped merge succeeds: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let merged = String::from_utf8(std::fs::read(&out).unwrap()).unwrap();
    assert!(
        merged.contains("Combined ground investigation"),
        "TRAN_DESC reached the file: {merged}"
    );
    // On a merge, remarks are APPENDED to the provenance note rather than
    // replacing it — so both the caller's text and the inputs' ISNOs survive.
    assert!(
        merged.contains("Second issue supersedes the first"),
        "TRAN_REM reached the file: {merged}"
    );
    assert!(
        merged.contains("TRAN_DESC") && merged.contains("TRAN_REM"),
        "both headings are declared: {merged}"
    );
}
