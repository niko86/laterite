import {
  For,
  Show,
  createEffect,
  createMemo,
  onMount,
  type Component,
  type JSX,
} from "solid-js";
import { createVirtualizer } from "@tanstack/solid-virtual";
import type {
  FindingDto,
  Severity,
  ValidationReport,
} from "../../lib/validator";
import { severityOf } from "../../lib/validator";
import { ruleAnchor, shortRule } from "../../lib/rules";
import { Chevron } from "../Chevron";
import { Chip, type ChipTone, type ChipVariant } from "@shared/components";
import {
  splitAgsFields,
  groupBlock,
  alignBlock,
  type AlignedRow,
} from "../../lib/agsline";

// Master's rich per-finding renderer (highlighting / severity bands /
// aligned columns) flattened into one windowed list: a pathologically
// dirty file concentrates findings in a *few* rules, so even one open
// group can be tens of thousands of rows (after the wasm per-rule cap) —
// we flatten every group header + its (open) findings into one row array
// and mount only the ~visible slice via @tanstack/solid-virtual. A
// collapsed group contributes only its header, so its findings never
// enter the array AND never mount (the lazy-render win, preserved).

/** ±3 lines of context around a 1-based source line. The fallback when
 *  there's no enclosing GROUP block (so no header/column context to show);
 *  ±3 (not ±1) so even the fallback gives a few rows above and below. */
function snippet(
  lines: string[],
  line: number,
): { n: number; text: string; hit: boolean }[] {
  const from = Math.max(1, line - 3);
  const to = Math.min(lines.length, line + 3);
  const out: { n: number; text: string; hit: boolean }[] = [];
  for (let n = from; n <= to; n++) {
    out.push({ n, text: lines[n - 1] ?? "", hit: n === line });
  }
  return out;
}

/** Hit-row band background — the status tier's quiet wash. Colour is
 *  supplementary here: the tier itself is stated by the finding's form-encoded
 *  chip (see SEVERITY_CHIP), so the list still reads in greyscale.
 *
 *  This used to take the raw wire field and fall back to the warning tint when
 *  it was absent — which painted every error in the warning colour, because absent is
 *  exactly what an error looks like on the wire. Taking `Severity` makes the
 *  switch exhaustive, so there is no fallback arm left to be wrong. */
function severityBand(severity: Severity): string {
  switch (severity) {
    case "error":
      return "bg-err-quiet";
    case "warning":
      return "bg-warn-quiet";
    case "fyi":
      return "bg-info-quiet";
  }
}

/** Char-level hit mark: a SOLID fill in the tier's status colour. Solid, not
 *  a wash — it sits inside a row already banded with the tier's quiet wash, so
 *  it has to be the loudest object in the block. `text-surface` rather than a
 *  fixed light foreground: the status colours lighten in dark, so the on-fill
 *  text flips to the dark surface with the theme. */
function severityMark(severity: Severity): string {
  switch (severity) {
    case "error":
      return "bg-err text-surface";
    case "warning":
      return "bg-warn text-surface";
    case "fyi":
      return "bg-info text-surface";
  }
}

/** Severity stated in FORM as well as hue (#404): error is a solid fill,
 *  warning a 3px stratum tick, fyi a hairline stencil — three shapes the Chip
 *  primitive already carries, so the tiers stay apart in greyscale. */
/** Exported so FixesPanel badges a fix with the SAME chip the finding it
 *  resolves wears (#412). Duplicating it there let the two drift apart on the
 *  next tone tweak, which is exactly the mismatch a shared badge exists to
 *  prevent. */
export const SEVERITY_CHIP: Record<
  Severity,
  { tone: ChipTone; variant: ChipVariant }
> = {
  error: { tone: "err", variant: "solid" },
  warning: { tone: "warn", variant: "rule" },
  fyi: { tone: "info", variant: "outline" },
};

/** Wrap a `[start, end)` code-point sub-range of `raw` in a strong
 *  highlight, leaving the surrounding text plain. Slicing is code-point
 *  correct (via Array.from) so a multibyte line can't split a char.
 *  `mark` defaults to the error tier's solid fill — the diff previews reuse
 *  this for their del side, where err is the honest tone. */
export function highlightSpan(
  raw: string,
  start: number,
  end: number,
  mark: string = severityMark("error"),
): JSX.Element {
  const cps = Array.from(raw);
  const s = Math.max(0, Math.min(start, cps.length));
  const e = Math.max(s, Math.min(end, cps.length));
  return (
    <>
      {cps.slice(0, s).join("")}
      <span class={`rounded-sm ${mark}`}>{cps.slice(s, e).join("")}</span>
      {cps.slice(e).join("")}
    </>
  );
}

