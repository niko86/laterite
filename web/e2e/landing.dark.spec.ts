/* The dark lane (#547): every "renders in both themes" criterion before this
 * was carried by the token pattern alone — the separation unit tests catch a
 * forgone fill statically, but nothing ever rendered the dark page. This
 * lane arrives with the system preference dark, which is the real visitor
 * path: index.html's bootstrap turns that preference into the `.dark` class
 * the variant is defined on (landing.css's `@custom-variant`). Assertions
 * stay at the token level — screenshots are out of the issue's scope, and
 * unnecessary for what this lane exists to catch.
 */

import { test, expect } from "@playwright/test";
import { INSTALL_CHANNELS } from "../landing/installChannels";
import { expectViewportWide } from "./viewport";
import { expectErrBorder, hexToRgb } from "./tokens";

test("dark: the canvas swaps at the token level, and a dark: class really applies", async ({
  page,
}) => {
  await page.goto("/");

  // The mechanism, not just the preference: the bootstrap must have put the
  // class on the root, because everything below hangs off it.
  await expect(page.locator("html")).toHaveClass(/\bdark\b/);
  // The transport aside was this probe's raised subject until the phone
  // declutter stopped rendering it below the breakpoint (#596) — this lane
  // runs at 390, so the group table's wrapper carries the same
  // dark:bg-surface-raised contract now, and the aside's absence is itself
  // asserted.
  await expect(page.locator('aside[aria-label="Transport"]')).toHaveCount(0);
  await expect(page.locator("section#loca table")).toBeVisible();

  // Two probes, two failure shapes — and they need different instruments.
  //
  // The CANVAS is one token holding two values, so flipping the class on the
  // live page proves the swap: same element, same token, two class states.
  //
  // The ASIDE cannot be probed that way. Its `bg-surface` base ALSO swaps
  // with the theme, so a flip shows a difference even when its own
  // `dark:bg-surface-raised` override is gone — the first cut of this test
  // stayed green under exactly that sabotage. The honest probe is token
  // EQUALITY: in dark, the aside's paint must BE the raised token's value
  // and not the plain surface's (#545's criterion, asserted by convention
  // until this lane).
  const probe = await page.evaluate(() => {
    const resolve = (token: string) => {
      const el = document.createElement("div");
      el.style.backgroundColor = `var(${token})`;
      document.body.appendChild(el);
      const v = getComputedStyle(el).backgroundColor;
      el.remove();
      return v;
    };
    const canvasEl = document.querySelector(".bg-canvas")!;
    const darkCanvas = getComputedStyle(canvasEl).backgroundColor;
    document.documentElement.classList.remove("dark");
    const lightCanvas = getComputedStyle(canvasEl).backgroundColor;
    document.documentElement.classList.add("dark");
    return {
      darkCanvas,
      lightCanvas,
      raisedCard: getComputedStyle(
        document
          .querySelector("section#loca table")!
          .closest('[class*="dark:bg-surface-raised"]')!,
      ).backgroundColor,
      // The SOURCE tokens, not the `--color-*` theme names: `@theme inline`
      // erases those at build time (utilities compile straight to the source
      // var), so only `--surface`/`--surface-raised` exist at runtime.
      surface: resolve("--surface"),
      raised: resolve("--surface-raised"),
    };
  });
  expect(probe.darkCanvas, "the canvas token must swap").not.toBe(
    probe.lightCanvas,
  );
  // Guard the probe against vacuity first: if the two tokens ever computed
  // equal, the raised-card assertion below would pass while proving nothing.
  expect(probe.raised, "surface and raised must differ in dark").not.toBe(
    probe.surface,
  );
  expect(probe.raisedCard, "the card's dark: fill must apply").toBe(
    probe.raised,
  );
});

test("dark: the page still fits the viewport", async ({ page }) => {
  await page.goto("/");
  // The fullest layout, not the shell: findings populate asynchronously and
  // are the widest content the phone viewport carries.
  await expect(page.locator("section#file li").first()).toBeVisible({
    timeout: 15_000,
  });
  await expectViewportWide(page);
});

test("dark: the delete-group control keeps its danger border", async ({
  page,
}) => {
  // The light half lives in landing.spec.ts (#593); same shared contract
  // (tokens.ts), here under the dark token set.
  await page.goto("/");
  await expectErrBorder(page, "Delete the PROJ group");
});

test("dark: the KEY region tint stays structural", async ({ page }) => {
  // The light half lives in landing.spec.ts (#590); the same two-bands-one-
  // tint claim must hold under dark's own stone mix.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const headerBg = (section: string) =>
    page
      .locator(`section#${section} th`)
      .nth(1)
      .evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(await headerBg("samp")).toBe(await headerBg("llpl"));

  // And the three treatments stay pairwise distinct under dark's tokens:
  // the orphan row's wash is not the KEY tint, and its edge marker renders.
  const llpl = page.locator("section#llpl");
  const td = (cell: string) =>
    llpl.locator(`[data-cell="${cell}"]`).locator("xpath=ancestor::td[1]");
  const washBg = await td("2-1").evaluate(
    (el) => getComputedStyle(el).backgroundColor,
  );
  const keyBg = await td("1-1").evaluate(
    (el) => getComputedStyle(el).backgroundColor,
  );
  expect(washBg, "row wash is not the KEY tint in dark").not.toBe(keyBg);
  expect(
    await td("2-0").evaluate((el) => getComputedStyle(el).boxShadow),
    "the row edge marker renders in dark",
  ).not.toBe("none");
});

