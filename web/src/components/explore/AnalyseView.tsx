import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import type { GroupMeta } from "../../lib/duckTypes";
import {
  referentialIntegrity,
  completeness,
  coverage,
  coverageTruncationNote,
  type DictKeyMap,
  type GroupCompleteness,
  type OrphanResult,
  type Coverage,
} from "../../lib/analytics";
import { typeDescription } from "../../lib/agsTypeInfo";
import { fetchUnion, isKeyStatus } from "../../lib/dict";
import { goTo } from "../../lib/nav";
import { Chevron } from "../Chevron";

// The Explore "Analyse" view: profiling that turns the validator into a data
// inspector. Reuses the already-ingested DuckDB tables (no re-parse) + the
// dictionary's parent/KEY metadata. Three cards: referential integrity
// (orphan finder), completeness (+ why each column is typed as it is), and a
// LOCA × group coverage matrix.

// The per-group slice of the union dictionary the orphan-finder needs: a
// parent + the headings (to derive KEY columns).
interface DictGroup {
  parent: string | null;
  headings: { name: string; status: string }[];
}

// Build the parent+KEY map from the canonical union ags_dictionary.json (the
// single web dict source — see lib/dict.ts). KEY detection is `+`-aware so
// combined statuses like "KEY+REQUIRED" still count.
async function loadKeyMap(): Promise<DictKeyMap> {
  const raw = await fetchUnion();
  const m: DictKeyMap = new Map();
  for (const [code, g] of Object.entries(raw.groups)) {
    const grp: DictGroup = g;
    m.set(code, {
      parent: grp.parent,
      keys: grp.headings
        .filter((h) => isKeyStatus(h.status))
        .map((h) => h.name),
    });
  }
  return m;
}

export const AnalyseView: Component<{ groups: GroupMeta[] }> = (props) => {
  const [dict] = createResource(loadKeyMap);

  const [analysis] = createResource(
    () => {
      const d = dict();
      return d && props.groups.length ? { d, metas: props.groups } : null;
    },
    async ({ d, metas }) => {
      // One shared DuckDB connection → run sequentially, not Promise.all.
      const { run } = await import("../../lib/duck");
      const ri = await referentialIntegrity(metas, d, run);
      const comp = await completeness(metas, run);
      const cov = await coverage(metas, run);
      return { ri, comp, cov };
    },
  );

  return (
    <Show
      when={!analysis.loading}
      fallback={<p class="text-sm text-fg-muted">Analysing…</p>}
    >
      <Show
        when={!analysis.error}
        fallback={
          <p class="text-sm text-err">
            Analyse error: {String(analysis.error)}
          </p>
        }
      >
        <Show when={analysis()}>
          {(a) => (
            <div class="flex min-w-0 flex-col gap-4">
              <RICard links={a().ri.links} orphans={a().ri.orphans} />
              <CompletenessCard groups={a().comp} />
              <Show when={a().cov}>{(c) => <CoverageCard cov={c()} />}</Show>
            </div>
          )}
        </Show>
      </Show>
    </Show>
  );
};

// --- referential integrity ---

const RICard: Component<{ links: number; orphans: OrphanResult[] }> = (
  props,
) => (
  <section class="rounded-lg border border-line bg-surface p-3">
    <h3 class="text-sm font-semibold text-fg">Referential integrity</h3>
    <p class="mt-0.5 text-xs text-fg-dim">
      Child rows whose KEY values match no parent row (checked across{" "}
      {props.links} dictionary parent–child link
      {props.links === 1 ? "" : "s"}).
    </p>
    <Show
      when={props.orphans.length > 0}
      fallback={
        <p class="mt-2 text-sm text-ok">
          ✓ All referential links intact — every child row resolves to a parent.
        </p>
      }
    >
      <div class="mt-2 flex flex-col gap-2">
        <For each={props.orphans}>
          {(o) => (
            <div class="rounded-sm border border-err/45 bg-err-quiet px-3 py-2">
              <div class="flex flex-wrap items-baseline gap-2 text-sm">
                <span class="mono font-medium text-fg">
                  {o.child} → {o.parent}
                </span>
                <span class="text-err">
                  {o.orphans} of {o.total} rows orphaned
                </span>
                <span class="text-xs text-fg-dim">on {o.keys.join(" + ")}</span>
              </div>
              <Show when={o.samples.length > 0}>
                <div class="mono mt-1 flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-fg-muted">
                  <For each={o.samples}>
                    {(s) => <span>{s.join(" | ")}</span>}
                  </For>
                </div>
              </Show>
            </div>
          )}
        </For>
        <button
          type="button"
          class="self-start text-xs text-accent underline-offset-2 hover:underline"
          onClick={() => {
            goTo("validate");
          }}
        >
          Open Validate (Rules 10c/14 flag these too) →
        </button>
      </div>
    </Show>
  </section>
);

// --- completeness (+ why typed) ---

function fillClass(pct: number): string {
  if (pct === 0) return "bg-err-quiet text-err";
  if (pct < 0.5) return "bg-warn-quiet text-warn";
  if (pct < 1) return "bg-info-quiet text-info";
  return "bg-ok-quiet text-ok";
}

