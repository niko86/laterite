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

import { For, type Component } from "solid-js";
import { Button, ThemeToggle } from "@shared/components";
import mark from "../../../assets/laterite-icon-128.png";

const NAV = [
  // Demo before Install (#533): the nav promises the order the page keeps.
  { href: "#file", label: "Demo" },
  { href: "#install", label: "Install" },
  { href: "https://docs.laterite.dev/", label: "Docs" },
  { href: "https://github.com/niko86/laterite", label: "Source" },
];

export const Masthead: Component = () => (
  <header class="sticky top-0 z-30 bg-surface">
    <div class="flex items-center gap-4 px-4 py-2.5 sm:px-6">
      <a
        href="/"
        class="flex items-center gap-2.5 no-underline"
        aria-label="laterite — home"
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
        <ThemeToggle />
        <Button variant="primary" size="sm" href="https://app.laterite.dev/">
          Open the app
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
