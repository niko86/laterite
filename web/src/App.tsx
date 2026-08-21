import {
  createEffect,
  createResource,
  createSignal,
  Show,
  type Component,
} from "solid-js";
import { ready as engineReady } from "./lib/validatorClient";
import { tokenizerReady } from "./lib/tokenizer";
import { pendingTab, setPendingTab } from "./lib/nav";
import { activeTab, setActiveTab, shareUrl } from "./lib/settings";
import { warmLazyAssets } from "./lib/prefetch";
import { type TabId, Tabs } from "./components/Tabs";
import { ValidatePane } from "./components/validate/ValidatePane";
import { FixPane } from "./components/fix/FixPane";
import { ExplorePane } from "./components/explore/ExplorePane";
import { ToolsPane } from "./components/tools/ToolsPane";
import { ExportPane } from "./components/export/ExportPane";
import { PwaUpdater } from "./components/PwaUpdater";
import { ThemeToggle } from "./shared/components";
import mark from "../../assets/laterite-icon-128.png";

const App: Component = () => {
  // Gate the panes on the tiny main-thread tokenizer wasm (laterite-dev#533) ALONE — the
  // validate/fix views + tools call it synchronously through
  // `splitAgsFields`/`quoteAgsField`, so a pane genuinely cannot paint before
  // it. The multi-MB engine is deliberately NOT awaited (#353): its deadline is
  // when a FILE is loaded, not when the page paints, and a human choosing a
  // file puts seconds between the two. A request that does beat it — the
  // sample buttons can, in milliseconds — queues inside the worker behind the
  // worker's own init promise, so it lands in Validate's existing loading state
  // rather than racing a live-before-ready engine.
  const [bootReady] = createResource(async () => {
    await tokenizerReady();
    return true;
  });
  // The active tab is persisted + shareable (lib/settings) — a reload (or a
  // shared link) restores it.
  const tab = activeTab;
  const setTab = (t: TabId) => {
    setActiveTab(t);
  };

  // Cross-tab jumps (e.g. Explore analytics → Validate) flow through the nav
  // channel so a deep component can switch tabs without prop-drilling setTab.
  createEffect(() => {
    const t = pendingTab();
    if (t) {
      setActiveTab(t as TabId);
      setPendingTab(null);
    }
  });

  // The engine is not awaited before paint — but a FAILED engine is still
  // reported here, because a rejection is not a wait, and #353 only takes the
  // engine out of the LOADING gate. Leaving the failure to the panes was tried
  // and is a permanent silent state: a Solid resource THROWS when read after an
  // error, so an unguarded pane's own error fallback never renders and the
  // tab sits on its spinner for ever (verified by aborting the wasm fetch —
  // #339's lesson, that a failed engine fetch must never go quiet). #359 has
  // since guarded the panes; boot failure stays reported at THIS altitude.
  const [engineUp, setEngineUp] = createSignal(false);
  const [engineError, setEngineError] = createSignal<string | null>(null);
  void engineReady().then(
    () => setEngineUp(true),
    (e: unknown) => setEngineError(String(e)),
  );

  // The two ways boot can end badly, at one altitude. Both blank the panes,
  // which is what they did before this ticket — the change is only that the
  // engine ARRIVING no longer holds them back, not that its failure is quieter.
  const fatal = () =>
    bootReady.error
      ? `The app failed to start: ${String(bootReady.error)}`
      : engineError()
        ? `Failed to load the validator engine: ${engineError()}`
        : null;

  // Once the validator is live, warm the heavy lazy assets (DuckDB / echarts /
  // arrow / proj4 / reference JSONs) during idle time so Explore / Charts /
  // Tools open instantly later — without delaying the validate-first load.
  // Sequenced behind the engine, not the paint: the speculative fetches must
  // not steal bandwidth from the artifact the user is actually waiting on.
  createEffect(() => {
    if (engineUp()) warmLazyAssets();
  });

  return (
    <div class="min-h-screen flex flex-col">
      <header class="border-b border-line">
        <div class="mx-auto w-full max-w-shell px-5 py-4">
          <div class="flex flex-wrap items-center gap-x-2.5 gap-y-1">
            {/* The system's lockup: mark · "laterite" in the display face ·
                a hairline divider · the product name in the UI face · a muted
                qualifier. The product name never sets in the display face — a
                long name in an 800 display weight reads as packaging; the
                display voice is the brand word's. The plate under the mark is
                dark-chrome only: its maroon outline disappears into a
                near-black canvas without one. The ramp token by reference
                (the landing masthead's call): no semantic surface token fits,
                because every surface role flips dark with the theme and this
                plate must stay light in dark chrome — that is its point. */}
            <img
              src={mark}
              alt=""
              width="28"
              height="28"
              class="size-7 rounded-md dark:bg-(--stone-50) dark:p-[3px]"
            />
            <h1 class="flex min-w-0 flex-wrap items-baseline gap-x-2.5 gap-y-1">
              <span class="font-display text-h3 font-extrabold text-accent">
                laterite
              </span>
              <span
                aria-hidden="true"
                class="w-px self-stretch bg-line-strong"
              />
              <span class="text-lead font-semibold text-fg">
                AGS4 Validator
              </span>
              <span class="hidden text-caption text-fg-muted sm:inline">
                + data explorer
              </span>
            </h1>
            <div class="ml-auto flex items-center gap-2">
              <ShareButton />
              <ThemeToggle />
            </div>
          </div>
          <p class="mt-1 text-xs text-fg-faint">
            Runs entirely in your browser — your file never leaves your machine.
            No server, nothing uploaded.
          </p>
        </div>
      </header>

      <Tabs active={tab()} onChange={setTab} />

      <main class="mx-auto w-full max-w-shell flex-1 px-5 py-6">
        {/* Failure first, then the value: reading an errored resource THROWS,
            so a failure checked from inside the fallback never gets to
            render itself. */}
        <Show when={!fatal()} fallback={<p class="text-err">{fatal()}</p>}>
          <Show
            when={bootReady()}
            fallback={<p class="text-fg-muted">Starting…</p>}
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
        </Show>
      </main>

      <footer class="border-t border-line">
        <div class="mx-auto flex w-full max-w-shell flex-wrap items-center gap-x-2 gap-y-1 px-5 py-3 text-xs text-fg-dim">
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
              href="https://docs.laterite.dev/reference/support/"
              target="_blank"
              rel="noopener noreferrer"
              class="text-accent hover:underline"
            >
              in beta
            </a>
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
        </div>
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
      onClick={() => void copy()}
      class="rounded-sm border border-line-strong px-2 py-1 text-xs text-fg-soft transition-colors hover:bg-chip"
      title="Copy a link that restores the current dictionary / encoding / view settings"
    >
      {copied() ? "Link copied ✓" : "Share"}
    </button>
  );
};

export default App;
