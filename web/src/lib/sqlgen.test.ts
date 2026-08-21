import { describe, expect, it } from "vitest";
import {
  chartSql,
  chartRankSql,
  selectSql,
  lit,
  likeLiteral,
  type JoinSpec,
} from "./sqlgen";

describe("chartSql", () => {
  it("composes a scatter query (raw X/Y, both non-null)", () => {
    expect(
      chartSql({
        table: "LOCA",
        x: "LOCA_NATE",
        y: "LOCA_NATN",
        chartType: "scatter",
        agg: "none",
        rowCap: 5000,
      }),
    ).toBe(
      `SELECT "LOCA_NATE" AS x, "LOCA_NATN" AS y FROM "LOCA" WHERE "LOCA_NATE" IS NOT NULL AND "LOCA_NATN" IS NOT NULL LIMIT 5000`,
    );
  });

  it("orders a line chart by X", () => {
    const s = chartSql({
      table: "T",
      x: "d",
      y: "v",
      chartType: "line",
      agg: "none",
      rowCap: 10,
    });
    expect(s).toContain(`ORDER BY "d" LIMIT 10`);
  });

  it("bar + count needs no Y and emits COUNT(*) grouped by X", () => {
    expect(
      chartSql({
        table: "GEOL",
        x: "GEOL_LEG",
        y: "",
        chartType: "bar",
        agg: "count",
        rowCap: 5000,
      }),
    ).toBe(
      `SELECT "GEOL_LEG" AS x, COUNT(*) AS y FROM "GEOL" GROUP BY "GEOL_LEG" ORDER BY x LIMIT 5000`,
    );
  });

  it("declines to compose an aggregate + colour without the probe's answer", () => {
    // The two-phase contract, held HERE rather than in the component (#457).
    // On this path the colour column is a group key, so the fold has to happen
    // inside the GROUP BY or not at all — and what survives is the probe's
    // answer. Composing without it would emit a `c` the assembler cannot match
    // against the ranking, which paints every series neutral: a chart that
    // looks drawn and is wrong. "" is the loud answer.
    expect(
      chartSql({
        table: "T",
        x: "g",
        y: "v",
        colour: "k",
        chartType: "bar",
        agg: "avg",
        rowCap: 100,
      }),
    ).toBe("");
  });

  it("bar + avg + colour folds the tail inside the GROUP BY", () => {
    expect(
      chartSql({
        table: "T",
        x: "g",
        y: "v",
        colour: "k",
        chartType: "bar",
        agg: "avg",
        rowCap: 100,
        fold: { keep: ["a", "b"], label: "Other" },
      }),
    ).toBe(
      `SELECT "g" AS x, AVG("v") AS y,` +
        ` CASE WHEN COALESCE(CAST("k" AS VARCHAR), '') IN ('a', 'b')` +
        ` THEN COALESCE(CAST("k" AS VARCHAR), '') ELSE 'Other' END AS c` +
        ` FROM "T" WHERE "v" IS NOT NULL` +
        ` GROUP BY "g", CASE WHEN COALESCE(CAST("k" AS VARCHAR), '') IN ('a', 'b')` +
        ` THEN COALESCE(CAST("k" AS VARCHAR), '') ELSE 'Other' END` +
        ` ORDER BY x LIMIT 100`,
    );
  });

  it("bar + count + colour: COUNT(*), no Y filter, folded and grouped", () => {
    expect(
      chartSql({
        table: "T",
        x: "g",
        y: "",
        colour: "k",
        chartType: "bar",
        agg: "count",
        rowCap: 100,
        fold: { keep: ["a"], label: "Other" },
      }),
    ).toBe(
      `SELECT "g" AS x, COUNT(*) AS y,` +
        ` CASE WHEN COALESCE(CAST("k" AS VARCHAR), '') IN ('a')` +
        ` THEN COALESCE(CAST("k" AS VARCHAR), '') ELSE 'Other' END AS c` +
        ` FROM "T"` +
        ` GROUP BY "g", CASE WHEN COALESCE(CAST("k" AS VARCHAR), '') IN ('a')` +
        ` THEN COALESCE(CAST("k" AS VARCHAR), '') ELSE 'Other' END` +
        ` ORDER BY x LIMIT 100`,
    );
  });

  it("aggregates with no colour at all, exactly as before", () => {
    // The fold only exists where a colour column does; nothing on this path
    // changed, and a `fold` handed in anyway is not a group key to apply.
    const bare = {
      table: "GEOL" as const,
      x: "GEOL_LEG",
      y: "",
      chartType: "bar" as const,
      agg: "count" as const,
      rowCap: 5000,
    };
    const expected = `SELECT "GEOL_LEG" AS x, COUNT(*) AS y FROM "GEOL" GROUP BY "GEOL_LEG" ORDER BY x LIMIT 5000`;
    expect(chartSql(bare)).toBe(expected);
    expect(chartSql({ ...bare, fold: { keep: ["a"], label: "Other" } })).toBe(
      expected,
    );
  });

  it("returns '' when the selection is incomplete", () => {
    expect(
      chartSql({
        table: "",
        x: "a",
        y: "b",
        chartType: "scatter",
        agg: "none",
        rowCap: 9,
      }),
    ).toBe("");
    expect(
      chartSql({
        table: "T",
        x: "",
        y: "b",
        chartType: "scatter",
        agg: "none",
        rowCap: 9,
      }),
    ).toBe("");
    // non-counting needs a Y
    expect(
      chartSql({
        table: "T",
        x: "a",
        y: "",
        chartType: "scatter",
        agg: "none",
        rowCap: 9,
      }),
    ).toBe("");
  });

  it("quotes identifiers so a rogue quote can't break out", () => {
    const s = chartSql({
      table: "T",
      x: 'a"b',
      y: "v",
      chartType: "scatter",
      agg: "none",
      rowCap: 5,
    });
    expect(s).toContain(`"a""b"`);
  });
});

