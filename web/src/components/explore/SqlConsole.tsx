import { createResource, createSignal, For, Show, type Component } from "solid-js";
import { ResultsGrid } from "./ResultsGrid";
import { ExportBar } from "./ExportBar";
import { Spinner } from "../Spinner";
import { Disclosure } from "../Disclosure";
import { isLowEndDevice } from "../../lib/device";
import { createMediaQuery } from "../../lib/media";
import { controlCompact } from "../../lib/controls";

// Free-form SQL over the ingested typed tables. Every group of the loaded
// file is already a DuckDB table (ExplorePane ingests them all for the
// dashboard counts), so cross-group typed JOINs work directly — the flagship
// payoff of the typed pipeline. Snippets persist in localStorage; example
// chips are generated from the file's actual groups.

const SNIPPET_KEY = "ags-explore-sql-snippets";
// Cap how many result rows we MATERIALISE + render (export stays uncapped).
// Lower on weak hardware: a 30k-row grid is fine on a Mac, a freeze on a
// 2-core machine. The ResultsGrid shows a "first N of M" banner when capped.
const DISPLAY_CAP = isLowEndDevice() ? 500 : 2000;
interface Snippet {
  name: string;
  sql: string;
}

function loadSnippets(): Snippet[] {
  try {
    const arr = JSON.parse(localStorage.getItem(SNIPPET_KEY) ?? "[]");
    return Array.isArray(arr) ? arr : [];
  } catch {
    return [];
  }
}

export const SqlConsole: Component<{
  groups: string[];
  /** Controlled SQL text — shared with ExplorePane so the SqlBuilder can
   *  populate the editor ("Use this SQL"). */
  sql: () => string;
  setSql: (s: string) => void;
  /** Dictionary-derived relationship example queries (CHILD ⋈ PARENT joins +
   *  templates) for the loaded groups — shown as one-click example chips. */
  related?: { name: string; sql: string }[];
}> = (props) => {
  const sql = props.sql;
  const setSql = props.setSql;
  const [submitted, setSubmitted] = createSignal<{ sql: string; n: number } | null>(
    null,
  );
  const [snippets, setSnippets] = createSignal<Snippet[]>(loadSnippets());
  const [snipName, setSnipName] = createSignal("");
  let runN = 0;
  // Examples/Saved collapse to one line on a phone (where the chip rows
  // balloon), stay open on a wide screen. Reactive so they re-collapse when the
  // window is narrowed rather than staying stuck open.
  const wide = createMediaQuery("(min-width: 1024px)");

  const [result] = createResource(submitted, async (req) => {
    const { run } = await import("../../lib/duck");
    const { arrowResult } = await import("../../lib/arrowResult");
    return arrowResult(await run(req.sql), DISPLAY_CAP);
  });

  const runIt = () => {
    const s = sql().trim();
    if (s) setSubmitted({ sql: s, n: ++runN });
  };
  const onKeyDown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      runIt();
    }
  };

  const persist = (list: Snippet[]) => {
    setSnippets(list);
    try {
      localStorage.setItem(SNIPPET_KEY, JSON.stringify(list));
    } catch {
      /* quota / private mode — keep the in-memory list, skip persistence */
    }
  };
  const save = () => {
    const name = snipName().trim();
    if (!name) return;
    persist([...snippets().filter((s) => s.name !== name), { name, sql: sql() }]);
    setSnipName("");
  };
  const del = (name: string) =>
    persist(snippets().filter((s) => s.name !== name));

  const examples = (): Snippet[] => {
    const g = props.groups;
    const ex: Snippet[] = [];
    if (g[0])
      ex.push({ name: `${g[0]} rows`, sql: `SELECT * FROM "${g[0]}" LIMIT 100` });
    // Relationship examples derived from the dictionary (CHILD ⋈ PARENT joins
    // + templates) — replaces the old hard-coded SAMP⋈LOCA with whatever the
    // loaded file actually relates.
    for (const r of props.related ?? []) ex.push(r);
    ex.push({ name: "list tables", sql: "SHOW TABLES" });
    return ex;
  };

  return (
    <div class="flex min-w-0 flex-col gap-3">
      <Disclosure
        summary="Examples"
        count={examples().length}
        open={wide()}
        bodyClass="flex flex-wrap gap-1.5 text-xs"
      >
        <For each={examples()}>
          {(s) => (
            <button
              type="button"
              class="rounded border border-line-strong px-2 py-0.5 text-fg-soft hover:bg-chip"
              onClick={() => setSql(s.sql)}
            >
              {s.name}
            </button>
          )}
        </For>
      </Disclosure>
      <Show when={snippets().length > 0}>
        <Disclosure
          summary="Saved"
          count={snippets().length}
          open={wide()}
          bodyClass="flex flex-wrap gap-1.5 text-xs"
        >
          <For each={snippets()}>
            {(s) => (
              <span class="inline-flex items-center gap-1 rounded border border-line-strong px-2 py-0.5 text-xs">
                <button
                  type="button"
                  class="text-accent hover:underline"
                  onClick={() => setSql(s.sql)}
                >
                  {s.name}
                </button>
                <button
                  type="button"
                  class="text-fg-dim hover:text-err"
                  title="Delete snippet"
                  onClick={() => del(s.name)}
                >
                  ×
                </button>
              </span>
            )}
          </For>
        </Disclosure>
      </Show>

      <textarea
        class="mono h-32 w-full resize-y rounded-lg border border-line-strong bg-surface-raised p-3 text-xs text-fg outline-none"
        spellcheck={false}
        value={sql()}
        onInput={(e) => setSql(e.currentTarget.value)}
        onKeyDown={onKeyDown}
      />

      <div class="flex flex-wrap items-center gap-3 text-xs">
        <button
          type="button"
          class="rounded bg-surface-raised px-3 py-1.5 font-medium text-fg hover:bg-chip"
          onClick={runIt}
        >
          Run <span class="text-fg-muted">⌘/Ctrl+↵</span>
        </button>
        <input
          class={`w-32 ${controlCompact} placeholder:text-fg-dim`}
          placeholder="snippet name"
          value={snipName()}
          onInput={(e) => setSnipName(e.currentTarget.value)}
        />
        <button
          type="button"
          class="rounded border border-line-strong px-2 py-1 text-fg-soft hover:bg-chip disabled:opacity-40"
          disabled={!snipName().trim()}
          onClick={save}
        >
          Save
        </button>
        <Show when={submitted()}>
          <ExportBar sql={() => submitted()!.sql} filename="query" />
        </Show>
      </div>

      <Show when={result.loading}>
        <Spinner label="Running…" />
      </Show>
      <Show when={result.error}>
        <p class="text-sm text-err">SQL error: {String(result.error)}</p>
      </Show>
      {/* Guard the resource read: accessing `result()` while the resource is in
          its ERROR state RE-THROWS (a Solid resource gotcha) — uncaught, that
          left a query error unshown and the console stuck on a stale result.
          The `!result.error` guard short-circuits before the throwing read. */}
      <Show when={!result.error && result()}>
        {/* flowOnMobile: this grid stacks under the editor, so on a phone let
            it grow with the page rather than nest a 70dvh scroll inside one. */}
        {(r) => <ResultsGrid result={r()} flowOnMobile />}
      </Show>
    </div>
  );
};
