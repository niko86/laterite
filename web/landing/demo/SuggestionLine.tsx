/* The did-you-mean suggestion under the error it explains (#1024).
 *
 * Drawn the way the panel draws a divergence note (DivergenceNoteBlock in
 * FileAndFindings): indented under its finding, set in the micro size, with a
 * rule on the left. It is commentary on a verdict, not a second verdict, so it
 * carries no severity tint and the reader can skip it and still have read the
 * findings. The "suggestion" label is the non-colour carrier. Every surface
 * that shows the Rule 16 error uses this one block, so the suggestion reads
 * the same in the panel, the group strip, the field card and the cell popover.
 */

import type { Component } from "solid-js";
import type { Finding } from "./engine";
import { suggestionText } from "./suggestions";

export const SuggestionLine: Component<{ suggestion: Finding }> = (props) => (
  <div
    class="mt-1 ml-3 border-l-2 border-line-strong pl-3 text-micro text-fg-muted"
    data-testid="suggestion"
  >
    <p class="font-mono uppercase tracking-(--track-micro)">suggestion</p>
    <p class="mt-1 normal-case">{suggestionText(props.suggestion)}</p>
  </div>
);