// The tail fold, on the one path that can merge it correctly (#457). An
// aggregating bar groups by (x, colour), so folding in the assembler leaves the
// tail with one point per folded value PER CATEGORY, drawn on top of each other
// — a single visible bar with no sign that more than one value is under it.
// Inside the GROUP BY the aggregate is computed over the merged group's own
// rows instead, which makes `avg` as correct as `sum`.
describe("chartSql — the fold", () => {
  const bar = {
    table: "T",
    x: "g",
    y: "v",
    colour: "k",
    chartType: "bar" as const,
    agg: "avg" as const,
    rowCap: 100,
  };

  it("quotes every survivor, including one that looks like a number", () => {
    // NOT through `lit`, whose bare-number branch is a trap here: DuckDB reads
    // `"k" IN (1)` as a NUMERIC comparison, which matches the string '01' to 1
    // and hard-errors on the first non-numeric row in the column. The left side
    // is text by construction, so the literals are too.
    const s = chartSql({ ...bar, fold: { keep: ["1", "01"], label: "Other" } });
    expect(s).toContain(`IN ('1', '01')`);
    expect(s).not.toContain(`IN (1,`);
  });

  it("escapes a quote in a survivor and in the label", () => {
    const s = chartSql({
      ...bar,
      fold: { keep: ["O'Brien"], label: "Other's" },
    });
    expect(s).toContain(`IN ('O''Brien')`);
    expect(s).toContain(`ELSE 'Other''s'`);
  });

  it("emits the caller's label, so a value named 'Other' keeps its own bar", () => {
    // `foldLabel` steps the fold aside from a survivor carrying that name, and
    // the query now MATERIALISES that label as data — so it has to be the same
    // string the legend will show, not a second guess at it.
    const s = chartSql({
      ...bar,
      fold: { keep: ["Other", "b"], label: "Other (2)" },
    });
    expect(s).toContain(`IN ('Other', 'b')`);
    expect(s).toContain(`ELSE 'Other (2)' END`);
  });

  it("keeps the CASE when the probe ranked nothing, on a predicate that cannot hold", () => {
    // SQL has no empty `IN ()`, and the answer is NOT to drop the CASE: that
    // composes a differently shaped query off an empty list — every distinct
    // value keeping its own group, which is the behaviour the fold exists to
    // prevent. One shape, so an empty list can only mean "everything folds".
    const s = chartSql({ ...bar, fold: { keep: [], label: "Other" } });
    expect(s).toContain(
      `CASE WHEN FALSE THEN COALESCE(CAST("k" AS VARCHAR), '') ELSE 'Other' END AS c`,
    );
    expect(s).not.toContain("IN (");
    // The shape is the same one a non-empty probe answer composes.
    const filled = chartSql({ ...bar, fold: { keep: ["a"], label: "Other" } });
    const shape = (q: string) =>
      q.replace(/CASE WHEN .*? THEN/g, "CASE WHEN … THEN");
    expect(shape(s)).toBe(shape(filled));
  });

  it.each(["sum", "avg", "min", "max"] as const)(
    "computes %s over the folded group rather than over its folded parts",
    (agg) => {
      // The merge IS the grouping: every aggregate the form offers runs over
      // the merged group's own rows, so none of them needs a merge rule of its
      // own. `avg` is the one that could not be merged after the fact, and it
      // takes this path unchanged alongside the three that could.
      const s = chartSql({
        ...bar,
        agg,
        fold: { keep: ["a"], label: "Other" },
      });
      expect(s).toContain(`${agg.toUpperCase()}("v") AS y`);
      expect(s).toContain(`GROUP BY "g", CASE WHEN`);
      expect(s).toContain(`ELSE 'Other' END ORDER BY x`);
    },
  );

  it("leaves scatter and line composing exactly as they did", () => {
    // Their pooling is CORRECT — a point cloud has nothing to merge — so the
    // fold must not reach them even when one is handed over.
    const raw = { ...bar, chartType: "scatter" as const, agg: "none" as const };
    const expected = `SELECT "g" AS x, "v" AS y, "k" AS c FROM "T" WHERE "g" IS NOT NULL AND "v" IS NOT NULL LIMIT 100`;
    expect(chartSql(raw)).toBe(expected);
    expect(chartSql({ ...raw, fold: { keep: ["a"], label: "Other" } })).toBe(
      expected,
    );
    // Bar with NO aggregate is a raw plot too, and takes the same path.
    expect(
      chartSql({
        ...raw,
        chartType: "bar",
        fold: { keep: ["a"], label: "Other" },
      }),
    ).toContain(`"k" AS c`);
  });

  it("folds a colour that came from the joined table", () => {
    const s = chartSql({
      ...bar,
      alias: "t0",
      joins: [
        {
          table: "GEOL",
          alias: "t1",
          kind: "LEFT",
          leftAlias: "t0",
          on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
        },
      ],
      colour: { alias: "t1", col: "GEOL_LEG" },
      fold: { keep: ["CL"], label: "Other" },
    });
    expect(s).toContain(`COALESCE(CAST(t1."GEOL_LEG" AS VARCHAR), '')`);
    expect(s).toContain(`GROUP BY t0."g", CASE WHEN`);
  });
});

