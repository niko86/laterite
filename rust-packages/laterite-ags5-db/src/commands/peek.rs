//! `lat-db peek <db> <group>` — safe row browser over `v_<group>`.
//!
//! Mirrors Python `_cmd_peek`. The query (field/where/limit/offset +
//! optional null-column drop) lives in `laterite_ags5_db::query::peek`; this shim
//! owns the *presentation*:
//!
//!   1. `--drop-null-cols` (all modes) — handed to the lib.
//!   2. In TABLE mode with no `--fields`, ALSO auto-drop nulls AND cap
//!      visible columns at `--max-cols` (default 8) so the table fits.
//!   3. Canonical-type sub-headers (TABLE only) + stderr hints so silent
//!      column loss can't surprise users.

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Args;

use crate::Ctx;
use crate::output::{OutputMode, Rows, render_rows};
use crate::progress;
use laterite_ags5_db::conn::open_readonly;
use laterite_ags5_db::db::type_labels_for;
use laterite_ags5_db::query::{all_null_columns, drop_columns, peek};

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Group code (e.g. LOCA)
    pub group: String,

    /// Comma-separated heading names (default: all)
    #[arg(long)]
    pub fields: Option<String>,

    /// 'field<op>value' filter; repeatable (ANDed)
    #[arg(long)]
    pub r#where: Vec<String>,

    /// Max rows to return (default 50)
    #[arg(long, default_value_t = 50)]
    pub limit: usize,

    /// Skip the first N rows (default 0)
    #[arg(long, default_value_t = 0)]
    pub offset: usize,

    /// In table mode (no --fields), cap visible columns at N (default 8; 0 = no cap)
    #[arg(long, default_value_t = 8)]
    pub max_cols: usize,

    /// Show every populated column (alias for --max-cols 0)
    #[arg(long, conflicts_with = "max_cols")]
    pub all_cols: bool,

    /// Drop columns where every returned row is NULL (auto-on in table mode)
    #[arg(long)]
    pub drop_null_cols: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let user_picked_fields = args.fields.is_some();
    let table_default = !user_picked_fields && ctx.mode == OutputMode::Table;

    // Data side. We pass drop_null_cols=false and do the trim here,
    // because the table-default auto-drop + the stderr hint need the
    // command's OutputMode + the which-columns-were-hidden context.
    let mut rows = peek(
        &args.db,
        &args.group,
        args.fields.as_deref(),
        &args.r#where,
        args.limit,
        args.offset,
        false,
    )?;

    let max_cols = if args.all_cols { 0 } else { args.max_cols };
    let drop_nulls = args.drop_null_cols || table_default;

    let mut hidden_null: Vec<String> = Vec::new();
    if drop_nulls && !rows.records.is_empty() {
        hidden_null = all_null_columns(&rows);
        if !hidden_null.is_empty() {
            drop_columns(&mut rows, &hidden_null);
        }
    }

    let mut hidden_capped: Vec<String> = Vec::new();
    if table_default && max_cols > 0 && rows.columns.len() > max_cols {
        hidden_capped = rows.columns[max_cols..].to_vec();
        cap_columns(&mut rows, max_cols);
    }

    if !hidden_null.is_empty() && !table_default {
        let preview = hidden_null
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if hidden_null.len() > 5 { ", ..." } else { "" };
        progress(
            &format!(
                "dropped {} all-null column(s): {}{}",
                hidden_null.len(),
                preview,
                suffix,
            ),
            ctx.quiet,
        );
    }
    if !hidden_capped.is_empty() {
        progress(
            &format!(
                "showing {} of {} populated columns; pass --max-cols 0 to show all, or --fields a,b,c to pick.",
                max_cols,
                max_cols + hidden_capped.len(),
            ),
            ctx.quiet,
        );
    }

    // TABLE mode gets canonical-type sub-headers; re-open read-only just
    // for the labels (the lib closed its connection). Other modes don't
    // need them, so they pay nothing.
    let labels = if ctx.mode == OutputMode::Table {
        let conn = open_readonly(&args.db)?;
        let code = args.group.to_uppercase();
        Some(type_labels_for(&conn, &code, &rows.columns)?)
    } else {
        None
    };
    render_rows(&rows, ctx.mode, labels.as_ref())?;
    Ok(())
}

fn cap_columns(rows: &mut Rows, max: usize) {
    let keep: HashSet<String> = rows.columns.iter().take(max).cloned().collect();
    rows.columns.truncate(max);
    for rec in &mut rows.records {
        let to_remove: Vec<String> = rec.keys().filter(|k| !keep.contains(*k)).cloned().collect();
        for k in to_remove {
            rec.remove(&k);
        }
    }
}
