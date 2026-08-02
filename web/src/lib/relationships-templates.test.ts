// `joinKeys`, `depthRangeOf` and the query templates built on them.
//
// These produce the SQL the Explore tab OFFERS the user, so a wrong answer here
// is not a crash — it is a suggested query that runs and returns the wrong rows,
// which is the worst outcome available. Three decisions carry that risk:
//
//   * `joinKeys` takes the ANCESTOR's keys and requires both tables to
//     physically carry each one. Real files drift (MOND has MOND_REF where MONG
//     has PIPE_REF), so a key that is declared but absent must simply not be
//     joined on rather than emitting a predicate on a missing column;
//   * `depthRangeOf` pairs a `*_TOP` with its SAME-PREFIX `*_BASE`. A child
//     group carries an inherited parent `*_TOP`, and pairing that with an
//     unrelated `*_BASE` yields an incoherent interval that still runs;
//   * `geologyTemplate` is the flagship suggestion and bails out at five
//     separate points — each of which, if it returned a template instead, would
//     put a broken query in front of the user.
import { describe, expect, it } from "vitest";

import type { DictGroupInfo, DictMap } from "./relationships";
import {
  depthRangeOf,
  geologyTemplate,
  isDepthRangeGroup,
  joinKeys,
  relExamples,
} from "./relationships";

function group(
  parent: string | null,
  keys: string[],
  headings: { name: string; type?: string; status?: string }[],
): DictGroupInfo {
  return {
    parent,
    keys,
    headings: headings.map((h) => ({
      name: h.name,
      status: h.status ?? (keys.includes(h.name) ? "KEY" : "OTHER"),
      type: h.type ?? "X",
    })),
    contents: "",
  };
}

const DICT: DictMap = new Map([
  ["PROJ", group(null, ["PROJ_ID"], [{ name: "PROJ_ID" }])],
  ["LOCA", group("PROJ", ["LOCA_ID"], [{ name: "LOCA_ID" }])],
  [
    "GEOL",
    group(
      "LOCA",
      ["LOCA_ID", "GEOL_TOP"],
      [
        { name: "LOCA_ID" },
        { name: "GEOL_TOP", type: "2DP" },
        { name: "GEOL_BASE", type: "2DP" },
        { name: "GEOL_LEG" },
        { name: "GEOL_DESC" },
      ],
    ),
  ],
  [
    "SAMP",
    group(
      "LOCA",
      ["LOCA_ID", "SAMP_TOP", "SAMP_REF"],
      [
        { name: "LOCA_ID" },
        { name: "SAMP_TOP", type: "2DP" },
        { name: "SAMP_REF" },
        { name: "SAMP_BASE", type: "2DP" },
      ],
    ),
  ],
  // A test group that carries no depth of its own — it hangs off SAMP for that.
  // TEST_GROUPS puts it ahead of SAMP, so it is what the template's search meets
  // first.
  [
    "TRIG",
    group(
      "SAMP",
      ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "TRIG_TESN"],
      [{ name: "LOCA_ID" }, { name: "TRIG_TESN" }, { name: "TRIG_REM" }],
    ),
  ],
]);

