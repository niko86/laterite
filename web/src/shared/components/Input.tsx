import { splitProps, type Component, type JSX } from "solid-js";

// The canonical text control. Everything else in the system — Select, the SQL
// console, the paste area — is this box with one property changed, which is why
// the class string is exported rather than copied.
//
// Width and font ride OUTSIDE the exported string, one class per axis: two
// utilities on the same property resolve by stylesheet order, not author
// intent, so a caller's `w-48` or `font-mono` beside a baked `w-full`/`font-ui`
// is silently inert (the baked one happens to sort later). The `width` prop
// REPLACES the full-width default rather than composing with it, and `mono` is
// a ternary rather than an addition, so the fight can't occur.

export const CONTROL_CLASS = [
  "text-control",
  "px-[0.4rem] py-[0.25rem] rounded-xs",
  "border border-line-strong bg-surface-raised text-fg",
  // outline-hidden so forced-colors mode (which discards the box-shadow ring)
  // still gets an outline to repaint.
  "outline-hidden focus-visible:[box-shadow:var(--focus-ring)] focus-visible:border-accent",
].join(" ");

export const Input: Component<
  {
    /** AGS codes, numbers and file text — anything that should align in a column. */
    mono?: boolean;
    invalid?: boolean;
    /** Sizing utilities (`w-48`, `min-w-0 flex-1`); replaces `w-full min-w-0`. */
    width?: string;
    class?: string;
  } & JSX.InputHTMLAttributes<HTMLInputElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["mono", "invalid", "width", "class"]);
  return (
    <input
      {...rest}
      aria-invalid={own.invalid ? "true" : undefined}
      class={[
        own.width ?? "w-full min-w-0",
        own.mono ? "font-mono" : "font-ui",
        CONTROL_CLASS,
        own.invalid ? "border-err" : "",
        own.class ?? "",
      ].join(" ")}
    />
  );
};
