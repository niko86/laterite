import { describe, it, expect } from "vitest";
import {
  prefillCodes,
  categoryOf,
  actionOf,
  codesForPreset,
  type SensitiveDoc,
} from "./sensitive";

const doc: SensitiveDoc = {
  categories: {
    lab: "...",
    coordinate: "...",
    geology: "...",
    location_id: "...",
    project_id: "...",
  },
  scrub_policy: {
    lab: "token",
    coordinate: "blank",
    geology: "token",
    location_id: "pseudonym",
    project_id: "filehash",
  },
  headings: {
    LLPL_LAB: { category: "lab" },
    LOCA_NATE: { category: "coordinate" },
    GEOL_FORM: { category: "geology" },
    LOCA_ID: { category: "location_id" },
    PROJ_ID: { category: "project_id" },
  },
};

describe("sensitive", () => {
  it("pre-fills ALL classified categories, incl. IDs (now pseudonymised)", () => {
    const pre = prefillCodes(doc);
    expect(pre.has("LLPL_LAB")).toBe(true);
    expect(pre.has("LOCA_NATE")).toBe(true);
    expect(pre.has("GEOL_FORM")).toBe(true);
    // The tool now pseudonymises IDs / hashes PROJ_ID instead of blanking, so
    // they're safe to include (cross-references stay intact).
    expect(pre.has("LOCA_ID")).toBe(true);
    expect(pre.has("PROJ_ID")).toBe(true);
  });

  it("maps heading code → category for the UI hint", () => {
    const c = categoryOf(doc);
    expect(c.get("LLPL_LAB")).toBe("lab");
    expect(c.get("LOCA_ID")).toBe("location_id");
    expect(c.get("GEOL_FORM")).toBe("geology");
  });

  it("maps heading code → its scrub_policy action", () => {
    const a = actionOf(doc);
    expect(a.get("LOCA_ID")).toBe("pseudonym");
    expect(a.get("PROJ_ID")).toBe("filehash");
    expect(a.get("LOCA_NATE")).toBe("blank");
    expect(a.get("LLPL_LAB")).toBe("token");
  });

  it("omits a heading whose category has no policy, rather than a blank action", () => {
    // The two halves of the SSOT can disagree: a heading classified into a
    // category that `scrub_policy` does not name. Mapping it to `undefined`
    // would put a heading in the action map with no action — the Anonymiser
    // would tick it and then redact it with nothing. Absent means "unhandled",
    // which is the honest state.
    const drifted: SensitiveDoc = {
      ...doc,
      headings: { ...doc.headings, ZZZZ_NEW: { category: "unclassified" } },
    };
    const a = actionOf(drifted);
    expect(a.has("ZZZZ_NEW")).toBe(false);
    // The rest of the map is unaffected — one unknown category is not a reason
    // to stop redacting the categories that ARE policed.
    expect(a.get("LOCA_NATE")).toBe("blank");
    expect(a.size).toBe(5);
    // …but the category lookup still knows it, so the UI can show the hint.
    expect(categoryOf(drifted).get("ZZZZ_NEW")).toBe("unclassified");
  });

  it("scopes preset pre-ticks by category", () => {
    expect([...codesForPreset(doc, "coords")]).toEqual(["LOCA_NATE"]);
    // 'all' is every classified heading.
    expect(codesForPreset(doc, "all").size).toBe(5);
  });
});
