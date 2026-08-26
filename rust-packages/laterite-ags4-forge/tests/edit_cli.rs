//! `forge edit` end to end, through the real binary (#655).
//!
//! The in-module tests cover `apply`. Nothing covered the path a contributor
//! actually uses: flags parsed by clap, turned into operations, applied, and
//! written to disk. That path has its own failure modes — a value swallowed by
//! shell-adjacent splitting, a preview run that writes anyway, an exit code
//! that says success on a refusal — and none of them are visible from `apply`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FILE: &str = concat!(
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n",
    "\"UNIT\",\"\",\"m\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
    "\"DATA\",\"BH1\",\"100.00\",\"first\"\r\n",
    "\"DATA\",\"BH2\",\"200.00\",\"second\"\r\n",
);

fn write_fixture(dir: &Path) -> PathBuf {
    let path = dir.join("delivery.ags");
    std::fs::write(&path, FILE).expect("fixture written");
    path
}

fn forge(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_laterite-ags4-forge"))
        .args(args)
        .output()
        .expect("the binary runs")
}

/// stdout is the result document; `--json` pins the mode so the assertion does
/// not depend on whether the test harness happens to be a TTY.
fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[test]
fn a_cell_is_set_through_the_flags_and_written_to_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let dst = dir.path().join("edited.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        // The value carries a comma AND an `=`, the two characters the
        // locator grammar also uses.
        "--set",
        "LOCA:1:LOCA_REM=north, then east = far",
        "--out",
        dst.to_str().unwrap(),
        "--json",
    ]);
    assert!(out.status.success(), "{:?}", out.status);
    let doc = json(&out);
    assert_eq!(doc["unchanged"], false);
    assert_eq!(doc["written"], true);
    assert_eq!(doc["lines_changed"], 1);

    let edited = std::fs::read_to_string(&dst).expect("output written");
    assert!(
        edited.contains("\"north, then east = far\""),
        "the value must arrive quoted and whole: {edited}"
    );
    // Every other line byte-for-byte, terminators included.
    let before: Vec<&str> = FILE.split_inclusive("\r\n").collect();
    let after: Vec<&str> = edited.split_inclusive("\r\n").collect();
    assert_eq!(before.len(), after.len());
    for (i, (b, a)) in before.iter().zip(&after).enumerate() {
        if i != 4 {
            assert_eq!(b, a, "line {} must be untouched", i + 1);
        }
    }
    // The input is not touched when `--out` names somewhere else.
    assert_eq!(std::fs::read_to_string(&src).unwrap(), FILE);
}

/// The default run is the preview. A tool that writes when you did not ask it
/// to is one you cannot use to answer "what would this do".
#[test]
fn without_out_or_in_place_nothing_is_written() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--delete-group",
        "LOCA",
        "--json",
    ]);
    assert!(out.status.success());
    let doc = json(&out);
    assert_eq!(doc["written"], false);
    assert_eq!(doc["out"], serde_json::Value::Null);
    assert_eq!(std::fs::read_to_string(&src).unwrap(), FILE);
}

#[test]
fn in_place_rewrites_the_input() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--blank",
        "LOCA:2:LOCA_REM",
        "--in-place",
        "--json",
    ]);
    assert!(out.status.success());
    assert_eq!(json(&out)["written"], true);
    let edited = std::fs::read_to_string(&src).unwrap();
    assert!(
        edited.contains("\"DATA\",\"BH2\",\"200.00\",\"\""),
        "{edited}"
    );
    assert!(edited.contains("\"first\""), "row 1 untouched: {edited}");
}

/// A patch file carries what the flags cannot: a row WITH values.
#[test]
fn a_patch_file_composes_several_operations() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let dst = dir.path().join("out.ags");
    let patch = dir.path().join("p.toml");
    std::fs::write(
        &patch,
        r#"
[[op]]
kind = "add-row"
group = "LOCA"
cells = { LOCA_ID = "BH3", LOCA_REM = "a value, with a comma" }

[[op]]
kind = "delete-row"
group = "LOCA"
row = 1

[[op]]
kind = "delete-column"
group = "LOCA"
heading = "LOCA_NATE"
"#,
    )
    .expect("patch written");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--patch",
        patch.to_str().unwrap(),
        "--out",
        dst.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(json(&out)["operations"].as_array().unwrap().len(), 3);

    let edited = std::fs::read_to_string(&dst).unwrap();
    assert!(edited.contains("\"a value, with a comma\""), "{edited}");
    assert!(!edited.contains("LOCA_NATE"), "{edited}");
    assert!(!edited.contains("\"BH1\""), "{edited}");
    // Both surviving data rows at the surviving arity — counted by the
    // tokenizer, not by commas. Counting commas is the very mistake this
    // whole layer exists to stop: the added value contains one.
    let parsed = laterite_ags4_parse::parse_str(&edited).expect("output re-parses");
    let g = &parsed.groups["LOCA"];
    assert_eq!(g.headings, ["LOCA_ID", "LOCA_REM"]);
    assert_eq!(g.rows.len(), 2);
    assert!(
        g.rows.iter().all(|r| r.values.len() == 2),
        "ragged: {:?}",
        g.rows
    );
}

