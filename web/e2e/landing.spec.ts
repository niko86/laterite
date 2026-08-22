import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

// The landing page (laterite.dev) at a STRICT 390px phone viewport — no
// `isMobile`, deliberately: mobile emulation absorbs a too-wide layout into
// zoom, so the page "fits" while every offender becomes invisible and every
// tap target shrinks. A strict viewport makes the overflow measurable, which
// is how #523 was found: the group-section grids let the nowrap tables size
// the page, and a 390px phone got a 783px layout viewport zoomed out to fit.
//
// This spec runs under the `landing` (390, fine pointer), `landing-touch`
// (390, coarse pointer) and `landing-wide` (1280) projects
// (playwright.config.ts), against the landing's OWN preview server — the
// landing is a separate build (see web/landing/vite.config.ts), so the app's
// server cannot serve it. Width- and modality-specific tests skip themselves
// on the other projects, the way layout.spec.ts branches on viewport. The
// editors split by MODALITY (#525): `hasTouch` is what makes Chromium report
// `pointer: coarse`, so it is the skip condition, not the width.

const GROUPS = ["proj", "loca", "samp", "llpl"] as const;

const width = (page: Page) => page.viewportSize()?.width ?? 0;

/** The seeded delivery's final depth, read from the SAME fixture the page
 *  bakes in — so editing LOCA_FDEP moves both the rail's bottom label and
 *  this expectation, with no code edit on either side (#524). */
function seededFinalDepthLabel(): string {
  const ags = readFileSync(
    path.join(
      path.dirname(fileURLToPath(import.meta.url)),
      "../landing/demo/seeded-delivery.ags",
    ),
    "utf8",
  );
  // AGS4 files are CRLF; a bare \n split leaves \r glued to each last cell.
  const lines = ags.split(/\r?\n/);
  const group = lines.findIndex((l) => l.startsWith('"GROUP","LOCA"'));
  const cells = (l: string) => l.replace(/^"|"$/g, "").split('","');
  const headings = cells(lines[group + 1] ?? "");
  const data = lines.slice(group).find((l) => l.startsWith('"DATA"'));
  return Number(cells(data ?? "")[headings.indexOf("LOCA_FDEP")] ?? 0).toFixed(
    2,
  );
}

/** The scroller that owns each group table's horizontal pan — the table's own
 *  wrapper, located through the table so the spec doesn't couple to a utility
 *  class name. */
const scroller = (page: Page, section: string) =>
  page.locator(`section#${section} table`).locator("xpath=..");

async function expectViewportWide(page: Page) {
  const doc = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
  }));
  // Exactly viewport-wide, no tolerance: any horizontal overflow is the bug.
  // Against clientWidth rather than a literal 390 so a runner that draws
  // space-taking scrollbars measures the same contract — with #523 unfixed,
  // scrollWidth read ~783 while clientWidth stayed at the viewport. The
  // ceiling closes the remaining direction, both numbers growing together
  // (a wrong server or a wrong project viewport).
  expect(doc.scrollWidth, "the page must not outgrow the viewport").toBe(
    doc.clientWidth,
  );
  expect(doc.scrollWidth).toBeLessThanOrEqual(width(page));
}

test("phone: the page stays viewport-wide and each group table pans in its own scroller", async ({
  page,
}) => {
  test.skip(
    width(page) >= 1024,
    "above the 64rem grid breakpoint the tables have room — this is the phone contract",
  );
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

  await expectViewportWide(page);

  // Each table must overflow ITS OWN scroller and pan there — the design
  // contract on the table component: "nine columns on a 390px phone must move
  // the table, not the page". Before the fix all four scrollers reported
  // clientWidth == scrollWidth: the page had grown instead.
  for (const section of GROUPS) {
    const el = scroller(page, section);
    const geo = await el.evaluate((node) => ({
      scrollWidth: node.scrollWidth,
      clientWidth: node.clientWidth,
    }));
    expect(
      geo.scrollWidth,
      `the ${section} table must overflow its scroller at phone width`,
    ).toBeGreaterThan(geo.clientWidth);

    const panned = await el.evaluate((node) => {
      node.scrollLeft = 100;
      return node.scrollLeft;
    });
    expect(panned, `the ${section} scroller must pan`).toBeGreaterThan(0);
  }
});

