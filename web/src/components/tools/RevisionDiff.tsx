import {
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { revisionDiff } from "../../lib/validatorClient";
import type { GroupDelta, RowDelta } from "../../lib/validator";
import { fileStore } from "../../lib/fileStore";

// Revision diff: compare two AGS4 files in the engine, not in JS. Rows are
// matched by their dictionary KEY headings (so a re-sorted file still pairs
// the same boreholes) and cells compared type-aware (a "1.0" → "1.00"
// reformat is NOT a change). This is the value a plain text diff (the Fix
// tab's FileDiff) can't give — it understands the AGS data model.

const MAX_ROWS_PER_GROUP = 500;

interface Picked {
  name: string;
  bytes: Uint8Array;
}

async function readFile(f: File): Promise<Picked> {
  return { name: f.name, bytes: new Uint8Array(await f.arrayBuffer()) };
}

export const RevisionDiff: Component = () => {
  const [a, setA] = createSignal<Picked | null>(null);
  const [b, setB] = createSignal<Picked | null>(null);

  // Seed the baseline with the file already loaded in the app, if any — the
  // common case is "diff what I'm working on against a previous delivery".
  const loaded = fileStore.bytes();
  if (loaded) setA({ name: fileStore.name() || "loaded file", bytes: loaded });

  const [delta] = createResource(
    () => {
      const x = a();
      const y = b();
      return x && y ? { x, y } : null;
    },
    ({ x, y }) => revisionDiff(x.bytes, y.bytes, "utf-8", MAX_ROWS_PER_GROUP),
  );

  const pick = (set: (p: Picked) => void) => async (e: Event) => {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) set(await readFile(f));
  };

  return (
    <div class="flex min-w-0 flex-col gap-4">
      <p class="text-sm text-fg-soft">
        Compare two AGS4 files. Rows are matched by their dictionary{" "}
        <span class="mono text-fg">KEY</span> headings and cells compared
        type-aware — a re-formatted value (<span class="mono">1.0</span> →{" "}
        <span class="mono">1.00</span>) is not reported as a change. Both files
        stay in your browser.
      </p>

      <div class="grid gap-3 sm:grid-cols-2">
        <FilePicker
          label="Baseline (a)"
          picked={a()}
          onPick={pick(setA)}
        />
        <FilePicker label="Revision (b)" picked={b()} onPick={pick(setB)} />
      </div>

      <Show
        when={a() && b()}
        fallback={
          <p class="text-sm text-fg-muted">
            Choose both files to see the differences.
          </p>
        }
      >
        <Show
          when={!delta.loading}
          fallback={<p class="text-sm text-fg-muted">Comparing…</p>}
        >
          <Show
            when={!delta.error}
            fallback={
              <p class="text-sm text-err">
                Could not compare: {String(delta.error)}
              </p>
            }
          >
            <Show when={delta()}>{(d) => <DeltaView delta={d()} />}</Show>
          </Show>
        </Show>
      </Show>
    </div>
  );
};

const FilePicker: Component<{
  label: string;
  picked: Picked | null;
  onPick: (e: Event) => void;
}> = (props) => (
  <label class="flex cursor-pointer flex-col gap-1 rounded-lg border border-dashed border-line-strong bg-surface px-3 py-3 text-sm hover:border-accent">
    <span class="font-medium text-fg-soft">{props.label}</span>
    <input
      type="file"
      accept=".ags,.txt,text/plain"
      class="text-xs text-fg-muted file:mr-2 file:rounded file:border-0 file:bg-chip file:px-2 file:py-1 file:text-fg-soft"
      onChange={(e) => props.onPick(e)}
    />
    <Show when={props.picked}>
      <span class="mono truncate text-xs text-accent">
        {props.picked!.name}
      </span>
    </Show>
  </label>
);

