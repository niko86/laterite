import type { ParentComponent } from "solid-js";

// The repeated panel wrapper (`rounded-lg border border-line bg-surface` with
// responsive padding) in one place, so spacing stays consistent and scales down
// on a phone (`p-3`) and up on larger screens (`sm:p-4`).
export const Card: ParentComponent<{ class?: string }> = (props) => (
  <section
    class={`rounded-lg border border-line bg-surface p-3 sm:p-4 ${props.class ?? ""}`}
  >
    {props.children}
  </section>
);
