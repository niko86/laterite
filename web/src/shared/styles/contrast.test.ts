import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { INSTALL_CHANNELS } from "../../../landing/installChannels";

// The contrast gate over the shared token layer (#403).
//
// The acceptance bar for the app's move onto these tokens was "contrast is
// verified across the app in both themes, not sampled on one screen" — and a
// one-off sample is exactly what rots: the next value retune re-fails nothing.
// So the verification lives here, against the palette each surface actually
// paints — the shared layer plus whatever that surface retunes on top of it
// (#452) — computed per WCAG 2.x relative luminance.
//
// Thresholds are the DESIGN's, not one blanket number, and they live in the
// assertions below and nowhere else:
//   - WCAG AA normal-text for the three body steps of the fg ramp, and for
//     status/accent text on the grounds it actually sets on.
//   - Two lower floors for fg-faint and fg-dim — hints, taglines, line
//     numbers: deliberately below body text, floored only so a retune
//     cannot make them vanish outright.
//   - AA normal-text for the rust CTA pair as of #682. It sat at the
//     UI-component floor until then, on the reasoning that a mid-ramp band
//     cannot reach the text bar against either extreme; that stopped being
//     true once rust was retired as a TEXT colour, which freed the fill to
//     move down a band. The boundary half of that pair is still 3.0, because
//     an edge answers to 1.4.11 rather than 1.4.3.
//
// ## What this gate cannot see, and where that has bitten
//
// It compares resolved TOKEN PAIRS. It has no DOM, no stacking context and no
// compositing, so:
//
//   - **Opacity modifiers are invisible to it.** A `text-cta/70` or an
//     `opacity-70` renders a colour that is in no token, against a background
//     it never computes. Both instances #682 found were under the bar while
//     this file was green — one in the SQL console's keyboard hint, one on the
//     finding callout's line reference. Neither was a token defect and neither
//     could have been caught here. Resolving them needs a real browser, which
//     is what the Lighthouse pass over the built landing is for.
//   - **It does not know which pairs actually RENDER.** Every assertion below
//     is a claim that some element sets that foreground on that ground. When a
//     role is retired the assertion has to go with it, or the gate starts
//     defending a pairing nothing paints — see the CTA block below, where
//     `cta-quiet` left the boundary list for exactly that reason.

const read = (file: string): string =>
  readFileSync(resolve(import.meta.dirname, file), "utf8");

const css = read("colors.css");
const landing = read("../../../landing/landing.css");

