//! `lat census --json` — dump this binary's own parser, **reflected**.
//!
//! The surface census (see `surface-census.json` + `tools/gen_census.py`) needs one
//! **authority** for every closed table the toolchain hand-copies across surfaces —
//! the CLI verbs, each verb's flags, and each flag's value enum. This is it.
//!
//! The load-bearing property is that **nothing here is a list**. Every name comes
//! from clap's introspection of the *same* [`Cli`] struct that parses the real
//! command line, so the census cannot drift from the tool. A second hand-written
//! table is exactly the failure this whole gate exists to catch: `lat merge` shipped
//! in the binary (#494) and never reached the uvx or npx launchers, and no gate saw
//! it — because every gate we had compared *knob names against another hand-list*,
//! and both hand-lists were equally wrong.
//!
//! Hidden from `--help`: this is a machine door for the census generator, not a user
//! command. It stays in the shipped binary rather than behind a feature flag because
//! the census must be able to interrogate the **released** CLI, not a test build.

use clap::CommandFactory;
use serde_json::{Value, json};

use crate::cli::Cli;

/// The census schema version — the set of TABLES a dump carries.
///
/// 1: `verbs`. 2: + `editions` / `fallback_edition`. 3: + `encodings`.
/// 4: per-verb `args` are now DIFFED (npx grew a per-verb flag table to report).
///
/// `tools/gen_census.py` pins a minimum and refuses anything older, so a launcher
/// built before a table existed fails loudly instead of reporting that table empty.
/// All three launchers declare this; they must agree.
pub const CENSUS_VERSION: u32 = 5;

/// The encoding labels the census resolves on every surface.
///
/// The accepted set is not enumerable — every WHATWG label goes through
/// `Encoding::for_label` — so the census compares RESOLUTIONS of a fixed probe list
/// instead: each launcher reports what *it* turns each label into, and the three
/// must agree.
///
/// Every entry earns its place:
///   * `latin-1` — hyphenated; WHATWG does not know it, the leaf does.
///   * `latin9` / `latin-9` — the two labels that ONLY the `lat` binary's private
///     table accepted, while the Python library rejected them. The whole reason this
///     table exists.
///   * `iso-8859-15` / `l9` — the same encoding by labels WHATWG *does* know, so a
///     surface that special-cases the aliases but drops the standard ones is caught.
///   * `shift_jis` — a WHATWG label we never special-case; proves `for_label` is
///     still reached rather than a hand-list having replaced it.
///   * `cp1252x` — **a typo, and it must resolve to NOTHING on every surface.** This
///     is the policy pin. Node used to answer `UTF-8` here (a silent fallback), so a
///     caller who fat-fingered a label got the wrong text and a clean bill of health.
pub const ENCODING_PROBES: &[&str] = &[
    "utf-8",
    "utf8",
    "cp1252",
    "windows-1252",
    "latin1",
    "latin-1",
    "iso-8859-1",
    "iso-8859-15",
    "latin9",
    "latin-9",
    "l9",
    "shift_jis",
    "cp1252x",
];

/// Each probe label → what THIS launcher resolves it to (`null` = refused).
///
/// Routed through the CLI's own `resolve_encoding`, NOT the parse leaf directly.
/// That is deliberate: the leaf was always right, and the bug lived in the thin
/// wrappers above it. A census that asked the leaf would agree with itself and see
/// nothing.
fn encodings_json() -> Value {
    let mut m = serde_json::Map::new();
    for label in ENCODING_PROBES {
        let resolved = super::common::resolve_encoding(label).map(encoding_rs::Encoding::name);
        m.insert((*label).to_string(), json!(resolved));
    }
    Value::Object(m)
}

