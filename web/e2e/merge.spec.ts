import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fixture, ready, tab } from "./helpers";

// Tools → Merge, end-to-end in a real browser: reconcile two AGS4 deliveries of
// one project through the wasm engine (the same laterite-ags4-merge leaf the CLI
// + laterite + laterite-node use), and assert the real merged output + the
// per-row revision audit — not just "the tab renders".

test("Tools → Merge reconciles two deliveries: revision audit + downloadable union", async ({
  page,
}) => {
  await ready(page);

  // Load the base delivery in Validate — it seeds the Merge tool's "Base (a)".
  await page.locator('input[type="file"]').setInputFiles(fixture("merge_base.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Merge$/ }).click();

  // Upload the incoming delivery into the second picker → the merge runs.
  await page
    .locator('label:has-text("Incoming") input[type="file"]')
    .setInputFiles(fixture("merge_incoming.ags"));

  // The incoming file revised BH1's LOCA_GL (10.00 → 11.50) — surfaced as a
  // typed row revision (a formatting-only change would NOT appear here).
  const audit = page.getByText(/Rows the incoming file revised/);
  await expect(audit).toBeVisible({ timeout: 30_000 });
  await expect(page.getByText(/BH1/)).toBeVisible();
  await expect(page.getByText(/changed LOCA_GL/)).toBeVisible();

  // Download the merged file and assert the union + the winning value.
  const [dl] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Download merged/ }).click(),
  ]);
  expect(dl.suggestedFilename()).toMatch(/\.merged\.ags$/);
  const out = readFileSync(await dl.path(), "utf8");

  // Union of both deliveries: BH1 + BH2 (base only) + BH3 (incoming only).
  for (const bh of ['"BH1"', '"BH2"', '"BH3"']) expect(out).toContain(bh);
  // The incoming file wins the BH1 key conflict: GL is 11.50, not the base's 10.00.
  expect(out).toContain('"11.50"');
  expect(out).not.toContain('"10.00"');
});
