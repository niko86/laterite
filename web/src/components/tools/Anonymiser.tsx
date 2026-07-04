import {
  createEffect,
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
import {
  loadSensitive,
  categoryOf,
  actionOf,
  codesForPreset,
  type Action,
  type Preset,
} from "../../lib/sensitive";

// Anonymiser / redactor: strip identifying data before sharing a file. Each
// column's action comes from the sensitive-headings SSOT (sensitive_headings.json,
// the SAME classification the corpus `censor` uses) — location IDs are
// PSEUDONYMISED (a stable per-column token, so cross-group references survive),
// PROJ_ID → the file's content hash, coordinates blanked, names/labs/etc →
// a token, free-text `[units]` stripped. A preset dropdown pre-ticks by scope;
// the checklist stays editable. Fully client-side; only selected DATA cell
// values change (everything else reproduced verbatim via the lossless
// splitAgsFields round-trip); output is canonical CRLF.

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
const headOf = (key: string) => key.slice(key.indexOf(".") + 1);

/** First 8 bytes of SHA-256, hex — a short, stable, anonymous content id for
 *  PROJ_ID (mirrors the corpus censor's file-hash id). */
async function sha16(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes.slice().buffer);
  return [...new Uint8Array(digest)]
    .slice(0, 8)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/** Rewrite one DATA line: for every selected column, replace the inner value
 *  via `redact(heading, value)` (null ⇒ leave verbatim); everything else
 *  (quoting, commas, unselected cells) is reproduced byte-for-byte. */
function redactLine(
  raw: string,
  headings: string[],
  selected: Set<string>,
  redact: (heading: string, value: string) => string | null,
): { line: string; cells: number } {
  const fields = splitAgsFields(raw);
  const cps = [...raw];
  let cells = 0;
  const line = fields
    .map((f, i) => {
      if (i < 1) return f.text;
      const h = headings[i - 1];
      if (!h || !selected.has(h)) return f.text;
      const out = redact(h, fieldValue(cps, f));
      if (out === null) return f.text;
      cells++;
      const comma = f.text.endsWith(",") ? "," : "";
      return quoteAgsField(out) + comma;
    })
    .join("");
  return { line, cells };
}

const PRESETS: { id: Preset | "custom"; label: string }[] = [
  { id: "all", label: "All identifying" },
  { id: "coords-text", label: "Coordinates + free-text" },
  { id: "coords", label: "Coordinates only" },
  { id: "custom", label: "Custom" },
];

// Human labels for the per-column action badge.
const ACTION_LABEL: Record<Action, string> = {
  pseudonym: "pseudonym",
  filehash: "hashed",
  blank: "blanked",
  token: "tokenised",
  brackets: "[stripped]",
};

export const Anonymiser: Component = () => {
  const text = createMemo(() => {
    const b = fileStore.bytes();
    return b ? decode(b) : "";
  });
  const groups = createMemo(() => (text() ? parseGroups(text()) : []));

  const [ssot] = createResource(loadSensitive);
  const cats = createMemo(() => {
    const d = ssot();
    return d ? categoryOf(d) : new Map<string, string>();
  });
  const acts = createMemo(() => {
    const d = ssot();
    return d ? actionOf(d) : new Map<string, Action>();
  });

  // Preset drives the pre-tick; the working set is editable on top of it.
  const [preset, setPreset] = createSignal<Preset | "custom">("all");
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [token, setToken] = createSignal("REDACTED");

  // (Re)seed the selection whenever the preset or the file/SSOT changes — the
  // columns present in THIS file whose heading is in the preset's code set.
  const presetSelection = createMemo(() => {
    const d = ssot();
    const p = preset();
    if (!d || p === "custom") return null;
    const codes = codesForPreset(d, p);
    const s = new Set<string>();
    for (const g of groups())
      for (const h of g.headings) if (codes.has(h)) s.add(colKey(g.code, h));
    return s;
  });
  createEffect(() => {
    const s = presetSelection();
    if (s) setSelected(s); // a real preset seeds; "custom" leaves edits alone
  });

  const toggle = (key: string) => {
    const base = new Set(selected());
    if (base.has(key)) base.delete(key);
    else base.add(key);
    setPreset("custom"); // any manual edit switches to Custom
    setSelected(base);
  };

  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);
  const [note, setNote] = createSignal<string | null>(null);

  // Live cell-count estimate (selected DATA cells), for the button label.
  const selectedCells = createMemo(() => {
    const t = text();
    const sel = selected();
    if (!t || sel.size === 0) return 0;
    let curGroup = "";
    let headings: string[] = [];
    let n = 0;
    for (const line of t.split(/\r?\n/)) {
      if (line.trim() === "") continue;
      const fields = splitAgsFields(line);
      const cps = [...line];
      const tag = fieldValue(cps, fields[0]);
      if (tag === "GROUP") {
        curGroup = fields[1] ? fieldValue(cps, fields[1]) : "";
        headings = [];
      } else if (tag === "HEADING") {
        headings = fields.slice(1).map((f) => fieldValue(cps, f));
      } else if (tag === "DATA") {
        for (const h of headings) if (sel.has(colKey(curGroup, h))) n++;
      }
    }
    return n;
  });

  // Build the anonymised text. Async because PROJ_ID hashing uses Web Crypto,
  // and pseudonyms need a first pass to assign stable per-column tokens.
  const anonymise = async (): Promise<{ text: string; cells: number }> => {
    const t = text();
    const raw = fileStore.bytes();
    if (!t || !raw) return { text: "", cells: 0 };
    const sel = selected();
    const action = acts();
    const tok = token();

    const selHeads = (group: string) => {
      const s = new Set<string>();
      for (const key of sel)
        if (key.startsWith(`${group}.`)) s.add(headOf(key));
      return s;
    };

    // Precompute the file-hash only if a hashed column is actually selected.
    const needHash = [...sel].some((k) => action.get(headOf(k)) === "filehash");
    const fileHash = needHash ? await sha16(raw) : "";

    const lines = t.split(/\r?\n/);
    // A trailing newline yields one empty segment — drop it so the final CRLF
    // re-adds exactly one, rather than doubling it.
    if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();

    // PASS 1: assign pseudonyms. Per heading-CODE map (shared across groups, so
    // a KEY that appears in several groups gets the SAME token → cross-refs
    // survive), populated in source order: first-seen value → `{SUFFIX}{0001}`.
    const pseudo = new Map<string, Map<string, string>>();
    {
      let curGroup = "";
      let headings: string[] = [];
      for (const line of lines) {
        if (line.trim() === "") continue;
        const fields = splitAgsFields(line);
        const cps = [...line];
        const tag = fieldValue(cps, fields[0]);
        if (tag === "GROUP") {
          curGroup = fields[1] ? fieldValue(cps, fields[1]) : "";
          headings = [];
        } else if (tag === "HEADING") {
          headings = fields.slice(1).map((f) => fieldValue(cps, f));
        } else if (tag === "DATA") {
          const sh = selHeads(curGroup);
          headings.forEach((h, i) => {
            if (!sh.has(h) || action.get(h) !== "pseudonym") return;
            const val = fieldValue(cps, fields[i + 1] ?? { valueStart: 0, valueEnd: 0 });
            if (!val) return;
            const map = pseudo.get(h) ?? pseudo.set(h, new Map()).get(h)!;
            if (!map.has(val)) {
              const suffix = (h.split("_").pop() ?? "ANON").toUpperCase();
              map.set(val, `${suffix}${String(map.size + 1).padStart(4, "0")}`);
            }
          });
        }
      }
    }

    const redact = (h: string, value: string): string | null => {
      const a = action.get(h);
      switch (a) {
        case "pseudonym":
          return value === "" ? null : (pseudo.get(h)?.get(value) ?? value);
        case "filehash":
          return value === "" ? null : fileHash;
        case "blank":
          return "";
        case "brackets":
          return value.replace(/\[[^\]]*\]/g, `[${tok}]`);
        // token, and any manually-ticked unclassified column, → the token.
        default:
          return tok;
      }
    };

    let cells = 0;
    let curGroup = "";
    let headings: string[] = [];
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
        const r = redactLine(line, headings, selHeads(curGroup), redact);
        cells += r.cells;
        return r.line;
      }
      return line;
    });
    return { text: out.join("\r\n") + "\r\n", cells };
  };

  const save = async () => {
    if (selected().size === 0) return;
    setBusy(true);
    setErr(null);
    setNote(null);
    try {
      const r = await anonymise();
      downloadBlob(
        r.text,
        `${baseName(fileStore.name())}.anon.ags`,
        "text/plain;charset=utf-8",
      );
      setNote(`Redacted ${r.cells} cell${r.cells === 1 ? "" : "s"} → .anon.ags`);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show
      when={fileStore.bytes()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Anonymiser</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab, then strip identifying data
            here before sharing it — IDs pseudonymised, coordinates blanked,
            names tokenised. Nothing is uploaded.
          </p>
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <p class="text-sm text-fg-soft">
          Each column's action comes from its category (see the badge): location
          IDs are <span class="text-accent">pseudonymised</span> (cross-references
          stay intact), <span class="mono">PROJ_ID</span> is hashed, coordinates
          blanked, names/labs tokenised.
        </p>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <label class="flex items-center gap-1.5 text-xs text-fg-muted">
            Preset
            <select
              class={controlCompact}
              value={preset()}
              onChange={(e) => setPreset(e.currentTarget.value as Preset | "custom")}
            >
              <For each={PRESETS}>
                {(p) => <option value={p.id}>{p.label}</option>}
              </For>
            </select>
          </label>
          <button
            type="button"
            class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={busy() || selected().size === 0}
            onClick={save}
          >
            {busy() ? "Redacting…" : `Download redacted (${selectedCells()} cells)`}
          </button>
          <label class="flex items-center gap-1.5 text-xs text-fg-muted">
            token
            <input
              class={`w-28 ${controlCompact}`}
              value={token()}
              onInput={(e) => setToken(e.currentTarget.value)}
            />
          </label>
        </div>

        <Show when={note()}>
          <p class="text-xs text-ok">✓ {note()}</p>
        </Show>
        <Show when={err()}>
          <p class="text-xs text-err">Redaction failed: {err()}</p>
        </Show>

        <div class="flex flex-col gap-2">
          <For each={groups()}>
            {(g) => (
              <div class="rounded-lg border border-line bg-surface px-3 py-2">
                <div class="mono text-sm font-medium text-fg">{g.code}</div>
                <div class="mt-1 flex flex-wrap gap-x-3 gap-y-1">
                  <For each={g.headings}>
                    {(h) => {
                      const key = colKey(g.code, h);
                      const action = () => acts().get(h);
                      return (
                        <label class="flex cursor-pointer items-center gap-1 text-xs">
                          <input
                            type="checkbox"
                            checked={selected().has(key)}
                            onChange={() => toggle(key)}
                          />
                          <span
                            class="mono"
                            classList={{
                              "text-accent": selected().has(key),
                              "text-fg-muted": !selected().has(key),
                            }}
                          >
                            {h}
                          </span>
                          <Show when={action()}>
                            {(a) => (
                              <span class="rounded bg-line/60 px-1 text-[10px] text-fg-faint">
                                {ACTION_LABEL[a()]}
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
