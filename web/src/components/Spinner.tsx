import { Show, type Component } from "solid-js";

// A small, asset-free "working, not hung" indicator — the antidote to the
// slow-hardware symptom where a multi-second wasm compile / query looks like a
// frozen tab. Inline SVG + Tailwind's `animate-spin`; announces politely to AT.
export const Spinner: Component<{ label?: string; class?: string }> = (props) => (
  <span
    role="status"
    aria-live="polite"
    class={`inline-flex items-center gap-2 text-fg-muted ${props.class ?? ""}`}
  >
    <svg
      class="size-4 shrink-0 animate-spin text-accent"
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-90"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.4 0 0 5.4 0 12h4z"
      />
    </svg>
    <Show when={props.label}>{(l) => <span class="text-sm">{l()}</span>}</Show>
  </span>
);