/// Clap's own view of one argument: its long name, whether it takes a value, and
/// the closed value-set if it has one (`--on-type-clash <error|widen|promote>`).
/// Positionals have no long name and are reported by their value name instead, so a
/// launcher that forgets to accept an argument is still visible to the diff.
fn arg_json(a: &clap::Arg) -> Option<Value> {
    let name = match a.get_long() {
        Some(long) => format!("--{long}"),
        // A positional. Report it as <NAME> so the census can still compare arity.
        None if a.is_positional() => format!("<{}>", a.get_id()),
        None => return None, // short-only (there are none today) — nothing to diff.
    };
    let values: Vec<String> = a
        .get_possible_values()
        .iter()
        .map(|p| p.get_name().to_string())
        .collect();
    // Ask the ACTION, not `get_num_args()`. Arity is `None` unless a `num_args` was
    // set explicitly — clap infers it from the action — so asking the arity reported
    // `takes_value: false` for `--dict-version`, a flag that plainly takes one. The
    // other launchers must know which flags eat the next token to parse at all, so a
    // wrong answer here is not a cosmetic blemish: it is the column they are diffed
    // against.
    let takes_value = matches!(
        a.get_action(),
        clap::ArgAction::Set | clap::ArgAction::Append
    );
    Some(json!({
        "name": name,
        "takes_value": takes_value,
        "required": a.is_required_set(),
        // Only present when clap enforces a closed set — the census diffs these
        // against the other launchers, which is how a stale value enum surfaces.
        "values": values,
    }))
}

/// The parser, as JSON: `{surface, verbs: [{verb, args: [...]}], global_args: [...]}`.
/// Shape is shared with `laterite._cli.census()` and `cli.ts census()`, so
/// `tools/gen_census.py` can diff the three without per-surface special-casing.
pub fn census_json() -> Value {
    let cmd = Cli::command();

    let verbs: Vec<Value> = cmd
        .get_subcommands()
        // `help` is clap's own, not ours — it is not a door the launchers must mirror.
        .filter(|s| s.get_name() != "help")
        .map(|s| {
            let mut args: Vec<Value> = s.get_arguments().filter_map(arg_json).collect();
            args.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            json!({ "verb": s.get_name(), "args": args })
        })
        .collect();

    let mut global: Vec<Value> = cmd.get_arguments().filter_map(arg_json).collect();
    global.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

    json!({
        // Bump whenever a TABLE is added. The generator refuses an older dump rather
        // than reading its missing tables as empty — a stale-but-answering launcher
        // reporting `editions: []` looks exactly like "no drift", which would be a
        // gate that disarms itself. (Found the hard way: a release `lat` built one
        // commit earlier answered `census` fine and reported no editions at all.)
        "census_version": CENSUS_VERSION,
        "surface": "cli-native",
        "authority": true,
        "verbs": verbs,
        "global_args": global,
        // The USER-FACING list — what README-cli.md documents and what the other two
        // launchers are expected to implement. Deliberately excludes hidden machine
        // doors (`census`), which is why it is not simply `verbs`. Pinned equal to
        // clap's visible verbs by `subcommands_const_is_faithful`.
        "documented_verbs": crate::cli::SUBCOMMANDS,
        // The bundled dictionary editions, from the GENERATED `DictVersion::ALL`.
        // The second census table: every launcher must accept exactly this set for
        // `--dict-version`. It used to be hand-copied about nine times across the
        // tree — and this binary's own copy was the worst of them, because the
        // rejection MESSAGE was generated from this list while the match arms that
        // did the accepting were not. A new edition would have shipped a CLI that
        // rejects `4.3` with a message advertising `4.3`.
        "editions": laterite_ags4_validator::dict::DictVersion::ALL
            .iter().map(|v| v.as_str()).collect::<Vec<_>>(),
        "fallback_edition": laterite_ags4_validator::dict::FALLBACK.as_str(),
        // The third table: what each launcher makes of an encoding label. A `null`
        // for `cp1252x` is the POLICY PIN — an unknown label must be refused, never
        // quietly decoded as UTF-8.
        "encodings": encodings_json(),
    })
}

