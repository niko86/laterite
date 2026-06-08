import { For, type Component } from "solid-js";

export type TabId = "validate" | "fix" | "explore" | "tools";

const TABS: { id: TabId; label: string }[] = [
  { id: "validate", label: "Validate" },
  { id: "fix", label: "Fix" },
  { id: "explore", label: "Explore" },
  { id: "tools", label: "Tools" },
];

export const Tabs: Component<{
  active: TabId;
  onChange: (t: TabId) => void;
}> = (props) => {
  return (
    <nav
      class="flex gap-1 overflow-x-auto border-b border-line px-4 sm:px-6"
      role="tablist"
      aria-label="Sections"
    >
      <For each={TABS}>
        {(t) => (
          <button
            type="button"
            role="tab"
            aria-selected={props.active === t.id}
            onClick={() => props.onChange(t.id)}
            class="relative -mb-px border-b-2 px-4 py-2 text-sm font-medium transition-colors"
            classList={{
              "border-accent text-accent": props.active === t.id,
              "border-transparent text-fg-muted hover:text-fg":
                props.active !== t.id,
            }}
          >
            {t.label}
          </button>
        )}
      </For>
    </nav>
  );
};
