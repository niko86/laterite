// The optional DuckDB engine — the Node port of laterite-py's lazy `_engine`.
//
// EVERY `@duckdb/node-api` call is isolated here behind a dynamic import, so the
// base read/validate/emit surface never loads DuckDB and users who don't run
// sql()/at() never need to install it. `@duckdb/node-api` is an OPTIONAL peer;
// absent → a helpful install error.
//
// Why the Appender (not Arrow): the modern `@duckdb/node-api` ("Neo") client has
// no Arrow bridge — no register/read_arrow/scan_arrow_ipc, results only as JS
// rows/cols. (The legacy `duckdb` package does have an Arrow API, but it
// segfaults on current Node + ships high-severity vulns.) So each group's
// born-typed arrow-js Table is loaded via a typed CREATE TABLE + the Appender —
// proven type-faithful (typed nulls, timestamps, booleans all correct).
import { type Table, tableFromIPC } from "apache-arrow";
import type { GroupMeta } from "./native";
import { Ags4Error } from "./errors";

/** One query result row — JS-native values (`getRowObjectsJS`: Dates, numbers,
 * nulls; 64-bit ints stay `bigint`). */
export type Row = Record<string, unknown>;

/** Coerce a non-null cell value to a VARCHAR string without the `[object
 *  Object]` footgun (an unexpected object → JSON; cells are primitives, so
 *  that branch is defensive only). */
function scalarString(value: unknown): string {
  if (typeof value === "string") return value;
  if (
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint"
  )
    return String(value);
  return JSON.stringify(value);
}

/** Query output shape. Default is row objects; `arrow: true` returns a born-typed
 * arrow-js `Table` (via DuckDB's `arrow` **community** extension, loaded on first
 * use — needs network the first time, so it's opt-in, not the default). */
export interface QueryOptions {
  arrow?: boolean;
}

/** Stats from `toDuckdb` — the written `path` plus how many tables/rows landed.
 * snake_case to match `laterite-py`'s `to_duckdb` dict and the Rust `ExcelStats`. */
export interface DuckdbStats {
  path: string;
  tables_written: number;
  rows_written: number;
}

// Minimal structural types for the slice of `@duckdb/node-api` we use — keeps
// the package's own typecheck free of any static dependency on the optional peer
// (the dynamic import below is deliberately untyped).
interface DuckAppender {
  appendNull(): void;
  appendVarchar(v: string): void;
  appendDouble(v: number): void;
  appendBigInt(v: bigint): void;
  appendBoolean(v: boolean): void;
  appendTimestamp(v: unknown): void;
  endRow(): void;
  flushSync(): void;
  closeSync(): void;
}
interface DuckConnection {
  run(sql: string): Promise<unknown>;
  runAndReadAll(
    sql: string,
    values?: unknown[],
  ): Promise<{ getRowObjectsJS(): Row[] }>;
  createAppender(table: string): Promise<DuckAppender>;
  disconnectSync(): void;
}
interface DuckInstance {
  connect(): Promise<unknown>;
}
// The slice of `@duckdb/node-api`'s module surface we touch. The dynamic import
// (loadDuck) is otherwise untyped — this keeps the two calls in `create()` typed
// without a static dependency on the optional peer's own types.
interface DuckModule {
  DuckDBInstance: { create(path: string): Promise<DuckInstance> };
  timestampValue: (micros: bigint) => unknown;
}

/** Quote a SQL identifier (group code / heading), doubling embedded quotes. */
export function quoteId(name: string): string {
  return `"${name.replace(/"/g, '""')}"`;
}

/** Drop the synthetic `_id`/`_parent_id` content-addressed key columns from an
 * arrow-js `Table` — for a FRAME (the accessor) or for EMIT (`buildAgs4`, which
 * is byte-faithful to the DATA and must never write a synthetic column). Only
 * the two IDENTITY columns are stripped; an opt-in `_content_hash` (#448) is a
 * VALUE fingerprint, not an identity, and survives into the default frame —
 * mirrors Python, which keeps `_content_hash` and strips only the ids. A table
 * with neither is returned unchanged. (#303) */
