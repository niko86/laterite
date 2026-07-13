import { createResource, createSignal, For, Show, type Component } from "solid-js";
import { mergeFiles, type MergeConversion } from "../../lib/validatorClient";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";
import { controlCompact } from "../../lib/controls";

// Merge: reconcile two AGS4 deliveries of one project into one file, in the
// engine (not in JS). Rows are matched by their dictionary KEY headings (a
// re-sorted borehole list still merges onto its prior self); a later file wins a
// KEY conflict; the merge is a union (a row in one file and absent in another is
// kept). A column the two files typed differently is a conflict — strict rejects
// it, lenient widens it to X (text). The browser tool merges two files; the CLI
// and the laterite/laterite-node libraries take N.

interface Picked {
  name: string;
  bytes: Uint8Array;
}

async function readFile(f: File): Promise<Picked> {
  return { name: f.name, bytes: new Uint8Array(await f.arrayBuffer()) };
}

export const MergeTool: Component = () => {
  const [a, setA] = createSignal<Picked | null>(null);
  const [b, setB] = createSignal<Picked | null>(null);
  const [lenient, setLenient] = createSignal(false);
  const [issue, setIssue] = createSignal("");
  const [date, setDate] = createSignal("");
  const [producer, setProducer] = createSignal("");

  // Seed the baseline with the file already loaded in the app — the common case
  // is "merge an incoming delivery into what I'm working on".
  const loaded = fileStore.bytes();
  if (loaded) setA({ name: fileStore.name() || "loaded file", bytes: loaded });

  const [result] = createResource(
    () => {
      const x = a();
      const y = b();
      // Re-run when the files or any option change.
      return x && y
        ? { x, y, lenient: lenient(), issue: issue(), date: date(), producer: producer() }
        : null;
    },
    ({ x, y, lenient, issue, date, producer }) =>
      mergeFiles(x.bytes, y.bytes, {
        encoding: "utf-8",
        lenient,
        // A merge-TRAN is stamped only when both an issue and a date are given.
        tranIssue: issue.trim() || null,
        tranDate: date.trim() || null,
        tranProducer: producer.trim() || null,
      }),
  );

  const pick = (set: (p: Picked) => void) => async (e: Event) => {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) set(await readFile(f));
  };

  const download = () => {
    const r = result();
    if (!r) return;
    downloadBlob(r.bytes, `${baseName(a()?.name)}.merged.ags`, "text/plain");
  };

  return (
    <div class="flex min-w-0 flex-col gap-4">
      <p class="text-sm text-fg-soft">
        Merge two AGS4 deliveries of one project into a single file. Rows are
        matched by their dictionary <span class="mono text-fg">KEY</span>{" "}
        headings and reconciled in the engine — the second file wins a key
        conflict, and a row present in only one file is kept (the merge is a
        union, not an intersection). Both files stay in your browser.
      </p>

      <div class="grid gap-3 sm:grid-cols-2">
        <FilePicker label="Base (a)" picked={a()} onPick={pick(setA)} />
        <FilePicker label="Incoming (b) — wins conflicts" picked={b()} onPick={pick(setB)} />
      </div>

      <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs text-fg-muted">
        <label class="flex items-center gap-1.5">
          <input
            type="checkbox"
            checked={lenient()}
            onChange={(e) => setLenient(e.currentTarget.checked)}
          />
          Widen conflicting column types to <span class="mono">X</span> (lenient)
        </label>
        <span class="flex items-center gap-1.5">
          <span class="text-fg-dim">Stamp a merge transmission (optional):</span>
          <input
            class={`${controlCompact} w-16`}
            placeholder="issue"
            value={issue()}
            onInput={(e) => setIssue(e.currentTarget.value)}
          />
          <input
            class={`${controlCompact} w-28`}
            placeholder="yyyy-mm-dd"
            value={date()}
            onInput={(e) => setDate(e.currentTarget.value)}
          />
          <input
            class={`${controlCompact} w-24`}
            placeholder="producer"
            value={producer()}
            onInput={(e) => setProducer(e.currentTarget.value)}
          />
        </span>
      </div>

      <Show
        when={a() && b()}
        fallback={
          <p class="text-sm text-fg-muted">Choose both files to merge them.</p>
        }
      >
        <Show
          when={!result.loading}
          fallback={<p class="text-sm text-fg-muted">Merging…</p>}
        >
          <Show
            when={!result.error}
            fallback={<MergeError error={result.error} onLenient={() => setLenient(true)} lenient={lenient()} />}
          >
            <Show when={result()}>
              {(r) => <MergeView result={r()} onDownload={download} />}
            </Show>
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
      <span class="mono truncate text-xs text-accent">{props.picked!.name}</span>
    </Show>
  </label>
);

