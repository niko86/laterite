import { test, expect, type Locator, type Page } from "@playwright/test";
import { expectViewportWide } from "./viewport";
import { expectErrBorder, hexToRgb } from "./tokens";
import { INSTALL_CHANNELS } from "../landing/installChannels";
import { SECTIONS } from "../landing/sections";
import { RAIL_INSET_PCT } from "../landing/components/railScroll";
import { readFileSync, readdirSync } from "node:fs";
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

/** The failing cell's corner flag (#616; re-anchored to the cell by
 *  #632) — the one absolutely-positioned span inside the cell's td. */
const cornerFlag = (cell: Locator) =>
  cell.locator("xpath=ancestor::td[1]//span[contains(@class, 'absolute')]");

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
  // clientWidth == scrollWidth: the page had grown instead. The contract
  // covers every table added since, too: "file" is the TRAN cover sheet
  // (#527), which lives in the File section rather than a descent section
  // of its own.
  for (const section of [...GROUPS, "file"]) {
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

  // The editor's own enter/exit rides the slow tier (#534) — this is the one
  // animated element the fine-pointer motion probe can never reach, and the
  // fade came unprobed through review once already. Both sides normalized to
  // milliseconds: the registered token serializes in seconds.
  const ms = (v: string) =>
    v.trim().endsWith("ms") ? parseFloat(v) : parseFloat(v) * 1000;
  const wrapperDuration = await carousel.evaluate(
    (el) =>
      getComputedStyle(el.closest(".transition-opacity") ?? el)
        .transitionDuration,
  );
  const slowToken = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--dur-slow"),
  );
  expect(ms(wrapperDuration), "editor wrapper").toBe(ms(slowToken));

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

test("touch: a typed word is one undo step, not one per keystroke", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the carousel is the coarse pointer's editor (#550)");
  await page.goto("/");

  const cell = page.getByRole("button", {
    name: "Edit LOCA_ID on row 1 of LLPL",
  });
  const before = await cell.innerText();
  await cell.click();

  // pressSequentially, never fill(): coalescing exists for the per-keystroke
  // commits the carousel really makes, and fill() would collapse them into
  // one input event — a test that cannot fail.
  const input = page.getByRole("textbox", { name: "LOCA_ID value" });
  await input.click();
  await page.keyboard.press("End");
  await input.pressSequentially("XYZ");
  await expect(cell).toContainText("XYZ");

  await page.getByRole("button", { name: "Close the row editor" }).click();
  await expect(
    page.getByRole("group", { name: "Editing row 1 of LLPL" }),
  ).toBeHidden();

  // ONE undo unwinds the whole word (#550) — before coalescing this needed
  // one per character, and the third press here would still show a suffix.
  await page.keyboard.press("Control+z");
  await expect(cell).toHaveText(before);

  // Redo rides the same coalescing: one step forward restores the whole
  // word, and one step back clears it again.
  await page.keyboard.press("Control+Shift+z");
  await expect(cell).toContainText("XYZ");
  await page.keyboard.press("Control+z");
  await expect(cell).toHaveText(before);

  // Reopening the SAME cell is a run boundary, not a continuation: two
  // stays, two undo steps. Without the pick-setter boundary the second stay
  // would fold into the first run's base and one undo would eat both words.
  await cell.click();
  await input.click();
  await page.keyboard.press("End");
  await input.pressSequentially("AB");
  await page.getByRole("button", { name: "Close the row editor" }).click();
  await cell.click();
  await input.click();
  await page.keyboard.press("End");
  await input.pressSequentially("CD");
  await page.getByRole("button", { name: "Close the row editor" }).click();
  await expect(
    page.getByRole("group", { name: "Editing row 1 of LLPL" }),
  ).toBeHidden();
  await page.keyboard.press("Control+z");
  await expect(cell).toContainText("AB");
  await expect(cell).not.toContainText("CD");
  await page.keyboard.press("Control+z");
  await expect(cell).toHaveText(before);
});

test("touch: while a card holds focus, undo stays native — the model timeline waits for the close", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the carousel is the coarse pointer's editor (#550)");
  await page.goto("/");

  // A model mutation to be undone: PROJ gains a second row.
  const proj = page.locator("section#proj");
  const rows = proj.locator("tbody tr");
  await proj.getByRole("button", { name: "+ row" }).click();
  await expect(rows).toHaveCount(2);

  // With a field card holding focus, the shortcut must stay NATIVE — #525's
  // recorded decision, pinned here (#550): the model shortcut would yank the
  // delivery out from under a half-typed value. The row must not vanish.
  await page
    .getByRole("button", { name: "Edit LOCA_ID on row 1 of LLPL" })
    .click();
  await page.getByRole("textbox", { name: "LOCA_ID value" }).click();
  await page.keyboard.press("Control+z");
  await expect(rows).toHaveCount(2);

  // Closed card, same shortcut: now it reaches the model and the add undoes.
  await page.getByRole("button", { name: "Close the row editor" }).click();
  await expect(
    page.getByRole("group", { name: "Editing row 1 of LLPL" }),
  ).toBeHidden();
  await page.keyboard.press("Control+z");
  await expect(rows).toHaveCount(1);
});

test.describe("fine: the clipboard contract (#551)", () => {
  // The contract is stated beside the handler it pins (GroupTable's onKeys):
  // selected-cell chords are OURS, open editors are the browser's. These are
  // the tests that go red if either half changes.
  test.use({ permissions: ["clipboard-read", "clipboard-write"] });

  test("fine: copy takes the raw value, paste commits once, undo unwinds it in one step", async ({
    page,
    hasTouch,
  }) => {
    test.skip(hasTouch, "selected-cell chords are the fine pointer's (#525)");
    await page.goto("/");

    // Copy: the cell's VALUE, never the status glyph rendered beside it.
    const src = page.getByRole("button", {
      name: "Edit LOCA_GL on row 1 of LOCA",
    });
    await src.click();
    await page.keyboard.press("Control+c");
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
      "11.8",
    );

    // Paste: committed through the store funnel — ONE history entry, so one
    // undo restores, exactly like a typed edit (the AC #551 names).
    const target = page.getByRole("button", {
      name: "Edit LOCA_ID on row 1 of LOCA",
    });
    await target.click();
    await page.keyboard.press("Control+v");
    await expect(target).toContainText("11.8");
    await page.keyboard.press("Control+z");
    await expect(target).toHaveText("BH01");
  });

  test("fine: an open editor is the browser's clipboard, not the cell handler's", async ({
    page,
    hasTouch,
  }) => {
    test.skip(hasTouch, "selected-cell chords are the fine pointer's (#525)");
    await page.goto("/");
    await page.evaluate(() => navigator.clipboard.writeText("SENTINEL"));

    const cell = page.getByRole("button", {
      name: "Edit LOCA_ID on row 1 of LOCA",
    });
    await cell.click();
    await page.keyboard.press("Enter");
    // With the editor open, the chord must reach the INPUT (native paste,
    // which Escape then discards) and never the cell-level handler — had the
    // handler fired, SENTINEL would be committed past the cancel.
    await page.keyboard.press("Control+v");
    await page.keyboard.press("Escape");
    // The cell handler's paste is ASYNC (clipboard read then commit), so an
    // immediate assertion can win the race and pass in a world where the
    // handler fired. The settle is an OBSERVABLE, not a timeout: the browser
    // services clipboard requests in order, so a read issued now cannot
    // resolve before any read the handler queued earlier — by the time this
    // returns, a fired handler would already have committed.
    await page.evaluate(() => navigator.clipboard.readText());
    await expect(cell).toHaveText("BH01");
  });

  test("fine: a multi-line clipboard lands as one line, the same one the editor's own input yields", async ({
    page,
    hasTouch,
  }) => {
    test.skip(hasTouch, "selected-cell chords are the fine pointer's (#525)");
    await page.goto("/");
    await page.evaluate(() => navigator.clipboard.writeText("AB\nCD"));

    // MEASURE FIRST, off a live input: a native single-line field is the rule
    // the handler is defined by, so the reference has to be observed rather
    // than restated. The editor opens select-all, so the paste replaces.
    const other = page.getByRole("button", {
      name: "Edit LOCA_REM on row 1 of LOCA",
    });
    await other.click();
    await page.keyboard.press("Enter");
    const input = page.getByRole("textbox", {
      name: "LOCA_REM on row 1 of LOCA",
    });
    // keyboard.press does NOT auto-wait; the editor focuses in onMount, and a
    // paste that arrives first lands on the cell button, where onKeys returns
    // early because editing() is already set — so nothing would paste anywhere.
    await expect(input).toBeFocused();
    await page.keyboard.press("ControlOrMeta+v");
    const native = await input.inputValue();
    await page.keyboard.press("Escape");

    // The handler path (#574): the ONE entry point no browser sanitizes for
    // us. Compare it to what was just MEASURED, not to a repeated literal —
    // two hardcoded constants agree with each other whatever the handler does,
    // which is exactly how a wrong normalization rule survives a green suite.
    const target = page.getByRole("button", {
      name: "Edit LOCA_ID on row 1 of LOCA",
    });
    await target.click();
    await page.keyboard.press("ControlOrMeta+v");
    // textContent, NOT toHaveText: the matcher normalizes whitespace, and the
    // browser collapses a rendered newline to a space — so an unfixed handler
    // that commits "AB\nCD" verbatim reads as "AB CD" and the assertion passes
    // over the very defect it exists for. The raw string is the only witness.
    await expect.poll(() => target.textContent()).toBe(native);

    // And pin the measurement itself, so a browser that changes its mind is a
    // visible failure here rather than a target both sides move to together.
    // The value is a SPACE, not nothing: reading HTML's value sanitization
    // algorithm suggests the terminator is stripped, and a live paste disagrees.
    expect(native).toBe("AB CD");
  });

  test("fine: a pasted newline cannot make another group's fix truncate this row", async ({
    page,
    hasTouch,
  }) => {
    test.skip(hasTouch, "selected-cell chords are the fine pointer's (#525)");
    await page.goto("/");
    await page.evaluate(() => navigator.clipboard.writeText("AB\nCD"));

    // The lossy path #574 was filed for. A terminator in a cell tears the
    // DATA record in two; the demo's own parser then drops the fragment
    // after the break, because it opens with no data descriptor. Applying
    // ANOTHER group's fixes reparses the whole file, so the damage lands on
    // a row the reader never touched — and the row COUNT survives it, which
    // is why this asserts a sibling cell rather than a row tally.
    const ref = page.getByRole("button", {
      name: "Edit SAMP_REF on row 1 of SAMP",
    });
    await ref.click();
    await page.keyboard.press("ControlOrMeta+v");
    // Deliberately NO assertion on the pasted cell here. In a regressed world
    // the handler commits a raw terminator, so checking the normalized value
    // first fails at this line — and the cross-group truncation below, which
    // is this test's entire subject, never runs. Settle the async clipboard
    // read instead, using the ordering guarantee the sibling test relies on:
    // a read issued now cannot resolve before one the handler queued earlier.
    await page.evaluate(() => navigator.clipboard.readText());

    // LOCA carries the one seeded engine fix and SAMP is not in its scope,
    // so the fixer never visits the pasted cell (#530 scopes by group).
    await page
      .getByRole("button", { name: "Fix 1 auto-fixable in LOCA" })
      .click();
    await expect(
      page.getByRole("button", { name: "Fix 0 auto-fixable in LOCA" }),
    ).toBeVisible();

    await expect(
      page.getByRole("button", { name: "Edit SAMP_ID on row 1 of SAMP" }),
    ).toHaveText("BH01-S1");
    await expect.poll(() => ref.textContent()).toBe("AB CD");
  });
});

test("fine: click selects, Enter edits in place, Esc cancels, Tab commits and moves, arrows navigate, typing replaces", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "in-place editing is the fine pointer's editor (#525)");
  await page.goto("/");

  // The seeded Rule 8 defect: LOCA_GL is 11.8 where 2DP demands 11.80. The
  // click selects the cell; the engine is already loading eagerly (#531),
  // and the click's own arm() is just the fast path — either way the
  // corner flag (#616) arrives on this cell.
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();
  // The pick's sign on a FAILING cell is the #618 ring — the wash stands
  // down there since #633, so the severity tint stays the only hue.
  await expect(cell.locator("xpath=ancestor::td[1]")).toHaveCSS(
    "box-shadow",
    /inset/,
  );
  await expect(cornerFlag(cell)).toBeVisible();

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
  await expect(cornerFlag(cell)).toHaveCount(0);

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
  // The blank row's findings land in the panel — the strip is the coarse
  // pointer's surface since #591, and this lane's pointer is fine.
  const projFindings = page.locator("#findings li").filter({ hasText: "PROJ" });
  // Attached, not visible: below the breakpoint the panel is a one-card
  // carousel (#592) and this card need not be the one showing — the claim
  // here is that the finding REACHED the panel.
  await expect(projFindings.first()).toBeAttached();
  await proj.getByRole("button", { name: "Delete row 2 of PROJ" }).click();
  await expect(rows).toHaveCount(1);
  await expect(projFindings).toHaveCount(0);

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
  // would also catch a clean cell's hover:bg-accent-quiet (#633 removed the
  // hover token from failing cells only).
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
  // #615's datum, revising a sliver of #585's flush ruling (pass-2 pin
  // D2-01): 0.00 m sits one spacing-token step BELOW the gradient bar, and
  // the expected line is read from a section's own scroll-margin — the one
  // token the ticks, the probe and the jumps all consume — so this test
  // cannot agree with the rail while disagreeing with where a jump lands.
  const datumOffset = await page.evaluate(() => {
    const section = document.querySelector("section");
    return section ? parseFloat(getComputedStyle(section).scrollMarginTop) : 0;
  });
  expect(
    datumOffset - masthead.height,
    "the datum keeps real air below the bar",
  ).toBeGreaterThan(4);
  expect(
    Math.abs(surfaceTick.y - (masthead.y + datumOffset)),
    "the 0.00 tick must sit on the datum line",
  ).toBeLessThanOrEqual(2);

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