test("touch: the carousel opens, pages, closes — and never widens the page", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the carousel is the coarse pointer's editor (#525)");
  await page.goto("/");

  // LLPL is the widest table (nine columns, every cell nowrap) and the
  // carousel is the pattern chosen to carry it at 390px — so it is the
  // pairing most able to push the page out. The first column is sticky, so
  // its cell is tappable without panning.
  await page
    .getByRole("button", { name: "Edit LOCA_ID on row 1 of LLPL" })
    .click();
  const carousel = page.getByRole("group", { name: "Editing row 1 of LLPL" });
  await expect(carousel).toBeVisible();
  await expectViewportWide(page);

  // Paging lands on the next field's card.
  await carousel
    .getByRole("button", { name: "Next field in this row" })
    .click();
  await expect(carousel.getByText("SAMP_TOP", { exact: true })).toBeVisible();

  await carousel.getByRole("button", { name: "Close the row editor" }).click();
  await expect(carousel).toBeHidden();
  await expectViewportWide(page);
});

test("touch: a row deletes from the carousel and the findings follow", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the carousel is the coarse pointer's editor (#525)");
  await page.goto("/");

  // The Rule 13 teach-loop the delete exists to unwind (#525): a second PROJ
  // row is flagged, and before row delete the only way back was a full reset.
  const proj = page.locator("section#proj");
  const rows = proj.locator("tbody tr");
  await proj.getByRole("button", { name: "+ row" }).click();
  await expect(rows).toHaveCount(2);
  await expect(proj.locator("li").first()).toBeVisible();

  await proj
    .getByRole("button", { name: "Edit PROJ_ID on row 2 of PROJ" })
    .click();
  await page
    .getByRole("group", { name: "Editing row 2 of PROJ" })
    .getByRole("button", { name: "Delete this row" })
    .click();
  await expect(rows).toHaveCount(1);
  // Deleting the row it edited closes the carousel — the row is gone.
  await expect(
    page.getByRole("group", { name: "Editing row 2 of PROJ" }),
  ).toBeHidden();
  await expect(proj.locator("li")).toHaveCount(0);
});

test("fine: click selects, Enter edits in place, Esc cancels, Tab commits and moves, arrows navigate, typing replaces", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "in-place editing is the fine pointer's editor (#525)");
  await page.goto("/");

  // The seeded Rule 8 defect: LOCA_GL is 11.8 where 2DP demands 11.80. The
  // first click selects AND arms the engine, so the ✗ arrives on this cell.
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();
  await expect(cell).toHaveClass(/bg-accent-quiet/);
  await expect(cell).toContainText("✗");

  // Enter opens the value in place; typing replaces the selection; Enter
  // commits — and the live revalidation clears the finding.
  await page.keyboard.press("Enter");
  const editor = page.getByLabel("LOCA_GL on row 1 of LOCA", {
    exact: true,
  });
  await expect(editor).toBeVisible();
  await page.keyboard.type("11.80");
  await page.keyboard.press("Enter");
  await expect(editor).toBeHidden();
  await expect(cell).toContainText("11.80");
  await expect(cell).not.toContainText("✗");

  // Esc restores: the draft dies with the editor.
  await page.keyboard.press("Enter");
  await page.keyboard.type("999");
  await page.keyboard.press("Escape");
  await expect(cell).toContainText("11.80");

  // Arrows move the selection, not the scroller.
  await page.keyboard.press("ArrowRight");
  const neighbour = page.getByRole("button", {
    name: "Edit LOCA_REM on row 1 of LOCA",
  });
  await expect(neighbour).toHaveClass(/bg-accent-quiet/);

  // Type-to-replace: a printable key opens the editor already holding it.
  await page.keyboard.type("X");
  const remEditor = page.getByLabel("LOCA_REM on row 1 of LOCA", {
    exact: true,
  });
  await expect(remEditor).toHaveValue("X");

  // Tab commits and moves on.
  await page.keyboard.press("Tab");
  await expect(neighbour).toHaveText("X");
  await expect(
    page.getByRole("button", { name: "Edit LOCA_FDEP on row 1 of LOCA" }),
  ).toHaveClass(/bg-accent-quiet/);

  await expectViewportWide(page);
});

