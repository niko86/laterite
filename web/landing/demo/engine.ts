/* The engine seam (#397; loading policy revised by #531): the shipped wasm
 * validator, behind a dynamic import and nothing else — no worker, no
 * dispatch layer, no protocol. The app has all three because it validates
 * arbitrary files of arbitrary size and must not block its own UI; the
 * landing page validates one small committed fixture, where a worker would
 * cost a second round trip per keystroke to avoid a pause too short to see.
 *
 * WHEN the import happens moved in #531: eager-idle after first paint rather
 * than behind first interaction — the demo is the page's thesis, and the
 * wasm travels brotli-compressed on the real delivery path, so the blank
 * pane cost more than the bytes. store.ts's armWhenIdle records that
 * decision; this module still only answers "load it now".
 *
 * That asymmetry is deliberate and it is the same reason the two surfaces have
 * two Vite configs: sharing a toolchain is not sharing a bundle.
 *
 * There is NO file input on this page. A reader who wants to open their own
 * delivery goes to the web app, and the page's job is to link there.
 */

type WasmModule = typeof import("../../src/wasm/ags4_wasm");

let loaded: WasmModule | null = null;
let loading: Promise<WasmModule> | null = null;

/** True once the engine is resident, so the UI can say "validating…" exactly
 *  once rather than flickering on every keystroke afterwards. */
export function isLoaded(): boolean {
  return loaded !== null;
}

export function engine(): Promise<WasmModule> {
  if (loaded) return Promise.resolve(loaded);
  loading ??= import("../../src/wasm/ags4_wasm").then(async (m) => {
    await m.default();
    loaded = m;
    return m;
  });
  return loading;
}

export type Severity = "error" | "warning" | "fyi";

/** One finding, flattened to what the page renders. Severity comes from the
 *  engine and is never decided here — the seeded SAMP_TYPE defect is an error,
 *  whatever the design handoff captions it. */
export type Finding = {
  readonly rule: string;
  readonly line: number | null;
  readonly group: string;
  readonly heading: string | null;
  /** 1-based row ordinal within the group, which is what lets a finding point
   *  at a CELL in the tables rather than only at a line in the output pane. */
  readonly dataRow: number | null;
  readonly severity: Severity;
  readonly desc: string;
};

export type Report = {
  readonly ok: boolean;
  readonly findings: readonly Finding[];
  readonly error?: { kind: string; message: string };
};

const encoder = new TextEncoder();

export async function validateText(text: string): Promise<Report> {
  const m = await engine();
  const raw = m.validate(encoder.encode(text));
  if (raw.error) return { ok: false, findings: [], error: raw.error };
  return { ok: raw.ok, findings: flatten(raw.findings) };
}

/** The engine's own fix record — re-exported so the store and the pure
 *  scoping helpers (fixes.ts) speak the wasm surface's type, never a copy. */
export type Fix = import("../../src/wasm/ags4_wasm").Fix;

/** The engine's fixes for this text, filtered to `risk === "safe"` — which is
 *  the filter `lat fix` applies by default, so the demo never repairs more
 *  than the CLI would (#530). The filter has to live HERE (#583): the engine
 *  hands over its whole list — `compute_fixes` does not read risk, and the
 *  wasm apply path never goes through `fix_document_selective`'s opt-in — and
 *  every consumer (the budget count, the fix button, the manual badge) reads
 *  this one seam, so filtering here is what keeps the three telling one
 *  story: a risky fix does not exist as far as this page is concerned, and
 *  its finding reads as manual. */
export async function computeFixesText(text: string): Promise<Fix[]> {
  const m = await engine();
  const all: Fix[] = m.compute_fixes(encoder.encode(text));
  return all.filter((f) => f.risk === "safe");
}

/** Apply a subset of the engine's fixes to the text, returning the new text.
 *  The subset is the caller's scoping decision; the edits are the engine's. */
export async function applyFixesText(
  text: string,
  fixes: readonly Fix[],
): Promise<string> {
  const m = await engine();
  return new TextDecoder().decode(
    m.apply_fixes(encoder.encode(text), null, fixes),
  );
}

type RuleGroupLike = {
  rule: string;
  items: {
    line: number | null;
    group: string;
    desc: string;
    heading?: string;
    data_row?: number;
    severity?: "warning" | "fyi";
  }[];
};

/** Findings arrive grouped by rule; the page renders one flat, line-ordered
 *  list. Two things the engine's shape makes easy to get wrong:
 *
 *  - **An absent `severity` means `error`**, not "unannotated". The engine omits
 *    the field for the most severe case rather than writing it, so a naive
 *    `severity ?? "info"` would downgrade every real error on the page.
 *  - Whole-file rules (14 here) carry `line: null`. They sort LAST rather than
 *    first, because the reader works down the output pane and a finding they
 *    cannot jump to should not sit above one they can.
 */
export function flatten(
  groups: readonly RuleGroupLike[] | undefined,
): Finding[] {
  const out: Finding[] = [];
  for (const g of groups ?? []) {
    for (const item of g.items) {
      out.push({
        rule: g.rule,
        line: item.line,
        group: item.group,
        heading: item.heading ?? null,
        dataRow: item.data_row ?? null,
        severity: item.severity ?? "error",
        desc: item.desc,
      });
    }
  }
  return out.sort(
    (a, b) =>
      (a.line ?? Number.MAX_SAFE_INTEGER) - (b.line ?? Number.MAX_SAFE_INTEGER),
  );
}

/** Rule 16 names its target only in prose: the finding carries no heading
 *  and no row — correctly, since it is a statement about the group's USE of
 *  an abbreviation, not about one cell. This reads the value and the heading
 *  back out of that prose so the tables can light the cells that carry it
 *  (#590). Null for every other finding shape; the message text is the
 *  contract, so the fixture test beside this pins the exact wording the
 *  engine emits. */
export function abbreviationTarget(
  f: Finding,
): { value: string; heading: string } | null {
  if (!f.rule.endsWith("Rule 16") || f.heading !== null || f.dataRow !== null)
    return null;
  const m = /^Abbreviation "(.*)" under (\S+) is not defined/.exec(f.desc);
  return m ? { value: m[1] as string, heading: m[2] as string } : null;
}

/** Every cell in one group's block that carries the abbreviation a Rule 16
 *  finding names — the finding→cell mapping. A group finding may light
 *  SEVERAL cells: pinning it to the first row would lie in a file where rows
 *  1 and 3 both carry the value, which is why the finding maps to all of
 *  them or none. */
export function abbreviationCells(
  f: Finding,
  headings: readonly string[],
  rows: readonly (readonly string[])[],
): { row: number; col: number }[] {
  const target = abbreviationTarget(f);
  if (!target) return [];
  const col = headings.indexOf(target.heading);
  if (col === -1) return [];
  const out: { row: number; col: number }[] = [];
  rows.forEach((r, row) => {
    if (r[col] === target.value) out.push({ row, col });
  });
  return out;
}