export function stripSynthKeys(table: Table): Table {
  const dataCols = table.schema.fields
    .filter((f) => f.name !== "_id" && f.name !== "_parent_id")
    .map((f) => f.name);
  return dataCols.length === table.numCols ? table : table.select(dataCols);
}

// Resolved lazily, once, on first sql()/at(). The string-variable import keeps
// TS from statically resolving (and thus requiring) the optional peer's types.
const DUCK_PACKAGE = "@duckdb/node-api";
let duckMod: Promise<DuckModule> | null = null;
function loadDuck(): Promise<DuckModule> {
  if (duckMod === null) {
    duckMod = import(DUCK_PACKAGE).catch(() => {
      throw new Ags4Error(
        "sql()/at() need the optional peer dependency '@duckdb/node-api'. " +
          "Install it with: npm install @duckdb/node-api",
      );
    });
  }
  return duckMod;
}

/** An in-memory DuckDB connection seeded with this file's groups as native
 * tables. Created lazily; closed synchronously (`disconnectSync`). */
export class DuckEngine {
  readonly #con: DuckConnection;
  readonly #timestampValue: (micros: bigint) => unknown;
  /** Group codes already loaded into the engine (CTAS'd once). */
  readonly registered = new Set<string>();
  // The `arrow` community extension is loaded lazily, once, on the first
  // `{ arrow: true }` query (it isn't needed for the default row-object path).
  #arrowLoaded = false;

  private constructor(
    con: DuckConnection,
    timestampValue: (m: bigint) => unknown,
  ) {
    this.#con = con;
    this.#timestampValue = timestampValue;
  }

  static async create(): Promise<DuckEngine> {
    const m = await loadDuck();
    const instance = await m.DuckDBInstance.create(":memory:");
    const con = (await instance.connect()) as DuckConnection;
    return new DuckEngine(con, m.timestampValue);
  }

  /** The raw `@duckdb/node-api` connection — the escape hatch for every engine
   * feature. */
  get connection(): unknown {
    return this.#con;
  }

  /** Load one group into a native DuckDB table (typed columns from `sqlTypes`,
   * rows appended from the born-typed arrow-js Table). Idempotent per code.
   *
   * Schema-driven: the column list follows `table.schema.fields` exactly (a
   * KNOWN group's Table carries `_id`/`_parent_id` first, see `table_ipc`, and
   * an opt-in `_content_hash` trailing, #448), so nothing the Arrow batch
   * carries is silently dropped. Any `_`-prefixed synthetic column is VARCHAR;
   * every other column is the dictionary's `sqlType` for that heading. Cross-
   * group joins in `sql()`/`at()` work with no opt-in because the ENGINE always
   * keeps the key columns — the user `table()` accessor is what strips
   * them. (#303) */
  async register(code: string, meta: GroupMeta, table: Table): Promise<void> {
    if (this.registered.has(code)) return;
    const cols = table.schema.fields.map((f) => ({
      name: f.name,
      sqlType: f.name.startsWith("_")
        ? "VARCHAR"
        : (meta.sqlTypes[meta.headings.indexOf(f.name)] ?? "VARCHAR"),
    }));
    const colsSql = cols
      .map((c) => `${quoteId(c.name)} ${c.sqlType}`)
      .join(", ");
    await this.#con.run(`CREATE TABLE ${quoteId(code)} (${colsSql})`);
    const appender = await this.#con.createAppender(code);
    const vectors = cols.map((c) => table.getChild(c.name));
    for (let r = 0; r < table.numRows; r++) {
      for (let c = 0; c < cols.length; c++) {
        const col = cols[c]; // c < cols.length → in-bounds.
        if (!col) continue;
        this.#appendCell(appender, col.sqlType, vectors[c]?.get(r));
      }
      appender.endRow();
    }
    appender.flushSync();
    appender.closeSync();
    this.registered.add(code);
  }

  /** Run SQL, binding positional `?` params; rows as JS-native objects. */
  async query(sql: string, params?: unknown[]): Promise<Row[]> {
    const reader = await this.#con.runAndReadAll(sql, params);
    return reader.getRowObjectsJS();
  }

