import { splitProps, type Component, type JSX } from "solid-js";
import { CONTROL_CLASS } from "./Input";

// Select with the brand's own chevron. Native selects pin the platform arrow to
// the very edge and add their own text inset, so a native select and an Input
// side by side never line up. Drawing the arrow with two gradients keeps the
// text flush with Input and gives the arrow room.
//
// Two hard-edged 45°/135° wedges, offset from each other, read as one caret.
const CARET = [
  "appearance-none pr-[1.4rem]",
  "bg-no-repeat",
  "bg-[linear-gradient(45deg,transparent_50%,var(--fg-muted)_50%),linear-gradient(135deg,var(--fg-muted)_50%,transparent_50%)]",
  "bg-[position:calc(100%-0.8rem)_50%,calc(100%-0.5rem)_50%]",
  "bg-[size:0.3rem_0.3rem]",
].join(" ");

export const Select: Component<
  { class?: string } & JSX.SelectHTMLAttributes<HTMLSelectElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["class", "children"]);
  return (
    <select {...rest} class={`${CONTROL_CLASS} ${CARET} ${own.class ?? ""}`}>
      {own.children}
    </select>
  );
};
