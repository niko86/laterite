/* The landing's icon contract (#598).
 *
 * The defect this pins: index.html declared /icons/icon-128.png while the
 * landing build shipped no public/ directory, so the favicon — and every
 * blind /favicon.ico probe — 404ed on prod. Vite copies public/ into dist/
 * verbatim, so "the href resolves inside public/" IS "the href resolves in
 * the deploy"; checking public/ keeps this buildless.
 *
 * The icons are deliberately the APP's icons, byte for byte: one brand mark,
 * two surfaces. Copies drift silently, so the identity is asserted rather
 * than trusted. favicon.ico is generated (tools/gen_landing_favicon.py) and
 * committed; here it just has to exist and actually be an ICO.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const LANDING = import.meta.dirname;
const PUBLIC = join(LANDING, "public");
const APP_ICONS = join(LANDING, "..", "public", "icons");

const html = readFileSync(join(LANDING, "index.html"), "utf8");
const iconHrefs = [
  ...html.matchAll(/rel="(?:icon|apple-touch-icon)"[^>]*href="([^"]+)"/gs),
]
  .map((m) => m[1])
  .filter((href): href is string => href !== undefined);

describe("declared icon links", () => {
  it("finds the icon links in index.html", () => {
    expect(iconHrefs).toContain("/icons/icon-128.png");
    expect(iconHrefs).toContain("/icons/icon-256.png");
    expect(iconHrefs).toContain("/icons/apple-touch-icon.png");
  });

  it("every declared href is a file this build ships", () => {
    for (const href of iconHrefs) {
      expect(() => readFileSync(join(PUBLIC, href))).not.toThrow();
    }
  });
});

describe("shipped icon files", () => {
  it("are byte-identical to the app's icon set", () => {
    for (const name of [
      "icon-128.png",
      "icon-256.png",
      "apple-touch-icon.png",
    ]) {
      const landing = readFileSync(join(PUBLIC, "icons", name));
      const app = readFileSync(join(APP_ICONS, name));
      expect(landing.equals(app), `${name} drifted from web/public/icons`).toBe(
        true,
      );
    }
  });

  it("serves a real ICO at /favicon.ico for blind probes", () => {
    const ico = readFileSync(join(PUBLIC, "favicon.ico"));
    // ICONDIR header: reserved 0, type 1 (icon).
    expect([...ico.subarray(0, 4)]).toEqual([0, 0, 1, 0]);
  });
});
