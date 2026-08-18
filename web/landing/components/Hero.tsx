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
 * format has the whole demo three sections down. It is `aria-hidden` because it
 * is a picture of the format rather than content — the same lines are live,
 * selectable and labelled in the output pane (#397).
 */

import { For, type Component } from "solid-js";
import { Button } from "@shared/components";

/* Four lines of the seeded delivery, chosen to show the shape rather than the
   data: the GROUP/HEADING/UNIT/TYPE stanza that opens every AGS4 group. Not
   read from the fixture — this is a fragment sized to the card, and the honest
   rendering of the whole file is #397's job. */
const EXCERPT = [
  ['"GROUP"', '"LOCA"'],
  ['"HEADING"', '"LOCA_ID"', '"LOCA_TYPE"', '"LOCA_GL"'],
  ['"UNIT"', '""', '""', '"m"'],
  ['"TYPE"', '"ID"', '"PA"', '"2DP"'],
  ['"DATA"', '"BH01"', '"CP"', '"11.80"'],
];

export const Hero: Component = () => (
  <div class="grid items-center gap-10 min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,24rem)]">
    <div>
      <p class="flex items-center gap-2 font-mono text-micro uppercase text-fg-muted">
        {/* The sand tick — 4px of the ramp's head, the smallest the band
            vocabulary appears anywhere on the page. */}
        <span
          aria-hidden="true"
          class="inline-block h-[4px] w-6 rounded-full bg-laterite-300"
        />
        AGS4 tooling · beta
      </p>

      <h1 class="mt-4 max-w-[22ch] font-display text-h1 font-extrabold text-balance text-accent min-[30rem]:text-display min-[64rem]:text-hero">
        AGS4, shown rather than explained.
      </h1>

      <p class="mt-5 max-w-[54ch] text-lead text-fg-soft">
        One Rust engine for AGS4 geotechnical transfer files — validate, fix,
        explore and write them, from Python, Node, the command line, DuckDB or a
        browser tab. Scroll down and break a real delivery; the validator runs
        here, in this page.
      </p>

      <div class="mt-7 flex flex-wrap items-center gap-3">
        <Button variant="primary" size="lg" href="#install">
          Pick your stack
        </Button>
        <Button variant="outline" size="lg" href="#file">
          See it break
        </Button>
      </div>
    </div>

    {/* Wide viewports only. `hidden` rather than a media query in CSS so the
        markup itself is absent from the phone layout's flow. */}
    <div
      aria-hidden="true"
      class="hidden rounded-lg border border-laterite-200 bg-surface-code p-4 min-[64rem]:block"
    >
      <p class="mb-3 font-mono text-micro uppercase tracking-(--track-micro) text-fg-faint">
        delivery.ags
      </p>
      <pre class="overflow-x-auto font-mono text-caption leading-[1.7] text-fg-soft">
        <For each={EXCERPT}>
          {(cells) => (
            <div class="whitespace-pre">
              <span class="text-accent">{cells[0]}</span>
              {cells.length > 1 ? "," : ""}
              {cells.slice(1).join(",")}
            </div>
          )}
        </For>
      </pre>
    </div>
  </div>
);
