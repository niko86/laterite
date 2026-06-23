import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { splitAgsFields, quoteAgsField } from "../../lib/agsline";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";
import { controlCompact } from "../../lib/controls";
import { loadSensitive, prefillCodes, categoryOf } from "../../lib/sensitive";

// Anonymiser / redactor: blank or replace identifying cell values (coords,
// remarks, names) before sharing a file. Column-checklist driven, with
// sensible defaults pre-selected. Fully client-side; only the inner value of
// selected DATA cells changes — every other cell is reproduced verbatim via
// the lossless splitAgsFields round-trip. Line endings are normalised to CRLF
// with a single trailing newline (AGS4 Rule 2a), so the output is canonical
// rather than byte-identical to a hand-edited LF source.
//
// The pre-tick defaults come from the sensitive-headings SSOT
// (sensitive_headings.json) — the SAME list the corpus `censor` tool uses,
// fetched at runtime (see lib/sensitive.ts). It deliberately skips the
// identifier categories (location_id / project_id): this tool blanks values
// and can't pseudonymise, so blanking a cross-referenced key would break the
// file. The user can still tick any column by hand.

interface FileGroup {
  code: string;
  headings: string[];
}

const decode = (b: Uint8Array) =>
  new TextDecoder("utf-8", { fatal: false }).decode(b);

const fieldValue = (cps: string[], f: { valueStart: number; valueEnd: number }) =>
  cps.slice(f.valueStart, f.valueEnd).join("");

/** First-seen group → its HEADING-row names. */
function parseGroups(text: string): FileGroup[] {
  const groups: FileGroup[] = [];
  let cur: FileGroup | null = null;
  for (const line of text.split(/\r?\n/)) {
    if (line.trim() === "") continue;
    const fields = splitAgsFields(line);
    const cps = [...line];
    const tag = fieldValue(cps, fields[0]);
    if (tag === "GROUP") {
      const code = fields[1] ? fieldValue(cps, fields[1]) : "";
      cur = { code, headings: [] };
      if (code && !groups.some((g) => g.code === code)) groups.push(cur);
    } else if (tag === "HEADING" && cur) {
      cur.headings = fields.slice(1).map((f) => fieldValue(cps, f));
    }
  }
  return groups.filter((g) => g.headings.length > 0);
}

const colKey = (group: string, heading: string) => `${group}.${heading}`;

/** Rewrite one DATA line, replacing the inner value of every selected column
 *  with `token`; everything else (quoting, commas, other cells) verbatim. */
function redactLine(raw: string, redactCols: Set<number>, token: string): string {
  const fields = splitAgsFields(raw);
  return fields
    .map((f, i) => {
      if (i >= 1 && redactCols.has(i - 1)) {
        const comma = f.text.endsWith(",") ? "," : "";
        return quoteAgsField(token) + comma;
      }
      return f.text;
    })
    .join("");
}

