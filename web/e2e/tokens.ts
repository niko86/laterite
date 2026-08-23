import { expect, type Page } from "@playwright/test";

/** The generated install hues are hex; computed borders come back as rgb().
 *  Shared by the light and dark specs (#595) so the two cannot drift — the
 *  same reasoning as expectErrBorder below. */
export const hexToRgb = (hex: string) => {
  const n = parseInt(hex.slice(1), 16);
  return `rgb(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255})`;
};

/** The danger-border contract, shared by the light and dark landing specs
 *  (#593) so the two cannot drift — the same reasoning that moved
 *  expectViewportWide into viewport.ts (#547). Resolves the theme's own
 *  --err through a probe element (the token may hold any colour syntax) and
 *  requires the named control's border to compute to exactly that value —
 *  not transparent, and not the other theme's. */
export const expectErrBorder = async (page: Page, name: string) => {
  const btn = page.getByRole("button", { name });
  await expect(btn).toBeVisible({ timeout: 15_000 });
  const [border, err] = await btn.evaluate((el) => {
    const probe = document.createElement("div");
    probe.style.color = getComputedStyle(document.documentElement)
      .getPropertyValue("--err")
      .trim();
    document.body.appendChild(probe);
    const resolved = getComputedStyle(probe).color;
    probe.remove();
    return [getComputedStyle(el).borderTopColor, resolved];
  });
  expect(err, "the err token must resolve").not.toBe("rgba(0, 0, 0, 0)");
  expect(border).toBe(err);
};
