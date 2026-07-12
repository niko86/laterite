import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import {
  projectEdition,
  loadStandardDict,
  loadEditionMeta,
  isKeyStatus,
  isRequiredStatus,
  type RawUnion,
} from "./dict";
import type { StandardDict, DictGroup } from "./validator";

// The canonical single source of truth — read at runtime so tsc doesn't infer
// the ~800 KB literal. This is the exact union every web consumer reads.
const REAL_UNION = JSON.parse(
  readFileSync(
    fileURLToPath(
      new URL(
        "../../../rust-packages/laterite-ags4-reference/data/ags_dictionary.json",
        import.meta.url,
      ),
    ),
    "utf8",
  ),
) as RawUnion;

const group = (d: StandardDict, code: string): DictGroup | undefined =>
  d.groups.find((g) => g.code === code);
const heading = (d: StandardDict, code: string, name: string) =>
  group(d, code)?.headings.find((h) => h.name === name);

describe("status helpers (+-aware, mirror registry.rs::is_key)", () => {
  it("isKeyStatus recognises combined statuses", () => {
    expect(isKeyStatus("KEY")).toBe(true);
    expect(isKeyStatus("KEY+REQUIRED")).toBe(true);
    expect(isKeyStatus("REQUIRED")).toBe(false);
    expect(isKeyStatus("OTHER")).toBe(false);
  });
  it("isRequiredStatus recognises combined statuses", () => {
    expect(isRequiredStatus("KEY+REQUIRED")).toBe(true);
    expect(isRequiredStatus("REQUIRED")).toBe(true);
    expect(isRequiredStatus("KEY")).toBe(false);
  });
});

describe("projectEdition — a faithful per-edition view from the union", () => {
  it("auto / unknown editions fall back to the union's fallback_edition", () => {
    expect(projectEdition(REAL_UNION, "auto").ags_edition).toBe(
      REAL_UNION.fallback_edition,
    );
    expect(projectEdition(REAL_UNION, "9.9").ags_edition).toBe(
      REAL_UNION.fallback_edition,
    );
    expect(projectEdition(REAL_UNION, "4.2").ags_edition).toBe("4.2");
  });

  it("drops groups not in the requested edition (ERES is gone in 4.2)", () => {
    expect(group(projectEdition(REAL_UNION, "4.1.1"), "ERES")).toBeDefined();
    expect(group(projectEdition(REAL_UNION, "4.2"), "ERES")).toBeUndefined();
    expect(group(projectEdition(REAL_UNION, "4.2"), "PROJ")).toBeDefined();
  });

  it("applies by_ed overrides (PROJ_CLNT description differs by edition)", () => {
    expect(
      heading(projectEdition(REAL_UNION, "4.2"), "PROJ", "PROJ_CLNT")?.description,
    ).toBe("Client organisation name");
    expect(
      heading(projectEdition(REAL_UNION, "4.0.3"), "PROJ", "PROJ_CLNT")?.description,
    ).toBe("Client name");
  });

  it("emits groups sorted by code, headings in dictionary order", () => {
    const d = projectEdition(REAL_UNION, "4.2");
    const codes = d.groups.map((g) => g.code);
    expect(codes).toEqual([...codes].sort());
    expect(group(d, "PROJ")?.headings[0]?.name).toBe("PROJ_ID");
  });

  it("every projected group is a member of the requested edition", () => {
    const ed = "4.0.3";
    for (const g of projectEdition(REAL_UNION, ed).groups) {
      const eds = REAL_UNION.groups[g.code].eds;
      expect(!eds || eds.includes(ed)).toBe(true);
    }
  });
});

describe("loadStandardDict — fetch + project, memoised", () => {
  it("fetches the union once and projects the requested edition", async () => {
    const orig = globalThis.fetch;
    let calls = 0;
    globalThis.fetch = (async () => {
      calls++;
      return { ok: true, json: async () => REAL_UNION } as Response;
    }) as typeof fetch;
    try {
      const d = await loadStandardDict("4.2");
      expect(d.ags_edition).toBe("4.2");
      expect(d.groups.length).toBeGreaterThan(50);
      // memoised: a second call (different edition) does NOT refetch
      const d2 = await loadStandardDict("4.1.1");
      expect(d2.ags_edition).toBe("4.1.1");
      expect(calls).toBe(1);
    } finally {
      globalThis.fetch = orig;
    }
  });

  // After the count-sensitive test above (the union is now cached), so this just
  // reads the cached union.
  it("loadEditionMeta exposes the union's editions + fallback", async () => {
    const meta = await loadEditionMeta();
    expect(meta.editions).toEqual(REAL_UNION.editions);
    expect(meta.fallback).toBe(REAL_UNION.fallback_edition);
  });
});
