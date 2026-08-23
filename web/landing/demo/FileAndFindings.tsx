/* The file and the findings (#397) — the half that closes the loop.
 *
 * The AGS4 file the tables above emit, line for line, and what the shipped
 * engine says about it. Severity comes from the engine and is never decided
 * here: the seeded SAMP_TYPE defect is an ERROR, not the warning the design
 * handoff captions it as, and hard-coding a severity in the UI is how a demo
 * comes to disagree with the tool it is advertising.
 *
 * The output pane scrolls sideways rather than wrapping. AGS4 lines are long,
 * and wrapping them destroys the column alignment that makes the format readable
 * at all — which is the one thing this pane exists to show.
 *
 * "Nothing is uploaded" sits with the findings panel (#594), not in the
 * footer: the verdict is the moment a reader wonders where their file just
 * went, so the answer stands directly above it.
 */

import { For, Index, Show, createMemo, type Component } from "solid-js";
import { Button } from "@shared/components";
import { EditableGroup } from "./EditableGroup";
import { FindingCallout } from "./FindingCallout";
import { severityLineTint, worstPerLine } from "./severity";
import {
  armed,
  busy,
  focusLine,
  isManualFinding,
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
   the eye until something names the group (#526). */
const FindingRow: Component<{ finding: Finding }> = (props) => (
  <li>
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
  </li>
);

export const FileAndFindings: Component<{ band: string }> = (props) => {
  const lines = createMemo(() => text().split("\r\n"));
  const findings = createMemo(() => report()?.findings ?? []);

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

      {/* The transport aside (#528): TRAN taken literally. The delivery is
          built to travel, so the envelope gets told as a story beside the
          transmission header — drawn, never run: this page's wasm build
          ships without the transport feature on purpose, and the only way
          out of the aside is the cookbook page. */}
      <aside
        aria-label="Transport"
        class="mt-8 max-w-[46rem] rounded-lg border border-line bg-surface p-4 dark:bg-surface-raised"
      >
        <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
          TRAN, taken literally
        </p>
        <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-caption">
          <code class="rounded-sm border border-line bg-surface-code px-2 py-1 text-fg">
            delivery.ags
          </code>
          <span aria-hidden="true" class="text-fg-faint">
            →
          </span>
          <span class="text-micro uppercase tracking-(--track-micro) text-fg-muted">
            pack
          </span>
          <span aria-hidden="true" class="text-fg-faint">
            →
          </span>
          <code class="rounded-sm border border-line bg-surface-code px-2 py-1 text-fg">
            delivery.ags.zst
          </code>
        </div>
        <div class="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-caption">
          <code class="rounded-sm border border-line bg-surface-code px-2 py-1 text-fg">
            delivery.ags
          </code>
          <span aria-hidden="true" class="text-fg-faint">
            →
          </span>
          <span class="text-micro uppercase tracking-(--track-micro) text-fg-muted">
            lock
          </span>
          <span aria-hidden="true" class="text-fg-faint">
            →
          </span>
          <code class="rounded-sm border border-line bg-surface-code px-2 py-1 text-fg">
            delivery.ags.zst.age
          </code>
        </div>
        <p class="mt-3 max-w-[60ch] text-caption text-fg-soft">
          The delivery itself is built for the trip its cover sheet describes:{" "}
          <code class="font-mono">pack</code> squeezes the file with zstd and{" "}
          <code class="font-mono">unpack</code> restores it byte-for-byte (the
          pair moves bytes, it never parses them) while{" "}
          <code class="font-mono">lock</code> starts from the original and seals
          its zstd pack inside a passphrase-encrypted age envelope. Shown here,
          not run: this page's engine deliberately ships without transport.{" "}
          <a
            class="font-semibold text-cta no-underline transition-colors hover:underline"
            href="https://docs.laterite.dev/cookbook/transport/"
          >
            Pack / encrypt for transport
          </a>
        </p>
      </aside>

      <div class="mt-6 grid gap-6 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,26rem)] min-[64rem]:items-start">
        <div class="min-w-0">
          {/* The validator-vs-fixer lesson (#530), sitting directly above the
              pane whose lines it is about (#594): shown while the orphan the
              copy describes actually stands — pinned to Rule 10c on LLPL, the
              orphan's own identity, so no OTHER manual LLPL finding can hold
              the note up after the orphan is repaired. Gated on the finding,
              not a click — the global fix button it used to follow is gone;
              each table now carries its own fix budget. The neutral "note"
              tone — same callout as every finding, no verdict. */}
          <Show
            when={report()?.findings.some(
              (f) =>
                f.rule === "AGS Format Rule 10c" &&
                f.group === "LLPL" &&
                isManualFinding(f),
            )}
          >
            <div class="mb-3 max-w-[70ch]">
              <FindingCallout severity="note">
                The orphaned <code class="font-mono">LLPL</code> row is left
                standing on purpose: no fix button will touch it, and its
                finding wears a manual badge. The engine can tell you a lab
                result points at a sample that does not exist, but only a human
                knows whether the sample reference is wrong or the sample is
                missing. That is the whole difference between a validator and a
                fixer.
              </FindingCallout>
            </div>
          </Show>

          {/* The output pane. */}
          <div class="overflow-hidden rounded-lg border border-line bg-surface-code">
            <p class="border-b border-line px-3 py-2 font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
              delivery.ags
            </p>
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
                <For each={lines()}>
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
                        <span>{line}</span>
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
          <p class="rounded-md border border-ok/40 bg-ok-quiet px-3 py-1.5 text-caption font-semibold text-ok">
            Nothing is uploaded. The engine runs in this tab.
          </p>
          <p class="mt-3 font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
            Findings
            <Show when={armed() && report()}>
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
              when={findings().length}
              fallback={
                <p class="mt-3 rounded-md border border-ok/40 bg-ok-quiet px-3 py-2 text-caption text-ok">
                  Clean: 0 findings.
                </p>
              }
            >
              {/* Index, not For (#534): every revalidation mints fresh
                  finding objects, so a reference-keyed For would recreate
                  every row per keystroke and re-fire the entrance fade
                  across the whole panel. Index updates rows in place; only
                  a row that genuinely appears fades in. */}
              <ul class="mt-3 list-none space-y-2 p-0">
                <Index each={findings()}>
                  {(finding) => <FindingRow finding={finding()} />}
                </Index>
              </ul>
            </Show>
          </Show>

          <p class="mt-4 text-caption text-fg-faint">
            Two findings for one bad cell is correct, not a duplicate: the
            SAMP_TYPE value is part of LLPL's key tuple, so it is wrong in both
            groups. That repetition is the format, working.
          </p>

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
