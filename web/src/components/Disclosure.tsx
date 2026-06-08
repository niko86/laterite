import { Show, type JSX, type ParentComponent } from "solid-js";
import { Chevron } from "./Chevron";

// A collapsible panel — the app's established `<details>` idiom (SqlBuilder,
// DataTable, DictionaryBrowser) promoted to one component so chip rows and
// example lists that used to sit always-expanded (eating vertical space on
// every screen) collapse to a single summary line by default. Native
// `<details>` keeps it zero-JS, keyboard-accessible, and identical on desktop
// and mobile. An optional `count` shows an active/total badge in the summary.
export const Disclosure: ParentComponent<{
  summary: JSX.Element;
  /** Open on first render (e.g. samples when no file is loaded yet). */
  open?: boolean;
  /** Optional badge in the summary (active filters, number of examples). */
  count?: number;
  /** Extra classes on the body wrapper. */
  bodyClass?: string;
}> = (props) => (
  <details class="group rounded-lg border border-line bg-surface" open={props.open}>
    <summary class="flex cursor-pointer list-none select-none items-center gap-2 px-3 py-2 text-sm font-medium text-fg-soft [&::-webkit-details-marker]:hidden">
      <Chevron />
      <span class="min-w-0">{props.summary}</span>
      <Show when={props.count !== undefined && props.count > 0}>
        <span class="ml-auto rounded-full bg-chip px-1.5 text-[10px] text-fg-soft">
          {props.count}
        </span>
      </Show>
    </summary>
    <div class={`border-t border-line-subtle p-3 ${props.bodyClass ?? ""}`}>
      {props.children}
    </div>
  </details>
);
