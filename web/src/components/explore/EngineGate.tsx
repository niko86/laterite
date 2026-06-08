import { createResource, createSignal, Show, type Component } from "solid-js";
import { isLowEndDevice } from "../../lib/device";
import { goTo } from "../../lib/nav";

// The cold-engine confirmation. Explore needs a 36 MB in-browser database
// engine (DuckDB-wasm) that downloads + compiles on first use — invisible on a
// fast Mac, but several seconds (+ a worker + tens of MB of RAM) on a weak one.
// So on a LOW-END device we ask before paying that, rather than letting the
// user click Explore and meet a silent multi-second freeze they didn't expect.
// Capable devices and repeat visits proceed straight in (no dialog).

const DISMISS_KEY = "ags-engine-gate-skip"; // localStorage: chose "don't ask again"
let confirmedThisSession = false;

/** True when a cold Explore should pause to confirm the engine bring-up: only
 *  on a low-end device, not already confirmed this session, not persistently
 *  dismissed. Capable hardware (and every repeat visit) returns false. */
export function engineGateNeeded(): boolean {
  if (confirmedThisSession) return false;
  if (!isLowEndDevice()) return false;
  try {
    if (localStorage.getItem(DISMISS_KEY) === "1") return false;
  } catch {
    /* private mode / blocked storage — fall through and ask */
  }
  return true;
}

export const EngineGate: Component<{ onConfirm: () => void }> = (props) => {
  const [dontAsk, setDontAsk] = createSignal(false);
  // Tailor the wording: if the engine wasm is already in the SW runtime cache
  // (a return visit, or a capable-device warm-fetch), only a compile remains —
  // no 38 MB download — so don't over-warn about bandwidth.
  const [wasmCached] = createResource(async () => {
    try {
      const c = await caches.open("ags-duckdb-wasm");
      return (await c.keys()).length > 0;
    } catch {
      return false;
    }
  });

  const confirm = () => {
    confirmedThisSession = true;
    if (dontAsk()) {
      try {
        localStorage.setItem(DISMISS_KEY, "1");
      } catch {
        /* ignore — the session flag still suppresses re-asking */
      }
    }
    props.onConfirm();
  };

  return (
    <div class="mx-auto max-w-prose rounded-lg border border-line-strong bg-surface p-5">
      <p class="text-base font-medium text-fg">Open the data explorer?</p>
      <p class="mt-2 text-sm text-fg-muted">
        Explore runs an in-browser database engine.
        <Show
          when={wasmCached()}
          fallback={
            <> The first use downloads it (~38&nbsp;MB) and compiles it, which can take several seconds on this device.</>
          }
        >
          <> Starting it can take a few seconds on this device.</>
        </Show>{" "}
        Validate and Fix don't need it.
      </p>
      <div class="mt-4 flex flex-wrap items-center gap-3">
        <button
          type="button"
          class="rounded bg-accent px-3 py-1.5 text-sm font-medium text-canvas hover:opacity-90"
          onClick={confirm}
        >
          Continue
        </button>
        <button
          type="button"
          class="rounded border border-line-strong px-3 py-1.5 text-sm text-fg-soft hover:bg-chip"
          onClick={() => goTo("validate")}
        >
          Back to Validate
        </button>
        <label class="ml-auto flex items-center gap-1.5 text-xs text-fg-faint">
          <input
            type="checkbox"
            checked={dontAsk()}
            onInput={(e) => setDontAsk(e.currentTarget.checked)}
          />
          Don't ask again
        </label>
      </div>
    </div>
  );
};
