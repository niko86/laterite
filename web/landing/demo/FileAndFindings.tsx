/* The file and the findings (#397) — the half that closes the loop.
 *
 * The AGS4 file the tables above emit, line for line, and what the shipped
 * engine says about it. Severity comes from the engine and is never decided
 * here: the seeded SAMP_TYPE defect is an ERROR, not the warning the design
 * handoff captions it as, and hard-coding a severity in the UI is how a demo
 * comes to disagree with the tool it is advertising.
 *
 * The output pane scrolls sideways rather than wrapping — AGS4 lines are
 * long, and wrapping them destroys the column alignment that makes the format
 * readable at all — EXCEPT below the layout breakpoint (#596): findings cite
 * line numbers, and on a phone a hidden line end is worse than a wrapped one,
 * so there the pane soft-wraps with a hanging indent under the number and
 * the number keeps meaning the logical line. Type size never shrinks either
 * way. The three secondary-prose pieces #596 used to drop below the
 * breakpoint — the status pill, the orphan explainer, the transport aside —
 * stopped existing at ANY width in #617: the pass-2 review judged each one
 * paid in scroll for a point the demo already makes (the privacy claim
 * survives in the findings outro; the transport story moved to one sentence
 * in the Pick-your-stack intro). What the breakpoint still swaps is layout,
 * not prose: the pane's wrap above, and the findings list's carousel (#592).
 */

import {
  For,
  Index,
  Show,
  createMemo,
  createSignal,
  type Component,
} from "solid-js";
import { Button, Checkbox } from "@shared/components";
import { EditableGroup } from "./EditableGroup";
import { FindingCallout } from "./FindingCallout";
import { Carousel } from "../components/Carousel";
import { severityLineTint, verdictTint, worstPerLine } from "./severity";
import { verdictState } from "./verdict";
import { alignLines } from "./align";
import { narrowViewport } from "../viewport";
import {
  armed,
  busy,
  focusLine,
  reset,
  report,
  setFocusLine,
  text,
} from "./store";
import type { Finding } from "./engine";

/* One row of the panel — the shared callout with the panel's two extras: the
   click-to-focus wiring, and the GROUP chip. The chip is what tells the two
   Rule 16 findings apart: the engine correctly reports the same abbreviation
   against SAMP and against LLPL, with byte-identical text — a duplicate to
   the eye until something names the group (#526). The `li` belongs to the
   callers since #592: the stack and the carousel each bring their own. */
const FindingRow: Component<{ finding: Finding }> = (props) => (
  <FindingCallout
    severity={props.finding.severity}
    rule={props.finding.rule}
    group={props.finding.group || undefined}
    line={props.finding.line}
    disabled={props.finding.line === null}
    onClick={() => setFocusLine(props.finding.line)}
  >
    {props.finding.desc}
  </FindingCallout>
);

