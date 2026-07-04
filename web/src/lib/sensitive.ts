// Loader for the sensitive-headings SSOT (sensitive_headings.json), the same
// classification the corpus `censor` tool uses. The Anonymiser fetches it to
// decide which columns to pre-select for redaction — one list, two anonymisers
// (each maps category → its own action). Synced to web/public by
// scripts/sync-sensitive.mjs (a predev/prebuild hook).

export interface SensitiveHeading {
  category: string;
  desc?: string;
}
export interface SensitiveDoc {
  categories: Record<string, string>;
  scrub_policy: Record<string, string>;
  headings: Record<string, SensitiveHeading>;
}

let cache: Promise<SensitiveDoc> | null = null;
/** Fetch the list once and share it across the app (a static, cacheable
 *  asset). */
export function loadSensitive(): Promise<SensitiveDoc> {
  if (!cache) {
    cache = fetch(`${import.meta.env.BASE_URL}sensitive_headings.json`).then(
      (res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json() as Promise<SensitiveDoc>;
      },
    );
  }
  return cache;
}

/** The redaction action for a heading, taken from the SSOT's `scrub_policy`
 *  (the same category→action map the corpus `censor` uses): `pseudonym` for
 *  location IDs (a stable per-column token, cross-references intact), `filehash`
 *  for PROJ_ID (the file's content hash), `blank` for coordinates, `token` for
 *  names/labs/etc., `brackets` for free-text `[units]`. */
export type Action = "filehash" | "pseudonym" | "blank" | "token" | "brackets";

/** Pre-tick scopes for the Anonymiser. */
export type Preset = "coords" | "coords-text" | "all";

const PRESET_CATEGORIES: Record<Preset, ReadonlySet<string> | "all"> = {
  coords: new Set(["coordinate"]),
  "coords-text": new Set(["coordinate", "remark", "freetext"]),
  all: "all",
};

/** Heading codes to pre-tick for a preset, by their SSOT category. */
export function codesForPreset(doc: SensitiveDoc, preset: Preset): Set<string> {
  const cats = PRESET_CATEGORIES[preset];
  const out = new Set<string>();
  for (const [code, h] of Object.entries(doc.headings)) {
    if (cats === "all" || cats.has(h.category)) out.add(code);
  }
  return out;
}

/** All classified sensitive headings — the default ("all identifying") pre-tick.
 *  Now includes the identifier categories: the Anonymiser pseudonymises IDs and
 *  hashes PROJ_ID (rather than blanking), so cross-references stay intact. */
export function prefillCodes(doc: SensitiveDoc): Set<string> {
  return codesForPreset(doc, "all");
}

/** Heading code → its redaction action (via category → `scrub_policy`). */
export function actionOf(doc: SensitiveDoc): Map<string, Action> {
  const m = new Map<string, Action>();
  for (const [code, h] of Object.entries(doc.headings)) {
    const a = doc.scrub_policy[h.category];
    if (a) m.set(code, a as Action);
  }
  return m;
}

/** Heading code → category, for the per-column UI hint. */
export function categoryOf(doc: SensitiveDoc): Map<string, string> {
  return new Map(
    Object.entries(doc.headings).map(([code, h]) => [code, h.category]),
  );
}
