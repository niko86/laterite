/* The engine seam (#397): the shipped wasm validator, behind first interaction.
 *
 * Somebody who arrived to copy an install command should not pay for an engine
 * they never scroll to. So this is a dynamic import and nothing else — no
 * worker, no dispatch layer, no protocol. The app has all three because it
 * validates arbitrary files of arbitrary size and must not block its own UI; the
 * landing page validates one 1.7 kB fixture, where a worker would cost a second
 * round trip per keystroke to avoid a pause too short to see.
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
