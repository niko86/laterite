import { splitProps, type Component, type JSX } from "solid-js";

// Small label — verdicts, filters, group codes, counts.
//
// Three forms, because severity must survive greyscale: `rule` (default) is a
// tinted block with a 3px coloured left edge, like a stratum tick; `solid` is a
// filled block for the loudest state; `outline` is a hairline stencil for calm
// or verified states. Never a soft pastel pill.
//
// Mono and uppercase by default, because most of what goes in one is an AGS4
// code. `sentence` switches it to the UI face for the cases that are prose.

export type ChipTone =
  "neutral" | "accent" | "ok" | "warn" | "err" | "info" | "muted";
export type ChipVariant = "rule" | "solid" | "outline";

const RULE: Record<ChipTone, string> = {
  neutral: "bg-surface-raised text-fg-soft border-l-line-strong",
  accent: "bg-accent-quiet text-accent border-l-accent",
  ok: "bg-ok-quiet text-ok border-l-ok",
  warn: "bg-warn-quiet text-warn border-l-warn",
  err: "bg-err-quiet text-err border-l-err",
  info: "bg-info-quiet text-info border-l-info",
  muted: "bg-surface-raised text-fg-muted border-l-line-strong",
};

// On-fill text is `text-surface`, not `text-fg-on-cta` (#404): the status and
// accent fills LIGHTEN in dark, so a fixed near-white foreground vanishes on
// them — the surface token flips to the dark ground with the theme, keeping
// dark text on light fills there and light text on dark fills in light. The
// CTA is the one fill that never changes, which is why fg-on-cta exists and
// stays fixed; none of these fills has that property.
const SOLID: Record<ChipTone, string> = {
  neutral: "bg-fg-soft text-surface",
  accent: "bg-accent text-surface",
  ok: "bg-ok text-surface",
  warn: "bg-warn text-surface",
  err: "bg-err text-surface",
  info: "bg-info text-surface",
  muted: "bg-fg-soft text-surface",
};

const OUTLINE: Record<ChipTone, string> = {
  neutral: "text-fg-soft border-line-strong/55",
  accent: "text-accent border-accent/55",
  ok: "text-ok border-ok/55",
  warn: "text-warn border-warn/55",
  err: "text-err border-err/55",
  info: "text-info border-info/55",
  muted: "text-fg-muted border-line-strong/55",
};

export const Chip: Component<
  {
    tone?: ChipTone;
    variant?: ChipVariant;
    /** Prose rather than a code: the UI face, sentence case, normal tracking. */
    sentence?: boolean;
    class?: string;
  } & JSX.HTMLAttributes<HTMLSpanElement>
> = (props) => {
  const [own, rest] = splitProps(props, [
    "tone",
    "variant",
    "sentence",
    "class",
    "children",
  ]);
  const tone = () => own.tone ?? "neutral";
  const form = () => {
    switch (own.variant ?? "rule") {
      case "solid":
        return `${SOLID[tone()]} px-[0.5rem] py-[0.14rem]`;
      case "outline":
        return `${OUTLINE[tone()]} bg-transparent border px-[0.45rem] py-[0.12rem]`;
      default:
        return `${RULE[tone()]} border-l-[3px] pl-[0.4rem] pr-[0.45rem] py-[0.14rem]`;
    }
  };
  return (
    <span
      {...rest}
      class={[
        "inline-flex items-center gap-[0.3rem] whitespace-nowrap rounded-xs font-semibold text-micro",
        own.sentence ? "font-ui" : "font-mono uppercase tracking-micro",
        form(),
        own.class ?? "",
      ].join(" ")}
    >
      {own.children}
    </span>
  );
};
