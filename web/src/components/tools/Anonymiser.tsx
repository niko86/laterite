import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { splitAgsFields } from "../../lib/agsline";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";
import { Button, Checkbox, Input, Select } from "@shared/components";
import { censorFile, type CensorResult } from "../../lib/validatorClient";
import {
  loadSensitive,
  actionOf,
  codesForPreset,
  type Action,
  type Preset,
} from "../../lib/sensitive";

// Anonymiser / redactor: strip identifying data before sharing a file. The
// scrub runs in the shared `laterite-ags4-censor` engine (#581) via the
// validator worker — the SAME engine the corpus `censor` tool drives, so the
// two can't drift. Each column's action comes from the sensitive-headings SSOT
// (sensitive_headings.json): location IDs are PSEUDONYMISED (a stable per-column
// token, cross-group references intact), PROJ_ID → the file's content hash,
// coordinates blanked, names/labs/etc → a token, free-text `[units]` stripped.
// The engine also drops custom (non-dictionary) groups/columns and tokenises
// sensitive ABBR pick-lists. A preset dropdown pre-ticks by scope; the checklist
// stays editable. Fully client-side; line endings + untouched cells are
// preserved byte-for-byte (the engine is cell-surgical), nothing is uploaded.

interface FileGroup {
  code: string;
  headings: string[];
}

const decode = (b: Uint8Array) =>
  new TextDecoder("utf-8", { fatal: false }).decode(b);

const fieldValue = (
  cps: string[],
  f: { valueStart: number; valueEnd: number } | undefined,
) => (f ? cps.slice(f.valueStart, f.valueEnd).join("") : "");

/** First-seen group → its HEADING-row names. */
function parseGroups(text: string): FileGroup[] {
  const groups: FileGroup[] = [];
  let cur: FileGroup | null = null;
  for (const line of text.split(/\r?\n/)) {
    if (line.trim() === "") continue;
    const fields = splitAgsFields(line);
    const cps = Array.from(line);
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
  const acts = createMemo(() => {
    const d = ssot();
    return d ? actionOf(d) : new Map<string, Action>();
  });

  // Preset drives the pre-tick; the working set is editable on top of it.
  const [preset, setPreset] = createSignal<Preset | "custom">("all");
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [token, setToken] = createSignal("REDACTED");
  // Drop custom (non-dictionary) groups/columns — a real anonymisation gain
  // (bespoke columns can hold un-classified client data), so ON by default; a
  // toggle since it removes data. The shared engine's `drop_custom`.
  const [dropCustom, setDropCustom] = createSignal(true);

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
      const cps = Array.from(line);
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

  // Anonymise via the shared scrub engine in the validator worker (#581) — the
  // SAME `laterite-ags4-censor` engine the corpus `censor` tool drives. A batch
  // action (Download), so it rides the worker asynchronously; the file hash,
  // pseudonym maps, custom-group dropping and quoting all live in the engine.
  const anonymise = async (): Promise<CensorResult | null> => {
    const raw = fileStore.bytes();
    const d = ssot();
    if (!raw || !d) return null;
    // The selection is per (group, heading); the engine scrubs by heading CODE
    // (a shared KEY gets the SAME pseudonym across groups), so collapse to the
    // set of selected codes and restrict the policy to them.
    const selectedCodes = [...new Set([...selected()].map(headOf))];
    return censorFile(raw, {
      sensitiveJson: JSON.stringify(d),
      selectedCodes,
      token: token(),
      dropCustom: dropCustom(),
      includeFreetext: false,
    });
  };

  const save = async () => {
    if (selected().size === 0) return;
    setBusy(true);
    setErr(null);
    setNote(null);
    try {
      const r = await anonymise();
      if (!r) return;
      downloadBlob(
        r.text,
        `${baseName(fileStore.name())}.anon.ags`,
        "text/plain;charset=utf-8",
      );
      const t = r.tally;
      const cells = t.pseudonym + t.blank + t.token + t.brackets;
      const dropped = t.dropped_groups + t.dropped_cols;
      const extra =
        dropped > 0
          ? `, dropped ${dropped} custom item${dropped === 1 ? "" : "s"}`
          : "";
      setNote(
        `Redacted ${cells} cell${cells === 1 ? "" : "s"}${extra} → .anon.ags`,
      );
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
          IDs are <span class="text-accent">pseudonymised</span>{" "}
          (cross-references stay intact), <span class="mono">PROJ_ID</span> is
          hashed, coordinates blanked, names/labs tokenised.
        </p>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <label class="flex items-center gap-1.5 text-xs text-fg-muted">
            Preset
            <Select
              width="w-auto"
              value={preset()}
              onChange={(e) =>
                setPreset(e.currentTarget.value as Preset | "custom")
              }
            >
              <For each={PRESETS}>
                {(p) => <option value={p.id}>{p.label}</option>}
              </For>
            </Select>
          </label>
          <Button
            variant="primary"
            disabled={busy() || selected().size === 0}
            onClick={() => void save()}
          >
            {busy()
              ? "Redacting…"
              : `Download redacted (${selectedCells()} cells)`}
          </Button>
          <label class="flex items-center gap-1.5 text-xs text-fg-muted">
            token
            <Input
              width="w-28"
              value={token()}
              onInput={(e) => setToken(e.currentTarget.value)}
            />
          </label>
          <Checkbox
            label="drop non-standard groups"
            checked={dropCustom()}
            onChange={(e) => setDropCustom(e.currentTarget.checked)}
          />
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
                        /* Native in-label input: the label carries the
                           selection-state repaint and an action badge — richer
                           than the Checkbox primitive's string label (the
                           FixesPanel call). */
                        <label class="flex cursor-pointer items-center gap-1 text-xs">
                          <input
                            type="checkbox"
                            checked={selected().has(key)}
                            onChange={() => {
                              toggle(key);
                            }}
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
                              <span class="rounded-xs bg-line/60 px-1 text-[10px] text-fg-faint">
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