test("fine: rows delete from the table, and undo/redo walk the model timeline", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "in-place editing is the fine pointer's editor (#525)");
  await page.goto("/");

  // Row delete, fine-pointer affordance: the per-row ✕, revalidating live.
  const proj = page.locator("section#proj");
  const rows = proj.locator("tbody tr");
  await proj.getByRole("button", { name: "+ row" }).click();
  await expect(rows).toHaveCount(2);
  await expect(proj.locator("li").first()).toBeVisible();
  await proj.getByRole("button", { name: "Delete row 2 of PROJ" }).click();
  await expect(rows).toHaveCount(1);
  await expect(proj.locator("li")).toHaveCount(0);

  // Undo resurrects the deleted row; again, the add; redo replays it.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(rows).toHaveCount(2);
  await page.keyboard.press("ControlOrMeta+z");
  await expect(rows).toHaveCount(1);
  await page.keyboard.press("ControlOrMeta+Shift+z");
  await expect(rows).toHaveCount(2);

  // Deleting AT the pick with rows still below it: the selection must close,
  // not silently re-aim at the row that shifted up. Four rows, pick row 2,
  // delete row 2 — an out-of-range guard alone would keep the pick alive
  // pointing at the former row 3.
  await proj.getByRole("button", { name: "+ row" }).click();
  await proj.getByRole("button", { name: "+ row" }).click();
  await expect(rows).toHaveCount(4);
  await proj
    .getByRole("button", { name: "Edit PROJ_ID on row 2 of PROJ" })
    .click();
  // `.bg-accent-quiet` matches the class TOKEN — a [class*=] substring match
  // would also catch every cell's hover:bg-accent-quiet.
  await expect(proj.locator("td button.bg-accent-quiet")).toHaveCount(1);
  await proj.getByRole("button", { name: "Delete row 2 of PROJ" }).click();
  await expect(rows).toHaveCount(3);
  await expect(proj.locator("td button.bg-accent-quiet")).toHaveCount(0);

  // Undo covers cell edits from the in-place editor too.
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();
  await page.keyboard.press("Enter");
  await page.keyboard.type("11.80");
  await page.keyboard.press("Enter");
  await expect(cell).toContainText("11.80");
  await page.keyboard.press("ControlOrMeta+z");
  await expect(cell).toContainText("11.8");
  await expect(cell).not.toContainText("11.80");
});

test("wide: the depth scale clears the masthead and labels the hole's floor", async ({
  page,
}) => {
  test.skip(
    width(page) < 1088,
    "the depth scale only renders above the rail's 68rem collapse breakpoint",
  );
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

  // At scroll 0 the first tick sits fully below the sticky masthead. The #524
  // defect padded a box its absolutely-positioned children ignore, which put
  // "0.00 m / Surface" underneath the masthead — visible only during macOS
  // overscroll bounce.
  const masthead = await page.locator("header").boundingBox();
  // The tick's CONTAINER, not its label: the 1px tick line renders at the
  // container's top, ~3px above the label's own box, and it is the line that
  // must clear the masthead.
  const surfaceTick = await page
    .getByText("0.00 m", { exact: true })
    .locator("xpath=..")
    .boundingBox();
  if (!masthead || !surfaceTick)
    throw new Error("masthead or 0.00 m tick did not render");
  expect(
    surfaceTick.y,
    "the 0.00 tick must clear the masthead",
  ).toBeGreaterThan(masthead.y + masthead.height);

  // The hole has a floor and the scale says so: a terminal tick labels the
  // seeded final depth, below every section tick. Section ticks mark section
  // TOPS, so before #524 the deepest label was the last section's top and the
  // bottom of the hole went unlabelled.
  const terminal = await page
    .getByText(`${seededFinalDepthLabel()} m`, { exact: true })
    .boundingBox();
  if (!terminal) throw new Error("no terminal tick at the seeded final depth");
  const tickTops = await page
    .getByText(/^\d+\.\d{2} m$/)
    .evaluateAll((els) => els.map((el) => el.getBoundingClientRect().y));
  expect(terminal.y).toBeCloseTo(Math.max(...tickTops), 1);

  // The pill agreement, end to end: the unit lane pins the ARITHMETIC (a band
  // top is railY of its fraction — railScroll.test.ts), and this pins that
  // the component wires both tick and pill through it: at full scroll the
  // pill reads the floor's depth and its centre sits on the terminal tick.
  // `.rounded-pill` is the depth pill's one distinguishing class; nothing
  // else on the page uses it.
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  const pill = page.locator(".rounded-pill");
  await expect(pill).toHaveText(seededFinalDepthLabel());
  const pillBox = await pill.boundingBox();
  const terminalTick = await page
    .getByText(`${seededFinalDepthLabel()} m`, { exact: true })
    .locator("xpath=..")
    .boundingBox();
  if (!pillBox || !terminalTick)
    throw new Error("pill or terminal tick did not render");
  const pillCentre = pillBox.y + pillBox.height / 2;
  expect(
    Math.abs(pillCentre - terminalTick.y),
    "the pill must sit ON the terminal tick, not merely read its number",
  ).toBeLessThanOrEqual(1);

  await expectViewportWide(page);
});