test("fine: findings speak one vocabulary — tinted cell with popover, chipped panel", async ({
  page,
  hasTouch,
}) => {
  test.skip(
    hasTouch,
    "the popover needs a pointer that can hover (#526, #591)",
  );
  await page.goto("/");

  // Select the seeded Rule 8 cell (arming is eager since #531 — the click
  // is only the fast path); the findings arrive and the cell wears the
  // ERROR grammar: tint + marker + the popover carrying what the engine
  // actually said (#591 — the strip is the coarse pointer's surface now).
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();
  await expect(cornerFlag(cell)).toBeVisible();
  const td = cell.locator("xpath=ancestor::td[1]");
  await expect(td).toHaveClass(/bg-err-quiet/);
  await cell.hover();
  const tooltip = page.getByRole("tooltip");
  await expect(tooltip).toBeVisible();
  await expect(tooltip).toContainText("Rule 8");

  // The two Rule 16 findings carry byte-identical text against SAMP and LLPL;
  // the panel's GROUP chip is what tells them apart. Without the chip no
  // panel row mentions LLPL at all — the text itself only names SAMP_TYPE.
  const panelRows = page
    .locator("section#file li")
    .filter({ hasText: "not defined" });
  await expect(panelRows).toHaveCount(2);
  await expect(panelRows.filter({ hasText: "LLPL" })).toHaveCount(1);
});

test("the corner flag pins the cell's corner, and a failing pick wears one wash", async ({
  page,
}) => {
  // #632: the flag was anchored to the inner edit button, which sits
  // inside the td's padding — so the triangle floated mid-cell instead of
  // pinning the corner the way a spreadsheet annotation does. The claim
  // is geometric: the flag's top-right IS the td's top-right.
  await page.goto("/");
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await expect(cornerFlag(cell)).toBeVisible({ timeout: 15_000 });
  const td = cell.locator("xpath=ancestor::td[1]");
  const flagBox = await cornerFlag(cell).boundingBox();
  const tdBox = await td.boundingBox();
  if (!flagBox || !tdBox) throw new Error("flag and cell must lay out");
  expect(flagBox.y, "flag top = cell top").toBeCloseTo(tdBox.y, 0);
  expect(flagBox.x + flagBox.width, "flag right = cell right").toBeCloseTo(
    tdBox.x + tdBox.width,
    0,
  );

  // #633: picking a FAILING cell used to add the accent wash on top of
  // the severity tint — two hues, one cell. The wash stands down there
  // (the #618 ring and the tint carry the pick); on fine lanes the mouse
  // is left sitting on the cell, so this also proves hover adds none.
  //
  // TIMING IS THE TRAP here: the button transitions its colours, so a
  // computed background read at click time reports the transition's
  // START value and a wash still fading in reads as absent. The clean
  // cell's wash is asserted with a retrying matcher (also the positive
  // control that keeps the claim falsifiable), and the failing cell is
  // read once only AFTER the clean cell's wash has fully faded back out
  // — a whole transition provably elapsed since the pick moved, so a
  // pre-fix wash would have landed by then.
  const TRANSPARENT = "rgba(0, 0, 0, 0)";
  const clean = page.getByRole("button", {
    name: "Edit LOCA_ID on row 1 of LOCA",
  });
  await clean.click();
  await expect(clean, "the clean cell keeps the wash").not.toHaveCSS(
    "background-color",
    TRANSPARENT,
  );
  const failing = page.getByRole("button", {
    name: "Edit SAMP_TYPE on row 1 of SAMP",
  });
  await failing.click();
  await expect(clean, "the pick moved off the clean cell").toHaveCSS(
    "background-color",
    TRANSPARENT,
  );
  expect(
    await failing.evaluate((el) => getComputedStyle(el).backgroundColor),
    "no second wash on a failing cell",
  ).toBe(TRANSPARENT);
  expect(
    await failing
      .locator("xpath=ancestor::td[1]")
      .evaluate((el) => getComputedStyle(el).boxShadow),
    "the ring still marks the pick",
  ).toContain("inset");
  // …and the severity tint is what remains painting the cell: the wash
  // stood down FOR it, so its absence would mean an untinted verdict.
  expect(
    await failing
      .locator("xpath=ancestor::td[1]")
      .evaluate((el) => getComputedStyle(el).backgroundColor),
    "the severity tint stands",
  ).not.toBe(TRANSPARENT);
});

test("a long finding stays inside its hover popover", async ({
  page,
  hasTouch,
}) => {
  // #636: the Rule 10c KEY-combination token is one long unbroken run,
  // and engines differ on whether a pipe is a break opportunity — the
  // report's engine let it burst past the popover's border. The claim
  // is overflow-free content, whatever the engine's line breaker does.
  test.skip(hasTouch, "the popover is the fine pointer's surface (#591)");
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  await page.locator('section#llpl [data-cell="2-1"]').hover();
  const pop = page.getByRole("tooltip");
  await expect(pop).toBeVisible();
  await expect(pop).toContainText("Rule 10c");
  expect(
    await pop.evaluate((el) => el.scrollWidth - el.clientWidth),
    "content escapes the popover",
  ).toBeLessThanOrEqual(0);

  // The callout's other homes keep their wrap contract: the panel's
  // cards were never nowrap-poisoned, and this pins that the shared
  // class change left them overflow-free too.
  expect(
    await page
      .locator("#findings li")
      .first()
      .evaluate((el) => el.scrollWidth - el.clientWidth),
    "the findings panel home stays overflow-free",
  ).toBeLessThanOrEqual(0);
});

test("the corner flag arrives with the findings, no pointer required", async ({
  page,
}) => {
  // #616's fine-AND-coarse criterion, honestly: the flag renders from the
  // FINDINGS, not from an interaction, so it must appear on every modality
  // without a click — which is exactly what lets the touch lane (the one
  // project every in-place-editor test skips) verify it. The inline glyph
  // it replaced is asserted gone in the same breath.
  await page.goto("/");
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await expect(cornerFlag(cell)).toBeVisible({ timeout: 15_000 });
  await expect(cell).not.toContainText("\u2717");
});

test("a picked cell wears the selection ring, and it travels with the arrows", async ({
  page,
  hasTouch,
}) => {
  // #618 (pass-2 pin D2-09): the first click PICKS (the #593 grammar), and
  // its only sign was the button's pale wash — read in the review as a dead
  // click. The ring is asserted on the TD's computed shadow rather than a
  // class name, so the test reads what the reader sees, on every modality:
  // fine and wide click, coarse taps, and the same pick either way.
  await page.goto("/");
  const cell = page.getByRole("button", {
    name: "Edit LOCA_TYPE on row 1 of LOCA",
  });
  await cell.click();
  const ringOf = (c: Locator) =>
    c
      .locator("xpath=ancestor::td[1]")
      .evaluate((el) => getComputedStyle(el).boxShadow);
  expect(await ringOf(cell), "first click draws the ring").toContain("inset");
  // On a CLEAN cell the wash stays underneath: the ring joins it, not
  // replaces it. (A failing cell is the wash-free case — #633.)
  await expect(cell).toHaveClass(/bg-accent-quiet/);

  if (!hasTouch) {
    // Arrows move the pick, and the ring is the pick's — it must follow.
    await page.keyboard.press("ArrowRight");
    const next = page.getByRole("button", {
      name: "Edit LOCA_GL on row 1 of LOCA",
    });
    expect(await ringOf(next), "the ring follows the arrow").toContain("inset");
    expect(await ringOf(cell), "and leaves the old cell").not.toContain(
      "inset",
    );
  }
});

test("the header marks speak the dictionary's status grammar", async ({
  page,
}) => {
  // #616: bare key glyph = KEY, boxed key = KEY+REQUIRED, `*` = REQUIRED
  // alone, nothing = OTHER — and the words live in the header's title, so
  // the marks stay decorative. One header of each status, from the page's
  // own headings: the axes are independent (10a identity, 10b non-empty)
  // and TRAN is where they separate.
  await page.goto("/");

  // LOCA_ID in its own group: KEY only — the glyph, unboxed.
  const locaId = page.locator("section#loca th").filter({ hasText: "LOCA_ID" });
  await expect(locaId.locator("svg")).toBeVisible();
  await expect(locaId.locator("span[class*='border-current']")).toHaveCount(0);
  await expect(
    locaId.locator("span[title*='KEY: part of the row']"),
  ).toHaveCount(1);

  // PROJ_ID: KEY+REQUIRED — the glyph in its box.
  const projId = page.locator("section#proj th").filter({ hasText: "PROJ_ID" });
  await expect(projId.locator("svg")).toBeVisible();
  await expect(projId.locator("span[class*='border-current']")).toBeVisible();
  await expect(projId.locator("span[title*='KEY and REQUIRED']")).toHaveCount(
    1,
  );

  // TRAN_DATE: REQUIRED without KEY — the form-convention asterisk.
  const tranDate = page
    .locator("section#file th")
    .filter({ hasText: "TRAN_DATE" });
  await expect(tranDate.getByText("*", { exact: true })).toBeVisible();
  await expect(tranDate.locator("svg")).toHaveCount(0);
  await expect(
    tranDate.locator("span[title*='REQUIRED: must not be empty']"),
  ).toHaveCount(1);

  // PROJ_NAME: OTHER — no mark at all.
  const projName = page
    .locator("section#proj th")
    .filter({ hasText: "PROJ_NAME" });
  await expect(projName.locator("svg")).toHaveCount(0);
  await expect(projName.getByText("*", { exact: true })).toHaveCount(0);
});

test("the aligned view keeps every line where it was, and jumps still land", async ({
  page,
}) => {
  // #620 (pass-2 pins D2-12, M2-06): the webapp's "Aligned columns" grammar
  // as a display-only VIEW. The load-bearing property is that alignment is
  // intra-line only — same line count, same numbers — so a finding jump
  // lands identically in both modes. The tag columns are the crisp probe:
  // "HEADING" and "DATA" differ in width, so their first commas differ in
  // raw and must coincide in aligned.
  await page.goto("/");
  const file = page.locator("section#file");
  await expect(file.locator("li").first()).toBeVisible({ timeout: 15_000 });
  const toggle = file.getByRole("checkbox", { name: "Aligned columns" });
  await expect(toggle).not.toBeChecked();

  const pane = file.locator(".overscroll-contain");
  const rows = pane.locator("div.flex");
  const rawCount = await rows.count();

  // A third, independent copy of the in-quote comma walk ON PURPOSE: the
  // helper under test must not be its own oracle, so the spec re-derives
  // the split rather than importing align.ts.
  const firstCommaAt = async (needle: string) => {
    const text = await rows
      .filter({ hasText: needle })
      .first()
      .locator("span")
      .nth(1)
      .innerText();
    let inQuote = false;
    for (let i = 0; i < text.length; i++) {
      if (text[i] === '"') inQuote = !inQuote;
      else if (text[i] === "," && !inQuote) return i;
    }
    return -1;
  };

  const jumpLandsOn = async () => {
    await page
      .locator("#findings li")
      .filter({ hasText: "Rule 8" })
      .first()
      .locator("button")
      .first()
      .click();
    const focused = pane.locator("div.flex[class*='--focus-ring']").first();
    await expect(focused).toBeVisible();
    return focused.locator("span").first().innerText();
  };

  // Needles that survive the padding: alignment inserts spaces BEFORE
  // commas, so a needle spanning one would stop matching in aligned mode.
  // First occurrence of "LOCA_ID" is LOCA's HEADING row; of "BH01", LOCA's
  // first DATA row.
  const rawComma = await firstCommaAt('"LOCA_ID"');
  expect(rawComma).not.toBe(await firstCommaAt('"BH01"'));
  const rawLanding = await jumpLandsOn();

  await toggle.check();
  await expect(rows).toHaveCount(rawCount);
  expect(await firstCommaAt('"LOCA_ID"')).toBe(await firstCommaAt('"BH01"'));
  expect(await jumpLandsOn()).toBe(rawLanding);

  // And back: raw is one uncheck away, bytes exact again.
  await toggle.uncheck();
  expect(await firstCommaAt('"LOCA_ID"')).toBe(rawComma);
});

