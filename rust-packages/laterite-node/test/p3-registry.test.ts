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

describe("registry traversal", () => {
  it("childGroups lists direct children alphabetically", () => {
    const children = registry.childGroups("PROJ").map((g) => g.code);
    expect(children).toContain("LOCA");
    expect([...children]).toEqual([...children].sort()); // alphabetical
  });

  it("ancestorChain walks code → root", () => {
    expect(registry.ancestorChain("PROJ")).toEqual(["PROJ"]); // root
    expect(registry.ancestorChain("SAMP")).toEqual(["SAMP", "LOCA", "PROJ"]);
    expect(() => registry.ancestorChain("NOPE")).toThrow(Ags4Error);
  });

  it("inheritedKeyNames gathers ancestors' KEY names", () => {
    const inherited = registry.inheritedKeyNames("SAMP");
    expect(inherited.has("LOCA_ID")).toBe(true); // from the LOCA ancestor
    expect(inherited.has("PROJ_ID")).toBe(true); // from the PROJ ancestor
  });
});
