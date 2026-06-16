import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { dictionary } from "../../lib/validatorClient";
import type { DictGroup, DictVersionOpt } from "../../lib/validator";
import { dictVersion, setDictVersion } from "../../lib/settings";
import { Chevron } from "../Chevron";
import { controlClass } from "../../lib/controls";

// Searchable reference for the AGS4 groups + their headings, for the SELECTED
// edition. Served from the engine's own per-edition standard dictionary (via
// the wasm `dictionary(edition)` export) — canonical names + descriptions +
// units + types + status, the same data validation checks against. (It used to
// fetch a static scaffolded merged JSON whose descriptions were ~91% empty, so
// searching by description found nothing.) Edition is the shared, shareable
// `dictVersion` setting; "auto" resolves to the engine's fallback edition.

// The edition the engine falls back to when none is forced (matches
// laterite_ags4_validator::dict::FALLBACK); shown so "auto" isn't a mystery.
const AUTO_EDITION = "4.1.1";
const EDITIONS: DictVersionOpt[] = [
  "auto",
  "4.0.3",
  "4.0.4",
  "4.1",
  "4.1.1",
  "4.2",
];

function statusClass(status: string): string {
  switch (status.toUpperCase()) {
    case "KEY":
      return "text-accent";
    case "REQUIRED":
      return "text-warn";
    default:
      return "text-fg-faint";
  }
}

export const DictionaryBrowser: Component = () => {
  // Reload whenever the edition changes; the worker resolves "auto" itself.
  const [dict] = createResource(
    () => dictVersion(),
    (ed) => dictionary(ed),
  );
  const [q, setQ] = createSignal("");

  const groups = createMemo<DictGroup[]>(() => {
    const d = dict();
    if (!d) return [];
    const term = q().trim().toUpperCase();
    if (!term) return d.groups;
    return d.groups.filter(
      (g) =>
        g.code.includes(term) ||
        g.contents.toUpperCase().includes(term) ||
        g.headings.some(
          (h) =>
            h.name.includes(term) ||
            h.description.toUpperCase().includes(term) ||
            h.type.toUpperCase().includes(term),
        ),
    );
  });

  return (
    <div class="flex min-w-0 flex-col gap-3">
      <div class="flex flex-wrap items-center gap-2">
        <input
          class="min-w-0 flex-1 rounded-lg border border-line-strong bg-surface-raised px-3 py-2 text-sm text-fg outline-none placeholder:text-fg-dim"
          placeholder="Search groups, headings, descriptions, types… (e.g. LOCA, depth, GEOL_TOP, DT)"
          value={q()}
          onInput={(e) => setQ(e.currentTarget.value)}
        />
        <label class="flex items-center gap-1.5 text-sm text-fg-muted">
          AGS edition
          <select
            class={controlClass}
            value={dictVersion()}
            onChange={(e) => setDictVersion(e.currentTarget.value as DictVersionOpt)}
          >
            <For each={EDITIONS}>
              {(ed) => (
                <option value={ed}>
                  {ed === "auto" ? `auto (→ ${AUTO_EDITION})` : ed}
                </option>
              )}
            </For>
          </select>
        </label>
      </div>
      <Show
        when={!dict.loading}
        fallback={<p class="text-sm text-fg-muted">Loading dictionary…</p>}
      >
        <Show
          when={!dict.error}
          fallback={
            <p class="text-sm text-err">
              Could not load the AGS4 dictionary: {String(dict.error)}
            </p>
          }
        >
          <p class="text-xs text-fg-muted">
            {groups().length} of {dict()?.groups.length} groups · AGS{" "}
            {dict()?.ags_edition} standard dictionary
          </p>
          <For each={groups()}>
            {(g) => (
              <details
                class="group rounded-lg border border-line bg-surface"
                open={q().trim().length > 0}
              >
                <summary class="flex cursor-pointer list-none select-none items-center px-3 py-2 text-sm [&::-webkit-details-marker]:hidden">
                  <Chevron class="mr-2" />
                  <span class="mono font-medium text-accent">{g.code}</span>
                  <span class="ml-2 text-fg-soft">{g.contents}</span>
                  <Show when={g.parent}>
                    <span class="ml-2 text-xs text-fg-dim">↑ {g.parent}</span>
                  </Show>
                  <span class="ml-2 text-xs text-fg-dim">
                    {g.headings.length} headings
                  </span>
                </summary>
                <div class="overflow-x-auto border-t border-line-subtle">
                  <table class="w-full text-xs">
                    <thead class="bg-surface-raised text-fg-muted">
                      <tr>
                        <th class="px-3 py-1 text-left font-medium">Heading</th>
                        <th class="px-3 py-1 text-left font-medium">Status</th>
                        <th class="px-3 py-1 text-left font-medium">Type</th>
                        <th class="px-3 py-1 text-left font-medium">Unit</th>
                        <th class="px-3 py-1 text-left font-medium">
                          Description
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={g.headings}>
                        {(h) => (
                          <tr class="border-t border-line-subtle">
                            <td class="mono px-3 py-1 text-fg">{h.name}</td>
                            <td
                              class={`px-3 py-1 font-medium ${statusClass(h.status)}`}
                            >
                              {h.status}
                            </td>
                            <td class="mono px-3 py-1 text-fg-faint">
                              {h.type}
                            </td>
                            <td class="mono px-3 py-1 text-fg-faint">
                              {h.unit ?? ""}
                            </td>
                            <td class="px-3 py-1 text-fg-soft">
                              {h.description}
                            </td>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </div>
              </details>
            )}
          </For>
        </Show>
      </Show>
    </div>
  );
};