export const FileAndFindings: Component<{ band: string }> = (props) => {
  const lines = createMemo(() => text().split("\r\n"));
  /* The aligned VIEW (#620, the webapp's grammar recomputed in align.ts):
     display-only, intra-line padding only, so `shown` always has exactly
     `lines()`'s count and every per-line signal (tints, focus, numbers)
     keys by the same index in both modes. Raw stays the default — the
     #396 byte-fidelity story belongs to it. */
  const [alignedView, setAlignedView] = createSignal(false);
  const shown = createMemo(() =>
    alignedView() ? alignLines(lines()) : lines(),
  );
  const findings = createMemo(() => report()?.findings ?? []);
  /* The refused run's surface (#638): an errored report carries an empty
     findings list, and this panel's zero-state read "Clean" over it. The
     refusal renders the engine's own message — the UI neither rewords a
     refusal nor decides how bad — and the finding count stands down: "0"
     under a refused run is the same false claim in digits. Derived through
     the same verdictState the chip reads, so the two surfaces cannot
     disagree about what kind of run this was. */
  const refusal = () => {
    const r = report();
    if (!r) return null;
    const s = verdictState(r);
    return s.kind === "refused" ? s : null;
  };

  /** Worst severity per banded line, so the pane can tint without a lookup
   *  per line — the last finding surface to route through severity.ts
   *  (#548): before this, every banded line wore the error tint whatever the
   *  tier. The absence-rule story (no line, no band) lives with the map's
   *  own docstring. */
  const lineTiers = createMemo(() => worstPerLine(findings()));

  return (
    <div style={{ "--band": `var(${props.band})` }}>
      <div>
        <h2 class="font-display text-h2 font-extrabold tracking-(--track-tight) text-accent">
          The file, and what the engine says
        </h2>
        <p class="mt-2 max-w-[60ch] text-fg-soft">
          This is the delivery the tables above emit, and these are the findings
          the shipped validator returns for it. The CLI and the Python library
          run the same engine.
        </p>
      </div>

      {/* The cover sheet (#527): TRAN, the delivery's transmission header —
          an ORDINARY editable group table, same component and contracts as
          the four above, not an eighth descent section (the strata ramp
          stays at seven; recorded decision). It lives here because a
          transmission header is ABOUT the file beside it. Rule 14 used to be
          a permanent seeded finding no interaction could clear; the seed now
          carries a clean TRAN, and the rule only fires when the reader
          deletes its one row — a finding they can cause, read, and undo. */}
      {/* The pairing grid is GroupSection's, restated (#594): same column
          template, same min-w-0 floors, and it continues the descent's
          alternation — LLPL led with the prose, so the cover sheet leads
          with the table. Restated rather than reused: GroupSection also owns
          the chip, the schema heading and the descent affordances, none of
          which the cover sheet wants, and the grid shell left over is two
          lines — smaller than the seam extracting it would cut. */}
      <div class="mt-8 grid gap-8 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,22rem)] min-[64rem]:items-start">
        <div class="min-w-0 min-[64rem]:order-2">
          <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
            The cover sheet · TRAN
          </p>
          <p class="mt-1 max-w-[60ch] text-caption text-fg-soft">
            Every delivery opens with its transmission header: who produced the
            file, for whom, and against which AGS edition. Delete its row, or
            the whole group, and Rule 14 has something to say.
          </p>
        </div>
        <div class="min-w-0 min-[64rem]:order-1">
          <EditableGroup code="TRAN" band={props.band} />
        </div>
      </div>

      <div class="mt-6 grid gap-6 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,26rem)] min-[64rem]:items-start">
        <div class="min-w-0">
          {/* The output pane. The validator-vs-fixer explainer that stood
              here retired in #617 (pass-2 pin D2-04): what remains of the
              story is the mechanism itself — the orphan's Rule 10c finding
              wears the manual badge in its group table and strip (the fixer
              refuses it), with no prose narrating it. */}
          <div class="overflow-hidden rounded-lg border border-line bg-surface-code">
            <div class="flex items-center justify-between border-b border-line px-3 py-2">
              <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
                delivery.ags
              </p>
              {/* The webapp's control, same label, same grammar (#620) —
                  the convergence direction is shared components, website
                  first, so the borrowed wording is deliberate. */}
              <Checkbox
                label="Aligned columns"
                title="A display-only view: the engine reads the raw bytes either way"
                checked={alignedView()}
                onChange={(e) => setAlignedView(e.currentTarget.checked)}
              />
            </div>
            <div class="max-h-[26rem] overflow-auto overscroll-contain">
              <Show
                when={armed()}
                fallback={
                  <p class="p-4 text-caption text-fg-muted">
                    The engine is on its way: it loads itself shortly after the
                    page paints, and this pane fills with the file and its
                    findings, live.
                  </p>
                }
              >
                <For each={shown()}>
                  {(line, i) => {
                    const n = () => i() + 1;
                    const tier = () => lineTiers().get(n());
                    const band = () => {
                      const t = tier();
                      return t === undefined
                        ? "border-l-transparent text-fg-soft"
                        : severityLineTint(t);
                    };
                    return (
                      <div
                        class={`flex gap-3 whitespace-pre border-l-[3px] px-3 font-mono text-caption leading-[1.7] ${band()}`}
                        classList={{
                          "[box-shadow:var(--focus-ring)]": focusLine() === n(),
                        }}
                        ref={(el) => {
                          // Scroll the pane, not the page, when a finding is clicked.
                          if (focusLine() === n()) {
                            queueMicrotask(() => {
                              el.scrollIntoView({ block: "center" });
                            });
                          }
                        }}
                      >
                        <span class="w-8 shrink-0 text-right text-fg-dim select-none">
                          {n()}
                        </span>
                        {/* Below the breakpoint the content wraps INSIDE its
                            own flex item (#596) — the number column stays a
                            fixed gutter, so continuation lines land as a
                            hanging indent under it, and `anywhere` is what
                            lets an unspaced AGS record break at all. Desktop
                            keeps the row unconstrained, which is what the
                            scroller's side-scroll rides on. */}
                        {/* Aligned mode opts OUT of the phone wrap (M2-06):
                            columnar text cannot wrap and stay columnar, so
                            the pane pans horizontally instead — the
                            scroller above is already overflow-auto. */}
                        <span
                          classList={{
                            "max-[64rem]:min-w-0 max-[64rem]:flex-1 max-[64rem]:whitespace-pre-wrap max-[64rem]:[overflow-wrap:anywhere]":
                              !alignedView(),
                          }}
                        >
                          {line}
                        </span>
                      </div>
                    );
                  }}
                </For>
              </Show>
            </div>
          </div>

          {/* The everything-at-once verb (#594): reset reverts every table
              on the page, and the pane above is the one place the WHOLE
              delivery is visible at once — which is what earns it this spot. */}
          <div class="mt-3 flex flex-wrap items-center gap-3">
            <Button variant="default" onClick={reset}>
              Reset the delivery
            </Button>
            <Show when={busy()}>
              <span class="text-caption text-fg-faint">validating…</span>
            </Show>
          </div>
        </div>

        {/* The findings list. The id is the scoreboard's jump target (#531):
            the chip states the verdict, this panel is its evidence. */}
        <div id="findings" class="min-w-0 scroll-mt-16">
          <p class="mt-3 font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
            Findings
            <Show when={armed() && report() && !refusal()}>
              <span class="ml-2 text-fg-faint">{findings().length}</span>
            </Show>
          </p>

          <Show
            when={armed()}
            fallback={
              <p class="mt-3 text-caption text-fg-muted">
                The engine is still on its way.
              </p>
            }
          >
            <Show
              when={!refusal() && findings().length}
              fallback={
                <Show
                  when={refusal()}
                  fallback={
                    <p class="mt-3 rounded-md border border-ok/40 bg-ok-quiet px-3 py-2 text-caption text-ok">
                      Clean: 0 findings.
                    </p>
                  }
                >
                  {(err) => (
                    <p
                      class={`mt-3 rounded-md border px-3 py-2 text-caption ${verdictTint(false)}`}
                    >
                      {err().message}
                    </p>
                  )}
                </Show>
              }
            >
              {/* Below the breakpoint the list becomes the one-card carousel
                  (#592) — same rows, same order, paged instead of stacked. */}
              <Show
                when={!narrowViewport()}
                fallback={
                  <Carousel
                    label="Findings"
                    items={findings()}
                    chrome="counter"
                    noun="finding"
                    card={(f) => <FindingRow finding={f()} />}
                  />
                }
              >
                {/* Index, not For (#534): every revalidation mints fresh
                    finding objects, so a reference-keyed For would recreate
                    every row per keystroke and re-fire the entrance fade
                    across the whole panel. Index updates rows in place; only
                    a row that genuinely appears fades in. */}
                <ul class="mt-3 list-none space-y-2 p-0">
                  <Index each={findings()}>
                    {(finding) => (
                      <li>
                        <FindingRow finding={finding()} />
                      </li>
                    )}
                  </Index>
                </ul>
              </Show>
            </Show>
          </Show>

          <p class="mt-3 text-caption text-fg-muted">
            Want to run this on your own delivery?{" "}
            <a
              class="font-semibold text-cta no-underline transition-colors hover:underline"
              href="https://app.laterite.dev/"
            >
              Open the webapp
            </a>
            ; it stays in your browser too.
          </p>
        </div>
      </div>
    </div>
  );
};
