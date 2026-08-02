// `selectSql`'s JOIN mode, and the two conversions that feed it.
//
// The single-table path is well covered; the join path is where the explorer
// actually goes wrong. Every one of its decisions is a silent failure:
//
//   * an unqualified column in a join is either a SQL error or — when both sides
//     carry the same heading, which is the NORMAL case for `LOCA_ID` — an
//     ambiguity DuckDB resolves by picking one, so the query runs and returns
//     the wrong column;
//   * the base alias defaults to `t0`, and every predicate, ORDER BY and star
//     expansion is written against it. A mismatch produces "no such table";
//   * `dedupeOut` renames colliding output names. Without it a two-table join on
//     `LOCA_ID` returns two columns of the same name and the grid shows one.
import { describe, expect, it } from "vitest";

import type { JoinSpec, QualifiedCol, SelectOpts } from "./sqlgen";
import { chartSql, selectSql } from "./sqlgen";

const LOCA_SAMP: JoinSpec = {
  table: "SAMP",
  alias: "t1",
  kind: "LEFT",
  leftAlias: "t0",
  on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
};

/** A complete join-mode request; each test overrides what it is about. */
function opts(over: Partial<SelectOpts> = {}): SelectOpts {
  return {
    table: "LOCA",
    columns: [],
    conditions: [],
    orderDir: "ASC",
    limit: 0,
    joins: [LOCA_SAMP],
    ...over,
  };
}

describe("join-mode SELECT", () => {
  it("aliases the base table and joins on the qualified key pair", () => {
    const sql = selectSql(opts());
    expect(sql).toContain('FROM "LOCA" t0');
    expect(sql).toContain('LEFT JOIN "SAMP" t1 ON t0."LOCA_ID" = t1."LOCA_ID"');
  });

  it("defaults the base alias to t0 and honours an explicit one", () => {
    // Everything downstream — the star, the WHERE, the ORDER BY — is written
    // against this alias, so a default that disagreed with `fromJoins` would
    // produce a query referencing a table that does not exist.
    expect(selectSql(opts())).toContain("t0.*");
    const aliased = selectSql(
      opts({ alias: "base", joins: [{ ...LOCA_SAMP, leftAlias: "base" }] }),
    );
    expect(aliased).toContain('FROM "LOCA" base');
    expect(aliased).toContain("base.*");
    expect(aliased).toContain('base."LOCA_ID" = t1."LOCA_ID"');
  });

  it("expands to the base table's star when no columns are picked", () => {
    // `t0.*` and not a bare `*`: a bare star would pull every joined table's
    // columns, which is a different (and much wider) result set.
    const sql = selectSql(opts());
    expect(sql).toContain("SELECT t0.*");
    expect(sql).not.toMatch(/SELECT \*/);
  });

  it("qualifies every picked column and names its output", () => {
    const select: QualifiedCol[] = [
      { alias: "t0", col: "LOCA_ID" },
      { alias: "t1", col: "SAMP_TOP", as: "top" },
    ];
    const sql = selectSql(opts({ select }));
    expect(sql).toContain('t0."LOCA_ID" AS "LOCA_ID"');
    expect(sql).toContain('t1."SAMP_TOP" AS "top"');
  });

  it("renames a colliding output name rather than emitting it twice", () => {
    // The normal case, not an edge one: both sides of a LOCA→SAMP join carry
    // LOCA_ID. Two columns with one name means the grid silently shows one.
    const select: QualifiedCol[] = [
      { alias: "t0", col: "LOCA_ID" },
      { alias: "t1", col: "LOCA_ID" },
    ];
    const sql = selectSql(opts({ select }));
    const outNames = [...sql.matchAll(/AS "([^"]+)"/g)].map((m) => m[1]);
    expect(outNames).toHaveLength(2);
    expect(new Set(outNames).size).toBe(2);
    expect(outNames[0]).toBe("LOCA_ID");
  });

  it("qualifies a WHERE column with its own alias, falling back to the base", () => {
    // A condition can name a joined table's column. If it defaulted to the base
    // alias regardless, the filter would apply to the wrong table's column —
    // and with a shared heading name it would still run.
    const sql = selectSql(
      opts({
        conditions: [
          { col: "SAMP_TOP", op: "=", val: "1.5", alias: "t1" },
          { col: "LOCA_ID", op: "=", val: "BH01" },
        ],
      }),
    );
    expect(sql).toContain('t1."SAMP_TOP"');
    expect(sql).toContain('t0."LOCA_ID"');
  });

  it("qualifies ORDER BY with the base alias", () => {
    const sql = selectSql(opts({ orderBy: "LOCA_ID", orderDir: "DESC" }));
    expect(sql).toContain('ORDER BY t0."LOCA_ID" DESC');
  });

  it("adds LIMIT only when one is asked for, and floors it", () => {
    expect(selectSql(opts({ limit: 0 }))).not.toContain("LIMIT");
    expect(selectSql(opts({ limit: 10.9 }))).toContain("LIMIT 10");
  });

  it("emits an INNER join when asked, so unmatched base rows drop", () => {
    // LEFT vs INNER is the difference between "every borehole, samples where
    // they exist" and "only boreholes that have samples" — a different answer,
    // not a different formatting.
    const sql = selectSql(
      opts({ joins: [{ ...LOCA_SAMP, kind: "INNER" as const }] }),
    );
    expect(sql).toContain('INNER JOIN "SAMP" t1');
    expect(sql).not.toContain("LEFT JOIN");
  });

  it("adds the half-open depth band for a range join", () => {
    // The GEOL stratum case: a sample at exactly the base depth belongs to the
    // NEXT stratum, so the upper bound must be strict. `<=` here would put
    // boundary samples in two strata at once.
    const sql = selectSql(
      opts({
        joins: [
          {
            table: "GEOL",
            alias: "t1",
            kind: "LEFT",
            leftAlias: "t0",
            on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
            range: {
              baseAlias: "t0",
              baseCol: "SAMP_TOP",
              top: "GEOL_TOP",
              base: "GEOL_BASE",
            },
          },
        ],
      }),
    );
    expect(sql).toContain('t0."SAMP_TOP" >= t1."GEOL_TOP"');
    expect(sql).toContain('t0."SAMP_TOP" < t1."GEOL_BASE"');
    expect(sql).not.toContain('<= t1."GEOL_BASE"');
  });

  it("returns nothing at all without a table", () => {
    expect(selectSql(opts({ table: "" }))).toBe("");
  });

  it("takes the single-table path when the join list is empty", () => {
    // The `!o.joins || o.joins.length === 0` guard. An empty array must not
    // fall into join mode and emit `t0.*` against an unaliased FROM.
    const sql = selectSql(opts({ joins: [] }));
    expect(sql).toContain('FROM "LOCA"');
    expect(sql).not.toContain("t0");
  });
});

