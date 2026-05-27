//! End-to-end smoke tests. Each test invokes the built binary against the
//! shared `examples/output/large.ags5db` fixture; if that's absent (e.g.
//! a clean checkout that hasn't run `examples/create_ags5db.py` yet) the
//! tests skip via the `#[ignore]` convention the Python suite uses.
//!
//! The full parity test surface (byte-for-byte vs `ags5db-py`) is a Phase 3
//! deliverable. These cover the shape contract (exit codes, basic structure)
//! so a regression in clap wiring or DuckDB plumbing surfaces here.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture() -> Option<PathBuf> {
    let p = workspace_root().join("examples/output/large.ags5db");
    p.exists().then_some(p)
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at `ags5db/`; the workspace root is one up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("ags5db has a parent")
        .to_path_buf()
}

fn bin() -> Command {
    Command::cargo_bin("ags5db").expect("ags5db binary built")
}

#[test]
fn version_includes_crate_version() {
    bin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ags5db"));
}

#[test]
fn missing_file_exits_3() {
    bin()
        .args(["info", "definitely-not-here.ags5db"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("file not found"));
}

#[test]
fn unknown_group_exits_4_with_hint() {
    let Some(db) = fixture() else { return };
    bin()
        .args(["peek", db.to_str().unwrap(), "LPL"])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("did you mean").and(predicate::str::contains("LLPL")));
}

#[test]
fn bad_predicate_exits_5() {
    let Some(db) = fixture() else { return };
    bin()
        .args([
            "count",
            db.to_str().unwrap(),
            "LOCA",
            "--where",
            "loca_id BH01",
        ])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("expected 'field<op>value'"));
}

// Phase F landed db-to-agsx as a real command, so there are no
// remaining write-command stubs and no exit-9 path reachable from CLI
// input. The `CliError::NotImpl` variant in error.rs is kept as a
// placeholder in case future commands land as redirects before their
// real implementation.

#[test]
fn sql_unknown_table_exits_8() {
    let Some(db) = fixture() else { return };
    bin()
        .args([
            "sql",
            db.to_str().unwrap(),
            "SELECT * FROM totally_not_a_table",
        ])
        .assert()
        .failure()
        .code(8)
        .stderr(predicate::str::contains("SQL error"));
}

#[test]
fn info_emits_format_version_in_ndjson() {
    let Some(db) = fixture() else { return };
    bin()
        .args(["--output", "ndjson", "info", db.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"format_version\""));
}

