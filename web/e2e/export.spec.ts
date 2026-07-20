import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { ready, tab } from "./helpers";

// The Export tab is the browser/offline half of the AGS4-output feature:
// per-group data (JSON) → the wasm `to_ags4` producer → a downloadable .ags.

test("Export tab builds & downloads a valid .ags from the example", async ({
  page,
}) => {
  await ready(page);
  await tab(page, "Export").click();

  // The pane is prefilled with a working example; build + download.
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Build & download \.ags/ }).click(),
  ]);
  expect(download.suggestedFilename()).toBe("delivery.ags");

  const content = readFileSync(await download.path(), "utf8");
  expect(content).toContain('"GROUP","PROJ"');
  expect(content).toContain('"DATA","BH01"');
  expect(content).toContain('"12.30"'); // a typed float (12.3) canonicalised to 2DP
  expect(content).toContain("\r\n"); // CRLF (Rule 2a)

  await expect(page.getByText(/safe fix\(es\) applied/)).toBeVisible();
});

test("Export Strict mode surfaces an error for an incomplete file", async ({
  page,
}) => {
  await ready(page);
  await tab(page, "Export").click();

  await page.getByLabel("Mode").selectOption("strict");
  await page
    .locator("textarea")
    .fill('[{"code":"LOCA","headings":["LOCA_ID"],"rows":[["BH01"]]}]');
  await page.getByRole("button", { name: /Preview only/ }).click();

  await expect(page.getByText(/strict mode rejected/)).toBeVisible();
});
