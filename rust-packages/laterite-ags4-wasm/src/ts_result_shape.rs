//! The result interfaces published in the `typescript_custom_section` blocks are
//! hand-written strings. wasm-bindgen copies them into the `.d.ts` verbatim and
//! never compares them to the Rust structs they claim to describe — so on their
//! own they are a second hand-maintained mirror, one directory closer to the
//! engine than the app's was. These tests are what make them a contract.
//!
//! Host-testable by construction: they read the `const &str` and serialise plain
//! structs, so no `JsValue` and no `wasm-bindgen-test` (which `ci.yml` does not
//! run for this crate) is involved.
//!
//! Crate-wide by necessity: the interfaces it holds are published from seven
//! different modules, and checking each against its own structs in seven places
//! would not notice a struct that lost its interface altogether.

use laterite_ags4_validator::{dict::FALLBACK, fixes};
use serde::Serialize;

use crate::build::{AppliedFix, BuildAgs4Report, EmitFinding, TS_BUILD_RESULT};
#[cfg(feature = "censor")]
use crate::censor::{CensorDto, TS_CENSOR_RESULT};
use crate::dictionary::TS_DICT_RESULT;
#[cfg(feature = "diff")]
use crate::diff::TS_DIFF_RESULT;
use crate::fixes::TS_FIXES_RESULT;
use crate::read::{GroupMeta, TS_GROUP_META};
use crate::validate::{FindingDto, RuleGroup, TS_VALIDATE_RESULT, ValErr, ValidationReport};

/// Field names declared by one `export interface` inside a TS source block.
///
/// Deliberately a small hand-rolled scan rather than a TS parser: it only
/// needs to survive the shape *we* write here — `name?: type;` one per line,
/// with `/** … */` doc comments and `|` continuation lines between. A field
/// line is one containing `:` before any `/`, outside a doc comment.
fn declared_fields(block: &str, interface: &str) -> Vec<String> {
    let start = block
        .find(&format!("export interface {interface} {{"))
        .unwrap_or_else(|| panic!("no `export interface {interface}` in the TS block"));
    let body = &block[start..];
    let end = body
        .find("\n}")
        .unwrap_or_else(|| panic!("unterminated `interface {interface}`"));
    let mut fields = Vec::new();
    let mut in_doc = false;
    for line in body[..end].lines().skip(1) {
        let t = line.trim();
        if in_doc {
            in_doc = !t.contains("*/");
            continue;
        }
        if t.starts_with("/**") {
            // A one-line `/** … */` opens and closes on the same line.
            in_doc = !t.contains("*/");
            continue;
        }
        let Some((name, _)) = t.split_once(':') else {
            continue; // a `|` union continuation line
        };
        let name = name.trim().trim_end_matches('?');
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            fields.push(name.to_string());
        }
    }
    assert!(!fields.is_empty(), "parsed no fields out of {interface}");
    fields
}

/// Serialised keys of a value, as serde actually emits them.
fn serde_keys<T: Serialize>(v: &T) -> Vec<String> {
    serde_json::to_value(v)
        .expect("plain data")
        .as_object()
        .expect("serialises as an object")
        .keys()
        .cloned()
        .collect()
}

/// Sorted on both sides: `serde_json`'s `preserve_order` IS on for this crate
/// (via the validator / core / reference deps), so key order is declaration
/// order, and an unsorted compare would fail on ORDER rather than NAMES.
fn assert_same(interface: &str, mut declared: Vec<String>, mut actual: Vec<String>) {
    declared.sort();
    declared.dedup();
    actual.sort();
    assert_eq!(
        declared, actual,
        "TS `{interface}` and its Rust struct have drifted"
    );
}

/// Every field must be populated here — a `None` in a `skip_serializing_if`
/// field would vanish from the serialised keys and the test would then be
/// asserting that an OPTIONAL field may be undeclared, which is the drift it
/// exists to catch.
fn a_finding() -> FindingDto {
    FindingDto {
        line: Some(1),
        group: "LOCA".into(),
        desc: "d".into(),
        target: Some("cell".into()),
        field_index: Some(0),
        heading: Some("LOCA_ID".into()),
        data_row: Some(1),
        char_span: Some([0, 1]),
        severity: Some("warning".into()),
    }
}