test("phone: aligned mode trades the wrap for a pan", async ({ page }) => {
  test.skip(
    width(page) >= 1024,
    "the wrap this trades away only exists below the layout breakpoint",
  );
  // M2-06: columnar text cannot wrap and stay columnar, so aligned mode
  // opts out of #596's soft wrap and rides the pane's own scroller
  // sideways instead. Raw keeps the wrap — that half is pinned by the
  // existing pane test, and re-checked here after a round trip.
  await page.goto("/");
  const file = page.locator("section#file");
  await expect(file.locator("li").first()).toBeVisible({ timeout: 15_000 });
  const scroller = file.locator(".overscroll-contain");
  const overflowsX = () =>
    scroller.evaluate((el) => el.scrollWidth > el.clientWidth + 1);
  expect(await overflowsX(), "raw wraps, no sideways scroll").toBe(false);

  await file.getByRole("checkbox", { name: "Aligned columns" }).check();
  expect(await overflowsX(), "aligned pans sideways").toBe(true);

  await file.getByRole("checkbox", { name: "Aligned columns" }).uncheck();
  expect(await overflowsX(), "raw's wrap comes back").toBe(false);
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

  // Select via the cover sheet itself (arming is eager since #531); once
  // findings render, Rule 14 is NOT
  // among them — the permanent unclearable finding is gone from the seed.
  await coverCell.click();
  await expect(file.getByText("AGS Format Rule 8").first()).toBeVisible();
  // Scoped to findings rows — the cover sheet's own PROSE names Rule 14
  // (that is the lesson), so a bare text match would always hit.
  await expect(file.locator("li").filter({ hasText: "Rule 14" })).toHaveCount(
    0,
  );

  // Blanking a REQUIRED value raises a finding live, read from the panel
  // (the engine answers Rule 10b, against the group; the strip is the
  // coarse pointer's surface since #591)...
  await page.keyboard.press("Enter");
  await page.keyboard.press("Backspace");
  await page.keyboard.press("Enter");
  const rule10b = page.locator("#findings li").filter({ hasText: "Rule 10b" });
  // Attached: the narrow panel pages one card at a time (#592).
  await expect(rule10b.first()).toBeAttached();

  // ...and undo clears it — cause, read, unwind.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(rule10b).toHaveCount(0);

  // Deleting the cover sheet's one row is what Rule 14 exists to catch.
  await file.getByRole("button", { name: "Delete row 1 of TRAN" }).click();
  const rule14 = page.locator("#findings li").filter({ hasText: "Rule 14" });
  await expect(rule14.first()).toBeAttached();
  await page.keyboard.press("ControlOrMeta+z");
  await expect(rule14).toHaveCount(0);
});

test("the stack intro carries transport and the guide, and the aside is gone", async ({
  page,
}) => {
  // #617 (pass-2 pins D2-08, D2-06), superseding #528's aside and a sliver
  // of #596: the pack/lock diagram no longer exists at ANY width — its
  // story is one sentence at the end of the Pick-your-stack intro, carrying
  // the same cookbook link — and the install-guide line moved up from below
  // the grid into the same tail, so the intro ends guide-in-hand.
  await page.goto("/");
  await expect(
    page.getByRole("complementary", { name: "Transport" }),
  ).toHaveCount(0);

  // One paragraph, both links, in the intro — not below the grid: the
  // sentence order pins the tail (transport, then guide), and link COUNTS
  // pin that the old below-grid line did not survive as a duplicate.
  const intro = page
    .locator("section#install p")
    .filter({ hasText: "One engine behind every one of these" });
  await expect(intro).toContainText("packs deliveries");
  const transport = intro.getByRole("link", {
    name: "Pack / encrypt for transport",
  });
  await expect(transport).toHaveAttribute(
    "href",
    "https://docs.laterite.dev/cookbook/transport/",
  );
  const guide = intro.getByRole("link", { name: "Full install guide" });
  await expect(guide).toHaveAttribute(
    "href",
    "https://docs.laterite.dev/learn/install/",
  );
  await expect(
    page.getByRole("link", { name: "Full install guide" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("link", { name: "Pack / encrypt for transport" }),
  ).toHaveCount(1);

  // The tail's ORDER is the decided design, not an accident of today's
  // markup: transport sentence first, then the guide, and the guide's beta
  // note is the paragraph's last word — "ends with", pinned as ends-with.
  expect(
    await intro.evaluate((el) => {
      const links = el.querySelectorAll("a");
      return Array.from(links).map((a) => a.textContent?.trim());
    }),
  ).toEqual(["Pack / encrypt for transport", "Full install guide"]);
  await expect(intro).toContainText(/version numbers still move quickly\.$/);

  // The page still fits the viewport with the longer intro in the layout.
  await expectViewportWide(page);
});

test("fine: a deleted group leaves a restore stub — restore means the seed, undo means the edits", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "drives the in-place editor (#529)");
  await page.goto("/");

  // Every rendered table carries the control — the descent four and TRAN —
  // and ONLY those: the groups that render solely in the output pane
  // (UNIT/TYPE/ABBR) have no table, so they get no control (#529).
  for (const code of ["PROJ", "LOCA", "SAMP", "LLPL", "TRAN"]) {
    await expect(
      page.getByRole("button", { name: `Delete the ${code} group` }),
    ).toBeVisible();
  }
  await expect(
    page.getByRole("button", { name: /Delete the (UNIT|TYPE|ABBR) group/ }),
  ).toHaveCount(0);

  // Edit TRAN_STAT first, so restore-vs-undo semantics become observable:
  // the two verbs answer differently AFTER an edit, and only then.
  const file = page.locator("section#file");
  await file
    .getByRole("button", { name: "Edit TRAN_STAT on row 1 of TRAN" })
    .click();
  await page.keyboard.press("Enter");
  await page.keyboard.type("Draft");
  await page.keyboard.press("Enter");

  // Delete the group, keyboard-activated (#529's keyboard-only criterion):
  // the stub stands in the table's place and Rule 14 fires live.
  await file
    .getByRole("button", { name: "Delete the TRAN group" })
    .press("Enter");
  await expect(file.getByText("TRAN deleted")).toBeVisible();
  await expect(file.locator("table")).toHaveCount(0);
  // Rule 14 reads from the panel — the strip belongs to coarse pointers
  // since #591, and a deleted group has no cells to pop from.
  const rule14 = page.locator("#findings li").filter({ hasText: "Rule 14" });
  await expect(rule14.first()).toBeAttached();
  // The stub must hold the page's width discipline while it stands in.
  await expectViewportWide(page);

  // Restore, also from the keyboard: the finding clears and the SEEDED value
  // returns — restore is the seed's rows, not the reader's edits.
  await file.getByRole("button", { name: "Restore TRAN" }).press("Enter");
  await expect(rule14).toHaveCount(0);
  await expect(
    file.getByRole("button", { name: "Edit TRAN_STAT on row 1 of TRAN" }),
  ).toContainText("Final");

  // Undo is the OTHER verb: walk the timeline back through restore and
  // delete, and the pre-delete edit is still there.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(file.getByText("TRAN deleted")).toBeVisible();
  await page.keyboard.press("ControlOrMeta+z");
  await expect(
    file.getByRole("button", { name: "Edit TRAN_STAT on row 1 of TRAN" }),
  ).toContainText("Draft");

  // Redo walks forward over a group op like any other commit.
  await page.keyboard.press("ControlOrMeta+Shift+z");
  await expect(file.getByText("TRAN deleted")).toBeVisible();
});

test("fine: a descent group's stub restores from its own button, and Reset clears a deletion too", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "shares the fine lane with the TRAN loop (#529)");
  await page.goto("/");

  // GroupSection's fallback branch — the TRAN loop above exercises only the
  // cover sheet's. LOCA's table swaps for the stub, in the same section.
  const loca = page.locator("section#loca");
  await loca.getByRole("button", { name: "Delete the LOCA group" }).click();
  await expect(loca.getByText("LOCA deleted")).toBeVisible();
  await expect(loca.locator("table")).toHaveCount(0);
  await loca.getByRole("button", { name: "Restore LOCA" }).click();
  await expect(loca.locator("table")).toHaveCount(1);

  // "Reset the delivery" is the everything-at-once verb: it must clear a
  // deleted-group state exactly like any edit.
  await loca.getByRole("button", { name: "Delete the LOCA group" }).click();
  await expect(loca.getByText("LOCA deleted")).toBeVisible();
  await page.getByRole("button", { name: "Reset the delivery" }).click();
  await expect(loca.locator("table")).toHaveCount(1);
});

test("touch: the delete/restore loop works from a tap, and closes an open carousel", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the coarse-pointer path for #529's controls");
  await page.goto("/");

  // Open the row editor first: deleting the group under it must close it —
  // the pick's group is GONE, and a carousel over a deleted group would be
  // editing a ghost.
  await page
    .getByRole("button", { name: "Edit TRAN_ISNO on row 1 of TRAN" })
    .click();
  const carousel = page.getByRole("group", { name: "Editing row 1 of TRAN" });
  await expect(carousel).toBeVisible();

  const file = page.locator("section#file");
  await file.getByRole("button", { name: "Delete the TRAN group" }).click();
  await expect(carousel).toBeHidden();
  await expect(file.getByText("TRAN deleted")).toBeVisible();
  await expect(
    page.getByRole("list", { name: "TRAN findings" }).getByText("Rule 14"),
  ).toBeVisible();
  await expectViewportWide(page);

  await file.getByRole("button", { name: "Restore TRAN" }).click();
  await expect(file.getByText("TRAN deleted")).toBeHidden();
  await expect(file.locator("table")).toHaveCount(1);
});

test("fine: the fix budget lives on each table — scoped to its group, honest about the rest", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the tooltip half needs a pointer that can hover (#530)");
  await page.goto("/");

  // Select the seeded Rule 8 cell (arming is eager since #531); the
  // counts arrive with the findings.
  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.click();

  // LOCA carries the one seeded engine fix; every other table is a visible,
  // disabled zero. The global button is GONE (#530).
  const locaFix = page.getByRole("button", {
    name: "Fix 1 auto-fixable in LOCA",
  });
  await expect(locaFix).toBeEnabled();
  for (const code of ["PROJ", "SAMP", "LLPL", "TRAN"]) {
    await expect(
      page.getByRole("button", { name: `Fix 0 auto-fixable in ${code}` }),
    ).toBeDisabled();
  }
  await expect(
    page.getByRole("button", { name: "Fix what is safe to fix" }),
  ).toHaveCount(0);

  // The badge grammar, read from the cell popovers (#591): the fixable
  // finding's popover says nothing about "manual"; the fixer-proof Rule 16
  // on SAMP's b cell wears the badge.
  await cell.hover();
  const tooltip = page.getByRole("tooltip");
  await expect(tooltip).toContainText("Rule 8");
  await expect(tooltip).not.toContainText("manual");
  await page
    .getByRole("button", { name: "Edit SAMP_TYPE on row 1 of SAMP" })
    .hover();
  await expect(
    page.getByRole("tooltip").getByText("manual", { exact: true }),
  ).toBeVisible();

  // Apply LOCA's budget: the decimal is repaired, the count hits a disabled
  // zero, and NOTHING outside LOCA moved — asserted on the emitted text the
  // output pane renders, not sampled cells: the before/after diff must be
  // exactly one line, the repaired one.
  const pane = page
    .locator("section#file")
    .locator('div[class*="max-h-"]')
    .first();
  const before = (await pane.innerText()).split("\n");
  await locaFix.click();
  await expect(cell).toContainText("11.80");
  const after = (await pane.innerText()).split("\n");
  expect(after.length).toBe(before.length);
  const changed = before.filter((line, i) => after[i] !== line);
  expect(changed).toHaveLength(1);
  expect(changed[0]).toContain("11.8");
  await expect(
    page.getByRole("button", { name: "Fix 0 auto-fixable in LOCA" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Edit SAMP_TYPE on row 1 of SAMP" }),
  ).toContainText("b");
  await expect(
    page.getByRole("button", { name: "Edit LLPL_LL on row 3 of LLPL" }),
  ).toContainText("38");
  await expect(
    page.locator("section#file li").filter({ hasText: "not defined" }),
  ).toHaveCount(2);

  // The validator-vs-fixer explainer retired at every width in #617
  // (pass-2 pin D2-04) — the orphan's finding wears the manual badge that
  // tells the same story in the panel. Absence asserted in the wide lanes
  // too: they are the ones that used to render it.
  await expect(
    page.getByText("difference between a validator and a fixer"),
  ).toHaveCount(0);

  // A scoped fix is one commit like any other: undo returns the raw decimal.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(cell).not.toContainText("11.80");
});

test("touch: the fix budget applies from a tap too", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the fine lane covers the hover-tooltip half (#530)");
  await page.goto("/");

  // Tap the seeded Rule 8 cell (opens the carousel — fine; arming is
  // eager since #531), then
  // spend LOCA's budget from its table header.
  await page
    .getByRole("button", { name: "Edit LOCA_GL on row 1 of LOCA" })
    .click();
  await page
    .getByRole("button", { name: "Fix 1 auto-fixable in LOCA" })
    .click();
  await expect(
    page.getByRole("button", { name: "Edit LOCA_GL on row 1 of LOCA" }),
  ).toContainText("11.80");
  await expect(
    page.getByRole("button", { name: "Fix 0 auto-fixable in LOCA" }),
  ).toBeDisabled();
});

test("fine: a table's budget reacts to new defects, and the tooltip badges the fixer-proof ones", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "drives the in-place editor and hovers (#530)");
  await page.goto("/");

  // SAMP starts with nothing fixable. A malformed 2DP the engine CAN repair
  // moves its budget 0 -> 1 live — the count derives from the fixes list,
  // recomputed with every revalidation, never a static seed.
  const top = page.getByRole("button", {
    name: "Edit SAMP_TOP on row 1 of SAMP",
  });
  await top.click();
  await expect(
    page.getByRole("button", { name: "Fix 0 auto-fixable in SAMP" }),
  ).toBeDisabled();
  await page.keyboard.press("Enter");
  await page.keyboard.type("1.5");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("button", { name: "Fix 1 auto-fixable in SAMP" }),
  ).toBeEnabled();

  // A value the fixer CANNOT repair is the other half: the finding stays,
  // the budget returns to zero, and the cell's tooltip says so — "manual",
  // in the same grammar as the strip.
  await page.keyboard.press("Enter");
  await page.keyboard.type("abc");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("button", { name: "Fix 0 auto-fixable in SAMP" }),
  ).toBeDisabled();
  await top.hover();
  await expect(page.getByRole("tooltip")).toContainText("manual");
});

