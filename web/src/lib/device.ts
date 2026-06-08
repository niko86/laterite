// A best-effort "is this a weak machine" signal, shared by the idle prefetch
// (skip the eager DuckDB warm) and the cold-engine confirmation dialog
// (EngineGate). The whole point: the heavy 36 MB DuckDB engine compile is
// invisible on a fast Mac but punishing on a 2 GB / 2-core machine, and the
// *right* signal for that is device capability — not the network type the
// prefetch used to gate on.
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
