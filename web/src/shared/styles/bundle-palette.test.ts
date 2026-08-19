/** No raw Tailwind palette utility may reach a shipped bundle (#437).
 *
 * `design/no-raw-palette` bans these classes in source, and cannot see the way
 * they actually arrive. Tailwind scans files as RAW TEXT and cannot tell code
 * from comment, so a comment that NAMES a complete utility emits it. The gate
 * that bans the class was itself shipping two of them: eslint.config.js cites
 * `bg-emerald-600` and `text-sky-50` as examples of what it forbids. A comment
 * in Masthead.tsx explaining why `bg-stone-50` was the WRONG class emitted
 * Tailwind's cool stone into both bundles.
 *
 * The eslint rule cannot ever catch this — it visits `Literal` and
 * `TemplateElement` nodes, and a comment is structurally invisible to both.
 * So this asserts the OUTCOME instead of the source: compile each entry the
 * way the Vite plugin does, and read what came out. That covers every route in
 * — comment, test file, config, or a class genuinely written in a component —
 * rather than the one route we happened to notice.
 *
 * The family list is read out of tailwindcss's own shipped theme, matching the
 * eslint rule, so the vendor adding a colour is covered without an edit here.
 * Our own ramp is `laterite-*`, which is not one of its families.
 *
 * WHAT THIS DOES NOT COVER: the sibling gates. `design/no-raw-effects` bans
 * `cursor-not-allowed` and its family, and eslint.config.js was emitting that
 * one too — fixed here by excluding the file, but NOT asserted, because those
 * patterns live inside the rule's closure and duplicating them here would make
 * a second place to remember. Covering them wants the patterns exported from
 * eslint.config.js first; until then a raw EFFECT reaching the bundle from a
 * comment in a rendering file is unguarded.
 */
import { readFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";

import { compile } from "@tailwindcss/node";
import { Scanner } from "@tailwindcss/oxide";
import { describe, expect, it } from "vitest";

const WEB = resolve(import.meta.dirname, "../../..");
const SHARED = resolve(WEB, "src/shared");

/** Tailwind's own colour families, read from the vendor file (as eslint does). */
const FAMILIES = [
  ...new Set(
    [
      ...readFileSync(
        resolve(WEB, "node_modules/tailwindcss/theme.css"),
        "utf8",
      ).matchAll(/--color-([a-z]+)-\d+/g),
    ].map((m) => m[1]),
  ),
];

/** A utility whose name carries a vendor family + step, in SELECTOR position.
 *  Anchored on the leading `.` so our own `--stone-200` custom properties —
 *  same names, different job — are not mistaken for utilities. */
const RAW_UTILITY = new RegExp(
  `\\.[a-z-]*\\b(?:${FAMILIES.join("|")})-\\d{2,3}\\b`,
  "g",
);

/** The two shipped entries, each with the Vite root its build runs from. */
const BUNDLES = [
  { entry: "src/app.css", viteRoot: WEB },
  { entry: "landing/landing.css", viteRoot: resolve(WEB, "landing") },
];

async function buildBundle(entry: string, viteRoot: string) {
  const file = resolve(WEB, entry);
  const compiler = await compile(readFileSync(file, "utf8"), {
    base: dirname(file),
    onDependency() {},
    // The `@shared` alias is Vite's; the compiler resolves CSS on its own.
    customCssResolver: (id: string) =>
      Promise.resolve(
        id.startsWith("@shared/")
          ? resolve(SHARED, id.slice("@shared/".length))
          : undefined,
      ),
  });
  // The plugin's own composition: the auto-detected root folds in AHEAD of the
  // `@source` list, which is why an entry with explicit sources still scans
  // everything under its Vite root.
  const rootSources =
    compiler.root === "none"
      ? []
      : compiler.root === null
        ? [{ base: viteRoot, pattern: "**/*", negated: false }]
        : [{ ...compiler.root, negated: false }];
  const scanner = new Scanner({
    sources: [...rootSources, ...compiler.sources],
  });
  const css = compiler.build(scanner.scan());
  return { css, scanned: scanner.scannedFiles.map((f) => relative(WEB, f)) };
}

describe.each(BUNDLES)("$entry", ({ entry, viteRoot }) => {
  it("actually compiles and scans something", async () => {
    // The positive control. Everything below asserts an ABSENCE, and an
    // absence is what a broken pipeline also produces: resolve the entry
    // wrongly, or lose the root, and the scan reads nothing, emits nothing and
    // passes. Both halves have to be observed working first.
    const { css, scanned } = await buildBundle(entry, viteRoot);
    expect(scanned.length, `${entry} scanned no files`).toBeGreaterThan(20);
    expect(css, `${entry} compiled to nothing`).toContain("--tw-");
  }, 60_000);

  // `dark:` keys off the theme CLASS, not the OS (#452).
  //
  // Tailwind's default dark variant is `prefers-color-scheme`, and every entry
  // has to override it, because the theme here is `.dark` on <html> from the
  // persisted-else-system choice — a reader whose OS and choice disagree is
  // exactly the case the class exists to serve. `src/app.css` declared the
  // override from the start; the landing's entry never did, so on that bundle
  // the dark utilities fired from the OS instead: a dark-machine reader viewing
  // the page in light got them with the LIGHT token values, which painted
  // GroupTable's card in `--surface-raised` — the landing's own canvas, so the
  // demo table had no fill. It looked right only to readers whose OS agreed.
  //
  // Per-entry, because that is the only way it can go wrong: the variant is
  // declared in the entry, and a new entry starts with Tailwind's default.
  it("keys its dark utilities off the theme class, not the OS", async () => {
    const { css } = await buildBundle(entry, viteRoot);
    expect(css, `${entry} emits no dark utility at all`).toContain(".dark\\:");
    expect(
      css.includes("prefers-color-scheme"),
      `${entry} compiles its dark variant to a media query — declare \`@custom-variant dark (&:where(.dark, .dark *));\` in the entry, as src/app.css does`,
    ).toBe(false);
  }, 60_000);

  it("emits no raw Tailwind palette utility", async () => {
    const { css } = await buildBundle(entry, viteRoot);
    const emitted = [
      ...new Set([...css.matchAll(RAW_UTILITY)].map((m) => m[0])),
    ];
    expect(
      emitted,
      `${entry} ships raw palette utilities. Something the scan reads NAMES them — most likely a comment, or a file that discusses classes without rendering any, since design/no-raw-palette keeps them out of component code. Exclude the file with \`@source not\`, or reword the prose to name the effect rather than the class.`,
    ).toEqual([]);
  }, 60_000);
});