describe("join-mode chartSql", () => {
  // `colRef` decides how a column is written. A chart's X/Y arrive as bare
  // strings when the user picks them from the base table's own list — and in a
  // join those must still be alias-qualified, because the ONE column both sides
  // of a LOCA→SAMP join always share is the join key itself. An unqualified
  // reference there is either a DuckDB ambiguity error or, worse, a chart drawn
  // from the wrong table's column.
  const base = {
    table: "LOCA",
    joins: [LOCA_SAMP],
    chartType: "scatter" as const,
    agg: "none" as const,
    rowCap: 5000,
  };

  it("qualifies a bare string column with the base alias when joined", () => {
    const sql = chartSql({ ...base, x: "LOCA_GL", y: "LOCA_FDEP" });
    expect(sql).toContain('t0."LOCA_GL" AS x');
    expect(sql).toContain('t0."LOCA_FDEP" AS y');
    expect(sql).toContain('LEFT JOIN "SAMP" t1');
    // The output aliases stay x/y whatever the qualification, since the chart
    // component maps on those names.
    expect(sql).toMatch(/AS x\b/);
  });

  it("leaves a bare string column unqualified with no joins", () => {
    // The other arm of the same ternary: single-table charts must not emit an
    // alias no FROM clause introduces.
    const sql = chartSql({ ...base, joins: [], x: "LOCA_GL", y: "LOCA_FDEP" });
    expect(sql).toContain('"LOCA_GL" AS x');
    expect(sql).not.toContain("t0.");
    expect(sql).toContain('FROM "LOCA"');
  });

  it("uses a QualifiedCol's own alias rather than the base one", () => {
    // Plotting the joined table's column is the reason to join at all; taking
    // the base alias here would silently chart LOCA's data as SAMP's.
    const sql = chartSql({
      ...base,
      x: "LOCA_GL",
      y: { alias: "t1", col: "SAMP_TOP" },
      colour: { alias: "t1", col: "SAMP_TYPE" },
    });
    expect(sql).toContain('t1."SAMP_TOP" AS y');
    expect(sql).toContain('t1."SAMP_TYPE" AS c');
    expect(sql).toContain('t0."LOCA_GL" AS x');
  });

  it("groups by the qualified refs when aggregating over a join", () => {
    // GROUP BY has to name the same references the SELECT does; a mismatch is a
    // "must appear in the GROUP BY clause" error the user cannot act on.
    const sql = chartSql({
      ...base,
      chartType: "bar",
      agg: "avg",
      x: "LOCA_TYPE",
      y: { alias: "t1", col: "SAMP_TOP" },
    });
    expect(sql).toContain('AVG(t1."SAMP_TOP") AS y');
    expect(sql).toContain('GROUP BY t0."LOCA_TYPE"');
    expect(sql).toContain('WHERE t1."SAMP_TOP" IS NOT NULL');
  });
});