test("fine: findings speak one vocabulary — tinted cell with tooltip, table strip, chipped panel", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the tooltip needs a pointer that can hover (#526)");
  await page.goto("/");

  // Arm the engine by selecting the seeded Rule 8 cell; the findings arrive
  // and the cell wears the ERROR grammar: tint + marker + tooltip carrying
  // what the engine actually said.
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();
  await expect(cell).toContainText("✗");
  const td = cell.locator("xpath=ancestor::td[1]");
  await expect(td).toHaveClass(/bg-err-quiet/);
  await cell.getByText("✗").hover();
  const tooltip = page.getByRole("tooltip");
  await expect(tooltip).toBeVisible();
  await expect(tooltip).toContainText("Rule 8");

  // Group-level findings live in a strip attached to the TABLE, not in the
  // prose column: the strip's parent is the table's own column.
  const strip = page.getByRole("list", { name: "SAMP findings" });
  await expect(strip.getByText(/Rule/).first()).toBeVisible();
  expect(
    await strip.evaluate((el) => !!el.parentElement?.querySelector("table")),
    "the strip must share a column with the table it judges",
  ).toBe(true);

  // The two Rule 16 findings carry byte-identical text against SAMP and LLPL;
  // the panel's GROUP chip is what tells them apart. Without the chip no
  // panel row mentions LLPL at all — the text itself only names SAMP_TYPE.
  const panelRows = page
    .locator("section#file li")
    .filter({ hasText: "not defined" });
  await expect(panelRows).toHaveCount(2);
  await expect(panelRows.filter({ hasText: "LLPL" })).toHaveCount(1);
});

test("fine: the TRAN cover sheet seeds clean, and Rule 14 only fires when the reader causes it", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "drives the in-place editor (#527)");
  await page.goto("/");

  // The cover sheet renders in the File section as an ordinary group table.
  const file = page.locator("section#file");
  const coverCell = file.getByRole("button", {
    name: "Edit TRAN_PROD on row 1 of TRAN",
  });
  await expect(coverCell).toBeVisible();

  // Arm via the cover sheet itself; once findings render, Rule 14 is NOT
  // among them — the permanent unclearable finding is gone from the seed.
  await coverCell.click();
  await expect(file.getByText("AGS Format Rule 8").first()).toBeVisible();
  // Scoped to findings rows — the cover sheet's own PROSE names Rule 14
  // (that is the lesson), so a bare text match would always hit.
  await expect(file.locator("li").filter({ hasText: "Rule 14" })).toHaveCount(
    0,
  );

  // Blanking a REQUIRED value raises a finding live, in the cover sheet's
  // own strip (the engine answers Rule 10b, against the group)...
  await page.keyboard.press("Enter");
  await page.keyboard.press("Backspace");
  await page.keyboard.press("Enter");
  const strip = page.getByRole("list", { name: "TRAN findings" });
  await expect(strip.getByText("Rule 10b")).toBeVisible();

  // ...and undo clears it — cause, read, unwind.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(strip).toBeHidden();

  // Deleting the cover sheet's one row is what Rule 14 exists to catch.
  await file.getByRole("button", { name: "Delete row 1 of TRAN" }).click();
  await expect(strip.getByText("Rule 14")).toBeVisible();
  await page.keyboard.press("ControlOrMeta+z");
  await expect(strip).toBeHidden();
});
