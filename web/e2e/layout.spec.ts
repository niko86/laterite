import { test, expect, type Page } from "@playwright/test";
import { ready, load, enterExplore } from "./helpers";

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
