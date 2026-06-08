// Persisted, shareable app settings. Each signal is seeded from the URL hash
// (a shared link — highest priority), then localStorage (your last choice),
// then the default; changes write back to localStorage. A "Copy link" action
// encodes the current settings into a `#key=val&…` hash so a validation
// configuration (dictionary edition, encoding, aligned view, active tab) is
// shareable and restorable — the lint-profile / view-spec recipe.
//
// Module-level + app-lifetime by design (one app, one settings set), so the
// header's Share button and the panes read the same signals without
// prop-drilling. A `valid` guard rejects a tampered link / stale stored value
// so junk can never wedge the UI.

import { createSignal, type Accessor } from "solid-js";
import type { DictVersionOpt, EncodingOpt } from "./validator";
import type { TabId } from "../components/Tabs";
import type { Tool } from "../components/tools/ToolsPane";

// Sub-view types (the in-pane selectors). Owned here so they persist + share
// through the same plumbing as the top tab.
export type ExploreView = "browse" | "sql" | "charts" | "analyse";
export type FixView = "fixes" | "diff";

const PREFIX = "ags4w:";

// Parse the hash once at load (a shared link applies on first paint only).
const hashParams: Record<string, string> = (() => {
  const out: Record<string, string> = {};
  const h = (typeof location !== "undefined" ? location.hash : "").replace(/^#/, "");
  if (!h) return out;
  for (const part of h.split("&")) {
    const eq = part.indexOf("=");
    if (eq > 0) out[part.slice(0, eq)] = decodeURIComponent(part.slice(eq + 1));
  }
  return out;
})();

function seed<T extends string>(key: string, def: T, valid: (s: string) => boolean): T {
  const fromHash = hashParams[key];
  if (fromHash && valid(fromHash)) return fromHash as T;
  try {
    const stored = localStorage.getItem(PREFIX + key);
    if (stored && valid(stored)) return stored as T;
  } catch {
    /* localStorage blocked (private mode) — fall through to default */
  }
  return def;
}

function persisted<T extends string>(
  key: string,
  def: T,
  valid: (s: string) => boolean,
): [Accessor<T>, (v: T) => void] {
  const [get, setRaw] = createSignal<T>(seed(key, def, valid));
  // Persist on write (not via a module-scope createEffect, which would have
  // no reactive owner). These signals are app-lifetime singletons.
  const set = (v: T) => {
    try {
      localStorage.setItem(PREFIX + key, v);
    } catch {
      /* localStorage blocked — keep the in-memory value */
    }
    setRaw(() => v);
  };
  return [get, set];
}

function persistedBool(key: string, def: boolean): [Accessor<boolean>, (v: boolean) => void] {
  const [get, set] = persisted(key, def ? "1" : "0", (s) => s === "0" || s === "1");
  return [() => get() === "1", (v: boolean) => set(v ? "1" : "0")];
}

const DICTS = ["auto", "4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"];
const ENCS = ["utf-8", "windows-1252"];
const TABS = ["validate", "fix", "explore", "tools"];
const EXPLORE_VIEWS = ["browse", "sql", "charts", "analyse"];
const FIX_VIEWS = ["fixes", "diff"];
const TOOLS_LIST = [
  "dictionary",
  "rules",
  "revision",
  "template",
  "anonymiser",
  "formatter",
  "coords",
];

export const [dictVersion, setDictVersion] = persisted<DictVersionOpt>(
  "dict",
  "auto",
  (s) => DICTS.includes(s),
);
export const [encoding, setEncoding] = persisted<EncodingOpt>(
  "enc",
  "utf-8",
  (s) => ENCS.includes(s),
);
export const [aligned, setAligned] = persistedBool("aligned", false);
export const [activeTab, setActiveTab] = persisted<TabId>(
  "tab",
  "validate",
  (s) => TABS.includes(s),
);
// In-pane sub-views — persisted + shareable so a link restores the exact view
// the sender saw (e.g. #tab=tools&tool=coords), not just the top tab.
export const [exploreView, setExploreView] = persisted<ExploreView>(
  "ev",
  "browse",
  (s) => EXPLORE_VIEWS.includes(s),
);
export const [fixView, setFixView] = persisted<FixView>(
  "fv",
  "fixes",
  (s) => FIX_VIEWS.includes(s),
);
export const [toolsTool, setToolsTool] = persisted<Tool>(
  "tool",
  "dictionary",
  (s) => TOOLS_LIST.includes(s),
);

/** Build a shareable URL with the current settings in the hash. */
export function shareUrl(): string {
  const params: Record<string, string> = {
    dict: dictVersion(),
    enc: encoding(),
    aligned: aligned() ? "1" : "0",
    tab: activeTab(),
    ev: exploreView(),
    fv: fixView(),
    tool: toolsTool(),
  };
  const hash = Object.entries(params)
    .map(([k, v]) => `${k}=${encodeURIComponent(v)}`)
    .join("&");
  return `${location.origin}${location.pathname}#${hash}`;
}