const DeltaView: Component<{
  delta: import("../../lib/validator").RevisionDelta;
}> = (props) => {
  const d = props.delta;
  const unchanged = () =>
    d.total_added === 0 &&
    d.total_removed === 0 &&
    d.total_changed === 0 &&
    d.groups_added.length === 0 &&
    d.groups_removed.length === 0;
  return (
    <Show
      when={!unchanged()}
      fallback={
        <div class="rounded-lg border border-line bg-surface p-6 text-sm text-fg-muted">
          No data differences — the two files are equivalent (any formatting
          differences are not data changes).
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <div class="flex flex-wrap items-center gap-3 text-xs">
          <span class="text-ok">+{d.total_added} rows added</span>
          <span class="text-err">−{d.total_removed} rows removed</span>
          <span class="text-warn">~{d.total_changed} rows changed</span>
        </div>

        <Show when={d.groups_added.length > 0}>
          <p class="text-xs text-ok">
            Groups only in revision: {d.groups_added.join(", ")}
          </p>
        </Show>
        <Show when={d.groups_removed.length > 0}>
          <p class="text-xs text-err">
            Groups only in baseline: {d.groups_removed.join(", ")}
          </p>
        </Show>

        <For each={d.groups}>{(g) => <GroupDeltaView g={g} />}</For>
      </div>
    </Show>
  );
};

const GroupDeltaView: Component<{ g: GroupDelta }> = (props) => {
  const g = props.g;
  const shown = () => g.rows.length;
  const total = () => g.added + g.removed + g.changed;
  return (
    <div class="rounded-lg border border-line bg-surface px-3 py-2">
      <div class="flex flex-wrap items-baseline gap-2 text-sm">
        <span class="mono font-medium text-fg">{g.code}</span>
        <Show
          when={g.keyed}
          fallback={
            <span class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-warn">
              unkeyed
            </span>
          }
        >
          <span class="text-xs text-fg-dim">
            matched by {g.key_headings.join(" + ")}
          </span>
        </Show>
        <span class="ml-auto text-xs">
          <span class="text-ok">+{g.added}</span>{" "}
          <span class="text-err">−{g.removed}</span>{" "}
          <span class="text-warn">~{g.changed}</span>
        </span>
      </div>

      <Show when={g.headings_added.length > 0 || g.headings_removed.length > 0}>
        <p class="mt-1 text-xs text-fg-dim">
          <Show when={g.headings_added.length > 0}>
            <span class="text-ok">+headings {g.headings_added.join(", ")}</span>{" "}
          </Show>
          <Show when={g.headings_removed.length > 0}>
            <span class="text-err">
              −headings {g.headings_removed.join(", ")}
            </span>
          </Show>
        </p>
      </Show>

      <div class="mono mt-2 flex flex-col gap-1 overflow-x-auto text-xs">
        <For each={g.rows}>{(r) => <RowDeltaView r={r} />}</For>
      </div>

      <Show when={shown() < total()}>
        <p class="mt-1 text-xs text-fg-dim italic">
          showing {shown()} of {total()} changed rows in this group.
        </p>
      </Show>
    </div>
  );
};

const RowDeltaView: Component<{ r: RowDelta }> = (props) => {
  const r = props.r;
  const band =
    r.kind === "added"
      ? "text-ok"
      : r.kind === "removed"
        ? "text-err"
        : "text-warn";
  const mark = r.kind === "added" ? "+" : r.kind === "removed" ? "−" : "~";
  return (
    <div class="min-w-max">
      <span class={`mr-2 select-none ${band}`}>{mark}</span>
      <span class="text-fg-soft">{r.key.join(" | ") || "(row)"}</span>
      <Show when={r.kind === "changed"}>
        <span class="ml-2 inline-flex flex-wrap gap-x-3">
          <For each={r.cells}>
            {(c) => (
              <span class="text-fg-dim">
                {c.heading}:{" "}
                <span class="text-err line-through">{c.a ?? "∅"}</span>{" "}
                <span class="text-ok">{c.b ?? "∅"}</span>
              </span>
            )}
          </For>
        </span>
      </Show>
    </div>
  );
};
