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
import { scoreboardLabel, tally } from "./verdict";

export const Scoreboard: Component = () => {
  const counts = createMemo(() => {
    const r = report();
    return r ? tally(r.findings) : null;
  });
  const clean = () => {
    const t = counts();
    return t !== null && t.errors === 0 && t.warnings === 0;
  };

  const [bump, setBump] = createSignal(false);
  /** How long the dip holds before the fade back — a chosen beat, not the
   *  system's --dur-base, which times the fade itself. */
  const BUMP_HOLD_MS = 300;
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(
    on(
      () => {
        const t = counts();
        return t ? scoreboardLabel(t) : null;
      },
      (label, prev) => {
        // Only a CHANGE bumps — the first report arriving is not news the
        // reader caused.
        if (prev == null || label === prev) return;
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
    <Show when={counts()}>
      {(t) => (
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
          {scoreboardLabel(t())}
          <span class="sr-only">, jump to the findings panel</span>
        </button>
      )}
    </Show>
  );
};

/** The floating mount: visible while any of the demo's tables is on screen,
 *  so the verdict follows the reader through the sections that can change
 *  it. IntersectionObserver over the table-bearing sections rather than a
 *  scroll listener — the browser already knows what is on screen. */
export const FloatingScoreboard: Component = () => {
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
  return (
    <Show when={onScreen()}>
      <div data-scoreboard="floating" class="fixed right-4 bottom-4 z-30">
        <Scoreboard />
      </div>
    </Show>
  );
};
