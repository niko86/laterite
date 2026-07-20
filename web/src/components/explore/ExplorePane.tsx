import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { fileStore } from "../../lib/fileStore";
import { parseDataset, arrowIpc } from "../../lib/validatorClient";
import type { GroupMeta } from "../../lib/duckTypes";
import { DataTable } from "./DataTable";
import { SqlConsole } from "./SqlConsole";
import { SqlBuilder } from "./SqlBuilder";
import { ChartBuilder } from "./ChartBuilder";
import { AnalyseView } from "./AnalyseView";
import { PillToggle } from "../PillToggle";
import {
  exploreView as view,
  setExploreView as setView,
} from "../../lib/settings";
import { goTo } from "../../lib/nav";
import { EngineGate, engineGateNeeded } from "./EngineGate";
import { Spinner } from "../Spinner";
import { Card } from "../Card";
import { loadDict, relExamples } from "../../lib/relationships";
import { controlClass, controlCompact } from "../../lib/controls";

interface GroupInfo {
  meta: GroupMeta;
  rows: number;
}

// The Explore tab: parse the loaded file into typed DuckDB tables (Rust wasm
// → Arrow IPC → DuckDB-wasm), then browse them. A left sidebar lists the
// groups (with row counts); the main panel shows either the per-file
// dashboard (the landing view) or, for a selected group, its schema + a
// paged data grid. Everything is client-side; nothing is uploaded.
export const ExplorePane: Component = () => {
  const [selected, setSelected] = createSignal<string | null>(null);
  // Free-text filter for the (now capped + scrollable) group sidebar, so a
  // 69-group file is a quick type-to-find instead of a 2000px scroll.
  const [groupFilter, setGroupFilter] = createSignal("");
  // SQL editor text, owned here so the visual SqlBuilder can populate the
  // console ("Use this SQL"). Seeded on first dataset arrival.
  const [sqlText, setSqlText] = createSignal("");
  // The AGS dictionary (parent + KEY metadata) — shared with the SQL/chart
  // builders so they can offer relationship-aware joins without hand-written
  // SQL. Cheap: a fetch of the (cached) union ags_dictionary.json.
  const [dict] = createResource(loadDict);

  // Overlapping-load guard: createResource does NOT cancel a superseded
  // fetcher, so a slow load of file A could keep pulling/ingesting (and
  // resetDuck/markLoaded) the worker-held dataset while a newer load of file B
  // is already underway — ingesting a table from the wrong file. A monotonic
  // token makes a stale run bail at its next await (its return value is
  // discarded by Solid anyway; this stops its side effects).
  let loadSeq = 0;

  // Cold-engine gate: on a low-end device, don't kick off the 36 MB DuckDB
  // download/compile until the user confirms (EngineGate). Capable devices and
  // repeat visits auto-proceed. The resource's source is gated on `proceed`, so
  // no heavy engine work starts while the dialog is up.
  const [proceed, setProceed] = createSignal(false);
  createEffect(() => {
    if (fileStore.bytes() && !proceed() && !engineGateNeeded())
      setProceed(true);
  });
  // Staged bring-up text, so a slow load shows progress instead of a frozen tab.
  const [stage, setStage] = createSignal("Starting the data engine…");

  const [dataset] = createResource(
    () => (proceed() ? fileStore.bytes() : undefined),
    async (bytes) => {
      if (bytes.length === 0) return null;
      const seq = ++loadSeq;
      const stale = () => seq !== loadSeq;
      const {
        getDuckDb,
        ingestGroup,
        run,
        resetDuck,
        isLoaded,
        markLoaded,
        getLoadedGroups,
      } = await import("../../lib/duck");
      if (stale()) return null;
      // Same file as last time (a tab switch re-mounts this pane) → return the
      // cached group info untouched: NO wasm re-parse, NO per-group count(*)
      // re-run. On a big file that re-derivation was multi-second pure waste.
      if (isLoaded(bytes)) {
        const cachedGroups = getLoadedGroups() as GroupInfo[] | null;
        if (cachedGroups) return cachedGroups;
      }
      // A new file: reset, ingest each group, count, and cache the result.
      await resetDuck();
      if (stale()) return null;
      setSelected(null); // a NEW file lands on the dashboard
      setStage("Starting the data engine…");
      await getDuckDb();
      if (stale()) return null;
      setStage("Parsing your file…");
      const groups = await parseDataset(bytes, "utf-8");
      if (stale()) return null;
      const out: GroupInfo[] = [];
      for (const g of groups) {
        setStage(
          `Loading tables… ${out.length + 1}/${groups.length} (${g.code})`,
        );
        // keys=true: ingest the content-addressed _id/_parent_id columns into
        // duckdb-wasm so the SQL console's cross-group joins resolve; the group
        // grid (DataTable) strips them from display. (#303) contentHash=true:
        // same deal for the trailing _content_hash value fingerprint, enabling
        // `SELECT DISTINCT ON (_content_hash)` in the SQL console; arrowResult's
        // `dropSynthKeys` strips every `_`-prefixed column (not just the two key
        // columns), so it's hidden from the grid the same way. (#448)
        const ipc = await arrowIpc(g.code, true, true);
        if (stale()) return null;
        await ingestGroup(g.code, ipc);
        if (stale()) return null;
        const t = await run(`SELECT count(*) AS n FROM "${g.code}"`);
        if (stale()) return null;
        // Single-row `SELECT count(*) AS n` result; DuckDB returns the count as
        // a bigint, so the cast names the shape we asked for.
        const countRow = t.toArray()[0] as { n: number | bigint };
        out.push({ meta: g, rows: Number(countRow.n) });
      }
      if (stale()) return null;
      markLoaded(bytes, out);
      return out;
    },
  );

  // Seed the SQL editor once the first dataset lands (the console is now
  // controlled by ExplorePane, so the default query is set here).
  createEffect(() => {
    const first = dataset()?.[0];
    if (first && !sqlText())
      setSqlText(`SELECT * FROM "${first.meta.code}" LIMIT 100`);
  });

  const totalRows = () => dataset()?.reduce((n, g) => n + g.rows, 0) ?? 0;
  const selectedInfo = () =>
    dataset()?.find((g) => g.meta.code === selected()) ?? null;
  const filteredGroups = createMemo(() => {
    const f = groupFilter().trim().toUpperCase();
    const ds = dataset() ?? [];
    if (!f) return ds;
    // Keep the currently-open group in the list even if it doesn't match, so the
    // active highlight doesn't vanish while its table is still on screen.
    const sel = selected();
    return ds.filter(
      (g) => g.meta.code.toUpperCase().includes(f) || g.meta.code === sel,
    );
  });

  // Dictionary-derived relationship example queries for the loaded groups
  // (CHILD ⋈ PARENT joins), shown as one-click chips in the SQL console.
  const relatedExamples = createMemo(() => {
    const d = dict();
    const ds = dataset();
    return d && ds
      ? relExamples(
          ds.map((g) => ({ code: g.meta.code, headings: g.meta.headings })),
          d,
        )
      : [];
  });

  return (
    <Show
      when={fileStore.bytes()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Data explorer</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab — it's parsed into typed,
            in-browser DuckDB tables you can browse here. Nothing is uploaded.
          </p>
          <button
            type="button"
            class="mt-4 rounded bg-accent/15 px-3 py-1.5 text-sm font-medium text-accent hover:bg-accent/25"
            onClick={() => {
              goTo("validate");
            }}
          >
            Go to Validate to load a file →
          </button>
        </div>
      }
    >
      <Show
        when={proceed()}
        fallback={<EngineGate onConfirm={() => setProceed(true)} />}
      >
        <Show
          when={!dataset.loading}
          fallback={
            <div class="py-2">
              <Spinner label={stage()} />
            </div>
          }
        >
          <Show
            when={!dataset.error}
            fallback={
              <p class="text-sm text-err">
                {/* The DuckDB engine wasm is NOT precached (it's 36+ MB), so a
                  FIRST Explore while offline can't fetch it — degrade to a
                  clear message rather than a raw "Failed to fetch". Validate /
                  Fix are unaffected (their wasm is precached). */}
                {!navigator.onLine
                  ? "The data engine isn't cached for offline use yet — open Explore once while online, and it'll work offline after. (Validate & Fix already work offline.)"
                  : `Explore error: ${String(dataset.error)}`}
              </p>
            }
          >
            <div class="flex min-w-0 flex-col gap-4">
              {/* Browse | SQL view toggle */}
              <div class="flex items-center gap-1 text-sm">
                <PillToggle
                  label="Browse"
                  active={view() === "browse"}
                  onClick={() => {
                    setView("browse");
                  }}
                />
                <PillToggle
                  label="SQL"
                  active={view() === "sql"}
                  onClick={() => {
                    setView("sql");
                  }}
                />
                <PillToggle
                  label="Charts"
                  active={view() === "charts"}
                  onClick={() => {
                    setView("charts");
                  }}
                />
                <PillToggle
                  label="Analyse"
                  active={view() === "analyse"}
                  onClick={() => {
                    setView("analyse");
                  }}
                />
              </div>
              <Show when={view() === "browse"}>
                {/* items-start so the capped sidebar sizes to its own content
                  instead of stretching to the (often taller) data panel. The
                  sidebar appears from sm+ so small tablets / landscape phones
                  get it too; below sm the main panel uses a group dropdown. */}
                <div class="grid items-start gap-4 sm:grid-cols-[13rem_minmax(0,1fr)]">
                  {/* Group sidebar — capped + scrollable + type-to-filter. Hidden
                    on a phone (the main panel gets a compact group dropdown
                    instead, so you don't scroll past 69 buttons to reach data). */}
                  <aside class="scroll-region hidden min-w-0 flex-col text-sm sm:flex">
                    {/* Opaque (canvas-coloured) sticky band so group buttons scroll
                  cleanly UNDER the rounded filter input — a bare sticky input
                  let a sliver of each row show through its gap + rounded corners. */}
                    <div class="sticky top-0 z-10 bg-canvas pb-1">
                      <input
                        type="search"
                        aria-label="filter groups"
                        placeholder="Filter groups…"
                        value={groupFilter()}
                        onInput={(e) => setGroupFilter(e.currentTarget.value)}
                        class={`w-full ${controlCompact}`}
                      />
                    </div>
                    <div class="flex flex-col gap-1">
                      <SidebarButton
                        label="Overview"
                        active={selected() === null}
                        onClick={() => setSelected(null)}
                      />
                      <For each={filteredGroups()}>
                        {(g) => (
                          <SidebarButton
                            label={g.meta.code}
                            count={g.rows}
                            active={selected() === g.meta.code}
                            onClick={() => setSelected(g.meta.code)}
                          />
                        )}
                      </For>
                    </div>
                  </aside>

                  {/* Main panel */}
                  <section class="min-w-0">
                    {/* Mobile group picker (the sidebar is hidden below md). */}
                    <label class="mb-3 flex flex-col gap-1 text-xs text-fg-muted sm:hidden">
                      Group
                      <select
                        aria-label="group"
                        class={controlClass}
                        value={selected() ?? ""}
                        onChange={(e) =>
                          setSelected(e.currentTarget.value || null)
                        }
                      >
                        <option value="">Overview</option>
                        <For each={dataset()}>
                          {(g) => (
                            <option value={g.meta.code}>
                              {g.meta.code} ({g.rows})
                            </option>
                          )}
                        </For>
                      </select>
                    </label>
                    <Show
                      when={selectedInfo()}
                      fallback={
                        <Dashboard
                          groups={dataset() ?? []}
                          totalRows={totalRows()}
                          onPick={setSelected}
                        />
                      }
                    >
                      {(info) => (
                        <DataTable
                          code={info().meta.code}
                          rows={info().rows}
                          meta={info().meta}
                        />
                      )}
                    </Show>
                  </section>
                </div>
              </Show>
              <Show when={view() === "sql"}>
                <div class="flex min-w-0 flex-col gap-3">
                  <SqlBuilder
                    groups={(dataset() ?? []).map((g) => g.meta)}
                    dict={dict()}
                    onApply={setSqlText}
                  />
                  <SqlConsole
                    groups={(dataset() ?? []).map((g) => g.meta.code)}
                    sql={sqlText}
                    setSql={setSqlText}
                    related={relatedExamples()}
                  />
                </div>
              </Show>
              <Show when={view() === "charts"}>
                <ChartBuilder
                  groups={(dataset() ?? []).map((g) => g.meta)}
                  dict={dict()}
                />
              </Show>
              <Show when={view() === "analyse"}>
                <AnalyseView groups={(dataset() ?? []).map((g) => g.meta)} />
              </Show>
            </div>
          </Show>
        </Show>
      </Show>
    </Show>
  );
};

