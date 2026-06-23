import { describe, expect, it } from "vitest";
import {
  relatedGroups,
  joinKeys,
  depthRangeOf,
  depthColumnFor,
  geologyTemplate,
  relExamples,
  asKeyMap,
  dictMapFromJson,
  type DictMap,
} from "./relationships";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { RawUnion } from "./dict";

const H = (name: string, status: string, type: string) => ({ name, status, type });

// The canonical single source of truth — read at runtime (not a static import,
// which would make tsc infer the ~800 KB literal). This is the exact file every
// other consumer reads via lib/dict.ts.
const REAL_UNION = JSON.parse(
  readFileSync(
    fileURLToPath(
      new URL(
        "../../../rust-packages/laterite-ags4-core/data/ags_dictionary.json",
        import.meta.url,
      ),
    ),
    "utf8",
  ),
) as RawUnion;

function dict(): DictMap {
  const m: DictMap = new Map();
  const add = (
    code: string,
    parent: string | null,
    headings: { name: string; status: string; type: string }[],
  ) =>
    m.set(code, {
      parent,
      keys: headings.filter((h) => h.status === "KEY").map((h) => h.name),
      headings,
      contents: `${code} group`,
    });
  add("PROJ", null, [H("PROJ_ID", "KEY", "ID")]);
  add("LOCA", "PROJ", [
    H("LOCA_ID", "KEY", "ID"),
    H("LOCA_TYPE", "OTHER", "PA"),
    H("LOCA_NATE", "OTHER", "2DP"),
  ]);
  add("SAMP", "LOCA", [
    H("LOCA_ID", "KEY", "ID"),
    H("SAMP_TOP", "KEY", "2DP"),
    H("SAMP_REF", "KEY", "X"),
    H("SAMP_TYPE", "KEY", "PA"),
    H("SAMP_ID", "KEY", "ID"),
  ]);
  add("GEOL", "LOCA", [
    H("LOCA_ID", "KEY", "ID"),
    H("GEOL_TOP", "KEY", "2DP"),
    H("GEOL_BASE", "REQUIRED", "2DP"),
    H("GEOL_DESC", "OTHER", "X"),
    H("GEOL_LEG", "OTHER", "X"),
  ]);
  add("TREG", "SAMP", [
    H("LOCA_ID", "KEY", "ID"),
    H("SAMP_TOP", "KEY", "2DP"), // inherited parent TOP — must NOT pair with SPEC_BASE
    H("SAMP_REF", "KEY", "X"),
    H("SAMP_TYPE", "KEY", "PA"),
    H("SAMP_ID", "KEY", "ID"),
    H("SPEC_REF", "KEY", "X"),
    H("SPEC_DPTH", "KEY", "2DP"),
    H("SPEC_BASE", "OTHER", "2DP"), // a *_BASE of a different prefix than SAMP_TOP
    H("SPEC_DESC", "OTHER", "X"),
  ]);
  // MONG/MOND: the pseudo-key drift case — MOND's 3rd key is MOND_REF where
  // MONG's is PIPE_REF (so the shared key must NOT include either).
  add("MONG", "LOCA", [
    H("LOCA_ID", "KEY", "ID"),
    H("MONG_ID", "KEY", "X"),
    H("PIPE_REF", "KEY", "X"),
  ]);
  add("MOND", "MONG", [
    H("LOCA_ID", "KEY", "ID"),
    H("MONG_ID", "KEY", "X"),
    H("MOND_REF", "KEY", "X"),
  ]);
  return m;
}
const cols = (m: DictMap, code: string) =>
  (m.get(code)?.headings ?? []).map((h) => h.name);

