/* The hero (#395).
 *
 * "AGS4, shown rather than explained" is the page's thesis and the demo below is
 * its proof, so the hero's job is to be brief and get out of the way. The
 * headline is capped near 22 characters a line — the measure Figtree 800 holds
 * at display size before it starts reading as a paragraph.
 *
 * The file excerpt is a WIDE-VIEWPORT affordance, not a universal one. The
 * mobile artboard drops it, and it is right to: on a phone, an install command
 * answers "what is this" faster than raw AGS4 does, and the reader who wants the
 * format has the whole demo three sections down.
 *
 * Since #531 the card is CONTENT, not a picture: its lines are sliced from the
 * committed fixture (heroExcerpt.ts — the hand-written excerpt it replaces had
 * drifted into showing a corrected value the live file deliberately does not
 * carry), and the scoreboard chip beside the filename is the engine's own live
 * verdict on that same file.
 */

import { For, type Component } from "solid-js";
import { Button } from "@shared/components";
import { HERO_LINES, HERO_LINE_COUNT } from "../demo/heroExcerpt";
import { Scoreboard } from "../demo/Scoreboard";
import { armed, text } from "../demo/store";

/** What the card renders (#531 "hydrates"): the LIVE file's opening lines
 *  once the engine is armed, the build-time slice before that. The two are
 *  identical until the reader edits — and after an edit the card labelled
 *  delivery.ags must show the delivery, not a snapshot of its seed, or the
 *  hero is back to disagreeing with the demo below it. */
const heroLines = (): readonly string[] =>
  armed() ? text().split(/\r?\n/).slice(0, HERO_LINE_COUNT) : HERO_LINES;

export const Hero: Component = () => (
  <div class="grid items-center gap-10 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,24rem)]">
    <div>
      <h1 class="max-w-[22ch] font-display text-h1 font-extrabold text-balance text-accent min-[30rem]:text-display min-[64rem]:text-hero">
        AGS4, shown rather than explained.
      </h1>

      <p class="mt-5 max-w-[54ch] text-lead text-fg-soft">
        One Rust engine for AGS4 geotechnical transfer files: validate, fix,
        explore and write them, from Python, Node, the command line, DuckDB or a
        browser tab. Scroll down and put a real delivery through the validator;
        it runs here, in this page.
      </p>

      {/* Demo first (#533): the page's thesis is the demo, so the filled
          primary sends the reader there; install is the outline second. */}
      <div class="mt-7 flex flex-wrap items-center gap-3">
        <Button variant="primary" size="lg" href="#file">
          See it catch faults
        </Button>
        <Button variant="outline" size="lg" href="#install">
          Pick your stack
        </Button>
      </div>
    </div>

    {/* Wide viewports only. `hidden` rather than a media query in CSS so the
        markup itself is absent from the phone layout's flow. */}
    <div class="hidden rounded-lg border border-laterite-200 bg-surface-code p-4 min-[64rem]:block">
      <div class="mb-3 flex items-center justify-between gap-3">
        <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-faint">
          delivery.ags
        </p>
        <Scoreboard />
      </div>
      <pre class="overflow-x-auto font-mono text-caption leading-[1.7] text-fg-soft">
        <For each={heroLines()}>
          {(line) => {
            const at = line.indexOf(",");
            return (
              <div class="whitespace-pre">
                <span class="text-accent">
                  {at === -1 ? line : line.slice(0, at)}
                </span>
                {at === -1 ? "" : line.slice(at)}
              </div>
            );
          }}
        </For>
      </pre>
    </div>
  </div>
);