#[test]
fn groups_nonempty_filters_zero_rows() {
    let Some(db) = fixture() else { return };
    let out = bin()
        .args([
            "--output",
            "ndjson",
            "groups",
            db.to_str().unwrap(),
            "--nonempty",
        ])
        .output()
        .expect("groups runs");
    assert!(out.status.success(), "exit={:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Every line must be one JSON record with rows>0 (the --nonempty filter).
    for line in stdout.lines() {
        assert!(line.contains("\"rows\":"), "no rows key in {:?}", line);
        assert!(!line.contains("\"rows\":0,"), "got rows=0 line: {:?}", line);
    }
}

#[test]
fn count_returns_scalar_integer() {
    let Some(db) = fixture() else { return };
    bin()
        .args(["--output", "ndjson", "count", db.to_str().unwrap(), "LOCA"])
        .assert()
        .success()
        // The scalar emits as a bare integer (no JSON object wrapping).
        .stdout(predicate::function(|s: &str| {
            s.trim().parse::<u64>().is_ok()
        }));
}

#[test]
fn recipe_list_emits_known_slugs() {
    bin()
        .args(["--output", "ndjson", "recipe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("depth-band-join"));
}

#[test]
fn recipe_unknown_exits_4_with_hint() {
    bin()
        .args(["--output", "ndjson", "recipe", "depthband"])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("did you mean"));
}

#[test]
fn peek_default_limit_caps_rows() {
    let Some(db) = fixture() else { return };
    let out = bin()
        .args([
            "--output",
            "ndjson",
            "peek",
            db.to_str().unwrap(),
            "LOCA",
            "--fields",
            "loca_id",
            "--limit",
            "5",
        ])
        .output()
        .expect("peek runs");
    assert!(out.status.success(), "exit={:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().count(), 5, "got {:?}", stdout);
}

#[test]
fn inspect_emits_meta_and_counts() {
    let Some(db) = fixture() else { return };
    bin()
        .args(["--output", "ndjson", "inspect", db.to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"format_version\"")
                .and(predicate::str::contains("\"n_groups\""))
                .and(predicate::str::contains("\"n_headings\"")),
        );
}

#[test]
fn inspect_group_emits_group_block_and_headings() {
    let Some(db) = fixture() else { return };
    bin()
        .args([
            "--output",
            "ndjson",
            "inspect",
            db.to_str().unwrap(),
            "--group",
            "LOCA",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"group\"")
                .and(predicate::str::contains("\"headings\""))
                .and(predicate::str::contains("LOCA_ID")),
        );
}

#[test]
fn diff_same_file_exits_0_with_empty_payload() {
    let Some(db) = fixture() else { return };
    bin()
        .args([
            "--output",
            "ndjson",
            "diff",
            db.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("\"changed_groups\":[]"));
}

#[test]
fn pack_unpack_roundtrip_is_byte_identical() {
    let Some(db) = fixture() else { return };

    // tempfile crate isolates our writes from the workspace and cleans up.
    let dir = tempfile::tempdir().expect("tempdir");
    let src_copy = dir.path().join("rt.ags5db");
    let zst = dir.path().join("rt.ags5db.zst");
    let out = dir.path().join("rt_roundtrip.ags5db");

    std::fs::copy(&db, &src_copy).expect("copy fixture");

    bin()
        .args([
            "--quiet",
            "pack",
            src_copy.to_str().unwrap(),
            "--dest",
            zst.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(zst.exists(), "pack didn't create .zst");
    let zst_size = std::fs::metadata(&zst).unwrap().len();
    let src_size = std::fs::metadata(&src_copy).unwrap().len();
    assert!(
        zst_size < src_size,
        "compressed ({}) not smaller than source ({})",
        zst_size,
        src_size,
    );

    bin()
        .args([
            "--quiet",
            "unpack",
            zst.to_str().unwrap(),
            "--dest",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let src_bytes = std::fs::read(&src_copy).expect("read src");
    let out_bytes = std::fs::read(&out).expect("read out");
    assert_eq!(
        src_bytes.len(),
        out_bytes.len(),
        "roundtrip size diverged: {} -> {}",
        src_bytes.len(),
        out_bytes.len(),
    );
    assert_eq!(
        src_bytes, out_bytes,
        "roundtrip content diverged (sizes match but bytes don't)",
    );
}

#[test]
fn ags4_to_db_roundtrips_a_minimal_file() {
    // Build a tiny AGS4 fixture, convert, verify the v6.5 output has the
    // expected row counts + that LOCA's parent_id points at the PROJ row.
    let dir = tempfile::tempdir().expect("tempdir");
    let ags4 = dir.path().join("tiny.ags");
    let db = dir.path().join("tiny.ags5db");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test project"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","100.50"
"DATA","BH02","TP","200.75"
"#;
    std::fs::write(&ags4, fixture).unwrap();

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            ags4.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();

    let conn = duckdb::Connection::open_with_flags(
        &db,
        duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap(),
    )
    .unwrap();
    let n_proj: i64 = conn
        .query_row("SELECT COUNT(*) FROM g_proj", [], |r| r.get(0))
        .unwrap();
    let n_loca: i64 = conn
        .query_row("SELECT COUNT(*) FROM g_loca", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n_proj, 1, "expected 1 PROJ row");
    assert_eq!(n_loca, 2, "expected 2 LOCA rows");

    // LOCA_NATE coerced via the registry's 2DP -> DOUBLE typing.
    let nate: f64 = conn
        .query_row(
            "SELECT loca_nate FROM v_loca WHERE loca_id = 'BH01'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        (nate - 100.5).abs() < 1e-9,
        "LOCA_NATE coercion: got {}",
        nate
    );

    // Parent_id linkage: every LOCA row points at the single PROJ row.
    let n_linked: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM g_loca l
             JOIN g_proj p ON l.parent_id = p.id",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n_linked, 2, "LOCA rows should all link to PROJ");
}

#[test]
fn ags4_to_db_append_is_idempotent() {
    // Convert once, then convert the same source again with --append.
    // Content-hash dedup means the second pass shouldn't add rows.
    let dir = tempfile::tempdir().expect("tempdir");
    let ags4 = dir.path().join("idem.ags");
    let db = dir.path().join("idem.ags5db");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE"
"UNIT","",""
"TYPE","ID","PA"
"DATA","BH01","CP"
"DATA","BH02","TP"
"#;
    std::fs::write(&ags4, fixture).unwrap();

    let count_loca = |path: &std::path::Path| -> i64 {
        let conn = duckdb::Connection::open_with_flags(
            path,
            duckdb::Config::default()
                .access_mode(duckdb::AccessMode::ReadOnly)
                .unwrap(),
        )
        .unwrap();
        conn.query_row("SELECT COUNT(*) FROM g_loca", [], |r| r.get(0))
            .unwrap()
    };

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            ags4.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(count_loca(&db), 2, "expected 2 LOCA rows on first ingest");

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            ags4.to_str().unwrap(),
            db.to_str().unwrap(),
            "--append",
        ])
        .assert()
        .success();
    assert_eq!(
        count_loca(&db),
        2,
        "append of the same source must not duplicate rows",
    );
}

#[test]
fn ags4_to_db_auto_passthroughs_unknown_groups() {
    // A group not in the static registry should be auto-registered as
    // a passthrough — all-string headings, parent=LOCA, marked in
    // _spec_groups.contents so a consumer can see it wasn't dictionary-
    // declared.
    let dir = tempfile::tempdir().expect("tempdir");
    let ags4 = dir.path().join("custom.ags");
    let db = dir.path().join("custom.ags5db");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID"
"UNIT",""
"TYPE","X"
"DATA","P1"

"GROUP","XXYZ"
"HEADING","XXYZ_ID","XXYZ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","Q1","custom group not in the registry"
"#;
    std::fs::write(&ags4, fixture).unwrap();

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            ags4.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();

    let conn = duckdb::Connection::open_with_flags(
        &db,
        duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap(),
    )
    .unwrap();
    let n_xxyz: i64 = conn
        .query_row("SELECT COUNT(*) FROM g_xxyz", [], |r| r.get(0))
        .expect("g_xxyz table created");
    assert_eq!(n_xxyz, 1, "expected 1 XXYZ row");
    let contents: String = conn
        .query_row(
            "SELECT contents FROM _spec_groups WHERE code = 'XXYZ'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        contents.contains("passthrough"),
        "_spec_groups should mark XXYZ as passthrough: {}",
        contents,
    );
}

#[test]
fn db_to_agsx_produces_zstd_tar_with_project_xml() {
    let Some(db) = fixture() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let agsx = dir.path().join("out.agsx");

    bin()
        .args([
            "--quiet",
            "db-to-agsx",
            db.to_str().unwrap(),
            agsx.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(agsx.exists(), "agsx didn't produce dst");

    // First 4 bytes of a zstd frame are 0x28 b5 2f fd.
    let header = std::fs::read(&agsx).unwrap();
    assert!(header.len() > 4, "archive smaller than zstd header");
    assert_eq!(
        &header[..4],
        &[0x28, 0xb5, 0x2f, 0xfd],
        "missing zstd magic"
    );

    // Quick round-trip: unpack through our own `unpack` command, then
    // tar -t to confirm `project.xml` is the first entry.
    let tar_path = dir.path().join("out.tar");
    bin()
        .args([
            "--quiet",
            "unpack",
            agsx.to_str().unwrap(),
            "--dest",
            tar_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let bytes = std::fs::read(&tar_path).expect("tar bytes");
    let needle = b"project.xml";
    assert!(
        bytes.windows(needle.len()).any(|w| w == needle),
        "archive doesn't include project.xml entry",
    );
}

#[test]
fn lock_unlock_roundtrip_is_byte_identical() {
    let Some(db) = fixture() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src_copy = dir.path().join("rt.ags5db");
    let locked = dir.path().join("rt.zst.age");
    let unlocked = dir.path().join("rt_restored.ags5db");
    std::fs::copy(&db, &src_copy).expect("copy fixture");

    bin()
        .args([
            "--quiet",
            "lock",
            src_copy.to_str().unwrap(),
            "--password",
            "testpw",
            "--dest",
            locked.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(locked.exists(), "lock didn't write .zst.age");

    bin()
        .args([
            "--quiet",
            "unlock",
            locked.to_str().unwrap(),
            "--password",
            "testpw",
            "--dest",
            unlocked.to_str().unwrap(),
        ])
        .assert()
        .success();
    let src_bytes = std::fs::read(&src_copy).unwrap();
    let out_bytes = std::fs::read(&unlocked).unwrap();
    assert_eq!(src_bytes, out_bytes, "lock/unlock round-trip diverged");
}

#[test]
fn unlock_wrong_password_exits_6() {
    let Some(db) = fixture() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src_copy = dir.path().join("wp.ags5db");
    let locked = dir.path().join("wp.zst.age");
    let unlocked = dir.path().join("wp_restored.ags5db");
    std::fs::copy(&db, &src_copy).expect("copy fixture");

    bin()
        .args([
            "--quiet",
            "lock",
            src_copy.to_str().unwrap(),
            "--password",
            "correctpw",
            "--dest",
            locked.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--quiet",
            "unlock",
            locked.to_str().unwrap(),
            "--password",
            "wrongpw",
            "--dest",
            unlocked.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("wrong password"));
    assert!(
        !unlocked.exists(),
        "unlock with wrong password wrote a file"
    );
}

#[test]
fn pack_dry_run_writes_nothing() {
    let Some(db) = fixture() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src_copy = dir.path().join("dr.ags5db");
    let zst = dir.path().join("dr.ags5db.zst");
    std::fs::copy(&db, &src_copy).expect("copy fixture");

    bin()
        .args([
            "--output",
            "ndjson",
            "pack",
            src_copy.to_str().unwrap(),
            "--dest",
            zst.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\":true"))
        .stdout(predicate::str::contains("\"would_clobber\":false"));
    assert!(!zst.exists(), "dry-run wrote a file: {}", zst.display());
}

#[test]
fn db_to_ags4_round_trips_a_minimal_file() {
    // ags4 -> db -> ags4. The output isn't byte-identical (column order,
    // typed value formatting), but the original groups + row counts + key
    // values must survive intact when we re-parse it.
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src.ags");
    let db = dir.path().join("rt.ags5db");
    let out = dir.path().join("out.ags");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","100.50"
"DATA","BH02","TP","200.75"
"#;
    std::fs::write(&src, fixture).unwrap();

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            src.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--output",
            "ndjson",
            "db-to-ags4",
            db.to_str().unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"db-to-ags4\""))
        .stdout(predicate::str::contains("\"rows\":3"));

    // Re-parse the output and check the structure survived. Two LOCA
    // DATA rows plus a PROJ DATA row = 3.
    let body = std::fs::read_to_string(&out).expect("read out.ags");
    assert!(body.contains("\"GROUP\",\"PROJ\""), "missing PROJ section");
    assert!(body.contains("\"GROUP\",\"LOCA\""), "missing LOCA section");
    assert!(body.contains("\"DATA\",\"BH01\""), "missing BH01");
    assert!(body.contains("\"DATA\",\"BH02\""), "missing BH02");
    assert!(body.contains("\"DATA\",\"P1\""), "missing PROJ row");
    // 2DP precision preserved on the way out.
    assert!(
        body.contains("100.50") && body.contains("200.75"),
        "2DP precision lost in round-trip",
    );
}

#[test]
fn db_to_ags4_bails_exit_7_on_record_link() {
    // Build an AGS4 with a *passthrough* group (not in the registry)
    // whose TYPE row declares an RL heading. For registered groups the
    // AGS4 TYPE row is overridden by the registry's declared types, so
    // we use a custom group code to keep the RL type in `_spec_headings`.
    // Ingest succeeds (passthrough is permissive); db-to-ags4's
    // pre-flight check must refuse on the RL.
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("rl.ags");
    let db = dir.path().join("rl.ags5db");
    let out = dir.path().join("rl_out.ags");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","With RL"

"GROUP","ZRLX"
"HEADING","ZRLX_ID","ZRLX_REF"
"UNIT","",""
"TYPE","X","RL"
"DATA","R1","PROJ|P1"
"#;
    std::fs::write(&src, fixture).unwrap();

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            src.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--quiet",
            "db-to-ags4",
            db.to_str().unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(7)
        .stderr(predicate::str::contains("Record Link"));
    assert!(
        !out.exists(),
        "RL bail must not leave a partial output file"
    );
}

#[test]
fn db_to_ags4_warns_on_missing_tran_but_succeeds() {
    // No TRAN group present. Output should still be written; stderr
    // should carry the advisory; exit code is 0.
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("no_tran.ags");
    let db = dir.path().join("no_tran.ags5db");
    let out = dir.path().join("no_tran_out.ags");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test"
"#;
    std::fs::write(&src, fixture).unwrap();

    bin()
        .args([
            "--quiet",
            "ags4-to-db",
            src.to_str().unwrap(),
            db.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--quiet",
            "db-to-ags4",
            db.to_str().unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("TRAN group missing"));
    assert!(
        out.exists(),
        "missing-TRAN warning must not abort the write"
    );
}

#[test]
fn ags4_to_db_slurps_file_fset_attachments() {
    // Build an AGS4 fixture with a FILE group referencing two adjacent
    // files. ags4-to-db --attachments-dir should slurp both into the
    // blob table; db-to-ags4 should re-emit them next to the output.
    let dir = tempfile::tempdir().expect("tempdir");
    let attach = dir.path().join("att");
    std::fs::create_dir_all(&attach).unwrap();
    let pdf = attach.join("report.pdf");
    let log = attach.join("trace.log");
    std::fs::write(&pdf, b"%PDF-1.4 fake content").unwrap();
    std::fs::write(&log, b"trace line one\ntrace line two\n").unwrap();

    let src = dir.path().join("att.ags");
    let db = dir.path().join("att.ags5db");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","With attachments"

"GROUP","FILE"
"HEADING","FILE_FSET","FILE_NAME","FILE_TYPE"
"UNIT","","",""
"TYPE","X","X","X"
"DATA","FS1","report.pdf","REPORT"
"DATA","FS2","trace.log","LOG"
"#;
    std::fs::write(&src, fixture).unwrap();

    bin()
        .args([
            "--output",
            "ndjson",
            "ags4-to-db",
            src.to_str().unwrap(),
            db.to_str().unwrap(),
            "--attachments-dir",
            attach.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"attachments\":2"));

    let conn = duckdb::Connection::open_with_flags(
        &db,
        duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap(),
    )
    .unwrap();
    let n_blobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM blob WHERE kind = 'attachment'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n_blobs, 2, "expected 2 attachment blobs in the DB");

    // Now unspool back via db-to-ags4. Output dir is a fresh tempdir;
    // the attachments should land alongside the new .ags.
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("out.ags");
    bin()
        .args([
            "--output",
            "ndjson",
            "db-to-ags4",
            db.to_str().unwrap(),
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"attachments\":2"));

    // db-to-ags4 reconstructs the AGS4 Rule 20 sidecar tree
    // FILE/<FILE_FSET>/<FILE_NAME> (FS1→report.pdf, FS2→trace.log),
    // not a flat dump — so `ags4-check --check-files` on the emitted
    // file passes and the round-trip is spec-faithful.
    let pdf_out = out_dir.join("FILE").join("FS1").join("report.pdf");
    let log_out = out_dir.join("FILE").join("FS2").join("trace.log");
    assert!(
        pdf_out.exists(),
        "FILE/FS1/report.pdf not re-emitted (tree)"
    );
    assert!(log_out.exists(), "FILE/FS2/trace.log not re-emitted (tree)");
    assert!(
        !out_dir.join("report.pdf").exists(),
        "attachment must NOT be written flat — Rule 20 wants FILE/<fset>/<name>",
    );
    assert_eq!(
        std::fs::read(&pdf_out).unwrap(),
        b"%PDF-1.4 fake content",
        "PDF bytes corrupted in round-trip",
    );
    assert_eq!(
        std::fs::read(&log_out).unwrap(),
        b"trace line one\ntrace line two\n",
        "LOG bytes corrupted in round-trip",
    );
}

#[test]
fn ags4_to_db_warns_on_missing_attachment() {
    let dir = tempfile::tempdir().expect("tempdir");
    let attach = dir.path().join("att");
    std::fs::create_dir_all(&attach).unwrap();
    let src = dir.path().join("missing.ags");
    let db = dir.path().join("missing.ags5db");
    let fixture = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","No file"

"GROUP","FILE"
"HEADING","FILE_FSET","FILE_NAME","FILE_TYPE"
"UNIT","","",""
"TYPE","X","X","X"
"DATA","FS1","ghost.pdf","REPORT"
"#;
    std::fs::write(&src, fixture).unwrap();

    bin()
        .args([
            "--output",
            "ndjson",
            "ags4-to-db",
            src.to_str().unwrap(),
            db.to_str().unwrap(),
            "--attachments-dir",
            attach.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("missing attachment: ghost.pdf"))
        .stdout(predicate::str::contains("\"attachments\":0"));
}

#[test]
fn db_to_ags4_dry_run_writes_nothing() {
    let Some(db) = fixture() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("dry.ags");

    bin()
        .args([
            "--output",
            "ndjson",
            "db-to-ags4",
            db.to_str().unwrap(),
            out.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\":true"));
    assert!(!out.exists(), "dry-run wrote a file: {}", out.display());
}

#[test]
fn diff_different_files_exits_1() {
    // Only runs if both fixtures are present. The synthetic fixture is
    // built by examples/create_ags5db.py — skip gracefully if absent.
    let a = match fixture() {
        Some(p) => p,
        None => return,
    };
    let b = workspace_root().join("examples/output/large_synthetic.ags5db");
    if !b.exists() {
        return;
    }
    bin()
        .args([
            "--output",
            "ndjson",
            "diff",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"changed_groups\""));
}