// The probe that decides which colour-by values keep a palette slot. What it
// must NOT be is a read of the plotted rows: the scatter path is a bare row
// LIMIT with no ORDER BY, so the values it returns are an arbitrary slice.
describe("chartRankSql", () => {
  const base = {
    table: "LOCA",
    x: "LOCA_NATE",
    y: "LOCA_NATN",
    colour: "LOCA_TYPE",
    chartType: "scatter" as const,
    agg: "none" as const,
    cap: 3,
  };

  it("ranks by row count, breaking ties on the value", () => {
    // Deterministic ties are what stop the same delivery assigning different
    // colours on two loads.
    expect(chartRankSql(base)).toBe(
      `SELECT "LOCA_TYPE" AS c, COUNT(*) AS n FROM "LOCA"` +
        ` WHERE "LOCA_NATE" IS NOT NULL AND "LOCA_NATN" IS NOT NULL` +
        ` GROUP BY "LOCA_TYPE" ORDER BY n DESC, c ASC LIMIT 3`,
    );
  });

  it("asks for exactly the cap, and no probe row for the tail", () => {
    // Nothing needs to know a tail EXISTS: a value the probe did not return
    // folds by being absent from the list, which is the same thing that happens
    // to one it returned past the cap. A `cap + 1` row would be read by no one.
    expect(chartRankSql({ ...base, cap: 5 })).toContain("LIMIT 5");
  });

  it("carries the plot's own row filter, so both count the same rows", () => {
    // An aggregate has already collapsed X, so the plot filters on Y alone —
    // and a probe that filtered on both would rank over a smaller population
    // than the chart draws.
    expect(chartRankSql({ ...base, chartType: "bar", agg: "avg" })).toContain(
      `FROM "LOCA" WHERE "LOCA_NATN" IS NOT NULL GROUP BY`,
    );
    // A COUNT counts rows, so no row is excluded — and neither query filters.
    expect(
      chartRankSql({ ...base, y: "", chartType: "bar", agg: "count" }),
    ).toContain(`FROM "LOCA" GROUP BY`);
  });

  it("ranks the same rendering of the value the plot query will emit", () => {
    // The assembler matches the probe's values against the plot's by STRING,
    // and on the aggregating path the plot has to render the colour as text to
    // name the fold at all — DuckDB writes a DOUBLE as '1.0' where JS writes
    // '1'. Rank the raw value there and no survivor would ever match its own
    // rows: every series would fall into the neutral (#457).
    expect(chartRankSql({ ...base, chartType: "bar", agg: "avg" })).toContain(
      `SELECT COALESCE(CAST("LOCA_TYPE" AS VARCHAR), '') AS c, COUNT(*) AS n`,
    );
    expect(chartRankSql({ ...base, chartType: "bar", agg: "avg" })).toContain(
      `GROUP BY COALESCE(CAST("LOCA_TYPE" AS VARCHAR), '') ORDER BY n DESC`,
    );
    // Scatter and line rank the raw value, exactly as they did.
    expect(chartRankSql(base)).toContain(`SELECT "LOCA_TYPE" AS c`);
    expect(chartRankSql({ ...base, chartType: "line" })).not.toContain("CAST");
  });

  it("renders the colour identically to the plot query it ranks for", () => {
    // The invariant the two-phase path stands on. The assembler matches the
    // probe's values against the plotted rows' BY STRING, so if these two
    // expressions ever diverge no survivor matches its own rows: every series
    // painted neutral, under a legend naming none of them, with no error
    // anywhere. Derived from both strings rather than restated, so a change to
    // one composer alone cannot leave this green.
    const expr = (sql: string) =>
      /COALESCE\(CAST\(.*? AS VARCHAR\), ''\)/.exec(sql)?.[0];
    const opts = { ...base, chartType: "bar" as const, agg: "avg" as const };
    const probe = chartRankSql({ ...opts, cap: 3 });
    const plot = chartSql({
      ...opts,
      rowCap: 100,
      fold: { keep: ["A"], label: "Other" },
    });
    expect(expr(probe)).toBeDefined();
    expect(expr(plot)).toBe(expr(probe));
  });

  it("returns '' on exactly the selections the plot query declines", () => {
    expect(chartRankSql({ ...base, table: "" })).toBe("");
    expect(chartRankSql({ ...base, x: "" })).toBe("");
    expect(chartRankSql({ ...base, y: "" })).toBe("");
  });

  it("quotes identifiers so a rogue quote can't break out", () => {
    expect(chartRankSql({ ...base, colour: 'a"b' })).toContain(`"a""b"`);
  });
});

