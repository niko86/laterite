/* The footer (#395).
 *
 * Two claims and no navigation. The clean-room statement is here rather than in
 * the hero because it answers a question a reader only asks once they have
 * decided to care — but it has to be answerable, because "another AGS4 library"
 * is the assumption this project spends its licence on not being.
 */

import type { Component } from "solid-js";

export const Footer: Component = () => (
  <footer class="border-t border-line px-4 py-8 text-caption text-fg-muted sm:px-6">
    <div class="mx-auto max-w-[72rem]">
      <p>
        MIT licensed.{" "}
        <a
          class="text-accent no-underline hover:underline"
          href="https://github.com/niko86/laterite/blob/main/LICENSE"
        >
          Read the licence
        </a>
        .
      </p>
      <p class="mt-2 max-w-[70ch]">
        The validator is clean-room from the published AGS4 specification, not
        adapted from another library's source. That separation is what lets
        laterite ship under MIT —{" "}
        <a
          class="text-accent no-underline hover:underline"
          href="https://github.com/niko86/laterite/blob/main/COMPAT.md"
        >
          COMPAT.md
        </a>{" "}
        and{" "}
        <a
          class="text-accent no-underline hover:underline"
          href="https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md"
        >
          OBSERVATIONS.md
        </a>{" "}
        record where it differs from python-ags4 and why.
      </p>
      <p class="mt-2">
        AGS4 is a format of the Association of Geotechnical and Geoenvironmental
        Specialists. laterite is not affiliated with the AGS.
      </p>
    </div>
  </footer>
);
