import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// The contrast gate over the shared token layer (#403).
//
// The acceptance bar for the app's move onto these tokens was "contrast is
// verified across the app in both themes, not sampled on one screen" — and a
// one-off sample is exactly what rots: the next value retune re-fails nothing.
// So the verification lives here, against the same colors.css every surface
// renders, computed per WCAG 2.x relative luminance.
//
// Thresholds are the DESIGN's, not one blanket number, and they live in the
// assertions below and nowhere else:
//   - WCAG AA normal-text for the three body steps of the fg ramp, and for
//     status/accent text on the grounds it actually sets on.
//   - Two lower floors for fg-faint and fg-dim — hints, taglines, line
//     numbers: deliberately below body text, floored only so a retune
//     cannot make them vanish outright.
//   - The UI-component floor for the rust CTA pair: mid-ramp bands cannot
//     reach the normal-text bar against either extreme (the reason
//     --fg-on-cta is a token at all), and the light theme accepted that
//     trade at #406. The gate catches the failure dark actually shipped
//     with — a wash the dark pass never dialled down (fixed in #403).

const css = readFileSync(resolve(import.meta.dirname, "colors.css"), "utf8");

function block(selector: RegExp): Record<string, string> {
  const body = css.match(selector)?.[1];
  if (body === undefined) throw new Error(`selector not found: ${selector}`);
  const out: Record<string, string> = {};
  // Comments in this file legitimately QUOTE declarations; parse only code.
  const code = body.replace(/\/\*[\s\S]*?\*\//g, "");
  for (const declaration of code.matchAll(/--([\w-]+):\s*([^;]+);/g)) {
    out[declaration[1] as string] = (declaration[2] as string).trim();
  }
  return out;
}

const light = block(/:root\s*\{([\s\S]*?)\n\}/);
const dark = { ...light, ...block(/\.dark\s*\{([\s\S]*?)\n\}/) };
const themes = { light, dark } as const;

function resolveVar(tokens: Record<string, string>, name: string): string {
  let value = tokens[name];
  if (value === undefined) throw new Error(`undefined token --${name}`);
  for (;;) {
    const ref: string | undefined = value.match(/^var\(--([\w-]+)\)$/)?.[1];
    if (ref === undefined) break;
    const next: string | undefined = tokens[ref];
    if (next === undefined) throw new Error(`undefined token --${ref}`);
    value = next;
  }
  if (!/^#[0-9a-f]{6}$/i.test(value)) {
    throw new Error(`--${name} did not resolve to a hex colour: ${value}`);
  }
  return value;
}

function luminance(hex: string): number {
  const channel = (offset: number) => {
    const c = parseInt(hex.slice(offset, offset + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
}

function ratio(tokens: Record<string, string>, fg: string, bg: string): number {
  const l1 = luminance(resolveVar(tokens, fg));
  const l2 = luminance(resolveVar(tokens, bg));
  const [hi, lo] = l1 > l2 ? [l1, l2] : [l2, l1];
  return (hi + 0.05) / (lo + 0.05);
}

const SURFACES = [
  "canvas",
  "surface",
  "surface-raised",
  "surface-code",
  "chip",
];

describe.each(Object.entries(themes))("%s theme", (_name, tokens) => {
  it("holds the three body steps of the fg ramp to AA on every surface", () => {
    for (const fg of ["fg", "fg-soft", "fg-muted"]) {
      for (const bg of SURFACES) {
        expect(ratio(tokens, fg, bg), `${fg} on ${bg}`).toBeGreaterThanOrEqual(
          4.5,
        );
      }
    }
  });

  it("keeps the two quiet fg steps legible for what they set", () => {
    for (const bg of SURFACES) {
      expect(
        ratio(tokens, "fg-faint", bg),
        `fg-faint on ${bg}`,
      ).toBeGreaterThanOrEqual(3.0);
      expect(
        ratio(tokens, "fg-dim", bg),
        `fg-dim on ${bg}`,
      ).toBeGreaterThanOrEqual(2.0);
    }
  });

  it("holds status and accent text to AA on its grounds and its own wash", () => {
    for (const fg of ["accent", "ok", "warn", "err", "info"]) {
      for (const bg of ["canvas", "surface", `${fg}-quiet`]) {
        expect(ratio(tokens, fg, bg), `${fg} on ${bg}`).toBeGreaterThanOrEqual(
          4.5,
        );
      }
    }
  });

  // The solid status fills (#404): the Chip's solid form and the findings'
  // char-level hit marks set `text-surface` on bg-ok/warn/err/info/accent —
  // the surface token flips with the theme, so the pairing has to hold AA in
  // both directions (light text on dark fills in light, dark text on the
  // lightened fills in dark).
  it("holds the on-fill text to AA on the solid status and accent fills", () => {
    for (const bg of ["accent", "ok", "warn", "err", "info"]) {
      expect(
        ratio(tokens, "surface", bg),
        `surface on ${bg}`,
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("holds the rust CTA pair to the UI-component floor", () => {
    expect(ratio(tokens, "fg-on-cta", "cta")).toBeGreaterThanOrEqual(3.0);
    expect(ratio(tokens, "fg-on-cta", "cta-hover")).toBeGreaterThanOrEqual(3.0);
    for (const bg of ["canvas", "surface", "cta-quiet"]) {
      expect(ratio(tokens, "cta", bg), `cta on ${bg}`).toBeGreaterThanOrEqual(
        3.0,
      );
    }
  });
});

// The landing's depth pill rides the borehole rail on one `text-surface`
// across both themes, because its FILL inverts with them: accent in light,
// laterite-300 in dark. The light half falls under the solid-fills case above;
// laterite-300 is a brand-ramp step rather than a status token, so the dark
// half is asserted here and nowhere else. Dark ONLY, deliberately — in light
// `--surface` is near-white and would sit on that same sand at no contrast at
// all, which is precisely why the pill does not use that combination there.
it("holds the depth pill's on-fill text to AA where its dark fill applies", () => {
  expect(ratio(dark, "surface", "laterite-300")).toBeGreaterThanOrEqual(4.5);
});

// The theme-color metas and the PWA manifest splash are read by the browser
// before any CSS loads, so they carry mirrored hexes of --canvas rather than
// a var() — the one duplication the platform forces. Mirrors drift; this
// holds them to the tokens they claim to mirror.
describe("the chrome mirrors of --canvas", () => {
  it("theme-color metas carry both themes' canvas", () => {
    const html = readFileSync(
      resolve(import.meta.dirname, "../../../index.html"),
      "utf8",
    );
    expect(html).toContain(`content="${resolveVar(light, "canvas")}"`);
    expect(html).toContain(`content="${resolveVar(dark, "canvas")}"`);
  });

  it("manifest splash carries the dark canvas", () => {
    const config = readFileSync(
      resolve(import.meta.dirname, "../../../vite.config.ts"),
      "utf8",
    );
    const canvas = resolveVar(dark, "canvas");
    expect(config).toContain(`theme_color: "${canvas}"`);
    expect(config).toContain(`background_color: "${canvas}"`);
  });
});

// "The rust CTA does NOT change. It is the one object that is the same colour
// in both themes." — colors.css. Said in values there; enforced here, because
// the first dark set broke it by inheritance without failing anything: the
// ramp shifted under var(--laterite-500) and the CTA rode it to burnt orange.
it("keeps the CTA the same colour in both themes", () => {
  for (const token of ["cta", "cta-hover", "fg-on-cta"]) {
    expect(resolveVar(dark, token)).toBe(resolveVar(light, token));
  }
});
