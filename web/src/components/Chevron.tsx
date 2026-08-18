import type { Component } from "solid-js";

// The single disclosure arrow for every collapsible panel. Before this, panels
// were a mix: some used a tiny `▸` glyph (text-xs, barely visible), others fell
// back to the browser's default `<details>` marker (a bigger, OS-dependent
// triangle) — so the same gesture looked different across the app. This is one
// comfortably-sized, GitHub-like chevron (~14px) that rotates 90° when its
// enclosing `<details class="group">` is open.
//
// Usage (two patterns, one look):
//   • Native <details>: first child of a `<summary>`; add `group` on the
//     `<details>` and `list-none [&::-webkit-details-marker]:hidden` on the
//     `<summary>`. Rotation is CSS-driven via `group-open` (no `open` prop).
//   • Manual toggle (a `<button>` flipping a signal, e.g. FindingsView /
//     AnalyseView group rows): pass `open={isOpen}` and rotation tracks it.
export const Chevron: Component<{ class?: string; open?: boolean }> = (
  props,
) => (
  <svg
    viewBox="0 0 16 16"
    aria-hidden="true"
    class={`h-3.5 w-3.5 shrink-0 text-fg-muted transition-transform ${props.open === undefined ? "group-open:rotate-90" : props.open ? "rotate-90" : ""} ${props.class ?? ""}`}
  >
    <path
      d="M6 4l4 4-4 4"
      fill="none"
      stroke="currentColor"
      stroke-width="1.75"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
);
