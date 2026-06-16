//! `lat-db inspect <db> [--group X]` — dump the `_spec_*` self-describing
//! tables.
//!
//! Thin CLI shim over `laterite_ags5_db::introspect::inspect` (lib-ified in F2a-2f).
//! Owns clap args + the labeled-scalar table-mode rendering + the NDJSON
//! payload shape; the data work lives in the lib.
//!
//! Phase 6.5.2 compat: `_spec_groups.index_parent` and
//! `_spec_headings.indexed` columns only exist on files written by ≥6.5.2
//! — the lib handles the detection; the JSON shape uses
//! `Option<Option<T>>` to distinguish absent (pre-6.5.2) from null.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::Ctx;
use crate::output::{OutputMode, Rows, render_record, render_rows};
use laterite_ags5_db::introspect;
use serde_json::Map;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Restrict to one group code (e.g. LOCA)
    #[arg(long)]
    pub group: Option<String>,
}

#[derive(Serialize)]
struct InspectPayload {
    format_version: String,
    library_version: String,
    written_at: String,
    note: String,
    n_groups: i64,
    n_headings: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<GroupBlockOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headings: Option<Vec<HeadingRecordOut>>,
}

#[derive(Serialize)]
struct GroupBlockOut {
    code: String,
    contents: String,
    parent: String,
    is_high_volume: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    index_parent: Option<Option<String>>,
}

#[derive(Serialize)]
struct HeadingRecordOut {
    name: String,
    status: String,
    ags_type: String,
    canonical_type: String,
    unit: String,
    display_hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed: Option<Option<bool>>,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let report = introspect::inspect(&args.db, args.group.as_deref())?;

    let payload = InspectPayload {
        format_version: report.format_version,
        library_version: report.library_version,
        written_at: report.written_at,
        note: report.note,
        n_groups: report.n_groups,
        n_headings: report.n_headings,
        group: report.group.map(|g| GroupBlockOut {
            code: g.code,
            contents: g.contents,
            parent: g.parent,
            is_high_volume: g.is_high_volume,
            index_parent: g.index_parent,
        }),
        headings: report.headings.map(|hs| {
            hs.into_iter()
                .map(|h| HeadingRecordOut {
                    name: h.name,
                    status: h.status,
                    ags_type: h.ags_type,
                    canonical_type: h.canonical_type,
                    unit: h.unit,
                    display_hint: h.display_hint,
                    indexed: h.indexed,
                })
                .collect()
        }),
    };

    if ctx.mode == OutputMode::Table {
        println!("format_version  {}", payload.format_version);
        println!("library_version {}", payload.library_version);
        if !payload.written_at.is_empty() {
            println!("written_at      {}", payload.written_at);
        }
        if !payload.note.is_empty() {
            println!("note            {}", payload.note);
        }
        println!("groups          {}", payload.n_groups);
        println!("headings        {}", payload.n_headings);
        if let Some(group) = &payload.group {
            println!();
            println!("=== {} ===", group.code);
            println!("  contents       {}", group.contents);
            println!("  parent         {}", group.parent);
            println!("  is_high_volume {}", group.is_high_volume);
            if let Some(ip) = &group.index_parent {
                println!("  index_parent   {}", ip.as_deref().unwrap_or("(null)"),);
            }
            println!();
        }
        if let Some(headings) = &payload.headings {
            let columns = vec![
                "name".into(),
                "status".into(),
                "ags_type".into(),
                "canonical_type".into(),
                "unit".into(),
                "display_hint".into(),
            ];
            let mut records: Vec<Map<String, Value>> = Vec::with_capacity(headings.len());
            for h in headings {
                let mut rec = Map::new();
                rec.insert("name".into(), Value::from(h.name.clone()));
                rec.insert("status".into(), Value::from(h.status.clone()));
                rec.insert("ags_type".into(), Value::from(h.ags_type.clone()));
                rec.insert(
                    "canonical_type".into(),
                    Value::from(h.canonical_type.clone()),
                );
                rec.insert("unit".into(), Value::from(h.unit.clone()));
                rec.insert("display_hint".into(), Value::from(h.display_hint.clone()));
                records.push(rec);
            }
            render_rows(&Rows { columns, records }, OutputMode::Table, None)?;
        }
        return Ok(());
    }

    render_record(&payload, ctx.mode)?;
    Ok(())
}
