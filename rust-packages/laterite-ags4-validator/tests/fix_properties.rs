//! Property + fixture-corpus invariants for the fix engine.
//!
//! These assert the guarantees the example tests never did — the gaps that let
//! three real bugs through: a risky transliteration that folded a curly quote to
//! a bare `"` and TRUNCATED the field; a no-op path that leaked non-UTF-8 bytes;
//! and fixes that were only ever checked with `out.contains(...)`, never
//! re-parsed. The meta-property, over generated dirty-but-well-formed AGS4 and
//! over the whole vendored fixture corpus:
//!
//!   (a) fix output is valid UTF-8 (for a UTF-8 source, or whenever a fix ran),
//!   (b) it re-parses,
//!   (c) every line keeps at least its field count — no fix ever drops a field
//!       (the truncation signature) — and lines no fix touched are byte-identical,
//!   (d) re-fixing reaches a fixpoint within a few passes (idempotence is FALSE
//!       in one pass — nSF decade-crossers and ≥3-dup headings need a second).
//!
//! NOT asserted (deliberately): finding-count *monotonicity* ("a fix never adds
//! an error rule"). It is FALSE by design — a structural fix (e.g. `StripEmbeddedCr`
//! removing a stray CR that was corrupting the parse) makes the file parse cleanly,
//! which *unmasks* content findings the broken parse couldn't evaluate. That is
//! correct behaviour, not a regression; genuine corruption is already caught by
//! (b) + (c).
//!
//! Also excluded from the generator (documented sharp edges, filed separately):
//! lone-CR (Mac-classic) line terminators, which `StripEmbeddedCr` mangles.

use std::collections::HashSet;

use encoding_rs::UTF_8;
use laterite_ags4_parse::split_ags_line;
use laterite_ags4_validator::CheckOptions;
use laterite_ags4_validator::fixes::{Fix, fix_document};
use laterite_ags4_validator::parse::parse_bytes;
use proptest::prelude::*;

// --- shared helpers -----------------------------------------------------

/// `include_fyi` so Rule 1 (non-ASCII findings are FYI-severity) actually
/// yields the typography fix — the same profile the in-crate `check()` uses.
fn check_opts() -> CheckOptions {
    CheckOptions {
        include_fyi: true,
        ..CheckOptions::default()
    }
}

/// The 1-based line numbers any applied `SpanEdit` touched.
fn edited_lines(applied: &[Fix]) -> HashSet<u32> {
    applied
        .iter()
        .flat_map(|f| f.edits.iter().map(|e| e.line))
        .collect()
}

/// (c) — cell preservation / anti-truncation. Compares logical lines
/// (`str::lines()` normalises line endings + a trailing newline; a leading BOM is
/// stripped first), so the whole-doc fixes (CRLF/BOM) don't perturb it. For every
/// line the field count never shrinks (a shrink is the truncation signature); and
/// a line no `SpanEdit` touched is byte-identical field-for-field.
fn assert_cells_preserved(raw: &[u8], fixed: &[u8], applied: &[Fix]) -> Result<(), TestCaseError> {
    let orig = String::from_utf8_lossy(raw);
    let out = String::from_utf8_lossy(fixed);
    let orig = orig.strip_prefix('\u{feff}').unwrap_or(&orig);
    let out = out.strip_prefix('\u{feff}').unwrap_or(&out);
    let ol: Vec<&str> = orig.lines().collect();
    let nl: Vec<&str> = out.lines().collect();
    prop_assert_eq!(
        ol.len(),
        nl.len(),
        "fix changed the line COUNT (no fix in the menu should)"
    );
    let touched = edited_lines(applied);
    for (i, (o, n)) in ol.iter().zip(nl.iter()).enumerate() {
        let of = split_ags_line(o);
        let nf = split_ags_line(n);
        prop_assert!(
            nf.len() >= of.len(),
            "line {} lost fields {} -> {} (truncation): {:?} -> {:?}",
            i + 1,
            of.len(),
            nf.len(),
            o,
            n
        );
        if !touched.contains(&((i + 1) as u32)) {
            prop_assert_eq!(&of, &nf, "untouched line {} changed", i + 1);
        }
    }
    Ok(())
}

