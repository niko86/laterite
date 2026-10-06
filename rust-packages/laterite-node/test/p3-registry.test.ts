// P3 — laterite.registry: the read-only group metadata, generated from the same
// dictionary JSON as the Python registry.
import { describe, expect, it } from "vitest";
import { Ags4Error, GroupDescriptor, registry } from "../ts/index";

describe("registry.GROUPS", () => {
  it("holds every standard group as a descriptor", () => {
    // The union dictionary spans editions 4.0.3–4.2 (174 groups at time of
    // writing); assert a floor, not an exact count, so adding a group later
    // doesn't break this test.
    expect(Object.keys(registry.GROUPS).length).toBeGreaterThan(150);
    expect(registry.get("PROJ")).toBeInstanceOf(GroupDescriptor);
    expect(registry.get("NOPE")).toBeUndefined();
  });

  it("a descriptor exposes table/view names + KEY split", () => {
    const loca = registry.get("LOCA")!;
    expect(loca.table).toBe("g_loca");
    expect(loca.view).toBe("v_loca");
    expect(loca.parent).toBe("PROJ");
    expect(loca.keyHeadings.every((h) => h.status === "KEY")).toBe(true);
    expect(loca.nonKeyHeadings.every((h) => h.status !== "KEY")).toBe(true);
    expect(loca.headings.find((h) => h.name === "LOCA_ID")?.status).toBe("KEY");
  });
});

describe("registry.dictionary(edition) — the per-edition STANDARD dictionary (#294 F#6)", () => {
  it("returns the shared {ags_edition, groups[…]} snapshot", () => {
    const d = registry.dictionary("4.2");
    expect(d.ags_edition).toBe("4.2");
    expect(d.groups.length).toBeGreaterThan(150);
    const proj = d.groups.find((g) => g.code === "PROJ")!;
    expect(proj.contents).toBeTruthy();
    expect(proj.headings[0]).toMatchObject({
      name: "PROJ_ID",
      status: expect.any(String),
    });
    // `type`, not `ags_type` — the shared shape across surfaces.
    expect(proj.headings[0]).toHaveProperty("type");
  });

  it("editions differ; auto/omitted fall back to the default", () => {
    expect(registry.dictionary("4.0.3").groups.length).toBeLessThan(
      registry.dictionary("4.2").groups.length,
    );
    expect(registry.dictionary().ags_edition).toBe(
      registry.dictionary("auto").ags_edition,
    );
  });

  it("throws on an unknown edition", () => {
    expect(() => registry.dictionary("9.9")).toThrow();
  });
});

describe("registry.abbreviations(edition) — the per-edition standard abbreviation list (#1014)", () => {
  const descOf = (edition: string, heading: string, code: string) =>
    registry
      .abbreviations(edition)
      .find((r) => r.heading === heading && r.code === code)?.description;

  it("flat rows carry exactly heading/code/description, ordered by heading then code", () => {
    const rows = registry.abbreviations("4.1.1");
    expect(rows.length).toBeGreaterThan(0);
    for (const r of rows)
      expect(Object.keys(r).sort()).toEqual(["code", "description", "heading"]);
    // Plain `<` (code-unit order), not localeCompare: the Rust builder sorts by
    // byte, and a locale collation would fold the case-only pairs together.
    const cmp = (
      a: { heading: string; code: string },
      b: { heading: string; code: string },
    ) =>
      a.heading === b.heading
        ? Number(a.code > b.code) - Number(a.code < b.code)
        : Number(a.heading > b.heading) - Number(a.heading < b.heading);
    expect(rows).toEqual([...rows].sort(cmp));
    expect(registry.abbreviations("4.1.1", { shape: "flat" })).toEqual(rows);
    expect(registry.abbreviations()).toEqual(registry.abbreviations("auto"));
  });

  it("nested regroups the flat list by heading", () => {
    const flat = registry.abbreviations("4.1.1");
    const nested = registry.abbreviations("4.1.1", { shape: "nested" });
    const back = Object.entries(nested).flatMap(([heading, codes]) =>
      codes.map((c) => ({ heading, code: c.code, description: c.description })),
    );
    expect(back).toEqual(flat);
  });

  it("is masked per edition, with per-edition descriptions", () => {
    expect(descOf("4.2", "CBRP_END", "BASE")).toBeDefined();
    expect(descOf("4.1.1", "CBRP_END", "BASE")).toBeUndefined();
    expect(descOf("4.1.1", "ELRG_CODE", "100-75-4")).toBe(
      "n-nitrosopiperidine",
    );
    expect(descOf("4.2", "ELRG_CODE", "100-75-4")).toBe("n-Nitrosopiperidine");
  });

  it("keeps case-only code pairs", () => {
    expect(descOf("4.1.1", "PTST_TYPE", "CONSTANT HEAD")).toBeDefined();
    expect(descOf("4.1.1", "PTST_TYPE", "Constant Head")).toBeDefined();
  });

  it("throws on an unknown edition or shape", () => {
    expect(() => registry.abbreviations("9.9")).toThrow(/unknown edition/);
    expect(() =>
      registry.abbreviations("4.2", { shape: "tree" as unknown as "flat" }),
    ).toThrow(/flat\|nested/);
  });
});

describe("registry traversal", () => {
  it("childGroups lists direct children alphabetically", () => {
    const children = registry.childGroups("PROJ").map((g) => g.code);
    expect(children).toContain("LOCA");
    expect([...children]).toEqual([...children].sort()); // alphabetical
  });

  // These two walks are now the native binding (laterite_ags4_core::registry),
  // the SAME leaf the Python wheel binds — not a TS re-implementation of the parent
  // walk / KEY-intersection (laterite-dev#532). The values below are the leaf's; the unknown-code
  // case also pins that the native error surfaces as an Ags4Error with the message intact.
  it("ancestorChain walks code → root (native binding, laterite-dev#532)", () => {
    expect(registry.ancestorChain("PROJ")).toEqual(["PROJ"]); // root
    expect(registry.ancestorChain("SAMP")).toEqual(["SAMP", "LOCA", "PROJ"]);
    expect(() => registry.ancestorChain("NOPE")).toThrow(Ags4Error);
    expect(() => registry.ancestorChain("NOPE")).toThrow(
      'unknown group code: "NOPE"',
    );
  });

  it("inheritedKeyNames is the direct-parent intersection (native binding, laterite-dev#532)", () => {
    const inherited = registry.inheritedKeyNames("SAMP");
    expect(inherited.has("LOCA_ID")).toBe(true); // shared with the direct parent LOCA
    expect(inherited.has("PROJ_ID")).toBe(false); // NOT inherited — SAMP carries no PROJ_ID key
    expect(() => registry.inheritedKeyNames("NOPE")).toThrow(Ags4Error);
    expect(() => registry.inheritedKeyNames("NOPE")).toThrow(
      'unknown group code: "NOPE"',
    );
  });
});
