import { splitProps, type Component, type JSX } from "solid-js";
import { ICONS, type IconName } from "../icons/icons";

// A Lucide glyph, inlined so it inherits `currentColor` and needs no request.
//
// The design system's own Icon fetches each glyph from a CDN and caches it in a
// module map, and its readme flags that as the thing to change for an offline
// build. This is that change: the markup is vendored and bundled (see
// scripts/sync-icons.mjs), so there is no pending state, no fetch to fail, and
// no empty box on a cold offline load. That matters more here than it did
// there — the app is a PWA with a precache, and an icon set that 404s offline
// is a validator full of unlabelled buttons.
//
// The stroke is the one number worth explaining. Lucide draws on a 24-unit
// viewBox with `stroke-width="2"`, so at the system's 16px default a naive
// render gives 2 × 16/24 = 1.33px, not the 1.5px the system specifies. Scaling
// the stroke by the box keeps the weight at 1.5px at ANY size, which is what
// stops a 24px icon beside a 16px one looking like a different family.
const STROKE_PX = 1.5;
const VIEWBOX = 24;

export const Icon: Component<
  {
    name: IconName;
    size?: number;
    /**
     * The accessible name. Omit it ONLY when the icon sits beside real text
     * that already says this — then it is decorative and hidden from the
     * accessibility tree. An icon-only control must pass one (or carry a
     * Tooltip), which is the system's rule and the reason this is not optional
     * by default.
     */
    label?: string;
    class?: string;
  } & JSX.SvgSVGAttributes<SVGSVGElement>
> = (props) => {
  const [own, rest] = splitProps(props, ["name", "size", "label", "class"]);
  const size = () => own.size ?? 16;
  return (
    <svg
      {...rest}
      class={own.class}
      width={size()}
      height={size()}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      fill="none"
      stroke="currentColor"
      stroke-width={(STROKE_PX * VIEWBOX) / size()}
      stroke-linecap="round"
      stroke-linejoin="round"
      role={own.label ? "img" : undefined}
      aria-label={own.label}
      aria-hidden={own.label ? undefined : "true"}
      /* eslint-disable-next-line solid/no-innerhtml --
         The rule is about unsanitized INPUT, and there is no input here. The
         only values this can receive are the module-scope constants in
         icons/icons.ts, built at bundle time from .svg files vendored verbatim
         from a pinned `lucide-static` and refreshed only by scripts/sync-icons.mjs.
         `name` is typed to the manifest's own keys, so a call site cannot reach
         anything else — there is no path from user data, a URL or the network
         to this string. Inlining is the point: it is what removes the CDN fetch
         the design system's Icon depends on, and with it the offline failure. */
      innerHTML={ICONS[own.name]}
    />
  );
};