describe("selectSql", () => {
  it("SELECT * when no columns are picked", () => {
    expect(
      selectSql({
        table: "LOCA",
        columns: [],
        conditions: [],
        orderDir: "ASC",
        limit: 100,
      }),
    ).toBe(`SELECT *\nFROM "LOCA"\nLIMIT 100`);
  });

  it("selects specific columns + ORDER BY + LIMIT", () => {
    expect(
      selectSql({
        table: "T",
        columns: ["a", "b"],
        conditions: [],
        orderBy: "a",
        orderDir: "DESC",
        limit: 50,
      }),
    ).toBe(`SELECT "a", "b"\nFROM "T"\nORDER BY "a" DESC\nLIMIT 50`);
  });

  it("builds WHERE: numeric unquoted, string quoted, IS NULL valueless", () => {
    const s = selectSql({
      table: "T",
      columns: [],
      conditions: [
        { col: "n", op: ">", val: "5" },
        { col: "s", op: "=", val: "BH01" },
        { col: "x", op: "IS NULL", val: "" },
      ],
      orderDir: "ASC",
      limit: 0,
    });
    expect(s).toBe(
      `SELECT *\nFROM "T"\nWHERE "n" > 5\n  AND "s" = 'BH01'\n  AND "x" IS NULL`,
    );
  });

  it("omits LIMIT when 0 and returns '' with no table", () => {
    expect(
      selectSql({
        table: "",
        columns: [],
        conditions: [],
        orderDir: "ASC",
        limit: 10,
      }),
    ).toBe("");
  });

  it("skips an incomplete value-condition (no `\"COL\" = ''`)", () => {
    // The builder seeds a new filter with an empty value; emitting `= ''`
    // against a numeric/date column is a DuckDB conversion error that wedges
    // the engine — so an empty-value value-op is dropped until a value is set.
    const s = selectSql({
      table: "LOCA",
      columns: [],
      conditions: [
        { col: "LOCA_NATE", op: "=", val: "" }, // incomplete → skipped
        { col: "LOCA_ID", op: "=", val: "BH01" }, // complete → kept
      ],
      orderDir: "ASC",
      limit: 100,
    });
    expect(s).toBe(
      `SELECT *\nFROM "LOCA"\nWHERE "LOCA_ID" = 'BH01'\nLIMIT 100`,
    );
    // All-incomplete ⇒ no WHERE clause at all (valid query, returns rows).
    const none = selectSql({
      table: "LOCA",
      columns: [],
      conditions: [{ col: "LOCA_NATE", op: "=", val: "" }],
      orderDir: "ASC",
      limit: 100,
    });
    expect(none).toBe(`SELECT *\nFROM "LOCA"\nLIMIT 100`);
  });
});

