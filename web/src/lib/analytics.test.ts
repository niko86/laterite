import { describe, expect, it } from "vitest";
import type { Table } from "apache-arrow";
import {
  coverageTruncationNote,
  referentialIntegrity,
  completeness,
  coverage,
  type Coverage,
  type DictKeyMap,
  type GroupKeyInfo,
} from "./analytics";
import type { GroupMeta } from "./duckTypes";

// The coverage matrix caps boreholes (rows) AND groups (columns). The notice
// must name whichever axis was actually clipped — a rows-only message misleads
// when only columns were dropped (the regression this guards).

const cov = (
  locas: number,
  totalLocas: number,
  groups: number,
  totalGroups: number,
): Coverage => ({
  groups: Array.from({ length: groups }, (_, i) => `G${i}`),
  locas: Array.from({ length: locas }, (_, i) => `L${i}`),
  present: {},
  totalGroups,
  totalLocas,
  truncated: locas < totalLocas || groups < totalGroups,
});

describe("coverageTruncationNote", () => {
  it("names only the borehole axis when only rows were capped", () => {
    expect(coverageTruncationNote(cov(60, 120, 10, 10))).toBe(
      "showing the first 60 of 120 boreholes.",
    );
  });

  it("names only the group axis when only columns were capped", () => {
    // The regression: this case used to read "...30 boreholes" though no
    // borehole was dropped.
    expect(coverageTruncationNote(cov(30, 30, 40, 50))).toBe(
      "showing the first 40 of 50 groups.",
    );
  });

  it("names both axes when both were capped", () => {
    expect(coverageTruncationNote(cov(60, 120, 40, 50))).toBe(
      "showing the first 60 of 120 boreholes and 40 of 50 groups.",
    );
  });
});

// --- a fake `run` (the only DuckDB seam) -------------------------------------
//
// referentialIntegrity / completeness / coverage take a `run(sql) => Table`,
// keeping them free of the duck/arrow imports. The mock returns a tiny Table
// stand-in (just `toArray()` — the only method the module calls) driven by a
// pattern → rows table, and records every SQL it was asked, so the tests can
// assert BOTH the result shape AND the queries the builders emit.

type Row = Record<string, unknown>;
const tbl = (rows: Row[]): Table =>
  ({ toArray: () => rows }) as unknown as Table;

/** Build a `run` from an ordered list of [match, rows] handlers; the first
 *  whose `match` (substring or RegExp) hits the SQL wins. Records every query
 *  on `.calls`. Throws on an unhandled SQL so a test can't silently pass on a
 *  query that never fired. */
function fakeRun(handlers: [string | RegExp, Row[]][]) {
  const calls: string[] = [];
  const run = (sql: string): Promise<Table> => {
    calls.push(sql);
    for (const [m, rows] of handlers) {
      if (typeof m === "string" ? sql.includes(m) : m.test(sql))
        return Promise.resolve(tbl(rows));
    }
    return Promise.reject(new Error(`unhandled SQL: ${sql}`));
  };
  return Object.assign(run, { calls });
}

const meta = (
  code: string,
  headings: string[],
  extra: Partial<GroupMeta> = {},
): GroupMeta => ({
  code,
  headings,
  units: extra.units ?? headings.map(() => ""),
  types: extra.types ?? headings.map(() => "X"),
  sql_types: extra.sql_types ?? headings.map(() => "VARCHAR"),
});

const dictKey = (entries: [string, GroupKeyInfo][]): DictKeyMap =>
  new Map(entries);

// --- referentialIntegrity -----------------------------------------------------

