//! `lat-db diff <a> <b> [--samples N]` — per-group row delta between two
//! `.ags5db` files.
//!
//! Thin CLI shim over `laterite_ags5_db::diff::diff_dbs` (lib-ified in F2a-2d).
//! The algorithm lives in the lib; this file owns the clap args + JSON
//! payload shape + table-mode rendering + exit-code mapping.
//!
//! Exit code: 0 if no changes, **1 if any group has added/removed/modified
//! rows or appears in only one file**. The exit-1 signal is load-bearing
//! for CI / agent contracts ("does delivery 2 differ from delivery 1?")
//! without parsing the JSON.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::Ctx;
use crate::output::{OutputMode, render_record};
use laterite_ags5_db::diff::{self, GroupDiff};
use laterite_ags5_db::error::CliError;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Baseline .ags5db path
    pub a: PathBuf,

    /// Comparison .ags5db path
    pub b: PathBuf,

    /// Show up to N sample KEY tuples per change category (default 3)
    #[arg(long, default_value_t = 3)]
    pub samples: usize,
}

#[derive(Serialize, Clone)]
struct ChangedGroupRecord {
    code: String,
    added: usize,
    removed: usize,
    modified: usize,
    unchanged: usize,
}

#[derive(Serialize)]
struct DiffPayload {
    changed_groups: Vec<ChangedGroupRecord>,
    groups_only_in_a: Vec<String>,
    groups_only_in_b: Vec<String>,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<i32> {
    if !args.a.exists() {
        return Err(CliError::FileNotFound(args.a.display().to_string()).into());
    }
    if !args.b.exists() {
        return Err(CliError::FileNotFound(args.b.display().to_string()).into());
    }

    let result = diff::diff_dbs(&args.a, &args.b, args.samples)?;
    let has_diff = result.has_changes();

    let changed_records: Vec<ChangedGroupRecord> = result
        .changed_groups
        .iter()
        .map(|gd| ChangedGroupRecord {
            code: gd.code.clone(),
            added: gd.added,
            removed: gd.removed,
            modified: gd.modified,
            unchanged: gd.unchanged,
        })
        .collect();

    if ctx.mode == OutputMode::Table {
        render_table(
            &changed_records,
            &result.changed_groups,
            &result.groups_only_in_a,
            &result.groups_only_in_b,
            args.samples,
        );
    } else {
        let payload = DiffPayload {
            changed_groups: changed_records,
            groups_only_in_a: result.groups_only_in_a,
            groups_only_in_b: result.groups_only_in_b,
        };
        render_record(&payload, ctx.mode)?;
    }

    Ok(if has_diff { 1 } else { 0 })
}

fn render_table(
    changed: &[ChangedGroupRecord],
    full: &[GroupDiff],
    only_in_a: &[String],
    only_in_b: &[String],
    samples: usize,
) {
    if changed.is_empty() && only_in_a.is_empty() && only_in_b.is_empty() {
        println!("No differences.");
        return;
    }
    if !changed.is_empty() {
        let width = changed.iter().map(|g| g.code.len()).max().unwrap_or(4);
        for g in changed {
            println!(
                "  {code:<width$}  +{added:<4}  -{removed:<4}  ~{modified:<4}  ({unchanged} unchanged)",
                code = g.code,
                width = width,
                added = g.added,
                removed = g.removed,
                modified = g.modified,
                unchanged = g.unchanged,
            );
        }
    }
    if !only_in_a.is_empty() {
        println!(
            "\nGroups only in A ({}): {}",
            only_in_a.len(),
            only_in_a.join(", "),
        );
    }
    if !only_in_b.is_empty() {
        println!(
            "\nGroups only in B ({}): {}",
            only_in_b.len(),
            only_in_b.join(", "),
        );
    }
    if samples > 0 {
        for gd in full {
            if gd.sample_added.is_empty()
                && gd.sample_removed.is_empty()
                && gd.sample_modified.is_empty()
            {
                continue;
            }
            println!("\n  {} samples:", gd.code);
            for k in &gd.sample_added {
                println!("    +  {}", format_key_tuple(k));
            }
            for k in &gd.sample_removed {
                println!("    -  {}", format_key_tuple(k));
            }
            for k in &gd.sample_modified {
                println!("    ~  {}", format_key_tuple(k));
            }
        }
    }
}

fn format_key_tuple(values: &[Value]) -> String {
    let parts: Vec<String> = values
        .iter()
        .map(|v| match v {
            Value::Null => "NULL".to_string(),
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect();
    format!("({})", parts.join(", "))
}