/// The template is only useful if it loads. Running the two commands back to
/// back is the check a reader would make.
#[test]
fn the_printed_patch_template_is_a_patch_the_tool_accepts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let template = forge(&["edit", "x.ags", "--patch-template"]);
    assert!(template.status.success());
    let patch = dir.path().join("t.toml");
    std::fs::write(&patch, &template.stdout).expect("template written");

    // The template edits LOCA, so give it a LOCA to edit.
    let src = write_fixture(dir.path());
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--patch",
        patch.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "the printed template must apply: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A refusal has to be visible to a script: a non-zero exit, the reason on
/// stderr, and nothing written.
#[test]
fn naming_something_that_is_not_there_fails_without_writing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let dst = dir.path().join("never.ags");
    for (args, expect) in [
        (["--set", "NOPE:1:X=1"], "NOPE"),
        (["--set", "LOCA:9:LOCA_ID=1"], "1-indexed"),
        (["--set", "LOCA:1:LOCA_NOPE=1"], "LOCA_NOPE"),
        (["--delete-row", "LOCA:0"], "1-indexed"),
    ] {
        let out = forge(&[
            "edit",
            src.to_str().unwrap(),
            args[0],
            args[1],
            "--out",
            dst.to_str().unwrap(),
            "--json",
        ]);
        assert!(!out.status.success(), "{args:?} must fail");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(expect), "{args:?} said: {stderr}");
        assert!(!dst.exists(), "{args:?} wrote a file anyway");
        assert_eq!(std::fs::read_to_string(&src).unwrap(), FILE);
    }
}

/// A run with no operations is a mistake, not a no-op — the caller asked for
/// something and spelled it wrong.
#[test]
fn a_run_with_no_operations_says_so() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let out = forge(&["edit", src.to_str().unwrap(), "--json"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no operations"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The operation exists to leave a UNIT undefined, so the empty value has to
/// survive clap. A unit carrying colons rides along in the same run, because
/// the grammar splits the locator and never the value.
#[test]
fn a_unit_is_rewritten_and_emptied_through_the_flags() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let dst = dir.path().join("edited.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--set-unit",
        "LOCA:LOCA_NATE=",
        "--set-unit",
        "LOCA:LOCA_REM=yyyy-mm-ddThh:mm:ss",
        "--out",
        dst.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let edited = std::fs::read_to_string(&dst).expect("written");
    let p = laterite_ags4_parse::parse_str(&edited).expect("output must re-parse");
    assert_eq!(p.groups["LOCA"].units, ["", "", "yyyy-mm-ddThh:mm:ss"]);

    // Index 2 is the UNIT row; nothing else may move.
    let before: Vec<_> = FILE.lines().collect();
    let after: Vec<_> = edited.lines().collect();
    assert_eq!(before.len(), after.len());
    for (i, (b, a)) in before.iter().zip(&after).enumerate() {
        if i != 2 {
            assert_eq!(b, a, "line {} must be byte-identical", i + 1);
        }
    }

    let doc = json(&out);
    assert_eq!(doc["operations"][0]["kind"], "set-unit");
    assert_eq!(doc["unchanged"], false);
}

/// A preview must preview. The write path is shared, but a flag that reported
/// a change and then wrote it anyway is one of the three faults #713 found in
/// this command, so the new mode asserts it rather than inheriting it.
#[test]
fn a_dry_run_set_unit_reports_the_change_and_writes_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let dst = dir.path().join("edited.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--set-unit",
        "LOCA:LOCA_NATE=",
        "--out",
        dst.to_str().unwrap(),
        "--dry-run",
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc = json(&out);
    assert_eq!(doc["written"], false);
    assert_eq!(
        doc["unchanged"], false,
        "the report must still say it would change"
    );
    assert!(!dst.exists(), "a dry run must not write the output file");
    assert_eq!(
        std::fs::read_to_string(&src).unwrap(),
        FILE,
        "a dry run must not touch the input either"
    );
}

/// Both spellings through the real binary, and a refusal that writes nothing.
/// A projection that half-wrote a column would be worse than one that refused,
/// so the failure path is asserted as hard as the success one.
#[test]
fn the_set_type_spellings_project_or_preserve_and_a_refusal_writes_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());

    let declared = |text: &str, want_type: &str, want_cell: &str| {
        let p = laterite_ags4_parse::parse_str(text).expect("output must re-parse");
        let g = p.groups.get("LOCA").expect("LOCA");
        let col = g.col("LOCA_NATE").expect("heading");
        assert_eq!(g.types[col], want_type);
        assert_eq!(g.cell(col, 0), Some(want_cell));
    };

    let projected = dir.path().join("projected.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--set-type",
        "LOCA:LOCA_NATE=0DP",
        "--out",
        projected.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    declared(
        &std::fs::read_to_string(&projected).expect("written"),
        "0DP",
        "100",
    );

    let raw = dir.path().join("raw.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--set-type-raw",
        "LOCA:LOCA_NATE=0DP",
        "--out",
        raw.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    declared(
        &std::fs::read_to_string(&raw).expect("written"),
        "0DP",
        "100.00",
    );

    // LOCA_REM holds text, so 2DP cannot be satisfied by row 1.
    let refused = dir.path().join("refused.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--set-type",
        "LOCA:LOCA_REM=2DP",
        "--out",
        refused.to_str().unwrap(),
        "--json",
    ]);
    assert!(!out.status.success(), "a refusal must not exit 0");
    assert!(!refused.exists(), "a refusal must write nothing");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("LOCA_REM") && stderr.contains("row 1"),
        "the refusal must name the heading and the row: {stderr}"
    );
}

