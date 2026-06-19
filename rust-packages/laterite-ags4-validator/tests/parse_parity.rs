//! Parser-parity GATE — laterite-ags4-core's `ags4_codec` (the parser that builds
//! the `.ags.idx` sidecar's byte index) must parse the same file *structure* as
//! the validator's `parse` (what `validate()` runs). The cert records core's index
//! alongside the validator's clean verdict, so a cert is only coherent if the two
//! parsers agree. This guards the cert consumer (`certify` / validate-skip).
//!
//! Scope is the **structural** parse: group order, per-group headings/units/types,
//! and per-row positional values. Encoding edge cases differ by design (core's csv
//! reader rejects non-UTF-8 where the validator lossily decodes) — those never
//! yield a clean validation, so they're outside the cert's path and skipped here.
//! For clean UTF-8 `parse_str` is structurally identical to the `parse_bytes` the
//! path-validate runs, so it's the faithful proxy without an encoding_rs dev-dep.

use std::fs;
use std::path::Path;

use laterite_ags4_core::ags4_codec::read_ags4_bytes;
use laterite_ags4_validator::parse::parse_str;

/// Compare the two parsers on one input. Panics with a precise message on any
/// structural divergence. Returns `false` if either parser rejected the input
/// (not a parity case — non-UTF-8 or a one-sided parse error).
fn parity(name: &str, bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false; // non-UTF-8: core's csv reader rejects it; out of scope
    };
    let (core, val) = match (read_ags4_bytes(bytes), parse_str(text)) {
        (Ok(c), Ok(v)) => (c, v),
        _ => return false,
    };

    assert_eq!(core.order, val.group_order, "{name}: group order differs");
    for code in &core.order {
        let cg = core.get(code).expect("core group present");
        let vg = val.groups.get(code).expect("validator group present");
        assert_eq!(cg.headings, vg.headings, "{name}/{code}: headings differ");
        assert_eq!(cg.units, vg.units, "{name}/{code}: units differ");
        assert_eq!(cg.types, vg.types, "{name}/{code}: types differ");
        assert_eq!(
            cg.rows.len(),
            vg.rows.len(),
            "{name}/{code}: row count differs"
        );
        for (i, vrow) in vg.rows.iter().enumerate() {
            // core stores a row as a heading->value map; the validator stores
            // positional values. Compare positionally over the shared headings.
            let crow = &cg.rows[i];
            for (j, h) in cg.headings.iter().enumerate() {
                let cv = crow.get(h).map(String::as_str).unwrap_or("");
                let vv = vrow.values.get(j).map(String::as_str).unwrap_or("");
                assert_eq!(
                    cv, vv,
                    "{name}/{code} row {i} col {j} ({h}): core {cv:?} != validator {vv:?}"
                );
            }
        }
    }
    true
}

// Exercises quoting (comma inside a field, doubled-quote escape) + a ragged
// (short) DATA row — the parse edge cases most likely to drift between a csv
// crate and a hand-rolled splitter.
const CLEAN: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Demo, \"\"quoted\"\" name\"\r\n",
    "\r\n",
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
    "\"UNIT\",\"\",\"m\"\r\n",
    "\"TYPE\",\"ID\",\"2DP\"\r\n",
    "\"DATA\",\"BH01\",\"12.30\"\r\n",
    "\"DATA\",\"BH02\"\r\n",
);

#[test]
fn inline_quoting_and_ragged_rows_agree() {
    assert!(
        parity("clean", CLEAN.as_bytes()),
        "the clean inline case must be parseable by both"
    );
}

#[test]
fn fixtures_corpus_agrees() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let (mut compared, mut skipped) = (0u32, 0u32);
    for entry in fs::read_dir(&dir).expect("read tests/fixtures") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("ags") {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if parity(&name, &bytes) {
            compared += 1;
        } else {
            skipped += 1;
        }
    }
    // Guard against silently skipping the whole corpus (the test would otherwise
    // pass vacuously if every fixture were one-sided).
    assert!(
        compared >= 3,
        "expected to compare >=3 fixtures, got {compared} (skipped {skipped})"
    );
    eprintln!("parse-parity: {compared} fixtures agree, {skipped} skipped (non-UTF-8 / one-sided)");
}
