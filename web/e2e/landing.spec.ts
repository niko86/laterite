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
// This spec runs under the `landing` (390) and `landing-wide` (1280) projects
// (playwright.config.ts), against the landing's OWN preview server — the
// landing is a separate build (see web/landing/vite.config.ts), so the app's
// server cannot serve it. Width-specific tests skip themselves on the other
// project, the way layout.spec.ts branches on viewport.

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

test("phone: opening the row editor does not widen the page", async ({
  page,
}) => {
  await page.goto("/");

  // LLPL is the widest table (nine columns, every cell nowrap) and the row
  // editor is the pattern chosen to carry it at 390px — so it is the
  // pairing most able to push the page out. The first column is sticky, so
  // its cell is tappable without panning.
  await page
    .getByRole("button", { name: "Edit LOCA_ID on row 1 of LLPL" })
    .click();
  await expect(
    page.getByRole("group", { name: "Editing row 1 of LLPL" }),
  ).toBeVisible();

  await expectViewportWide(page);
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
