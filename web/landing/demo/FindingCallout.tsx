/* One finding, one look (#526).
 *
 * Findings used to render three hand-rolled ways — red boxes in the prose
 * column, red boxes under the carousel's field card, severity boxes in the
 * panel — and the reader had to learn each one meant the same thing. This is
 * the single callout they all use now: severity tint from the engine (via
 * severity.ts — the UI never decides how bad), the rule bold, an optional
 * GROUP chip for the panel where two findings can carry byte-identical text
 * (Rule 16 reports the same abbreviation against SAMP and LLPL — correct,
 * and indistinguishable without naming the group), and an optional line.
 *
 * The chip is deliberately colourless: band colour means group IDENTITY —
 * GroupTable's header enumerates the only places it appears (#396) — and a
 * banded chip would start reading as severity.
 *
 * With `onClick` it renders as a button (the panel's click-to-focus); without,
 * a plain block. `severity="note"` is the neutral tone for explanatory boxes
 * that carry no verdict.
 */

import { Show, type Component, type JSX } from "solid-js";
import { Dynamic } from "solid-js/web";
import { severityTint } from "./severity";

export const FindingCallout: Component<{
  severity: string;
  rule?: string;
  /** The group the finding names — the panel's disambiguator. */
  group?: string;
  /** #530: true when the engine's fixer will not touch this finding — the
   *  reader's work, badged so the fix buttons' silence is explained. */
  manual?: boolean;
  line?: number | null;
  onClick?: () => void;
  disabled?: boolean;
  children: JSX.Element;
}> = (props) => (
  <Dynamic
    component={props.onClick ? "button" : "div"}
    type={props.onClick ? "button" : undefined}
    disabled={props.onClick ? props.disabled : undefined}
    onClick={props.onClick}
    class={[
      /* whitespace-normal (#636): the hover popover mounts INSIDE the
         cell's td, and the td's nowrap — which keeps table cells
         one-line — inherits into the panel, so the finding could not
         wrap at all and walked out of the box. The callout declares its
         own wrapping context instead of trusting whatever home it is
         mounted in. break-words is the second guard: a KEY-combination
         token is one unbroken run, and a long enough one must break
         mid-token rather than escape. */
      "w-full rounded-md border px-3 py-2 text-left text-caption whitespace-normal break-words",
      // The callout FADES IN on insertion (#534): findings appearing is the
      // page's most frequent motion, and it rides the fast opacity tier.
      "transition-opacity duration-(--dur-fast) starting:opacity-0",
      "focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]",
      severityTint(props.severity),
    ].join(" ")}
  >
    <Show
      when={props.rule || props.group || props.manual || props.line != null}
    >
      <span class="flex flex-wrap items-baseline gap-2">
        <Show when={props.rule}>
          <span class="font-semibold">{props.rule}</span>
        </Show>
        <Show when={props.group}>
          <span class="rounded-sm border border-current/30 px-1 font-mono text-micro">
            {props.group}
          </span>
        </Show>
        <Show when={props.manual}>
          <span class="rounded-sm border border-current/30 px-1 font-mono text-micro uppercase">
            manual
          </span>
        </Show>
        <Show when={props.line != null}>
          <span class="font-mono text-micro opacity-70">line {props.line}</span>
        </Show>
      </span>
    </Show>
    <span class="mt-1 block font-normal first:mt-0">{props.children}</span>
  </Dynamic>
);
