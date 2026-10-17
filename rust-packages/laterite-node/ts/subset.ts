import type { Table } from "apache-arrow";
import type { Ags4File } from "./ags4-file";
import type { QueryOptions, Row } from "./duckdb";
import { Ags4Error } from "./errors";

/** What an `at()` filter keys on: `undefined` (the default) is the group's
 * `{group}_ID` heading; `"_id"` is the synthetic row identity — the group's own
 * `_id`, its direct children's `_parent_id`. */
export type AtBy = "_id";

/** Options for `at()`. */
export interface AtOptions {
  /** `"_id"` keys on row identity, for a group whose `{group}_ID` is optional
   * (`SAMP_ID`) or absent (`GEOL`). */
  by?: AtBy;
}

/** An `at()` filter, e.g. `["LOCA", undefined, ["BH01", "BH02"]]`. */
export type Filter = [group: string, by: AtBy | undefined, values: unknown[]];

/** Validate one `at()` call against the file and build its filter. Both public
 * `at()`s come through here, so the two can't drift apart.
 * @internal */
export function atFilter(
  parent: Ags4File,
  group: string,
  values: Iterable<unknown>,
  by: string | undefined,
): Filter {
  const vals = [...values];
  if (by === undefined) {
    const key = `${group}_ID`;
    if (!parent.groups.some((g) => parent.headings(g).includes(key))) {
      throw new Ags4Error(
        `no group in this file carries ${key}, so at(${JSON.stringify(group)}, …) ` +
          `would filter nothing — check has(${JSON.stringify(group)}) first, or key ` +
          `on row identity with { by: "_id" }`,
      );
    }
    // A blank ID reads as null, which IN never matches: the rows exist, this
    // filter cannot reach them.
    if (vals.some((v) => v === null || v === undefined)) {
      process.emitWarning(
        `a null in at(${JSON.stringify(group)}, …) matches no row: a blank ${key} ` +
          `reads as null, which IN never matches — key on row identity with ` +
          `{ by: "_id" } to reach those rows`,
      );
    }
  } else if (by === "_id") {
    if (
      !parent.groups.some((g) => g === group || parent._parentOf(g) === group)
    ) {
      throw new Ags4Error(
        `neither ${group} nor any direct child of it is in this file, so ` +
          `at(${JSON.stringify(group)}, …, { by: "_id" }) would filter nothing`,
      );
    }
  } else {
    throw new Ags4Error(
      `at() by takes "_id" or undefined (got ${JSON.stringify(by)})`,
    );
  }
  return [group, by, vals];
}

/** The column one filter narrows `code` on, or `undefined` where it passes
 * through. A `by: "_id"` filter reaches `code` by parentage, not by the columns
 * present: a related group that carries no ids (a keyless declared group) is
 * still reached, and none of its rows can be "at" an id.
 * @internal */
export function keyColumn(
  parent: Ags4File,
  [group, by]: Filter,
  code: string,
  cols: ReadonlySet<string>,
): string | undefined {
  if (by === undefined) {
    const key = `${group}_ID`;
    return cols.has(key) ? key : undefined;
  }
  if (code === group) return "_id";
  if (parent._parentOf(code) === group) return "_parent_id";
  return undefined;
}

/**
 * A key-filtered view over an `Ags4File`'s engine — the Node port of laterite-py's
 * `AgsQuery` fan-out, returned by `Ags4File.at()`. Filters ACCUMULATE by chaining
 * (`at("LOCA", […]).at("SAMP", […])` keeps both): `table(code)` applies every
 * filter that reaches `code`. A group no filter reaches passes through
 * unfiltered — unless a `{ by: "_id" }` filter is present, when `table(code)`
 * rejects instead. Async, because the engine is promise-based
 * (`@duckdb/node-api`).
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
   * @param opts - `{ by: "_id" }` keys on row identity instead: `group`'s rows by
   *   `_id`, its direct children's by `_parent_id`.
   * @returns A new `AgsSubset` carrying this filter plus all the existing ones.
   * @throws {Ags4Error} No group in the file carries `{group}_ID`; with
   *   `{ by: "_id" }`, neither `group` nor a direct child of it is in the file.
   */
  at(
    group: string,
    values: Iterable<unknown>,
    opts: AtOptions = {},
  ): AgsSubset {
    return new AgsSubset(this.#parent, [
      ...this.#filters,
      atFilter(this.#parent, group, values, opts.by),
    ]);
  }

  /** The related groups — those at least one filter reaches with a column to
   * filter on (a `{ by: "_id" }` child that carries no ids is left out). */
  get groups(): string[] {
    const p = this.#parent;
    return p.groups.filter((g) => {
      if (!p.has(g)) return false;
      const headings = new Set(p.headings(g));
      return this.#filters.some((f) => {
        const key = keyColumn(p, f, g, headings);
        return (
          key !== undefined && (f[1] === undefined || p._columns(g).has(key))
        );
      });
    });
  }

  /**
   * `code` filtered by every applicable key, as JS-native row objects by
   * default — or a born-typed arrow-js `Table` with `{ arrow: true }`. A group
   * carrying none of the keys passes through unfiltered, unless a
   * `{ by: "_id" }` filter is present.
   *
   * @param code - The group to read (its clean name).
   * @param opts - Query options; `{ arrow: true }` switches the return to an
   *   arrow-js `Table`.
   * @returns The filtered rows as `Row[]`, or a `Table` when arrow output is
   *   requested.
   * @throws {Ags4Error} A `{ by: "_id" }` filter is present and no filter reaches
   *   `code` (a parent, sibling or grandchild of the filtered group).
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
  async frames(
    opts: QueryOptions = {},
  ): Promise<Record<string, Row[] | Table>> {
    const out: Record<string, Row[] | Table> = {};
    for (const g of this.groups) out[g] = await this.table(g, opts);
    return out;
  }
}
