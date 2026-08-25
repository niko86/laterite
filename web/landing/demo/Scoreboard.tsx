/* The scoreboard (#531): one verdict, two mounts.
 *
 * The same chip renders in the hero card and floats beside the demo, because
 * two components would eventually disagree — and a page whose header says
 * "valid" while its footer counts errors is the demo contradicting itself.
 * It renders nothing until the engine's first report lands: a count of zero
 * and "not yet counted" are different claims, and the chip only ever makes
 * the first.
 *
 * The pulse on change is an OPACITY dip, not a scale: the motion charter
 * (web/src/shared/styles/motion.css) sanctions colour and opacity changes
 * only — nothing scales in this design system. It sits behind Tailwind's
 * `motion-safe:` variant, so a reduced-motion reader gets the new number
 * with no theatrics, and the jump-to-findings scroll is instant for the
 * same reason (programmatic smooth scroll ignores the OS preference).
 */

import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  onMount,
  type Component,
} from "solid-js";
import { report } from "./store";
import { SECTIONS } from "../sections";
import { verdictTint } from "./severity";
import { REFUSED_LABEL, scoreboardLabel, verdictState } from "./verdict";

export const Scoreboard: Component = () => {
  /* The refusal is not a zero-count (#638): an errored run carries an
     empty findings list, and tallying that list dressed the chip as
     "\u2713 valid AGS4" over a run the engine refused. The claim comes from
     the state's KIND; only a counted state can ever be clean. */
  const state = createMemo(() => {
    const r = report();
    return r ? verdictState(r) : null;
  });
  const clean = () => {
    const s = state();
    return (
      s?.kind === "counted" && s.tally.errors === 0 && s.tally.warnings === 0
    );
  };
  const label = () => {
    const s = state();
    if (!s) return null;
    return s.kind === "refused" ? REFUSED_LABEL : scoreboardLabel(s.tally);
  };

  const [bump, setBump] = createSignal(false);
  /** How long the dip holds before the fade back — a chosen beat, not the
   *  system's --dur-base, which times the fade itself. */
  const BUMP_HOLD_MS = 300;
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(
    on(
      label,
      (l, prev) => {
        // Only a CHANGE bumps — the first report arriving is not news the
        // reader caused.
        if (prev == null || l === prev) return;
        setBump(true);
        clearTimeout(timer);
        timer = setTimeout(() => {
          setBump(false);
        }, BUMP_HOLD_MS);
      },
      { defer: true },
    ),
  );
  onCleanup(() => {
    clearTimeout(timer);
  });

  return (
    <Show when={label()}>
      {(l) => (
        <button
          type="button"
          class={[
            "inline-flex cursor-pointer items-center rounded-full border px-3 py-1 font-mono text-micro font-semibold",
            "motion-safe:transition-opacity",
            "focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]",
            verdictTint(clean()),
            bump() ? "motion-safe:opacity-40" : "",
          ].join(" ")}
          onClick={() => {
            document.getElementById("findings")?.scrollIntoView({
              block: "start",
            });
          }}
        >
          {l()}
          <span class="sr-only">, jump to the findings panel</span>
        </button>
      )}
    </Show>
  );
};

/** The masthead mount: visible while any of the demo's tables is on screen,
 *  so the verdict follows the reader through the sections that can change it.
 *  IntersectionObserver over the table-bearing sections rather than a scroll
 *  listener — the browser already knows what is on screen.
 *
 *  DOCKED, not floating (#691). It was `fixed right-4 bottom-4`, which put a
 *  clickable overlay on top of whatever the page had in its bottom-right
 *  corner — and the demo's tables reach that corner at every width except the
 *  one where the content column stops short of it. Measured by walking the
 *  page and hit-testing under the chip: at 1280 it covered a fix button, at
 *  1024 a cell editor, at 390 three different controls at three scroll
 *  positions. Only 1440 was clean, and only by accident of gutter width.
 *
 *  The e2e suite could not catch it, and still cannot: Playwright scrolls a
 *  target into view before clicking, which lands it away from a viewport-fixed
 *  chip. A reader clicking where the button already is gets the chip.
 *
 *  In the masthead it overlaps nothing, keeps its door to the findings panel,
 *  and still follows the reader, because the masthead is sticky. It is hidden
 *  below the same breakpoint the text nav uses: the mobile masthead is full —
 *  measured at 6 free pixels against a chip that needs eighty-odd — and the
 *  phone already renders findings under each table, so the verdict is not the
 *  only feedback there. */
/** True while any table-bearing section is on screen. Shared by both mounts
 *  (#691), so the two can never disagree about when the verdict is live —
 *  IntersectionObserver over the sections rather than a scroll listener,
 *  because the browser already knows what is on screen. */
const useTablesOnScreen = () => {
  const [onScreen, setOnScreen] = createSignal(false);
  onMount(() => {
    // Derived from sections.ts, the page's one copy of the sequence: the
    // table-bearing sections are the group ones plus the file section.
    const els = SECTIONS.filter((sec) => sec.group || sec.id === "file")
      .map((sec) => document.getElementById(sec.id))
      .filter((el): el is HTMLElement => el !== null);
    const seen = new Set<Element>();
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) seen.add(e.target);
        else seen.delete(e.target);
      }
      setOnScreen(seen.size > 0);
    });
    for (const el of els) io.observe(el);
    onCleanup(() => {
      io.disconnect();
    });
  });
  return onScreen;
};

export const DockedScoreboard: Component = () => {
  const onScreen = useTablesOnScreen();
  return (
    <Show when={onScreen()}>
      <div data-scoreboard="verdict" class="hidden min-[52rem]:block">
        <Scoreboard />
      </div>
    </Show>
  );
};

/** The phone mount (#691). Below the breakpoint the masthead is full — measured
 *  at single-figure free pixels against a chip that needs eighty-odd — so the
 *  chip stays the corner float it has always been there, and stays in the page
 *  body, where its tab order matches where it draws. Mounting one chip in the
 *  header and letting it render as a corner float traded an overlap defect for
 *  a focus-order one, which is how this shape was arrived at.
 *
 *  #531 recorded the 390 touch reader as this chip's audience, so hiding it
 *  there instead was not this ticket's call. The overlap is therefore FIXED
 *  above the breakpoint and KNOWN below it, and the landing lane pins that
 *  state rather than skipping it — whoever solves the phone case is told by a
 *  red test that this comment is out of date.
 *
 *  Same `data-scoreboard` hook as the docked mount: callers want "the verdict
 *  the reader can see", never "the desktop one". Exactly one is ever visible. */
export const FloatingScoreboard: Component = () => {
  const onScreen = useTablesOnScreen();
  return (
    <Show when={onScreen()}>
      <div
        data-scoreboard="verdict"
        class="fixed right-4 bottom-4 z-30 min-[52rem]:hidden"
      >
        <Scoreboard />
      </div>
    </Show>
  );
};