describe("joinKeys", () => {
  it("joins on the ancestor's keys, whichever order the pair is given in", () => {
    const samp = { code: "SAMP", cols: ["LOCA_ID", "SAMP_TOP"] };
    const loca = { code: "LOCA", cols: ["LOCA_ID"] };
    // LOCA is SAMP's ancestor, so LOCA's key is the join key either way round.
    expect(joinKeys(samp, loca, DICT)).toEqual([
      { left: "LOCA_ID", right: "LOCA_ID" },
    ]);
    expect(joinKeys(loca, samp, DICT)).toEqual([
      { left: "LOCA_ID", right: "LOCA_ID" },
    ]);
  });

  it("skips a declared key that a table does not physically carry", () => {
    // The pseudo-key-drift guard. A predicate on a column the ingested table
    // lacks is a DuckDB "column not found" — the suggestion has to omit it.
    const withKey = { code: "SAMP", cols: ["LOCA_ID", "SAMP_TOP"] };
    const without = { code: "LOCA", cols: ["LOCA_NAME"] }; // no LOCA_ID column
    expect(joinKeys(withKey, without, DICT)).toEqual([]);
  });

  it("still joins a CUSTOM group that physically carries the known side's key", () => {
    // A bespoke group is not in the dictionary, so it cannot be anyone's
    // ancestor — the known side supplies the keys, and the custom table joins on
    // the ones it actually has. This is what makes a custom group usable in the
    // explorer rather than isolated.
    expect(
      joinKeys(
        { code: "ZZZZ", cols: ["LOCA_ID", "ZZZZ_VAL"] },
        { code: "LOCA", cols: ["LOCA_ID"] },
        DICT,
      ),
    ).toEqual([{ left: "LOCA_ID", right: "LOCA_ID" }]);
    // …and joins on nothing when it does not carry the key.
    expect(
      joinKeys(
        { code: "ZZZZ", cols: ["ZZZZ_VAL"] },
        { code: "LOCA", cols: ["LOCA_ID"] },
        DICT,
      ),
    ).toEqual([]);
  });

  it("joins two groups the dictionary has never heard of on nothing", () => {
    // Neither side can be the other's ancestor, so there are no declared keys to
    // take — and inventing a join on a shared column name would be a guess. Two
    // custom tables stay unrelated until the dictionary says otherwise.
    expect(
      joinKeys(
        { code: "YYYY", cols: ["LOCA_ID", "YYYY_VAL"] },
        { code: "ZZZZ", cols: ["LOCA_ID", "ZZZZ_VAL"] },
        DICT,
      ),
    ).toEqual([]);
  });
});

describe("depthRangeOf", () => {
  it("finds a group's own TOP/BASE band", () => {
    expect(depthRangeOf("GEOL", DICT)).toEqual({
      loca: "LOCA_ID",
      top: "GEOL_TOP",
      base: "GEOL_BASE",
    });
  });

  it("requires the band columns to be physically present when columns are known", () => {
    // Many groups declare a `*_BASE` that real files omit. Emitting a predicate
    // on it would produce a query that cannot run.
    expect(depthRangeOf("GEOL", DICT, ["LOCA_ID", "GEOL_TOP"])).toBeNull();
    expect(
      depthRangeOf("GEOL", DICT, ["LOCA_ID", "GEOL_TOP", "GEOL_BASE"]),
    ).not.toBeNull();
  });

  it("rejects a group with no LOCA_ID, present or declared", () => {
    expect(depthRangeOf("PROJ", DICT)).toBeNull();
    // …and rejects it on the live columns too, even though the dictionary has it.
    expect(depthRangeOf("GEOL", DICT, ["GEOL_TOP", "GEOL_BASE"])).toBeNull();
  });

  it("returns null for an unknown group", () => {
    expect(depthRangeOf("ZZZZ", DICT)).toBeNull();
  });

  it("agrees with isDepthRangeGroup", () => {
    expect(
      isDepthRangeGroup("GEOL", DICT, ["LOCA_ID", "GEOL_TOP", "GEOL_BASE"]),
    ).toBe(true);
    expect(isDepthRangeGroup("PROJ", DICT, ["PROJ_ID"])).toBe(false);
  });
});