test("fine: the budget only offers what lat fix would apply — a risky rewrite never joins it", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "drives the in-place editor (#530)");
  await page.goto("/");

  // An accented remark is an ordinary thing to type into a free-text cell —
  // and the engine CAN repair it, by Rule 1's transliteration, which it
  // classifies risky ("guesses intent") and `lat fix` withholds by default.
  // Before #583 the budget counted it: LOCA read "Fix 2" and the click
  // rewrote this cell to "cafe -- soft ground" — the demo repairing MORE
  // than the CLI it sells. The budget must hold at the one safe seeded fix.
  const rem = page.getByRole("button", {
    name: "Edit LOCA_REM on row 1 of LOCA",
  });
  await rem.click();
  await page.keyboard.press("Enter");
  await page.keyboard.type("café — soft ground");
  await page.keyboard.press("Enter");
  const nonAscii = page
    .locator("section#file li")
    .filter({ hasText: "Non-ASCII" });
  await expect(nonAscii.first()).toBeVisible();
  const locaFix = page.getByRole("button", {
    name: "Fix 1 auto-fixable in LOCA",
  });
  await expect(locaFix).toBeEnabled();

  // Spending the budget repairs the safe Rule 8 decimal and NOTHING else:
  // the remark keeps its accent and its dash, the Rule 1 finding stands —
  // exactly the state `lat fix` would leave the reader's own file in.
  await locaFix.click();
  await expect(
    page.getByRole("button", { name: "Edit LOCA_GL on row 1 of LOCA" }),
  ).toContainText("11.80");
  await expect(rem).toContainText("café — soft ground");
  await expect(nonAscii.first()).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Fix 0 auto-fixable in LOCA" }),
  ).toBeDisabled();
});

test("the engine arrives uninvited — after paint, without a touch", async ({
  page,
}) => {
  // The waiter goes on BEFORE navigation: under a warm cache the eager
  // fetch can start and finish before goto() returns, and a waiter armed
  // after the fact would time out on a request that already happened.
  const sawWasm = page.waitForRequest((r) => r.url().endsWith(".wasm"), {
    timeout: 15_000,
  });
  await page.goto("/");
  // The size-quoting fallback copy died with the lazy gate: sweep the prose
  // BEFORE the findings replace the fallback (where the figure lived), and
  // again after. The pattern also catches "2MB", which \bMB\b would not —
  // no word boundary sits between a digit and a letter.
  const sizeFigure = /megabyte|\d\s*MB\b|\bMB\b/i;
  expect(await page.locator("body").innerText()).not.toMatch(sizeFigure);
  // No interaction at all: the fetch must begin on its own (#531 replaces
  // touch-gated arming with eager-idle), and the findings must follow.
  await sawWasm;
  await expect(page.locator("section#file li").first()).toBeVisible({
    timeout: 15_000,
  });
  expect(await page.locator("body").innerText()).not.toMatch(sizeFigure);
  // "After first paint" is the other half of the criterion, and the browser
  // records both instants: the wasm fetch must not start before first paint.
  const startedAfterPaint = await page.evaluate(() => {
    const paint = performance.getEntriesByName("first-paint")[0];
    const wasm = performance
      .getEntriesByType("resource")
      .find((e) => e.name.endsWith(".wasm"));
    if (!paint || !wasm) return "missing-entries";
    return wasm.startTime >= paint.startTime;
  });
  expect(startedAfterPaint).toBe(true);
});

test("wide: the hero card is the seed's own opening lines, and it verdicts live", async ({
  page,
}) => {
  test.skip(
    width(page) < 1024,
    "the hero card is a wide-viewport affordance — absent below the grid breakpoint",
  );
  await page.goto("/");

  // The drift gate, not discipline (#531): the rendered card must be the
  // committed fixture's opening lines, byte for byte. The old hand-written
  // excerpt showed a corrected decimal the live file deliberately does not
  // carry — the one thing the picture must never do.
  const expected = readFileSync(
    new URL("../landing/demo/seeded-delivery.ags", import.meta.url),
    "utf8",
  )
    .split(/\r?\n/)
    .slice(0, 5);
  const rendered = (await page.locator("section#top pre").innerText()).split(
    "\n",
  );
  expect(rendered).toEqual(expected);

  // The card hydrates: the seeded verdict appears in the hero without any
  // interaction, engine willing.
  const heroChip = page.locator("section#top").getByText(/error/);
  await expect(heroChip).toBeVisible({ timeout: 15_000 });

  // And it is LIVE, not a snapshot: spending LOCA's fix budget below must
  // move the hero's count too — both mounts are one component on one store.
  const before = await heroChip.innerText();
  await page
    .getByRole("button", { name: "Fix 1 auto-fixable in LOCA" })
    .click();
  await expect(heroChip).not.toHaveText(before);
  await expect(heroChip).toContainText(/error/);

  // The LINES hydrate too: a card labelled delivery.ags must show the
  // delivery, not a snapshot of its seed. Edit the project name and the
  // card's PROJ row follows.
  await page
    .getByRole("button", { name: "Edit PROJ_NAME on row 1 of PROJ" })
    .click();
  await page.keyboard.press("Enter");
  await page.keyboard.type("Barrow Lane, Phase 3");
  await page.keyboard.press("Enter");
  await expect(page.locator("section#top pre")).toContainText(
    "Barrow Lane, Phase 3",
  );
});

test("touch: the floating verdict follows without covering the editor", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the 390 touch reader is the floating chip's audience");
  await page.goto("/");

  // Open the carousel on the widest table — chip and editor now share the
  // 390px viewport. The chip must be visible…
  await page
    .getByRole("button", { name: "Edit LOCA_ID on row 1 of LLPL" })
    .click();
  const floating = page.locator('[data-scoreboard="floating"]');
  await expect(floating).toBeVisible({ timeout: 15_000 });

  // …and must NOT sit on the carousel's controls: Playwright refuses a tap
  // that another element would intercept, so paging the carousel IS the
  // occlusion probe.
  const carousel = page.getByRole("group", { name: "Editing row 1 of LLPL" });
  await carousel.scrollIntoViewIfNeeded();
  await carousel
    .getByRole("button", { name: "Next field in this row" })
    .click();
  await expect(carousel.getByText("SAMP_TOP", { exact: true })).toBeVisible();
  await expectViewportWide(page);
});

test("fine: the scoreboard follows the reader and lands on the verdict", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "drives the in-place editor (#531)");
  await page.goto("/");

  // The floating chip appears once a table is on screen, carrying the count.
  await page.locator("section#loca").scrollIntoViewIfNeeded();
  const floating = page.locator('[data-scoreboard="floating"]');
  await expect(floating).toBeVisible({ timeout: 15_000 });
  await expect(floating).toContainText(/error/);

  // Clear every seeded finding: spend LOCA's fix budget (Rule 8), retire the
  // unlisted abbreviation on BOTH rows that carry it (Rule 16 twice — SAMP's
  // key and LLPL's restatement must move together or the chain orphans), and
  // delete the orphaned lab row (Rule 10c).
  await page
    .getByRole("button", { name: "Fix 1 auto-fixable in LOCA" })
    .click();
  for (const name of [
    "Edit SAMP_TYPE on row 1 of SAMP",
    "Edit SAMP_TYPE on row 1 of LLPL",
  ]) {
    await page.getByRole("button", { name }).click();
    await page.keyboard.press("Enter");
    await page.keyboard.type("D");
    await page.keyboard.press("Enter");
  }
  await page.getByRole("button", { name: "Delete row 3 of LLPL" }).click();

  // Zero findings is a stated verdict, not an empty panel.
  await expect(floating).toContainText("valid AGS4");

  // The chip is a door: clicking it lands the reader on the findings panel.
  await floating.click();
  await expect(page.locator("#findings")).toBeInViewport();
  await expectViewportWide(page);
});

test("the page's hierarchy runs demo-first, and the CLI card is the binary", async ({
  page,
}) => {
  await page.goto("/");

  // Nav order matches section order (#533): the demo comes before install on
  // the page, so it comes before install in the nav. toHaveText retries and
  // reads textContent, so the assertion also holds on the lanes where the
  // nav is display:none below the 52rem breakpoint.
  await expect(page.locator("header nav a")).toHaveText([
    "Demo",
    "Install",
    "Docs",
    "Source",
  ]);

  // The hero's filled primary is the demo; install is the outline second.
  const ctas = page.locator("section#top").getByRole("link", {
    name: /See it catch faults|Pick your stack/,
  });
  await expect(ctas.first()).toHaveText("See it catch faults");
  await expect(ctas.first()).toHaveAttribute("href", "#file");
  // Primacy is the FILL, not just the order: the demo CTA wears the rust
  // primary (bg-cta), the install CTA does not.
  await expect(ctas.first()).toHaveClass(/bg-cta/);
  await expect(ctas.nth(1)).toHaveAttribute("href", "#install");
  await expect(ctas.nth(1)).not.toHaveClass(/bg-cta/);

  // The CLI card: the binary is the identity, the releases download is the
  // action, and no pip command appears anywhere on it. CSS locators, not
  // roles: below the grid's column break the deck parks non-current cards
  // invisible (#595, #622), and a parked card has no accessibility tree
  // to query — but its claims must still hold.
  const cli = page
    .locator(".install-card")
    .filter({ has: page.locator("a", { hasText: /^lat$/ }) });
  await expect(cli).toHaveCount(1);
  await expect(
    cli.locator("a").filter({ hasText: /release/i }),
  ).toHaveAttribute("href", "https://github.com/niko86/laterite/releases");
  await expect(cli).not.toContainText("pip install");
  // The note's wording is the GENERATOR's to own — assert the card renders
  // whatever the generated module says, not a second copy of its prose.
  const cliChannel = INSTALL_CHANNELS.find((c) => c.id === "cli");
  await expect(cli).toContainText(cliChannel?.note ?? "unreachable");
});

