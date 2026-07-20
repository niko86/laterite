import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { agsLine } from "../../lib/agsline";
import { downloadBlob } from "../../lib/download";
import {
  loadStandardDict,
  loadEditionMeta,
  isKeyStatus,
  isRequiredStatus,
} from "../../lib/dict";
import type {
  DictGroup,
  DictHeading,
  DictVersionOpt,
} from "../../lib/validator";
import { dictVersion, setDictVersion } from "../../lib/settings";
import { controlClass } from "../../lib/controls";

// Template generator: pick AGS groups for the SELECTED edition and emit a blank
// GROUP/HEADING/UNIT/TYPE skeleton (no DATA rows) ready to fill in — with the
// canonical UNIT + TYPE rows projected from the union `ags_dictionary.json` (the
// single web dict source — see lib/dict.ts) down to the chosen edition. Fully
// client-side. Edition = the shared, shareable `dictVersion` setting; the edition
// list + "auto" fallback come from the union too (loadEditionMeta).

export const TemplateGenerator: Component = () => {
  const [dict] = createResource(
    () => dictVersion(),
    (ed) => loadStandardDict(ed),
  );
  const [editionMeta] = createResource(loadEditionMeta);

  const [q, setQ] = createSignal("");
  const [picked, setPicked] = createSignal<Set<string>>(new Set());
  // Required-only trims each group to its KEY + REQUIRED headings; off emits
  // every standard heading.
  const [requiredOnly, setRequiredOnly] = createSignal(false);

  const groups = createMemo<DictGroup[]>(() => {
    const d = dict();
    if (!d) return [];
    const term = q().trim().toLowerCase();
    if (!term) return d.groups;
    return d.groups.filter(
      (g) =>
        g.code.toLowerCase().includes(term) ||
        g.contents.toLowerCase().includes(term),
    );
  });

  const toggle = (code: string) =>
    setPicked((prev) => {
      const next = new Set(prev);
      if (next.has(code)) next.delete(code);
      else next.add(code);
      return next;
    });

  const keep = (h: DictHeading) =>
    !requiredOnly() || isKeyStatus(h.status) || isRequiredStatus(h.status);

  // Build the skeleton in dictionary order for the picked groups.
  const template = createMemo<string>(() => {
    const d = dict();
    const sel = picked();
    if (!d || sel.size === 0) return "";
    const blocks: string[] = [];
    for (const g of d.groups) {
      if (!sel.has(g.code)) continue;
      const hs = g.headings.filter(keep);
      if (hs.length === 0) continue;
      const names = hs.map((h) => h.name);
      blocks.push(
        [
          agsLine(["GROUP", g.code]),
          agsLine(["HEADING", ...names]),
          // Canonical UNIT row from the dictionary (blank where the heading
          // is unitless) — a more useful skeleton than all-empty units.
          agsLine(["UNIT", ...hs.map((h) => h.unit ?? "")]),
          agsLine(["TYPE", ...hs.map((h) => h.type)]),
        ].join("\r\n"),
      );
    }
    // AGS4 groups are separated by a blank line; trailing newline at EOF.
    return blocks.join("\r\n\r\n") + "\r\n";
  });

  const save = () => {
    downloadBlob(template(), "template.ags", "text/plain;charset=utf-8");
  };

  return (
    <div class="flex min-w-0 flex-col gap-3">
      <p class="text-sm text-fg-soft">
        Pick groups to emit a blank{" "}
        <span class="mono text-fg">GROUP/HEADING/UNIT/TYPE</span> skeleton.
      </p>

      <div class="flex flex-wrap items-center gap-3">
        <input
          class="min-w-0 flex-1 rounded-lg border border-line-strong bg-surface-raised px-3 py-2 text-sm text-fg outline-none placeholder:text-fg-dim"
          placeholder="Search groups… (e.g. LOCA, sample)"
          value={q()}
          onInput={(e) => setQ(e.currentTarget.value)}
        />
        <label class="flex cursor-pointer items-center gap-1.5 text-xs text-fg-muted">
          <input
            type="checkbox"
            checked={requiredOnly()}
            onChange={(e) => setRequiredOnly(e.currentTarget.checked)}
          />
          Required headings only
        </label>
        <label class="flex items-center gap-1.5 text-xs text-fg-muted">
          AGS edition
          <select
            class={controlClass}
            value={dictVersion()}
            onChange={(e) => {
              setDictVersion(e.currentTarget.value as DictVersionOpt);
            }}
          >
            <For each={["auto", ...(editionMeta()?.editions ?? [])]}>
              {(ed) => (
                <option value={ed}>
                  {ed === "auto"
                    ? `auto (→ ${editionMeta()?.fallback ?? ""})`
                    : ed}
                </option>
              )}
            </For>
          </select>
        </label>
      </div>

      <div class="flex flex-wrap items-center gap-3 text-sm">
        <button
          type="button"
          class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-40"
          disabled={picked().size === 0}
          onClick={save}
        >
          Download template ({picked().size})
        </button>
        <Show when={picked().size > 0}>
          <button
            type="button"
            class="text-xs text-fg-muted underline-offset-2 hover:text-fg hover:underline"
            onClick={() => setPicked(new Set())}
          >
            Clear selection
          </button>
        </Show>
      </div>

      <Show
        when={!dict.loading}
        fallback={<p class="text-sm text-fg-muted">Loading dictionary…</p>}
      >
        <div class="grid gap-1 sm:grid-cols-2 lg:grid-cols-3">
          <For each={groups()}>
            {(g) => (
              <label class="flex cursor-pointer items-start gap-2 rounded border border-line bg-surface px-2 py-1.5 text-sm">
                <input
                  type="checkbox"
                  class="mt-1"
                  checked={picked().has(g.code)}
                  onChange={() => toggle(g.code)}
                />
                <span class="min-w-0">
                  <span class="mono font-medium text-fg">{g.code}</span>
                  <span class="ml-1.5 text-xs text-fg-muted">{g.contents}</span>
                </span>
              </label>
            )}
          </For>
        </div>
      </Show>

      <Show when={template()}>
        <pre class="mono max-h-72 max-w-full overflow-auto rounded-lg border border-line bg-surface-code p-2 text-xs leading-relaxed text-fg-soft">
          {template()}
        </pre>
      </Show>
    </div>
  );
};
