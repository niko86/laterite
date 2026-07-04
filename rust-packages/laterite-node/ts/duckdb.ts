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

/** Query output shape. Default is row objects; `arrow: true` returns a born-typed
 * arrow-js `Table` (via DuckDB's `arrow` **community** extension, loaded on first
 * use — needs network the first time, so it's opt-in, not the default). */
export interface QueryOptions {
  arrow?: boolean;
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

/** Quote a SQL identifier (group code / heading), doubling embedded quotes. */
export function quoteId(name: string): string {
  return `"${name.replace(/"/g, '""')}"`;
}

/** Drop the synthetic `_id`/`_parent_id` content-addressed key columns from an
 * arrow-js `Table` — for a FRAME (the accessor) or for EMIT (`buildAgs4`, which
 * is byte-faithful to the DATA and must never write a synthetic column). AGS
 * headings never start with `_`, so this is safe; a table with none is returned
 * unchanged. (#303) */
export function stripSynthKeys(table: Table): Table {
  const dataCols = table.schema.fields
    .filter((f) => !f.name.startsWith("_"))
    .map((f) => f.name);
  return dataCols.length === table.numCols ? table : table.select(dataCols);
}

// Resolved lazily, once, on first sql()/at(). The string-variable import keeps
// TS from statically resolving (and thus requiring) the optional peer's types.
const DUCK_PACKAGE = "@duckdb/node-api";
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let duckMod: Promise<any> | null = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function loadDuck(): Promise<any> {
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

  private constructor(con: DuckConnection, timestampValue: (m: bigint) => unknown) {
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
   * rows appended from the born-typed arrow-js Table). Idempotent per code. */
  async register(code: string, meta: GroupMeta, table: Table): Promise<void> {
    if (this.registered.has(code)) return;
    // A KNOWN group's Table carries the two content-addressed key columns (`_id`,
    // `_parent_id`) first (see `table_ipc`). Prepend them to the relational table
    // as VARCHAR so cross-group joins work in `sql()`/`at()` — the user `table()`
    // accessor strips them, but the ENGINE always keeps them. (#303)
    const keyCols = table.schema.fields.some((f) => f.name === "_id")
      ? ["_id", "_parent_id"]
      : [];
    const cols = [
      ...keyCols.map((k) => `${quoteId(k)} VARCHAR`),
      ...meta.headings.map((h, i) => `${quoteId(h)} ${meta.sqlTypes[i]}`),
    ].join(", ");
    await this.#con.run(`CREATE TABLE ${quoteId(code)} (${cols})`);
    const appender = await this.#con.createAppender(code);
    const keyVectors = keyCols.map((k) => table.getChild(k));
    const vectors = meta.headings.map((h) => table.getChild(h));
    for (let r = 0; r < table.numRows; r++) {
      for (let c = 0; c < keyCols.length; c++) {
        this.#appendCell(appender, "VARCHAR", keyVectors[c]?.get(r));
      }
      for (let c = 0; c < meta.headings.length; c++) {
        this.#appendCell(appender, meta.sqlTypes[c] ?? "VARCHAR", vectors[c]?.get(r));
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
    const reader = await this.#con.runAndReadAll(`SELECT * FROM to_arrow_ipc((${sql}))`, params);
    const chunks = reader
      .getRowObjectsJS()
      .map((r) => Buffer.from(r.ipc as Uint8Array));
    return tableFromIPC(chunks.length === 1 ? chunks[0]! : Buffer.concat(chunks));
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
        appender.appendBigInt(typeof value === "bigint" ? value : BigInt(value as number));
        break;
      case "BOOLEAN":
        appender.appendBoolean(Boolean(value));
        break;
      case "TIMESTAMP":
        appender.appendTimestamp(this.#timestampValue(toMicros(value)));
        break;
      default: // VARCHAR + any unmapped type
        appender.appendVarchar(String(value));
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