function block(css: string, selector: RegExp): Record<string, string> {
  const body = css.match(selector)?.[1];
  if (body === undefined) throw new Error(`selector not found: ${selector}`);
  const out: Record<string, string> = {};
  // Comments in these files legitimately QUOTE declarations; parse only code.
  const code = body.replace(/\/\*[\s\S]*?\*\//g, "");
  for (const declaration of code.matchAll(/--([\w-]+):\s*([^;]+);/g)) {
    out[declaration[1] as string] = (declaration[2] as string).trim();
  }
  return out;
}

const ROOT = /:root\s*\{([\s\S]*?)\n\}/;
const DARK = /\.dark\s*\{([\s\S]*?)\n\}/;
// A light-only retune has to say so in its SELECTOR — colors.css's dark block
// explains why a bare `:root` would beat the shared dark set on source order.
const LIGHT_ONLY = /:root:not\(\.dark\)\s*\{([\s\S]*?)\n\}/;

const light = block(css, ROOT);
const dark = { ...light, ...block(css, DARK) };

// The RENDERED themes, not the authored ones (#452).
//
// colors.css names `--canvas` as the one token a surface is expected to retune,
// and the landing page takes that offer. So the light set above is not what
// laterite.dev paints, and every threshold below was being asserted about a
// palette one shipped surface does not use. A surface joins this map by having
// its stylesheet parsed here, and then costs nothing further: the cases are all
// `describe.each` over it already.
//
// The overlay order is the cascade's, not a convenience: the landing's blocks
// are imported AFTER colors.css, so its bare `:root` outranks the shared `.dark`
// (the two tie on specificity), and its `:root:not(.dark)` outranks its own
// `:root` on specificity.
const landingRoot = block(landing, ROOT);
const themes = {
  light,
  dark,
  "landing light": { ...light, ...landingRoot, ...block(landing, LIGHT_ONLY) },
  "landing dark": { ...dark, ...landingRoot, ...block(landing, DARK) },
} as const;

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

  // The rust CTA (#682). This pair sat at the 3.0 UI-component floor because a
  // mid-ramp band could not reach the normal-text bar against either extreme,
  // and the light theme accepted that trade at #406. Lighthouse reported it on
  // every run, correctly: text on a fill is text, and 1.4.3 wants 4.5.
  //
  // What broke the deadlock was not a better rust. `--fg-on-cta` had no
  // headroom left — it is already all but white — so the FILL had to darken,
  // and darkening it made `cta`-as-TEXT worse, which is why the floor existed.
  // Retiring `cta` as a text colour (the `action` variant now takes the accent
  // family, as `outline` already did) removed that second constraint and freed
  // the fill to darken.
  //
  // It did NOT free it onto the ramp. The band below collides with `--warn`
  // under simulated colour-blindness, which separation.test.ts beside this file
  // catches and this one cannot — the two gates have to be read together, and a
  // `--cta` that satisfies only this one is not shippable. So the value is a
  // literal off the ramp, and the sweep that found it is described where it
  // lives, in colors.css.
  it("holds the rust CTA pair to AA now that rust is a fill and not a text", () => {
    expect(ratio(tokens, "fg-on-cta", "cta")).toBeGreaterThanOrEqual(4.5);
    expect(ratio(tokens, "fg-on-cta", "cta-hover")).toBeGreaterThanOrEqual(4.5);
    // The fill's own EDGE against the page it sits on is 1.4.11, not 1.4.3 —
    // a boundary, not a text. `cta-quiet` has left this list because nothing
    // renders rust against it any more; the wash keeps its own family's text.
    for (const bg of ["canvas", "surface"]) {
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
/** The hex a `theme-color` meta carries FOR one scheme.
 *
 *  Bound to the scheme rather than merely present in the file (#452): a mirror
 *  that holds both hexes and swaps which is which contains every value it is
 *  supposed to, and paints a light reader's browser chrome in the dark canvas.
 *  Both shells write `media` ahead of `content`, which is what makes the pair
 *  readable in a single match. */
const themeColorFor = (html: string, scheme: string): string | undefined =>
  html.match(
    new RegExp(
      `prefers-color-scheme:\\s*${scheme}\\)"[\\s\\S]{0,120}?content="(#[0-9a-f]{6})"`,
      "i",
    ),
  )?.[1];

describe("the chrome mirrors of --canvas", () => {
  it("theme-color metas carry both themes' canvas", () => {
    const html = read("../../../index.html");
    expect(themeColorFor(html, "light")).toBe(resolveVar(light, "canvas"));
    expect(themeColorFor(html, "dark")).toBe(resolveVar(dark, "canvas"));
  });

  // The landing carries its own pair, and they have to mirror the RETUNED
  // canvas rather than the shared one — the same blind spot as the thresholds
  // above, on the one value #452 is about moving.
  it("the landing's theme-color metas carry the landing's canvas", () => {
    const html = read("../../../landing/index.html");
    for (const scheme of ["light", "dark"] as const) {
      expect(themeColorFor(html, scheme), `${scheme} meta`).toBe(
        resolveVar(themes[`landing ${scheme}`], "canvas"),
      );
    }
  });

  it("manifest splash carries the dark canvas", () => {
    const config = read("../../../vite.config.ts");
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

// The install cards' hue dress (#595). The five hues are generated DATA
// (installChannels.ts), not tokens, so the describe.each above never sees
// them — without this block the issue's "contrast gates green" criterion
// would be true only vacuously. The wash is recomputed here exactly as
// landing.css mixes it (srgb, the theme's --install-wash-pct into
// --surface), and each text role the card sets on that wash is held to the
// same bar the role answers to on the named surfaces.
describe("the install cards' hue dress (#595)", () => {
  const srgbMix = (a: string, pct: number, b: string): string => {
    const at = (h: string, o: number) => parseInt(h.slice(o, o + 2), 16);
    const to2 = (n: number) => Math.round(n).toString(16).padStart(2, "0");
    const m = (o: number) => at(a, o) * pct + at(b, o) * (1 - pct);
    return `#${to2(m(1))}${to2(m(3))}${to2(m(5))}`;
  };
  const against = (
    tokens: Record<string, string>,
    fg: string,
    bgHex: string,
  ): number => {
    const l1 = luminance(resolveVar(tokens, fg));
    const l2 = luminance(bgHex);
    const [hi, lo] = l1 > l2 ? [l1, l2] : [l2, l1];
    return (hi + 0.05) / (lo + 0.05);
  };

  const cases: [string, Record<string, string>, "light" | "dark"][] = [
    ["landing light", themes["landing light"], "light"],
    ["landing dark", themes["landing dark"], "dark"],
  ];

  for (const [name, tokens, side] of cases) {
    it(`${name}: every card's text roles hold their bars on its wash`, () => {
      const pct = parseFloat(tokens["install-wash-pct"] ?? "") / 100;
      expect(pct, "the wash dial must parse").toBeGreaterThan(0);
      const surface = resolveVar(tokens, "surface");
      for (const channel of INSTALL_CHANNELS) {
        const wash = srgbMix(channel.hue[side], pct, surface);
        for (const fg of ["fg", "fg-muted", "accent"]) {
          expect(
            against(tokens, fg, wash),
            `${fg} on the ${channel.id} wash`,
          ).toBeGreaterThanOrEqual(4.5);
        }
        expect(
          against(tokens, "fg-faint", wash),
          `fg-faint on the ${channel.id} wash`,
        ).toBeGreaterThanOrEqual(3.0);
        // The border IS the card's identity, so it gets the UI-component
        // floor against the wash it outlines.
        const border = luminance(channel.hue[side]);
        const ground = luminance(wash);
        const [hi, lo] = border > ground ? [border, ground] : [ground, border];
        expect(
          (hi + 0.05) / (lo + 0.05),
          `the ${channel.id} border on its wash`,
        ).toBeGreaterThanOrEqual(3.0);
      }
    });
  }
});