describe("referentialIntegrity", () => {
  const dict = dictKey([
    ["PROJ", { parent: null, keys: ["PROJ_ID"] }],
    ["LOCA", { parent: "PROJ", keys: ["LOCA_ID"] }],
    ["SAMP", { parent: "LOCA", keys: ["LOCA_ID", "SAMP_ID"] }],
  ]);

  it("finds orphan SAMP rows whose LOCA_ID matches no LOCA, with samples", async () => {
    const metas = [
      meta("LOCA", ["LOCA_ID", "LOCA_TYPE"]),
      meta("SAMP", ["LOCA_ID", "SAMP_ID"]),
    ];
    const run = fakeRun([
      // anti-join count: 2 orphans
      [/SELECT count\(\*\) AS n FROM "SAMP" c LEFT JOIN/, [{ n: 2n }]],
      // total rows in SAMP
      [/SELECT count\(\*\) AS n FROM "SAMP"$/, [{ n: 5 }]],
      // sample key tuples (LEFT JOIN, qualified select)
      [
        /SELECT c\."LOCA_ID" FROM "SAMP" c LEFT JOIN/,
        [{ LOCA_ID: "BH99" }, { LOCA_ID: "BH98" }],
      ],
    ]);

    const { links, orphans } = await referentialIntegrity(metas, dict, run);
    expect(links).toBe(1); // only SAMP→LOCA (PROJ not loaded; LOCA's parent absent)
    expect(orphans).toHaveLength(1);
    const o = orphans[0]!;
    expect(o.child).toBe("SAMP");
    expect(o.parent).toBe("LOCA");
    expect(o.keys).toEqual(["LOCA_ID"]); // only the SHARED key (SAMP_ID isn't LOCA's)
    expect(o.orphans).toBe(2); // bigint coerced to number
    expect(o.total).toBe(5);
    expect(o.samples).toEqual([["BH99"], ["BH98"]]);

    // The anti-join must filter on the parent-key-IS-NULL + child-keys-not-null.
    const antiJoin = run.calls.find((s) => /LEFT JOIN "LOCA" p/.test(s))!;
    expect(antiJoin).toContain('p."LOCA_ID" IS NULL');
    expect(antiJoin).toContain('c."LOCA_ID" IS NOT NULL');
  });

  it("emits no sample query and pushes nothing when there are zero orphans", async () => {
    const metas = [
      meta("LOCA", ["LOCA_ID"]),
      meta("SAMP", ["LOCA_ID", "SAMP_ID"]),
    ];
    const run = fakeRun([
      [/SELECT count\(\*\) AS n FROM "SAMP" c LEFT JOIN/, [{ n: 0 }]],
      [/SELECT count\(\*\) AS n FROM "SAMP"$/, [{ n: 5 }]],
    ]);
    const { links, orphans } = await referentialIntegrity(metas, dict, run);
    expect(links).toBe(1);
    expect(orphans).toEqual([]);
    // No "SELECT <keys> ... LIMIT 8" sample fetch when orphanCount === 0.
    expect(run.calls.some((s) => /LIMIT 8/.test(s))).toBe(false);
  });

  it("skips a child whose dictionary parent isn't loaded (no link, no query)", async () => {
    // SAMP's parent LOCA is absent → no link is even attempted.
    const metas = [meta("SAMP", ["LOCA_ID", "SAMP_ID"])];
    const run = fakeRun([]); // any query would throw
    const { links, orphans } = await referentialIntegrity(metas, dict, run);
    expect(links).toBe(0);
    expect(orphans).toEqual([]);
    expect(run.calls).toEqual([]);
  });

  it("skips a root group (no parent) and a child with no shared key columns", async () => {
    // PROJ is a root (parent null → skipped). A SAMP variant that physically
    // lacks LOCA_ID shares no key with LOCA → no link.
    const metas = [
      meta("PROJ", ["PROJ_ID"]),
      meta("LOCA", ["LOCA_ID"]),
      meta("SAMP", ["SAMP_ID"]), // no LOCA_ID column → shared.length === 0
    ];
    const run = fakeRun([]);
    const { links, orphans } = await referentialIntegrity(metas, dict, run);
    expect(links).toBe(0);
    expect(orphans).toEqual([]);
    expect(run.calls).toEqual([]);
  });

  it("skips a child whose code isn't in the dictionary at all", async () => {
    const metas = [meta("LOCA", ["LOCA_ID"]), meta("XXXX", ["LOCA_ID"])];
    const run = fakeRun([]);
    const { links } = await referentialIntegrity(metas, dict, run);
    expect(links).toBe(0);
  });
});

// --- completeness -------------------------------------------------------------

