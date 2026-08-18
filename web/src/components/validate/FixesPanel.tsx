import { For, Show, createMemo, type Component, type JSX } from "solid-js";
import type { Fix, SpanEdit } from "../../lib/validator";
import type { Severity } from "./FilterBar";
import { shortRule, ruleAnchor } from "../../lib/rules";
import { Chip, type ChipTone, type ChipVariant } from "@shared/components";

// A fix wears the SAME chip as the finding it resolves — the map is imported,
// not copied, so the two cannot drift. Until #412 these were three tints of one
// form here, so the tiers were told apart by colour ALONE, while the findings
// list next door had already moved to form (#404). `unlabelled` is the fourth
// state: no report to join against, so no severity is known — see
// lib/fixSeverity for why that is distinct from a rule the report didn't raise.
const chipFor = (
  s: Severity | undefined,
): { tone: ChipTone; variant: ChipVariant } =>
  s ? SEVERITY_CHIP[s] : { tone: "muted", variant: "outline" };
import { highlightSpan, SEVERITY_CHIP } from "./FindingsView";
import { fixBlock, alignBlock, type AlignedRow } from "../../lib/agsline";
import { fixHighlight } from "../../lib/fixpreview";

/** A fix key the parent's selected-set keys off. There's at most one fix
 *  of a byte-level kind, and per-line in-line fixes, so kind+rule+line+
 *  first-span uniquely identifies one (stable across recomputes for the
 *  same file). */
export function fixKey(f: Fix): string {
  const e = f.edits[0];
  const span = e ? `${e.line}:${e.start}-${e.end}` : "doc";
  return `${f.kind}|${f.rule}|${f.line ?? "-"}|${span}`;
}

/** Splice `replacement` into the [start, end) char range of `raw`,
 *  returning the AFTER line with the replacement region highlighted.
 *  Code-point correct (via Array.from), matching the engine's slicing. */
function afterLine(raw: string, edit: SpanEdit): JSX.Element {
  const cps = Array.from(raw);
  const s = Math.max(0, Math.min(edit.start, cps.length));
  const e = Math.max(s, Math.min(edit.end, cps.length));
  const repl = Array.from(edit.replacement);
  const before = cps.slice(0, s).join("");
  const after = cps.slice(e).join("");
  return (
    <>
      {before}
      <span class="rounded-sm bg-ok text-surface">{repl.join("")}</span>
      {after}
    </>
  );
}

/** Per-edit before/after diff for one in-line fix. */
const EditDiff: Component<{ edit: SpanEdit; lines: () => string[] }> = (
  props,
) => {
  const raw = createMemo(() => props.lines()[props.edit.line - 1] ?? "");
  return (
    <div class="mt-1 space-y-0.5">
      <div class="flex items-start gap-2">
        <span class="select-none text-xs text-err">−</span>
        <span class="min-w-0 break-all">
          {highlightSpan(raw(), props.edit.start, props.edit.end)}
        </span>
      </div>
      <div class="flex items-start gap-2">
        <span class="select-none text-xs text-ok">+</span>
        <span class="min-w-0 break-all">{afterLine(raw(), props.edit)}</span>
      </div>
    </div>
  );
};

const GUTTER = "mr-2 inline-block w-10 select-none text-right text-fg-dim";

/** Render one aligned fix-preview row: a before (`del`) / after (`ins`) pair
 *  inside the enclosing GROUP block. The changed cell highlights the EXACT
 *  changed character(s) (cell-relative span), not the whole cell — so a
 *  one-char edit in a long text field is pinpointed. `delHl`/`insHl` are the
 *  [start,end) ranges within the changed cell for the del / ins variant. */