pub fn run() -> ! {
    println!(
        "{}",
        serde_json::to_string_pretty(&census_json()).unwrap_or_default()
    );
    std::process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The census must see every verb the tool actually dispatches. This is the
    /// assertion that makes the census an AUTHORITY rather than a third hand-list:
    /// it is derived from clap, and clap is what parses the real command line.
    #[test]
    fn census_reflects_every_clap_subcommand() {
        let c = census_json();
        let verbs: Vec<&str> = c["verbs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["verb"].as_str().unwrap())
            .collect();
        for expected in [
            "validate", "read", "fix", "diff", "merge", "certify", "rules",
        ] {
            assert!(
                verbs.contains(&expected),
                "census lost {expected}: {verbs:?}"
            );
        }
        assert!(
            !verbs.contains(&"help"),
            "clap's own `help` is not our door"
        );
    }

    /// The user-facing `SUBCOMMANDS` const (what the README documents, and what
    /// `test_wiki_cli_faithful` gates the guide against) must be exactly clap's
    /// VISIBLE verbs — no more, no less.
    ///
    /// This closes the THIRD hand-list. `SUBCOMMANDS` used to drive the
    /// default-subcommand pre-scan as well, so adding a hidden verb made
    /// `lat census` parse as `lat validate census`. The pre-scan now asks clap
    /// directly (`main.rs::with_default_subcommand`); this const answers the
    /// separate "what do we document" question and is pinned here.
    #[test]
    fn subcommands_const_is_faithful() {
        let visible: Vec<String> = Cli::command()
            .get_subcommands()
            .filter(|s| !s.is_hide_set() && s.get_name() != "help")
            .map(|s| s.get_name().to_string())
            .collect();
        let declared: Vec<String> = crate::cli::SUBCOMMANDS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            declared, visible,
            "cli::SUBCOMMANDS has drifted from clap's visible verbs — a verb added to \
             the Commands enum never reached the const (or vice versa)"
        );
        assert!(
            !declared.contains(&"census".to_string()),
            "census is a hidden machine door; it must not enter the user-facing list"
        );
    }

    /// `takes_value` must say which flags eat the next token — it is the column the
    /// other two launchers' parsers are diffed against, and a launcher that gets it
    /// wrong mis-parses the command line rather than merely mis-reporting it.
    ///
    /// Pinned because the first implementation asked `get_num_args()`, which clap
    /// leaves `None` unless someone set an explicit arity: every valued flag in the
    /// tool reported `takes_value: false`.
    #[test]
    fn census_knows_which_flags_take_a_value() {
        let c = census_json();
        let validate = c["verbs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["verb"] == "validate")
            .expect("validate is a verb");
        let arg = |n: &str| -> bool {
            validate["args"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["name"] == n)
                .unwrap_or_else(|| panic!("validate has {n}"))["takes_value"]
                .as_bool()
                .unwrap()
        };
        for valued in ["--dict-version", "--dict", "--encoding", "--index", "--out"] {
            assert!(arg(valued), "{valued} takes a value");
        }
        for boolean in ["--no-warnings", "--show-fyi", "--check-files"] {
            assert!(!arg(boolean), "{boolean} is a bare switch");
        }
    }

    /// A closed value-set must survive the reflection — this is how the census
    /// notices that one launcher still offers a retired mode (or lacks a new one).
    #[test]
    fn census_carries_closed_value_sets() {
        let c = census_json();
        let merge = c["verbs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["verb"] == "merge")
            .expect("merge is a verb");
        let clash = merge["args"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "--on-type-clash")
            .expect("merge has --on-type-clash");
        let values: Vec<&str> = clash["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(values, ["error", "widen", "promote"]);
    }

    /// The `encodings` table and the positional args must survive reflection.
    ///
    /// `encodings` is a load-bearing census output: the `cp1252x` policy pin (an
    /// unknown label must resolve to `null`, never a silent UTF-8 fallback) and the
    /// `latin9` alias that only the binary once accepted. Positional arguments (a
    /// verb's `<file>`) are reflected as `<NAME>` so a launcher that drops one is
    /// still visible to the diff. Neither had an assertion — a blanked `encodings`
    /// table or a dropped-positionals arm went uncaught.
    #[test]
    fn census_carries_the_encodings_table_and_positional_args() {
        let c = census_json();

        // the encodings table: a known label resolves, an alias only the binary
        // once knew resolves, and the cp1252x typo is refused (the policy pin).
        let enc = &c["encodings"];
        assert_eq!(enc["utf-8"], "UTF-8");
        assert_eq!(enc["latin9"], "ISO-8859-15");
        assert_eq!(
            enc["cp1252x"],
            Value::Null,
            "an unknown label must resolve to null, not a silent fallback"
        );

        // positionals are reflected as <NAME> — validate's is present, not dropped.
        let validate = c["verbs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["verb"] == "validate")
            .expect("validate is a verb");
        let names: Vec<&str> = validate["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert!(
            names.iter().any(|n| n.starts_with('<') && n.ends_with('>')),
            "validate's positional argument is missing from the census: {names:?}"
        );
    }
}