const MergeError: Component<{ error: unknown; lenient: boolean; onLenient: () => void }> = (
  props,
) => (
  <div class="rounded-lg border border-err/40 bg-err/5 p-4 text-sm">
    <p class="text-err">Could not merge: {String(props.error)}</p>
    <Show when={!props.lenient && String(props.error).toLowerCase().includes("type conflict")}>
      <p class="mt-2 text-xs text-fg-muted">
        The two files declare a column with different data types.{" "}
        <button
          type="button"
          class="text-accent underline hover:no-underline"
          onClick={() => props.onLenient()}
        >
          Widen it to X (lenient)
        </button>{" "}
        to keep every value, or reconcile the types and try again.
      </p>
    </Show>
  </div>
);

const MergeView: Component<{ result: MergeConversion; onDownload: () => void }> = (props) => {
  const r = props.result;
  return (
    <div class="flex min-w-0 flex-col gap-3">
      <div class="flex flex-wrap items-center gap-3">
        <button
          type="button"
          class="rounded bg-emerald-600/80 px-3 py-1.5 text-sm font-medium text-emerald-50 hover:bg-emerald-600"
          onClick={() => props.onDownload()}
        >
          Download merged (.ags)
        </button>
        <span class="text-xs text-fg-dim">
          {r.revisions.length} row revision{r.revisions.length === 1 ? "" : "s"} ·{" "}
          {r.warnings.length} warning{r.warnings.length === 1 ? "" : "s"}
        </span>
      </div>

      <Show when={r.revisions.length > 0}>
        <div class="rounded-lg border border-line bg-surface px-3 py-2">
          <p class="mb-1 text-xs font-medium uppercase tracking-wide text-fg-dim">
            Rows the incoming file revised
          </p>
          <div class="mono flex flex-col gap-1 overflow-x-auto text-xs">
            <For each={r.revisions}>
              {(rev) => (
                <div class="min-w-max">
                  <span class="text-warn">~</span>{" "}
                  <span class="text-fg">{rev.group}</span>{" "}
                  <span class="text-fg-soft">{rev.key.join(" | ") || "(row)"}</span>
                  <span class="ml-2 text-fg-dim">changed {rev.changed.join(", ")}</span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      <Show when={r.warnings.length > 0}>
        <div class="rounded-lg border border-line bg-surface px-3 py-2">
          <p class="mb-1 text-xs font-medium uppercase tracking-wide text-fg-dim">Warnings</p>
          <ul class="flex flex-col gap-1 text-xs text-fg-muted">
            <For each={r.warnings}>
              {(w) => (
                <li>
                  <span class="mono text-warn">[{w.kind}]</span>{" "}
                  <Show when={w.group}>
                    <span class="mono text-fg-soft">{w.group}</span>{" "}
                  </Show>
                  {w.message}
                </li>
              )}
            </For>
          </ul>
        </div>
      </Show>

      <Show when={r.revisions.length === 0 && r.warnings.length === 0}>
        <p class="rounded-lg border border-line bg-surface p-4 text-sm text-fg-muted">
          Clean merge — every shared row agreed and nothing needed widening. The
          merged file is ready to download.
        </p>
      </Show>
    </div>
  );
};