test("fine: motion rides the tokens — the probed elements transition like the shared Button", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "hover states are the fine pointer's vocabulary (#534)");
  await page.goto("/");
  // Findings must exist before finding rows can be probed.
  await expect(page.locator("section#file li").first()).toBeVisible({
    timeout: 15_000,
  });

  const durationOf = (sel: string) =>
    page
      .locator(sel)
      .first()
      .evaluate((el) => getComputedStyle(el).transitionDuration);

  // The shared Button is the reference implementation of the motion tokens —
  // every bespoke interactive element must compute the same duration it
  // does, and that duration must be a real one.
  const reference = await page
    .getByRole("link", { name: "Open webapp" })
    .evaluate((el) => getComputedStyle(el).transitionDuration);
  expect(reference).not.toBe("0s");

  expect(await durationOf("header nav a"), "nav link").toBe(reference);
  expect(
    await durationOf('section#loca [data-cell="0-0"]'),
    "cell button",
  ).toBe(reference);
  expect(
    await page
      .getByRole("button", { name: "Copy the Python install command" })
      .evaluate((el) => getComputedStyle(el).transitionDuration),
    "copy button",
  ).toBe(reference);
  // The finding row rides the fast opacity tier, not the base one — compare
  // against the token's own computed value rather than a bare "not zero".
  // Both sides normalized to milliseconds: the registered token serializes
  // in seconds while an unregistered one would say "120ms".
  const ms = (v: string) =>
    v.trim().endsWith("ms") ? parseFloat(v) : parseFloat(v) * 1000;
  const fastToken = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--dur-fast"),
  );
  expect(ms(await durationOf("section#file li > *")), "finding row").toBe(
    ms(fastToken),
  );
  expect(await durationOf("footer a"), "footer link").toBe(reference);
});

test("anchor jumps ease under normal motion, and stay instant under reduced", async ({
  page,
}) => {
  // Two halves, deliberately: the computed value is the CONTRACT (frame
  // capture is machine-dependent; `scroll-behavior` is what the one
  // document-level rule promises — smooth when motion is welcome, the
  // browser's instant default when the reader asked for reduced), and the
  // pill click is the JOURNEY — scrollIntoView with no `behavior` key
  // resolves through that computed value, and a broken handler would leave
  // the contract green while the reader goes nowhere (#589).
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.goto("/");
  const behavior = () =>
    page.evaluate(
      () => getComputedStyle(document.documentElement).scrollBehavior,
    );
  expect(await behavior()).toBe("smooth");

  await page
    .getByRole("button", { name: /jump to the findings panel/ })
    .first()
    .click();
  await expect(page.locator("#findings")).toBeInViewport();
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);

  await page.emulateMedia({ reducedMotion: "reduce" });
  expect(await behavior()).toBe("auto");
});

test("no em dash reaches the reader", async ({ page }) => {
  // The em dash is banned from reader-facing landing copy (#587) — every
  // sentence that carried one was rewritten, and this scan is what stops the
  // character creeping back with the next copy edit. Scope, said out loud:
  // the default render on this lane's viewport and pointer — its title and
  // meta description, every visible text node (the engine's own finding text
  // deliberately included), and the strings assistive tech reads from
  // aria-label and title attributes — plus the deleted-group stub, the one
  // conditional surface a single click reaches. Copy that only renders
  // mid-journey (the carousel's type glossary, the all-clear findings
  // state, the engine-loading placeholder) is beyond the DOM's reach here,
  // so the built chunks are scanned as text below — esbuild strips
  // comments, which makes every surviving string literal reader-bound.
  await page.goto("/");
  await expect(page.locator("section#file li").first()).toBeVisible({
    timeout: 15_000,
  });

  const offenders = () =>
    page.evaluate(() => {
      const hits: string[] = [];
      if (document.title.includes("—")) {
        hits.push(`<title>: ${document.title}`);
      }
      const desc =
        document
          .querySelector('meta[name="description"]')
          ?.getAttribute("content") ?? "";
      if (desc.includes("—")) hits.push(`meta description: ${desc}`);
      for (const el of document.querySelectorAll("[aria-label], [title]")) {
        for (const attr of ["aria-label", "title"]) {
          const v = el.getAttribute(attr);
          if (v?.includes("—")) hits.push(`${attr}: ${v}`);
        }
      }
      const walker = document.createTreeWalker(
        document.body,
        NodeFilter.SHOW_TEXT,
      );
      while (walker.nextNode()) {
        const t = walker.currentNode.textContent ?? "";
        if (t.includes("—")) hits.push(t.trim());
      }
      return hits;
    });

  expect(await offenders()).toEqual([]);

  await page.locator("section#llpl").scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Delete the LLPL group" }).click();
  await expect(page.getByText("LLPL deleted")).toBeVisible();
  expect(await offenders()).toEqual([]);

  // The bundle half: every string literal in the built chunks, comment-free,
  // so the states the DOM half cannot reach are still pinned. dist/index.html
  // sits this out — HTML comments survive the build, and that file's rendered
  // surface is already the DOM half's title/meta scan.
  const assets = path.join(
    path.dirname(fileURLToPath(import.meta.url)),
    "../landing/dist/assets",
  );
  const offendingChunks = readdirSync(assets)
    .filter((f) => f.endsWith(".js") || f.endsWith(".css"))
    .filter((f) => readFileSync(path.join(assets, f), "utf8").includes("—"));
  expect(offendingChunks).toEqual([]);
});

test("the file region reads pane, reset, findings — and the cover sheet joins the alternation", async ({
  page,
}) => {
  // #594: the region's pieces sit where they are read; #617 shortened the
  // read — the explainer and the pill retired at every width, so their old
  // wide-lane slots are asserted EMPTY here, in the lanes that rendered
  // them. DOM order IS the contract for what remains: below the grid
  // breakpoint it is literally the reading order, and above it the same
  // order splits into the file column (pane, reset) and the findings
  // column, so one assertion set covers every lane. The pane's landmark is
  // its header line, pinned by tag and exact text so future prose that
  // merely mentions the filename cannot steal the match.
  await page.goto("/");
  const file = page.locator("section#file");
  await expect(file.locator("li").first()).toBeVisible({ timeout: 15_000 });

  await expect(file.getByText("left standing on purpose")).toHaveCount(0);
  await expect(file.getByText("Nothing is uploaded")).toHaveCount(0);

  const pane = file.locator("p", { hasText: /^delivery\.ags$/ }).first();
  const resetBtn = file.getByRole("button", { name: "Reset the delivery" });
  const findingsLabel = file
    .locator("#findings p")
    .filter({ hasText: "Findings" })
    .first();

  const precedes = async (a: Locator, b: Locator, claim: string) => {
    const aH = (await a.elementHandle())!;
    const bH = (await b.elementHandle())!;
    expect(
      await aH.evaluate(
        (x, y) =>
          !!(x.compareDocumentPosition(y) & Node.DOCUMENT_POSITION_FOLLOWING),
        bH,
      ),
      claim,
    ).toBe(true);
  };

  await precedes(pane, resetBtn, "the file pane precedes the reset button");
  await precedes(
    resetBtn,
    findingsLabel,
    "the reset button precedes the findings panel",
  );

  // The cover sheet pairs with its prose in the side-by-side grid the four
  // descent groups set up, continuing their alternation: LLPL led with the
  // prose, so TRAN leads with the table.
  if (width(page) >= 1024) {
    // Measured at the right grains: the table ELEMENT reports its full
    // scrollable width — past its own clip — so the horizontal check reads
    // its scroller, and the prose is read as its whole column (label plus
    // paragraph), not the one-line label, which sits above the table's top
    // where the toolbar row intervenes.
    const proseCol = await file
      .getByText("The cover sheet · TRAN")
      .locator("xpath=..")
      .boundingBox();
    const scroller = await file
      .locator("table")
      .locator("xpath=..")
      .boundingBox();
    expect(proseCol).not.toBeNull();
    expect(scroller).not.toBeNull();
    expect(
      scroller!.x + scroller!.width,
      "table left, prose right",
    ).toBeLessThanOrEqual(proseCol!.x + 1);
    expect(
      Math.max(scroller!.y, proseCol!.y),
      "the two share a row",
    ).toBeLessThan(
      Math.min(scroller!.y + scroller!.height, proseCol!.y + proseCol!.height),
    );
  }
});

test("fine: column widths hold still while a cell is edited", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the carousel is the coarse pointer's editor (#525)");
  // #593: the in-place editor must not re-solve the table under the reader's
  // keystrokes — on the unfixed layout, thirteen characters into SAMP_TYPE
  // grew that column by ~100px and shrank every sibling. Cancel closes the
  // session, so before/during/after all measure the same geometry.
  await page.goto("/");
  const samp = page.locator("section#samp");
  await expect(
    samp.getByRole("button", { name: "Edit SAMP_TYPE on row 1 of SAMP" }),
  ).toBeVisible({ timeout: 15_000 });

  const widths = () =>
    samp
      .locator("th")
      .evaluateAll((ths) => ths.map((t) => t.getBoundingClientRect().width));
  const before = await widths();

  await samp
    .getByRole("button", { name: "Edit SAMP_TYPE on row 1 of SAMP" })
    .click();
  await page.keyboard.type("PISTON SAMPLE");
  await expect(samp.getByRole("textbox")).toHaveValue(/PISTON SAMPLE$/);

  const during = await widths();
  during.forEach((w, i) => {
    expect(Math.abs(w - before[i]!), `column ${i} during typing`).toBeLessThan(
      0.5,
    );
  });

  await page.keyboard.press("Escape");
  await expect(samp.getByRole("textbox")).not.toBeVisible();
  const after = await widths();
  after.forEach((w, i) => {
    expect(Math.abs(w - before[i]!), `column ${i} after cancel`).toBeLessThan(
      0.5,
    );
  });

  // The commit path releases the freeze too — proven with the value it
  // already had, where "identical after" is the contract. (A LONGER commit
  // legitimately re-solves once for the new text; that release is the
  // point of unfreezing rather than a regression.)
  await page.keyboard.press("Enter");
  await expect(samp.getByRole("textbox")).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(samp.getByRole("textbox")).not.toBeVisible();
  const afterCommit = await widths();
  afterCommit.forEach((w, i) => {
    expect(Math.abs(w - before[i]!), `column ${i} after commit`).toBeLessThan(
      0.5,
    );
  });
});

test("the cover sheet carries the full toolbar, and a second TRAN row is a teachable state", async ({
  page,
  hasTouch,
}) => {
  // #593, superseding #527's recorded "+ row is meaningless on a one-row
  // header": the owner's review chose the opposite — the affordance grammar
  // is worth more than the guard, and the engine has its own verdict about
  // an extra transmission row, which is the demo working as intended.
  await page.goto("/");
  const file = page.locator("section#file");
  await expect(file.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });

  await expect(file.getByRole("button", { name: "+ row" })).toBeVisible();
  await expect(file.getByText(/Click a cell|Tap any cell/)).toBeVisible();

  await file.getByRole("button", { name: "+ row" }).click();
  // The verdict lands in the panel on every pointer; the strip is the
  // coarse pointer's surface (#591).
  await expect(
    file.locator("#findings li").filter({ hasText: "TRAN" }).first(),
  ).toBeAttached();
  if (hasTouch) {
    const strip = page.getByRole("list", { name: "TRAN findings" });
    await expect(strip.locator("li").first()).toBeVisible();
    // The strip stays ATTACHED to the table it judges — the relationship
    // the fine-pointer vocabulary test used to pin before #591 moved the
    // strips to this pointer.
    expect(
      await strip.evaluate((el) => !!el.parentElement?.querySelector("table")),
      "the strip must share a column with the table it judges",
    ).toBe(true);
  }
});

test("the delete-group control reads as a button, not a caption", async ({
  page,
}) => {
  // #593: the ghost variant's transparent border is what made "delete group"
  // read as bare text. The control now wears the toolbar button's border,
  // repainted by the danger tone — the shared contract in tokens.ts, which
  // the dark spec asserts under its own token set.
  await page.goto("/");
  await expectErrBorder(page, "Delete the PROJ group");
});

test("fine: the bad abbreviation lights its cell, and fixing the value clears it", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the carousel is the coarse pointer's editor (#525)");
  // #590: Rule 16 arrives group-level — no heading, no row — so no cell used
  // to light up; the reader saw the alert text but not the cell. The mapping
  // lights EVERY cell carrying the named value, one mark per group that
  // reports it.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const samp = page.locator("section#samp");
  const bCell = samp.getByRole("button", {
    name: "Edit SAMP_TYPE on row 1 of SAMP",
  });
  await expect(cornerFlag(bCell)).toBeVisible();
  // The LLPL copy of the same value is its own group's finding, its own mark.
  await expect(
    cornerFlag(
      page
        .locator("section#llpl")
        .getByRole("button", { name: "Edit SAMP_TYPE on row 1 of LLPL" }),
    ),
  ).toBeVisible();
  // D is defined in ABBR: committing it clears SAMP's Rule 16, and the mark
  // goes with it.
  await bCell.click();
  await bCell.click();
  await page.keyboard.type("D");
  await page.keyboard.press("Enter");
  await expect(cornerFlag(bCell)).toHaveCount(0);
});

