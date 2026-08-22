/* The file and the findings (#397) — the half that closes the loop.
 *
 * The AGS4 file the four tables emit, line for line, and what the shipped engine
 * says about it. Severity comes from the engine and is never decided here: the
 * seeded SAMP_TYPE defect is an ERROR, not the warning the design handoff
 * captions it as, and hard-coding a severity in the UI is how a demo comes to
 * disagree with the tool it is advertising.
 *
 * The output pane scrolls sideways rather than wrapping. AGS4 lines are long,
 * and wrapping them destroys the column alignment that makes the format readable
 * at all — which is the one thing this pane exists to show.
 *
 * "Nothing is uploaded" sits HERE, beside the demo, not in the footer. A reader
 * who has to go looking for that sentence has already decided not to try.
 */

import { For, Show, createMemo, createSignal, type Component } from "solid-js";
import { Button } from "@shared/components";
import { FindingCallout } from "./FindingCallout";
import {
  applyEngineFixes,
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
  const [fixNote, setFixNote] = createSignal<string | null>(null);
  const lines = createMemo(() => text().split("\r\n"));
  const findings = createMemo(() => report()?.findings ?? []);

  /** Lines carrying a finding, so the pane can band them without a lookup per
   *  line. Rules with no line (14, 16) band nothing, correctly — there is no
   *  line in the file where "TRAN group not found" happened. */
  const bandedLines = createMemo(
    () =>
      new Set(
        findings()
          .map((f) => f.line)
          .filter((n): n is number => n !== null),
      ),
  );

  return (
    <div style={{ "--band": `var(${props.band})` }}>
      <div class="flex flex-wrap items-baseline justify-between gap-3">
        <div>
          <h2 class="font-display text-h2 font-extrabold tracking-(--track-tight) text-accent">
            The file, and what the engine says
          </h2>
          <p class="mt-2 max-w-[60ch] text-fg-soft">
            This is the delivery those four tables emit, and these are the
            findings the shipped validator returns for it — the same engine the
            CLI and the Python library run.
          </p>
        </div>
        <p class="rounded-md border border-ok/40 bg-ok-quiet px-3 py-1.5 text-caption font-semibold text-ok">
          Nothing is uploaded. The engine runs in this tab.
        </p>
      </div>

      <div class="mt-6 flex flex-wrap items-center gap-3">
        <Button
          variant="action"
          onClick={() => {
            void applyEngineFixes().then((n) => {
              setFixNote(
                n === 0
                  ? "Nothing left that the fixer will touch on its own."
                  : `Applied ${n} mechanical ${n === 1 ? "fix" : "fixes"}. What is left needs a human.`,
              );
            });
          }}
        >
          Fix what is safe to fix
        </Button>
        <Button variant="default" onClick={reset}>
          Reset the delivery
        </Button>
        <Show when={busy()}>
          <span class="text-caption text-fg-faint">validating…</span>
        </Show>
      </div>

      <Show when={fixNote()}>
        {/* The neutral "note" tone — same callout as every finding, no
            verdict. (Its surface/raised pairing carries #452's fix.) */}
        <div class="mt-3 max-w-[70ch]">
          <FindingCallout severity="note">
            {fixNote()} The orphaned <code class="font-mono">LLPL</code> row is
            left standing on purpose: the engine can tell you a lab result
            points at a sample that does not exist, but only a human knows
            whether the sample reference is wrong or the sample is missing. That
            is the whole difference between a validator and a fixer.
          </FindingCallout>
        </div>
      </Show>

      <div class="mt-6 grid gap-6 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,26rem)] min-[64rem]:items-start">
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
                  Edit any cell above, or press Fix, and the engine loads —
                  about two megabytes of WebAssembly, fetched only if you
                  actually want it.
                </p>
              }
            >
              <For each={lines()}>
                {(line, i) => {
                  const n = () => i() + 1;
                  const banded = () => bandedLines().has(n());
                  return (
                    <div
                      class="flex gap-3 whitespace-pre px-3 font-mono text-caption leading-[1.7]"
                      classList={{
                        "border-l-[3px] border-l-err bg-err-quiet text-err":
                          banded(),
                        "border-l-[3px] border-l-transparent text-fg-soft":
                          !banded(),
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

        {/* The findings list. */}
        <div>
          <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
            Findings
            <Show when={armed() && report()}>
              <span class="ml-2 text-fg-faint">{findings().length}</span>
            </Show>
          </p>

          <Show
            when={armed()}
            fallback={
              <p class="mt-3 text-caption text-fg-muted">
                The engine has not been loaded yet.
              </p>
            }
          >
            <Show
              when={findings().length}
              fallback={
                <p class="mt-3 rounded-md border border-ok/40 bg-ok-quiet px-3 py-2 text-caption text-ok">
                  Clean — 0 findings.
                </p>
              }
            >
              <ul class="mt-3 list-none space-y-2 p-0">
                <For each={findings()}>
                  {(finding) => <FindingRow finding={finding} />}
                </For>
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
              class="font-semibold text-cta no-underline hover:underline"
              href="https://app.laterite.dev/"
            >
              Open the web app
            </a>{" "}
            — it stays in your browser too.
          </p>
        </div>
      </div>
    </div>
  );
};
