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
import { expectViewportWide } from "./viewport";
import { expectErrBorder } from "./tokens";

test("dark: the canvas swaps at the token level, and a dark: class really applies", async ({
  page,
}) => {
  await page.goto("/");

  // The mechanism, not just the preference: the bootstrap must have put the
  // class on the root, because everything below hangs off it.
  await expect(page.locator("html")).toHaveClass(/\bdark\b/);
  await expect(page.locator('aside[aria-label="Transport"]')).toBeVisible();

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
      aside: getComputedStyle(
        document.querySelector('aside[aria-label="Transport"]')!,
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
  // equal, the aside assertion below would pass while proving nothing.
  expect(probe.raised, "surface and raised must differ in dark").not.toBe(
    probe.surface,
  );
  expect(probe.aside, "the aside's dark: fill must apply").toBe(probe.raised);
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