/// The invariant bundle (a)-(d) for one input, in one risk mode.
fn assert_fix_invariants(raw: &[u8], risky: bool) -> Result<(), TestCaseError> {
    let opts = check_opts();
    let input_utf8 = std::str::from_utf8(raw).is_ok();

    let out = fix_document(raw, &opts, risky)
        .map_err(|e| TestCaseError::fail(format!("fix_document errored: {e}")))?;

    // (a) valid UTF-8 — always for a UTF-8 source, and whenever a fix re-emitted.
    // (A non-UTF-8 file read with the default encoding is passed through verbatim
    // per the #416 contract, so only require UTF-8 output when that can't apply.)
    if input_utf8 || !out.applied.is_empty() {
        std::str::from_utf8(&out.fixed)
            .map_err(|e| TestCaseError::fail(format!("output is not valid UTF-8: {e}")))?;
    }

    // (b) re-parses.
    parse_bytes(&out.fixed, UTF_8)
        .map_err(|e| TestCaseError::fail(format!("output did not re-parse: {e:?}")))?;

    // (c) cell preservation / no truncation.
    assert_cells_preserved(raw, &out.fixed, &out.applied)?;

    // (d) fixpoint within 4 passes (idempotence is false in one).
    let mut cur = out.fixed;
    let mut reached = false;
    for _ in 0..4 {
        let step = fix_document(&cur, &opts, risky)
            .map_err(|e| TestCaseError::fail(format!("fix_document errored on re-pass: {e}")))?;
        if step.applied.is_empty() {
            prop_assert_eq!(&step.fixed, &cur, "a no-op pass still changed bytes");
            reached = true;
            break;
        }
        cur = step.fixed;
    }
    prop_assert!(reached, "no fixpoint within 4 passes");

    Ok(())
}

// --- generator ----------------------------------------------------------

/// Arbitrary cell text — the parse leaf's field shape widened with the
/// syntactically-loaded troublemakers: `"` and `,`, curly quotes (fold to `"`),
/// an ideographic comma (folds to `,`), the replacement char (folds to `?`),
/// accents, a CJK slice — but never a raw line terminator.
fn cell_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            prop::char::range(' ', '~'),
            Just('"'),
            Just(','),
            prop::char::range('\u{00a1}', '\u{017f}'),
            prop::char::range('\u{4e00}', '\u{4e80}'),
            Just('°'),
            Just('µ'),
            Just('\u{201c}'),
            Just('\u{201d}'),
            Just('\u{3001}'),
            Just('\u{2028}'), // line separator — folds to "\n" (must not split the record)
            Just('\u{2029}'), // paragraph separator — same
            Just('\u{fffd}'),
            Just('🦀'),
        ]
        .prop_filter("no line terminators", |c| *c != '\r' && *c != '\n'),
        0..8,
    )
    .prop_map(|cs| cs.into_iter().collect())
}

/// One cell: mostly arbitrary text (the typography/quoting stress), sometimes a
/// numeric or datetime literal so that — landing under a numeric/DT column — the
/// reformat / canonicalise fixes fire too.
fn any_cell() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => cell_text(),
        1 => prop_oneof![
            Just("1.5"), Just("0.997"), Just("150"), Just("12.30"), Just("1234.5678"), Just("-0.99"),
        ].prop_map(str::to_string),
        1 => prop_oneof![
            // Unambiguous (Safe, applied under the default tier) + one genuinely
            // mm/dd-ambiguous value (Risky, applied only in the risky run).
            Just("2020-08-18"), Just("18/08/2020"), Just("2020-8-1"),
            Just("01/02/2020"), Just("notadate"),
        ].prop_map(str::to_string),
    ]
}

fn type_code() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("X"),
        Just("2DP"),
        Just("1SCI"),
        Just("3SF"),
        Just("DT"),
    ]
    .prop_map(str::to_string)
}

fn group_code() -> impl Strategy<Value = String> {
    "[A-Z]{4}".prop_filter("not PROJ/TRAN", |c: &String| {
        c.as_str() != "PROJ" && c.as_str() != "TRAN"
    })
}

/// One non-PROJ group: a KEY `<CODE>_ID` column plus 0-2 further typed columns
/// (`types`), and 1-2 data rows.
#[derive(Debug, Clone)]
struct Grp {
    code: String,
    types: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn grp() -> impl Strategy<Value = Grp> {
    (group_code(), prop::collection::vec(type_code(), 0..3))
        .prop_flat_map(|(code, types)| {
            let ncols = types.len() + 1; // + the ID column
            (
                Just(code),
                Just(types),
                prop::collection::vec(prop::collection::vec(any_cell(), ncols..=ncols), 1..3),
            )
        })
        .prop_map(|(code, types, rows)| Grp { code, types, rows })
}

#[derive(Debug, Clone)]
struct Dirt {
    bom: bool,
    lf_only: bool,
    no_trailing_nl: bool,
    dup_heading: bool,
    short_row: bool,
    cr_in_cell: bool,
}

fn dirt() -> impl Strategy<Value = Dirt> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(bom, lf_only, no_trailing_nl, dup_heading, short_row, cr_in_cell)| Dirt {
                bom,
                lf_only,
                no_trailing_nl,
                dup_heading,
                short_row,
                cr_in_cell,
            },
        )
}