test("fine: the orphaned row wears the row treatment, and restoring its parent clears it", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the carousel is the coarse pointer's editor (#525)");
  // #590: Rule 10c arrives heading-less but row-pinned — a claim about the
  // whole row, so it reads as one: wash across the cells plus an edge
  // marker, distinct from the cell verdict's text-and-weight treatment.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const llpl = page.locator("section#llpl");
  const td = (cell: string) =>
    llpl.locator(`[data-cell="${cell}"]`).locator("xpath=ancestor::td[1]");
  const shadow = (cell: string) =>
    td(cell).evaluate((el) => getComputedStyle(el).boxShadow);
  const bg = (cell: string) =>
    td(cell).evaluate((el) => getComputedStyle(el).backgroundColor);

  expect(await shadow("2-0"), "the orphan row's edge marker").not.toBe("none");
  expect(await shadow("1-0"), "a healthy row has none").toBe("none");
  expect(
    await bg("2-1"),
    "the wash replaces the KEY tint on the condemned row",
  ).not.toBe(await bg("1-1"));

  // Restore the SAMP parent the orphan names: BH02|4.50|S3|D|BH02-S3.
  const samp = page.locator("section#samp");
  const edit = async (name: string, value: string) => {
    const cell = samp.getByRole("button", { name });
    await cell.click();
    await cell.click();
    await page.keyboard.type(value);
    await page.keyboard.press("Enter");
    // Serialize the commits — the next edit must not race this one's close.
    await expect(samp.getByRole("textbox")).not.toBeVisible();
  };
  await edit("Edit SAMP_TOP on row 3 of SAMP", "4.50");
  await edit("Edit SAMP_REF on row 3 of SAMP", "S3");
  await edit("Edit SAMP_ID on row 3 of SAMP", "BH02-S3");
  await expect(async () => {
    expect(await shadow("2-0")).toBe("none");
  }).toPass();
});

test("the KEY region tint is structural, not a verdict", async ({ page }) => {
  // #590, the owner's recorded choice on the issue: stone, one value across
  // every table. The band-coloured region sat on the red-brown ramp, so on
  // SAMP and LLPL whole columns read as failed; identity stays on the group
  // chip and the table's cap. Two tables on different bands rendering the
  // SAME header tint is what "structural" means.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const headerBg = (section: string) =>
    page
      .locator(`section#${section} th`)
      .nth(1)
      .evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(await headerBg("samp")).toBe(await headerBg("llpl"));
});

test("fine: finding text surfaces at the cell — popover on hover and focus, Escape dismisses, strips gone", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the popover is fine-pointer vocabulary (#591)");
  // #591: the rule text lives ON the failing cell, not in callout strips
  // under the table. Hover, select and keyboard focus each summon it;
  // Escape and leaving dismiss it; the panel stays the one complete list.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });

  const cell = page.getByRole("button", {
    name: "Edit LOCA_GL on row 1 of LOCA",
  });
  await cell.hover();
  const pop = page.getByRole("tooltip");
  await expect(pop).toBeVisible();
  await expect(pop).toContainText("Rule 8");
  await page.locator("section#loca h2").hover();
  await expect(pop).not.toBeVisible();

  await cell.click();
  await expect(pop).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(pop).not.toBeVisible();

  // KEYBOARD arrival is its own path, not click's twin: select the left
  // neighbour, then ArrowRight moves selection AND focus onto the failing
  // cell — the popover follows focus. Arrowing on again is the blur, and
  // the popover leaves with it.
  await page
    .getByRole("button", { name: "Edit LOCA_TYPE on row 1 of LOCA" })
    .click();
  await page.keyboard.press("ArrowRight");
  await expect(pop).toBeVisible();
  await expect(pop).toContainText("Rule 8");
  await page.keyboard.press("ArrowRight");
  await expect(pop).not.toBeVisible();

  // A row-level finding pops from any cell of the condemned row — this cell
  // carries no cell finding of its own.
  await page.locator('section#llpl [data-cell="2-1"]').hover();
  await expect(pop).toBeVisible();
  await expect(pop).toContainText("Rule 10c");

  // The strips are gone at this pointer; the panel remains complete.
  await expect(
    page.getByRole("list", { name: /(PROJ|LOCA|SAMP|LLPL|TRAN) findings/ }),
  ).toHaveCount(0);
  await expect(
    page.locator("#findings li").filter({ hasText: "not defined" }),
  ).toHaveCount(2);
});

test("the findings panel is one card below the breakpoint, a stack at desktop", async ({
  page,
}) => {
  // #592: at 390px the four stacked callouts cost most of a screen; the
  // panel becomes a single card paged with wrap. Width, not pointer — the
  // stack's cost is vertical space, which a fine-pointered narrow window
  // pays too.
  await page.goto("/");
  const panel = page.locator("#findings");
  const cards = panel.locator("ul li");
  await expect(cards.first()).toBeVisible({ timeout: 15_000 });
  await expect(cards).toHaveCount(4);

  if (width(page) >= 1024) {
    // Desktop unchanged: every card visible, no paging chrome.
    await expect(cards.nth(3)).toBeVisible();
    await expect(
      panel.getByRole("button", { name: "Next finding" }),
    ).toHaveCount(0);
    return;
  }

  // One visible card; the rest are parked in the DOM, not gone — absence
  // assertions elsewhere keep their strength.
  await expect(cards.nth(1)).toBeHidden();
  await expect(panel.getByText("1 / 4")).toBeVisible();

  // Infinite wrap, both directions: four nexts come home, and prev from
  // the first card lands on the last.
  const next = panel.getByRole("button", { name: "Next finding" });
  await next.click();
  await expect(panel.getByText("2 / 4")).toBeVisible();
  await next.click();
  await next.click();
  await next.click();
  await expect(panel.getByText("1 / 4")).toBeVisible();
  await panel.getByRole("button", { name: "Previous finding" }).click();
  await expect(panel.getByText("4 / 4")).toBeVisible();

  // A leftward drag is the same verb as Next — the swipe the issue names.
  const list = panel.getByRole("list", { name: "Findings" });
  await list.dispatchEvent("pointerdown", { clientX: 300, pointerId: 1 });
  await list.dispatchEvent("pointerup", { clientX: 180, pointerId: 1 });
  await expect(panel.getByText("1 / 4")).toBeVisible();

  // A REAL mouse drag that starts on the card's own button must page — and
  // ONLY page: a mouse fires `click` after any down/up pair, so without the
  // swallow the swipe would also focus the card's file line.
  const ringed = page.locator(
    "section#file .overscroll-contain [class*='--focus-ring']",
  );
  const box = await panel.locator("ul li:visible button").first().boundingBox();
  if (!box) throw new Error("the visible card has no button to drag");
  const y = box.y + box.height / 2;
  await page.mouse.move(box.x + box.width - 10, y);
  await page.mouse.down();
  await page.mouse.move(box.x + 10, y, { steps: 5 });
  await page.mouse.up();
  await expect(panel.getByText("2 / 4")).toBeVisible();
  await expect(ringed).toHaveCount(0);

  // The click verb itself still works at this width — the positive control
  // that makes the no-focus assertion above falsifiable.
  await panel.locator("ul li:visible button").first().click();
  await expect(ringed).toHaveCount(1);

  // RowCarousel's keyboard idiom answers here too: Alt+Arrow pages.
  await page.keyboard.press("Alt+ArrowRight");
  await expect(panel.getByText("3 / 4")).toBeVisible();
});

test("touch: under-table findings page one at a time, and one finding gets no affordance", async ({
  page,
  hasTouch,
}) => {
  test.skip(!hasTouch, "the strip is the coarse pointer's surface (#591)");
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });

  // LLPL carries two group-level findings (Rule 16 + the 10c orphan): a
  // carousel, one card at a time, wrapping.
  const llpl = page.locator("section#llpl");
  const strip = llpl.getByRole("list", { name: "LLPL findings" });
  await expect(strip.locator("li")).toHaveCount(2);
  await expect(strip.locator("li").nth(1)).toBeHidden();
  await expect(llpl.getByText("1 / 2")).toBeVisible();
  await llpl.getByRole("button", { name: "Next finding" }).click();
  await expect(llpl.getByText("2 / 2")).toBeVisible();
  await llpl.getByRole("button", { name: "Next finding" }).click();
  await expect(llpl.getByText("1 / 2")).toBeVisible();

  // SAMP has exactly one: the card stands alone, no counter, no buttons —
  // paging chrome on a single card is an affordance with nothing to afford.
  const samp = page.locator("section#samp");
  await expect(
    samp.getByRole("list", { name: "SAMP findings" }).locator("li"),
  ).toHaveCount(1);
  await expect(samp.getByRole("button", { name: "Next finding" })).toHaveCount(
    0,
  );
});

test("reduced motion swaps cards instantly", async ({ page }) => {
  test.skip(
    width(page) >= 1024,
    "the carousel is the narrow presentation (#592)",
  );
  // #592: the fade rides motion-safe, so a reduced-motion reader gets the
  // swap with no transition at all — asserted from the computed style, and
  // falsifiable: the no-preference half must show a real duration.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  const panel = page.locator("#findings");
  await expect(panel.locator("li").first()).toBeVisible({ timeout: 15_000 });
  // The ACTIVE card carries the transition (#622 moved it off the parked
  // cards, whose pose must snap), so the card being swapped is the one
  // the claim reads.
  expect(
    await panel
      .locator("li:visible")
      .evaluate((el) => getComputedStyle(el).transitionDuration),
  ).toBe("0s");
  await panel.getByRole("button", { name: "Next finding" }).click();
  await expect(panel.getByText("2 / 4")).toBeVisible();

  await page.emulateMedia({ reducedMotion: "no-preference" });
  expect(
    await panel
      .locator("li:visible")
      .evaluate((el) => getComputedStyle(el).transitionDuration),
  ).not.toBe("0s");
});

test("every install card wears its surface hue", async ({ page }) => {
  // #595: the border is the hue the generator emitted, the fill is that hue
  // washed into the surface — asserted per card against installChannels.ts
  // itself, so a card and its data cannot disagree. Computed styles resolve
  // on hidden elements too, so this holds on the phone deck's parked cards.
  await page.goto("/");
  const cards = page.locator(".install-card");
  await expect(cards).toHaveCount(INSTALL_CHANNELS.length);
  for (const [i, channel] of INSTALL_CHANNELS.entries()) {
    expect(
      await cards.nth(i).evaluate((el) => getComputedStyle(el).borderTopColor),
      `${channel.id} border`,
    ).toBe(hexToRgb(channel.hue.light));
  }
  // The wash is a real fill, not transparency over the page — and per-card,
  // so two cards never share a dress.
  const bg = (i: number) =>
    cards.nth(i).evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(await bg(0)).not.toBe("rgba(0, 0, 0, 0)");
  expect(await bg(0)).not.toBe(await bg(1));
});

test("every install card opens a door to its surface's get-started page", async ({
  page,
}) => {
  // #619 (pass-2 pin D2-11): a reader sold by a card gets a path to "how do
  // I start with THIS surface". The hrefs come from the generated data the
  // page itself renders, so the five decided targets are pinned on the
  // Python side (test_install_channels.py) and the DOM side here cannot
  // disagree with them. On a phone the deck parks four cards invisible, so
  // the visible-and-tappable half reads the ACTIVE card, then pages once —
  // reachability on the deck is the AC, not mere presence in the DOM.
  await page.goto("/");
  const install = page.locator("section#install");

  if (width(page) >= 608) {
    // By INDEX like the hue test above, not by label text: a hasText filter
    // reads the card's whole subtree, and the CLI card's note names Python
    // (and Python's note names duckdb), so label filtering resolves two
    // cards and trips strict mode. The grid renders INSTALL_CHANNELS in
    // order; nth is the established pin for that.
    const cards = install.locator(".install-card");
    for (const [i, channel] of INSTALL_CHANNELS.entries()) {
      const link = cards.nth(i).getByRole("link", { name: "Get started" });
      await expect(link, `${channel.id} card's door`).toBeVisible();
      await expect(link).toHaveAttribute("href", channel.docs);
    }
    return;
  }

  const activeDoor = () =>
    install
      .locator("ul li:visible .install-card")
      .getByRole("link", { name: "Get started" });
  await expect(activeDoor()).toBeVisible();
  await expect(activeDoor()).toHaveAttribute("href", INSTALL_CHANNELS[0].docs);
  await install.getByRole("button", { name: "Go to card 2" }).click();
  await expect(activeDoor()).toHaveAttribute("href", INSTALL_CHANNELS[1].docs);
});