/** Render the hit line. Precedence:
 *  1. `char_span` (Rules 1/6, or the wasm-injected field span) → slice the
 *     raw line by those char offsets and wrap exactly that range.
 *  2. else `field_index` → wrap the targeted field's INNER value (between
 *     the quotes, no trailing comma), not the whole token — the bug fix.
 *  3. else → plain raw.
 *  `field_index` is tag-stripped, so the raw on-line field is `+1`. */
function renderLine(raw: string, f: FindingDto): JSX.Element {
  const mark = severityMark(severityOf(f));
  // char_span supersedes field_index.
  if (f.char_span) {
    return highlightSpan(raw, f.char_span[0], f.char_span[1], mark);
  }

  const targeted =
    (f.target === "heading" || f.target === "cell" || f.target === "group") &&
    f.field_index != null;
  if (!targeted) return raw;

  const fields = splitAgsFields(raw);
  const idx = (f.field_index as number) + 1; // +1 skips the leading tag.
  if (idx < 0 || idx >= fields.length) return raw; // out-of-range fallback.

  // Highlight the field's inner value (no quotes, no comma) — fixes the
  // over-wide token highlight that lit `"ERES_LAB",` comma and all.
  const field = fields[idx]; // idx bounds checked above → in-bounds.
  if (!field) return raw;
  return highlightSpan(raw, field.valueStart, field.valueEnd, mark);
}

/** Render one aligned-columns row. The hit row highlights the targeted
 *  padded cell — `char_span` doesn't survive alignment (it indexes the raw
 *  line), so in aligned mode we fall back to the field's inner value within
 *  its padded cell (still the precise sub-range), keyed off `field_index`. */
function renderAlignedRow(row: AlignedRow, f: FindingDto): JSX.Element {
  if (!row.hit) {
    return <>{row.cells.map((c) => c.padded).join("")}</>;
  }
  const mark = severityMark(severityOf(f));
  const idx =
    (f.target === "heading" || f.target === "cell" || f.target === "group") &&
    f.field_index != null
      ? f.field_index + 1 // +1 skips the leading tag.
      : -1;
  return (
    <>
      <For each={row.cells}>
        {(c, i) =>
          // eslint-disable-next-line solid/reactivity -- <For> over positional cells that never reorder + idx is constant, so i()'s once-read stays correct
          i() === idx
            ? highlightSpan(c.padded, c.valueStart, c.valueEnd, mark)
            : c.padded
        }
      </For>
    </>
  );
}

