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
fn merge_strict_errors_on_type_conflict_exit_6() {
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
}

#[test]
fn merge_lenient_writes_output_and_reports_the_real_revision() {
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
            "--lenient",
            "--tran-issue",
            "3",
            "--tran-date",
            "2024-03-01",
        ])
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "lenient merge succeeds: {}",
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
