import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { typeDescription } from "./agsTypeInfo";

// The canonical AGS TYPE vocabulary = the distinct `type` codes in the union
// dictionary (the single source). Read at runtime so a code added there is
// caught here rather than silently glossed as "text".
const UNION = JSON.parse(
  readFileSync(
    fileURLToPath(
      new URL(
        "../../../rust-packages/laterite-ags4-core/data/ags_dictionary.json",
        import.meta.url,
      ),
    ),
    "utf8",
  ),
) as { groups: Record<string, { headings: { type: string }[] }> };

const dictTypes = [
  ...new Set(
    Object.values(UNION.groups).flatMap((g) => g.headings.map((h) => h.type)),
  ),
].sort();

describe("typeDescription — covers the dictionary's TYPE vocabulary", () => {
  it("every AGS type code in the union has a real (non-fallback) description", () => {
    // "X" is the one code that legitimately means free text; every other code
    // must map to a specific description, so a newly-added type can't silently
    // fall through to the "text" fallback unnoticed (this caught RL).
    const unhandled = dictTypes.filter(
      (t) => t !== "X" && typeDescription(t) === "text",
    );
    expect(unhandled).toEqual([]);
  });

  it("pins a few representative descriptions", () => {
    expect(typeDescription("0DP")).toBe("whole number");
    expect(typeDescription("2DP")).toBe("decimal, 2 places");
    expect(typeDescription("3SF")).toBe("decimal, 3 significant figures");
    expect(typeDescription("DT")).toBe("date / time");
    expect(typeDescription("PA")).toBe("picklist (abbreviation)");
    expect(typeDescription("RL")).toBe("record link");
    expect(typeDescription("X")).toBe("text"); // the legitimate free-text type
    expect(typeDescription("ZZ")).toBe("text"); // unknown → fallback
  });
});
