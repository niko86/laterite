import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { controlFocus } from "../../lib/controls";

// Plain-English reference for the AGS4 validation rules: what each rule
// checks, its severity, whether the validator can auto-fix it, and any known
// differences from the python-ags4 checker (the O-N divergences).
// Reads the static catalogue web/public/rules-catalogue.json — a verbatim copy
// of the single source of truth, the validator's rules_meta.json (synced by
// scripts/sync-rules.mjs, gated by src/lib/rulesCatalogue.test.ts) — so the
// page can't drift from the rules the engine actually emits. Fully client-side,
// useful with no file loaded.

interface Obs {
  id: string;
  note: string;
}
interface Rule {
  rule: string;
  title: string;
  checks: string;
  severity: string;
  fixable?: boolean;
  observations?: Obs[];
}
interface Catalogue {
  rules: Rule[];
}

function sevClass(sev: string): string {
  switch (sev) {
    case "error":
      return "bg-err-quiet text-err";
    case "warning":
      return "bg-warn-quiet text-warn";
    // A rule whose finding is always an error, but which can ALSO emit a
    // related FYI/Warning bucket (e.g. Rule 1 extended-ASCII, Rule 16/18 DICT)
    // — the catalogue marks these `mixed`. Amber, like a warning.
    case "mixed":
      return "bg-warn-quiet text-warn";
    case "fyi":
      return "bg-info-quiet text-info";
    default:
      return "bg-chip text-fg-soft";
  }
}

export const RuleExplainer: Component = () => {
  const [cat] = createResource(async () => {
    const res = await fetch(`${import.meta.env.BASE_URL}rules-catalogue.json`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as Catalogue;
  });
  const [q, setQ] = createSignal("");

  const rules = createMemo<Rule[]>(() => {
    const c = cat();
    if (!c) return [];
    const term = q().trim().toLowerCase();
    if (!term) return c.rules;
    return c.rules.filter((r) =>
      `${r.rule} ${r.title} ${r.checks}`.toLowerCase().includes(term),
    );
  });

  return (
    <div class="flex min-w-0 flex-col gap-3">
      {/* The prominent search role — deliberately larger (px-3 py-2) than the
          Input control, per lib/controls.ts; radius + focus are the same
          contract (#408). */}
      <input
        class={`w-full rounded-xs border border-line-strong bg-surface-raised px-3 py-2 text-sm text-fg ${controlFocus} placeholder:text-fg-dim`}
        placeholder="Search rules… (e.g. duplicate, datetime, heading)"
        value={q()}
        onInput={(e) => setQ(e.currentTarget.value)}
      />
      <p class="text-xs text-fg-dim">
        The AGS Format Rules are stable across editions 4.0.3–4.2, so there's no
        edition to pick here — only the group/heading/type definitions change.
        For those, use the{" "}
        <span class="font-medium text-fg-soft">Dictionary</span> tool's edition
        selector.
      </p>
      <Show
        when={!cat.loading}
        fallback={<p class="text-sm text-fg-muted">Loading rule catalogue…</p>}
      >
        <Show
          when={!cat.error}
          fallback={
            <p class="text-sm text-err">
              Could not load the rule catalogue: {String(cat.error)}
            </p>
          }
        >
          <For each={rules()}>
            {(r) => (
              <div class="rounded-lg border border-line bg-surface px-3 py-2">
                <div class="flex flex-wrap items-baseline gap-2">
                  <span class="mono font-medium text-fg">Rule {r.rule}</span>
                  <span class="text-fg-soft">{r.title}</span>
                  <span
                    class={`rounded-xs px-1.5 py-0.5 text-[10px] uppercase tracking-wide ${sevClass(r.severity)}`}
                  >
                    {r.severity}
                  </span>
                  <Show when={r.fixable}>
                    <span class="rounded-xs bg-ok-quiet px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-ok">
                      auto-fixable
                    </span>
                  </Show>
                </div>
                <p class="mt-1 text-sm text-fg-soft">{r.checks}</p>
                <Show when={r.observations && r.observations.length > 0}>
                  <div class="mt-2 border-t border-line-subtle pt-2">
                    <p class="text-xs font-medium text-fg-muted">
                      Differences from the python-ags4 checker:
                    </p>
                    <ul class="mt-1 space-y-0.5">
                      <For each={r.observations}>
                        {(o) => (
                          <li class="text-xs text-fg-faint">
                            <span class="mono text-fg-dim">{o.id}</span>{" "}
                            {o.note}
                          </li>
                        )}
                      </For>
                    </ul>
                  </div>
                </Show>
              </div>
            )}
          </For>
        </Show>
      </Show>
    </div>
  );
};
