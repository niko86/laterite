import { describe, it, expect } from "vitest";
import { prefillCodes, categoryOf, type SensitiveDoc } from "./sensitive";

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
  it("pre-fills redactable categories but excludes the identifier ones", () => {
    const pre = prefillCodes(doc);
    expect(pre.has("LLPL_LAB")).toBe(true);
    expect(pre.has("LOCA_NATE")).toBe(true);
    expect(pre.has("GEOL_FORM")).toBe(true);
    // The web tool blanks values; it can't pseudonymise, so blanking these
    // cross-referenced keys would break the file → not pre-ticked.
    expect(pre.has("LOCA_ID")).toBe(false);
    expect(pre.has("PROJ_ID")).toBe(false);
  });

  it("maps heading code → category for the UI hint", () => {
    const c = categoryOf(doc);
    expect(c.get("LLPL_LAB")).toBe("lab");
    expect(c.get("LOCA_ID")).toBe("location_id");
    expect(c.get("GEOL_FORM")).toBe("geology");
  });
});
