import { For, Show, createMemo, type Component } from "solid-js";
import { diffLines, toHunks, type DiffRow } from "../../lib/linediff";

// The Fix tab's audit trail: a unified diff of the originally-loaded file
// against the file as it now stands (after any applied fixes / hand-edits).
// Raw lines, not column-aligned — alignment is a per-GROUP-block concept, and
// a whole-file diff spans many groups; the per-fix preview is where the
// aligned in-context view lives. Unchanged runs collapse to a "⋯ N" marker so
// the view is the changes plus a few lines of context, nothing more.

const decode = (b: Uint8Array): string =>
  new TextDecoder("utf-8", { fatal: false }).decode(b);

// Safety net: even after hunking, a wholesale rewrite could produce a huge
// row list. Cap what we mount and say so (the engine's fixes never approach
// this — it guards a pathological hand-edited file).
const MAX_ROWS = 4000;

export const FileDiff: Component<{
  /** the originally-loaded baseline bytes (fileStore.originalBytes). */
  a: () => Uint8Array | null;
  /** the current bytes the engine sees (fileStore.canonicalBytes). */
  b: () => Uint8Array | null;
}> = (props) => {
  const result = createMemo(() => {
    const ab = props.a();
    const bb = props.b();
    if (!ab || !bb) return null;
    const aLines = decode(ab).split(/\r?\n/);
    const bLines = decode(bb).split(/\r?\n/);
    return diffLines(aLines, bLines);
  });

  const rows = createMemo<DiffRow[]>(() => {
    const r = result();
    return r ? toHunks(r.ops, 3) : [];
  });

  const shown = createMemo(() => rows().slice(0, MAX_ROWS));
  const overflow = createMemo(() => Math.max(0, rows().length - MAX_ROWS));

  return (
    <Show
      when={result()}
      fallback={
        <p class="text-sm text-fg-muted">Load a file to see its diff.</p>
      }
    >
      {(r) => (
        <Show
          when={r().added > 0 || r().removed > 0}
          fallback={
            <div class="rounded-lg border border-line bg-surface p-6 text-sm text-fg-muted">
              No changes — the current file is identical to the original.
            </div>
          }
        >
          <div class="flex min-w-0 flex-col gap-2">
            <div class="flex flex-wrap items-center gap-3 text-xs">
              <span class="text-ok">+{r().added} added</span>
              <span class="text-err">−{r().removed} removed</span>
              <span class="text-fg-dim">vs. the originally-loaded file</span>
              <Show when={r().capped}>
                <span class="rounded-xs bg-warn-quiet px-1.5 py-0.5 text-warn">
                  large change — shown as a block replace
                </span>
              </Show>
            </div>

            <pre class="mono max-w-full overflow-x-auto rounded-lg border border-line bg-surface-code p-2 text-xs leading-relaxed">
              <For each={shown()}>{(row) => <DiffRowView row={row} />}</For>
            </pre>

            <Show when={overflow() > 0}>
              <p class="text-xs text-fg-dim italic">
                +{overflow()} more diff rows not shown.
              </p>
            </Show>
          </div>
        </Show>
      )}
    </Show>
  );
};

const GUTTER = "mr-2 inline-block w-10 select-none text-right text-fg-dim";

const DiffRowView: Component<{ row: DiffRow }> = (props) => {
  // Rendered under <For> (value-keyed), so props.row is stable for this
  // instance's lifetime: the snapshot never goes stale and the type-based early
  // return can't strand a later branch (re-renders come from For, not a prop
  // update in place).
  // eslint-disable-next-line solid/reactivity
  const row = props.row;
  if (row.type === "gap") {
    // eslint-disable-next-line solid/components-return-once
    return (
      <div class="min-w-max text-fg-faint italic">
        <span class={GUTTER}>⋯</span>
        {row.count} unchanged line{row.count === 1 ? "" : "s"}
      </div>
    );
  }
  const band =
    row.type === "del"
      ? "bg-err-quiet text-err"
      : row.type === "ins"
        ? "bg-ok-quiet text-ok"
        : "text-fg-muted";
  const mark = row.type === "del" ? "−" : row.type === "ins" ? "+" : " ";
  return (
    <div class={`min-w-max ${band}`}>
      <span class={GUTTER}>{row.aLine ?? ""}</span>
      <span class={GUTTER}>{row.bLine ?? ""}</span>
      <span class="mr-2 select-none">{mark}</span>
      {row.text}
    </div>
  );
};
