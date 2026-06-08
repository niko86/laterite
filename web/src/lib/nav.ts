// Tiny cross-tab navigation channel. A pane (e.g. Explore's analytics) can
// request a jump to another tab without the tab state being threaded through
// props — App watches `pendingTab` and applies it. App-lifetime, single
// consumer, by design (mirrors fileStore's module-level signals).

import { createSignal } from "solid-js";

export const [pendingTab, setPendingTab] = createSignal<string | null>(null);

/** Request the app switch to `tab` (e.g. "validate" | "fix" | "explore"). */
export function goTo(tab: string): void {
  setPendingTab(tab);
}
