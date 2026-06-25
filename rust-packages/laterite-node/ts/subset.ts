import type { Table } from "apache-arrow";
import type { Ags4File } from "./ags4-file";
import type { QueryOptions, Row } from "./duckdb";

/** A `(keyColumn, values)` filter, e.g. `["LOCA_ID", ["BH01", "BH02"]]`. */
export type Filter = [key: string, values: unknown[]];

/**
 * A key-filtered view over an `Ags4File`'s engine — the Node port of laterite-py's
 * `_AgsSubset`, returned by `Ags4File.at()`. Filters ACCUMULATE by chaining
 * (`at("LOCA", […]).at("SAMP", […])` keeps both): `table(code)` applies every
 * filter whose key column is present in `code` (others ignored), so groups
 * carrying none of the keys pass through unfiltered. Async, because the engine
 * is promise-based (`@duckdb/node-api`).
 */
export class AgsSubset {
  readonly #parent: Ags4File;
  readonly #filters: Filter[];

  constructor(parent: Ags4File, filters: Filter[]) {
    this.#parent = parent;
    this.#filters = filters;
  }

  /**
   * Narrow further by another entity's id (e.g. add a SAMP filter on top of a
   * LOCA one). Returns a fresh subset — filters accumulate, they never mutate.
   *
   * @param group - The group code whose `_ID` key to filter on (`"SAMP"` becomes
   *   the `SAMP_ID` key column).
   * @param values - The ids to keep; a row survives when its `<group>_ID` is one
   *   of these.
   * @returns A new `AgsSubset` carrying this filter plus all the existing ones.
   */
  at(group: string, values: Iterable<unknown>): AgsSubset {
    return new AgsSubset(this.#parent, [...this.#filters, [`${group}_ID`, [...values]]]);
  }

  /** The related groups — those carrying at least one filter's key column. */
  get groups(): string[] {
    const keys = new Set(this.#filters.map(([k]) => k));
    return this.#parent.groups.filter(
      (g) => this.#parent.has(g) && this.#parent.headings(g).some((h) => keys.has(h)),
    );
  }

  /**
   * `code` filtered by every applicable key (groups carrying none of the keys
   * pass through unfiltered), as JS-native row objects by default — or a
   * born-typed arrow-js `Table` with `{ arrow: true }`.
   *
   * @param code - The group to read (its clean name).
   * @param opts - Query options; `{ arrow: true }` switches the return to an
   *   arrow-js `Table`.
   * @returns The filtered rows as `Row[]`, or a `Table` when arrow output is
   *   requested.
   * @throws If `{ arrow: true }` is set but DuckDB's `arrow` community extension
   *   can't be installed/loaded (offline or air-gapped).
   */
  table(code: string): Promise<Row[]>;
  table(code: string, opts: { arrow: true }): Promise<Table>;
  table(code: string, opts?: QueryOptions): Promise<Row[] | Table>;
  table(code: string, opts?: QueryOptions): Promise<Row[] | Table> {
    // `_filteredRows` is the Ags4File internal that owns the engine + SQL.
    return this.#parent._filteredRows(code, this.#filters, opts);
  }

  /**
   * `{group: rows}` for every related group, each filtered — a location's whole
   * related record set in one call. Returns arrow-js `Table`s with
   * `{ arrow: true }`, JS-native row objects otherwise.
   *
   * @param opts - Query options; `{ arrow: true }` makes each value a `Table`.
   * @returns A map from group code to its filtered rows (or `Table`).
   * @throws If `{ arrow: true }` is set but DuckDB's `arrow` community extension
   *   can't be installed/loaded (offline or air-gapped).
   */
  frames(): Promise<Record<string, Row[]>>;
  frames(opts: { arrow: true }): Promise<Record<string, Table>>;
  frames(opts?: QueryOptions): Promise<Record<string, Row[] | Table>>;
  async frames(opts: QueryOptions = {}): Promise<Record<string, Row[] | Table>> {
    const out: Record<string, Row[] | Table> = {};
    for (const g of this.groups) out[g] = await this.table(g, opts);
    return out;
  }
}