const FindingRow: Component<{
  f: FindingDto;
  lines: () => string[];
  aligned: () => boolean;
}> = (props) => {
  // The enclosing GROUP block — the GROUP/HEADING/UNIT/TYPE rows plus a
  // windowed set of data rows around the hit — gives every finding its
  // structural context (so positional-CSV misalignment is eyeballable).
  // Shown in BOTH raw and aligned modes; the `aligned` toggle only controls
  // whether columns are space-padded. null ⇒ no enclosing GROUP (or past the
  // scan cap) ⇒ the ±3 raw snippet fallback.
  const block = createMemo(() =>
    props.f.line == null ? null : groupBlock(props.lines(), props.f.line),
  );
  const alignedBlock = createMemo(() => {
    const b = block();
    return b && props.aligned() ? alignBlock(b) : null;
  });
  const severity = () => severityOf(props.f);
  return (
    <div class="min-w-0 border-t border-line px-3 py-2 text-sm">
      <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <Chip
          tone={SEVERITY_CHIP[severity()].tone}
          variant={SEVERITY_CHIP[severity()].variant}
        >
          {severity()}
        </Chip>
        <span class="text-fg-faint">
          {props.f.line != null ? `line ${props.f.line}` : "—"}
        </span>
        <span class="rounded-xs bg-chip px-1.5 py-0.5 text-xs text-fg-soft">
          {props.f.group || "—"}
        </span>
        <span class="text-fg">{props.f.desc}</span>
      </div>
      <Show when={props.f.line != null}>
        <pre class="mono mt-2 max-w-full overflow-x-auto rounded-sm bg-surface-code p-2 text-xs leading-relaxed text-fg-muted">
          <Show
            when={alignedBlock()}
            fallback={
              // No alignment: render the same GROUP block as raw rows when we
              // have one (headers + windowed data + ellipsis markers), else
              // the ±3 raw snippet.
              <Show
                when={block()}
                fallback={
                  <For each={snippet(props.lines(), props.f.line ?? 0)}>
                    {(row) => (
                      // min-w-max so the row-band background spans the full
                      // scrolled line width inside the overflow-x-auto <pre>,
                      // not just the visible viewport.
                      <div
                        class="min-w-max"
                        classList={{
                          [severityBand(severityOf(props.f))]: row.hit,
                        }}
                      >
                        <span class="mr-3 inline-block w-10 select-none text-right text-fg-dim">
                          {row.n}
                        </span>
                        {row.hit ? renderLine(row.text, props.f) : row.text}
                      </div>
                    )}
                  </For>
                }
              >
                {(b) => (
                  <For each={b().rows}>
                    {(row) => (
                      <div
                        class="min-w-max"
                        classList={{
                          [severityBand(severityOf(props.f))]: row.hit,
                        }}
                      >
                        <span class="mr-3 inline-block w-10 select-none text-right text-fg-dim">
                          {row.ellipsis !== undefined ? "⋯" : row.n}
                        </span>
                        {row.ellipsis !== undefined ? (
                          <span class="select-none text-fg-faint italic">
                            {row.ellipsis} more data row
                            {row.ellipsis === 1 ? "" : "s"}
                          </span>
                        ) : row.hit ? (
                          renderLine(row.raw, props.f)
                        ) : (
                          row.raw
                        )}
                      </div>
                    )}
                  </For>
                )}
              </Show>
            }
          >
            {(ab) => (
              <For each={ab().rows}>
                {(row) => (
                  <div
                    class="min-w-max"
                    classList={{
                      [severityBand(severityOf(props.f))]: row.hit,
                    }}
                  >
                    <span class="mr-3 inline-block w-10 select-none text-right text-fg-dim">
                      {row.ellipsis !== undefined ? "⋯" : row.n}
                    </span>
                    {row.ellipsis !== undefined ? (
                      <span class="select-none text-fg-faint italic">
                        {row.ellipsis} more data row
                        {row.ellipsis === 1 ? "" : "s"}
                      </span>
                    ) : (
                      renderAlignedRow(row, props.f)
                    )}
                  </div>
                )}
              </For>
            )}
          </Show>
        </pre>
      </Show>
    </div>
  );
};

// Flattened virtual-row model.
type HeaderRow = {
  kind: "header";
  rule: string;
  count: number;
  open: boolean;
};
type FindingRowData = { kind: "finding"; f: FindingDto };
type Row = HeaderRow | FindingRowData;

