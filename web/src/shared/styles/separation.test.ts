import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

// The SEPARATION gate over the shared token layer (#434).
//
// contrast.test.ts beside this file asks whether a colour is legible ON a
// ground. It never asks whether two colours are legible AGAINST EACH OTHER,
// and that is a different question with different failures. Every defect this
// file was written for passed the contrast gate:
//
//   - `--warn` and `--accent` in dark were ΔE 1.1 apart under deuteranopia. A
//     link and a warning were the same colour, and nothing failed.
//   - `--info` and `--ok` were 1.2 apart under protanopia, because both were
//     low-chroma and a cone deficiency removes the only channel separating
//     them.
//   - `--chip` sat ΔL 0.002 from `--surface-raised`.
//   - The chart palette was cut from the brand ramp, which is a SEQUENTIAL
//     scale, so its series collapsed into each other on the dark surface.
//
// The measure throughout is ΔE: Euclidean distance in OKLab ×100, evaluated
// under normal vision and under protanopia and deuteranopia simulated with
// Machado–Oliveira–Fernandes 2009 at severity 1.0. The simulation model is part
// of the standard, not an implementation detail — thresholds are calibrated to
// it. Hue angle is NOT the measure and must not be reintroduced: it scored the
// olive/ochre pair as 29° apart and safe while the two were ΔE 1.1 and
// indistinguishable.
//
// Thresholds live in the assertions below and nowhere else.

const read = (file: string): string =>
  readFileSync(resolve(import.meta.dirname, file), "utf8");

