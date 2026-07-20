import { type Component } from "solid-js";

// One shared pill-button for the in-pane view/tool selectors (Explore's
// Browse/SQL/Charts/Analyse, Fix's Fixes/Diff, the Tools sub-tool row). These
// were three near-identical hand-rolled components; consolidating them means a
// styling or token change happens once. (The top-level tab bar in Tabs.tsx is
// a different affordance — an underlined `role="tab"` — so it stays separate.)
export const PillToggle: Component<{
  label: string;
  active: boolean;
  onClick: () => void;
}> = (props) => (
  <button
    type="button"
    onClick={() => {
      props.onClick();
    }}
    class="rounded px-3 py-1 font-medium transition-colors"
    classList={{
      "bg-chip text-accent": props.active,
      "text-fg-muted hover:text-fg": !props.active,
    }}
  >
    {props.label}
  </button>
);
