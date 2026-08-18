import { For, type Component } from "solid-js";

export type TabId = "validate" | "fix" | "explore" | "tools" | "export";

const TABS: { id: TabId; label: string }[] = [
  { id: "validate", label: "Validate" },
  { id: "fix", label: "Fix" },
  { id: "explore", label: "Explore" },
  { id: "tools", label: "Tools" },
  { id: "export", label: "Export" },
];

export const Tabs: Component<{
  active: TabId;
  onChange: (t: TabId) => void;
}> = (props) => {
  return (
    /* Full-width bar, centred strip (#407): the hairline is an absolutely
       positioned sibling of the scrollable tab strip rather than a border on
       either box, so it runs edge to edge while the tabs align to the shell
       column — and the active tab's 2px accent underline sits ON it (the
       inactive tabs' transparent border lets it show through), not stacked
       1px above it. */
    <nav class="relative" aria-label="Sections">
      <div
        aria-hidden="true"
        class="absolute inset-x-0 bottom-0 border-b border-line"
      />
      <div
        role="tablist"
        class="relative mx-auto flex w-full max-w-shell gap-1 overflow-x-auto px-5"
      >
        <For each={TABS}>
          {(t) => (
            <button
              type="button"
              role="tab"
              aria-selected={props.active === t.id}
              onClick={() => {
                props.onChange(t.id);
              }}
              class="border-b-2 px-4 py-2 text-sm font-medium transition-colors"
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
      </div>
    </nav>
  );
};
