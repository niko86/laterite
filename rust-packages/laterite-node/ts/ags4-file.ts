import { writeFileSync } from "node:fs";
import { type Table, tableFromIPC } from "apache-arrow";
import { DuckEngine, type QueryOptions, quoteId, type Row } from "./duckdb";
import type { GroupMeta } from "./native";
import { Reading } from "./native";
import { AgsSubset, type Filter } from "./subset";

/**
 * A parsed AGS4 file — the Node port of laterite-py's `Ags4File`. The read
 * surface is **Arrow-direct**: `table(code)` decodes the native typed Arrow IPC
 * straight to an arrow-js Table (no DuckDB round-trip). Python routes reads
 * through DuckDB only because its pandas path is otherwise pyarrow-bound; Node
 * has no such reason, so the base read surface needs no engine at all.
 *
 * The OPTIONAL DuckDB layer — `sql()` / `at()` / `connection` — is lazy: the
 * engine spins up only on first use, and `@duckdb/node-api` is an optional peer
 * (absent → a helpful install error). These are async (the Neo client is
 * promise-based); everything else stays sync.
 */
export class Ags4File {
  readonly #reading: Reading;
  // One arrow-js Table per group, decoded once on first `table(code)`.
  readonly #tables = new Map<string, Table>();
  // The DuckDB engine — created lazily on first sql()/at()/connection.
  #engine: DuckEngine | null = null;
  // Memoised AGS4 re-emit (the `text`/`bytes` getters; the emit is O(size)).
  #text?: string;
  #bytes?: Buffer;

  constructor(reading: Reading) {
    this.#reading = reading;
  }

  // --- metadata (no Arrow decode) ------------------------------------------

  /** Group codes in file order. */
  get groups(): string[] {
    return this.#reading.groupCodes();
  }

  /** The file's `TRAN_AGS` edition string, if present. */
  get tranAgs(): string | null {
    return this.#reading.tranAgs;
  }