#[test]
fn ts_interfaces_match_the_serde_structs() {
    assert_same(
        "FindingDto",
        declared_fields(TS_VALIDATE_RESULT, "FindingDto"),
        serde_keys(&a_finding()),
    );
    assert_same(
        "RuleGroup",
        declared_fields(TS_VALIDATE_RESULT, "RuleGroup"),
        serde_keys(&RuleGroup {
            rule: "AGS Format Rule 1".into(),
            total: 1,
            items: vec![a_finding()],
        }),
    );
    assert_same(
        "ValErr",
        declared_fields(TS_VALIDATE_RESULT, "ValErr"),
        serde_keys(&ValErr {
            kind: "not_ags4".into(),
            message: "m".into(),
        }),
    );
    assert_same(
        "ValidationReport",
        declared_fields(TS_VALIDATE_RESULT, "ValidationReport"),
        serde_keys(&ValidationReport::failure("not_ags4", "m".into())),
    );
    assert_same(
        "EmitFinding",
        declared_fields(TS_BUILD_RESULT, "EmitFinding"),
        serde_keys(&EmitFinding {
            rule: "AGS Format Rule 1".into(),
            line: Some(1),
            group: "LOCA".into(),
            desc: "d".into(),
            severity: Some("warning".into()),
        }),
    );
    assert_same(
        "AppliedFix",
        declared_fields(TS_BUILD_RESULT, "AppliedFix"),
        serde_keys(&AppliedFix {
            kind: fixes::FixKind::StripBom,
            label: "l".into(),
            rule: "AGS Format Rule 1".into(),
            line: Some(1),
            risk: fixes::FixRisk::Safe,
        }),
    );
    assert_same(
        "BuildReport",
        declared_fields(TS_BUILD_RESULT, "BuildReport"),
        serde_keys(&BuildAgs4Report {
            text: String::new(),
            findings: Vec::new(),
            applied: Vec::new(),
            fixes_applied: 0,
        }),
    );
    // The gated surfaces' shapes are checked when they are built. A slim
    // build ships neither the export nor its TS, so there is no pair left
    // to have drifted — but the `full` build CI tests still covers both.
    #[cfg(feature = "censor")]
    {
        assert_same(
            "CensorTally",
            declared_fields(TS_CENSOR_RESULT, "CensorTally"),
            serde_keys(&laterite_ags4_censor::Tally::default()),
        );
        assert_same(
            "CensorResult",
            declared_fields(TS_CENSOR_RESULT, "CensorResult"),
            serde_keys(&CensorDto {
                text: String::new(),
                tally: laterite_ags4_censor::Tally::default(),
            }),
        );
    }
    assert_same(
        "GroupMeta",
        declared_fields(TS_GROUP_META, "GroupMeta"),
        serde_keys(&GroupMeta {
            headings: Vec::new(),
            units: Vec::new(),
            types: Vec::new(),
            sql_types: Vec::new(),
        }),
    );

    #[cfg(feature = "diff")]
    {
        let cell = || laterite_ags4_diff::CellDelta {
            heading: "LOCA_ID".into(),
            ags_type: "ID".into(),
            a: Some("BH01".into()),
            b: Some("BH02".into()),
        };
        let row = || laterite_ags4_diff::RowDelta {
            kind: "changed",
            key: vec!["BH01".into()],
            line_a: Some(1),
            line_b: Some(1),
            cells: vec![cell()],
        };
        assert_same(
            "CellDelta",
            declared_fields(TS_DIFF_RESULT, "CellDelta"),
            serde_keys(&cell()),
        );
        assert_same(
            "RowDelta",
            declared_fields(TS_DIFF_RESULT, "RowDelta"),
            serde_keys(&row()),
        );
        assert_same(
            "GroupDelta",
            declared_fields(TS_DIFF_RESULT, "GroupDelta"),
            serde_keys(&laterite_ags4_diff::GroupDelta {
                code: "LOCA".into(),
                added: 0,
                removed: 0,
                changed: 1,
                headings_added: Vec::new(),
                headings_removed: Vec::new(),
                keyed: true,
                key_headings: vec!["LOCA_ID".into()],
                rows: vec![row()],
            }),
        );
        assert_same(
            "RevisionDelta",
            declared_fields(TS_DIFF_RESULT, "RevisionDelta"),
            serde_keys(&laterite_ags4_diff::RevisionDelta {
                groups: Vec::new(),
                groups_added: Vec::new(),
                groups_removed: Vec::new(),
                total_added: 0,
                total_removed: 0,
                total_changed: 0,
            }),
        );
    }

    // Straight off the real builder rather than a hand-made value: these
    // three carry `skip_serializing_if` fields (`unit`, `parent`), and a
    // literal with them set to `None` would drop the keys and quietly assert
    // that an optional field may go undeclared.
    let dict = laterite_ags4_validator::dict::dictionary_dto(FALLBACK);
    let group = dict
        .groups
        .iter()
        .find(|g| g.parent.is_some())
        .expect("a non-root group");
    let heading = group
        .headings
        .iter()
        .find(|h| h.unit.is_some())
        .expect("a heading with a unit");
    assert_same(
        "DictHeading",
        declared_fields(TS_DICT_RESULT, "DictHeading"),
        serde_keys(heading),
    );
    assert_same(
        "DictGroup",
        declared_fields(TS_DICT_RESULT, "DictGroup"),
        serde_keys(group),
    );
    assert_same(
        "StandardDict",
        declared_fields(TS_DICT_RESULT, "StandardDict"),
        serde_keys(&dict),
    );

    let edit = || fixes::SpanEdit {
        line: 1,
        start: 0,
        end: 1,
        replacement: "b".into(),
        expected: "a".into(),
    };
    assert_same(
        "SpanEdit",
        declared_fields(TS_FIXES_RESULT, "SpanEdit"),
        serde_keys(&edit()),
    );
    assert_same(
        "Fix",
        declared_fields(TS_FIXES_RESULT, "Fix"),
        serde_keys(&fixes::Fix {
            kind: fixes::FixKind::StripBom,
            label: "l".into(),
            rule: "AGS Format Rule 1".into(),
            line: Some(1),
            risk: fixes::FixRisk::Safe,
            edits: vec![edit()],
        }),
    );
}

