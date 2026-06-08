// AGS group RELATIONSHIPS for the Explore builders. AGS relates groups through
// a compound KEY + a parent code, and the in-browser DuckDB tables keep the
// denormalised layout (each child physically carries its parent's KEY columns),
// so joins work on the data as-is — this module just DERIVES the join shape
// from the dictionary so the builders can offer it without hand-written SQL.
//
// Pure metadata (dict + plain args → value); no DuckDB/Arrow imports, so it's
// cheap to use anywhere and unit-testable with a hand-built DictMap.

import type { GroupKeyInfo, DictKeyMap } from "./analytics";
import { selectSql, type JoinSpec, type QualifiedCol } from "./sqlgen";

const DEPTH_TYPE = "2DP"; // the AGS TYPE that lands as a DOUBLE column

/** Per-heading metadata we retain (superset of the KEY-only info analytics
 *  needs): adds the AGS TYPE so depth columns (`*_TOP`/`*_BASE` of TYPE 2DP)
 *  are detectable. */
export interface DictHeading {
  name: string;
  status: string; // KEY | REQUIRED | OTHER
  type: string; // AGS TYPE code
}
/** A `DictGroupInfo` IS a `GroupKeyInfo` (parent + keys) plus all headings. */
export interface DictGroupInfo extends GroupKeyInfo {
  parent: string | null;
  keys: string[];
  headings: DictHeading[];
  /** The group's human-readable name (dictionary `contents`, e.g. LOCA →
   *  "Location Details") — shown beside the code in the builder's dropdowns. */
  contents: string;
}
export type DictMap = Map<string, DictGroupInfo>;

/** Fetch + parse ags5_dictionary.json into a DictMap once (replaces
 *  AnalyseView's KEY-only loadKeyMap; the parent+keys shape is unchanged). */
