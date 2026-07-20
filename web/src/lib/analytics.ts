// Explore analytics: referential-integrity orphan finding, column
// completeness, and LOCA × group coverage — all computed in DuckDB-wasm over
// the already-ingested typed tables. Pure query builders that take a `run`
// (so this stays free of the duck/apache-arrow imports; AnalyseView passes
// duck.run, keeping those behind the lazy Explore boundary).

import { scalarText, type GroupMeta } from "./duckTypes";
import type { Table } from "apache-arrow";

type Run = (sql: string) => Promise<Table>;

/** Quote a SQL identifier (table/column), doubling internal quotes. */
const q = (id: string) => `"${id.replace(/"/g, '""')}"`;

/** Display a sample cell value (KEY values are ids/numbers; tame bigint/null). */
const fmt = (v: unknown): string => scalarText(v);

const num = (t: Table, col = "n"): number =>
  Number((t.toArray()[0] as Record<string, unknown>)[col]);

// --- referential integrity (orphan finder) ---

/** parent code + KEY heading names, per group code, from the dictionary. */
export interface GroupKeyInfo {
  parent: string | null;
  keys: string[];
}
export type DictKeyMap = Map<string, GroupKeyInfo>;

export interface OrphanResult {
  child: string;
  parent: string;
  /** the shared KEY columns the link was checked on. */
  keys: string[];
  /** child rows with no matching parent row (non-null keys only). */
  orphans: number;
  total: number;
  /** up to 8 example orphan key tuples. */
  samples: string[][];
}

/** For every child group whose dictionary parent is also loaded, anti-join on
 *  the parent's KEY columns to count rows with no matching parent. */
export async function referentialIntegrity(
  metas: GroupMeta[],
  dict: DictKeyMap,
  run: Run,
): Promise<{ links: number; orphans: OrphanResult[] }> {
  const present = new Map(metas.map((m) => [m.code, m]));
  const orphans: OrphanResult[] = [];
  let links = 0;
  for (const m of metas) {
    const info = dict.get(m.code);
    if (!info?.parent) continue;
    const parentMeta = present.get(info.parent);
    if (!parentMeta) continue;
    const parentKeys = dict.get(info.parent)?.keys ?? [];
    const childCols = new Set(m.headings);
    const parentCols = new Set(parentMeta.headings);
    // The link columns: the parent's KEY headings that the child carries too.
    const shared = parentKeys.filter(
      (k) => childCols.has(k) && parentCols.has(k),
    );
    const firstKey = shared[0];
    if (firstKey === undefined) continue;
    links++;

    const on = shared.map((k) => `c.${q(k)} = p.${q(k)}`).join(" AND ");
    const childNotNull = shared
      .map((k) => `c.${q(k)} IS NOT NULL`)
      .join(" AND ");
    // An orphan: the child key tuple is fully populated but matches no parent.
    const where = `p.${q(firstKey)} IS NULL AND ${childNotNull}`;
    const base = `FROM ${q(m.code)} c LEFT JOIN ${q(info.parent)} p ON ${on} WHERE ${where}`;

    const orphanCount = num(await run(`SELECT count(*) AS n ${base}`));
    const total = num(await run(`SELECT count(*) AS n FROM ${q(m.code)}`));
    let samples: string[][] = [];
    if (orphanCount > 0) {
      // Qualify with the child alias: after the join both sides carry the
      // shared columns, so an unqualified select would be ambiguous.
      const sel = shared.map((k) => `c.${q(k)}`).join(", ");
      const t = await run(`SELECT ${sel} ${base} LIMIT 8`);
      samples = t
        .toArray()
        .map((r) => shared.map((k) => fmt((r as Record<string, unknown>)[k])));
    }
    if (orphanCount > 0) {
      orphans.push({
        child: m.code,
        parent: info.parent,
        keys: shared,
        orphans: orphanCount,
        total,
        samples,
      });
    }
  }
  return { links, orphans };
}

// --- column completeness (+ the "why typed" inputs) ---

export interface ColCompleteness {
  heading: string;
  /** AGS TYPE code (from the file's TYPE row). */
  type: string;
  /** the DuckDB column type the value lands as. */
  sqlType: string;
  filled: number;
  /** 0..1 fraction of rows with a non-null value. */
  pct: number;
}
export interface GroupCompleteness {
  code: string;
  total: number;
  cols: ColCompleteness[];
  /** headings that are 100% empty (present but never populated). */
  emptyCols: string[];
  /** mean fill fraction across columns (0..1). */
  overall: number;
}

