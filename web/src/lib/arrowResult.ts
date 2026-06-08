// Turn a DuckDB-wasm query result (an Apache Arrow Table) into plain
// {columns, rows} of display strings — the shape every grid renders. Lives
// behind the DuckDB lazy boundary (dynamically imported), so apache-arrow's
// runtime (the DataType guards) stays out of the entry chunk.
//
// Crucially this is where the two un-renderable values are tamed: Int64/
// BIGINT come back as JS bigint, and TIMESTAMP as a micros integer (not a
// Date) — formatCell() handles both, driven by the column's sql_type, which
// we derive from the Arrow field type so this works for ANY query (the SQL
// console, not just SELECT * over a known group).

import { DataType, type Field, type Table } from "apache-arrow";
import { formatCell } from "./duckTypes";

export interface ResultColumn {
  name: string;
  /** A coarse DuckDB-ish type label, derived from the Arrow field type —
   *  enough to drive formatCell + a column-type hint in the header. */
  sqlType: string;
}

export interface ResultSet {
  columns: ResultColumn[];
  /** Pre-formatted display strings (formatCell already applied per column).
   *  May be capped for display — see `total`. */
  rows: string[][];
  /** Full row count of the underlying result (≥ rows.length when capped). */
  total: number;
}

function arrowSqlType(field: Field): string {
  const t = field.type;
  if (DataType.isTimestamp(t)) return "TIMESTAMP";
  if (DataType.isDate(t)) return "DATE";
  if (DataType.isBool(t)) return "BOOLEAN";
  if (DataType.isFloat(t)) return "DOUBLE";
  if (DataType.isInt(t)) {
    const bits = (t as { bitWidth?: number }).bitWidth ?? 32;
    return bits >= 64 ? "BIGINT" : "INTEGER";
  }
  return "VARCHAR";
}

export function arrowResult(table: Table, cap?: number): ResultSet {
  const columns: ResultColumn[] = table.schema.fields.map((f) => ({
    name: f.name,
    sqlType: arrowSqlType(f),
  }));
  const total = table.numRows;
  // Materialise at most `cap` rows. The DuckDB query still runs in full (the
  // Arrow result is columnar + compact), but the EXPENSIVE part — formatCell
  // per cell into JS strings, then a DOM node per cell — is bounded, instead of
  // paid for every one of tens of thousands of rows (a multi-second freeze /
  // OOM risk on weak hardware). The grid shows "first N of total"; export
  // re-runs the query uncapped.
  const limit = cap != null && cap < total ? cap : total;
  const src = limit < total ? table.slice(0, limit) : table;
  const rows: string[][] = [];
  // toArray() yields row proxies keyed by column name.
  for (const r of src.toArray()) {
    const row = r as Record<string, unknown>;
    rows.push(columns.map((c) => formatCell(row[c.name], c.sqlType)));
  }
  return { columns, rows, total };
}
