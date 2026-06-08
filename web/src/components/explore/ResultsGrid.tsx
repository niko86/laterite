import { For, Show, type Component } from "solid-js";
import type { ResultSet } from "../../lib/arrowResult";

// Shared read-only grid for a query result (already formatted by
// arrowResult/formatCell). Used by the table browser's paged SELECT * and by
// the SQL console (which caps how many rows it materialises) — so the DOM stays
// bounded; a banner notes when the display was truncated.
export const ResultsGrid: Component<{
  result: ResultSet;
  maxHeight?: string;
  /** When set, the vertical cap applies only from md up — on a phone the grid
   *  flows with the page (avoids a scroll-within-scroll where it stacks under
   *  the SQL editor). Ignored when an explicit `maxHeight` is given. */
  flowOnMobile?: boolean;
}> = (props) => (
  <Show
    when={props.result.columns.length > 0}
    fallback={
      <p class="rounded-lg border border-line bg-surface px-3 py-2 text-sm text-fg-muted">
        No columns returned.
      </p>
    }
  >
    <div class="flex min-w-0 flex-col gap-2">
      <Show when={props.result.total > props.result.rows.length}>
        <p class="text-xs text-fg-faint">
          Showing the first {props.result.rows.length.toLocaleString()} of{" "}
          {props.result.total.toLocaleString()} rows — add a LIMIT to narrow it,
          or Export for the full result.
        </p>
      </Show>
      <div
        class={`rounded-lg border border-line ${props.maxHeight ? "overflow-auto" : props.flowOnMobile ? "scroll-region-soft" : "scroll-region"}`}
        style={props.maxHeight ? { "max-height": props.maxHeight } : undefined}
      >
      <table class="min-w-full text-xs">
        <thead class="sticky top-0 z-10 bg-surface-raised text-fg-soft [&_th]:border-b [&_th]:border-line">
          <tr>
            <For each={props.result.columns}>
              {(c) => (
                <th class="whitespace-nowrap px-3 py-1.5 text-left font-medium">
                  {c.name}
                  <span class="ml-1 font-normal text-fg-dim">
                    {c.sqlType.toLowerCase()}
                  </span>
                </th>
              )}
            </For>
          </tr>
        </thead>
        <tbody class="mono">
          <For each={props.result.rows}>
            {(row) => (
              <tr class="border-t border-line-subtle hover:bg-surface-raised">
                <For each={row}>
                  {(cell) => (
                    <td class="whitespace-nowrap px-3 py-1 text-fg-soft">
                      {cell || "—"}
                    </td>
                  )}
                </For>
              </tr>
            )}
          </For>
        </tbody>
      </table>
      </div>
    </div>
  </Show>
);