function fixRow(
  row: AlignedRow,
  changedCol: number,
  delHl: [number, number] | null,
  insHl: [number, number] | null,
  appendFrom: number | null,
): JSX.Element {
  if (row.ellipsis !== undefined) {
    return (
      <div class="min-w-max">
        <span class={GUTTER}>⋯</span>
        <span class="select-none text-fg-faint italic">
          {row.ellipsis} more data row{row.ellipsis === 1 ? "" : "s"}
        </span>
      </div>
    );
  }
  const band =
    row.variant === "del"
      ? "bg-err-quiet"
      : row.variant === "ins"
        ? "bg-ok-quiet"
        : "";
  const mark = row.variant === "del" ? "−" : row.variant === "ins" ? "+" : " ";
  const hl =
    row.variant === "del" ? delHl : row.variant === "ins" ? insHl : null;
  return (
    <div class={`min-w-max ${band}`}>
      <span class={GUTTER}>{row.n}</span>
      <span class="mr-2 select-none text-fg-faint">{mark}</span>
      <For each={row.cells}>
        {(c, i) => {
          // <For> over positional cells (stable index); hl/changedCol/appendFrom
          // are constant for this render, so these i() once-reads stay correct.
          /* eslint-disable solid/reactivity */
          if (hl && i() === changedCol)
            return highlightSpan(c.padded, hl[0], hl[1]);
          // Row-padding appends whole new cells (no original to sub-span) —
          // highlight them wholesale on the ins side so the additions are visible.
          if (row.variant === "ins" && appendFrom !== null && i() >= appendFrom)
            return (
              <span class="rounded-sm bg-ok text-surface">{c.padded}</span>
            );
          /* eslint-enable solid/reactivity */
          return c.padded;
        }}
      </For>
    </div>
  );
}

/** Per-edit diff shown as the aligned enclosing GROUP block (headers + nearby
 *  data rows for column context, the changed row as a del/ins pair). Falls
 *  back to the single-line {@link EditDiff} when there's no enclosing GROUP
 *  block (so the edit still previews). */
const FixBlockDiff: Component<{ edit: SpanEdit; lines: () => string[] }> = (
  props,
) => {
  // All preview geometry (spliced AFTER line, changed field, cell-relative
  // del/ins highlight spans, append flag) in one unit-tested pure helper.
  const hl = createMemo(() =>
    fixHighlight(props.lines()[props.edit.line - 1] ?? "", props.edit),
  );
  const aligned = createMemo(() => {
    const block = fixBlock(props.lines(), props.edit.line, hl().after);
    return block ? alignBlock(block) : null;
  });
  return (
    <Show
      when={aligned()}
      fallback={<EditDiff edit={props.edit} lines={props.lines} />}
    >
      {(ab) => (
        <For each={ab().rows}>
          {(row) =>
            fixRow(
              row,
              hl().changedCol,
              hl().delHl,
              hl().insHl,
              hl().appendFrom,
            )
          }
        </For>
      )}
    </Show>
  );
};

/** Apply Fixes panel: lists every computed safe fix with a checkbox +
 *  per-fix diff preview, an "Apply selected" button (re-validation +
 *  re-compute happen reactively since the parent's report/fixes memos
 *  derive from bytes), and an "Export" download. */
