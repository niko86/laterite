// Export tab — build a valid AGS4 file from your own per-group data, entirely
// client-side (the `laterite-ags4-wasm` `to_ags4` producer). Each group's column
// headings are the AGS headings; UNIT/TYPE fill from the chosen edition's
// dictionary; the mode picks AutoFix / Report / Strict. Nothing is uploaded.

import { createSignal, For, Show, type Component } from "solid-js";
import { Card } from "../Card";
import { toAgs4 } from "../../lib/validatorClient";
import { downloadBlob } from "../../lib/download";
import type { DictVersionOpt, EmitMode, ExportResult } from "../../lib/validator";

type Edition = Exclude<DictVersionOpt, "auto">;

const EDITIONS: Edition[] = ["4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"];
const MODES: { id: EmitMode; label: string }[] = [
  { id: "autofix", label: "AutoFix — apply safe fixes" },
  { id: "report", label: "Report — emit as-is + findings" },
  { id: "strict", label: "Strict — reject if invalid" },
];

// A working example so the tab does something useful on first load: columns
// are AGS headings; a typed float (12.3) canonicalises to 2DP on emit.
const EXAMPLE = JSON.stringify(
  [
    { code: "PROJ", headings: ["PROJ_ID", "PROJ_NAME"], rows: [["P1", "Demo project"]] },
    { code: "TRAN", headings: ["TRAN_DLIM", "TRAN_RCON"], rows: [["|", ";"]] },
    {
      code: "LOCA",
      headings: ["LOCA_ID", "LOCA_NATE", "LOCA_GL"],
      rows: [
        ["BH01", 523145.1, 12.3],
        ["BH02", 523200, 13],
      ],
    },
    {
      code: "GEOL",
      headings: ["LOCA_ID", "GEOL_TOP", "GEOL_BASE", "GEOL_LEG", "GEOL_GEOL"],
      rows: [["BH01", 0, 1.5, "101", "CLAY"]],
    },
  ],
  null,
  2,
);

export const ExportPane: Component = () => {
  const [edition, setEdition] = createSignal<Edition>("4.1.1");
  const [mode, setMode] = createSignal<EmitMode>("autofix");
  const [json, setJson] = createSignal(EXAMPLE);
  const [result, setResult] = createSignal<ExportResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  const build = async (download: boolean) => {
    setBusy(true);
    setError(null);
    try {
      const r = await toAgs4(json(), edition(), mode());
      setResult(r);
      if (download) downloadBlob(r.text, "delivery.ags", "text/plain");
    } catch (e) {
      setResult(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="flex flex-col gap-4">
      <Card>
        <h2 class="mb-1 text-lg font-semibold text-fg">Build &amp; export AGS4</h2>
        <p class="mb-3 text-sm text-fg-muted">
          Produce a valid AGS4 file from your own per-group data — entirely in
          your browser, nothing uploaded. Each group's column headings are the
          AGS headings; UNIT/TYPE fill from the chosen edition's dictionary.
        </p>

        <div class="flex flex-wrap gap-4">
          <label class="flex flex-col gap-1 text-xs text-fg-muted">
            Edition
            <select
              class="rounded border border-line bg-surface px-2 py-1 text-sm text-fg"
              value={edition()}
              onInput={(e) => setEdition(e.currentTarget.value as Edition)}
            >
              <For each={EDITIONS}>{(ed) => <option value={ed}>{ed}</option>}</For>
            </select>
          </label>
          <label class="flex flex-col gap-1 text-xs text-fg-muted">
            Mode
            <select
              class="rounded border border-line bg-surface px-2 py-1 text-sm text-fg"
              value={mode()}
              onInput={(e) => setMode(e.currentTarget.value as EmitMode)}
            >
              <For each={MODES}>{(m) => <option value={m.id}>{m.label}</option>}</For>
            </select>
          </label>
        </div>

        <label class="mt-3 flex flex-col gap-1 text-xs text-fg-muted">
          Group data (JSON) — an array of <code>{"{ code, headings, rows }"}</code>
          <textarea
            class="h-64 w-full rounded border border-line bg-surface px-2 py-1 font-mono text-xs text-fg"
            spellcheck={false}
            value={json()}
            onInput={(e) => setJson(e.currentTarget.value)}
          />
        </label>

        <div class="mt-3 flex flex-wrap items-center gap-2">
          <button
            type="button"
            class="rounded border border-accent px-3 py-1.5 font-medium text-accent hover:bg-chip disabled:opacity-50"
            disabled={busy()}
            onClick={() => build(true)}
          >
            Build &amp; download .ags
          </button>
          <button
            type="button"
            class="rounded border border-line-strong px-3 py-1.5 font-medium text-fg-soft hover:bg-chip disabled:opacity-50"
            disabled={busy()}
            onClick={() => build(false)}
          >
            Preview only
          </button>
          <button
            type="button"
            class="rounded px-2 py-1.5 text-sm text-fg-muted hover:text-fg"
            onClick={() => {
              setJson(EXAMPLE);
              setResult(null);
              setError(null);
            }}
          >
            Reset example
          </button>
          <Show when={busy()}>
            <span class="text-xs text-fg-muted">Building…</span>
          </Show>
        </div>

        <Show when={error()}>
          {(err) => (
            <p class="mt-3 rounded border border-line bg-chip px-3 py-2 text-sm text-warn">
              {err()}
            </p>
          )}
        </Show>
      </Card>

      <Show when={result()}>
        {(r) => (
          <Card>
            <div class="mb-2 flex flex-wrap items-center gap-3 text-sm">
              <span class="font-medium text-fg">Result</span>
              <span class="text-fg-muted">{r().fixes_applied} safe fix(es) applied</span>
              <span class="text-fg-muted">{r().findings.length} finding(s)</span>
            </div>
            <Show when={r().findings.length > 0}>
              <ul class="mb-3 flex max-h-40 flex-col gap-1 overflow-auto text-xs text-fg-muted">
                <For each={r().findings.slice(0, 50)}>
                  {(f) => (
                    <li>
                      <span class="text-fg-soft">{f.rule}</span> — {f.group}: {f.desc}
                    </li>
                  )}
                </For>
              </ul>
            </Show>
            <pre class="max-h-96 overflow-auto rounded border border-line bg-surface p-2 font-mono text-xs text-fg">
              {r().text}
            </pre>
          </Card>
        )}
      </Show>
    </div>
  );
};