describe("relatedGroups", () => {
  it("SAMP → LOCA (parent), TREG (child), GEOL (related sibling, shares LOCA_ID)", () => {
    const r = relatedGroups("SAMP", ["LOCA", "SAMP", "GEOL", "TREG"], dict());
    expect(r).toEqual([
      { code: "LOCA", direction: "parent", distance: 1 },
      { code: "TREG", direction: "child", distance: 1 },
      { code: "GEOL", direction: "related", distance: 1 },
    ]);
  });
  it("GEOL → LOCA (parent) only; lone-LOCA_ID non-depth siblings are suppressed", () => {
    // Sibling tightening: SAMP/TREG share only LOCA_ID with GEOL and aren't
    // depth-range groups, so offering them would mean a per-borehole fan-out
    // (every stratum × every sample). The meaningful direction — base=SAMP/TREG,
    // related=GEOL (the depth band) — is still offered (see the SAMP case above).
    const r = relatedGroups("GEOL", ["LOCA", "SAMP", "GEOL", "TREG"], dict());
    expect(r).toEqual([{ code: "LOCA", direction: "parent", distance: 1 }]);
  });
});

describe("joinKeys", () => {
  const d = dict();
  it("SAMP ⋈ LOCA on LOCA_ID", () => {
    expect(
      joinKeys({ code: "SAMP", cols: cols(d, "SAMP") }, { code: "LOCA", cols: cols(d, "LOCA") }, d),
    ).toEqual([{ left: "LOCA_ID", right: "LOCA_ID" }]);
  });
  it("TREG ⋈ SAMP on the full 5-part sample key", () => {
    const pairs = joinKeys(
      { code: "TREG", cols: cols(d, "TREG") },
      { code: "SAMP", cols: cols(d, "SAMP") },
      d,
    );
    expect(pairs.map((p) => p.left)).toEqual([
      "LOCA_ID",
      "SAMP_TOP",
      "SAMP_REF",
      "SAMP_TYPE",
      "SAMP_ID",
    ]);
  });
  it("MOND ⋈ MONG drops the drifted key (PIPE_REF/MOND_REF) — joins on shared only", () => {
    const pairs = joinKeys(
      { code: "MOND", cols: cols(d, "MOND") },
      { code: "MONG", cols: cols(d, "MONG") },
      d,
    );
    expect(pairs.map((p) => p.left)).toEqual(["LOCA_ID", "MONG_ID"]);
    expect(pairs.some((p) => /REF/.test(p.left))).toBe(false);
  });
  it("GEOL ⋈ SAMP (siblings under LOCA) join on LOCA_ID, order-independent", () => {
    const g = { code: "GEOL", cols: cols(d, "GEOL") };
    const s = { code: "SAMP", cols: cols(d, "SAMP") };
    expect(joinKeys(g, s, d)).toEqual([{ left: "LOCA_ID", right: "LOCA_ID" }]);
    expect(joinKeys(s, g, d)).toEqual([{ left: "LOCA_ID", right: "LOCA_ID" }]);
  });
  it("yields [] when the shared key is physically absent from a table's columns", () => {
    // If SAMP's ingested columns don't carry LOCA_ID (a passthrough/partial
    // group), there is no usable equi-join — joinKeys returns [] so the builder
    // falls back to a single-table query rather than a silent cartesian product.
    const sampNoLoca = {
      code: "SAMP",
      cols: cols(d, "SAMP").filter((c) => c !== "LOCA_ID"),
    };
    expect(joinKeys(sampNoLoca, { code: "LOCA", cols: cols(d, "LOCA") }, d)).toEqual([]);
  });
});