test("the install cards become a one-card looping deck on a phone", async ({
  page,
}) => {
  // #595: below the grid's own first column break (38rem) the five cards
  // page one at a time with position dots; at 38rem and up the grid stands.
  await page.goto("/");
  const install = page.locator("section#install");
  const lis = install.locator("ul li");
  await expect(lis).toHaveCount(INSTALL_CHANNELS.length);

  if (width(page) >= 608) {
    // Desktop unchanged in structure: the grid, every card visible, no
    // paging chrome.
    await expect(lis.nth(4)).toBeVisible();
    await expect(
      install.getByRole("button", { name: "Next card" }),
    ).toHaveCount(0);
    await expect(
      install.getByRole("button", { name: /Go to card/ }),
    ).toHaveCount(0);
    return;
  }

  // One visible card — the rest parked, not gone — plus five dots.
  const current = install.locator("ul li:visible");
  await expect(current).toHaveCount(1);
  await expect(current).toContainText("Python");
  const dots = install.getByRole("button", { name: /Go to card/ });
  await expect(dots).toHaveCount(5);

  // Infinite wrap, both directions.
  const next = install.getByRole("button", { name: "Next card" });
  await next.click();
  await expect(current).toContainText("Node.js");
  await next.click();
  await next.click();
  await next.click();
  await next.click();
  await expect(current).toContainText("Python");
  await install.getByRole("button", { name: "Previous card" }).click();
  await expect(current).toContainText("Browser");

  // A dot is a door, and the current one says so.
  await install.getByRole("button", { name: "Go to card 3" }).click();
  await expect(current).toContainText("CLI");
  await expect(
    install.getByRole("button", { name: "Go to card 3" }),
  ).toHaveAttribute("aria-current", "true");

  // Swipe pages, like the findings carousels.
  const deck = install.getByRole("list", { name: "Install channels" });
  await deck.dispatchEvent("pointerdown", { clientX: 300, pointerId: 1 });
  await deck.dispatchEvent("pointerup", { clientX: 180, pointerId: 1 });
  await expect(current).toContainText("DuckDB");

  // Reduced motion swaps instantly — no transition on the cards at all.
  await page.emulateMedia({ reducedMotion: "reduce" });
  expect(
    await current.evaluate((el) => getComputedStyle(el).transitionDuration),
  ).toBe("0s");
  await next.click();
  await expect(current).toContainText("Browser");
});

test("a deck holds one height while paging, and its chrome sits centred", async ({
  page,
  hasTouch,
}) => {
  // #622 (M2-03..M2-05): display parking sized each deck to whichever card
  // was showing, so paging pumped the page below it — and the chrome row
  // hugged the left edge under a centred card. Grid-stack parking reserves
  // the tallest card's height (the ONE thing paging may change is which
  // card shows), and both chromes centre under their cards.
  test.skip(width(page) >= 608, "both decks are grids at this width");
  await page.goto("/");

  const heightOf = (l: Locator) =>
    l.evaluate((el) => el.getBoundingClientRect().height);
  // The chrome row spans the column even unfixed, so the falsifiable claim
  // is the content's symmetry INSIDE it: the paging buttons' two insets.
  const insetSkew = async (prev: Locator, next: Locator) => {
    const r = await prev.locator("..").boundingBox();
    const p = await prev.boundingBox();
    const n = await next.boundingBox();
    if (!r || !p || !n) throw new Error("the chrome must lay out");
    return Math.abs(p.x - r.x - (r.x + r.width - (n.x + n.width)));
  };

  const install = page.locator("section#install");
  const deck = install.getByRole("list", { name: "Install channels" });
  await expect(deck.locator("li:visible")).toHaveCount(1);
  const deckHeight = await heightOf(deck);
  const nextCard = install.getByRole("button", { name: "Next card" });
  for (const channel of INSTALL_CHANNELS.slice(1)) {
    await nextCard.click();
    await expect(deck.locator("li:visible")).toContainText(channel.label);
    expect(await heightOf(deck), `the deck on ${channel.id}`).toBeCloseTo(
      deckHeight,
      0,
    );
  }
  expect(
    await insetSkew(
      install.getByRole("button", { name: "Previous card" }),
      nextCard,
    ),
    "the dots chrome sits centred",
  ).toBeLessThanOrEqual(1);

  // Visibility parking must hold BOTH halves display parking held: a
  // parked card sits outside the accessibility tree and cannot take
  // focus. The active card is the positive control that keeps the focus
  // probe falsifiable.
  const listitems = (await deck.ariaSnapshot()).match(/- listitem/g);
  expect(listitems, "one card in the accessibility tree").toHaveLength(1);
  const takesFocus = (l: Locator) =>
    l.evaluate((el) => {
      el.focus();
      return document.activeElement === el;
    });
  expect(await takesFocus(deck.locator("li:visible a").first())).toBe(true);
  expect(
    await takesFocus(deck.locator("li").first().locator("a").first()),
  ).toBe(false);

  const panel = page.locator("#findings");
  await expect(panel.locator("li").first()).toBeVisible({ timeout: 15_000 });
  const findings = panel.getByRole("list", { name: "Findings" });
  const findingsHeight = await heightOf(findings);
  const nextFinding = panel.getByRole("button", { name: "Next finding" });
  for (const n of [2, 3, 4]) {
    await nextFinding.click();
    await expect(panel.getByText(`${n} / 4`)).toBeVisible();
    expect(await heightOf(findings), `the findings deck on ${n}`).toBeCloseTo(
      findingsHeight,
      0,
    );
  }
  expect(
    await insetSkew(
      panel.getByRole("button", { name: "Previous finding" }),
      nextFinding,
    ),
    "the counter chrome sits centred",
  ).toBeLessThanOrEqual(1);

  if (hasTouch) {
    // The bare-deck half: a single card reserves nothing extra — SAMP's
    // one finding IS its deck's whole height.
    const samp = page.getByRole("list", { name: "SAMP findings" });
    expect(await heightOf(samp)).toBeCloseTo(
      await heightOf(samp.locator("li")),
      0,
    );
  }
});

test("a page turn enters from the side of travel, even on a reversal", async ({
  page,
}) => {
  // #622's parking rewrite almost lost #534's slide grammar: the parked
  // nudge is rendered state and a transition starts from the last
  // COMPUTED style, so a direction flip landing in the same recalc as the
  // turn would slide the entering card in from the PREVIOUS turn's side.
  // The component turns a reversal in two renders instead; this pins it.
  // The probe stretches the transition so the read lands at its start,
  // then reads the entering card's translate in flight — the SIGN is the
  // claim, not the magnitude.
  test.skip(width(page) >= 608, "the deck is a grid at this width");
  await page.goto("/");
  const install = page.locator("section#install");
  const next = install.getByRole("button", { name: "Next card" });
  await next.click();
  await next.click();
  const deck = install.getByRole("list", { name: "Install channels" });
  await expect(deck.locator("li:visible")).toContainText("CLI");
  const pose = await install.evaluate(async (section) => {
    const style = document.createElement("style");
    style.textContent = "section#install ul li { --dur-fast: 60s; }";
    document.head.append(style);
    const prev = [...section.querySelectorAll("button")].find(
      (b) => b.getAttribute("aria-label") === "Previous card",
    );
    if (!prev) throw new Error("no Previous button to reverse with");
    prev.click();
    // A reversal turns one frame after the pose renders — wait past it,
    // then catch the stretched transition just after it starts.
    await new Promise((r) =>
      requestAnimationFrame(() =>
        requestAnimationFrame(() => requestAnimationFrame(r)),
      ),
    );
    const li = [...section.querySelectorAll("ul li")].find(
      (el) => getComputedStyle(el).visibility === "visible",
    );
    if (!li) throw new Error("no visible card mid-turn");
    const translate = getComputedStyle(li).translate;
    style.remove();
    return translate;
  });
  // Previous enters from the LEFT: a negative x nudge easing to rest.
  expect(parseFloat(pose), pose).toBeLessThan(-5);
});

test("the file pane wraps on a phone and side-scrolls on desktop", async ({
  page,
}) => {
  // #596: findings cite line numbers, so hidden content is worse than
  // wrapped content — below the breakpoint every character is visible,
  // wrapped with a hanging indent under the number; desktop keeps the
  // side-scroll that preserves column alignment.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const scroller = page.locator("section#file .overscroll-contain");
  const overflowsX = () =>
    scroller.evaluate((el) => el.scrollWidth > el.clientWidth + 1);

  if (width(page) >= 1024) {
    expect(await overflowsX(), "desktop keeps the side-scroll").toBe(true);
    return;
  }

  expect(await overflowsX(), "no sideways scroll on a phone").toBe(false);
  // A long DATA line really wraps — its row stands taller than a single
  // line — and the severity band spans the whole logical line: the banded
  // row's paint is as tall as the row, wrapped or not.
  const rows = scroller.locator("div.flex");
  const heights = await rows.evaluateAll((els) =>
    els.map((el) => el.getBoundingClientRect().height),
  );
  const single = Math.min(...heights.filter((h) => h > 0));
  expect(Math.max(...heights), "some line must wrap").toBeGreaterThan(
    single * 1.5,
  );
  // The hanging indent is geometry, not inference: every row's content span
  // starts at the same x — one content column — and that column sits past
  // the number gutter, so continuation lines can only land under content.
  const contentX = await scroller
    .locator("div.flex > span:nth-child(2)")
    .evaluateAll((els) => els.map((el) => el.getBoundingClientRect().x));
  expect(new Set(contentX).size, "one content column").toBe(1);
  const gutterRight = await scroller
    .locator("div.flex > span:first-child")
    .first()
    .evaluate((el) => el.getBoundingClientRect().right);
  expect(contentX[0], "content clears the number gutter").toBeGreaterThan(
    gutterRight,
  );
  // AC: the severity band spans the full logical line when wrapped — the
  // banded Rule 8 row is itself one of the wrapped ones, and its border is
  // the row box's own left edge, so tall row = tall band.
  const banded = scroller.locator("div.flex[class*='border-l-err']").first();
  expect(
    await banded.evaluate((el) => el.getBoundingClientRect().height),
    "the banded row wraps and its band spans it",
  ).toBeGreaterThan(single * 1.5);
  // One row per LOGICAL line, counted from the same fixture the page bakes
  // in (the seededFinalDepthLabel convention, #524) — editing the seed moves
  // both sides with no code edit here.
  const seedLines = readFileSync(
    path.join(
      path.dirname(fileURLToPath(import.meta.url)),
      "../landing/demo/seeded-delivery.ags",
    ),
    "utf8",
  ).split("\r\n").length;
  expect(heights.length, "one row per logical line").toBe(seedLines);
  // Line numbers stay one per LOGICAL line: the numbers run 1..N with no
  // repeats, so "line 17" in a finding still means the 17th row here.
  const numbers = await scroller
    .locator("div.flex > span:first-child")
    .allTextContents();
  expect(numbers.map(Number)).toEqual(numbers.map((_, i) => i + 1));
});

test("the section rhythm steps down on a phone", async ({ page }) => {
  // #596: the desktop rhythm at 390px ran the page ~17 screens tall. The
  // step-down is the dial the owner reviews; what this pins is that a phone
  // pays LESS than the old desktop rhythm and desktop pays what it did.
  await page.goto("/");
  const pad = await page
    .locator("section#loca > div")
    .evaluate((el) => parseFloat(getComputedStyle(el).paddingTop));
  if (width(page) >= 1088) {
    expect(pad, "the 68rem rhythm holds").toBe(64);
  } else if (width(page) < 640) {
    expect(pad, "a phone pays less than the old py-12").toBeLessThan(48);
    expect(pad, "but not drastically less").toBeGreaterThanOrEqual(24);
  }
});

test("the mobile masthead carries source and install icons; the desktop nav stands", async ({
  page,
}) => {
  // #597: the text nav hides below 52rem, which left a phone no path to the
  // source or the install anchor from the top bar. Two icon links fill it —
  // tap-target floor 44px, labelled — while the CTA keeps its words (#586).
  await page.goto("/");
  const header = page.locator("header");
  const source = header.getByRole("link", { name: "Source on GitHub" });
  const install = header.getByRole("link", { name: "Jump to install" });

  if (width(page) >= 832) {
    // Desktop unchanged: the text nav carries these destinations; the icon
    // forms are not in the way.
    await expect(source).toBeHidden();
    await expect(install).toBeHidden();
    await expect(header.locator("nav a")).toHaveText([
      "Demo",
      "Install",
      "Docs",
      "Source",
    ]);
    return;
  }

  await expect(source).toBeVisible();
  await expect(source).toHaveAttribute(
    "href",
    "https://github.com/niko86/laterite",
  );
  for (const link of [source, install]) {
    const box = await link.boundingBox();
    expect(box, "the icon link must lay out").not.toBeNull();
    expect(box?.width, "tap-target width floor").toBeGreaterThanOrEqual(44);
    expect(box?.height, "tap-target height floor").toBeGreaterThanOrEqual(44);
  }
  // The CTA stays words, not a glyph (#586).
  await expect(header.getByRole("link", { name: "Open webapp" })).toBeVisible();
  // The masthead still fits the phone with both icons added.
  await expectViewportWide(page);

  // Light ink resolves and the two glyphs share it — the dark spec holds
  // the same claim under its token set.
  const ink = (l: Locator) => l.evaluate((el) => getComputedStyle(el).color);
  expect(await ink(source)).not.toBe("rgba(0, 0, 0, 0)");
  expect(await ink(install)).toBe(await ink(source));

  // #621 (superseding #597's bare-icon ruling): both links wear the theme
  // toggle's box. Matching the toggle's own computed border and radius —
  // never a pinned colour or radius value — IS the claim: one control
  // family, so a retuned token moves all three together unnoticed. Only
  // the 1px width is pinned, and that pin says the border is real, not
  // which token draws it.
  const toggle = header.getByRole("button", { name: "Toggle colour theme" });
  const boxOf = (l: Locator) =>
    l.evaluate((el) => {
      const s = getComputedStyle(el);
      return {
        width: s.borderTopWidth,
        color: s.borderTopColor,
        radius: s.borderRadius,
      };
    });
  const toggleBox = await boxOf(toggle);
  expect(toggleBox.width, "the family's border is real").toBe("1px");
  for (const link of [source, install]) {
    expect(await boxOf(link)).toEqual(toggleBox);
  }

  // #621's other half: the cluster's two gaps are equal edge-to-edge. The
  // boxes make the flex gap the VISIBLE gap — between borders, not glyph
  // whitespace — so measuring bounding-box edges now measures what a
  // reader sees.
  const sBox = await source.boundingBox();
  const iBox = await install.boundingBox();
  const tBox = await toggle.boundingBox();
  if (!sBox || !iBox || !tBox)
    throw new Error("masthead controls must lay out");
  const gapOne = iBox.x - (sBox.x + sBox.width);
  const gapTwo = tBox.x - (iBox.x + iBox.width);
  expect(Math.abs(gapOne - gapTwo), "one consistent gap").toBeLessThanOrEqual(
    0.5,
  );

  // The focus state is visible, not just declared: keyboard focus must
  // paint the ring. Tab from the top of the document — logo first, then
  // the source icon.
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  await expect(source).toBeFocused();
  expect(
    await source.evaluate((el) => getComputedStyle(el).boxShadow),
    "keyboard focus paints the ring",
  ).not.toBe("none");

  // The install glyph is a working door: the jump lands the install section
  // in view, riding the document's own anchor behaviour (#589).
  await install.click();
  await expect(page.locator("section#install")).toBeInViewport();
});

