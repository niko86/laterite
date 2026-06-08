import { expect, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";
import path from "node:path";

// Shared e2e helpers. Kept separate from app.spec.ts so multiple spec files
// reuse them without duplicating the wasm-ready gate / fixture path logic.

export const APP = "/ags5_concept/";

export const fixture = (name: string) =>
  path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);

/** Navigate + wait until the wasm validator is live (the sample buttons only
 *  render once the worker reports ready — see App's wasmReady gate). */
export async function ready(page: Page) {
  await page.goto(APP);
  await expect(
    page.getByRole("button", { name: /Clean \(minimal\)/ }),
  ).toBeVisible();
}

export const tab = (page: Page, name: string) =>
  page.getByRole("tab", { name: new RegExp(`^${name}$`) });

/** Upload a fixture through the Validate tab's file input. */
export async function load(page: Page, name: string) {
  await page.locator('input[type="file"]').setInputFiles(fixture(name));
}

/** Open the Explore tab and ensure the DuckDB dashboard is up — dismissing the
 *  cold-engine gate if it appears. On a CAPABLE machine the engine auto-loads
 *  (no gate). On a device fingerprinted low-end — which CI runners often are,
 *  reporting ≤2 cores / <4 GB so `engineGateNeeded()` fires — the gate
 *  intercepts the 36 MB download until the user confirms; without dismissing it
 *  every DuckDB test hangs at this gate instead of reaching "data rows". Race
 *  the two paths so a capable runner pays no extra wait. */
export async function enterExplore(page: Page) {
  await tab(page, "Explore").click();
  const gate = page.getByRole("button", { name: /^Continue$/ });
  const rows = page.getByText(/data rows/);
  await expect(rows.or(gate).first()).toBeVisible({ timeout: 90_000 });
  if (await gate.isVisible().catch(() => false)) await gate.click();
  await expect(rows).toBeVisible({ timeout: 90_000 });
}
