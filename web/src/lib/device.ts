// A best-effort "is this a weak machine" signal, shared by the idle prefetch
// and the cold-engine confirmation dialog (EngineGate). It replaced a gate on
// network type, and the reason was a compile: the heavy 36 MB DuckDB engine
// compile is invisible on a fast Mac and punishing on a 2 GB / 2-core machine,
// and the right signal for that is device capability rather than link quality.
//
// But what it MEANS is broader than what prompted it: "this machine cannot
// afford speculation", full stop. Two of its three call sites gate a
// warm-*fetch* that deliberately stops short of compiling anything —
// `repo:web/src/lib/prefetch.ts` for DuckDB and, since #356, for the tier-2
// engine — and only EngineGate gates a real compile. That is intentional, and
// the reading matters: `deviceMemory < 4` and `hardwareConcurrency <= 2` predict
// nothing about the cost of putting bytes in a cache, so under the narrow
// reading those two call sites look like a bug someone should "fix". They are
// not. Speculative bytes are exactly what you do not spend on a constrained
// device — storage quota is tighter there, and the bytes are for a tab the
// visitor may never open. One predicate for all three because it is one policy
// (see the comment at prefetch.ts's heavy-engine gate, which says the same from
// the caller's side).
//
// Inputs are browser-dependent: `deviceMemory` is Chromium-only (and capped at
// 8 GB); `hardwareConcurrency` is universal; the Network Information API
// (`saveData`/`effectiveType`) is Chromium-only. Unknown ⇒ assume capable, so a
// browser that simply doesn't report (Firefox/Safari) is never punished — we
// only down-tier on a *positive* low-end reading.

interface NetInfo {
  saveData?: boolean;
  effectiveType?: string;
}

/** Treat the device as constrained — skip eager engine warm-up, prefer asking
 *  before a big download/compile. True on Data Saver, a slow link, ≤ 2 GB RAM,
 *  or ≤ 2 logical cores. */
export function isLowEndDevice(): boolean {
  const nav = navigator as Navigator & {
    deviceMemory?: number;
    connection?: NetInfo;
  };
  const conn = nav.connection;
  if (conn?.saveData) return true;
  if (conn?.effectiveType && /(^|\b)(2g|slow-2g|3g)\b/.test(conn.effectiveType))
    return true;
  // deviceMemory reports 0.25 | 0.5 | 1 | 2 | 4 | 8; `< 4` ⇒ ≤ 2 GB.
  if (typeof nav.deviceMemory === "number" && nav.deviceMemory < 4) return true;
  if (
    typeof navigator.hardwareConcurrency === "number" &&
    navigator.hardwareConcurrency <= 2
  )
    return true;
  return false;
}
