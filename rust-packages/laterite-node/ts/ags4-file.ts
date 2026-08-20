import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { type Table, tableFromIPC } from "apache-arrow";
import {
  DuckEngine,
  type DuckdbStats,
  type QueryOptions,
  quoteId,
  type Row,
} from "./duckdb";
import { Ags4Error, raiseFor } from "./errors";
// The chained verbs (`fix`/`diff`/`toExcel`) reuse the free functions with this
// handle's retained source, so the ONE engine call + error mapping lives in one
// place. (`validate` calls the native door directly — it has a certificate to hand
// over, and that is not a public knob.) index.ts imports Ags4File, so this is a
// deliberate cycle — safe because
// the free fns are referenced only inside method bodies (resolved at call time,
// never at module-eval time), the standard ESM live-binding pattern.
import {
  type DiffOptions,
  type DiffSource,
  type FixOptions,
  type RevisionDelta,
  type ValidateOptions,
  diff as diffFree,
  fix as fixFree,
  toExcel as toExcelFree,
} from "./index";
import type { FixResult } from "./fix-result";
import type { ExcelStats, GroupMeta } from "./native";
import { Reading, Sidecar, parseArrow, runCheck } from "./native";
import { Report } from "./report";
import { AgsSubset, type Filter } from "./subset";

/** The source a handle was read from — retained so the chained `validate`/`fix`/
 * `diff` verbs re-run against the TRUE bytes (matching original line numbers),
 * not the byte-faithful `.bytes` re-emit. A synthesised handle has none. */
export interface Ags4Source {
  path?: string;
  text?: string;
  data?: Uint8Array;
  encoding?: string;
}

/** Knobs for {@link Ags4File.certify} / {@link Ags4File.certifyBytes}: the edition pin
 *  and the #568 custom-`--dict` overlay the certificate is minted against. Each defaults
 *  to the last `.validate()` on this handle (see `#mint`). */