describe("lit", () => {
  it("leaves a number bare and quotes/escapes a string", () => {
    expect(lit("42")).toBe("42");
    expect(lit("-3.14")).toBe("-3.14");
    expect(lit("BH01")).toBe("'BH01'");
    expect(lit("a'b")).toBe("'a''b'");
  });
});

describe("likeLiteral (LIKE wildcard placement)", () => {
  it("wraps per mode and defaults to contains", () => {
    expect(likeLiteral("ab", "contains")).toBe("'%ab%'");
    expect(likeLiteral("ab", "starts")).toBe("'ab%'");
    expect(likeLiteral("ab", "ends")).toBe("'%ab'");
    expect(likeLiteral("ab", "exact")).toBe("'ab'");
    expect(likeLiteral("ab")).toBe("'%ab%'");
  });
  it("escapes LIKE metacharacters (\\ % _) and the quote so user input is literal", () => {
    // a%b → the % is escaped (\%), then wrapped; ' is SQL-doubled.
    expect(likeLiteral("a%b", "contains")).toBe("'%a\\%b%'");
    expect(likeLiteral("a_b", "starts")).toBe("'a\\_b%'");
    expect(likeLiteral("o'h", "exact")).toBe("'o''h'");
    // A literal backslash is itself escaped (→ \\), so paired with ESCAPE '\'
    // it matches one backslash rather than starting an escape sequence.
    expect(likeLiteral("a\\b", "contains")).toBe("'%a\\\\b%'");
  });
});

describe("selectSql — LIKE in the single-table path", () => {
  it("emits the wrapped pattern + ESCAPE", () => {
    const s = selectSql({
      table: "GEOL",
      columns: [],
      conditions: [
        { col: "GEOL_DESC", op: "LIKE", val: "CLAY", wildcard: "starts" },
      ],
      orderDir: "ASC",
      limit: 0,
    });
    expect(s).toBe(
      `SELECT *\nFROM "GEOL"\nWHERE "GEOL_DESC" LIKE 'CLAY%' ESCAPE '\\'`,
    );
  });
});