/// The parser must be able to fail. Without this, a `declared_fields` that
/// silently returned the wrong thing would make every assertion above pass
/// against itself.
#[test]
fn the_interface_parser_can_see_a_missing_field() {
    let mut fields = declared_fields(TS_VALIDATE_RESULT, "ValErr");
    assert_eq!(fields, vec!["kind", "message"]);
    fields.pop();
    assert_ne!(fields, serde_keys(&a_finding()));
}

/// `AppliedFix.kind` / `.risk` are unions over enums owned by
/// **laterite-ags4-validator**. A new `FixKind` variant there would otherwise
/// silently fall outside the union we publish here, and the `.d.ts` would lie
/// about a value consumers can actually receive.
///
/// Rather than keep a second hand-written variant list to compare against,
/// this asks serde for the authoritative one: deserialising a bogus token
/// fails with `unknown variant ..., expected one of ...`, which enumerates
/// every variant the enum really has.
#[test]
fn fix_unions_match_the_validators_enums() {
    fn variants<'de, T: serde::Deserialize<'de>>() -> Vec<String> {
        // `.err()` rather than `expect_err`: the latter needs `T: Debug`,
        // and these enums are only required to be Deserialize.
        let err = serde_json::from_str::<T>("\"__not_a_variant__\"")
            .err()
            .expect("a bogus token must not deserialise");
        let msg = err.to_string();
        // Read the backticked tokens rather than a fixed phrase: serde says
        // "expected one of `a`, `b`, `c`" for three or more variants but
        // "expected `safe` or `risky`" for two, and FixRisk has two. The
        // first backticked token is always the bogus name we passed in.
        let all: Vec<String> = msg
            .split('`')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect();
        let (first, rest) = all
            .split_first()
            .unwrap_or_else(|| panic!("serde changed its unknown-variant message: {msg}"));
        assert_eq!(
            first, "__not_a_variant__",
            "unexpected message shape: {msg}"
        );
        assert!(!rest.is_empty(), "no variants listed in: {msg}");
        rest.to_vec()
    }

    fn union_members(block: &str, field: &str) -> Vec<String> {
        let at = block.find(field).expect("field is declared");
        let body = &block[at..];
        let end = body.find(';').expect("field declaration ends in `;`");
        body[..end]
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }

    // Both blocks: `AppliedFix` (what `build_ags4` reports it did) and `Fix`
    // (what `compute_fixes` offers) publish the SAME two unions from the same
    // two enums, in two separately-written strings. Checking only one leaves
    // the other free to drift.
    for (block, what) in [(TS_BUILD_RESULT, "AppliedFix"), (TS_FIXES_RESULT, "Fix")] {
        for (field, mut actual) in [
            ("kind:", variants::<fixes::FixKind>()),
            ("risk:", variants::<fixes::FixRisk>()),
        ] {
            let mut declared = union_members(block, field);
            declared.sort();
            actual.sort();
            assert_eq!(
                declared, actual,
                "TS `{what}.{field}` union and the validator's enum have drifted"
            );
        }
    }
}
