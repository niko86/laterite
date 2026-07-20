// Shared types + cell coercion for the typed-data (Explore / Tools) path.
//
// The Rust `read()` (laterite-ags4-wasm) hands back one typed Arrow IPC stream per
// group plus a per-group `meta`. DuckDB-wasm ingests the IPC and queries it;
// the values it reads back need careful coercion before they touch the DOM
// (see formatCell). This module holds the contract so the worker, the
// DuckDB layer, and every grid agree on shapes — even before DuckDB lands
// (PR-2): in PR-1 only the worker/client use GroupMeta.

/** Per-group schema, as assembled from the wasm `ParsedDataset.meta(code)`
 *  (`{headings, units, types, sql_types}`) with its `code` attached. */
export interface GroupMeta {
  /** The 4-letter AGS group code (PROJ, LOCA, SAMP, …) — also the table
   *  name the group is ingested under in DuckDB. */
  code: string;
  headings: string[];
  units: string[];
  /** AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID"). */
  types: string[];
  /** The DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
   *  "TIMESTAMP", "VARCHAR", …) — drives both the schema panel and the
   *  per-column cell formatting below. */
  sql_types: string[];
}

/** Coerce a non-null scalar cell value to a display string. AGS cells are
 *  primitives; an unexpected object falls back to JSON so nothing reaches the
 *  DOM as "[object Object]". `null`/`undefined` → empty string. */
export function scalarText(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint"
  )
    return String(value);
  return JSON.stringify(value);
}

/** Coerce one cell value (as Arrow JS / DuckDB-wasm hands it back) to a
 *  display string, driven by the column's DuckDB sql_type. Two values must
 *  never reach the DOM raw:
 *   - Int64 / BIGINT come back as a JS `bigint` (rendering one in JSX, or
 *     `JSON.stringify`-ing it, throws);
 *   - TIMESTAMP comes back as an integer count of MICROSECONDS since the
 *     epoch (not a `Date` — DuckDB-wasm issue #393).
 *  This is the single audited place those are normalised; every grid runs
 *  cells through it. `null`/`undefined` → empty string (an em-dash is
 *  applied at the view layer if a placeholder is wanted). */
export function formatCell(value: unknown, sqlType: string): string {
  if (value === null || value === undefined) return "";
  const t = sqlType.toUpperCase();

  if (t.startsWith("TIMESTAMP") || t.startsWith("DATE")) {
    // micros-since-epoch (bigint or number), or already a Date.
    const micros =
      typeof value === "bigint"
        ? Number(value)
        : typeof value === "number"
          ? value
          : value instanceof Date
            ? value.getTime() * 1000
            : null;
    if (micros !== null && Number.isFinite(micros)) {
      // "YYYY-MM-DD HH:MM:SS" (drop the ISO 'T'/'Z' for a flatter look;
      // tz-naive, matching the Rust cast).
      return new Date(micros / 1000)
        .toISOString()
        .replace("T", " ")
        .replace(/\.\d+Z$/, "")
        .replace("Z", "");
    }
    return scalarText(value);
  }

  return scalarText(value);
}