describe("depth helpers", () => {
  const d = dict();
  it("depthRangeOf detects GEOL's TOP/BASE band, null for a non-range group", () => {
    expect(depthRangeOf("GEOL", d)).toEqual({
      loca: "LOCA_ID",
      top: "GEOL_TOP",
      base: "GEOL_BASE",
    });
    expect(depthRangeOf("LOCA", d)).toBeNull();
  });
  it("depthColumnFor prefers SPEC_DPTH (specimen) then SAMP_TOP (sample)", () => {
    expect(depthColumnFor("TREG", cols(d, "TREG"), d)).toEqual({
      col: "SPEC_DPTH",
      level: "specimen",
    });
    expect(depthColumnFor("SAMP", cols(d, "SAMP"), d)).toEqual({
      col: "SAMP_TOP",
      level: "sample",
    });
  });
  it("depthRangeOf is cols-aware: no band when the *_BASE column is absent", () => {
    // GEOL declares GEOL_TOP/GEOL_BASE, but a band is only built when both are
    // physically present in the ingested table (else a DuckDB column-not-found).
    expect(depthRangeOf("GEOL", d, ["LOCA_ID", "GEOL_TOP", "GEOL_BASE"])).toEqual({
      loca: "LOCA_ID",
      top: "GEOL_TOP",
      base: "GEOL_BASE",
    });
    expect(depthRangeOf("GEOL", d, ["LOCA_ID", "GEOL_TOP"])).toBeNull();
  });
  it("depthRangeOf requires same-prefix TOP/BASE (no SAMP_TOP × SPEC_BASE band)", () => {
    // TREG carries an inherited SAMP_TOP and its own SPEC_BASE; those must NOT
    // pair into an incoherent [SAMP_TOP, SPEC_BASE) interval — so TREG is not a
    // depth-range group (its SAMP_TOP has no SAMP_BASE here).
    expect(depthRangeOf("TREG", d)).toBeNull();
  });
});

describe("geologyTemplate", () => {
  const d = dict();
  it("builds a TREG × GEOL half-open depth band with specimen + geology desc", () => {
    const metas = ["GEOL", "TREG"].map((code) => ({ code, headings: cols(d, code) }));
    const t = geologyTemplate(metas, d)!;
    expect(t).not.toBeNull();
    expect(t.name).toBe("TREG × GEOL stratum");
    expect(t.sql).toContain('LEFT JOIN "GEOL" g');
    expect(t.sql).toContain('t."SPEC_DPTH" >= g."GEOL_TOP"');
    expect(t.sql).toContain('t."SPEC_DPTH" < g."GEOL_BASE"');
    expect(t.sql).toContain('t."SPEC_DESC"');
    expect(t.sql).toContain('g."GEOL_DESC"');
  });
  it("returns null when no sample/test base group is present", () => {
    expect(
      geologyTemplate([{ code: "GEOL", headings: cols(d, "GEOL") }], d),
    ).toBeNull();
  });
});

describe("relExamples", () => {
  it("generates a CHILD ⋈ PARENT LEFT-join example per loaded child-with-parent", () => {
    const d = dict();
    const metas = ["LOCA", "SAMP", "GEOL", "TREG"].map((code) => ({
      code,
      headings: cols(d, code),
    }));
    const ex = relExamples(metas, d);
    const names = ex.map((e) => e.name);
    expect(names).toContain("SAMP ⋈ LOCA");
    expect(names).toContain("GEOL ⋈ LOCA");
    expect(names).toContain("TREG ⋈ SAMP");
    const samp = ex.find((e) => e.name === "SAMP ⋈ LOCA")!;
    expect(samp.sql).toContain('LEFT JOIN "LOCA" p');
    expect(samp.sql).toContain('c."LOCA_ID" = p."LOCA_ID"');
  });
});

describe("asKeyMap", () => {
  it("hands a DictMap to the analytics orphan-finder as a parent+keys view", () => {
    // Same Map object, narrowed to the GroupKeyInfo (parent + keys) shape the
    // analytics module consumes — no copy, no second fetch.
    const d = dict();
    const km = asKeyMap(d);
    expect(km).toBe(d as unknown);
    expect(km.get("SAMP")?.parent).toBe("LOCA");
    expect(km.get("SAMP")?.keys).toContain("LOCA_ID");
  });
});

describe("depthColumnFor — self fallback", () => {
  const d = dict();
  it("falls back to the group's own *_TOP (2DP) when no SPEC_DPTH/SAMP_TOP", () => {
    // GEOL has neither SPEC_DPTH nor SAMP_TOP, but its own GEOL_TOP is a 2DP
    // KEY band-top → the 'self' level.
    expect(depthColumnFor("GEOL", cols(d, "GEOL"), d)).toEqual({
      col: "GEOL_TOP",
      level: "self",
    });
  });
  it("returns null when there is no usable depth column at all", () => {
    // LOCA carries no *_TOP of TYPE 2DP, and the live cols lack SPEC_DPTH/SAMP_TOP.
    expect(depthColumnFor("LOCA", cols(d, "LOCA"), d)).toBeNull();
  });
});