const CompletenessCard: Component<{ groups: GroupCompleteness[] }> = (
  props,
) => {
  const [open, setOpen] = createSignal<Set<string>>(new Set());
  const toggle = (code: string) =>
    setOpen((prev) => {
      const next = new Set(prev);
      if (next.has(code)) next.delete(code);
      else next.add(code);
      return next;
    });
  return (
    <section class="rounded-lg border border-line bg-surface p-3">
      <h3 class="text-sm font-semibold text-fg">Completeness</h3>
      <p class="mt-0.5 text-xs text-fg-dim">
        How fully each column is populated. Expand a group for per-column fill +
        why it's typed as it is.
      </p>
      <div class="mt-2 flex flex-col gap-1">
        <For each={props.groups}>
          {(g) => (
            <div class="rounded-sm border border-line-subtle">
              <button
                type="button"
                class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm hover:bg-surface-raised"
                onClick={() => toggle(g.code)}
              >
                <Chevron open={open().has(g.code)} />
                <span class="mono font-medium text-fg">{g.code}</span>
                <span class="text-xs text-fg-dim">{g.total} rows</span>
                <span
                  class={`ml-auto rounded-xs px-1.5 py-0.5 text-xs ${fillClass(g.overall)}`}
                >
                  {Math.round(g.overall * 100)}% filled
                </span>
                <Show when={g.emptyCols.length > 0}>
                  <span class="rounded-xs bg-err-quiet px-1.5 py-0.5 text-xs text-err">
                    {g.emptyCols.length} empty col
                    {g.emptyCols.length === 1 ? "" : "s"}
                  </span>
                </Show>
              </button>
              <Show when={open().has(g.code)}>
                <div class="overflow-x-auto border-t border-line-subtle">
                  <table class="min-w-full text-xs">
                    <thead class="bg-surface-raised text-fg-soft">
                      <tr>
                        <th class="px-3 py-1 text-left font-medium">Heading</th>
                        <th class="px-3 py-1 text-left font-medium">
                          AGS type
                        </th>
                        <th class="px-3 py-1 text-left font-medium">
                          Stored as
                        </th>
                        <th class="px-3 py-1 text-right font-medium">Filled</th>
                      </tr>
                    </thead>
                    <tbody class="mono">
                      <For each={g.cols}>
                        {(c) => (
                          <tr class="border-t border-line-subtle">
                            <td class="px-3 py-1 text-fg-soft">{c.heading}</td>
                            <td class="px-3 py-1 text-fg-muted">
                              {c.type || "—"}
                              <span class="ml-1 text-fg-dim">
                                ({typeDescription(c.type)})
                              </span>
                            </td>
                            <td class="px-3 py-1 text-fg-dim">
                              {c.sqlType.toLowerCase()}
                            </td>
                            <td
                              class={`px-3 py-1 text-right ${c.pct === 0 ? "text-err" : "text-fg-soft"}`}
                            >
                              {Math.round(c.pct * 100)}%
                            </td>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </div>
              </Show>
            </div>
          )}
        </For>
      </div>
    </section>
  );
};

// --- coverage matrix ---

const CoverageCard: Component<{ cov: Coverage }> = (props) => {
  const missing = createMemo(() => {
    // count of (loca, group) cells that are absent — the gaps.
    let n = 0;
    for (const id of props.cov.locas)
      for (const g of props.cov.groups) if (!props.cov.present[g]?.has(id)) n++;
    return n;
  });
  return (
    <section class="rounded-lg border border-line bg-surface p-3">
      <h3 class="text-sm font-semibold text-fg">Coverage (LOCA × group)</h3>
      <p class="mt-0.5 text-xs text-fg-dim">
        Which boreholes appear in which groups — {missing()} gap
        {missing() === 1 ? "" : "s"} (a borehole with no rows in that group).
      </p>
      <div class="scroll-region mt-2 rounded-sm border border-line-subtle">
        <table class="min-w-full text-xs">
          {/* Two-axis frozen header/column: stack order corner (z-30) > header
              row + pinned column (z-20/z-10) > data cells, all opaque so nothing
              bleeds through on scroll. */}
          <thead class="sticky top-0 z-20 bg-surface-raised text-fg-soft">
            <tr>
              <th class="sticky left-0 z-30 bg-surface-raised px-2 py-1 text-left font-medium">
                LOCA_ID
              </th>
              <For each={props.cov.groups}>
                {(g) => (
                  <th class="mono px-2 py-1 text-center font-medium">{g}</th>
                )}
              </For>
            </tr>
          </thead>
          <tbody class="mono">
            <For each={props.cov.locas}>
              {(id) => (
                <tr class="border-t border-line-subtle">
                  <td class="sticky left-0 z-10 bg-surface-raised px-2 py-1 text-fg-soft">
                    {id}
                  </td>
                  <For each={props.cov.groups}>
                    {(g) => {
                      const here = props.cov.present[g]?.has(id);
                      return (
                        <td
                          class="px-2 py-1 text-center"
                          classList={{
                            "bg-ok-quiet text-ok": here,
                            "text-fg-dim": !here,
                          }}
                        >
                          {here ? "✓" : "·"}
                        </td>
                      );
                    }}
                  </For>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
      <Show when={props.cov.truncated}>
        <p class="mt-1 text-xs text-fg-dim italic">
          {coverageTruncationNote(props.cov)}
        </p>
      </Show>
    </section>
  );
};