  /** Run SQL and return a born-typed arrow-js `Table` — wraps the query in
   * DuckDB's `to_arrow_ipc` (the `arrow` community extension) and decodes the IPC
   * blobs. Lazy-loads the extension; a clear error if it can't (offline). */
  async queryArrow(sql: string, params?: unknown[]): Promise<Table> {
    await this.#ensureArrow();
    // `to_arrow_ipc((<query>))` emits one row per IPC message (schema then
    // batches); concatenating the `ipc` blobs in order is a valid IPC stream.
    const reader = await this.#con.runAndReadAll(
      `SELECT * FROM to_arrow_ipc((${sql}))`,
      params,
    );
    const chunks = reader
      .getRowObjectsJS()
      .map((r) => Buffer.from(r.ipc as Uint8Array));
    // One IPC blob → pass it straight through (no copy); many → concatenate.
    const single = chunks.length === 1 ? chunks[0] : undefined;
    return tableFromIPC(single ?? Buffer.concat(chunks));
  }

  async #ensureArrow(): Promise<void> {
    if (this.#arrowLoaded) return;
    try {
      await this.#con.run("INSTALL arrow FROM community");
      await this.#con.run("LOAD arrow");
      this.#arrowLoaded = true;
    } catch (e) {
      throw new Ags4Error(
        "arrow output ({ arrow: true }) needs DuckDB's 'arrow' community extension, " +
          "which could not be installed/loaded (offline or air-gapped?). Use the default " +
          "row-object output, or pre-install it: INSTALL arrow FROM community. " +
          `(${e instanceof Error ? e.message : String(e)})`,
      );
    }
  }

  /** Persist the given already-`register()`ed group tables to a DuckDB database
   * at `path`: ATTACH the on-disk db, copy each group in with a per-group CTAS
   * (not `COPY FROM DATABASE`, so a caller-registered view can't leak in and
   * `codes` selects/orders exactly), then DETACH. The tables are copied WITH
   * their `_id`/`_parent_id` keys — the point of a persisted store. Returns the
   * total rows written. */
  async persist(path: string, codes: string[]): Promise<number> {
    // ATTACH takes no bind parameter for the path — single-quote-escape it.
    const escaped = path.replace(/'/g, "''");
    await this.#con.run(`ATTACH '${escaped}' AS _lat_out`);
    let rows = 0;
    try {
      for (const code of codes) {
        await this.#con.run(
          `CREATE TABLE _lat_out.${quoteId(code)} AS SELECT * FROM ${quoteId(code)}`,
        );
        const r = await this.#con.runAndReadAll(
          `SELECT count(*) AS n FROM _lat_out.${quoteId(code)}`,
        );
        // count(*) always yields exactly one row, so index [0] is present —
        // cast past the `noUncheckedIndexedAccess` `| undefined`; `n` is a bigint.
        const [row] = r.getRowObjectsJS() as [Row];
        rows += Number(row.n);
      }
    } finally {
      await this.#con.run("DETACH _lat_out");
    }
    return rows;
  }

  /** Close the connection (synchronous — mirrors `Ags4File.close`). */
  close(): void {
    this.#con.disconnectSync();
    this.registered.clear();
  }

  #appendCell(appender: DuckAppender, sqlType: string, value: unknown): void {
    if (value === null || value === undefined) {
      appender.appendNull();
      return;
    }
    switch (sqlType) {
      case "DOUBLE":
        appender.appendDouble(Number(value));
        break;
      case "BIGINT":
        appender.appendBigInt(
          typeof value === "bigint" ? value : BigInt(value as number),
        );
        break;
      case "BOOLEAN":
        appender.appendBoolean(Boolean(value));
        break;
      case "TIMESTAMP":
        appender.appendTimestamp(this.#timestampValue(toMicros(value)));
        break;
      default: // VARCHAR + any unmapped type
        appender.appendVarchar(scalarString(value));
        break;
    }
  }
}

/** arrow-js Timestamp(µs).get() yields epoch-**ms** (a number, or a Date);
 * DuckDB's `timestampValue` wants epoch-**µs**. */
function toMicros(value: unknown): bigint {
  const ms = value instanceof Date ? value.getTime() : Number(value);
  return BigInt(Math.round(ms * 1000));
}
