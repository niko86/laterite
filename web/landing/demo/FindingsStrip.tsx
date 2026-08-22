/* The group-level findings strip (#526, shared with the TRAN cover sheet in
 * #527): the callouts a table wears for findings the engine attaches to the
 * GROUP rather than a cell. Attached to the table it judges — a finding in
 * the prose beside it reads as commentary; here it reads as a verdict.
 */

import { For, Show, type Component } from "solid-js";
import { FindingCallout } from "./FindingCallout";
import type { Finding } from "./engine";

export const FindingsStrip: Component<{
  code: string;
  findings: readonly Finding[];
}> = (props) => (
  <Show when={props.findings.length}>
    <ul
      aria-label={`${props.code} findings`}
      class="mt-3 list-none space-y-2 p-0"
    >
      <For each={props.findings}>
        {(f) => (
          <li>
            <FindingCallout severity={f.severity} rule={f.rule}>
              {f.desc}
            </FindingCallout>
          </li>
        )}
      </For>
    </ul>
  </Show>
);
