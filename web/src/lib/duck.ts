// Lazy DuckDB-wasm singleton for the Explore / Tools typed-data path.
//
// DuckDB-wasm is multi-MB, so the engine JS is dynamically imported (and its
// wasm + worker assets fetched) only on first use — never on the validate
// path. The asset URLs come from Vite's `?url`, so they're fingerprinted and
// rewritten under the deploy base (/laterite/) — the same base-path-safe
// trick the validator worker already uses for ags4_wasm_bg.wasm (an
// import.meta.url fetch would 404 under a non-root base). We ship the MVP +
// EH bundles only — NOT coi, which needs the COOP/COEP cross-origin-isolation
// headers GitHub Pages can't set; selectBundle() picks EH on modern browsers
// and MVP as the fallback.

import type {
  AsyncDuckDB,
  AsyncDuckDBConnection,
  DuckDBBundles,
} from "@duckdb/duckdb-wasm";
import type { Table } from "apache-arrow";
import mvpWasm from "@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url";
import mvpWorker from "@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url";
import ehWasm from "@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url";
import ehWorker from "@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url";

interface DuckHandle {
  db: AsyncDuckDB;
  conn: AsyncDuckDBConnection;
}

let instance: Promise<DuckHandle> | null = null;
// Codes whose table is currently ingested (within the active file). Cleared
// by resetDuck() so the next file re-ingests from scratch.
const ingested = new Set<string>();
// Identity of the file whose groups are currently ingested. ExplorePane
// re-mounts (so its resource re-runs) on every tab switch; this lets it skip
// the expensive re-ingest when the bytes haven't changed.
let loadedFile: Uint8Array | null = null;
// The computed group info (meta + row counts) for loadedFile — cached so an
// Explore re-mount returns without re-parsing the file or re-running count(*).
let loadedGroups: unknown = null;

// The fingerprinted-?url bundle map. Module-scope (not local to instantiate)
// so warmFetch() can prime the SAME variant selectBundle will later compile.
const bundles: DuckDBBundles = {
  mvp: { mainModule: mvpWasm, mainWorker: mvpWorker },
  eh: { mainModule: ehWasm, mainWorker: ehWorker },
};

async function instantiate(): Promise<DuckHandle> {
  // Dynamic import keeps the multi-MB duckdb engine JS out of the entry
  // chunk — it lands in its own lazily-fetched chunk.
  const duckdb = await import("@duckdb/duckdb-wasm");
  const bundle = await duckdb.selectBundle(bundles);
  // bundle.mainWorker is the ?url string Vite rewrote under the base; a
  // classic Worker (duckdb's worker is not an ES module). Typed `string | null`,
  // though a selected bundle always carries one — guard for a clear failure.
  if (!bundle.mainWorker)
    throw new Error("DuckDB bundle is missing its worker URL");
  const worker = new Worker(bundle.mainWorker);
  const db = new duckdb.AsyncDuckDB(new duckdb.ConsoleLogger(), worker);
  await db.instantiate(bundle.mainModule, bundle.pthreadWorker);
  const conn = await db.connect();
  return { db, conn };
}

/** Lazily instantiate (and cache) the DuckDB instance + connection. */
export function getDuckDb(): Promise<DuckHandle> {
  if (!instance) instance = instantiate();
  return instance;
}

/** Prime the engine wasm + worker into the HTTP/SW cache WITHOUT compiling, so
 *  a later Explore click downloads nothing — yet a validate-only session never
 *  pays the 36 MB wasm-compile / worker-spawn / engine-heap cost. Best-effort;
 *  only the variant selectBundle would pick is fetched. No-op once instantiated. */
export async function warmFetch(): Promise<void> {
  if (instance) return;
  try {
    const duckdb = await import("@duckdb/duckdb-wasm");
    const bundle = await duckdb.selectBundle(bundles);
    await Promise.allSettled([
      fetch(bundle.mainModule),
      bundle.mainWorker ? fetch(bundle.mainWorker) : Promise.resolve(),
    ]);
  } catch {
    /* best-effort cache priming — a real Explore click will fetch as needed */
  }
}

/** True once the engine is instantiated (or instantiating) this session. Lets
 *  the cold-engine gate skip its confirmation when there's no compile cost left. */
export function isEngineReady(): boolean {
  return instance !== null;
}

/** Ingest one group's typed Arrow IPC stream as a table named by its code.
 *  Idempotent per code within the active file (the ingested-set guards). */
export async function ingestGroup(
  code: string,
  ipc: Uint8Array,
): Promise<void> {
  if (ingested.has(code)) return;
  const { conn } = await getDuckDb();
  await conn.insertArrowFromIPCStream(ipc, { name: code, create: true });
  ingested.add(code);
}

/** True if `code`'s table is already ingested for the active file. */
export function isIngested(code: string): boolean {
  return ingested.has(code);
}