export const FixesPanel: Component<{
  fixes: () => Fix[];
  text: () => string;
  /** the selected-fix keys (see fixKey); a fix is applied iff selected. */
  selected: () => Set<string>;
  onToggle: (key: string) => void;
  onApply: (fixes: Fix[]) => void;
  /** When true, preview each fix as its aligned enclosing GROUP block. */
  aligned?: () => boolean;
  /** Optional: the severity of the finding each fix resolves → renders a badge
   *  so it's clear a fix touches an FYI-only finding (hidden on Validate).
   *  Returns `undefined` when the resolver has no report to join against — the
   *  fix stands, its label doesn't, and the badge says so rather than guessing
   *  (#412). */
  severityOf?: (f: Fix) => Severity | undefined;
}> = (props) => {
  const lines = createMemo(() => props.text().split(/\r?\n/));

  // Order by anchor line (whole-file kinds — null line — first), so the
  // list reads top-to-bottom like the file.
  const ordered = createMemo(() =>
    [...props.fixes()].sort(
      (a, b) => (a.line ?? -1) - (b.line ?? -1) || a.kind.localeCompare(b.kind),
    ),
  );
  // Split safe (bulk-applicable) from risky (opt-in) — rendered as two
  // labelled groups; the risky group carries a warning + is unticked by
  // default (the parent seeds `selected` with safe keys only).
  const safeOrdered = createMemo(() =>
    ordered().filter((f) => f.risk !== "risky"),
  );
  const riskyOrdered = createMemo(() =>
    ordered().filter((f) => f.risk === "risky"),
  );

  const selectedFixes = createMemo(() =>
    ordered().filter((f) => props.selected().has(fixKey(f))),
  );

  // One fix card (checkbox + label + rule link + per-edit diff). Shared by
  // the safe + risky groups.
  const fixCard = (f: Fix): JSX.Element => {
    const key = fixKey(f);
    const byteLevel = f.edits.length === 0;
    return (
      <div class="rounded-lg border border-line bg-surface px-3 py-2 text-sm">
        <label class="flex cursor-pointer items-start gap-2">
          <input
            type="checkbox"
            class="mt-1"
            checked={props.selected().has(key)}
            onChange={() => {
              props.onToggle(key);
            }}
          />
          <span class="min-w-0 flex-1">
            <span class="text-fg">{f.label}</span>
            <a
              href={`#${ruleAnchor(f.rule)}`}
              class="ml-2 rounded-sm bg-chip px-1.5 py-0.5 text-xs text-fg-soft hover:text-fg"
            >
              {shortRule(f.rule)}
            </a>
            <Show when={props.severityOf}>
              {(severityOf) => (
                <Chip class="ml-1.5" {...chipFor(severityOf()(f))}>
                  {severityOf()(f) ?? "unlabelled"}
                </Chip>
              )}
            </Show>
          </span>
        </label>
        <Show
          when={!byteLevel}
          fallback={
            <p class="mt-1 pl-6 text-xs text-fg-faint italic">
              Whole-file change (no per-line diff).
            </p>
          }
        >
          <pre class="mono mt-2 max-w-full overflow-x-auto rounded-sm bg-surface-code p-2 text-xs leading-relaxed text-fg-soft">
            <For each={f.edits}>
              {(edit) =>
                props.aligned?.() ? (
                  <FixBlockDiff edit={edit} lines={lines} />
                ) : (
                  <EditDiff edit={edit} lines={lines} />
                )
              }
            </For>
          </pre>
        </Show>
      </div>
    );
  };

  return (
    <Show
      when={props.fixes().length > 0}
      fallback={
        <div class="rounded-lg border border-line bg-surface p-6 text-sm text-fg-muted">
          No automatic fixes available for this file.
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <div class="flex flex-wrap items-center gap-3">
          <button
            type="button"
            class="rounded-md bg-cta px-3 py-1.5 text-sm font-medium text-fg-on-cta hover:bg-cta-hover disabled:opacity-45"
            disabled={selectedFixes().length === 0}
            onClick={() => {
              props.onApply(selectedFixes());
            }}
          >
            Apply selected ({selectedFixes().length})
          </button>
          <span class="text-xs text-fg-faint">
            {props.fixes().length} fix{props.fixes().length === 1 ? "" : "es"}{" "}
            available
          </span>
        </div>

        <For each={safeOrdered()}>{(f) => fixCard(f)}</For>

        <Show when={riskyOrdered().length > 0}>
          <div class="rounded-lg border border-warn/45 bg-warn-quiet px-3 py-2">
            <p class="text-sm font-medium text-warn">
              Risky fixes ({riskyOrdered().length}) — opt-in
            </p>
            <p class="mt-0.5 text-xs text-fg-dim">
              These guess intent (a lossy or surprising rewrite) and are{" "}
              <span class="font-medium">excluded from "Fix all safe"</span>.
              Review and tick the ones you want, then Apply selected.
            </p>
            <div class="mt-2 flex flex-col gap-2">
              <For each={riskyOrdered()}>{(f) => fixCard(f)}</For>
            </div>
          </div>
        </Show>
      </div>
    </Show>
  );
};