const SidebarButton: Component<{
  label: string;
  count?: number;
  active: boolean;
  onClick: () => void;
}> = (props) => (
  <button
    type="button"
    onClick={() => {
      props.onClick();
    }}
    class="flex items-center justify-between rounded px-2.5 py-1 text-left transition-colors"
    classList={{
      "bg-chip text-accent font-medium": props.active,
      "text-fg-soft hover:bg-surface-raised": !props.active,
    }}
  >
    <span class="mono truncate">{props.label}</span>
    <Show when={props.count !== undefined}>
      <span class="ml-2 text-xs text-fg-dim">{props.count}</span>
    </Show>
  </button>
);

const Dashboard: Component<{
  groups: GroupInfo[];
  totalRows: number;
  onPick: (code: string) => void;
}> = (props) => (
  <div class="flex min-w-0 flex-col gap-4">
    <Card class="flex flex-wrap items-center gap-x-8 gap-y-2">
      <div>
        <div class="text-2xl font-semibold text-fg">{props.groups.length}</div>
        <div class="text-fg-muted">groups</div>
      </div>
      <div>
        <div class="text-2xl font-semibold text-fg">{props.totalRows}</div>
        <div class="text-fg-muted">data rows</div>
      </div>
      <p class="ml-auto max-w-xs text-xs text-fg-faint">
        Overview of the parsed file — pick a group (from the list or the table
        below) to see its rows.
      </p>
    </Card>
    <div class="overflow-x-auto rounded-lg border border-line">
      <table class="w-full text-sm">
        <thead class="bg-surface-raised text-fg-soft">
          <tr>
            <th class="px-3 py-1.5 text-left font-medium">Group</th>
            <th class="px-3 py-1.5 text-right font-medium">Rows</th>
            <th class="px-3 py-1.5 text-right font-medium">Columns</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.groups}>
            {(g) => (
              <tr
                class="cursor-pointer border-t border-line-subtle hover:bg-surface-raised"
                onClick={() => {
                  props.onPick(g.meta.code);
                }}
              >
                <td class="mono px-3 py-1 text-accent">{g.meta.code}</td>
                <td class="px-3 py-1 text-right text-fg-soft">{g.rows}</td>
                <td class="px-3 py-1 text-right text-fg-soft">
                  {g.meta.headings.length}
                </td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  </div>
);