describe("selectSql — join mode", () => {
  it("equi-joins, qualifies the SELECT list, and dedupes colliding output names", () => {
    const join: JoinSpec = {
      table: "LOCA",
      alias: "p",
      kind: "LEFT",
      leftAlias: "c",
      on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
    };
    const s = selectSql({
      table: "SAMP",
      alias: "c",
      joins: [join],
      select: [
        { alias: "c", col: "LOCA_ID" },
        { alias: "c", col: "SAMP_ID" },
        { alias: "p", col: "LOCA_ID" }, // collides with c.LOCA_ID
        { alias: "p", col: "LOCA_TYPE" },
      ],
      columns: [],
      conditions: [],
      orderDir: "ASC",
      limit: 100,
    });
    expect(s).toBe(
      `SELECT c."LOCA_ID" AS "LOCA_ID", c."SAMP_ID" AS "SAMP_ID",` +
        ` p."LOCA_ID" AS "p_LOCA_ID", p."LOCA_TYPE" AS "LOCA_TYPE"\n` +
        `FROM "SAMP" c\n` +
        `LEFT JOIN "LOCA" p ON c."LOCA_ID" = p."LOCA_ID"\n` +
        `LIMIT 100`,
    );
  });

  it("dedupes a 3-way output-name collision (name, alias_name, alias_name_2)", () => {
    const join: JoinSpec = {
      table: "LOCA",
      alias: "p",
      kind: "LEFT",
      leftAlias: "c",
      on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
    };
    const s = selectSql({
      table: "SAMP",
      alias: "c",
      joins: [join],
      // Three identically-named picks force the numeric-suffix branch.
      select: [
        { alias: "p", col: "LOCA_ID" },
        { alias: "p", col: "LOCA_ID" },
        { alias: "p", col: "LOCA_ID" },
      ],
      columns: [],
      conditions: [],
      orderDir: "ASC",
      limit: 0,
    });
    expect(s).toContain(
      `SELECT p."LOCA_ID" AS "LOCA_ID", p."LOCA_ID" AS "p_LOCA_ID",` +
        ` p."LOCA_ID" AS "p_LOCA_ID_2"`,
    );
  });

  it("range join: a half-open depth band (>= top AND < base)", () => {
    const join: JoinSpec = {
      table: "GEOL",
      alias: "g",
      kind: "LEFT",
      leftAlias: "t",
      on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
      range: {
        baseAlias: "t",
        baseCol: "SPEC_DPTH",
        top: "GEOL_TOP",
        base: "GEOL_BASE",
      },
    };
    const s = selectSql({
      table: "TREG",
      alias: "t",
      joins: [join],
      select: [
        { alias: "t", col: "SPEC_DPTH" },
        { alias: "g", col: "GEOL_DESC" },
      ],
      columns: [],
      conditions: [],
      orderDir: "ASC",
      limit: 100,
    });
    expect(s).toContain(
      `LEFT JOIN "GEOL" g ON t."LOCA_ID" = g."LOCA_ID"` +
        ` AND t."SPEC_DPTH" >= g."GEOL_TOP" AND t."SPEC_DPTH" < g."GEOL_BASE"`,
    );
  });

  it("qualifies a WHERE condition by its alias", () => {
    const join: JoinSpec = {
      table: "GEOL",
      alias: "g",
      kind: "LEFT",
      leftAlias: "t",
      on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
    };
    const s = selectSql({
      table: "TREG",
      alias: "t",
      joins: [join],
      select: [{ alias: "g", col: "GEOL_LEG" }],
      columns: [],
      conditions: [{ alias: "g", col: "GEOL_LEG", op: "=", val: "CL" }],
      orderDir: "ASC",
      limit: 0,
    });
    expect(s).toContain(`WHERE g."GEOL_LEG" = 'CL'`);
  });
});

describe("chartSql — join mode", () => {
  it("qualifies x/y/colour and emits the join, keeping x/y/c output aliases", () => {
    const join: JoinSpec = {
      table: "GEOL",
      alias: "g",
      kind: "LEFT",
      leftAlias: "t",
      on: [{ left: "LOCA_ID", right: "LOCA_ID" }],
      range: {
        baseAlias: "t",
        baseCol: "SPEC_DPTH",
        top: "GEOL_TOP",
        base: "GEOL_BASE",
      },
    };
    const s = chartSql({
      table: "TREL",
      alias: "t",
      joins: [join],
      x: { alias: "t", col: "SPEC_DPTH" },
      y: { alias: "t", col: "TREL_MNUM" },
      colour: { alias: "g", col: "GEOL_LEG" },
      chartType: "scatter",
      agg: "none",
      rowCap: 5000,
    });
    expect(s).toContain(
      `SELECT t."SPEC_DPTH" AS x, t."TREL_MNUM" AS y, g."GEOL_LEG" AS c`,
    );
    expect(s).toContain(`LEFT JOIN "GEOL" g ON t."LOCA_ID" = g."LOCA_ID"`);
    expect(s).toContain(
      `t."SPEC_DPTH" >= g."GEOL_TOP" AND t."SPEC_DPTH" < g."GEOL_BASE"`,
    );
  });
});
