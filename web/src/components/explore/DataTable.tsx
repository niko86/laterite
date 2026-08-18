import { Button } from "@shared/components";
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from "solid-js";
import type { GroupMeta } from "../../lib/duckTypes";
import { ResultsGrid } from "./ResultsGrid";
import { Chevron } from "../Chevron";

// Rows per page. Paging keeps the DOM bounded (≤ PAGE rows mounted), so the
// grid needs no row virtualization — a group with 50k rows browses one page
// at a time. SELECT … LIMIT/OFFSET keeps DuckDB's work bounded too.
const PAGE = 100;

/** Schema panel (straight from meta(), no DuckDB round-trip) + a paged
 *  SELECT * grid for one ingested group. */
export const DataTable: Component<{
  code: string;
  rows: number;
  meta: GroupMeta;
}> = (props) => {
  const [page, setPage] = createSignal(0);
  // Reset to the first page whenever the selected group changes.
  createEffect(
    on(
      () => props.code,
      () => setPage(0),
    ),
  );

  const [result] = createResource(
    () => ({ code: props.code, page: page() }),
    async (key) => {
      const { run } = await import("../../lib/duck");
      const { arrowResult } = await import("../../lib/arrowResult");
      const t = await run(
        `SELECT * FROM "${key.code}" LIMIT ${PAGE} OFFSET ${key.page * PAGE}`,
      );
      // Hide the synthetic _id/_parent_id key columns from the group grid (they
      // stay in the engine for the SQL console's cross-group joins). (#303)
      return arrowResult(t, undefined, true);
    },
  );

  const pageCount = createMemo(() => Math.max(1, Math.ceil(props.rows / PAGE)));

  return (
    <div class="flex min-w-0 flex-col gap-3">
      <details class="group rounded-lg border border-line bg-surface">
        <summary class="flex cursor-pointer list-none select-none items-center gap-2 px-3 py-2 text-sm font-medium text-fg-soft [&::-webkit-details-marker]:hidden">
          <Chevron />
          <span class="min-w-0">
            {props.code} schema — {props.meta.headings.length} columns
          </span>
        </summary>
        <div class="overflow-x-auto border-t border-line-subtle">
          <table class="w-full text-xs">
            <thead class="bg-surface-raised text-fg-muted">
              <tr>
                <th class="px-3 py-1 text-left font-medium">Heading</th>
                <th class="px-3 py-1 text-left font-medium">Unit</th>
                <th class="px-3 py-1 text-left font-medium">AGS type</th>
                <th class="px-3 py-1 text-left font-medium">SQL type</th>
              </tr>
            </thead>
            <tbody>
              <For each={props.meta.headings}>
                {(h, i) => (
                  <tr class="border-t border-line-subtle">
                    <td class="mono px-3 py-1 text-fg">{h}</td>
                    <td class="px-3 py-1 text-fg-faint">
                      {props.meta.units[i()] || "—"}
                    </td>
                    <td class="px-3 py-1 text-fg-soft">
                      {props.meta.types[i()] || "—"}
                    </td>
                    <td class="mono px-3 py-1 text-accent">
                      {props.meta.sql_types[i()] || "VARCHAR"}
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </details>

      <Show
        when={!result.loading}
        fallback={<p class="text-sm text-fg-muted">Querying…</p>}
      >
        <Show
          when={!result.error}
          fallback={
            <p class="text-sm text-err">Query error: {String(result.error)}</p>
          }
        >
          <Show when={result()}>{(r) => <ResultsGrid result={r()} />}</Show>
        </Show>
      </Show>

      <Show when={pageCount() > 1}>
        <div class="flex items-center gap-3 text-xs text-fg-muted">
          <Button
            size="sm"
            disabled={page() === 0}
            onClick={() => setPage((p) => Math.max(0, p - 1))}
          >
            ← Prev
          </Button>
          <span>
            Page {page() + 1} of {pageCount()} · {props.rows} rows
          </span>
          <Button
            size="sm"
            disabled={page() >= pageCount() - 1}
            onClick={() => setPage((p) => Math.min(pageCount() - 1, p + 1))}
          >
            Next →
          </Button>
        </div>
      </Show>
    </div>
  );
};
