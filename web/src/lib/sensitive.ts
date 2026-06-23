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

// The web Anonymiser blanks/tokenises whole cell VALUES — it can't
// pseudonymise — so it must NOT pre-tick identifier columns whose values are
// cross-referenced: blanking them would break the file. (The corpus `censor`
// pseudonymises these instead; same list, different action.)
const NON_PREFILL = new Set(["location_id", "project_id"]);

/** Heading codes to pre-select for redaction: every classified heading except
 *  the identifier categories the web tool can't safely blank. */
export function prefillCodes(doc: SensitiveDoc): Set<string> {
  const out = new Set<string>();
  for (const [code, h] of Object.entries(doc.headings)) {
    if (!NON_PREFILL.has(h.category)) out.add(code);
  }
  return out;
}

/** Heading code → category, for the per-column UI hint. */
export function categoryOf(doc: SensitiveDoc): Map<string, string> {
  return new Map(
    Object.entries(doc.headings).map(([code, h]) => [code, h.category]),
  );
}
