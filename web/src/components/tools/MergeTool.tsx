import {
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { mergeFiles, type MergeConversion } from "../../lib/validatorClient";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";
import { controlCompact } from "../../lib/controls";
import type { TypeClashMode } from "../../lib/validator";

// Merge: reconcile two AGS4 deliveries of one project into one file, in the
// engine (not in JS). Rows are matched by their dictionary KEY headings (a
// re-sorted borehole list still merges onto its prior self); a later file wins a
// KEY conflict; the merge is a union (a row in one file and absent in another is
// kept). A column the two files typed differently is settled by `onTypeClash`:
// error (refuse), widen (fall back to X — raw values kept, TYPE thrown away), or
// promote (keep the greatest nDP precision, zero-padding the coarser values). The
// browser tool merges two files; the CLI and the laterite/laterite-node libraries
// take N.

/** The three ways to settle a TYPE clash, in lattice order — least resolution
 *  first. Promote is listed before widen because it KEEPS the type; offering only
 *  the lossy way out is what pushes every clash toward `X`. */
const CLASH_MODES: { value: TypeClashMode; label: string; hint: string }[] = [
  {
    value: "error",
    label: "Refuse (default)",
    hint: "Reconciling two producers' declared types is high-stakes — merge will not guess.",
  },
  {
    value: "promote",
    label: "Keep the greatest precision",
    hint: "2DP + 5DP → 5DP, coarser values zero-padded (10.00 → 10.00000). No digit changes. Significant figures (nSF) fall back to X.",
  },
  {
    value: "widen",
    label: "Widen to X (text)",
    hint: "Every raw value is kept byte-for-byte, but the column's data type is thrown away.",
  },
];

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
  const [onTypeClash, setOnTypeClash] = createSignal<TypeClashMode>("error");
  const [issue, setIssue] = createSignal("");
  const [date, setDate] = createSignal("");
  const [producer, setProducer] = createSignal("");
  // All five, because all five are REQUIRED TRAN headings. The form used to
  // collect three and let the engine write the other two empty, which produced
  // a TRAN that failed Rule 10b on cells the user was never asked about.
  const [recipient, setRecipient] = createSignal("");
  const [status, setStatus] = createSignal("");

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
        ? {
            x,
            y,
            onTypeClash: onTypeClash(),
            issue: issue(),
            date: date(),
            producer: producer(),
            recipient: recipient(),
            status: status(),
          }
        : null;
    },
    ({ x, y, onTypeClash, issue, date, producer, recipient, status }) => {
      // All five or none: a partial stamp is rejected by the engine, so an
      // incomplete form means "no merge-TRAN" rather than an error the user
      // can't act on while they are still typing.
      const t = {
        issue: issue.trim(),
        date: date.trim(),
        producer: producer.trim(),
        recipient: recipient.trim(),
        status: status.trim(),
      };
      const tran = Object.values(t).every((v) => v) ? t : null;
      return mergeFiles(x.bytes, y.bytes, {
        encoding: "utf-8",
        onTypeClash,
        tran,
      });
    },
  );

  const pick = (set: (p: Picked) => void) => (e: Event) => {
    void (async () => {
      const f = (e.currentTarget as HTMLInputElement).files?.[0];
      if (f) set(await readFile(f));
    })();
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
        <FilePicker
          label="Incoming (b) — wins conflicts"
          picked={b()}
          onPick={pick(setB)}
        />
      </div>

      <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs text-fg-muted">
        <label class="flex items-center gap-1.5">
          <span class="text-fg-dim">
            If the files type a column differently:
          </span>
          <select
            class={controlCompact}
            value={onTypeClash()}
            onChange={(e) =>
              setOnTypeClash(e.currentTarget.value as TypeClashMode)
            }
          >
            <For each={CLASH_MODES}>
              {(m) => <option value={m.value}>{m.label}</option>}
            </For>
          </select>
        </label>
        <span class="flex items-center gap-1.5">
          <span class="text-fg-dim">
            Stamp a merge transmission (all five, or none):
          </span>
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
          <input
            class={`${controlCompact} w-24`}
            placeholder="recipient"
            value={recipient()}
            onInput={(e) => setRecipient(e.currentTarget.value)}
          />
          <input
            class={`${controlCompact} w-20`}
            placeholder="status"
            value={status()}
            onInput={(e) => setStatus(e.currentTarget.value)}
          />
        </span>
      </div>

      <p class="-mt-2 text-xs text-fg-dim">
        {CLASH_MODES.find((m) => m.value === onTypeClash())?.hint}
      </p>

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
            fallback={
              <MergeError
                error={result.error as unknown}
                mode={onTypeClash()}
                onChoose={setOnTypeClash}
              />
            }
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
      class="text-xs text-fg-muted file:mr-2 file:rounded-sm file:border-0 file:bg-chip file:px-2 file:py-1 file:text-fg-soft"
      onChange={(e) => {
        props.onPick(e);
      }}
    />
    <Show when={props.picked}>
      {(picked) => (
        <span class="mono truncate text-xs text-accent">{picked().name}</span>
      )}
    </Show>
  </label>
);