export const Anonymiser: Component = () => {
  const text = createMemo(() => {
    const b = fileStore.bytes();
    return b ? decode(b) : "";
  });
  const groups = createMemo(() => (text() ? parseGroups(text()) : []));

  // The sensitive-headings SSOT (fetched once). `prefill` = codes to pre-tick;
  // `cats` = code → category for the per-column hint. Empty until it resolves,
  // so the defaults recompute reactively when it arrives.
  const [ssot] = createResource(loadSensitive);
  const prefill = createMemo(() => {
    const d = ssot();
    return d ? prefillCodes(d) : new Set<string>();
  });
  const cats = createMemo(() => {
    const d = ssot();
    return d ? categoryOf(d) : new Map<string, string>();
  });

  // Selected columns as "GROUP.HEADING" keys; default to the sensitive ones.
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  // Seed once when groups first arrive (createMemo recompute keeps it in sync
  // with a newly-loaded file via the effect-free derived default below).
  const defaults = createMemo(() => {
    const s = new Set<string>();
    const pre = prefill();
    for (const g of groups())
      for (const h of g.headings) if (pre.has(h)) s.add(colKey(g.code, h));
    return s;
  });
  // The working set: user edits override, else the computed defaults.
  const [touched, setTouched] = createSignal(false);
  const effective = () => (touched() ? selected() : defaults());

  const toggle = (key: string) => {
    const base = new Set(effective());
    if (base.has(key)) base.delete(key);
    else base.add(key);
    setTouched(true);
    setSelected(base);
  };

  const [token, setToken] = createSignal("REDACTED");
  const [blank, setBlank] = createSignal(false);

  // group code → set of selected heading names.
  const byGroup = createMemo(() => {
    const m = new Map<string, Set<string>>();
    for (const key of effective()) {
      const dot = key.indexOf(".");
      const g = key.slice(0, dot);
      const h = key.slice(dot + 1);
      (m.get(g) ?? m.set(g, new Set()).get(g)!).add(h);
    }
    return m;
  });

  const result = createMemo(() => {
    const t = text();
    if (!t) return { text: "", cells: 0 };
    const sel = byGroup();
    const repl = blank() ? "" : token();
    let cells = 0;
    let curGroup = "";
    let headings: string[] = [];
    const lines = t.split(/\r?\n/);
    // A trailing newline yields one empty segment — drop it so the final
    // "\r\n" below re-adds exactly one, rather than doubling it.
    if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
    const out = lines.map((line) => {
      if (line.trim() === "") return line;
      const fields = splitAgsFields(line);
      const cps = [...line];
      const tag = fieldValue(cps, fields[0]);
      if (tag === "GROUP") {
        curGroup = fields[1] ? fieldValue(cps, fields[1]) : "";
        headings = [];
        return line;
      }
      if (tag === "HEADING") {
        headings = fields.slice(1).map((f) => fieldValue(cps, f));
        return line;
      }
      if (tag === "DATA") {
        const gsel = sel.get(curGroup);
        if (!gsel || gsel.size === 0) return line;
        const cols = new Set<number>();
        headings.forEach((h, i) => {
          if (gsel.has(h)) cols.add(i);
        });
        if (cols.size === 0) return line;
        cells += cols.size;
        return redactLine(line, cols, repl);
      }
      return line;
    });
    return { text: out.join("\r\n") + "\r\n", cells };
  });

  const save = () =>
    downloadBlob(
      result().text,
      `${baseName(fileStore.name())}.anon.ags`,
      "text/plain;charset=utf-8",
    );

  const selectedCount = () => effective().size;

  return (
    <Show
      when={fileStore.bytes()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Anonymiser</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab, then blank or replace
            identifying columns (coordinates, remarks, names) here before
            sharing it. Nothing is uploaded.
          </p>
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <p class="text-sm text-fg-soft">
          Tick the columns to redact (location + free-text columns are
          pre-selected). Only <span class="mono">DATA</span> values change.
        </p>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <button
            type="button"
            class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={selectedCount() === 0}
            onClick={save}
          >
            Download redacted ({result().cells} cells)
          </button>
          <label class="flex cursor-pointer items-center gap-1.5 text-xs text-fg-muted">
            <input
              type="checkbox"
              checked={blank()}
              onChange={(e) => setBlank(e.currentTarget.checked)}
            />
            Blank (else replace with)
          </label>
          <input
            class={`w-32 ${controlCompact} disabled:opacity-40`}
            value={token()}
            disabled={blank()}
            onInput={(e) => setToken(e.currentTarget.value)}
          />
        </div>

        <div class="flex flex-col gap-2">
          <For each={groups()}>
            {(g) => (
              <div class="rounded-lg border border-line bg-surface px-3 py-2">
                <div class="mono text-sm font-medium text-fg">{g.code}</div>
                <div class="mt-1 flex flex-wrap gap-x-3 gap-y-1">
                  <For each={g.headings}>
                    {(h) => {
                      const key = colKey(g.code, h);
                      return (
                        <label class="flex cursor-pointer items-center gap-1 text-xs">
                          <input
                            type="checkbox"
                            checked={effective().has(key)}
                            onChange={() => toggle(key)}
                          />
                          <span
                            class="mono"
                            classList={{
                              "text-accent": effective().has(key),
                              "text-fg-muted": !effective().has(key),
                            }}
                          >
                            {h}
                          </span>
                          <Show when={cats().get(h)}>
                            {(c) => (
                              <span class="rounded bg-line/60 px-1 text-[10px] text-fg-faint">
                                {c()}
                              </span>
                            )}
                          </Show>
                        </label>
                      );
                    }}
                  </For>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
};
