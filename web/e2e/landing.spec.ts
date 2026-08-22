import { test, expect, type Page } from "@playwright/test";

// The landing page (laterite.dev) at a STRICT 390px phone viewport — no
// `isMobile`, deliberately: mobile emulation absorbs a too-wide layout into
// zoom, so the page "fits" while every offender becomes invisible and every
// tap target shrinks. A strict viewport makes the overflow measurable, which
// is how #523 was found: the group-section grids let the nowrap tables size
// the page, and a 390px phone got a 783px layout viewport zoomed out to fit.
//
// This spec runs only under the `landing` project (playwright.config.ts),
// against the landing's OWN preview server — the landing is a separate build
// (see web/landing/vite.config.ts), so the app's server cannot serve it.

const GROUPS = ["proj", "loca", "samp", "llpl"] as const;

/** The scroller that owns each group table's horizontal pan — the table's own
 *  wrapper, located through the table so the spec doesn't couple to a utility
 *  class name. */
const scroller = (page: Page, section: string) =>
  page.locator(`section#${section} table`).locator("xpath=..");

async function expectViewportWide(page: Page) {
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
  expect(doc.scrollWidth).toBeLessThanOrEqual(390);
}

test("phone: the page stays viewport-wide and each group table pans in its own scroller", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

  await expectViewportWide(page);

  // Each table must overflow ITS OWN scroller and pan there — the design
  // contract on the table component: "nine columns on a 390px phone must move
  // the table, not the page". Before the fix all four scrollers reported
  // clientWidth == scrollWidth: the page had grown instead.
  for (const section of GROUPS) {
    const el = scroller(page, section);
    const geo = await el.evaluate((node) => ({
      scrollWidth: node.scrollWidth,
      clientWidth: node.clientWidth,
    }));
    expect(
      geo.scrollWidth,
      `the ${section} table must overflow its scroller at phone width`,
    ).toBeGreaterThan(geo.clientWidth);

    const panned = await el.evaluate((node) => {
      node.scrollLeft = 100;
      return node.scrollLeft;
    });
    expect(panned, `the ${section} scroller must pan`).toBeGreaterThan(0);
  }
});

test("phone: opening the row editor does not widen the page", async ({
  page,
}) => {
  await page.goto("/");

  // LLPL is the widest table (nine columns, every cell nowrap) and the row
  // editor is the pattern chosen to carry it at 390px — so it is the
  // pairing most able to push the page out. The first column is sticky, so
  // its cell is tappable without panning.
  await page
    .getByRole("button", { name: "Edit LOCA_ID on row 1 of LLPL" })
    .click();
  await expect(
    page.getByRole("group", { name: "Editing row 1 of LLPL" }),
  ).toBeVisible();

  await expectViewportWide(page);
});
