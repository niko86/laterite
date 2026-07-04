//! Frame a typed AGS4 group as a single-batch Arrow **IPC stream**.
//!
//! `laterite-node` (→ napi `Buffer`) and `laterite-ags4-wasm` (→ `Uint8Array`
//! for duckdb-wasm) both turn one group into an Arrow IPC stream the exact same
//! way: build the typed columns via [`crate::arrow_cols::build_record_batch`],
//! then wrap the single batch in a `StreamWriter`. That composition lived
//! verbatim in both crates; it lives here once instead.
//!
//! Parser-agnostic by construction — the caller passes headings, AGS type
//! codes, the row count, and a positional `cell(col, row)` accessor — so
//! `laterite-types` stays the parser-free wasm-safe leaf (it gains no
//! dependency on whatever `ParsedGroup` the hosts happen to hold).

use arrow::error::ArrowError;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

use crate::arrow_cols::{build_record_batch, build_record_batch_with_ids};

/// Build one group's typed [`RecordBatch`] (via [`build_record_batch`]) and
/// frame it as a single-batch Arrow IPC stream. `cell(col, row)` returns the
/// raw string for a cell, or `None` for a short/ragged row (→ null).
pub fn build_group_ipc<'a, F>(
    headings: &[String],
    ags_types: &[String],
    n_rows: usize,
    cell: F,
) -> Result<Vec<u8>, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let batch = build_record_batch(headings, ags_types, n_rows, cell)?;
    record_batch_to_ipc(&batch)
}

/// Like [`build_group_ipc`], but the batch carries the two content-addressed key
/// columns `_id` / `_parent_id` first (see [`build_record_batch_with_ids`]) —
/// the IPC the node/wasm relational path frames when the keys travel with the
/// data. `ids[row]` is the `(_id, _parent_id)` pair the host computed via
/// `laterite_ags4_core::keychain::group_row_ids`.
pub fn build_group_ipc_with_ids<'a, F>(
    ids: &[(String, Option<String>)],
    headings: &[String],
    ags_types: &[String],
    n_rows: usize,
    cell: F,
) -> Result<Vec<u8>, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let batch = build_record_batch_with_ids(ids, headings, ags_types, n_rows, cell)?;
    record_batch_to_ipc(&batch)
}

/// Frame a [`RecordBatch`] as a single-batch Arrow IPC stream (`Vec<u8>`).
/// Pure Arrow — split out so a caller that already holds a batch can reuse the
/// exact framing.
pub fn record_batch_to_ipc(batch: &RecordBatch) -> Result<Vec<u8>, ArrowError> {
    let mut buf = Vec::new();
    let mut writer = StreamWriter::try_new(&mut buf, &batch.schema())?;
    writer.write(batch)?;
    writer.finish()?;
    drop(writer);
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, Float64Array, StringArray};
    use arrow::ipc::reader::StreamReader;

    #[test]
    fn build_group_ipc_round_trips_typed_columns() {
        // LOCA_ID (ID → Utf8) + LOCA_GL (2DP → Float64).
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let types = vec!["ID".to_string(), "2DP".to_string()];
        let rows = [["BH01", "12.34"], ["BH02", "56.78"]];

        let ipc = build_group_ipc(&headings, &types, rows.len(), |col, row| {
            rows.get(row).and_then(|r| r.get(col)).copied()
        })
        .expect("frames");

        let mut reader = StreamReader::try_new(std::io::Cursor::new(ipc), None).expect("reader");
        let batch = reader.next().expect("a batch").expect("ok");
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 2);
        let id = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("ID column is Utf8");
        assert_eq!(id.value(0), "BH01");
        let gl = batch
            .column(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("2DP column is Float64");
        assert!((gl.value(1) - 56.78).abs() < 1e-9);
    }

    #[test]
    fn build_group_ipc_with_ids_prepends_key_columns() {
        // _id (col 0) + _parent_id (col 1, NULL when None), then the typed
        // headings shifted by two — the duckdb-extension column recipe.
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let types = vec!["ID".to_string(), "2DP".to_string()];
        let rows = [["BH01", "12.34"], ["BH02", "56.78"]];
        let ids = vec![
            ("id-a".to_string(), Some("parent-a".to_string())),
            ("id-b".to_string(), None), // a root row → NULL _parent_id
        ];

        let ipc = build_group_ipc_with_ids(&ids, &headings, &types, rows.len(), |col, row| {
            rows.get(row).and_then(|r| r.get(col)).copied()
        })
        .expect("frames");

        let mut reader = StreamReader::try_new(std::io::Cursor::new(ipc), None).expect("reader");
        let batch = reader.next().expect("a batch").expect("ok");
        assert_eq!(batch.num_columns(), 4, "_id + _parent_id + 2 headings");
        assert_eq!(batch.schema().field(0).name(), "_id");
        assert_eq!(batch.schema().field(1).name(), "_parent_id");

        let id = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("_id is Utf8");
        assert_eq!(id.value(0), "id-a");
        assert_eq!(id.value(1), "id-b");

        let pid = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("_parent_id is Utf8");
        assert_eq!(pid.value(0), "parent-a");
        assert!(pid.is_null(1), "None _parent_id → NULL cell");

        // The heading columns are still correctly typed, just shifted by two.
        let gl = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("2DP heading still Float64 at col 3");
        assert!((gl.value(0) - 12.34).abs() < 1e-9);
    }
}