#[derive(Debug, Clone)]
struct Doc {
    groups: Vec<Grp>,
    dirt: Dirt,
}

fn doc() -> impl Strategy<Value = Doc> {
    (prop::collection::vec(grp(), 1..3), dirt()).prop_map(|(groups, dirt)| Doc { groups, dirt })
}

fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
fn row(cells: &[String]) -> String {
    cells.iter().map(|c| q(c)).collect::<Vec<_>>().join(",")
}

/// Render a `Doc` to AGS4 bytes with its dirt applied.
fn render(d: &Doc) -> Vec<u8> {
    // A minimal PROJ root (the fixer needs structure, not semantic validity).
    let mut lines: Vec<String> = vec![
        row(&["GROUP".into(), "PROJ".into()]),
        row(&["HEADING".into(), "PROJ_ID".into()]),
        row(&["UNIT".into(), "".into()]),
        row(&["TYPE".into(), "ID".into()]),
        row(&["DATA".into(), "P1".into()]),
    ];

    for (gi, g) in d.groups.iter().enumerate() {
        let mut headings: Vec<String> = vec!["HEADING".into(), format!("{}_ID", g.code)];
        for ci in 0..g.types.len() {
            headings.push(format!("{}_{}", g.code, ci));
        }
        // Rule 7 duplicate-heading dirt: make the 2nd heading equal the 1st.
        if d.dirt.dup_heading && gi == 0 && headings.len() >= 3 {
            headings[2] = headings[1].clone();
        }
        let mut units: Vec<String> = vec!["UNIT".into(), "".into()];
        let mut types: Vec<String> = vec!["TYPE".into(), "ID".into()];
        for ty in &g.types {
            units.push(if ty == "DT" {
                "yyyy-mm-dd".into()
            } else {
                "".into()
            });
            types.push(ty.clone());
        }
        lines.push(row(&["GROUP".into(), g.code.clone()]));
        lines.push(row(&headings));
        lines.push(row(&units));
        lines.push(row(&types));
        for (ri, r) in g.rows.iter().enumerate() {
            let mut cells: Vec<String> = vec!["DATA".into()];
            cells.extend(r.iter().cloned());
            // short-row dirt: drop the final field of the very last data row.
            let short = d.dirt.short_row
                && gi + 1 == d.groups.len()
                && ri + 1 == g.rows.len()
                && cells.len() > 2;
            if short {
                cells.pop();
            }
            let mut line = row(&cells);
            // embedded-CR dirt: splice a lone \r inside a quoted DATA cell of the
            // first group's first row (Rule 6). Placed mid-line, never as a
            // terminator, so `str::lines()` keeps it in-line.
            if d.dirt.cr_in_cell && gi == 0 && ri == 0 {
                if let Some(pos) = line.find("\",\"") {
                    line.insert(pos + 3, '\r');
                }
            }
            lines.push(line);
        }
    }

    let sep = if d.dirt.lf_only { "\n" } else { "\r\n" };
    let mut body = lines.join(sep);
    if !d.dirt.no_trailing_nl {
        body.push_str(sep);
    }
    let mut bytes = Vec::new();
    if d.dirt.bom {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

// --- properties ---------------------------------------------------------

proptest! {
    #[test]
    fn fix_invariants_safe(d in doc()) {
        assert_fix_invariants(&render(&d), false)?;
    }

    #[test]
    fn fix_invariants_risky(d in doc()) {
        assert_fix_invariants(&render(&d), true)?;
    }
}

// --- deterministic sweep over the vendored fixture corpus ---------------

#[test]
fn fixture_corpus_upholds_the_invariants() {
    let dir =
    if !dir.is_dir() {
        eprintln!("skipping fixture sweep: {} absent", dir.display());
        return;
    }
    let (mut checked, mut skipped) = (0, 0);
    for entry in std::fs::read_dir(&dir).expect("read vendor dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("ags") {
            continue;
        }
        let raw = std::fs::read(&path).expect("read fixture");
        // A non-AGS4 fixture (e.g. the AGS3 sample) can't be fixed — fix_document
        // rejects the edition. That refusal is correct; skip it, don't fail.
        if fix_document(&raw, &check_opts(), false).is_err() {
            skipped += 1;
            continue;
        }
        for risky in [false, true] {
            if let Err(e) = assert_fix_invariants(&raw, risky) {
                panic!("{} (risky={risky}): {e:?}", path.display());
            }
        }
        checked += 1;
    }
    eprintln!("fixture sweep: {checked} checked, {skipped} skipped (non-AGS4)");
    assert!(
        checked >= 80,
        "expected the full vendored corpus, saw {checked}"
    );
}
