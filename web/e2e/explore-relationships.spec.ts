import { test, expect, type Page } from "@playwright/test";
import { ready, load, enterExplore } from "./helpers";

// Relationship-aware Explore: cross-group joins (incl. the GEOL depth-range
// stratum enrichment) + LIKE wildcards. Driven end-to-end against real
// DuckDB-wasm on strata.ags (1 LOCA; GEOL strata MADE GROUND 0–2.5 / CLAY
// 2.5–6 / SAND 6–10; SAMP at 1/4/9.5/12 m; TREG specimens at 4.2 & 6.0 m).

async function exploreSqlStrata(page: Page) {
  await ready(page);
  await load(page, "strata.ags");
  await enterExplore(page);
  await page.getByRole("button", { name: "SQL" }).click();
}
const runSql = async (page: Page, sql: string) => {
  await page.locator("textarea").fill(sql);
  await page.getByRole("button", { name: /^Run/ }).click();
};

test("Explore: the GEOL-stratum template enriches specimens with geology + specimen description", async ({
  page,
}) => {
  await exploreSqlStrata(page);
  // The dictionary-derived flagship example chip (base = TREG, range-joined to GEOL).
  await page.getByRole("button", { name: /× GEOL stratum/ }).click();
  await page.getByRole("button", { name: /^Run/ }).click();
  await expect(page.getByText(/SQL error/)).toHaveCount(0);
  // Each specimen lands in its stratum, and its specimen description surfaces.
  await expect(page.getByText("CLAY").first()).toBeVisible({ timeout: 30_000 });
  await expect(page.getByText("SAND").first()).toBeVisible();
  await expect(page.getByText("Stiff clay specimen").first()).toBeVisible();
});

test("Explore: a half-open depth band puts a boundary depth in the lower stratum", async ({
  page,
}) => {
  await exploreSqlStrata(page);
  // SP2 is at SPEC_DPTH 6.00 — exactly CLAY's base / SAND's top. Half-open
  // [top, base) ⇒ it belongs to SAND (SA), not CLAY (CL).
  await runSql(
    page,
    `SELECT g."GEOL_LEG" AS leg FROM "TREG" t LEFT JOIN "GEOL" g` +
      ` ON t."LOCA_ID" = g."LOCA_ID" AND t."SPEC_DPTH" >= g."GEOL_TOP" AND t."SPEC_DPTH" < g."GEOL_BASE"` +
      ` WHERE t."SPEC_REF" = 'SP2'`,
  );
  await expect(page.getByText("SA", { exact: true })).toBeVisible({
    timeout: 30_000,
  });
  await expect(page.getByText("CL", { exact: true })).toHaveCount(0);
});

test("Explore: a sample below all strata still shows (LEFT join keeps it, stratum NULL)", async ({
  page,
}) => {
  await exploreSqlStrata(page);
  // SAMP at 12.00 m is below GEOL's deepest base (10.00) — LEFT JOIN keeps it.
  await runSql(
    page,
    `SELECT s."SAMP_ID" AS id, g."GEOL_LEG" AS leg FROM "SAMP" s LEFT JOIN "GEOL" g` +
      ` ON s."LOCA_ID" = g."LOCA_ID" AND s."SAMP_TOP" >= g."GEOL_TOP" AND s."SAMP_TOP" < g."GEOL_BASE"` +
      ` WHERE s."SAMP_TOP" = 12.0`,
  );
  await expect(page.getByText("BH01-S4", { exact: true })).toBeVisible({
    timeout: 30_000,
  });
});

test("Explore: the content-addressed keys join a child to its parent (SAMP._parent_id = LOCA._id)", async ({
  page,
}) => {
  await exploreSqlStrata(page);
  // #303: duckdb-wasm carries the synthetic _id/_parent_id columns, so a
  // parent/child join needs no AGS-key knowledge — every SAMP on BH01 resolves
  // to that LOCA row (same rows as `USING (LOCA_ID)`, one column pair for every
  // edge). This used to raise a Binder Error: the keys were extension-only.
  await runSql(
    page,
    `SELECT l."LOCA_ID" AS loca FROM "SAMP" s JOIN "LOCA" l ON s."_parent_id" = l."_id"`,
  );
  await expect(page.getByText(/SQL error/)).toHaveCount(0);
  await expect(page.getByText("BH01", { exact: true }).first()).toBeVisible({
    timeout: 30_000,
  });
});

test("Explore: SqlBuilder auto depth-band joins SAMP→GEOL, and LIKE wildcards inject %", async ({
  page,
}) => {
  await exploreSqlStrata(page);
  await page.getByText("Build a query with controls").click();
  await page.getByLabel("Table").selectOption("SAMP");
  await page.getByLabel("related group").selectOption("GEOL");

  // Joining SAMP to the depth-range group becomes a depth-band join.
  const pre = page.locator("details pre").first();
  await expect(pre).toContainText('LEFT JOIN "GEOL" j');
  await expect(pre).toContainText('c."SAMP_TOP" >= j."GEOL_TOP"');
  await expect(pre).toContainText('c."SAMP_TOP" < j."GEOL_BASE"');

  // LIKE "starts with" injects the trailing % (+ ESCAPE), not typed by the user.
  await page.getByRole("button", { name: /\+ add/ }).click();
  await page.getByLabel("filter column").selectOption("j.GEOL_DESC");
  await page.getByLabel("filter operator").selectOption("LIKE");
  await page.getByLabel("filter wildcard").selectOption("starts");
  await page.locator('input[placeholder="value"]').fill("CL");
  await expect(pre).toContainText(`j."GEOL_DESC" LIKE 'CL%' ESCAPE`);
});