describe("geologyTemplate", () => {
  const geolMeta = {
    code: "GEOL",
    headings: ["LOCA_ID", "GEOL_TOP", "GEOL_BASE", "GEOL_LEG", "GEOL_DESC"],
  };
  const sampMeta = {
    code: "SAMP",
    headings: ["LOCA_ID", "SAMP_TOP", "SAMP_REF"],
  };

  it("builds a half-open stratum join between the sample depth and the band", () => {
    const t = geologyTemplate([geolMeta, sampMeta], DICT);
    expect(t).not.toBeNull();
    // The band must be half-open: a sample exactly at GEOL_BASE belongs to the
    // NEXT stratum, so `<=` would place it in two at once.
    expect(t!.sql).toContain('>= g."GEOL_TOP"');
    expect(t!.sql).toContain('< g."GEOL_BASE"');
    expect(t!.sql).not.toContain('<= g."GEOL_BASE"');
    // And it selects the stratum description, which is the point of the query.
    expect(t!.sql).toContain("GEOL_DESC");
  });

  it("returns nothing when no depth-range group is loaded", () => {
    expect(geologyTemplate([sampMeta], DICT)).toBeNull();
  });

  it("returns nothing when no sample group is loaded to hang off it", () => {
    expect(geologyTemplate([geolMeta], DICT)).toBeNull();
  });

  it("returns nothing when the range group's own band columns are absent", () => {
    const noBase = { code: "GEOL", headings: ["LOCA_ID", "GEOL_TOP"] };
    expect(geologyTemplate([noBase, sampMeta], DICT)).toBeNull();
  });

  it("passes over a test group with no depth column and keeps looking", () => {
    // TRIG is searched before SAMP and carries LOCA_ID, but has no depth of its
    // own. Taking it anyway would mean building a depth-band predicate on a
    // column that is not there; abandoning the search at it would drop the
    // template entirely when a perfectly good SAMP is loaded.
    const trigMeta = { code: "TRIG", headings: ["LOCA_ID", "TRIG_TESN"] };
    const t = geologyTemplate([geolMeta, trigMeta, sampMeta], DICT);
    expect(t).not.toBeNull();
    expect(t!.sql).toContain('t."SAMP_TOP"');
    expect(t!.sql).toContain('"SAMP"');
    // …and with only the depth-less group loaded there is nothing to hang the
    // stratum off, so no template at all.
    expect(geologyTemplate([geolMeta, trigMeta], DICT)).toBeNull();
  });

  it("returns nothing when the two sides share no LOCA_ID to join on", () => {
    const noLoca = { code: "SAMP", headings: ["SAMP_TOP", "SAMP_REF"] };
    expect(geologyTemplate([geolMeta, noLoca], DICT)).toBeNull();
  });
});

describe("relExamples", () => {
  it("offers the geology template first when it applies", () => {
    const out = relExamples(
      [
        {
          code: "GEOL",
          headings: ["LOCA_ID", "GEOL_TOP", "GEOL_BASE", "GEOL_DESC"],
        },
        { code: "SAMP", headings: ["LOCA_ID", "SAMP_TOP", "SAMP_REF"] },
        { code: "LOCA", headings: ["LOCA_ID"] },
      ],
      DICT,
    );
    expect(out.length).toBeGreaterThan(0);
    expect(out[0]!.sql).toContain("GEOL");
  });

  it("offers a child ⋈ parent example for each loaded pair", () => {
    const out = relExamples(
      [
        { code: "LOCA", headings: ["LOCA_ID"] },
        { code: "SAMP", headings: ["LOCA_ID", "SAMP_TOP", "SAMP_REF"] },
      ],
      DICT,
    );
    expect(out.some((e) => e.sql.includes("SAMP"))).toBe(true);
    expect(out.every((e) => e.name.length > 0)).toBe(true);
  });

  it("skips a child whose parent is not loaded", () => {
    // Nothing to join to — suggesting the query anyway would offer SQL against
    // a table the session does not have.
    const out = relExamples(
      [{ code: "SAMP", headings: ["LOCA_ID", "SAMP_TOP", "SAMP_REF"] }],
      DICT,
    );
    expect(out).toEqual([]);
  });

  it("skips a pair that shares no physically present key", () => {
    const out = relExamples(
      [
        { code: "LOCA", headings: ["LOCA_NAME"] }, // no LOCA_ID column
        { code: "SAMP", headings: ["LOCA_ID", "SAMP_TOP"] },
      ],
      DICT,
    );
    expect(out).toEqual([]);
  });

  it("offers nothing for a root group with no loaded children", () => {
    expect(
      relExamples([{ code: "PROJ", headings: ["PROJ_ID"] }], DICT),
    ).toEqual([]);
  });
});