/** True if `bytes` is the file already ingested — lets the Explore pane skip
 *  a re-ingest on re-mount (same dropped file, just a tab switch). */
export function isLoaded(bytes: Uint8Array): boolean {
  return loadedFile === bytes;
}

/** The per-group metadata + row counts cached for the loaded file (opaque to
 *  duck — it's ExplorePane's GroupInfo[]). Returning this on a re-mount avoids
 *  re-parsing the whole file in wasm + re-running count(*) per group, which
 *  was pure waste on every Explore tab switch. */
export function getLoadedGroups(): unknown {
  return loadedGroups;
}

/** Record `bytes` as the ingested file + cache its computed group info, so a
 *  re-mount of the Explore pane returns instantly. */
export function markLoaded(bytes: Uint8Array, groups?: unknown): void {
  loadedFile = bytes;
  if (groups !== undefined) loadedGroups = groups;
}

// A query error in DuckDB-wasm can fail to REJECT `conn.query` — the worker
// raises, but the pending promise never settles, so the SQL console (and every
// later query) would hang on "Running…" forever, showing stale results. So we
// race a timeout and, on any failure, swap in a fresh connection before
// re-throwing: the error surfaces AND the next query works.
const QUERY_TIMEOUT_MS = 8_000;

/** Run a SQL query against the ingested tables. Resilient to the wedge above:
 *  resolves the result, or rejects (timeout or real error) after resetting the
 *  connection so the console recovers. */
export async function run(sql: string): Promise<Table> {
  const { conn } = await getDuckDb();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      reject(
        new Error(
          `Query exceeded ${QUERY_TIMEOUT_MS / 1000}s — it likely hit an ` +
            `error the engine couldn't report (e.g. comparing a number or ` +
            `date column to text). The engine was reset; adjust the query ` +
            `and run again.`,
        ),
      );
    }, QUERY_TIMEOUT_MS);
  });
  try {
    return await Promise.race([conn.query(sql), timeout]);
  } catch (e) {
    await reconnect();
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

/** Replace a (possibly wedged) connection with a fresh one on the same db, so
 *  a failed query can't permanently hang the console. Tables are db-level, so
 *  they survive the reconnect. If reconnecting itself fails (the worker is
 *  dead), drop the whole instance so the next call re-instantiates. */
async function reconnect(): Promise<void> {
  if (!instance) return;
  const prev = instance;
  try {
    const { db } = await prev;
    // db.connect can itself hang against a wedged worker — bound it, so run()'s
    // rejection (and thus the error message) is never blocked.
    const conn = await Promise.race([
      db.connect(),
      new Promise<AsyncDuckDBConnection>((_, reject) =>
        setTimeout(() => {
          reject(new Error("reconnect timed out"));
        }, 3_000),
      ),
    ]);
    if (instance === prev) instance = Promise.resolve({ db, conn });
  } catch {
    // The worker itself is unusable — drop the instance so the next call
    // re-instantiates (tables are lost; the Explore pane re-ingests on the
    // next file load / tab re-mount).
    if (instance === prev) {
      instance = null;
      ingested.clear();
      loadedFile = null;
      loadedGroups = null;
    }
  }
}

let exportSeq = 0;

/** Export a query's result to CSV / JSON / Parquet bytes via DuckDB COPY to
 *  its in-memory FS, then read the buffer back. Parquet may need an
 *  extension that can't autoload offline — callers should fall back to CSV
 *  on error. */
export async function exportQuery(
  sql: string,
  format: "csv" | "json" | "parquet",
): Promise<Uint8Array> {
  const { db, conn } = await getDuckDb();
  const file = `__export_${exportSeq++}.${format}`;
  const opts =
    format === "csv"
      ? "(FORMAT csv, HEADER)"
      : format === "json"
        ? "(FORMAT json, ARRAY true)"
        : "(FORMAT parquet)";
  // Wrapping the query as a COPY subquery — a trailing ";" (natural when the
  // SQL came from the console) would close the COPY early and error.
  const inner = sql.replace(/[\s;]+$/, "");
  try {
    await conn.query(`COPY (${inner}) TO '${file}' ${opts}`);
    return await db.copyFileToBuffer(file);
  } finally {
    await db.dropFile(file).catch(() => {});
  }
}

/** Drop every ingested table and clear the ingested-set so the next file
 *  re-ingests from scratch. Awaited (not fire-and-forget) so a re-ingest of
 *  the same code can't race a late DROP of the freshly-created table. No-op
 *  if DuckDB was never instantiated. */
export async function resetDuck(): Promise<void> {
  const codes = [...ingested];
  ingested.clear();
  loadedFile = null;
  loadedGroups = null;
  if (!instance) return;
  const { conn } = await instance;
  for (const c of codes) {
    await conn.query(`DROP TABLE IF EXISTS "${c}"`).catch(() => {});
  }
}
