// The parent-chain walkers, and the guards that stop a bad dictionary hanging the
// tab.
//
// `isAncestor` and `relatedGroups` both walk `parent` pointers in a `while` loop.
// Each carries a `seen`/`guard` Set whose only job is to terminate if the chain
// ever loops — and nothing exercised that arm, so the guards were unproven code
// protecting against the one failure a user cannot recover from. A wrong answer
// renders wrongly; a cycle freezes the page, and this walk runs on the main
// thread while a file is being explored.
//
// The dictionary is loaded at runtime (`loadDict()` fetches it), so a cycle does
// not have to come from our own bundled data to arrive here.
import { describe, expect, it } from "vitest";

import {
  type DictGroupInfo,
  type DictMap,
  isAncestor,
  relatedGroups,
} from "./relationships";

/** Minimal dictionary entry — only `parent`/`keys` matter to the walkers. */
function group(
  parent: string | null,
  keys: string[] = [],
  headingNames: string[] = keys,
): DictGroupInfo {
  return {
    parent,
    keys,
    headings: headingNames.map((name) => ({
      name,
      status: keys.includes(name) ? "KEY" : "OTHER",
      type: "X",
    })),
    contents: "",
  };
}

/** PROJ → LOCA → SAMP → SPEC, the real spine of an AGS file. */
const CHAIN: DictMap = new Map([
  ["PROJ", group(null, ["PROJ_ID"])],
  ["LOCA", group("PROJ", ["LOCA_ID"])],
  ["SAMP", group("LOCA", ["LOCA_ID", "SAMP_TOP", "SAMP_REF"])],
  ["SPEC", group("SAMP", ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SPEC_REF"])],
]);

describe("isAncestor", () => {
  it("counts a group as its own ancestor", () => {
    // The `anc === code` short-circuit, taken before the walk starts. Callers use
    // this to ask "is this row's group at or below X" — excluding self would drop
    // the group from its own subtree.
    expect(isAncestor("LOCA", "LOCA", CHAIN)).toBe(true);
  });

  it("finds a direct parent", () => {
    expect(isAncestor("LOCA", "SAMP", CHAIN)).toBe(true);
  });

  it("finds a distant ancestor across the whole chain", () => {
    expect(isAncestor("PROJ", "SPEC", CHAIN)).toBe(true);
  });

  it("is directional — a parent is not a descendant of its child", () => {
    expect(isAncestor("SPEC", "PROJ", CHAIN)).toBe(false);
  });

  it("returns false for an unrelated group", () => {
    const dict: DictMap = new Map([...CHAIN, ["ABBR", group(null)]]);
    expect(isAncestor("ABBR", "SPEC", dict)).toBe(false);
  });

  it("returns false for a code the dictionary has never heard of", () => {
    // `dict.get(code)?.parent ?? null` — the walk starts at null and ends
    // immediately rather than throwing on undefined.
    expect(isAncestor("PROJ", "ZZZZ", CHAIN)).toBe(false);
    expect(isAncestor("ZZZZ", "SPEC", CHAIN)).toBe(false);
  });

  it("terminates on a two-group cycle", () => {
    // A → B → A. Without the `seen` guard this loops forever and the tab stops
    // responding, with no error and nothing in the console.
    const cyclic: DictMap = new Map([
      ["AAAA", group("BBBB")],
      ["BBBB", group("AAAA")],
    ]);
    expect(isAncestor("ZZZZ", "AAAA", cyclic)).toBe(false);
    expect(isAncestor("BBBB", "AAAA", cyclic)).toBe(true);
  });

  it("terminates on a self-parent", () => {
    // The degenerate cycle, and the easiest one to produce by a one-character
    // slip in a dictionary edit.
    const selfish: DictMap = new Map([["AAAA", group("AAAA")]]);
    expect(isAncestor("ZZZZ", "AAAA", selfish)).toBe(false);
  });

  it("terminates on a longer cycle that does not include the start", () => {
    // A → B → C → B. The guard has to catch a loop entered part-way through, not
    // just one that returns to where the walk began.
    const tail: DictMap = new Map([
      ["AAAA", group("BBBB")],
      ["BBBB", group("CCCC")],
      ["CCCC", group("BBBB")],
    ]);
    expect(isAncestor("ZZZZ", "AAAA", tail)).toBe(false);
  });
});

describe("relatedGroups", () => {
  it("orders parents before children before merely-related groups", () => {
    const rel = relatedGroups("SAMP", ["PROJ", "LOCA", "SPEC"], CHAIN);
    // The rank is explicit — parent(0) → child(1) → related(2) — and deliberately
    // NOT alphabetical, so comparing against a default `.sort()` would assert the
    // wrong order ("child" sorts before "parent").
    const RANK = { parent: 0, child: 1, related: 2 } as const;
    const ranks = rel.map((r) => RANK[r.direction]);
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    expect(rel[0]?.direction).toBe("parent");
  });

  it("puts nearer relatives first within a direction", () => {
    const parents = relatedGroups(
      "SPEC",
      ["PROJ", "LOCA", "SAMP"],
      CHAIN,
    ).filter((r) => r.direction === "parent");
    expect(parents.map((r) => r.code)).toEqual(["SAMP", "LOCA", "PROJ"]);
    expect(parents.map((r) => r.distance)).toEqual([1, 2, 3]);
  });

  it("never lists the base group as its own relative", () => {
    const rel = relatedGroups("SAMP", ["PROJ", "LOCA", "SAMP", "SPEC"], CHAIN);
    expect(rel.map((r) => r.code)).not.toContain("SAMP");
  });

  it("lists each relative once", () => {
    const codes = relatedGroups("SAMP", ["PROJ", "LOCA", "SPEC"], CHAIN).map(
      (r) => r.code,
    );
    expect(codes).toEqual([...new Set(codes)]);
  });

  it("ignores loaded codes that are not in the dictionary", () => {
    expect(() => relatedGroups("SAMP", ["PROJ", "ZZZZ"], CHAIN)).not.toThrow();
    const codes = relatedGroups("SAMP", ["PROJ", "ZZZZ"], CHAIN).map(
      (r) => r.code,
    );
    expect(codes).not.toContain("ZZZZ");
  });

  it("returns nothing for a base the dictionary does not define", () => {
    // `dict.get(base)?.parent ?? null` and `dict.get(base)?.keys ?? []` — both
    // fallbacks, and both only reachable with an unknown base.
    expect(relatedGroups("ZZZZ", ["PROJ", "LOCA"], CHAIN)).toEqual([]);
  });

  it("terminates when the parent chain above the base is cyclic", () => {
    // The ancestor walk's own `seen` guard, on the same malformed-dictionary
    // scenario as isAncestor's.
    const cyclic: DictMap = new Map([
      ["AAAA", group("BBBB", ["K"])],
      ["BBBB", group("AAAA", ["K"])],
    ]);
    expect(() => relatedGroups("AAAA", ["BBBB"], cyclic)).not.toThrow();
  });

  it("terminates when a LOADED group's chain is cyclic", () => {
    // A separate guard from the one above: the descendant search walks UP from
    // each loaded code looking for the base, so a cycle anywhere in the loaded
    // set hangs it even when the base's own chain is clean.
    const cyclic: DictMap = new Map([
      ["PROJ", group(null, ["PROJ_ID"])],
      ["AAAA", group("BBBB", ["K"])],
      ["BBBB", group("AAAA", ["K"])],
    ]);
    expect(() => relatedGroups("PROJ", ["AAAA", "BBBB"], cyclic)).not.toThrow();
  });

  it("requires more than one shared key before calling two groups related", () => {
    // The documented fan-out rule: a lone shared LOCA_ID is a per-borehole
    // cross-product, so it is NOT surfaced as a join suggestion. Two shared keys
    // disambiguate and are.
    const dict: DictMap = new Map([
      ["LOCA", group(null, ["LOCA_ID"])],
      ["AAAA", group(null, ["LOCA_ID"])], // one shared key → not related
      ["BBBB", group(null, ["LOCA_ID", "SAMP_TOP"])],
      ["CCCC", group(null, ["LOCA_ID", "SAMP_TOP"])], // two shared → related
    ]);
    const rel = relatedGroups("CCCC", ["AAAA", "BBBB"], dict).map(
      (r) => r.code,
    );
    expect(rel).toContain("BBBB");
    expect(rel).not.toContain("AAAA");
  });
});
