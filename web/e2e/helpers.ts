import { expect, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";
import path from "node:path";

// Shared e2e helpers. Kept separate from app.spec.ts so multiple spec files
// reuse them without duplicating the wasm-ready gate / fixture path logic.

// The deploy base, read from the SAME env var vite.config.ts reads. Five spec
// files used to hardcode "/laterite/" each; when the site moved to its own
// domain and the base became "/", that was five edits with nothing to catch a
// missed one — the PWA test failed on exactly that. Derive it once instead, so
// a base change is one edit in vite.config.ts and the suite follows.
export const APP = process.env.VITE_BASE ?? "/";

export const fixture = (name: string) =>
  path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);

/** Navigate + wait until the app has painted. The sample buttons render on the
 *  ~30 KB tokenizer alone (#353) — the engine may still be arriving — but every
 *  caller here goes on to validate something, which waits for it anyway. */
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