export async function loadDict(): Promise<DictMap> {
  const res = await fetch(`${import.meta.env.BASE_URL}ags5_dictionary.json`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const d = (await res.json()) as {
    groups: {
      code: string;
      contents?: string;
      parent: string | null;
      headings: { name: string; status: string; type: string }[];
    }[];
  };
  const m: DictMap = new Map();
  for (const g of d.groups) {
    const headings = g.headings.map((h) => ({
      name: h.name,
      status: h.status,
      type: h.type,
    }));
    m.set(g.code, {
      parent: g.parent,
      keys: headings.filter((h) => h.status === "KEY").map((h) => h.name),
      headings,
      contents: g.contents ?? "",
    });
  }
  return m;
}

/** A DictMap is structurally a DictKeyMap (parent+keys); this hands it to the
 *  analytics orphan-finder without a second fetch. (Map is invariant in TS, so
 *  the cast is needed; it is runtime-safe — DictGroupInfo ⊇ GroupKeyInfo.) */
export const asKeyMap = (d: DictMap): DictKeyMap =>
  d as unknown as DictKeyMap;

/** True if `anc` is `code` or appears on `code`'s parent chain. */
export function isAncestor(anc: string, code: string, dict: DictMap): boolean {
  if (anc === code) return true;
  const seen = new Set<string>([code]);
  let p = dict.get(code)?.parent ?? null;
  while (p && !seen.has(p)) {
    if (p === anc) return true;
    seen.add(p);
    p = dict.get(p)?.parent ?? null;
  }
  return false;
}

export interface RelatedGroup {
  code: string;
  /** "parent"/"child" = an ancestor/descendant on the parent chain; "related"
   *  = a SIBLING that shares a compound-key column (e.g. GEOL ⋈ SAMP, both
   *  under LOCA, sharing LOCA_ID — the depth-range / stratum case). */
  direction: "parent" | "child" | "related";
  distance: number; // 1 = direct parent/child; siblings use 1
}

/** Groups related to `base` that are ALSO loaded: every loaded ancestor up the
 *  parent chain, every loaded descendant, AND every loaded sibling that shares
 *  a KEY column with `base` (so the builder can offer GEOL for a sample/test —
 *  they're siblings under LOCA, not ancestor/descendant). Parents, then
 *  children, then related; nearest-first, then by code. */
export function relatedGroups(
  base: string,
  loadedCodes: readonly string[],
  dict: DictMap,
): RelatedGroup[] {
  const loaded = new Set(loadedCodes);
  const claimed = new Set<string>([base]); // codes already placed (no dupes)
  const out: RelatedGroup[] = [];

  // Ancestors up the parent chain.
  const seen = new Set<string>([base]);
  let cur = dict.get(base)?.parent ?? null;
  let dist = 1;
  while (cur && !seen.has(cur)) {
    seen.add(cur);
    if (loaded.has(cur)) {
      out.push({ code: cur, direction: "parent", distance: dist });
      claimed.add(cur);
    }
    cur = dict.get(cur)?.parent ?? null;
    dist++;
  }

  // Descendants (loaded groups whose chain reaches base).
  for (const code of loadedCodes) {
    if (claimed.has(code)) continue;
    const guard = new Set<string>([code]);
    let p = dict.get(code)?.parent ?? null;
    let d = 1;
    while (p && !guard.has(p)) {
      if (p === base) {
        out.push({ code, direction: "child", distance: d });
        claimed.add(code);
        break;
      }
      guard.add(p);
      p = dict.get(p)?.parent ?? null;
      d++;
    }
  }

  // Siblings: not on the parent chain, but joinable via shared compound keys.
  // A bare single-key overlap (typically just LOCA_ID) is offered ONLY when the
  // sibling is a depth-range group — there the band (e.g. the GEOL stratum)
  // disambiguates the match. Otherwise a lone-LOCA_ID equi-join is a
  // per-borehole fan-out (every base row × every sibling row in that LOCA), so
  // require a compound overlap (≥2 shared keys) before surfacing it.
  const baseKeys = new Set(dict.get(base)?.keys ?? []);
  for (const code of loadedCodes) {
    if (claimed.has(code)) continue;
    const shared = (dict.get(code)?.keys ?? []).filter((k) => baseKeys.has(k));
    if (shared.length > 1 || (shared.length === 1 && isDepthRangeGroup(code, dict))) {
      out.push({ code, direction: "related", distance: 1 });
      claimed.add(code);
    }
  }

  const rank = { parent: 0, child: 1, related: 2 };
  out.sort((a, b) =>
    a.direction !== b.direction
      ? rank[a.direction] - rank[b.direction]
      : a.distance !== b.distance
        ? a.distance - b.distance
        : a.code.localeCompare(b.code),
  );
  return out;
}

export interface JoinKeyPair {
  left: string;
  right: string;
}

/** The equi-join columns between two loaded groups: the ancestor's KEY headings
 *  that BOTH tables physically carry (`cols` = the live denormalised columns).
 *  Requiring presence in both is what makes pseudo-key drift safe — e.g. MOND
 *  has MOND_REF where MONG has PIPE_REF, so that key simply isn't in the pairs. */
export function joinKeys(
  a: { code: string; cols: readonly string[] },
  b: { code: string; cols: readonly string[] },
  dict: DictMap,
): JoinKeyPair[] {
  const parent = isAncestor(a.code, b.code, dict) ? a : b;
  const parentKeys = dict.get(parent.code)?.keys ?? [];
  const aSet = new Set(a.cols);
  const bSet = new Set(b.cols);
  return parentKeys
    .filter((k) => aSet.has(k) && bSet.has(k))
    .map((k) => ({ left: k, right: k }));
}

export interface DepthRange {
  loca: string;
  top: string;
  base: string;
}

/** A depth-range group spans a (top, base) interval per LOCA (GEOL is the
 *  canonical one): has LOCA_ID, a `*_TOP` KEY of TYPE 2DP, and a `*_BASE` of
 *  TYPE 2DP. Returns the column names so callers don't hard-code GEOL. */
export function depthRangeOf(
  code: string,
  dict: DictMap,
  cols?: readonly string[],
): DepthRange | null {
  const info = dict.get(code);
  if (!info || !info.headings.some((h) => h.name === "LOCA_ID")) return null;
  // When the live columns are known, require the band columns to be PHYSICALLY
  // present: many groups (SAMP, CORE, PIPE, …) declare a `*_BASE` in the
  // dictionary that real files routinely omit, and emitting a predicate on a
  // column the ingested table lacks is a DuckDB "column not found".
  const present = cols ? new Set(cols) : null;
  if (present && !present.has("LOCA_ID")) return null;
  // The band is one group's OWN interval: a `*_TOP` KEY of depth type paired
  // with its SAME-prefix `*_BASE`. Same-prefix matters because a child carries
  // an inherited parent `*_TOP` (e.g. TREG's SAMP_TOP) that must not pair with
  // an unrelated `*_BASE` (SPEC_BASE) into an incoherent [SAMP_TOP, SPEC_BASE).
  for (const h of info.headings) {
    if (!/_TOP$/.test(h.name) || h.type !== DEPTH_TYPE || h.status !== "KEY")
      continue;
    const baseName = `${h.name.slice(0, -"_TOP".length)}_BASE`;
    const hasBase = info.headings.some(
      (b) => b.name === baseName && b.type === DEPTH_TYPE,
    );
    if (!hasBase) continue;
    if (present && !(present.has(h.name) && present.has(baseName))) continue;
    return { loca: "LOCA_ID", top: h.name, base: baseName };
  }
  return null;
}

export const isDepthRangeGroup = (
  code: string,
  dict: DictMap,
  cols?: readonly string[],
): boolean => depthRangeOf(code, dict, cols) !== null;

export interface DepthColumn {
  col: string;
  level: "specimen" | "sample" | "self";
}

/** The single depth to probe a stratum with, for a sample/test group: SPEC_DPTH
 *  if present (specimen level), else SAMP_TOP (sample level), else the group's
 *  own `*_TOP`. `cols` = the live denormalised columns. */
export function depthColumnFor(
  code: string,
  cols: readonly string[],
  dict: DictMap,
): DepthColumn | null {
  const set = new Set(cols);
  if (set.has("SPEC_DPTH")) return { col: "SPEC_DPTH", level: "specimen" };
  if (set.has("SAMP_TOP")) return { col: "SAMP_TOP", level: "sample" };
  const own = dict
    .get(code)
    ?.headings.find((h) => /_TOP$/.test(h.name) && h.type === DEPTH_TYPE);
  return own && set.has(own.name) ? { col: own.name, level: "self" } : null;
}

/** Sample/test groups, deepest (most specific) first — the base for the GEOL
 *  stratum template. The deepest one carries the specimen description. */
const TEST_GROUPS = ["TREL", "TRET", "TREG", "TRIL", "TRIT", "TRIG", "SAMP"];

/** The flagship "× GEOL stratum" template: relate a sample/test (at a depth) to
 *  the geology stratum whose band contains it — a NON-equi (depth-range) join,
 *  surfacing the stratum description (GEOL_LEG/GEOL_GEOL/GEOL_DESC) and, when
 *  the base is a specimen group, its SPEC_DESC. Returns null unless a depth-
 *  range group (GEOL) and a sample/test group with a depth column are loaded. */
export function geologyTemplate(
  metas: { code: string; headings: string[] }[],
  dict: DictMap,
): { name: string; sql: string } | null {
  const present = new Map(metas.map((m) => [m.code, m]));
  const geolCode = present.has("GEOL")
    ? "GEOL"
    : metas.find((m) => isDepthRangeGroup(m.code, dict, m.headings))?.code;
  if (!geolCode) return null;
  const geol = present.get(geolCode);
  if (!geol) return null;
  const range = depthRangeOf(geolCode, dict, geol.headings);
  if (!range) return null;

  let base: { code: string; headings: string[] } | undefined;
  let depth: DepthColumn | null = null;
  for (const code of TEST_GROUPS) {
    const m = present.get(code);
    if (!m || code === geolCode || !m.headings.includes("LOCA_ID")) continue;
    const dc = depthColumnFor(code, m.headings, dict);
    if (dc) {
      base = m;
      depth = dc;
      break;
    }
  }
  if (!base || !depth) return null;

  const pairs = joinKeys(
    { code: base.code, cols: base.headings },
    { code: geolCode, cols: geol.headings },
    dict,
  ).filter((p) => p.left === "LOCA_ID");
  if (!pairs.length) return null;

  const baseCols = new Set(base.headings);
  const geolCols = new Set(geol.headings);
  const sel: QualifiedCol[] = [];
  if (baseCols.has("LOCA_ID")) sel.push({ alias: "t", col: "LOCA_ID" });
  sel.push({ alias: "t", col: depth.col });
  if (baseCols.has("SPEC_DESC")) sel.push({ alias: "t", col: "SPEC_DESC" });
  for (const c of ["GEOL_LEG", "GEOL_GEOL", "GEOL_DESC"])
    if (geolCols.has(c)) sel.push({ alias: "g", col: c });

  const join: JoinSpec = {
    table: geolCode,
    alias: "g",
    kind: "LEFT",
    leftAlias: "t",
    on: pairs,
    range: { baseAlias: "t", baseCol: depth.col, top: range.top, base: range.base },
  };
  return {
    name: `${base.code} × ${geolCode} stratum`,
    sql: selectSql({
      table: base.code,
      alias: "t",
      joins: [join],
      select: sel,
      columns: [],
      conditions: [],
      orderDir: "ASC",
      limit: 100,
    }),
  };
}

/** Curated example queries from the loaded groups + dict: the flagship GEOL
 *  stratum template (when applicable), then for each loaded child whose parent
 *  is loaded (with ≥1 shared key) a `CHILD ⋈ PARENT` LEFT-join selecting the
 *  child's keys + a few parent columns. */
export function relExamples(
  metas: { code: string; headings: string[] }[],
  dict: DictMap,
): { name: string; sql: string }[] {
  const present = new Set(metas.map((m) => m.code));
  const byCode = new Map(metas.map((m) => [m.code, m]));
  const out: { name: string; sql: string }[] = [];
  const geo = geologyTemplate(metas, dict);
  if (geo) out.push(geo);
  for (const m of metas) {
    const parent = dict.get(m.code)?.parent;
    if (!parent || !present.has(parent)) continue;
    const pm = byCode.get(parent)!;
    const pairs = joinKeys(
      { code: m.code, cols: m.headings },
      { code: parent, cols: pm.headings },
      dict,
    );
    if (!pairs.length) continue;
    const childKeys = (dict.get(m.code)?.keys ?? []).filter((k) =>
      m.headings.includes(k),
    );
    const parentKeySet = new Set(dict.get(parent)?.keys ?? []);
    const parentExtra = pm.headings.filter((h) => !parentKeySet.has(h)).slice(0, 3);
    const select: QualifiedCol[] = [
      ...childKeys.map((col) => ({ alias: "c", col })),
      ...parentExtra.map((col) => ({ alias: "p", col })),
    ];
    const join: JoinSpec = {
      table: parent,
      alias: "p",
      kind: "LEFT",
      leftAlias: "c",
      on: pairs,
    };
    out.push({
      name: `${m.code} ⋈ ${parent}`,
      sql: selectSql({
        table: m.code,
        alias: "c",
        joins: [join],
        select,
        columns: [],
        conditions: [],
        orderDir: "ASC",
        limit: 100,
      }),
    });
  }
  return out;
}
