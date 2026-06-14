// A key-filtered view over an Ags4File's engine — the Node port of laterite-py's
// `_AgsSubset`, returned by `Ags4File.at()`. Filters ACCUMULATE by chaining
// (`at("LOCA", […]).at("SAMP", […])` keeps both): `table(code)` applies every
// filter whose key column is present in `code` (others ignored), so groups
// carrying none of the keys pass through unfiltered. Async, because the engine
// is (`@duckdb/node-api` is promise-based).
import type { Table } from "apache-arrow";
import type { Ags4File } from "./ags4-file";
import type { QueryOptions, Row } from "./duckdb";

/** A `(keyColumn, values)` filter, e.g. `["LOCA_ID", ["BH01", "BH02"]]`. */
export type Filter = [key: string, values: unknown[]];

export class AgsSubset {
  readonly #parent: Ags4File;
  readonly #filters: Filter[];

  constructor(parent: Ags4File, filters: Filter[]) {
    this.#parent = parent;
    this.#filters = filters;
  }

  /** Narrow further by another entity's id (e.g. add a SAMP filter). */
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

  /** `code` filtered by every applicable key (groups carrying none pass
   * through), as JS-native row objects (or an arrow-js `Table` with
   * `{ arrow: true }`). */
  table(code: string): Promise<Row[]>;
  table(code: string, opts: { arrow: true }): Promise<Table>;
  table(code: string, opts?: QueryOptions): Promise<Row[] | Table>;
  table(code: string, opts?: QueryOptions): Promise<Row[] | Table> {
    // `_filteredRows` is the Ags4File internal that owns the engine + SQL.
    return this.#parent._filteredRows(code, this.#filters, opts);
  }

  /** `{group: rows}` for every related group, each filtered — a location's whole
   * related record set in one call (arrow-js Tables with `{ arrow: true }`). */
  frames(): Promise<Record<string, Row[]>>;
  frames(opts: { arrow: true }): Promise<Record<string, Table>>;
  frames(opts?: QueryOptions): Promise<Record<string, Row[] | Table>>;
  async frames(opts: QueryOptions = {}): Promise<Record<string, Row[] | Table>> {
    const out: Record<string, Row[] | Table> = {};
    for (const g of this.groups) out[g] = await this.table(g, opts);
    return out;
  }
}
