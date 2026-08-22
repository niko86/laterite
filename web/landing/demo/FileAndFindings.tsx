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
 * "Nothing is uploaded" sits HERE, beside the demo, not in the footer. A reader
 * who has to go looking for that sentence has already decided not to try.
 */

import { For, Show, createMemo, type Component } from "solid-js";
import { Button } from "@shared/components";
import { FindingCallout } from "./FindingCallout";
import { FindingsStrip } from "./FindingsStrip";
import { GroupStub } from "./GroupStub";
import { GroupTable } from "./GroupTable";
import { RowCarousel } from "./RowCarousel";
import { DEMO_GROUPS } from "./schema";
import { coarsePointer } from "./pointer";
import {
  applyGroupFixes,
  arm,
  armed,
  busy,
  deleteGroup,
  deleteRow,
  delivery,
  findingsForGroup,
  focusLine,
  groupFixes,
  isManualFinding,
  picked,
  reset,
  report,
  restoreGroup,
  setCell,
  setFocusLine,
  setPicked,
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

  /* The TRAN cover sheet (#527). Since #529 schema-without-data is no longer
     a fixture mismatch — it means the reader DELETED the group, and the
     narrowing's fallback answers with the restore stub, mirroring
     GroupSection. */
  const tran = createMemo(() => {
    const schema = DEMO_GROUPS.find((g) => g.code === "TRAN");
    const data = delivery().find((g) => g.code === "TRAN");
    return schema && data ? { schema, data } : undefined;
  });
  const tranOpen = createMemo(() => {
    const p = picked();
    return p && p.group === "TRAN" ? { row: p.row, col: p.col } : null;
  });

  /** Lines carrying a finding, so the pane can band them without a lookup per
   *  line. Rules that report an absence carry no line and band nothing,
   *  correctly — delete the TRAN row and there is no line in the file where
   *  "TRAN group not found" happened. */
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
            This is the delivery the tables above emit, and these are the
            findings the shipped validator returns for it — the same engine the
            CLI and the Python library run.
          </p>
        </div>
        <p class="rounded-md border border-ok/40 bg-ok-quiet px-3 py-1.5 text-caption font-semibold text-ok">
          Nothing is uploaded. The engine runs in this tab.
        </p>
      </div>

      <div class="mt-6 flex flex-wrap items-center gap-3">
        <Button variant="default" onClick={reset}>
          Reset the delivery
        </Button>
        <Show when={busy()}>
          <span class="text-caption text-fg-faint">validating…</span>
        </Show>
      </div>

      {/* The validator-vs-fixer lesson (#530): shown while the orphan the
          copy describes actually stands — pinned to Rule 10c on LLPL, the
          orphan's own identity, so no OTHER manual LLPL finding can hold the
          note up after the orphan is repaired. Gated on the finding, not a
          click — the global fix button it used to follow is gone; each table
          now carries its own fix budget. The neutral "note" tone — same
          callout as every finding, no verdict. */}
      <Show
        when={report()?.findings.some(
          (f) =>
            f.rule === "AGS Format Rule 10c" &&
            f.group === "LLPL" &&
            isManualFinding(f),
        )}
      >
        <div class="mt-3 max-w-[70ch]">
          <FindingCallout severity="note">
            The orphaned <code class="font-mono">LLPL</code> row is left
            standing on purpose — no fix button will touch it, and its finding
            wears a <span class="font-mono text-micro uppercase">manual</span>{" "}
            badge. The engine can tell you a lab result points at a sample that
            does not exist, but only a human knows whether the sample reference
            is wrong or the sample is missing. That is the whole difference
            between a validator and a fixer.
          </FindingCallout>
        </div>
      </Show>

      {/* The cover sheet (#527): TRAN, the delivery's transmission header —
          an ORDINARY editable group table, same component and contracts as
          the four above, not an eighth descent section (the strata ramp
          stays at seven; recorded decision). It lives here because a
          transmission header is ABOUT the file beside it. Rule 14 used to be
          a permanent seeded finding no interaction could clear; the seed now
          carries a clean TRAN, and the rule only fires when the reader
          deletes its one row — a finding they can cause, read, and undo. */}
      <div class="mt-8 max-w-[46rem]">
        <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
          The cover sheet — TRAN
        </p>
        <p class="mt-1 max-w-[60ch] text-caption text-fg-soft">
          Every delivery opens with its transmission header: who produced the
          file, for whom, and against which AGS edition. Delete its row — or the
          whole group — and Rule 14 has something to say.
        </p>
        <Show
          when={tran()}
          fallback={
            <>
              <div class="mt-3">
                <GroupStub
                  code="TRAN"
                  band={props.band}
                  onRestore={() => {
                    restoreGroup("TRAN");
                  }}
                />
              </div>
              <FindingsStrip code="TRAN" findings={findingsForGroup("TRAN")} />
            </>
          }
        >
          {(t) => (
            <>
              <div class="mt-3">
                <GroupTable
                  schema={t().schema}
                  data={t().data}
                  band={props.band}
                  picked={tranOpen()}
                  onPick={(row, col) => {
                    arm();
                    setPicked({ group: "TRAN", row, col });
                  }}
                  onCommit={(row, col, value) => {
                    setCell("TRAN", row, col, value);
                  }}
                  onDeleteRow={(row) => {
                    deleteRow("TRAN", row);
                  }}
                  fixCount={groupFixes("TRAN").length}
                  onFix={() => {
                    void applyGroupFixes("TRAN");
                  }}
                />
              </div>
              <FindingsStrip code="TRAN" findings={findingsForGroup("TRAN")} />
              <div class="mt-3">
                <Button
                  variant="ghost"
                  size="sm"
                  tone="danger"
                  aria-label="Delete the TRAN group"
                  onClick={() => {
                    deleteGroup("TRAN");
                  }}
                >
                  delete group
                </Button>
              </div>
              <Show when={coarsePointer() ? tranOpen() : null}>
                {(cell) => (
                  <RowCarousel
                    schema={t().schema}
                    data={t().data}
                    band={props.band}
                    row={cell().row}
                    col={cell().col}
                    onMove={(col) =>
                      setPicked({ group: "TRAN", row: cell().row, col })
                    }
                    onClose={() => setPicked(null)}
                    onDelete={() => {
                      deleteRow("TRAN", cell().row);
                    }}
                  />
                )}
              </Show>
            </>
          )}
        </Show>
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
          <code class="font-mono">unpack</code> restores it byte-for-byte — the
          pair moves bytes, it never parses them — while{" "}
          <code class="font-mono">lock</code> starts from the original and seals
          its zstd pack inside a passphrase-encrypted age envelope. Shown here,
          not run: this page's engine deliberately ships without transport.{" "}
          <a
            class="font-semibold text-cta no-underline hover:underline"
            href="https://docs.laterite.dev/cookbook/transport/"
          >
            Pack / encrypt for transport
          </a>
        </p>
      </aside>

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