  #meta(code: string): GroupMeta {
    const m = this.#reading.meta(code);
    if (m === null) throw new Error(`group ${JSON.stringify(code)} not in file`);
    return m;
  }

  /** The HEADING-row codes of `code`, in file order. Throws if `code` isn't in the file. */
  headings(code: string): string[] {
    return this.#meta(code).headings;
  }
  /** The UNIT row of `code`, one per heading. Throws if `code` isn't in the file. */
  units(code: string): string[] {
    return this.#meta(code).units;
  }
  /** The TYPE row of `code` (the AGS data types), one per heading. Throws if `code` isn't in the file. */
  types(code: string): string[] {
    return this.#meta(code).types;
  }
  /** AGS TYPE → the DuckDB column type each heading lands as (for the P3 engine). */
  sqlTypes(code: string): string[] {
    return this.#meta(code).sqlTypes;
  }
  /** 1-indexed source line of each DATA row. */
  lineNumbers(code: string): number[] {
    return this.#meta(code).lineNumbers;
  }

  /** Whether `code` is one of the file's groups — the cheap membership check `headings`/`table` would otherwise throw on. */
  has(code: string): boolean {
    return this.#reading.meta(code) !== null;
  }

  // --- born-typed data (Arrow-direct) --------------------------------------

  /** One group as a born-typed arrow-js `Table` (a 2DP heading is Float64, an ID
   * Utf8, a DT a Timestamp) — the SAME typing the Python/wasm hosts produce,
   * byte-identical by construction (one shared `build_record_batch`). Cached per
   * group. Throws if `code` isn't in the file. */
  table(code: string): Table {
    const cached = this.#tables.get(code);
    if (cached !== undefined) return cached;
    const ipc = this.#reading.tableIpc(code);
    if (ipc === null) throw new Error(`group ${JSON.stringify(code)} not in file`);
    const table = tableFromIPC(ipc);
    this.#tables.set(code, table);
    return table;
  }

  // --- emit / save ---------------------------------------------------------

  /** Spec-correct AGS4 as text — byte-faithful to the source DATA values
   * (re-emitted native-side from the retained parse). Memoised. */
  get text(): string {
    return (this.#text ??= this.#reading.emit());
  }

  /** `text` encoded UTF-8 — the bytes `save()` writes. Memoised. */
  get bytes(): Buffer {
    return (this.#bytes ??= Buffer.from(this.text, "utf8"));
  }

  /** Write the AGS4 to `path` (UTF-8) — the inverse of `read`. The bytes are
   * byte-faithful to the source DATA values, re-emitted from the retained parse.
   *
   * @param path Filesystem path to write the UTF-8 AGS4 to.
   * @returns The same `path`, for chaining. */
  save(path: string): string {
    writeFileSync(path, this.bytes);
    return path;
  }

  // --- optional DuckDB engine (sql / at / connection) ----------------------

  async #getEngine(): Promise<DuckEngine> {
    if (this.#engine === null) this.#engine = await DuckEngine.create();
    return this.#engine;
  }

  /** Load one group into the engine on demand (CTAS from its born-typed Table). */
  async #register(code: string, engine: DuckEngine): Promise<void> {
    await engine.register(code, this.#meta(code), this.table(code));
  }

  async #registerAll(engine: DuckEngine): Promise<void> {
    for (const code of this.groups) await this.#register(code, engine);
  }

  /** Run SQL over the file's groups by their clean names — e.g.
   * `await ags.sql("SELECT * FROM SAMP JOIN LOCA USING (LOCA_ID) WHERE …")`.
   * Any group may be referenced, so this loads them all into the engine.
   *
   * @param query SQL referencing groups by their bare AGS code as table names.
   * @param opts `{ arrow: true }` returns a born-typed arrow-js `Table` (loads the
   *   `arrow` community extension on first use) instead of JS-native rows.
   * @returns JS-native row objects by default, or a `Table` when `arrow` is set.
   * @throws If the optional `@duckdb/node-api` peer is absent. */
  sql(query: string): Promise<Row[]>;
  sql(query: string, opts: { arrow: true }): Promise<Table>;
  sql(query: string, opts?: QueryOptions): Promise<Row[] | Table>;
  async sql(query: string, opts: QueryOptions = {}): Promise<Row[] | Table> {
    const engine = await this.#getEngine();
    await this.#registerAll(engine);
    return opts.arrow ? engine.queryArrow(query) : engine.query(query);
  }

  /** Filter to a parent entity's records — `ags.at("LOCA", ["BH01", "BH02"])`
   * returns a view whose `table(code)` yields only the rows whose `{group}_ID`
   * is in `values`. Chain to narrow (`.at("SAMP", […])`); `sub.groups` is the
   * related groups, `sub.frames()` pulls them all. Groups carrying none of the
   * keys pass through. For any other predicate, use `sql("… WHERE …")`.
   *
   * @param group The parent group whose `{group}_ID` key drives the filter.
   * @param values The id values to keep (empty matches nothing).
   * @returns An `AgsSubset` view; further `.at()` calls accumulate filters. */
  at(group: string, values: Iterable<unknown>): AgsSubset {
    return new AgsSubset(this, [[`${group}_ID`, [...values]]]);
  }

  /** The raw `@duckdb/node-api` connection — every engine feature — seeded with
   * all of this file's groups under their clean names. */
  get connection(): Promise<unknown> {
    return (async () => {
      const engine = await this.#getEngine();
      await this.#registerAll(engine);
      return engine.connection;
    })();
  }

  /** @internal — backs `AgsSubset.table`: `code` filtered by every applicable
   * key (an empty value list matches nothing; groups carrying no key pass). */
  async _filteredRows(
    code: string,
    filters: Filter[],
    opts: QueryOptions = {},
  ): Promise<Row[] | Table> {
    const engine = await this.#getEngine();
    await this.#register(code, engine);
    const cols = new Set(this.headings(code));
    const clauses: string[] = [];
    const params: unknown[] = [];
    for (const [key, values] of filters) {
      if (!cols.has(key)) continue;
      if (values.length === 0) {
        clauses.push("FALSE"); // an empty selection matches nothing
      } else {
        clauses.push(`${quoteId(key)} IN (${values.map(() => "?").join(", ")})`);
        params.push(...values);
      }
    }
    const where = clauses.length > 0 ? clauses.join(" AND ") : "TRUE";
    const sql = `SELECT * FROM ${quoteId(code)} WHERE ${where}`;
    return opts.arrow ? engine.queryArrow(sql, params) : engine.query(sql, params);
  }

  // --- lifecycle -----------------------------------------------------------

  /** Drop the decoded-Table cache and close the DuckDB engine (if any).
   * `using f = read(…)` runs this automatically. */
  close(): void {
    this.#tables.clear();
    if (this.#engine !== null) {
      this.#engine.close();
      this.#engine = null;
    }
  }

  /** `using f = read(…)` disposal hook — delegates to `close()`. */
  [Symbol.dispose](): void {
    this.close();
  }

  /** A compact one-line summary — group count and `TRAN_AGS` — for logs and the REPL. */
  toString(): string {
    return `<Ags4File groups=${this.groups.length} tranAgs=${JSON.stringify(this.tranAgs)}>`;
  }
}
