import { test, expect } from "@playwright/test";
import { ready, load, tab } from "./helpers";

// A BOM-prefixed file produces a Rule 1 finding (the fixable, severity-bearing
// one) AND a sibling "FYI (Related to Rule 1)" advisory. The single safe fix
// (strip the BOM) therefore also clears an FYI — the thing that surprised the
// owner. The Fix tab must make that legible: badge each fix with the severity
// it addresses, and explain that fixing is file-level (not gated by the
// Validate severity filter).
test("Fix tab badges severity + explains FYI side-effects for a BOM file", async ({
  page,
}) => {
  await ready(page);
  await load(page, "bom_only.ags");
  await tab(page, "Fix").click();

  // The one safe fix is the BOM strip.
  await expect(
    page.getByText("Strip the UTF-8 byte-order mark (Rule 1)"),
  ).toBeVisible();

  // It carries a severity badge (the BOM is a Rule 1 error).
  await expect(
    page.getByText("error", { exact: true }).first(),
  ).toBeVisible();

  // The explainer is shown (a fix can clear a related FYI; the filter doesn't
  // gate fixing).
  await expect(
    page.getByText(/doesn't limit what gets fixed/i),
  ).toBeVisible();
});
