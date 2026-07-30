import { For, Show, createMemo, type Component } from "solid-js";
import type { Severity, ValidationReport } from "../../lib/validator";
import { severityOf } from "../../lib/validator";
import { shortRule } from "../../lib/rules";
import { Disclosure } from "../Disclosure";
import { createMediaQuery } from "../../lib/media";

// Re-exported so the many components that import `Severity` from here keep
// working. The definition lives with `severityOf` in lib/validator, because the
// resolved union and the resolver have to agree.
export type { Severity } from "../../lib/validator";

/** Filter state lives in ParentPane and is threaded in/out here, mirroring
 *  how `aligned` is threaded. A muted rule/severity/group is one NOT present
 *  in its selected-set (so "everything on" is the empty-exclusion default we
 *  initialise to in ValidatePane). Counts shown are TOTALS over the whole
 *  report (the simplest-correct v1 — they don't re-narrow as other filters
 *  change), so a chip always advertises how many findings it would surface
 *  in isolation. */
export const FilterBar: Component<{
  report: ValidationReport;
  // Rule filter: a rule is shown when its key is in `selectedRules`.
  selectedRules: () => Set<string>;
  onSelectedRules: (s: Set<string>) => void;
  // Severity filter (default: error+warning on, fyi off).
  selectedSeverities: () => Set<Severity>;
  onSelectedSeverities: (s: Set<Severity>) => void;
  // Group filter (default: all distinct groups on).
  selectedGroups: () => Set<string>;
  onSelectedGroups: (s: Set<string>) => void;
  // Free-text (already raw; debouncing happens here on input).
  search: () => string;
  onSearch: (v: string) => void;
  // Showing N of M after all active filters.
  shownCount: () => number;
  totalCount: () => number;
  // Jump to a rule group (scroll + force-open), provided by FindingsView.
  onJump: (rule: string) => void;
}> = (props) => {
  // Per-rule total counts, in report order.
  const rules = createMemo(() =>
    props.report.findings.map((g) => ({
      rule: g.rule,
      count: g.items.length,
    })),
  );

  // Distinct severities present, in a stable error→warning→fyi order, with
  // total counts. Only render chips for severities that actually occur.
  const SEV_ORDER: Severity[] = ["error", "warning", "fyi"];
  const severities = createMemo(() => {
    const counts = new Map<Severity, number>();
    for (const g of props.report.findings) {
      for (const f of g.items) {
        const s = severityOf(f);
        counts.set(s, (counts.get(s) ?? 0) + 1);
      }
    }
    return SEV_ORDER.map((s) => ({ sev: s, count: counts.get(s) })).filter(
      (e): e is { sev: Severity; count: number } => e.count !== undefined,
    );
  });

  // Distinct groups present (sorted), with total counts.
  const groups = createMemo(() => {
    const counts = new Map<string, number>();
    for (const g of props.report.findings) {
      for (const f of g.items) {
        const k = f.group || "—";
        counts.set(k, (counts.get(k) ?? 0) + 1);
      }
    }
    return [...counts.entries()]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([group, count]) => ({ group, count }));
  });

  const toggle = <T,>(set: Set<T>, key: T): Set<T> => {
    const next = new Set(set);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    return next;
  };

  // Severity chip colours, consistent with severityBand (rose/amber/sky).
  const sevActiveClass = (s: Severity): string => {
    switch (s) {
      case "error":
        return "border-err/60 bg-err/15 text-err";
      case "warning":
        return "border-warn/60 bg-warn/15 text-warn";
      case "fyi":
        return "border-accent/60 bg-accent/15 text-accent";
    }
  };

  // Debounce the free-text input (~180ms) so typing doesn't refilter on
  // every keystroke. The input is uncontrolled-ish: we push debounced — but
  // a CLEARED box fires immediately, so deleting the query repopulates the
  // full list at once (no debounce window where the old narrow result lingers
  // and reads as "shows nothing").
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  const onSearchInput = (v: string) => {
    clearTimeout(searchTimer);
    if (v === "") {
      props.onSearch("");
      return;
    }
    searchTimer = setTimeout(() => {
      props.onSearch(v);
    }, 180);
  };

  const chipBase =
    "rounded-full border px-2.5 py-1 text-xs transition-colors cursor-pointer select-none";
  const chipOff = "border-line bg-chip text-fg-faint hover:border-line-strong";
  const countBadge =
    "ml-1.5 rounded-full bg-chip px-1.5 text-[10px] text-fg-soft";

  // The whole filter bar collapses to one line on a phone (where the chip rows
  // stack 200–300px tall) but stays open on a wide screen. Reactive: re-asserts
  // the breakpoint default when the window is resized / the device rotated, so
  // it can't get stuck open on a narrowed viewport.
  const wide = createMediaQuery("(min-width: 1024px)");
  // How many filter dimensions are currently narrowing the view — shown as a
  // badge so an active filter is visible even when the bar is collapsed.
  const activeFilters = createMemo(() => {
    let n = 0;
    if (props.selectedRules().size < rules().length) n++;
    if (props.selectedSeverities().size < severities().length) n++;
    if (props.selectedGroups().size < groups().length) n++;
    if (props.search().trim() !== "") n++;
    return n;
  });

  return (
    <Disclosure
      summary="Filters"
      open={wide()}
      count={activeFilters()}
      bodyClass="flex flex-col gap-3"
    >
      {/* Rule toggles + jump. Active = shown. */}
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-xs font-medium text-fg-muted">Rules</span>
        <For each={rules()}>
          {(r) => {
            const active = () => props.selectedRules().has(r.rule);
            return (
              <span
                class={chipBase}
                classList={{
                  "border-accent bg-accent/15 text-accent": active(),
                  [chipOff]: !active(),
                }}
                title={r.rule}
              >
                <button
                  type="button"
                  class="cursor-pointer"
                  onClick={() => {
                    props.onSelectedRules(
                      toggle(props.selectedRules(), r.rule),
                    );
                  }}
                >
                  {shortRule(r.rule)}
                </button>
                {/* Jump affordance: arrow forces the group open + scrolls. */}
                <button
                  type="button"
                  class="ml-1 cursor-pointer text-fg-muted hover:text-accent"
                  title="Jump to this rule"
                  onClick={() => {
                    props.onJump(r.rule);
                  }}
                >
                  ↳
                </button>
                <span class={countBadge}>{r.count}</span>
              </span>
            );
          }}
        </For>
        <button
          type="button"
          class="ml-1 text-xs text-fg-muted underline-offset-2 hover:text-fg hover:underline"
          onClick={() => {
            props.onSelectedRules(new Set(rules().map((r) => r.rule)));
          }}
        >
          All
        </button>
        <button
          type="button"
          class="text-xs text-fg-muted underline-offset-2 hover:text-fg hover:underline"
          onClick={() => {
            props.onSelectedRules(new Set());
          }}
        >
          None
        </button>
      </div>

      {/* Severity + group rows. */}
      <div class="flex flex-wrap items-start gap-x-6 gap-y-3">
        <Show when={severities().length > 0}>
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs font-medium text-fg-muted">Severity</span>
            <For each={severities()}>
              {(s) => {
                const active = () => props.selectedSeverities().has(s.sev);
                return (
                  <button
                    type="button"
                    class={chipBase}
                    classList={{
                      [sevActiveClass(s.sev)]: active(),
                      [chipOff]: !active(),
                    }}
                    onClick={() => {
                      props.onSelectedSeverities(
                        toggle(props.selectedSeverities(), s.sev),
                      );
                    }}
                  >
                    {s.sev}
                    <span class={countBadge}>{s.count}</span>
                  </button>
                );
              }}
            </For>
          </div>
        </Show>

        <Show when={groups().length > 0}>
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs font-medium text-fg-muted">Group</span>
            <For each={groups()}>
              {(g) => {
                const active = () => props.selectedGroups().has(g.group);
                return (
                  <button
                    type="button"
                    class={chipBase}
                    classList={{
                      "border-accent bg-accent/15 text-accent": active(),
                      [chipOff]: !active(),
                    }}
                    onClick={() => {
                      props.onSelectedGroups(
                        toggle(props.selectedGroups(), g.group),
                      );
                    }}
                  >
                    {g.group}
                    <span class={countBadge}>{g.count}</span>
                  </button>
                );
              }}
            </For>
          </div>
        </Show>
      </div>

      {/* Free-text + showing-N-of-M. */}
      <div class="flex flex-wrap items-center gap-3">
        <input
          type="search"
          placeholder="Search line text, descriptions, headings, groups…"
          value={props.search()}
          onInput={(e) => {
            onSearchInput(e.currentTarget.value);
          }}
          class="min-w-0 flex-1 rounded border border-line-strong bg-surface-raised px-2.5 py-1.5 text-sm text-fg outline-none focus:border-accent"
        />
        <span class="text-xs whitespace-nowrap text-fg-muted">
          showing {props.shownCount()} of {props.totalCount()} findings
        </span>
      </div>
    </Disclosure>
  );
};