export const FindingsView: Component<{
  // The FILTERED groups to render (filtering happens in ValidatePane).
  report: ValidationReport;
  text: () => string;
  aligned: () => boolean;
  // Open-state is owned by ValidatePane so the FilterBar jump can force a
  // group open. `isOpen` reads it; `onToggle` flips one rule; the
  // expand/collapse-all setters take the full rule list themselves.
  isOpen: (rule: string) => boolean;
  onToggle: (rule: string) => void;
  onExpandAll: () => void;
  onCollapseAll: () => void;
  /** Register an index-based scroll so the FilterBar jump can reach a rule
   *  even when its header is windowed out of the DOM. */
  registerJump?: (fn: (rule: string) => void) => void;
}> = (props) => {
  const lines = createMemo(() => props.text().split(/\r?\n/));

  // Flatten respecting the parent-owned open-state; a closed group
  // contributes only its header row. Also build rule → header-index for
  // the jump-to action.
  const model = createMemo(() => {
    const rows: Row[] = [];
    const headerIndex = new Map<string, number>();
    for (const g of props.report.findings) {
      headerIndex.set(g.rule, rows.length);
      const open = props.isOpen(g.rule);
      rows.push({ kind: "header", rule: g.rule, count: g.items.length, open });
      if (open) for (const f of g.items) rows.push({ kind: "finding", f });
    }
    return { rows, headerIndex };
  });
  const rows = () => model().rows;

  let scrollEl!: HTMLDivElement;
  const virtualizer = createVirtualizer({
    get count() {
      return rows().length;
    },
    getScrollElement: () => scrollEl,
    // Header is short and fixed; findings vary (snippet / aligned block) so
    // this is just a seed — measureElement corrects each mounted row.
    estimateSize: (i) => (rows()[i]?.kind === "header" ? 44 : 96),
    overscan: 12,
  });

  // @tanstack/solid-virtual caches each row's measured size BY INDEX. Our
  // flattened model remaps indices on every expand/collapse and filter
  // change (index N flips between a 44px header and a 96px+ finding), so
  // those cached sizes go stale — producing mispositioned/overlapping
  // rows and a wrong total height (the "expand/collapse is bugged"
  // symptom). Reset the measurement cache whenever the model changes so
  // the visible slice lays out against fresh measurements.
  //
  // Two further triggers are load-bearing:
  //   * props.aligned() — toggling aligned re-renders every row's block
  //     (raw ↔ space-padded columns), changing widths/heights, but it does
  //     NOT change `model`. Without tracking it the cache stays stale and
  //     the aligned view mis-lays-out ("aligned only shows the first one").
  //   * requestAnimationFrame — this effect first runs during the initial
  //     render (and on every re-mount when a new file's report arrives),
  //     BEFORE the browser lays out the scroll container, so the first
  //     measure() reads a zero rect and getVirtualItems() comes back empty
  //     → the list stays blank until some later model change re-measures
  //     ("findings don't show on load until you toggle"). Re-measuring after
  //     layout populates the initial view.
  createEffect(() => {
    model(); // track: any expand/collapse or filter change
    props.aligned(); // track: aligned toggle re-renders rows (no model change)
    virtualizer.measure();
    requestAnimationFrame(() => {
      virtualizer.measure();
    });
  });

  onMount(() => {
    // eslint-disable-next-line solid/reactivity -- imperative jump handler invoked on user action; reading model() at call-time is intended (event-handler-like)
    props.registerJump?.((rule) => {
      const idx = model().headerIndex.get(rule);
      if (idx != null) virtualizer.scrollToIndex(idx, { align: "start" });
    });
  });

  return (
    <Show when={props.report.findings.length > 0} fallback={null}>
      <div class="flex min-w-0 flex-col gap-3">
        <div class="flex items-center gap-3 text-xs text-fg-muted">
          <button
            type="button"
            class="underline-offset-2 hover:text-fg hover:underline"
            onClick={() => {
              props.onExpandAll();
            }}
          >
            Expand all
          </button>
          <button
            type="button"
            class="underline-offset-2 hover:text-fg hover:underline"
            onClick={() => {
              props.onCollapseAll();
            }}
          >
            Collapse all
          </button>
        </div>
        <div
          ref={scrollEl}
          class="scroll-region rounded-lg border border-line bg-surface"
        >
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            <For each={virtualizer.getVirtualItems()}>
              {(vi) => {
                // Reactive row read (NOT `const row = rows()[vi.index]`):
                // when the model changes but @tanstack reuses the
                // VirtualItem for this index, a once-captured `row` would
                // render stale content — the "values don't fully update"
                // symptom. Reading rows()[vi.index] reactively re-renders
                // the row when the model changes. Guard the index too: a
                // collapse can shrink rows() below a still-mounted index.
                const row = () => rows()[vi.index];
                return (
                  <Show when={row()}>
                    {(r) => (
                      <div
                        data-index={vi.index}
                        ref={(el) => {
                          queueMicrotask(() => {
                            virtualizer.measureElement(el);
                          });
                        }}
                        style={{
                          position: "absolute",
                          top: 0,
                          left: 0,
                          width: "100%",
                          transform: `translateY(${vi.start}px)`,
                        }}
                      >
                        <Show
                          when={
                            r().kind === "header" ? (r() as HeaderRow) : null
                          }
                          fallback={
                            <FindingRow
                              f={(r() as FindingRowData).f}
                              lines={lines}
                              aligned={props.aligned}
                            />
                          }
                        >
                          {(h) => (
                            <button
                              type="button"
                              id={ruleAnchor(h().rule)}
                              onClick={() => {
                                props.onToggle(h().rule);
                              }}
                              class="flex w-full scroll-mt-4 items-baseline gap-2 border-b border-line bg-surface-raised px-3 py-2 text-left text-sm font-medium text-fg"
                            >
                              <Chevron open={h().open} class="self-center" />
                              {shortRule(h().rule)}
                              <span class="ml-2 text-xs font-normal text-fg-faint">
                                {h().count} finding{h().count === 1 ? "" : "s"}
                              </span>
                            </button>
                          )}
                        </Show>
                      </div>
                    )}
                  </Show>
                );
              }}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
};
