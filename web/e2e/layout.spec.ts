import { test, expect, type Page } from "@playwright/test";
import { ready, load, enterExplore, tab } from "./helpers";

// Responsive layout checks. This spec is viewport-aware and runs under BOTH the
// desktop `chromium` (1280) project and the `mobile` (390) project, asserting
// the right behaviour for each width. The headline guard is "the page never
// scrolls horizontally" — the single best catch for a layout that escapes a
// phone's width (a missing min-w-0, an overflowing chip row, a wide table).

const isMobile = (page: Page) => (page.viewportSize()?.width ?? 1280) < 768;

async function expectNoHScroll(page: Page) {
  const overflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth -
      document.documentElement.clientWidth,
  );
  expect(overflow, "page must not scroll horizontally").toBeLessThanOrEqual(1);
}

test("Shell: the lockup, the centred column, the full-width tab hairline, the toast layer", async ({
  page,
}) => {
  await ready(page);

  // The lockup (#407): the brand word sets in the display face at 800; the
  // product name sets in the UI face at 600 — never the display face, which
  // is the rule the system spells out (a long product name in a heavy display
  // weight reads as packaging).
  const h1 = page.getByRole("heading", { level: 1 });
  await expect(h1).toContainText("laterite");
  await expect(h1).toContainText("AGS4 Validator");
  const brand = h1.locator("span", { hasText: /^laterite$/ });
  const product = h1.locator("span", { hasText: /^AGS4 Validator$/ });
  expect(
    await brand.evaluate((el) => getComputedStyle(el).fontFamily),
  ).toContain("Figtree");
  expect(await brand.evaluate((el) => getComputedStyle(el).fontWeight)).toBe(
    "800",
  );
  expect(
    await product.evaluate((el) => getComputedStyle(el).fontFamily),
  ).toContain("Public Sans");
  expect(await product.evaluate((el) => getComputedStyle(el).fontWeight)).toBe(
    "600",
  );

  // The shell centres at the system's width with its fixed gutters.
  const main = page.locator("main");
  expect(await main.evaluate((el) => getComputedStyle(el).maxWidth)).toBe(
    "1280px",
  );
  expect(await main.evaluate((el) => getComputedStyle(el).paddingLeft)).toBe(
    "20px",
  );

  // The tab bar's hairline runs the full viewport width even though the strip
  // centres, and the active tab is marked by the 2px accent underline.
  const hairline = page.locator('nav[aria-label="Sections"] > div').first();
  const hairlineWidth = await hairline.evaluate((el) => el.clientWidth);
  // Full width = the document's client width (the viewport minus any
  // scrollbar), not the nominal viewport size.
  expect(hairlineWidth).toBe(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
  const active = page.getByRole("tab", { selected: true });
  expect(
    await active.evaluate((el) => getComputedStyle(el).borderBottomWidth),
  ).toBe("2px");

  // The toast host layers above everything — its z-index token must RESOLVE
  // (the bracket-var regression this branch fixes left it invalid, which
  // computes as auto).
  const toastHost = page.locator('[class*="z-(--z-toast)"]');
  expect(await toastHost.evaluate((el) => getComputedStyle(el).zIndex)).toBe(
    "60",
  );

  await expectNoHScroll(page);
});

test("Fix: the whole-file diff scrolls inside its cap instead of growing the page", async ({
  page,
}) => {
  await ready(page);
  await load(page, "fixable.ags");
  await tab(page, "Fix").click();
  // The diff pre only renders once the current file differs from the original,
  // so apply the safe fixes first to give it content.
  await page.getByRole("button", { name: /Fix all safe/ }).click();
  await page.getByRole("button", { name: /^Diff$/ }).click();
  const pre = page.locator("pre.scroll-region");
  await expect(pre).toBeVisible();
  const overflowY = await pre.evaluate((el) => getComputedStyle(el).overflowY);
  expect(["auto", "scroll"]).toContain(overflowY);
  expect(await pre.evaluate((el) => getComputedStyle(el).maxHeight)).not.toBe(
    "none",
  );
  await expectNoHScroll(page);
});

test("Validate: fits the viewport; the sample list collapses once a file is loaded", async ({
  page,
}) => {
  await ready(page);
  // Empty editor → the sample list is open (its 'Clean (minimal)' button shows;
  // ready() itself relies on that).
  await expect(
    page.getByRole("button", { name: /Clean \(minimal\)/ }),
  ).toBeVisible();
  // Load a file → the editor fills → the sample list collapses to its summary,
  // reclaiming the vertical space the owner flagged as ballooning.
  await load(page, "fixable.ags");
  await expect(
    page.getByRole("button", { name: /Clean \(minimal\)/ }),
  ).toBeHidden({ timeout: 15_000 });
  await expectNoHScroll(page);
});

test("Explore: fits the viewport; capped sidebar on desktop, group dropdown on mobile; SQL examples collapse on a phone", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await ready(page);
  await load(page, "fixable.ags");
  await enterExplore(page);
  await expectNoHScroll(page);

  const sidebarFilter = page.getByLabel("filter groups");
  const groupSelect = page.getByLabel("group", { exact: true });
  if (isMobile(page)) {
    // The long button column is replaced by a compact dropdown.
    await expect(groupSelect).toBeVisible();
    await expect(sidebarFilter).toBeHidden();
  } else {
    await expect(sidebarFilter).toBeVisible();
    await expect(groupSelect).toBeHidden();
    // The sidebar is height-capped + internally scrollable, not unbounded.
    const aside = page.locator("aside").first();
    const overflowY = await aside.evaluate(
      (el) => getComputedStyle(el).overflowY,
    );
    expect(["auto", "scroll"]).toContain(overflowY);
    expect(
      await aside.evaluate((el) => getComputedStyle(el).maxHeight),
    ).not.toBe("none");
  }

  // SQL view: the Examples panel is open on a wide screen, collapsed on a phone.
  await page.getByRole("button", { name: "SQL" }).click();
  const listTables = page.getByRole("button", { name: "list tables" });
  if (isMobile(page)) await expect(listTables).toBeHidden();
  else await expect(listTables).toBeVisible();
  await expectNoHScroll(page);
});
