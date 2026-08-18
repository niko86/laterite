import { splitProps, type Component, type JSX } from "solid-js";

// A count on a toolbar button or panel header — and the only place the 0.65rem
// step is allowed. Square-ish and mono: an instrument readout, not a
// notification dot, which is why it does not go round and never turns red just
// for being non-zero.

export type CountBubbleTone = "warn" | "accent" | "err" | "info" | "muted";

const TONES: Record<CountBubbleTone, string> = {
  warn: "bg-warn",
  accent: "bg-accent",
  err: "bg-err",
  info: "bg-info",
  muted: "bg-fg-dim",
};

export const CountBubble: Component<
  {
    tone?: CountBubbleTone;
    class?: string;
  } & JSX.HTMLAttributes<HTMLSpanElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["tone", "class", "children"]);
  return (
    <span
      {...rest}
      class={[
        "inline-grid place-items-center min-w-[1.15rem] h-[1.1rem] px-[0.22rem]",
        "rounded-xs font-mono font-bold text-[length:var(--size-bubble)] text-fg-on-cta",
        TONES[own.tone ?? "warn"],
        own.class ?? "",
      ].join(" ")}
    >
      {own.children}
    </span>
  );
};
