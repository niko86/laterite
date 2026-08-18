import { splitProps, type Component, type JSX } from "solid-js";

// Checkbox + inline label as one clickable unit — the app's toolbar toggle.
//
// A real <input> inside a <label> rather than a drawn box: the label text
// toggles it for free, the platform gives keyboard and AT behaviour, and
// `accent-color` tints the native control without redrawing it. A hand-drawn
// checkbox would be a third of this file and worse at all three.

export const Checkbox: Component<
  { label: string; class?: string } & JSX.InputHTMLAttributes<HTMLInputElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["label", "class"]);
  return (
    <label
      class={`inline-flex items-center gap-[0.35rem] text-caption text-fg-soft cursor-pointer ${own.class ?? ""}`}
    >
      <input
        {...rest}
        type="checkbox"
        class="w-[0.9rem] h-[0.9rem] m-0 accent-(--accent)"
      />
      {own.label}
    </label>
  );
};