/// One patch projects a file: create a column, declare its UNIT and TYPE, and
/// fill a cell. This is the shape a parity investigation actually uses, and it
/// only works because operations resolve against what the patch has already
/// done to the group rather than against the parse alone.
#[test]
fn one_patch_creates_a_column_declares_it_and_fills_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());
    let patch = dir.path().join("project.toml");
    std::fs::write(
        &patch,
        r#"
[[op]]
kind = "set"
group = "LOCA"
row = 1
heading = "LOCA_GL"
value = "12.34"

[[op]]
kind = "add-column"
group = "LOCA"
heading = "LOCA_GL"

[[op]]
kind = "set-unit"
group = "LOCA"
heading = "LOCA_GL"
unit = "m"

[[op]]
kind = "set-type"
group = "LOCA"
heading = "LOCA_GL"
type = "2DP"
"#,
    )
    .expect("patch written");

    let dst = dir.path().join("projected.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--patch",
        patch.to_str().unwrap(),
        "--out",
        dst.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let text = std::fs::read_to_string(&dst).expect("written");
    let p = laterite_ags4_parse::parse_str(&text).expect("output must re-parse");
    let g = p.groups.get("LOCA").expect("LOCA");
    let col = g.col("LOCA_GL").expect("the created heading");
    assert_eq!(g.units[col], "m");
    assert_eq!(g.types[col], "2DP");
    assert_eq!(g.cell(col, 0), Some("12.34"));
    assert_eq!(g.cell(col, 1), Some(""));
    for (i, row) in g.rows.iter().enumerate() {
        assert_eq!(
            row.values.len(),
            g.headings.len(),
            "row {} must not be ragged",
            i + 1
        );
    }
    // The operations were listed in a deliberately awkward order — the write
    // before the column it writes into — so this also asserts that canonical
    // order, not listing order, decides the result.
    assert_eq!(json(&out)["unchanged"], false);
}

/// Inserting through the flag, and a position past the last row refused with a
/// non-zero exit and nothing written. A typo that quietly appended would give a
/// reproducer that does not reproduce.
#[test]
fn a_row_is_inserted_at_a_position_and_a_bad_position_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = write_fixture(dir.path());

    let dst = dir.path().join("inserted.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--insert-row",
        "LOCA:1",
        "--out",
        dst.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::fs::read_to_string(&dst).expect("written");
    let p = laterite_ags4_parse::parse_str(&text).expect("output must re-parse");
    let g = p.groups.get("LOCA").expect("LOCA");
    let col = g.col("LOCA_ID").expect("heading");
    assert_eq!(g.rows.len(), 3);
    assert_eq!(g.cell(col, 0), Some(""), "the new row takes position 1");
    assert_eq!(g.cell(col, 1), Some("BH1"));
    assert!(
        g.rows.iter().all(|r| r.values.len() == g.headings.len()),
        "no row may be ragged: {text}"
    );

    let refused = dir.path().join("refused.ags");
    let out = forge(&[
        "edit",
        src.to_str().unwrap(),
        "--insert-row",
        "LOCA:9",
        "--out",
        refused.to_str().unwrap(),
        "--json",
    ]);
    assert!(!out.status.success(), "a bad position must not exit 0");
    assert!(!refused.exists(), "a refusal must write nothing");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("LOCA") && stderr.contains("row 9"),
        "the refusal must name the group and the position: {stderr}"
    );
}
