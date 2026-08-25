/* The footer (#395; prose retired to reader-facts in #532).
 *
 * Two claims and no navigation. The differs-from-python-ags4 paragraph sits
 * here rather than in the hero because it answers a question a reader only
 * asks once they have decided to care — and what they need then is where the
 * differences are written down, not the project's licensing rationale. */

import type { Component } from "solid-js";

export const Footer: Component = () => (
  <footer class="border-t border-line px-4 py-8 text-caption text-fg-muted sm:px-6">
    <div class="mx-auto max-w-[72rem]">
      <p>
        MIT licensed.{" "}
        <a
          class="text-accent underline underline-offset-2 decoration-1 transition-colors hover:decoration-2"
          href="https://github.com/niko86/laterite/blob/main/LICENSE"
        >
          Read the licence
        </a>
        .
      </p>
      <p class="mt-2 max-w-[70ch]">
        laterite implements the published AGS4 rules independently of
        python-ags4, so results can differ between the two;{" "}
        <a
          class="text-accent underline underline-offset-2 decoration-1 transition-colors hover:decoration-2"
          href="https://github.com/niko86/laterite/blob/main/COMPAT.md"
        >
          COMPAT.md
        </a>{" "}
        and{" "}
        <a
          class="text-accent underline underline-offset-2 decoration-1 transition-colors hover:decoration-2"
          href="https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md"
        >
          OBSERVATIONS.md
        </a>{" "}
        record where and why.
      </p>
      <p class="mt-2">
        AGS4 is a format of the Association of Geotechnical and Geoenvironmental
        Specialists. laterite is not affiliated with the AGS.
      </p>
    </div>
  </footer>
);
