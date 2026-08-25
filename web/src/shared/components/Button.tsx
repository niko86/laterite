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

// Shape and colour are separate maps so a tone can REPLACE a variant's
// colours rather than pile more colour utilities on top: two utilities for
// the same property tie on specificity, stylesheet order picks the winner,
// and that is how the danger repaint silently lost to the variant coat
// (#593) — ghost's `.text-fg-muted` sorts after `.text-err` in the compiled
// sheet, so a danger ghost rendered muted, and default's `.border-line`
// beat `.border-err` the same way.
const SHAPES: Record<ButtonVariant, string> = {
  // Toolbar text button.
  default: "border rounded-md px-[0.8rem] py-[0.3rem]",
  // Filled commit.
  primary: "border rounded-md px-[0.8rem] py-[0.3rem] font-semibold",
  // "Runs something".
  action: "border rounded-md px-[0.9rem] py-[0.26rem] font-semibold",
  // Dashed "+ thing" affordance.
  add: "border border-dashed rounded-xs px-[0.5rem] py-[0.15rem]",
  // Quiet icon / ✕ button.
  ghost: "rounded-xs px-[0.3rem] py-[0.1rem]",
  // The secondary CTA (#395).
  outline: "border rounded-md px-[0.8rem] py-[0.3rem] font-semibold",
};

const COLORS: Record<ButtonVariant, string> = {
  default: "border-line bg-surface text-fg hover:bg-chip",
  // The border does NOT follow the fill on hover (#682). It is what carries the
  // control's edge against the page (1.4.11), and the hover band is dark enough
  // that in the dark theme it sinks into the canvas — under the boundary floor
  // contrast.test.ts holds, where the resting band clears it. So the fill
  // darkens for feedback and the border stays put, holding the edge.
  primary: "border-cta bg-cta text-fg-on-cta hover:bg-cta-hover",
  // Rust tint, rust edge, MAROON label (#682). The wash and the border stay
  // rust because #406 is right that an action which runs should sit in a rust
  // tint. The label is the one part that is text, and rust cannot carry text at
  // this size: on its own wash it lands nearer the boundary floor than the text
  // one. Maroon clears the text bar with room, and it is the same call
  // `outline` below already made, for the same reason.
  action:
    "border-cta bg-cta-quiet text-accent hover:text-accent-hover hover:bg-cta-quiet",
  add: "border-line-strong bg-surface text-accent hover:text-accent-hover hover:border-accent",
  // Muted until hover.
  ghost: "bg-transparent text-fg-muted hover:text-fg",
  // Maroon outline on the canvas, beside the rust fill. Accent rather than
  // cta on purpose — "maroon reads, rust acts", so the quieter of two
  // adjacent calls to action takes the reading colour, and the page never
  // shows two rust buttons competing to be pressed.
  outline:
    "border-accent bg-transparent text-accent hover:bg-accent-quiet hover:text-accent-hover",
};

/** Destructive repaint: the full colour coat per variant, err where the
 *  variant's own hue was. */
const DANGER: Record<ButtonVariant, string> = {
  default: "border-err bg-surface text-err hover:bg-chip hover:text-err",
  primary: "border-err bg-err text-fg-on-cta hover:bg-err hover:border-err",
  action: "border-err bg-cta-quiet text-err hover:text-err",
  add: "border-err bg-surface text-err hover:text-err",
  ghost: "bg-transparent text-err hover:text-err",
  outline:
    "border-err bg-transparent text-err hover:bg-accent-quiet hover:text-err",
};

// `md` is the unstyled middle rung — the variant's own padding stands.
const SIZES: Record<ButtonSize, string> = {
  sm: "text-micro px-[0.55rem] py-[0.2rem]",
  md: "",
  lg: "text-body px-[1rem] py-[0.4rem]",
};

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
        SHAPES[variant()],
        (own.tone === "danger" ? DANGER : COLORS)[variant()],
        SIZES[own.size ?? "md"],
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
