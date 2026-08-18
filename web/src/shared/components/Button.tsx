import { splitProps, type Component, type JSX } from "solid-js";
import { Dynamic } from "solid-js/web";

// The brand's button families. One component, six variants — the app shipped
// five of them as five copies; this is the extraction, plus the outline CTA
// #395 needed beside the rust fill.
//
// THE FILLED VARIANTS ARE RUST, NOT THE SYSTEM'S ACCENT. The design system
// fills `primary` with `var(--accent)`, which was brand brick when it was
// written. #394 resolved `--accent` to maroon and gave rust its own `--cta`, on
// "maroon reads, rust acts" — so implementing that contract literally would
// make every commit button the same colour as the prose around it. The prop
// names, the variants and the metrics are the system's; the colour role is the
// newer decision. `add` and `ghost` keep maroon, because they read as links
// rather than commits.

export type ButtonVariant =
  "default" | "primary" | "action" | "add" | "ghost" | "outline";
export type ButtonSize = "sm" | "md" | "lg";
export type ButtonTone = "neutral" | "danger";

const BASE =
  "inline-flex items-center gap-[0.4rem] font-ui leading-normal cursor-pointer " +
  "transition-colors duration-(--dur-base) ease-(--ease-out) " +
  "focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]";

const VARIANTS: Record<ButtonVariant, string> = {
  // Toolbar text button.
  default:
    "border border-line bg-surface text-fg rounded-md px-[0.8rem] py-[0.3rem] hover:bg-chip",
  // Filled commit.
  primary:
    "border border-cta bg-cta text-fg-on-cta rounded-md px-[0.8rem] py-[0.3rem] " +
    "font-semibold hover:bg-cta-hover hover:border-cta-hover",
  // "Runs something" — tinted wash, rust text.
  action:
    "border border-cta bg-cta-quiet text-cta rounded-md px-[0.9rem] py-[0.26rem] " +
    "font-semibold hover:text-cta-hover hover:border-cta-hover",
  // Dashed "+ thing" affordance.
  add:
    "border border-dashed border-line-strong bg-surface text-accent rounded-xs " +
    "px-[0.5rem] py-[0.15rem] hover:text-accent-hover hover:border-accent",
  // Quiet icon / ✕ button, muted until hover.
  ghost:
    "bg-transparent text-fg-muted rounded-xs px-[0.3rem] py-[0.1rem] hover:text-fg",
  // The secondary CTA (#395): maroon outline on the canvas, beside the rust
  // fill. Accent rather than cta on purpose — "maroon reads, rust acts", so the
  // quieter of two adjacent calls to action takes the reading colour, and the
  // page never shows two rust buttons competing to be pressed.
  outline:
    "border border-accent bg-transparent text-accent rounded-md px-[0.8rem] " +
    "py-[0.3rem] font-semibold hover:bg-accent-quiet hover:text-accent-hover",
};

// `md` is the unstyled middle rung — the variant's own padding stands.
const SIZES: Record<ButtonSize, string> = {
  sm: "text-micro px-[0.55rem] py-[0.2rem]",
  md: "",
  lg: "text-body px-[1rem] py-[0.4rem]",
};

/** Destructive repaint. A ghost has no border to recolour, so it loses one. */
const danger = (variant: ButtonVariant): string =>
  variant === "primary"
    ? "bg-err border-err text-fg-on-cta hover:bg-err hover:border-err"
    : variant === "ghost"
      ? "text-err border-transparent hover:text-err"
      : "text-err border-err hover:text-err";

export const Button: Component<
  {
    variant?: ButtonVariant;
    size?: ButtonSize;
    tone?: ButtonTone;
    iconLeft?: JSX.Element;
    iconRight?: JSX.Element;
    class?: string;
    /** Render an anchor instead of a button.
     *
     * A call to action that NAVIGATES is a link, and has to behave like one —
     * middle-clickable, copyable, crawlable, and reachable with the keyboard's
     * link semantics. The landing page's two hero CTAs and its masthead CTA all
     * navigate (#395). Styling an anchor to look like this button from the
     * calling surface would be the second button the shared layer exists to
     * prevent, so the polymorphism lives here. */
    href?: string;
    target?: string;
    rel?: string;
  } & JSX.ButtonHTMLAttributes<HTMLButtonElement>
> = (props) => {
  const [own, rest] = splitProps(props, [
    "variant",
    "size",
    "tone",
    "iconLeft",
    "iconRight",
    "class",
    "children",
    "href",
  ]);
  const variant = () => own.variant ?? "default";
  return (
    <Dynamic
      component={own.href ? "a" : "button"}
      type={own.href ? undefined : "button"}
      href={own.href}
      {...rest}
      class={[
        BASE,
        VARIANTS[variant()],
        SIZES[own.size ?? "md"],
        own.tone === "danger" ? danger(variant()) : "",
        // Never a grey repaint — the control keeps its colour and loses its
        // affordance, so a disabled primary still reads as the primary action.
        props.disabled ? "opacity-45 cursor-default" : "",
        own.class ?? "",
      ].join(" ")}
    >
      {own.iconLeft}
      {own.children}
      {own.iconRight}
    </Dynamic>
  );
};