test("wide: the rail weighs its bands and its labels are doors", async ({
  page,
}) => {
  test.skip(
    width(page) < 1088,
    "the depth scale only renders above the rail's 68rem collapse breakpoint",
  );
  // #585, reversing #524's recorded equal-bands choice: each band's height
  // is its section's measured share of the page, and the depth-scale labels
  // are real links.
  await page.goto("/");
  // The fullest layout first — the bands weigh RENDERED heights, and the
  // findings are the demo's tallest content.
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });

  // The strip's bands are nowhere near equal on the real page — equality is
  // the regression this pins. The strip is the veil's parent; the veil is
  // the one child that is not a band.
  const strip = page.locator("div.border-t-steel-500").locator("xpath=..");
  const bandHeights = await strip.evaluate((el) =>
    Array.from(el.children)
      .filter((c) => !c.className.includes("border-t-steel-500"))
      .map((c) => c.getBoundingClientRect().height),
  );
  expect(bandHeights).toHaveLength(7);
  expect(
    new Set(bandHeights.map((h) => Math.round(h))).size,
    "weighted bands cannot all round to one height",
  ).toBeGreaterThan(1);

  // The keyboard door first, from the top of a fresh document: the rail
  // renders first, so its links are the page's first tab stops — four Tabs
  // land on SAMP with real keyboard provenance, which is what lets
  // :focus-visible paint the ring (a scripted focus() would not). This half
  // runs before any click because a fragment navigation moves the
  // sequential-focus starting point to its target, and Tab would continue
  // from inside the landed section.
  const samp = page.getByRole("link", { name: "Jump to SAMP" });
  for (let i = 0; i < 4; i++) await page.keyboard.press("Tab");
  await expect(samp).toBeFocused();
  expect(
    await samp.evaluate((el) => getComputedStyle(el).boxShadow),
    "keyboard focus paints the ring",
  ).not.toBe("none");
  await page.keyboard.press("Enter");
  await expect(page.locator("section#samp")).toBeInViewport();

  // And the pointer door.
  await page.getByRole("link", { name: "Jump to LOCA" }).click();
  await expect(page.locator("section#loca")).toBeInViewport();

  // The reduced-motion half of the AC, on a rail label specifically: the
  // jump rides the document's one scroll-behavior rule (#589), so under
  // reduce it resolves instant — and still arrives.
  await page.emulateMedia({ reducedMotion: "reduce" });
  expect(
    await page.evaluate(
      () => getComputedStyle(document.documentElement).scrollBehavior,
    ),
  ).toBe("auto");
  await page.getByRole("link", { name: "Jump to LLPL" }).click();
  await expect(page.locator("section#llpl")).toBeInViewport();

  // The ornaments stay ornaments: the pill is aria-hidden, the labels are
  // the rail's only citizens of the accessibility tree.
  await expect(page.locator(".rounded-pill")).toHaveAttribute(
    "aria-hidden",
    "true",
  );
});

test("phone: the narrow strip inherits the datum mapping", async ({ page }) => {
  test.skip(
    width(page) >= 1088,
    "the narrow strip is the rail's below-68rem form",
  );
  // #615's one narrow-width AC: the 8px strip inherits the mapping and
  // nothing else. There are no doors down here (the depth scale never
  // renders), so the pin is the veil: scrolled to SAMP's landing line, its
  // top must sit at railY of SAMP's tick fraction — the datum-keyed value,
  // re-derived here from the sections' own rendered heights. The retired
  // document-fraction mapping puts a visibly different number at the same
  // scroll, so this goes red on the old wiring, narrow or not.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const ids = SECTIONS.map((s) => s.id);
  await page.evaluate(() => {
    const el = document.getElementById("samp");
    if (!el) return;
    const offset = parseFloat(getComputedStyle(el).scrollMarginTop) || 0;
    window.scrollTo(
      0,
      el.getBoundingClientRect().top + window.scrollY - offset,
    );
  });
  await page.waitForFunction(() => {
    const el = document.getElementById("samp");
    if (!el) return false;
    const offset = parseFloat(getComputedStyle(el).scrollMarginTop) || 0;
    const target = el.getBoundingClientRect().top + window.scrollY - offset;
    return Math.abs(window.scrollY - target) <= 1;
  });
  const expected = await page.evaluate(
    ([sectionIds, insetPct]) => {
      const heights = (sectionIds as string[]).map(
        (id) => document.getElementById(id)?.offsetHeight ?? 0,
      );
      const total = heights.reduce((a, b) => a + b, 0);
      const before = heights
        .slice(0, (sectionIds as string[]).indexOf("samp"))
        .reduce((a, b) => a + b, 0);
      return (before / total) * (100 - (insetPct as number));
    },
    [ids, RAIL_INSET_PCT] as const,
  );
  const veil = page.locator("div.border-t-steel-500");
  const probeTop = parseFloat(await veil.evaluate((el) => el.style.top));
  expect(
    probeTop,
    "the strip's veil must sit at SAMP's tick fraction",
  ).toBeCloseTo(expected, 1);
});

test("wide: a rail jump lands the pill exactly on its tick", async ({
  page,
}) => {
  test.skip(
    width(page) < 1088,
    "the depth scale only renders above the rail's 68rem collapse breakpoint",
  );
  // #615, retiring the document-fraction probe: the probe now reads the
  // section under the DATUM line, so a jump's landing IS the tick's own
  // fraction. Measured on prod before the fix, "Jump to SAMP" read 8.23 m
  // against a 6.86 m tick. Assertions run only once the scroll has SETTLED
  // on the landing line: the old mapping glides THROUGH the correct number
  // on its way past it, and a mid-glide read would go green on the defect.
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });
  const pill = page.locator(".rounded-pill");

  const settled = (id: string) =>
    page.waitForFunction((sel) => {
      const el = document.getElementById(sel);
      if (!el) return false;
      const offset = parseFloat(getComputedStyle(el).scrollMarginTop) || 0;
      const max = document.documentElement.scrollHeight - window.innerHeight;
      const target = Math.max(
        0,
        Math.min(el.getBoundingClientRect().top + window.scrollY - offset, max),
      );
      return Math.abs(window.scrollY - target) <= 1;
    }, id);

  // A CSS attribute selector, not getByRole: role-name matching is
  // case-insensitive, and the masthead carries its own lowercase "Jump to
  // install" door that would straddle the rail's "Jump to Install".
  const jumpAndRead = async (label: string, id: string) => {
    const door = page.locator(`a[aria-label="Jump to ${label}"]`);
    const tick = (
      await door.locator("xpath=preceding-sibling::p").innerText()
    ).replace(/ m$/, "");
    await door.click();
    await settled(id);
    await expect(pill, `${label}'s landing must read its tick`).toHaveText(
      tick,
    );
  };

  // Every door on the scale, in descent order, from the page's own sequence —
  // Surface included: its landing clamps at the very top, where the datum
  // construction pins 0.00 exactly.
  for (const section of SECTIONS) {
    await jumpAndRead(section.label, section.id);
  }

  // The floor: max scroll reads the seeded final depth — the stretched
  // tail's whole purpose, since the deepest landing line sits well above
  // the page bottom.
  await page.evaluate(() =>
    window.scrollTo(0, document.documentElement.scrollHeight),
  );
  // scrollTo rides the document's smooth behavior too, so wait for the
  // glide to reach the floor before reading.
  await page.waitForFunction(() => {
    const max = document.documentElement.scrollHeight - window.innerHeight;
    return Math.abs(window.scrollY - max) <= 1;
  });
  await expect(pill).toHaveText(seededFinalDepthLabel());

  // Reduced motion: the jump resolves instant (#589) and lands identically.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await jumpAndRead("SAMP", "samp");
});

test("fine: a paste whose target moved is abandoned, not re-aimed", async ({
  page,
  hasTouch,
}) => {
  test.skip(hasTouch, "the selected-cell clipboard is fine-pointer (#551)");
  // #580: the pick is captured at keydown but the clipboard resolves later,
  // and between the two the reader can delete the row the pick names. The
  // recorded decision: such a paste is ABANDONED — dropStalePick's grounds,
  // never silently edit data the reader did not choose. The race is made
  // deterministic by holding the clipboard promise open until the row is
  // gone.
  await page.addInitScript(() => {
    let pending: ((v: string) => void) | null = null;
    (window as unknown as Record<string, unknown>).__resolvePaste = (
      v: string,
    ) => {
      pending?.(v);
      pending = null;
    };
    navigator.clipboard.readText = () =>
      new Promise<string>((resolve) => {
        pending = resolve;
      });
  });
  await page.goto("/");
  await expect(page.locator("#findings li").first()).toBeVisible({
    timeout: 15_000,
  });

  // Resolving flushes a macrotask turn too, so the handler's .then has run
  // by the time this returns — the SNEAK assertions below are about a
  // completed paste path, not a race against it.
  const resolvePaste = (v: string) =>
    page.evaluate(async (value) => {
      (
        window as unknown as { __resolvePaste: (v: string) => void }
      ).__resolvePaste(value);
      await new Promise((r) => setTimeout(r, 0));
    }, v);

  // The surviving-pick window, where the defect actually bites: edit a cell,
  // select it, start a paste, then RESET while the clipboard is pending. The
  // pick survives (same position, still in range) but the row under it is
  // the seed's again — data the reader never chose to paste into.
  const proj = page.locator("section#proj");
  const cell = proj.getByRole("button", {
    name: "Edit PROJ_ID on row 1 of PROJ",
  });
  await cell.click();
  await page.keyboard.type("XYZ");
  await page.keyboard.press("Enter");
  await expect(cell).toContainText("XYZ");
  // Enter commits AND keeps the selection (#593's spreadsheet grammar), so
  // the cell is already picked — a second click would OPEN the editor and
  // hand the paste to the input's native clipboard instead.
  await page.keyboard.press("ControlOrMeta+v");
  await page.getByRole("button", { name: "Reset the delivery" }).click();
  await expect(cell).not.toContainText("XYZ");
  await resolvePaste("SNEAK");
  await expect(page.locator("main")).not.toContainText("SNEAK");
  // No phantom commit either: one undo returns the pre-reset edit.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(cell).toContainText("XYZ");

  // And the closed-pick window: the row the pick named is deleted outright
  // while the clipboard hangs. Benign-looking today, but the decision says
  // abandoned, and this holds it so.
  const rows = proj.locator("tbody tr");
  await proj.getByRole("button", { name: "+ row" }).click();
  await expect(rows).toHaveCount(2);
  await proj
    .getByRole("button", { name: "Edit PROJ_ID on row 2 of PROJ" })
    .click();
  await page.keyboard.press("ControlOrMeta+v");
  await proj.getByRole("button", { name: "Delete row 2 of PROJ" }).click();
  await expect(rows).toHaveCount(1);
  await resolvePaste("SNEAK");
  await expect(page.locator("main")).not.toContainText("SNEAK");
  // One undo brings the deleted row back — no phantom step in between.
  await page.keyboard.press("ControlOrMeta+z");
  await expect(rows).toHaveCount(2);
});
