// The web's SINGLE dictionary source: the canonical multi-edition UNION
// `ags_dictionary.json` — generated from the official AGS Standard_dictionary
// `.ags` files by `tools/gen_dictionary.py` (the one converter). Every web dict
// consumer reads THIS, and nothing else:
//   • Explore relationships + Analyse build a DictMap from the union directly
//     (see relationships.ts / AnalyseView.tsx);
//   • the Tools reference UIs (Dictionary browser / Template generator) render a
//     single edition PROJECTED from the union (`projectEdition`).
// No per-surface dictionary copy, no second source. Editions are recoverable
// from the union itself: each group carries `eds` (which editions it spans) and
// each heading may carry `eds` (heading-level membership) + `by_ed` (older-
// edition field overrides), so a faithful per-edition view needs no extra data.

import type { StandardDict, DictGroup, DictHeading } from "./validator";

// The edition "auto"/none falls back to is the union's own `fallback_edition`
// (the same value the validator's dict::FALLBACK is generated from) — read from
// the loaded dictionary, never hard-coded here. See `resolveEdition`.

// AGS dictionary statuses are the official source's verbatim values, which can
// be COMBINED (e.g. "KEY+REQUIRED"). Membership is therefore `+`-separated —
// mirroring the Rust `registry.rs::is_key`. A bare `status === "KEY"` would miss
// the combined ones.
const hasStatus = (status: string, part: string): boolean =>
  status.split("+").some((p) => p.trim().toUpperCase() === part);
export const isKeyStatus = (s: string): boolean => hasStatus(s, "KEY");
export const isRequiredStatus = (s: string): boolean =>
  hasStatus(s, "REQUIRED");

/** Older-edition overrides for a heading whose definition differs from the
 *  latest-edition (flat) value. */
interface HeadingByEd {
  description?: string;
  status?: string;
  type?: string;
  unit?: string | null;
}
/** A heading in the heading-local union: the flat fields are the LATEST-edition
 *  definition; `eds` (when present) restricts which editions it exists in;
 *  `by_ed` carries per-edition overrides for older editions. */
export interface UnionHeading {
  name: string;
  status: string;
  type: string;
  unit?: string | null;
  description?: string;
  eds?: string[];
  by_ed?: Record<string, HeadingByEd>;
}
export interface UnionGroup {
  eds?: string[];
  parent: string | null;
  description?: string;
  headings: UnionHeading[];
}
export interface RawUnion {
  default_edition: string;
  /** The auto-select fallback edition (python-parity) — the validator's
   *  dict::FALLBACK is generated from the same field. */
  fallback_edition: string;
  editions: string[];
  groups: Record<string, UnionGroup>;
}

let unionCache: Promise<RawUnion> | null = null;
/** Fetch the union JSON once and share it across every consumer (a static,
 *  cacheable asset; one network round-trip for the whole app). */
export function fetchUnion(): Promise<RawUnion> {
  if (!unionCache) {
    unionCache = fetch(`${import.meta.env.BASE_URL}ags_dictionary.json`).then(
      (res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json() as Promise<RawUnion>;
      },
    );
  }
  return unionCache;
}

/** Group- or heading-level edition membership: a missing `eds` means "present
 *  in every edition this entry spans". */
const inEdition = (eds: string[] | undefined, ed: string): boolean =>
  !eds || eds.includes(ed);

/** Map a UI dict-version to a concrete edition in the union: "auto"/null/"" and
 *  any unrecognised value → the union's `fallback_edition`; a known edition →
 *  itself. */
function resolveEdition(
  raw: RawUnion,
  edition: string | null | undefined,
): string {
  if (!edition || edition === "auto") return raw.fallback_edition;
  return raw.editions.includes(edition) ? edition : raw.fallback_edition;
}

/** Project the union down to ONE edition's standard dictionary — the
 *  `{ags_edition, groups:[…]}` shape the Tools reference UIs render. Groups and
 *  headings not in `edition` are dropped; `by_ed[edition]` overrides are applied
 *  so an older edition shows its own descriptions/types/units. Heading order is
 *  preserved; groups are sorted by code (as the prior wasm export did). */
export function projectEdition(raw: RawUnion, edition: string): StandardDict {
  const ed = resolveEdition(raw, edition);
  // Object.entries carries each group value alongside its code, so there's no
  // indexed re-lookup (and no non-null assertion) below; the sort replicates the
  // prior `.sort()` default lexicographic order on the codes.
  const groups: DictGroup[] = Object.entries(raw.groups)
    .filter(([, g]) => inEdition(g.eds, ed))
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([code, g]) => {
      const headings: DictHeading[] = g.headings
        .filter((h) => inEdition(h.eds, ed))
        .map((h) => {
          const o = h.by_ed?.[ed];
          const unit = o && "unit" in o ? o.unit : h.unit;
          return {
            name: h.name,
            status: o?.status ?? h.status,
            type: o?.type ?? h.type,
            ...(unit ? { unit } : {}),
            description: o?.description ?? h.description ?? "",
          };
        });
      return {
        code,
        contents: g.description ?? "",
        ...(g.parent ? { parent: g.parent } : {}),
        headings,
      };
    });
  return { ags_edition: ed, groups };
}

/** Fetch + project: the per-edition standard dictionary for the Tools UIs. */
export async function loadStandardDict(
  edition: string | null | undefined,
): Promise<StandardDict> {
  return projectEdition(await fetchUnion(), edition ?? "auto");
}

/** The edition set + auto-select fallback, read from the union — so the Tools
 *  edition picker lists exactly what the engine knows, never a hand-copied
 *  array. */
export async function loadEditionMeta(): Promise<{
  editions: string[];
  fallback: string;
}> {
  const raw = await fetchUnion();
  return { editions: raw.editions, fallback: raw.fallback_edition };
}
