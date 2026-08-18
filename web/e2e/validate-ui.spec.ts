import { test, expect, type Page } from "@playwright/test";
import { ready, load, tab } from "./helpers";

// UI-behaviour coverage for the wasm Validate + Fix pages: the *interactions*
// (search, severity filtering, encoding toggle, dictionary selector, list
// virtualization) rather than "does the engine flag rule X" — that path is
// covered by app.spec.ts. Each test drives the real wasm app in the browser.

const showing = (page: Page) => page.getByText(/showing \d+ of \d+ findings/);
const shownNow = async (page: Page) => {
  const m = ((await showing(page).textContent()) ?? "").match(
    /showing (\d+) of (\d+)/,
  );
  return { shown: Number(m![1]), total: Number(m![2]) };
};

test("search filters the findings, and clearing the box restores them all", async ({
  page,
}) => {
  await ready(page);
  await load(page, "many_findings.ags"); // ~250 Rule 9 findings
  await expect(showing(page)).toBeVisible();
  const { shown: s0, total } = await shownNow(page);
  expect(total).toBeGreaterThan(150);

  const box = page.getByPlaceholder(/Search line text/);
  // A guaranteed-no-match query empties the list (search actually filters) …
  await box.fill("NO_SUCH_TOKEN_ZZZ_QWERTY");
  await expect(showing(page)).toHaveText(new RegExp(`showing 0 of ${total}`));
  // … and DELETING the query repopulates the full list. This is the
  // regression: a cleared box used to leave the list empty ("shows nothing").
  await box.fill("");
  await expect(showing(page)).toHaveText(
    new RegExp(`showing ${s0} of ${total}`),
  );
});

test("severity chips show/hide findings of that severity", async ({ page }) => {
  await ready(page);
  await load(page, "mixed_error_fyi.ags"); // 1 error (Rule 9) + 1 FYI (Rule 1)
  await expect(showing(page)).toBeVisible();
  const start = await shownNow(page); // FYI is OFF by default → error(s) only

  const fyiChip = page.getByRole("button", { name: /^fyi/ });
  await fyiChip.click(); // turn FYI on → its findings appear
  await expect
    .poll(async () => (await shownNow(page)).shown)
    .toBeGreaterThan(start.shown);
  await fyiChip.click(); // turn FYI back off → restored
  await expect.poll(async () => (await shownNow(page)).shown).toBe(start.shown);
});

test("the encoding toggle re-decodes the file and re-validates", async ({
  page,
}) => {
  await ready(page);
  await load(page, "cp1252.ags"); // clean apart from one raw Windows-1252 é byte
  // UTF-8 (default): the 0xE9 byte decodes to U+FFFD → a Rule 1 ERROR, and the
  // app offers the "Switch encoding" hint.
  await expect(page.getByText("✗").first()).toBeVisible();
  await expect(page.getByText(/Switch encoding to Windows-1252/)).toBeVisible();

  // Switch to Windows-1252: the byte is now é (a Rule 1 FYI) → FYI-only amber
  // banner, no error, hint gone. Same bytes, different validation.
  await page.getByLabel("Encoding").selectOption("windows-1252");
  await expect(page.getByText(/informational \(FYI\) finding/)).toBeVisible();
  await expect(page.getByText("✗")).toHaveCount(0);
  await expect(page.getByText(/Switch encoding to Windows-1252/)).toHaveCount(
    0,
  );
});

test("the dictionary-edition selector re-validates without breaking", async ({
  page,
}) => {
  await ready(page);
  await load(page, "many_findings.ags");
  await expect(showing(page)).toBeVisible();
  const sel = page.getByLabel("Dictionary edition");
  await sel.selectOption("4.0.3");
  // Re-validation completes, the list is still rendered, and the selection
  // holds. (The per-edition dictionary CONTENT differences are covered by the
  // Tools → Dictionary test; here we just exercise the control + re-validate.)
  await expect(showing(page)).toBeVisible();
  await expect(sel).toHaveValue("4.0.3");
});

// The windowing falsification, shared by both virtualized lists (findings,
// safe fixes): far fewer [data-index] rows in the DOM than `total` proves the
// list is windowed rather than all-mounted, and scrolling the region to the
// bottom must mount clearly higher-indexed rows. Only virtual rows carry
// data-index, and panes unmount on tab switch, so the count can't cross-match
// the other tab's list.
async function expectWindowed(page: Page, total: number) {
  const indices = () =>
    page
      .locator("[data-index]")
      .evaluateAll((els) =>
        els.map((e) => Number(e.getAttribute("data-index"))),
      );
  const before = await indices();
  expect(before.length).toBeGreaterThan(0);
  expect(before.length).toBeLessThan(Math.min(60, total));
  const maxBefore = Math.max(...before);

  const scroller = page
    .locator("div.scroll-region")
    .filter({ has: page.locator("[data-index]") });
  await scroller.evaluate((el) => el.scrollTo(0, el.scrollHeight));
  await expect
    .poll(async () => Math.max(...(await indices())))
    .toBeGreaterThan(maxBefore + 50);
}

test("the findings list virtualizes — only a window renders, scrolling reveals more", async ({
  page,
}) => {
  await ready(page);
  await load(page, "many_findings.ags");
  await page.getByRole("button", { name: "Expand all" }).click();
  const { total } = await shownNow(page);
  expect(total).toBeGreaterThan(150);

  await expectWindowed(page, total);
});

test("the safe-fix list virtualizes — a window of cards, with the risky group reachable below", async ({
  page,
}) => {
  await ready(page);
  // One 1dp reformat per LOCA data row (safe), plus one ambiguous DT date
  // (risky) — the issue's synthetic shape.
  await load(page, "many_fixes.ags");
  await tab(page, "Fix").click();

  // Every safe fix is on offer (the count is real). Polled: the button
  // renders "(0)" until the engine's computeFixes batch lands.
  const fixAll = page.getByRole("button", { name: /Fix all safe \(\d+\)/ });
  const totalNow = async () =>
    Number(((await fixAll.textContent()) ?? "").match(/\((\d+)\)/)?.[1] ?? 0);
  await expect.poll(totalNow).toBeGreaterThan(300);

  // Only a window of cards is in the DOM (#435): the scroll cap bounded the
  // page while every card still mounted its full aligned diff block.
  await expectWindowed(page, await totalNow());

  // The risky group — small, opt-in, NOT virtualised — still renders in full
  // below the safe list.
  await expect(page.getByText(/Risky fixes \(1\)/)).toBeVisible();
  await expect(page.getByText(/Canonicalise datetime/)).toBeVisible();
});