test("Explore charts: colour a base plot by GEOL stratum via the depth-band join", async ({
  page,
}) => {
  await ready(page);
  await load(page, "strata.ags");
  await enterExplore(page);
  await page.getByRole("button", { name: "Charts" }).click();

  await page.getByLabel("Table").selectOption("SAMP");
  await page.getByLabel("related group").selectOption("GEOL");
  await page.getByLabel("x axis").selectOption("c.SAMP_TOP");
  await page.getByLabel("y axis").selectOption("c.SAMP_TOP");
  await page.getByLabel("colour by").selectOption("j.GEOL_LEG");

  // The chart renders, and its SQL is the depth-band join coloured by stratum.
  await expect(page.locator("canvas").first()).toBeVisible({ timeout: 90_000 });
  await page.locator("details summary").first().click();
  const pre = page.locator("details pre").first();
  await expect(pre).toContainText('LEFT JOIN "GEOL" j');
  await expect(pre).toContainText('c."SAMP_TOP" >= j."GEOL_TOP"');
  await expect(pre).toContainText('j."GEOL_LEG" AS c');
});

test("Explore charts: a colour column with more values than slots still plots", async ({
  page,
}) => {
  // The fold's own SQL never reaches the disclosure pane — the chart runs a
  // SECOND query, the cardinality probe that ranks the colour values, and the
  // unit suite can only assert the string it composes. This is what asserts
  // DuckDB accepts it: with a colour-by set, the plot WAITS on the probe, so a
  // canvas at all means the probe ran, returned, and the tail folded.
  //
  // SAMP_ID is unique per row, so this fixture's samples are its distinct
  // colour values — more of them than the scatter cap has slots, which is the
  // case that used to hand ECharts a palette to cycle. Shrink the fixture below
  // the cap and this stops exercising the fold: it still checks the probe, it
  // just checks less. The cap itself is pinned in the node lane, not here.
  await ready(page);
  await load(page, "strata.ags");
  await enterExplore(page);
  await page.getByRole("button", { name: "Charts" }).click();

  await page.getByLabel("Table").selectOption("SAMP");
  await page.getByLabel("x axis").selectOption("c.SAMP_TOP");
  await page.getByLabel("y axis").selectOption("c.SAMP_TOP");
  await page.getByLabel("colour by").selectOption("c.SAMP_ID");

  await expect(page.locator("canvas").first()).toBeVisible({ timeout: 90_000 });
  // Either query — the plot's or the probe's — and the banner names which.
  await expect(page.getByText(/query error/)).toHaveCount(0);
});

test("Explore charts: an aggregated bar MERGES the folded tail, avg included", async ({
  page,
}) => {
  // The numeric half of #457, which only real DuckDB can answer. The unit lane
  // pins the string the composer emits; what it cannot show is that grouping on
  // that expression makes the aggregate run over the merged group's RAW rows.
  //
  // chart_fold.ags is built for exactly that: SAMP_REF has 7 distinct values,
  // two more than a bar's slot count, and the two that fold carry an uneven
  // number of rows under SAMP_TYPE "U" — S6 one row at 20.00, S7 two at 40.00
  // and 120.00. Merged, the mean is 60. The old fold drew those as TWO bars,
  // at 20 and at 80, at one category; averaging them after the fact would give
  // 50, which is the answer `avg` has no honest way to reach from the plotted
  // rows. Even row counts would leave 60 and 50 equal and the assertion
  // toothless.
  await ready(page);
  await load(page, "chart_fold.ags");
  await enterExplore(page);
  await page.getByRole("button", { name: "Charts" }).click();

  await page.getByLabel("Table").selectOption("SAMP");
  await page.getByLabel("Chart").selectOption("bar");
  await page.getByLabel("Aggregate").selectOption("avg");
  await page.getByLabel("x axis").selectOption("c.SAMP_TYPE");
  await page.getByLabel("y axis").selectOption("c.SAMP_TOP");
  await page.getByLabel("colour by").selectOption("c.SAMP_REF");

  await expect(page.locator("canvas").first()).toBeVisible({ timeout: 90_000 });
  await expect(page.getByText(/query error/)).toHaveCount(0);

  // The pane shows the query that RAN — the folded one, composed only once the
  // probe had ranked the survivors. Before this it showed the plain query and
  // never mentioned the probe at all.
  await page.locator("details summary").first().click();
  const pre = page.locator("details pre").first();
  await expect(pre).toContainText(`IN ('S1', 'S2', 'S3', 'S4', 'S5')`);
  await expect(pre).toContainText(`ELSE 'Other' END`);
  const sql = (await pre.textContent()) ?? "";

  // Run that same query through the SQL console: the canvas cannot be read, and
  // the value the tail bar is drawn at is the whole claim.
  await page.getByRole("button", { name: "SQL" }).click();
  await runSql(page, sql);
  await expect(page.getByText(/SQL error/)).toHaveCount(0);

  const tail = page.locator("tbody tr").filter({ hasText: "Other" });
  // ONE row per category, not one per folded value — the merge itself.
  await expect(tail).toHaveCount(2, { timeout: 30_000 });
  // Asserted on the CELL, not on the row's run-together text: 60 is the mean of
  // 20, 40 and 120. 20 and 80 are the two bars the pooled fold drew here, 50
  // their after-the-fact mean — none of the three survive the merge.
  const cell = (v: string | RegExp) =>
    page.getByRole("cell", { name: v, exact: true });
  await expect(tail.filter({ has: cell("60") })).toHaveCount(1);
  await expect(tail.filter({ has: cell(/^(50|80|20)$/) })).toHaveCount(0);
});
