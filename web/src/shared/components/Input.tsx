import { splitProps, type Component, type JSX } from "solid-js";

// The canonical text control. Everything else in the system — Select, the SQL
// console, the paste area — is this box with one property changed, which is why
// the class string is exported rather than copied.

export const CONTROL_CLASS = [
  "font-ui text-control w-full min-w-0",
  "px-[0.4rem] py-[0.25rem] rounded-xs",
  "border border-line-strong bg-surface-raised text-fg",
  "outline-none focus-visible:[box-shadow:var(--focus-ring)] focus-visible:border-accent",
].join(" ");

export const Input: Component<
  {
    /** AGS codes, numbers and file text — anything that should align in a column. */
    mono?: boolean;
    invalid?: boolean;
    class?: string;
  } & JSX.InputHTMLAttributes<HTMLInputElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["mono", "invalid", "class"]);
  return (
    <input
      {...rest}
      aria-invalid={own.invalid ? "true" : undefined}
      class={[
        CONTROL_CLASS,
        own.mono ? "font-mono" : "",
        own.invalid ? "border-err" : "",
        own.class ?? "",
      ].join(" ")}
    />
  );
};
