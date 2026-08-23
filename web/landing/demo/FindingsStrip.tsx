/* The group-level findings strip (#526, shared with the TRAN cover sheet in
 * #527): the callouts a table wears for findings the engine attaches to the
 * GROUP rather than a cell. Attached to the table it judges — a finding in
 * the prose beside it reads as commentary; here it reads as a verdict.
 *
 * Below the page's layout breakpoint the same findings wear the one-card
 * carousel instead (#592): the strip only exists on coarse pointers (#591),
 * and on a phone the stacked callouts were most of a section's height. The
 * width switch lives HERE so both callers — the table branch and the
 * deleted-group stub branch — inherit it from one place.
 */

import { Index, Show, type Component } from "solid-js";
import { Carousel } from "../components/Carousel";
import { FindingCallout } from "./FindingCallout";
import { isManualFinding } from "./store";
import { narrowViewport } from "../viewport";
import type { Finding } from "./engine";

/* The strip's callout flavour — the manual badge and nothing else — written
   once for both dresses, the way the panel's FindingRow is (#592). */
const StripCard: Component<{ finding: Finding }> = (props) => (
  <FindingCallout
    severity={props.finding.severity}
    rule={props.finding.rule}
    manual={isManualFinding(props.finding)}
  >
    {props.finding.desc}
  </FindingCallout>
);

export const FindingsStrip: Component<{
  code: string;
  findings: readonly Finding[];
}> = (props) => (
  <Show
    when={!narrowViewport()}
    fallback={
      <Carousel
        label={`${props.code} findings`}
        items={props.findings}
        chrome="counter"
        noun="finding"
        card={(f) => <StripCard finding={f()} />}
      />
    }
  >
    <Show when={props.findings.length}>
      <ul
        aria-label={`${props.code} findings`}
        class="mt-3 list-none space-y-2 p-0 transition-opacity duration-(--dur-fast) starting:opacity-0"
      >
        {/* Index for the same reason the panel uses it (#534): fresh finding
            objects arrive every revalidation, and only a truly new row should
            fire the entrance fade. */}
        <Index each={props.findings}>
          {(f) => (
            <li>
              <StripCard finding={f()} />
            </li>
          )}
        </Index>
      </ul>
    </Show>
  </Show>
);