describe("completeness", () => {
  it("computes per-column fill %, empty columns, and overall mean", async () => {
    const metas = [
      meta("LOCA", ["LOCA_ID", "LOCA_TYPE", "LOCA_NOTE"], {
        types: ["ID", "PA", "X"],
        sql_types: ["VARCHAR", "VARCHAR", "VARCHAR"],
      }),
    ];
    // 4 rows: LOCA_ID full (4), LOCA_TYPE half (2), LOCA_NOTE empty (0).
    const run = fakeRun([[/FROM "LOCA"/, [{ n: 4, c0: 4, c1: 2, c2: 0 }]]]);
    const out = await completeness(metas, run);
    expect(out).toHaveLength(1);
    const g = out[0]!;
    expect(g.code).toBe("LOCA");
    expect(g.total).toBe(4);
    expect(g.cols.map((c) => c.pct)).toEqual([1, 0.5, 0]);
    expect(g.cols[0]).toMatchObject({
      heading: "LOCA_ID",
      type: "ID",
      sqlType: "VARCHAR",
      filled: 4,
    });
    expect(g.emptyCols).toEqual(["LOCA_NOTE"]); // 100%-empty present column
    expect(g.overall).toBeCloseTo((1 + 0.5 + 0) / 3, 10);

    // Single pass: one count(*)+per-column-count query.
    expect(run.calls).toHaveLength(1);
    expect(run.calls[0]).toContain('count("LOCA_ID") AS c0');
  });

  it("skips a group with no headings", async () => {
    const run = fakeRun([]);
    expect(await completeness([meta("EMPT", [])], run)).toEqual([]);
    expect(run.calls).toEqual([]);
  });

  it("an empty table (0 rows) reports pct 0 and no emptyCols (avoids /0)", async () => {
    // total === 0 → pct guarded to 0, and emptyCols requires total > 0 so a
    // never-populated table doesn't flag every column as 'empty'.
    const metas = [meta("LOCA", ["LOCA_ID", "LOCA_TYPE"])];
    const run = fakeRun([[/FROM "LOCA"/, [{ n: 0, c0: 0, c1: 0 }]]]);
    const out = await completeness(metas, run);
    const g = out[0]!;
    expect(g.total).toBe(0);
    expect(g.cols.map((c) => c.pct)).toEqual([0, 0]);
    expect(g.emptyCols).toEqual([]);
    expect(g.overall).toBe(0);
  });

  it("falls back to empty type/sqlType when the meta arrays are short", async () => {
    // headings longer than types/sql_types — the `?? ""` guards must hold.
    const metas = [
      meta("LOCA", ["LOCA_ID", "LOCA_TYPE"], { types: ["ID"], sql_types: [] }),
    ];
    const run = fakeRun([[/FROM "LOCA"/, [{ n: 1, c0: 1, c1: 1 }]]]);
    const out = await completeness(metas, run);
    const col = out[0]!.cols[1]!;
    expect(col.type).toBe("");
    expect(col.sqlType).toBe("");
  });
});

// --- coverage (LOCA × group matrix) ------------------------------------------

describe("coverage", () => {
  it("returns null when fewer than two groups carry LOCA_ID", async () => {
    const run = fakeRun([]);
    // Only one LOCA-bearing group → nothing to cross-reference.
    expect(await coverage([meta("LOCA", ["LOCA_ID"])], run)).toBeNull();
    // A group without LOCA_ID doesn't count toward the threshold.
    expect(
      await coverage(
        [meta("LOCA", ["LOCA_ID"]), meta("PROJ", ["PROJ_ID"])],
        run,
      ),
    ).toBeNull();
    expect(run.calls).toEqual([]);
  });

  it("builds a LOCA → present-set map across LOCA-bearing groups", async () => {
    const metas = [
      meta("LOCA", ["LOCA_ID"]),
      meta("SAMP", ["LOCA_ID", "SAMP_ID"]),
      meta("PROJ", ["PROJ_ID"]), // no LOCA_ID → excluded from the matrix
    ];
    const run = fakeRun([
      [/FROM "LOCA"/, [{ id: "BH02" }, { id: "BH01" }, { id: null }]],
      [/FROM "SAMP"/, [{ id: "BH01" }, { id: "BH03" }]],
    ]);
    const c = (await coverage(metas, run))!;
    expect(c).not.toBeNull();
    expect(c.groups).toEqual(["LOCA", "SAMP"]); // PROJ excluded
    expect(c.locas).toEqual(["BH01", "BH02", "BH03"]); // distinct + sorted, null dropped
    expect([...c.present.LOCA!].sort()).toEqual(["BH01", "BH02"]);
    expect([...c.present.SAMP!].sort()).toEqual(["BH01", "BH03"]);
    expect(c.totalGroups).toBe(2);
    expect(c.totalLocas).toBe(3);
    expect(c.truncated).toBe(false);
    // The per-group DISTINCT query filters out null LOCA_IDs.
    expect(run.calls[0]).toContain('WHERE "LOCA_ID" IS NOT NULL');
  });

  it("caps boreholes at MAX_LOCA (60) and flags truncation", async () => {
    // 70 distinct LOCAs across two groups → rows capped to 60, truncated=true.
    const ids = Array.from(
      { length: 70 },
      (_, i) => `BH${String(i).padStart(3, "0")}`,
    );
    const metas = [meta("LOCA", ["LOCA_ID"]), meta("SAMP", ["LOCA_ID"])];
    const run = fakeRun([
      [/FROM "LOCA"/, ids.map((id) => ({ id }))],
      [/FROM "SAMP"/, ids.map((id) => ({ id }))],
    ]);
    const c = (await coverage(metas, run))!;
    expect(c.locas).toHaveLength(60); // MAX_LOCA
    expect(c.totalLocas).toBe(70);
    expect(c.groups).toHaveLength(2);
    expect(c.truncated).toBe(true);
  });
});
