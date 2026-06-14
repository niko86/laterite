import {
  createEffect,
  createResource,
  createSignal,
  Show,
  type Component,
} from "solid-js";
import { ready as validatorReady } from "./lib/validatorClient";
import { pendingTab, setPendingTab } from "./lib/nav";
import { activeTab, setActiveTab, shareUrl } from "./lib/settings";
import { warmLazyAssets } from "./lib/prefetch";
import { type TabId, Tabs } from "./components/Tabs";
import { ValidatePane } from "./components/validate/ValidatePane";
import { FixPane } from "./components/fix/FixPane";
import { ExplorePane } from "./components/explore/ExplorePane";
import { ToolsPane } from "./components/tools/ToolsPane";
import { ExportPane } from "./components/export/ExportPane";
import { ThemeToggle } from "./components/ThemeToggle";
import { PwaUpdater } from "./components/PwaUpdater";

const App: Component = () => {
  // Gate the panes on wasm instantiation (now inside the worker). A
  // resource keeps the loading / error states declarative — no top-level
  // await, no race where a pane calls validate() before the module is live.
  const [wasmReady] = createResource(async () => {
    await validatorReady();
    return true;
  });
  // The active tab is persisted + shareable (lib/settings) — a reload (or a
  // shared link) restores it.
  const tab = activeTab;
  const setTab = (t: TabId) => setActiveTab(t);

  // Cross-tab jumps (e.g. Explore analytics → Validate) flow through the nav
  // channel so a deep component can switch tabs without prop-drilling setTab.
  createEffect(() => {
    const t = pendingTab();
    if (t) {
      setActiveTab(t as TabId);
      setPendingTab(null);
    }
  });

  // Once the validator is live, warm the heavy lazy assets (DuckDB / echarts /
  // arrow / proj4 / reference JSONs) during idle time so Explore / Charts /
  // Tools open instantly later — without delaying the validate-first load.
  createEffect(() => {
    if (wasmReady()) warmLazyAssets();
  });

  return (
    <div class="min-h-screen flex flex-col">
      <header class="border-b border-line px-4 py-4 sm:px-6">
        <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <h1 class="text-xl font-semibold text-fg">AGS4 Validator</h1>
          <span class="hidden text-sm text-fg-muted sm:inline">
            + data explorer
          </span>
          <div class="ml-auto flex items-center gap-2 self-center">
            <ShareButton />
            <ThemeToggle />
          </div>
        </div>
        <p class="mt-1 text-xs text-fg-faint">
          Runs entirely in your browser — your file never leaves your machine.
          No server, nothing uploaded.
        </p>
      </header>

      <Tabs active={tab()} onChange={setTab} />

      <main class="mx-auto w-full max-w-7xl flex-1 px-4 py-6 sm:px-6">
        <Show
          when={wasmReady()}
          fallback={
            <Show
              when={!wasmReady.error}
              fallback={
                <p class="text-err">
                  Failed to load the validator engine: {String(wasmReady.error)}
                </p>
              }
            >
              <p class="text-fg-muted">Loading validator engine…</p>
            </Show>
          }
        >
          <Show when={tab() === "validate"}>
            <ValidatePane />
          </Show>
          <Show when={tab() === "fix"}>
            <FixPane />
          </Show>
          <Show when={tab() === "explore"}>
            <ExplorePane />
          </Show>
          <Show when={tab() === "tools"}>
            <ToolsPane />
          </Show>
          <Show when={tab() === "export"}>
            <ExportPane />
          </Show>
        </Show>
      </main>

      <footer class="flex flex-wrap items-center gap-x-2 gap-y-1 border-t border-line px-4 py-3 text-xs text-fg-dim sm:px-6">
        <span>
          Powered by{" "}
          <a
            href="https://github.com/niko86/laterite"
            target="_blank"
            rel="noopener noreferrer"
            class="font-medium text-accent hover:underline"
          >
            laterite
          </a>
          , a clean-room Rust AGS4 engine compiled to WebAssembly — the same
          engine runs this app.
        </span>
        <span class="whitespace-nowrap">
          <span class="text-fg-faint" aria-hidden="true">
            ·
          </span>{" "}
          <a
            href="https://github.com/niko86/laterite"
            target="_blank"
            rel="noopener noreferrer"
            class="text-accent hover:underline"
          >
            GitHub
          </a>
        </span>
        <span class="whitespace-nowrap">
          <span class="text-fg-faint" aria-hidden="true">
            ·
          </span>{" "}
          <a
            href="https://pypi.org/project/laterite/"
            target="_blank"
            rel="noopener noreferrer"
            class="text-accent hover:underline"
          >
            PyPI
          </a>
        </span>
      </footer>

      <PwaUpdater />
    </div>
  );
};

// Copies a link that restores the current settings (dictionary edition,
// encoding, aligned view, active tab) — the shareable view-spec.
const ShareButton: Component = () => {
  const [copied, setCopied] = createSignal(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl());
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard blocked — no-op */
    }
  };
  return (
    <button
      type="button"
      onClick={copy}
      class="rounded border border-line-strong px-2 py-1 text-xs text-fg-soft transition-colors hover:bg-chip"
      title="Copy a link that restores the current dictionary / encoding / view settings"
    >
      {copied() ? "Link copied ✓" : "Share"}
    </button>
  );
};

export default App;
