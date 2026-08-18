import { splitProps, type Component, type JSX } from "solid-js";

// A hard verdict, stencilled like a core-box label: mono, uppercase, wide
// tracking, hairline box. `solid` fills it for a failure you cannot miss.
//
// Distinct from Chip on purpose — a Chip labels a thing, a StatusBadge states
// the result. The 1.5px stencil border is the tell, and it is thicker than the
// system's hairline everywhere else so the box reads as stamped on.

export type StatusTone = "pass" | "fail" | "warn" | "unknown";
export type StatusVariant = "stencil" | "solid";

const STENCIL: Record<StatusTone, string> = {
  pass: "text-ok border-ok",
  fail: "text-err border-err",
  warn: "text-warn border-warn",
  unknown: "text-(--steel-500) border-(--steel-500)",
};

const SOLID: Record<StatusTone, string> = {
  pass: "bg-ok border-ok",
  fail: "bg-err border-err",
  warn: "bg-warn border-warn",
  unknown: "bg-(--steel-500) border-(--steel-500)",
};

export const StatusBadge: Component<
  {
    tone?: StatusTone;
    variant?: StatusVariant;
    class?: string;
  } & JSX.HTMLAttributes<HTMLSpanElement>
> = (props) => {
  const [own, rest] = splitProps(props, [
    "tone",
    "variant",
    "class",
    "children",
  ]);
  const tone = () => own.tone ?? "pass";
  const solid = () => (own.variant ?? "stencil") === "solid";
  return (
    <span
      {...rest}
      class={[
        "inline-block font-mono font-bold uppercase rounded-xs",
        "text-[length:var(--size-bubble)] tracking-[0.09em]",
        solid()
          ? `border ${SOLID[tone()]} text-fg-on-cta px-[0.5rem] py-[0.16rem]`
          : `border-[1.5px] bg-transparent ${STENCIL[tone()]} px-[0.45rem] py-[0.1rem]`,
        own.class ?? "",
      ].join(" ")}
    >
      {own.children}
    </span>
  );
};