function block(css: string, selector: RegExp): Record<string, string> {
  const body = css.match(selector)?.[1];
  if (body === undefined) throw new Error(`selector not found: ${selector}`);
  const out: Record<string, string> = {};
  // Comments here legitimately quote declarations; parse only code.
  const code = body.replace(/\/\*[\s\S]*?\*\//g, "");
  for (const declaration of code.matchAll(/--([\w-]+):\s*([^;]+);/g)) {
    out[declaration[1] as string] = (declaration[2] as string).trim();
  }
  return out;
}

const colors = read("./colors.css");
const charts = read("./charts.css");
const landing = read("../../../landing/landing.css");
const ROOT = /:root\s*\{([\s\S]*?)\n\}/;
const DARK = /\.dark\s*\{([\s\S]*?)\n\}/;
// A light-only retune has to say so in its SELECTOR — colors.css's dark block
// explains why a bare `:root` would beat the shared dark set on source order.
const LIGHT_ONLY = /:root:not\(\.dark\)\s*\{([\s\S]*?)\n\}/;

const light = { ...block(colors, ROOT), ...block(charts, ROOT) };
const dark = {
  ...light,
  ...block(colors, DARK),
  ...block(charts, DARK),
};
type Tokens = Record<string, string>;

// The RENDERED themes, not the authored ones (#452).
//
// colors.css names `--canvas` as the one token a surface is expected to retune,
// and the landing page takes that offer — so the light set above is not what
// laterite.dev paints, and none of the rules below had ever been evaluated
// against the palette it does. The elevation ladder is what that hid: the
// landing's canvas and `--surface-raised` were one value, so a panel drawn
// `border-line bg-surface-raised` had no fill of its own at all.
//
// The overlay order is the cascade's, not a convenience: the landing's blocks
// are imported AFTER colors.css, so its bare `:root` outranks the shared `.dark`
// (the two tie on specificity), and its `:root:not(.dark)` outranks its own
// `:root` on specificity.
//
// `mode` rather than the set's name is what the two calibrated tables below key
// on — the chart lightness band and the accepted-pair ratchet are properties of
// the theme a surface renders IN, not of the surface.
//
// `forgone` is the other thing a retune can produce: a rung the surface no
// longer HAS. The landing's canvas lands on `--surface-raised`'s own value, so
// in light the two are one colour and "raised" is not a step that exists there.
// Dropping it from the ladder is only honest because "the landing never fills
// with it in light" is itself asserted — see the elevation section.
type Mode = "light" | "dark";
type Theme = { mode: Mode; tokens: Tokens; forgone: readonly string[] };
const landingRoot = block(landing, ROOT);
const themes: Record<string, Theme> = {
  light: { mode: "light", tokens: light, forgone: [] },
  dark: { mode: "dark", tokens: dark, forgone: [] },
  "landing light": {
    mode: "light",
    tokens: { ...light, ...landingRoot, ...block(landing, LIGHT_ONLY) },
    forgone: ["surface-raised"],
  },
  "landing dark": {
    mode: "dark",
    tokens: { ...dark, ...landingRoot, ...block(landing, DARK) },
    forgone: [],
  },
};

function resolveVar(tokens: Tokens, name: string): string {
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

// ── colour maths ───────────────────────────────────────────────────────────
const MACHADO = {
  protan: [
    [0.152286, 1.052583, -0.204868],
    [0.114503, 0.786281, 0.099216],
    [-0.003882, -0.048116, 1.051998],
  ],
  deutan: [
    [0.367322, 0.860646, -0.227968],
    [0.280085, 0.672501, 0.047413],
    [-0.01182, 0.04294, 0.968881],
  ],
} as const;
type Vision = "normal" | keyof typeof MACHADO;
const VISIONS: readonly Vision[] = ["normal", "protan", "deutan"];

function linear(hex: string): [number, number, number] {
  const channel = (offset: number): number => {
    const c = parseInt(hex.slice(offset, offset + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return [channel(1), channel(3), channel(5)];
}

function oklab(rgb: [number, number, number]): [number, number, number] {
  const [r, g, b] = rgb;
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function simulate(hex: string, vision: Vision): [number, number, number] {
  const rgb = linear(hex);
  if (vision === "normal") return rgb;
  const m = MACHADO[vision];
  const clamp = (c: number): number => Math.max(0, Math.min(1, c));
  return [
    clamp(m[0][0] * rgb[0] + m[0][1] * rgb[1] + m[0][2] * rgb[2]),
    clamp(m[1][0] * rgb[0] + m[1][1] * rgb[1] + m[1][2] * rgb[2]),
    clamp(m[2][0] * rgb[0] + m[2][1] * rgb[1] + m[2][2] * rgb[2]),
  ];
}

function deltaE(a: string, b: string, vision: Vision = "normal"): number {
  const x = oklab(simulate(a, vision));
  const y = oklab(simulate(b, vision));
  return 100 * Math.hypot(x[0] - y[0], x[1] - y[1], x[2] - y[2]);
}

/** Worst ΔE across normal, protan and deutan — the number the rules test. */
const worst = (a: string, b: string): number =>
  Math.min(...VISIONS.map((v) => deltaE(a, b, v)));

const lightness = (hex: string): number => oklab(linear(hex))[0];
const chroma = (hex: string): number => {
  const [, a, b] = oklab(linear(hex));
  return Math.hypot(a, b);
};

function contrast(a: string, b: string): number {
  const lum = (hex: string): number => {
    const [r, g, b2] = linear(hex);
    return 0.2126 * r + 0.7152 * g + 0.0722 * b2;
  };
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

// ── the chart vocabulary ───────────────────────────────────────────────────
const CHART_SLOTS = Object.keys(block(charts, ROOT))
  .filter((n) => /^chart-\d+$/.test(n))
  .sort((a, b) => Number(a.slice(6)) - Number(b.slice(6)));

/** Slots validated for chart forms where any two marks can sit side by side. */
const ALL_PAIRS_CAP = 3;

// The dataviz method's own thresholds.
const BAND = { light: [0.43, 0.77], dark: [0.48, 0.67] } as const;
const CHROMA_FLOOR = 0.1;
const CVD_TARGET = 8;
const NORMAL_FLOOR = 15;
const MARK_CONTRAST = 3;

const pairs = (n: number, all: boolean): [number, number][] => {
  const out: [number, number][] = [];
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      if (all || j === i + 1) out.push([i, j]);
    }
  }
  return out;
};

describe.each(Object.entries(themes))("%s — chart tokens", (_n, theme) => {
  const { mode, tokens: t } = theme;
  const palette = CHART_SLOTS.map((slot) => resolveVar(t, slot));

  it("seats every slot inside the mode's lightness band", () => {
    const [lo, hi] = BAND[mode];
    for (const [i, hex] of palette.entries()) {
      expect(lightness(hex), `${CHART_SLOTS[i]} ${hex}`).toBeGreaterThanOrEqual(
        lo,
      );
      expect(lightness(hex), `${CHART_SLOTS[i]} ${hex}`).toBeLessThanOrEqual(
        hi,
      );
    }
  });

  it("keeps every slot above the chroma floor, so none reads as grey", () => {
    for (const [i, hex] of palette.entries()) {
      expect(chroma(hex), `${CHART_SLOTS[i]} ${hex}`).toBeGreaterThanOrEqual(
        CHROMA_FLOOR,
      );
    }
  });

  // Bar and line: only touching marks need to separate.
  it("separates adjacent slots for every reader", () => {
    for (const [i, j] of pairs(palette.length, false)) {
      for (const vision of VISIONS) {
        const floor = vision === "normal" ? NORMAL_FLOOR : CVD_TARGET;
        expect(
          deltaE(palette[i] as string, palette[j] as string, vision),
          `${CHART_SLOTS[i]}↔${CHART_SLOTS[j]} under ${vision}`,
        ).toBeGreaterThanOrEqual(floor);
      }
    }
  });

  // Scatter, bubble, map: any two marks can be neighbours, which is a strictly
  // harder test and is why the cap exists.
  it("separates ALL pairs within the scatter-safe head of the sequence", () => {
    for (const [i, j] of pairs(ALL_PAIRS_CAP, true)) {
      for (const vision of VISIONS) {
        const floor = vision === "normal" ? NORMAL_FLOOR : CVD_TARGET;
        expect(
          deltaE(palette[i] as string, palette[j] as string, vision),
          `${CHART_SLOTS[i]}↔${CHART_SLOTS[j]} under ${vision}`,
        ).toBeGreaterThanOrEqual(floor);
      }
    }
  });

  it("holds every slot to the mark-contrast floor on its surface", () => {
    const surface = resolveVar(t, "surface");
    for (const [i, hex] of palette.entries()) {
      expect(
        contrast(hex, surface),
        `${CHART_SLOTS[i]} on --surface`,
      ).toBeGreaterThanOrEqual(MARK_CONTRAST);
    }
  });
});

// ── chart tokens against the status vocabulary ─────────────────────────────
//
// A chart mark and a status chip share a screen, so a series that reads as a
// warning is a real misreading. A hard floor of 15 is NOT reachable and never
// was: rust is 7.2 from the light warning, because the brand's action colour
// and the status hues are cut from the same strata. The method's own two-tier
// structure fits — target 8, floor 6, where the floor band is legal only with
// secondary encoding, and a status chip's mandatory icon and label IS that
// encoding.
const STATUS = ["ok", "warn", "err", "info"] as const;
const STATUS_FLOOR = 6;

describe.each(Object.entries(themes))(
  "%s — chart vs status",
  (_n, { tokens: t }) => {
    it("keeps every chart slot clear of every status colour", () => {
      for (const slot of CHART_SLOTS) {
        for (const status of STATUS) {
          expect(
            worst(resolveVar(t, slot), resolveVar(t, status)),
            `--${slot} vs --${status}`,
          ).toBeGreaterThanOrEqual(STATUS_FLOOR);
        }
      }
    });
  },
);

// ── the semantic layer against itself ──────────────────────────────────────
//
// Two pairs are ACCEPTED rather than fixed, and are pinned here at their
// measured values so they can only improve:
//
//   light warn/err — both must clear 4.5:1 as TEXT on a near-white ground,
//     which forces both into the same dark warm region. The best colour-only
//     separation available costs a near-black warning and a muddy error. Both
//     always ship with an icon and a label, which is the mitigation the method
//     prescribes when colour cannot do the work.
//   dark err/accent — err is semantically pinned red; accent is sand by the
//     #400 dark-set decision. Clearing this needs a brand change to the accent,
//     which is a bigger call than this gate should force.
const MEANING = ["ok", "warn", "err", "info", "accent", "cta"] as const;
const MEANING_FLOOR = 6;
const ACCEPTED: Record<string, number> = {
  "light warn/err": 2.7,
  "dark err/accent": 5.75,
};

describe.each(Object.entries(themes))(
  "%s — semantic layer",
  (_n, { mode, tokens: t }) => {
    it("separates meaning-carrying colours, or records why not", () => {
      for (let i = 0; i < MEANING.length; i++) {
        for (let j = i + 1; j < MEANING.length; j++) {
          const a = MEANING[i] as string;
          const b = MEANING[j] as string;
          const measured = worst(resolveVar(t, a), resolveVar(t, b));
          const accepted = ACCEPTED[`${mode} ${a}/${b}`];
          // An accepted pair is a ratchet, not an exemption: it may not get worse.
          const floor = accepted ?? MEANING_FLOOR;
          expect(measured, `--${a} vs --${b}`).toBeGreaterThanOrEqual(floor);
        }
      }
    });
  },
);

// ── the elevation ladder ───────────────────────────────────────────────────
//
// Surfaces do identity work through lightness alone, so ΔL is the measure and
// a hue-blind reader loses nothing. `--chip` sat 0.002 from `--surface-raised`
// before this gate existed.
const LADDER = ["canvas", "surface", "surface-raised", "chip"] as const;
const STEP_FLOOR = 0.02;

describe.each(Object.entries(themes))(
  "%s — elevation",
  (_n, { tokens: t, forgone }) => {
    it("keeps every surface a visible step from every other", () => {
      const rungs = LADDER.filter((rung) => !forgone.includes(rung));
      for (let i = 0; i < rungs.length; i++) {
        for (let j = i + 1; j < rungs.length; j++) {
          const a = rungs[i] as string;
          const b = rungs[j] as string;
          expect(
            Math.abs(lightness(resolveVar(t, a)) - lightness(resolveVar(t, b))),
            `--${a} vs --${b}`,
          ).toBeGreaterThanOrEqual(STEP_FLOOR);
        }
      }
    });

    // The PREMISE, which is the half a forgone rung rests on and the half that
    // can rot. `forgone` says a rung is not a step this surface has; if the
    // retune that made it true is edited away, the rung is a real step again
    // and BOTH halves go wrong silently — the ladder stops comparing it against
    // anything here, hiding a fresh collision, while the markup guard below
    // keeps rejecting a fill that has become legitimate. So the collision that
    // justifies the drop is asserted, not assumed.
    it("only forgoes a rung the ladder genuinely cannot seat", () => {
      for (const rung of forgone) {
        const others = LADDER.filter((r) => r !== rung).map((r) =>
          Math.abs(
            lightness(resolveVar(t, rung)) - lightness(resolveVar(t, r)),
          ),
        );
        expect(
          Math.min(...others),
          `--${rung} is a distinct step again — stop forgoing it`,
        ).toBeLessThan(STEP_FLOOR);
      }
    });
  },
);

// A forgone rung is a CLAIM about markup, and this is what holds it (#452).
//
// The landing's light canvas is `--surface-raised`'s own value, so a panel
// drawn `bg-surface-raised` there has no fill at all — RowCarousel's tray and
// FileAndFindings' note were exactly that, visible only by their border, on the
// page with the most first-time readers. Dropping the rung above records that
// the step does not exist; without this, nothing stops the next component
// reaching for it and the record becomes prose.
//
// Behind `dark:` is fine and is the established pairing on this page
// (GroupTable's card): the dark ladder has the room the light one does not.
//
// WHAT THIS DOES NOT COVER: a SHARED component that fills with it and is
// rendered here — `Chip`'s neutral and muted forms do, and so does `Input`.
// The landing imports `Button` and `ThemeToggle` and nothing else, so scanning
// its own tree is the whole of its markup today; importing one of those would
// walk straight past this.
const LANDING_DIR = resolve(import.meta.dirname, "../../../landing");
const landingSources = readdirSync(LANDING_DIR, {
  recursive: true,
  encoding: "utf8",
})
  // Everything landing.css's `@source` list reaches, and nothing it does not.
  // `.ts` as well as `.tsx`, because this codebase does keep class strings in
  // plain modules (`src/lib/controls.ts`); `.html`, because the entry names
  // `./index.html` and a class in the shell ships like any other. Test files
  // are skipped for the reason the entry `@source not`s them — a class named
  // there never reaches a bundle.
  .filter(
    (f) =>
      /\.(tsx?|css|html)$/.test(f) &&
      !/\.test\.tsx?$/.test(f) &&
      !f.startsWith("dist") &&
      !f.includes("node_modules"),
  )
  .map((f) => [f, readFileSync(resolve(LANDING_DIR, f), "utf8")] as const);

/** Each use of the fill, with its whole variant chain captured.
 *
 *  The chain, not just the segment in front: a lookbehind for `dark:` reads
 *  only the one segment before the utility, so it rejects
 *  `dark:hover:bg-surface-raised` — correctly guarded, and the natural next
 *  thing to write — while telling the author to drop the guard. Order within
 *  the chain carries no meaning to Tailwind, so it carries none here either. */
const RAISED_FILL = /((?:[\w.[\]%/-]+:)*)bg-surface-raised\b/g;
const guardedByDark = (chain: string): boolean =>
  chain.split(":").includes("dark");

describe("the landing's forgone rung", () => {
  // The positive control. The case below asserts an ABSENCE, which a scan that
  // reads nothing — or a pattern that matches nothing — produces just as well.
  it("reads the landing's markup, and the pattern matches", () => {
    expect(landingSources.length).toBeGreaterThan(5);
    const uses = landingSources.flatMap(([, src]) => [
      ...src.matchAll(RAISED_FILL),
    ]);
    expect(uses.length).toBeGreaterThan(0);
    expect(uses.some((m) => guardedByDark(m[1] as string))).toBe(true);
  });

  it("never fills with --surface-raised except behind dark:", () => {
    const offenders = landingSources
      .filter(([, src]) =>
        [...src.matchAll(RAISED_FILL)].some(
          (m) => !guardedByDark(m[1] as string),
        ),
      )
      .map(([file]) => file);
    expect(
      offenders,
      "the landing's light canvas IS --surface-raised, so this fill is invisible there — lift with the surface step or recess with the chip step, and keep the raised one behind a dark: guard",
    ).toEqual([]);
  });
});

// ── fills against strokes ──────────────────────────────────────────────────
//
// The elevation gate above compares a surface to a surface. A FILL against a
// STROKE is a different relation, and nothing asserted it: light `--chip` and
// light `--line` both resolved to `--stone-200`, so a chip drawn as `bg-chip
// border-line` (FilterBar's inactive filters) had no edge at all — it read as
// a gap in the border rather than as an object. `--surface-raised` and
// `--line-subtle` were the same collision one step up: a row ruled with
// `border-line-subtle` lost that rule under `hover:bg-surface-raised` (#446).
//
// The fills here are the SURFACES — the grounds a container is made of, the
// same set contrast.test.ts holds the fg ramp against. The `*-quiet` washes
// are fills too, but they are not in this relation: each is drawn with a
// border in its OWN status hue (`border-warn/45 bg-warn-quiet`), never against
// the neutral stroke vocabulary.
//
// ΔL is the measure for the reason the elevation gate gives — these are all
// cut from one warm-neutral ramp, so lightness is the only channel separating
// any two of them and a hue-blind reader loses nothing.
//
// The floor is HALF the elevation step, and that is what the band can hold
// rather than a considered relaxation. Light `--chip` is fenced above by
// `--line` and below by contrast.test.ts's floor for `--fg-dim` sitting on it;
// what is left between `--surface-raised` and the darkest legal chip has to
// seat two rules as well, and the light neutrals are packed evenly into it
// with nothing spare. A retune that wants a wider step has to move
// `--surface-raised` or `--fg-dim` first.
const FILLS = [
  "canvas",
  "surface",
  "surface-raised",
  "surface-code",
  "chip",
] as const;
const STROKES = ["line", "line-subtle", "line-strong"] as const;
const EDGE_FLOOR = 0.01;

describe.each(Object.entries(themes))(
  "%s — fill vs stroke",
  (_n, { tokens: t, forgone }) => {
    it("gives every fill an edge against every stroke", () => {
      for (const fill of FILLS.filter((f) => !forgone.includes(f))) {
        for (const stroke of STROKES) {
          expect(
            Math.abs(
              lightness(resolveVar(t, fill)) - lightness(resolveVar(t, stroke)),
            ),
            `--${fill} vs --${stroke}`,
          ).toBeGreaterThanOrEqual(EDGE_FLOOR);
        }
      }
    });
  },
);

// ── the code surface ───────────────────────────────────────────────────────
//
// The docs map syntax highlighting onto the status and accent tokens
// (--md-code-hl-* in docs-site/docs/stylesheets/laterite.css), which puts them
// on `--surface-code`. contrast.test.ts checks those colours against canvas,
// surface and their own wash — never against the surface they actually sit on.
describe.each(Object.entries(themes))(
  "%s — code surface",
  (_n, { tokens: t }) => {
    it("holds the syntax colours to AA on the surface they sit on", () => {
      for (const token of [...STATUS, "accent"]) {
        expect(
          contrast(resolveVar(t, token), resolveVar(t, "surface-code")),
          `--${token} on --surface-code`,
        ).toBeGreaterThanOrEqual(4.5);
      }
    });
  },
);

// ── the fence ──────────────────────────────────────────────────────────────
describe("the chart fence", () => {
  it("pins slot 1 to the CTA by assertion rather than by reference", () => {
    expect(resolveVar(light, "chart-1")).toBe(resolveVar(light, "cta"));
    expect(resolveVar(dark, "chart-1")).toBe(resolveVar(dark, "cta"));
  });

  it("keeps chartTheme's palette in step with the tokens", () => {
    const source = readFileSync(
      resolve(import.meta.dirname, "../../lib/chartTheme.ts"),
      "utf8",
    );
    const used = [...source.matchAll(/readVar\("--(chart-\d+)"\)/g)].map(
      (m) => m[1],
    );
    expect(used).toEqual(CHART_SLOTS);
  });

  it("keeps the chart tokens out of the semantic layer", () => {
    // A `var(--chart-N)` anywhere in colors.css would mean a UI role had taken
    // a chart colour, which is the leak the separate file exists to prevent.
    expect(colors).not.toMatch(/var\(--chart-\d+\)/);
  });
});
