import { describe, expect, it } from "vitest";
import { chartSql, selectSql, lit, likeLiteral, type JoinSpec } from "./sqlgen";

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

  it("bar + avg + colour groups by X and the colour column", () => {
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
    ).toBe(
      `SELECT "g" AS x, AVG("v") AS y, "k" AS c FROM "T" WHERE "v" IS NOT NULL GROUP BY "g", "k" ORDER BY x LIMIT 100`,
    );
  });

  it("bar + count + colour: COUNT(*), no Y filter, grouped by X and colour", () => {
    expect(
      chartSql({
        table: "T",
        x: "g",
        y: "",
        colour: "k",
        chartType: "bar",
        agg: "count",
        rowCap: 100,
      }),
    ).toBe(
      `SELECT "g" AS x, COUNT(*) AS y, "k" AS c FROM "T" GROUP BY "g", "k" ORDER BY x LIMIT 100`,
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
