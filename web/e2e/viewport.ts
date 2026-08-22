/* The viewport-wide contract, shared by the light lanes (landing.spec.ts)
 * and the dark lane (landing.dark.spec.ts) — one contract, one definition,
 * so the two cannot drift (#547 moved it here from landing.spec.ts). */

import { expect, type Page } from "@playwright/test";

export async function expectViewportWide(page: Page) {
  const doc = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
  }));
  // Exactly viewport-wide, no tolerance: any horizontal overflow is the bug.
  // Against clientWidth rather than a literal 390 so a runner that draws
  // space-taking scrollbars measures the same contract — with #523 unfixed,
  // scrollWidth read ~783 while clientWidth stayed at the viewport. The
  // ceiling closes the remaining direction, both numbers growing together
  // (a wrong server or a wrong project viewport).
  expect(doc.scrollWidth, "the page must not outgrow the viewport").toBe(
    doc.clientWidth,
  );
}
