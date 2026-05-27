//! Generate DuckDB DDL from the registry — port of `ags5_db._ddl`.
//!
//! `build_ddl()` emits, in order:
//!   * one CREATE TABLE per group (own KEYs + non-KEYs + _content_hash)
//!   * indexes per Phase 6.5.2 rules
//!   * one CREATE VIEW per group (JOINs inherited KEYs from ancestors)
//!   * the generic blob table + index
//!
//! Output is byte-comparable to Python's `build_ddl()` on the same
//! registry — the migrate parity test relies on identical SQL so DuckDB
//! produces identical files.

use std::collections::HashSet;

use crate::ags_types::sql_type;
use crate::registry::{
    GroupDescriptor, Registry, ancestor_chain, heading_storage_index, inherited_key_names,
};

fn table_ddl(reg: &Registry, g: &GroupDescriptor) -> String {
    let inherited = inherited_key_names(reg, g);
    let mut cols: Vec<String> = vec!["id UUID PRIMARY KEY".into(), "parent_id UUID".into()];
    for h in g.key_headings() {
        if inherited.contains(&h.name) {
            continue;
        }
        cols.push(format!("{} {}", h.name, sql_type(&h.ags_type)));
    }
    for h in g.non_key_headings() {
        cols.push(format!("{} {}", h.name, sql_type(&h.ags_type)));
    }
    cols.push("_content_hash VARCHAR".into());
    let body = cols.join(",\n    ");
    format!(
        "CREATE TABLE IF NOT EXISTS {} (\n    {}\n);",
        g.table(),
        body,
    )
}

fn index_ddl(reg: &Registry, g: &GroupDescriptor) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    let want_parent = g.index_parent.unwrap_or(g.parent.is_some());
    if want_parent && g.parent.is_some() {
        out.push(format!(
            "CREATE INDEX IF NOT EXISTS idx_{table}_parent ON {table}(parent_id);",
            table = g.table(),
        ));
    }

    let inherited = inherited_key_names(reg, g);
    for h in g.key_headings() {
        if inherited.contains(&h.name) {
            continue;
        }
        let want = h
            .indexed
            .unwrap_or(h.is_key() && h.ags_type.eq_ignore_ascii_case("ID"));
        if want {
            out.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_{table}_{col} ON {table}({col_orig});",
                table = g.table(),
                col = h.name.to_lowercase(),
                col_orig = h.name,
            ));
        }
    }
    out
}

fn view_ddl(reg: &Registry, g: &GroupDescriptor) -> String {
    let chain = ancestor_chain(reg, &g.code);

    // Storage index per KEY heading.
    let storage: Vec<(String, usize)> = g
        .key_headings()
        .map(|h| (h.name.clone(), heading_storage_index(reg, g, &h.name)))
        .collect();
    let max_depth = storage.iter().map(|(_, i)| *i).max().unwrap_or(0);

    let mut select_lines: Vec<String> = Vec::new();
    select_lines.push("    t0.id AS id".into());
    select_lines.push("    t0.parent_id".into());
    select_lines.push("    t0._content_hash".into());

    let storage_map: std::collections::HashMap<&str, usize> =
        storage.iter().map(|(n, i)| (n.as_str(), *i)).collect();
    for h in g.key_headings() {
        let idx = *storage_map.get(h.name.as_str()).unwrap_or(&0);
        select_lines.push(format!("    t{}.{} AS {}", idx, h.name, h.py_name()));
    }
    for h in g.non_key_headings() {
        select_lines.push(format!("    t0.{} AS {}", h.name, h.py_name()));
    }

    let mut from_clause = format!("{} t0", g.table());
    for (i, parent) in chain.iter().enumerate().take(max_depth + 1).skip(1) {
        from_clause.push_str(&format!(
            "\nJOIN {} t{} ON t{}.id = t{}.parent_id",
            parent.table(),
            i,
            i,
            i - 1,
        ));
    }

    let body = select_lines.join(",\n");
    format!(
        "CREATE OR REPLACE VIEW {} AS\nSELECT\n{}\nFROM {};",
        g.view(),
        body,
        from_clause,
    )
}

fn blob_ddl() -> &'static str {
    "CREATE SEQUENCE IF NOT EXISTS seq_blob;
CREATE TABLE IF NOT EXISTS blob (
    id           BIGINT PRIMARY KEY DEFAULT nextval('seq_blob'),
    parent_table VARCHAR NOT NULL,
    parent_id    VARCHAR NOT NULL,
    kind         VARCHAR NOT NULL,
    mime_type    VARCHAR,
    filename     VARCHAR,
    sha256       VARCHAR,
    data         BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_blob_parent ON blob(parent_table, parent_id);
"
}

/// Emit the full DDL bundle for the in-process registry.
///
/// Order matches Python: tables (all), indexes (all), views (all), blob.
/// Tables come before indexes so the indexes have something to attach to;
/// views come last because they reference the typed tables.
pub fn build_ddl(reg: &Registry) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push("-- generated from ags5_models registry".into());
    for g in reg.iter() {
        parts.push(table_ddl(reg, g));
    }
    for g in reg.iter() {
        parts.extend(index_ddl(reg, g));
    }
    for g in reg.iter() {
        parts.push(view_ddl(reg, g));
    }
    parts.push(blob_ddl().into());
    parts.join("\n\n")
}

/// Just the inherited-key set for a group — handy for callers that
/// don't want the full DDL but need to skip inherited KEYs when reading
/// or writing. Re-export so `commands/migrate.rs` can use the same
/// canonical implementation.
pub fn inherited(reg: &Registry, g: &GroupDescriptor) -> HashSet<String> {
    inherited_key_names(reg, g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::registry;

    #[test]
    fn proj_table_has_expected_columns() {
        let reg = registry();
        let proj = reg.get("PROJ").unwrap();
        let ddl = table_ddl(reg, proj);
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS g_proj"));
        assert!(ddl.contains("id UUID PRIMARY KEY"));
        assert!(ddl.contains("PROJ_ID VARCHAR"));
        assert!(ddl.contains("_content_hash VARCHAR"));
    }

    #[test]
    fn samp_drops_inherited_loca_id() {
        let reg = registry();
        let samp = reg.get("SAMP").unwrap();
        let ddl = table_ddl(reg, samp);
        // LOCA_ID is inherited from LOCA — must not appear as a typed
        // column on g_samp. The view re-exposes it via the JOIN chain.
        assert!(
            !ddl.contains("LOCA_ID VARCHAR"),
            "g_samp should not have a LOCA_ID column:\n{}",
            ddl,
        );
    }

    #[test]
    fn view_joins_parent_for_inherited_keys() {
        let reg = registry();
        let samp = reg.get("SAMP").unwrap();
        let view = view_ddl(reg, samp);
        // SAMP view JOINs g_loca so the LOCA_ID column resolves there.
        assert!(view.contains("JOIN g_loca"));
    }
}
