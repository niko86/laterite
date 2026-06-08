import type { ParentComponent } from "solid-js";

// A responsive control row. The builders used ad-hoc `flex flex-wrap items-end
// gap-3` rows that wrap into ragged 2–3 row piles on a phone (ChartBuilder's
// seven selects are the worst). A real grid stacks predictably 1 → 2 → 3
// columns (the same reflux already used in TemplateGenerator/RevisionDiff), and
// `[&>label]:min-w-0` lets a long <option> shrink instead of forcing overflow.
// Each direct child (typically a <label> field) takes one cell.
export const ControlGrid: ParentComponent<{ class?: string }> = (props) => (
  <div
    class={`grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 [&>label]:min-w-0 ${props.class ?? ""}`}
  >
    {props.children}
  </div>
);