test("dark: the cell popover renders for cell and row findings", async ({
  page,
}) => {
  // The light journeys live in landing.spec.ts (#591); dark asserts the
  // same two surfaces exist under its token set — the callout inside is
  // severity.ts's tint, which the tint tests already hold per theme.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  await page
    .getByRole("button", { name: "Edit LOCA_GL on row 1 of LOCA" })
    .hover();
  const pop = page.getByRole("tooltip");
  await expect(pop).toBeVisible();
  await expect(pop).toContainText("Rule 8");
  await page.locator('section#llpl [data-cell="2-1"]').hover();
  await expect(pop).toContainText("Rule 10c");
});

test("dark: the masthead icons take real ink", async ({ page }) => {
  // #597's "crisp in both themes" is a colour claim at this altitude: both
  // glyphs draw with currentColor, so what the dark theme must prove is
  // that the ink they inherit resolves to a real colour, not transparent —
  // and that the two carry the SAME ink, one nav in two forms.
  await page.goto("/");
  const source = page.getByRole("link", { name: "Source on GitHub" });
  await expect(source).toBeVisible();
  const ink = (name: string) =>
    page
      .getByRole("link", { name })
      .evaluate((el) => getComputedStyle(el).color);
  const sourceInk = await ink("Source on GitHub");
  expect(sourceInk).not.toBe("rgba(0, 0, 0, 0)");
  expect(await ink("Jump to install")).toBe(sourceInk);
});

test("dark: every install card wears its dark hue", async ({ page }) => {
  // The light half lives in landing.spec.ts (#595). Dark's hue is its own
  // tuned value, not an inversion — asserted against the generated data, so
  // the card and installChannels.ts cannot disagree.
  await page.goto("/");
  const cards = page.locator(".install-card");
  await expect(cards).toHaveCount(INSTALL_CHANNELS.length);
  for (const [i, channel] of INSTALL_CHANNELS.entries()) {
    expect(
      await cards.nth(i).evaluate((el) => getComputedStyle(el).borderTopColor),
      `${channel.id} dark border`,
    ).toBe(hexToRgb(channel.hue.dark));
  }
});

test("dark: the status marks and the corner flag take real ink", async ({
  page,
}) => {
  // #616's "both themes" half. The key glyph strokes with currentColor from
  // the band variable and the corner flag borrows the severity ink the same
  // way, so what dark must prove is that both resolve to real colours here
  // — the glyph to its band, the flag to the error ink — not transparent.
  await page.goto("/");
  const keyGlyph = page
    .locator("section#loca th")
    .filter({ hasText: "LOCA_ID" })
    .locator("svg");
  await expect(keyGlyph).toBeVisible();
  const stroke = await keyGlyph.evaluate((el) => getComputedStyle(el).stroke);
  expect(stroke).not.toBe("none");
  expect(stroke).not.toBe("rgba(0, 0, 0, 0)");

  // The seeded Rule 8 cell wears the flag once findings arrive; its border
  // ink is the cell's severity colour, which must differ from the header
  // glyph's band ink — severity is never band identity (#590).
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  const flag = cell.locator("span.absolute");
  await expect(flag).toBeVisible({ timeout: 15_000 });
  const flagInk = await flag.evaluate(
    (el) => getComputedStyle(el).borderTopColor,
  );
  expect(flagInk).not.toBe("rgba(0, 0, 0, 0)");
  expect(flagInk).not.toBe(
    await keyGlyph.evaluate((el) => getComputedStyle(el).color),
  );

  // The other half of the grammar: the KEY+REQUIRED box draws with the
  // same currentColor as its glyph, and the REQUIRED-only asterisk takes
  // its band ink too — both must resolve here, not just the bare glyph.
  const boxed = page
    .locator("section#proj th")
    .filter({ hasText: "PROJ_ID" })
    .locator("span[class*='border-current']");
  await expect(boxed).toBeVisible();
  expect(
    await boxed.evaluate((el) => getComputedStyle(el).borderTopColor),
  ).not.toBe("rgba(0, 0, 0, 0)");
  const star = page
    .locator("section#file th")
    .filter({ hasText: "TRAN_DATE" })
    .getByText("*", { exact: true });
  await expect(star).toBeVisible();
  expect(await star.evaluate((el) => getComputedStyle(el).color)).not.toBe(
    "rgba(0, 0, 0, 0)",
  );
});