export interface CertifyOptions {
  /** Force the base edition; default is the last `.validate()`'s, else auto from TRAN_AGS. */
  dictVersion?: string;
  /** A custom AGS4 dictionary (path or raw `.ags`/JSON bytes) to certify against — its
   *  `{name, hash}` is stamped into the cert (O-48). */
  dictionary?: string | Uint8Array;
  /** Treat `dictionary` as a full replacement (no base edition). */
  dictReplace?: boolean;
}

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
  // One RAW keyed arrow-js Table per group (with _id/_parent_id for known
  // groups), built ONLY when keys are needed: table({keys:true}) and the
  // relational sql()/at() layer (`#register`). The keychain is ~96% of the
  // native build cost, so the DEFAULT keys-less table() must not come through
  // here (#6).
  readonly #tables = new Map<string, Table>();
  // The DEFAULT keys-less typed Table per group — the native build with the
  // keychain SKIPPED, cached independently so a plain table() never poisons the
  // keyed `#tables` the relational layer needs (#6). Its columns equal the old
  // stripped view's (data + trailing _content_hash), so table()'s output is
  // unchanged — just ~28x cheaper to build.
  readonly #framesKeyless = new Map<string, Table>();
  // The DuckDB engine — created lazily on first sql()/at()/connection.
  #engine: DuckEngine | null = null;
  // Memoised AGS4 re-emit (the `text`/`bytes` getters; the emit is O(size)).
  #text?: string;
  #bytes?: Buffer;
  // The retained read source (for faithful chained re-runs) + the last verb
  // outcomes: `#report` from `.validate()`, `#fixReport` on a `.fix()` result.
  readonly #src?: Ags4Source;
  #report?: Report;
  #fixReport?: FixResult;
  // The `.ags.idx` certificate carried from `read(..., { index })`. It is HANDED to the
  // engine on `.validate()`; this class no longer decides whether it may be trusted.
  #cert?: Sidecar;
  // The edition the caller last asked for. Provenance for a following `.certify()`, so
  // `validate({dictVersion}).certify()` mints a cert for the edition you validated
  // against. NOT a trust claim — the mint re-validates; this only says with which
  // dictionary.
  #lastDictVersion: string | undefined;
  // Same provenance for a `--dict` custom overlay (#568): the dictionary the last
  // `.validate()` overlaid and whether it replaced the base, so a following `.certify()`
  // mints against the same effective dictionary (and stamps its {name, hash}).
  #lastDictionary: string | Uint8Array | undefined;
  #lastDictReplace = false;

  // Whether this handle's relational tables CARRY a `_content_hash` column —
  // the typed, blank-insensitive fingerprint of a row's whole VALUE (as
  // against `_id`, which fingerprints its IDENTITY: two deliveries of the
  // same borehole with a corrected level share an `_id` and differ here).
  // HANDLE-level (set once at construction, not per `table()` call) so the
  // `#tables` memoisation keyed by `code` alone stays correct — mirrors
  // Python's `Ags4File(content_hash=…)`. (#448)
  readonly #contentHash: boolean;

  constructor(reading: Reading, src?: Ags4Source, contentHash = false) {
    this.#reading = reading;
    this.#src = src;
    this.#contentHash = contentHash;
  }

  /** @internal — `read(..., { index })` attaches a freshness-checked certificate
   * so a later errors-only `.validate()` can skip the rule engine. */
  _attachCert(cert: Sidecar): void {
    this.#cert = cert;
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
    if (m === null)
      throw new Error(`group ${JSON.stringify(code)} not in file`);
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

  /** The raw KEYED Table straight from IPC — a KNOWN group carries the two
   * content-addressed key columns `_id`/`_parent_id` first (#303). Built only
   * when keys are needed (`table({keys:true})` and the relational `#register`),
   * because the keychain is ~96% of the native build (#6). Cached in `#tables`. */
  #rawTable(code: string): Table {
    const cached = this.#tables.get(code);
    if (cached !== undefined) return cached;
    const ipc = this.#reading.tableIpc(code, this.#contentHash, true);
    if (ipc === null)
      throw new Error(`group ${JSON.stringify(code)} not in file`);
    const table = tableFromIPC(ipc);
    this.#tables.set(code, table);
    return table;
  }

  /** The DEFAULT keys-less Table — the native build with the keychain SKIPPED
   * (`withKeys=false`), so a keys-less read never pays for `_id`/`_parent_id` it
   * would only strip (#6). Byte-equal columns to the old `stripSynthKeys(raw)`
   * view (data + trailing `_content_hash`), so `table(code)`'s output is
   * unchanged. Cached in `#framesKeyless`, kept separate from the keyed `#tables`
   * so it never poisons the relational layer. */
  #keylessTable(code: string): Table {
    const cached = this.#framesKeyless.get(code);
    if (cached !== undefined) return cached;
    const ipc = this.#reading.tableIpc(code, this.#contentHash, false);
    if (ipc === null)
      throw new Error(`group ${JSON.stringify(code)} not in file`);
    const table = tableFromIPC(ipc);
    this.#framesKeyless.set(code, table);
    return table;
  }

  /** One group as a born-typed arrow-js `Table` (a 2DP heading is Float64, an ID
   * Utf8, a DT a Timestamp) — the SAME typing the Python/wasm hosts produce,
   * byte-identical by construction (one shared `build_record_batch`). Cached per
   * group. Throws if `code` isn't in the file.
   *
   * By default the synthetic `_id`/`_parent_id` key columns are **absent** (the
   * native keychain is skipped entirely, not built-then-stripped — #6); pass
   * `{ keys: true }` to include them (the relational `sql()`/`at()` layer always
   * carries them regardless — that's what makes cross-group joins work). */
  table(code: string, opts?: { keys?: boolean }): Table {
    return opts?.keys ? this.#rawTable(code) : this.#keylessTable(code);
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

  /** Write this file's groups to an `.xlsx` workbook — one worksheet per group,
   * from the spec-correct {@link bytes}. With `xlsxPath` given the workbook is
   * written there and the conversion stats returned; omit it to get the `.xlsx`
   * **bytes** back. Mirrors `laterite.Ags4File.to_excel()` and the free
   * {@link toExcel}. `groups` fixes the worksheet order (default source order). */
  toExcel(xlsxPath: string, opts?: { groups?: string[] }): ExcelStats;
  toExcel(xlsxPath?: undefined, opts?: { groups?: string[] }): Buffer;
  toExcel(
    xlsxPath?: string,
    opts: { groups?: string[] } = {},
  ): ExcelStats | Buffer {
    // Delegate to the free verb with this handle's bytes — one write path.
    return xlsxPath === undefined
      ? toExcelFree(this.bytes, undefined, opts)
      : toExcelFree(this.bytes, xlsxPath, opts);
  }

  /** Persist this file's groups to a DuckDB database at `path` — one born-typed
   * table per group under its clean 4-letter code, each carrying the content-
   * addressed `_id`/`_parent_id` key columns, so the store is join-ready and
   * version-diffable by `_id` (what the `read_ags` DuckDB extension diffs on). The
   * DuckDB counterpart to {@link save} (AGS4) and {@link toExcel} (XLSX).
   *
   * Returns `{ path, tables_written, rows_written }`. `groups` optionally
   * restricts/re-orders the tables (default: all, source order). The tables are
   * always keyed — unlike a {@link table} frame, which drops the ids for display.
   * Refuses to overwrite an existing `path`. Needs the optional `@duckdb/node-api`
   * peer. Mirrors `laterite.Ags4File.to_duckdb()`. */
  async toDuckdb(
    path: string,
    opts: { groups?: string[] } = {},
  ): Promise<DuckdbStats> {
    if (existsSync(path)) {
      throw new Ags4Error(
        `${path} exists; toDuckdb writes a fresh database (remove it first)`,
      );
    }
    const codes = opts.groups ?? this.groups;
    for (const code of codes) {
      if (!this.has(code)) throw new Ags4Error(`group '${code}' not in file`);
    }
    const engine = await this.#getEngine();
    for (const code of codes) await this.#register(code, engine);
    const rows = await engine.persist(path, codes);
    return { path, tables_written: codes.length, rows_written: rows };
  }

  // --- fluent verbs (validate / fix / diff) --------------------------------

  /** Resolve this handle to the `(source, opts)` the free fns take: the retained
   * read source (so line numbers match the original), or the re-emit for a
   * synthesised handle. `text` is not re-encoded; a path/bytes source carries its
   * read `encoding`. */
  #freeSource(): [
    string | Uint8Array | undefined,
    { text?: string; encoding?: string },
  ] {
    const s = this.#src;
    if (s?.path !== undefined) return [s.path, { encoding: s.encoding }];
    if (s?.text !== undefined) return [undefined, { text: s.text }];
    if (s?.data !== undefined) return [s.data, { encoding: s.encoding }];
    return [this.bytes, {}]; // no retained source → validate/fix the re-emit
  }

  /** The last `.validate()` outcome (`undefined` until validated). */
  get report(): Report | undefined {
    return this.#report;
  }

  /** The `FixResult` on a handle returned by `.fix()` (`undefined` otherwise). */
  get fixReport(): FixResult | undefined {
    return this.#fixReport;
  }

  /** Validate this file against the AGS4 rules and return `this` (chainable —
   * `read(p).validate().sql(...)`); the outcome lands on `.report`. Same engine as
   * the free `validate()`, run on the source this handle was read from so line
   * numbers match the original. Errors + WARNINGs by default (`warnings`); `fyi`
   * adds the low-signal tier. `encoding` defaults to the one this handle was read
   * with. Mirrors `laterite.Ags4File.validate()`. */
  validate(opts: Omit<ValidateOptions, "text"> = {}): this {
    // The certificate short-circuit used to live HERE, as this class's own conjunction of
    // `matchesNativeValidator()` + `profileCovers()` + "errors only" — one of five such
    // conjunctions across the surfaces, no two alike, four of them able to report a file
    // clean that was not. The cert is now simply HANDED to the engine, which decides
    // whether it can answer this question, and skips the rules only if it can answer it
    // completely.
    //
    // `checkFiles` is never answered from a certificate: Rule 20's on-disk `FILE/` tree
    // can be deleted without changing a byte of the .ags, so no statement about the file's
    // bytes can speak for it. It runs live, every time.
    this.#lastDictVersion = opts.dictVersion;
    this.#lastDictionary = opts.dictionary;
    this.#lastDictReplace = opts.dictReplace ?? false;
    const [source, base] = this.#freeSource();
    const path = typeof source === "string" ? source : undefined;
    const data =
      typeof source === "string" || source == null ? undefined : source;
    const dictPath =
      typeof opts.dictionary === "string" ? opts.dictionary : undefined;
    const dictBytes =
      typeof opts.dictionary === "string" || opts.dictionary == null
        ? undefined
        : opts.dictionary;
    // Not the free `validate()`: the certificate is a HANDLE-scoped fact (it arrived with
    // `read(..., { index })`), not a knob a caller passes, so it is not in the public
    // `ValidateOptions`. Same native door, extra arguments. Mirrors laterite-py.
    this.#report = new Report(
      raiseFor(
        runCheck(
          path,
          base.text,
          data,
          opts.dictVersion,
          opts.warnings,
          opts.fyi,
          opts.warningsAsErrors,
          opts.checkFiles,
          opts.encoding ?? base.encoding,
          dictPath,
          dictBytes,
          opts.dictReplace,
          this.#cert,
        ),
      ),
    );
    return this;
  }

  /** Mechanically repair this file and return a NEW, repaired `Ags4File` — the
   * fluent transform, so `read(p).fix().validate().save(out)` reads as one chain.
   * The `FixResult` (what was applied + residual findings) rides on the returned
   * handle's `.fixReport`. Safe fixes always apply; `risky` adds the intent-
   * guessing set. `encoding` defaults to this handle's read encoding. Non-
   * destructive — the source on disk is untouched. Mirrors
   * `laterite.Ags4File.fix()`. */
  fix(opts: Omit<FixOptions, "text"> = {}): Ags4File {
    const [source, base] = this.#freeSource();
    const result = fixFree(source, {
      ...base,
      ...opts,
      encoding: opts.encoding ?? base.encoding,
    });
    // The repaired handle's source IS the repaired UTF-8 bytes (BOM-stripped).
    const repaired = new Ags4File(
      parseArrow(undefined, undefined, result.bytes, undefined),
      {
        data: result.bytes,
      },
    );
    repaired.#fixReport = result;
    return repaired;
  }

  /** Compare this file (the baseline) against `other` (the revision) — the
   * `RevisionDelta` the free `diff()` returns. `other` is a path, bytes, or another
   * `Ags4File`. `encoding` defaults to this handle's read encoding. Mirrors
   * `laterite.Ags4File.diff()`. */
  diff(other: DiffSource, opts: DiffOptions = {}): RevisionDelta {
    return diffFree(this, other, {
      ...opts,
      encoding: opts.encoding ?? this.#src?.encoding,
    });
  }

  // --- certificate (.ags.idx) ----------------------------------------------

  /** The ORIGINAL source bytes — what a certificate indexes + fingerprints, NOT
   * the spec-correct re-emit (which can differ from a non-canonically-formatted
   * on-disk file). A path is re-read, raw bytes returned as-is, text UTF-8-encoded;
   * a synthesised handle falls back to the re-emit. Mirrors `_source_bytes`. */
  #sourceBytes(): Uint8Array {
    const s = this.#src;
    if (s?.path !== undefined) return readFileSync(s.path);
    if (s?.data !== undefined) return s.data;
    if (s?.text !== undefined) return Buffer.from(s.text, "utf8");
    return this.bytes;
  }

  /** Mint this file's `.ags.idx` validity certificate — an error-clean validation plus a
   * byte-offset index — and write it beside the file.
   *
   * `certify` **runs the validation itself**, with every tier on, and records what the
   * rules actually returned. It used to require a prior `.validate()` and then vouch for
   * whatever that found — which made the certificate's contents an assertion by the
   * caller, and the caller got them wrong: the mint's `warnings`/`fyi` parameters were
   * OPTIONAL and defaulted to zero, and nothing ever passed them.
   *
   * It refuses a file with **errors**. Warnings and FYI findings are recorded, not fatal.
   *
   * `path` is the certificate's OUTPUT location (default `<source>.idx`), not a file to
   * certify — it refuses to overwrite the source or any existing non-certificate file.
   * Mirrors `laterite.Ags4File.certify()`. */
  certify(path?: string, opts: CertifyOptions = {}): string {
    const srcPath = this.#src?.path;
    const out = path ?? (srcPath !== undefined ? `${srcPath}.idx` : undefined);
    if (out === undefined) {
      throw new Ags4Error(
        "no source path to derive the .ags.idx location from; pass certify(path) for a handle read from text/bytes",
      );
    }
    // `out` is where the .ags.idx is WRITTEN, never a file to certify. Guard the
    // data-loss footgun read(p).certify(p), and refuse to clobber any existing
    // non-certificate file — certify only ever writes/replaces an .ags.idx.
    if (srcPath !== undefined && resolve(out) === resolve(srcPath)) {
      throw new Ags4Error(
        `certify(path) is the .ags.idx OUTPUT location, not the file to certify — refusing to overwrite the source ${out}`,
      );
    }
    if (existsSync(out) && statSync(out).size > 0) {
      const head = readFileSync(out)
        .subarray(0, 64)
        .toString("utf8")
        .trimStart();
      if (!head.startsWith("{")) {
        throw new Ags4Error(
          `refusing to overwrite ${out}: it is not a laterite certificate (certify writes or replaces an .ags.idx)`,
        );
      }
    }
    writeFileSync(
      out,
      this.#mint(opts.dictVersion, opts.dictionary, opts.dictReplace).toJson(),
    );
    return out;
  }

  /** Mint this file's `.ags.idx` certificate and return its **bytes** in memory — the
   * filesystem-free twin of {@link certify}. Same behaviour (it validates, refuses a file
   * with errors, and records the counts it measured) and the same output, so the bytes
   * interop with `read({ index })`, the CLI `--index`, and the browser cert. Mirrors
   * `laterite.Ags4File.certify_bytes()`. */
  certifyBytes(opts: CertifyOptions = {}): Buffer {
    return this.#mint(
      opts.dictVersion,
      opts.dictionary,
      opts.dictReplace,
    ).toJson();
  }

  /** Mint the `Sidecar` over the ORIGINAL source bytes.
   *
   * The mint validates; it is not told a verdict. There is no longer a parameter through
   * which a caller could assert one. The edition input is the caller's: an explicit
   * `dictVersion` wins, else the one the last `.validate()` used, else auto-resolution.
   * The custom `--dict` overlay follows the same rule — an explicit `dictionary` wins,
   * else the last validate's (with its replace flag). */
  #mint(
    dictVersion?: string,
    dictionary?: string | Uint8Array,
    dictReplace?: boolean,
  ): Sidecar {
    // An explicit `dictionary` brings its own `dictReplace`; falling back to the last
    // validate's overlay inherits that run's replace flag too. Mirrors laterite-py.
    let dict = dictionary;
    let replace = dictReplace ?? false;
    if (dict === undefined) {
      dict = this.#lastDictionary;
      replace = replace || this.#lastDictReplace;
    }
    const dictPath = typeof dict === "string" ? dict : undefined;
    const dictBytes =
      typeof dict === "string" || dict == null ? undefined : dict;
    return Sidecar.mint(
      this.#sourceBytes(),
      new Date().toISOString(),
      dictVersion ?? this.#lastDictVersion,
      this.#src?.encoding,
      undefined,
      dictPath,
      dictBytes,
      replace,
    );
  }

  // --- optional DuckDB engine (sql / at / connection) ----------------------

  async #getEngine(): Promise<DuckEngine> {
    if (this.#engine === null) this.#engine = await DuckEngine.create();
    return this.#engine;
  }

  /** Load one group into the engine on demand (CTAS from its born-typed Table).
   * Uses the RAW keyed Table so the relational layer carries `_id`/`_parent_id`
   * (the accessor strips them; the engine keeps them for joins). */
  async #register(code: string, engine: DuckEngine): Promise<void> {
    await engine.register(code, this.#meta(code), this.#rawTable(code));
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
        clauses.push(
          `${quoteId(key)} IN (${values.map(() => "?").join(", ")})`,
        );
        params.push(...values);
      }
    }
    const where = clauses.length > 0 ? clauses.join(" AND ") : "TRUE";
    // Strip the synthetic key columns from this FRAME surface (the engine table
    // keeps them for joins; the `.at()` accessor returns AGS data). A passthrough
    // group has none, so a plain `*`. (#303)
    const keyed = this.#rawTable(code).schema.fields.some(
      (f) => f.name === "_id",
    );
    const select = keyed ? "* EXCLUDE (_id, _parent_id)" : "*";
    const sql = `SELECT ${select} FROM ${quoteId(code)} WHERE ${where}`;
    return opts.arrow
      ? engine.queryArrow(sql, params)
      : engine.query(sql, params);
  }

  // --- lifecycle -----------------------------------------------------------

  /** Drop the decoded-Table cache and close the DuckDB engine (if any).
   * `using f = read(…)` runs this automatically. */
  close(): void {
    this.#tables.clear();
    this.#framesKeyless.clear();
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
