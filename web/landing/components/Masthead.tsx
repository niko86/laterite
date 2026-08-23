/* The masthead (#395): sticky chrome, closed by the strata hairline.
 *
 * The 3px gradient rule under it is the page's signature in miniature — the
 * same ramp the borehole rail runs vertically, laid on its side. It is the one
 * place the four-band gradient appears; a table cap uses a single solid band
 * (#396), because a gradient there would read as four groups rather than one.
 *
 * The chrome is OPAQUE (#408). It used to be a 95%-alpha surface over a small
 * backdrop filter — the last blurred surface anywhere, against a system whose
 * rule is that nothing is. Frosting the content sliding underneath cost a
 * compositor layer to say what the strata rule below already says: the chrome
 * is in front.
 *
 * The filter utility is NAMED here in prose rather than spelled as its class,
 * deliberately. Tailwind scans this file as raw text and cannot tell code from
 * comment, so writing the class out — even to record removing it — puts it back
 * in the generated CSS. The eslint gate cannot catch that either: it reads
 * string literals, not comments. Verified by rebuilding and grepping the
 * emitted stylesheet for the class, which is where the truth is.
 *
 * The brand lockup is the mark plus the wordmark in Figtree 800 maroon. The
 * display face and the wordmark are the same face now, so the two read as one
 * object rather than a logo beside a label. On light chrome the mark sits bare;
 * on dark it takes a plate (#400), which is why the plate classes are on the
 * <img> rather than baked into the asset.
 */

import { For, type Component, type JSX } from "solid-js";
import { Button, Icon, ThemeToggle } from "@shared/components";
import mark from "../../../assets/laterite-icon-128.png";

const NAV = [
  // Demo before Install (#533): the nav promises the order the page keeps.
  { href: "#file", label: "Demo" },
  { href: "#install", label: "Install" },
  { href: "https://docs.laterite.dev/", label: "Docs" },
  { href: "https://github.com/niko86/laterite", label: "Source" },
];

/* The GitHub MARK, not a Lucide glyph — deliberately outside the vendored
   icon set. The pinned lucide-static dropped its brand icons, and the icon
   system's own rule for that case is "say so, never hand-draw a lookalike":
   this is the saying-so. The path is GitHub's official mark, vendored
   verbatim (their logo policy invites exactly this use — a mark that links
   to a GitHub presence), filled with currentColor so it takes the same ink
   as the Lucide glyph beside it in both themes.

   18px where the Lucide glyph gets 20: the mark fills its 16-unit box
   edge-to-edge while Lucide draws inside a ~2-unit gutter of its 24, so
   equal boxes render the mark visibly heavier — the two only read as one
   nav with the box compensating for the gutter. */
const GithubMark: Component = () => (
  <svg
    viewBox="0 0 16 16"
    width="18"
    height="18"
    fill="currentColor"
    aria-hidden="true"
  >
    <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
  </svg>
);

/* One mobile icon link, wearing the theme toggle's box (#621, superseding
   #597's bare-icon ruling; resized by #631 after the 44px square dwarfed
   the family it joined). The box takes the TOGGLE'S OWN vertical recipe —
   its padding around a glyph slot sized to its text line. The slot and
   the line are two independent theme knobs that agree today; the e2e
   height-equality assertions are the gate that holds them together if
   either retunes. The 44px tap floor survives as the centred
   pseudo-element hit square, probed on both axes by the same suite; the
   cluster's gap is one step wider than #621's so adjacent hit squares
   never overlap. Hidden exactly where the text nav appears. */
const IconLink: Component<{
  href: string;
  label: string;
  children: JSX.Element;
}> = (props) => (
  <a
    href={props.href}
    aria-label={props.label}
    class="relative flex items-center justify-center rounded-sm border border-line-strong px-2 py-1 text-fg-soft transition-colors before:absolute before:top-1/2 before:left-1/2 before:size-11 before:-translate-x-1/2 before:-translate-y-1/2 before:content-[''] hover:border-accent hover:text-accent focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)] min-[52rem]:hidden"
  >
    <span class="flex size-5 items-center justify-center">
      {props.children}
    </span>
  </a>
);

export const Masthead: Component = () => (
  <header class="sticky top-0 z-30 bg-surface">
    <div class="flex items-center gap-4 px-4 py-2.5 sm:px-6">
      <a
        href="/"
        class="flex items-center gap-2.5 no-underline"
        aria-label="laterite home"
      >
        <img
          src={mark}
          alt=""
          width="28"
          height="28"
          /* The plate is dark-chrome only (#400): the mark's outline is maroon
             and disappears into a near-black canvas without it.

             The ramp token by reference, not Tailwind's own stone background
             utility — that palette is the COOL stone, while ours is warm; the
             two differ by enough to read as a grey chip behind a warm mark.
             The class is named in prose, not spelled, for the same reason as
             the filter note above: spelling it here re-emits it. No semantic token
             fits: every surface role flips dark with the theme, and this plate
             must stay light in dark chrome, which is the whole point of it. */
          class="size-7 rounded-[5px] dark:bg-[var(--stone-50)] dark:p-[3px]"
        />
        <span class="font-display text-h3 font-extrabold tracking-(--track-tight) text-accent">
          laterite
        </span>
      </a>

      <nav class="ml-auto hidden items-center gap-5 min-[52rem]:flex">
        <For each={NAV}>
          {(item) => (
            <a
              href={item.href}
              class="text-caption text-fg-soft no-underline transition-colors hover:text-accent hover:underline"
            >
              {item.label}
            </a>
          )}
        </For>
      </nav>

      <div class="ml-auto flex items-center gap-2 min-[52rem]:ml-0">
        {/* The mobile nav is these two icons (#597): the text nav is hidden
            below 52rem, which used to leave a phone no path to the source or
            the install anchor from the top bar. Iconography over text HERE
            only — the CTA keeps its words (#586). The install glyph is the
            arrow-into-tray; the jump rides the document's smooth-scroll rule
            (#589) like every anchor. */}
        <IconLink
          href="https://github.com/niko86/laterite"
          label="Source on GitHub"
        >
          <GithubMark />
        </IconLink>
        <IconLink href="#install" label="Jump to install">
          <Icon name="download" size={20} />
        </IconLink>
        <ThemeToggle />
        <Button variant="primary" size="sm" href="https://app.laterite.dev/">
          Open webapp
        </Button>
      </div>
    </div>

    {/* Not a border: a border cannot carry a gradient. 3px, and the ramp runs
        the same direction the rail runs down. */}
    <div
      aria-hidden="true"
      class="h-[3px] w-full"
      style={{
        background:
          "linear-gradient(to right, var(--laterite-300), var(--laterite-400), " +
          "var(--laterite-500), var(--laterite-600), var(--laterite-900))",
      }}
    />
  </header>
);