describe("dictMapFromJson — built from the real ags_dictionary.json", () => {
  const real = dictMapFromJson(REAL_UNION);

  it("parses the union: real parent chains + populated contents", () => {
    expect(real.get("SAMP")?.parent).toBe("LOCA");
    expect(real.get("LOCA")?.parent).toBe("PROJ");
    expect(real.get("PROJ")?.parent).toBeNull();
    // descriptions are present in the faithful dictionary (the old scaffolded
    // copy had ~91% empty), so the builder's `contents` is non-empty.
    expect(real.get("LOCA")?.contents).toBeTruthy();
  });

  it("treats combined KEY+REQUIRED statuses as KEY (e.g. PROJ_ID)", () => {
    // The official dictionary marks PROJ_ID 'KEY+REQUIRED'; a bare
    // `status === "KEY"` would miss it. Guards the +-aware KEY detection.
    expect(real.get("PROJ")?.keys).toContain("PROJ_ID");
  });

  it("derives GEOL's real TOP/BASE depth band from the union", () => {
    expect(depthRangeOf("GEOL", real)).toEqual({
      loca: "LOCA_ID",
      top: "GEOL_TOP",
      base: "GEOL_BASE",
    });
  });

  it("offers real relationships: SAMP ⋈ LOCA appears for a loaded subset", () => {
    const codes = ["PROJ", "LOCA", "SAMP", "GEOL"];
    const metas = codes.map((code) => ({
      code,
      headings: (real.get(code)?.headings ?? []).map((h) => h.name),
    }));
    const names = relExamples(metas, real).map((e) => e.name);
    expect(names).toContain("SAMP ⋈ LOCA");
  });
});

describe("geologyTemplate — non-GEOL depth-range group", () => {
  it("uses any loaded depth-range group as the stratum when GEOL is absent", () => {
    // A bespoke STRT group plays GEOL's role: it's a depth-range group
    // (STRT_TOP/STRT_BASE, both 2DP) under LOCA. geologyTemplate must discover
    // it via isDepthRangeGroup rather than the hard-coded "GEOL".
    const m: DictMap = new Map();
    const add = (
      code: string,
      parent: string | null,
      headings: { name: string; status: string; type: string }[],
    ) =>
      m.set(code, {
        parent,
        keys: headings.filter((h) => h.status === "KEY").map((h) => h.name),
        headings,
        contents: `${code} group`,
      });
    add("LOCA", null, [H("LOCA_ID", "KEY", "ID")]);
    add("STRT", "LOCA", [
      H("LOCA_ID", "KEY", "ID"),
      H("STRT_TOP", "KEY", "2DP"),
      H("STRT_BASE", "REQUIRED", "2DP"),
      H("STRT_DESC", "OTHER", "X"),
    ]);
    add("SAMP", "LOCA", [
      H("LOCA_ID", "KEY", "ID"),
      H("SAMP_TOP", "KEY", "2DP"),
      H("SAMP_ID", "KEY", "ID"),
    ]);
    const cz = (code: string) => (m.get(code)?.headings ?? []).map((h) => h.name);
    const metas = ["STRT", "SAMP"].map((code) => ({ code, headings: cz(code) }));
    const t = geologyTemplate(metas, m)!;
    expect(t).not.toBeNull();
    expect(t.name).toBe("SAMP × STRT stratum");
    expect(t.sql).toContain('LEFT JOIN "STRT" g');
    expect(t.sql).toContain('t."SAMP_TOP" >= g."STRT_TOP"');
    expect(t.sql).toContain('t."SAMP_TOP" < g."STRT_BASE"');
  });
});