/** A TYPE clash offers a way out; a UNIT clash deliberately does NOT (no mode can
 *  absorb metres-vs-millimetres, and offering a button would send the user in a
 *  circle — see #501). So only offer the modes when the engine says "type conflict". */
const MergeError: Component<{
  error: unknown;
  mode: TypeClashMode;
  onChoose: (m: TypeClashMode) => void;
}> = (props) => {
  const isTypeClash = () =>
    String(props.error).toLowerCase().includes("type conflict");
  // Promote first — it is the option that keeps the column's type.
  const offers = () =>
    CLASH_MODES.filter((m) => m.value !== "error" && m.value !== props.mode);

  return (
    <div class="rounded-lg border border-err/45 bg-err-quiet p-4 text-sm">
      <p class="text-err">Could not merge: {String(props.error)}</p>
      <Show when={isTypeClash() && offers().length > 0}>
        <p class="mt-2 text-xs text-fg-muted">
          The two files declare a column with different data types. Settle it:
        </p>
        <ul class="mt-1.5 flex flex-col gap-1 text-xs text-fg-muted">
          <For each={offers()}>
            {(m) => (
              <li>
                <button
                  type="button"
                  class="text-accent underline hover:no-underline"
                  onClick={() => {
                    props.onChoose(m.value);
                  }}
                >
                  {m.label}
                </button>{" "}
                <span class="text-fg-dim">— {m.hint}</span>
              </li>
            )}
          </For>
        </ul>
        <p class="mt-1.5 text-xs text-fg-dim">
          …or reconcile the types in the source files and try again.
        </p>
      </Show>
    </div>
  );
};

const MergeView: Component<{
  result: MergeConversion;
  onDownload: () => void;
}> = (props) => {
  // MergeView is remounted per result — the <Show when={!result.loading}> above
  // unmounts it during a re-merge — so props.result never changes in place.
  // eslint-disable-next-line solid/reactivity
  const r = props.result;
  return (
    <div class="flex min-w-0 flex-col gap-3">
      <div class="flex flex-wrap items-center gap-3">
        <button
          type="button"
          class="rounded-md bg-cta px-3 py-1.5 text-sm font-medium text-fg-on-cta hover:bg-cta-hover"
          onClick={() => {
            props.onDownload();
          }}
        >
          Download merged (.ags)
        </button>
        <span class="text-xs text-fg-dim">
          {r.revisions.length} row revision{r.revisions.length === 1 ? "" : "s"}{" "}
          · {r.warnings.length} warning{r.warnings.length === 1 ? "" : "s"}
        </span>
      </div>

      <Show when={r.revisions.length > 0}>
        <div class="rounded-lg border border-line bg-surface px-3 py-2">
          <p class="mb-1 text-xs font-medium uppercase tracking-wide text-fg-dim">
            Rows the incoming file revised
          </p>
          {/* scroll-region (#407): one line per revised row is unbounded on a
              big merge — the list scrolls inside its cap. */}
          <div class="scroll-region mono flex flex-col gap-1 text-xs">
            <For each={r.revisions}>
              {(rev) => (
                <div class="min-w-max">
                  <span class="text-warn">~</span>{" "}
                  <span class="text-fg">{rev.group}</span>{" "}
                  <span class="text-fg-soft">
                    {rev.key.join(" | ") || "(row)"}
                  </span>
                  <span class="ml-2 text-fg-dim">
                    changed {rev.changed.join(", ")}
                  </span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      <Show when={r.warnings.length > 0}>
        <div class="rounded-lg border border-line bg-surface px-3 py-2">
          <p class="mb-1 text-xs font-medium uppercase tracking-wide text-fg-dim">
            Warnings
          </p>
          <ul class="scroll-region flex flex-col gap-1 text-xs text-fg-muted">
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