export async function completeness(
  metas: GroupMeta[],
  run: Run,
): Promise<GroupCompleteness[]> {
  const out: GroupCompleteness[] = [];
  for (const m of metas) {
    if (m.headings.length === 0) continue;
    // One pass: total rows + non-null count per column (count(col) skips null).
    const counts = m.headings
      .map((h, i) => `count(${q(h)}) AS c${i}`)
      .join(", ");
    const row = (
      await run(`SELECT count(*) AS n, ${counts} FROM ${q(m.code)}`)
    ).toArray()[0] as Record<string, unknown>;
    const total = Number(row.n);
    const cols: ColCompleteness[] = m.headings.map((h, i) => {
      const filled = Number(row[`c${i}`]);
      return {
        heading: h,
        type: m.types[i] ?? "",
        sqlType: m.sql_types[i] ?? "",
        filled,
        pct: total ? filled / total : 0,
      };
    });
    const emptyCols = cols
      .filter((c) => total > 0 && c.filled === 0)
      .map((c) => c.heading);
    const overall = cols.length
      ? cols.reduce((s, c) => s + c.pct, 0) / cols.length
      : 0;
    out.push({ code: m.code, total, cols, emptyCols, overall });
  }
  return out;
}

// --- LOCA × group coverage matrix ---

const MAX_LOCA = 60;
const MAX_GROUPS = 40;

export interface Coverage {
  /** group codes carrying a LOCA_ID column (the matrix columns), bounded to
   *  MAX_GROUPS. */
  groups: string[];
  /** distinct LOCA_IDs (the matrix rows), bounded to MAX_LOCA. */
  locas: string[];
  /** group code → set of LOCA_IDs present in that group. */
  present: Record<string, Set<string>>;
  /** pre-cap totals, so the UI can say which axis was truncated and by how
   *  much (the matrix only renders the capped `groups`/`locas`). */
  totalGroups: number;
  totalLocas: number;
  truncated: boolean;
}

/** Which boreholes (LOCA_ID) appear in which groups — the classic GI
 *  completeness check ("does every LOCA have GEOL / SAMP / …?"). */
export async function coverage(
  metas: GroupMeta[],
  run: Run,
): Promise<Coverage | null> {
  const withLoca = metas.filter((m) => m.headings.includes("LOCA_ID"));
  if (withLoca.length < 2) return null; // nothing to cross-reference
  const present: Record<string, Set<string>> = {};
  const all = new Set<string>();
  for (const m of withLoca) {
    const t = await run(
      `SELECT DISTINCT ${q("LOCA_ID")} AS id FROM ${q(m.code)} WHERE ${q("LOCA_ID")} IS NOT NULL`,
    );
    const s = new Set<string>();
    for (const r of t.toArray()) {
      const id = fmt((r as Record<string, unknown>).id);
      if (id) {
        s.add(id);
        all.add(id);
      }
    }
    present[m.code] = s;
  }
  const locas = [...all].sort();
  const groups = withLoca.map((m) => m.code);
  return {
    groups: groups.slice(0, MAX_GROUPS),
    locas: locas.slice(0, MAX_LOCA),
    present,
    totalGroups: groups.length,
    totalLocas: locas.length,
    truncated: locas.length > MAX_LOCA || groups.length > MAX_GROUPS,
  };
}

/** Axis-aware truncation notice for the coverage matrix. The matrix caps BOTH
 *  boreholes (rows, MAX_LOCA) and groups (columns, MAX_GROUPS), so the notice
 *  must name whichever axis was actually clipped — a rows-only message would
 *  mislead when only columns were dropped (and vice-versa). */
export function coverageTruncationNote(c: Coverage): string {
  const parts: string[] = [];
  if (c.locas.length < c.totalLocas)
    parts.push(`${c.locas.length} of ${c.totalLocas} boreholes`);
  if (c.groups.length < c.totalGroups)
    parts.push(`${c.groups.length} of ${c.totalGroups} groups`);
  return `showing the first ${parts.join(" and ")}.`;
}
