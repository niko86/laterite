# Handoff: laterite site direction — demo site (light + dark) and docs

## Overview
This bundle documents the agreed visual direction for two laterite surfaces:

1. **laterite.dev demo site** — the single-page "show, don't tell" site (four editable AGS4 group tables wired to a live file and findings), redesigned on the "bedrock" pairing with a scroll-linked borehole rail. Light theme (**3a**) and dark theme (**3b**).
2. **docs.laterite.dev** — the MkDocs documentation site carried onto the same direction (**4a**): one cookbook page demonstrates every proposed change.

The direction was chosen from four colour/type pairings and two rail concepts explored on the same canvas (turns 1–2 in the design file, kept for provenance).

## About the design files
The files here are **design references created in HTML** — prototypes showing intended look and behaviour, not production code to copy. The task is to **recreate these designs in the target codebases**:

- Demo site → `templates/demo-site-svelte/` in the laterite design-system repo (Svelte 5 + Vite + TypeScript, per `guidelines/architecture.md`). That template already implements the page's engine and store; this handoff changes its **presentation**.
- Docs site → the MkDocs + Material override layer in `web/docs-site/` of [niko86/laterite](https://github.com/niko86/laterite). Everything except the TOC rail is CSS; the rail needs one small JS hook.

Open `Demo Site Pairings.dc.html` in a browser to see everything. It is a pan/zoom canvas, newest work at the top. Option ids: **4a** docs page · **3a** demo light · **3b** demo dark · turns 1–2 are the earlier explorations (1a–1d pairings, 2a/2b rail studies) and are context, not deliverables. Each option carries a mono `PAIRING` / `TRADE-OFF` caption under the frame — those captions are part of the spec.

## Fidelity
**High-fidelity.** Colours, type, spacing, radii and copy in frames 3a/3b/4a are final intent. Recreate pixel-perfectly with the design system's tokens (bundled under `_ds/`, same values as the laterite design system repo). The scroll-linked rail behaviour in the mocks is real and interactive — scroll inside each frame.

## The pairing (applies to both surfaces)
- **Display / headings:** Zilla Slab 600–700 (Google Fonts), tracking −0.015 to −0.022em. h1/hero headlines in maroon `#611a1e`. Slab never sets body copy.
- **Body / UI:** Public Sans 400–700.
- **Code / AGS codes / data:** IBM Plex Mono (unchanged from the design system).
- **Colour roles:** **maroon reads, rust acts.** Links, headlines, active nav = maroon `#611a1e` (hover `#3f1114`). Every button/CTA = rust `#ce5640` with `#faf7f5` text. Severity tokens unchanged (`--err #a51f14`, `--warn #8f5312`, `--info #5f5761`, `--ok #2f5d5a` in light).
- **Canvas:** demo site `#ece5e0` with `#faf7f5` surfaces (strong card lift); docs `#f4efeb` — one step lighter, for long reading.

## Screens / views

### 3a — demo site, light
Frame: 1240×900 in the mock; real page is fluid with `max-width: 84rem` content.

**Masthead** (sticky): light `--surface #faf7f5`, closed by a 3px strata gradient hairline `linear-gradient(90deg,#f09c56,#db7841 30%,#ce5640 55%,#be3b2e 75%,#611a1e)`. Brand lockup: mark (26px, no plate on light chrome) + "laterite" in Zilla Slab 700 1.35rem `#611a1e`. Nav links `--fg-soft` 1rem. Right: theme glyph ☾, rust CTA "Open the web app" (`#ce5640`, radius 5px, `0.45rem 1.05rem`, 600).

**Borehole rail** (the page's signature): fixed left gutter, **96px wide**, `--surface` fill, 1px right hairline, above the scrolling content (z 3; masthead z 2 scrolls under it… in the mock the masthead is inside the scroller at z 2 — in production make the rail and masthead siblings with the rail below the masthead).
- Strata strip: 26px wide at left 10px, seven equal vertical bands top→bottom: `#f09c56 #db7841 #ce5640 #be3b2e #9b3932 #7d2622 #611a1e`.
- Veil: the part of the strip **below** the probe is covered by `color-mix(in srgb, #ece5e0 52%, transparent)` with a 2px `#5f5761` top edge — layers "uncover" as you descend.
- Probe: 2px steel `#5f5761` horizontal line spanning the full rail width, `z-index` above the rail fill.
- Depth pill: sits **on the strip** (left 6px), maroon `#611a1e` fill, radius 3px, padding `0.12rem 0.15rem`, IBM Plex Mono 0.66rem 600 `#faf7f5`. Shows depth only (e.g. `12.50`) — no unit, no group name (the scale carries those).
- Depth scale: tick column at left **52px** (must clear the pill's widest 5-char state), one tick per section: `0.00 m / 3.57 / 7.14 / 10.71 / 14.29 / 17.86 / 21.43` with group labels `hero PROJ LOCA SAMP LLPL file install` (IBM Plex Mono 0.7–0.72rem; number `--fg-soft` 600, label `--fg-faint`), 1px top hairline per tick.
- **Scroll math:** `p = scrollTop / (scrollHeight − clientHeight)`, clamped 0–1. Depth = `(p × 25).toFixed(2)` (2DP — the wink at AGS TYPE `2DP`; total 25.00 m matches the seeded `LOCA_FDEP`). Probe/pill top = `calc(p·100% + (0.5 − p) · 26px)` — a 13px-inset track so the pill is never clipped at 0% or 100%.
- Rail collapses below ~1080px to the hairline version (2a in the file: 8px strip, no scale); below ~820px the interactive demo hides per the design system's mobile policy.

**Sections** (each is one rail band, top→bottom): hero · PROJ · LOCA · SAMP · LLPL · the file + findings · install. Zig-zag: table column always `1.35fr`, prose `1fr`, sides alternating. Section blocks `min-height 300px`, `padding 1.8rem 0`, 1px top hairline.

**Group ↔ band keying** (the boldness system): each group's chip, table cap and KEY-column edge use that group's band colour — PROJ `#db7841`, LOCA `#ce5640`, SAMP `#be3b2e`, LLPL `#9b3932`, install `#611a1e`.
- **Group chip idiom:** band tint `color-mix(in srgb, <band> 26%, transparent)` + `inset 3px 0 0 <band>` left rule + **maroon `#611a1e` text**, IBM Plex Mono 700, radius 3px, padding `0.15rem 0.6rem`. Never white or black text on a band fill — mid-ramp bands fail contrast both ways.
- **Spotlight tables** (the one sanctioned spotlight): 1px `#f0a67f` border, radius 8px, shadow `0 14px 34px -18px rgb(97 26 30 / 38%), 0 2px 6px -2px rgb(97 26 30 / 14%)`, 3px **solid band-colour** cap (not the 4-band gradient).
- **KEY columns:** `color-mix(in srgb, <band> 18%, transparent)` fill + `inset 3px 0 0 <band>` + ◆ marker in the band colour beside the heading.
- Seeded findings: `LOCA_GL` "11.8" vs TYPE 2DP (error, `--err` text + 2px underline + `--err-quiet` fill), `SAMP_TYPE` "b" (warning), LLPL row keyed to `BH03` — an orphaned key (error, deliberately not fixable).

**Hero:** mono uppercase eyebrow with a 4px sand tick, `#9b3932`; h1 Zilla Slab 700 **3.9rem**, −0.022em, lh 1.02, `#611a1e`, max 22ch; lead 1.2rem `--fg-soft`; rust CTA + maroon-outline ghost; file excerpt card right (mono 0.92rem, lh 1.8, `#611a1e` text).

**The file + findings:** findings list left (design-system `SummaryBanner kind=err` + `Chip` rows: error=solid, warning=rule, fixable=ok outline), output pane right — mono file lines, the offending line banded `--err-quiet` with a 3px `--err` inset and `--err` text.

**Install — "Pick your stack":** heading + one-liner, then a 5-column card grid (Python·PyPI / Node·npm / CLI·crates.io / DuckDB·community / Browser·wasm). Each card: radius 8, 1px `--line`, 2px maroon top cap, micro uppercase surface label, mono package name 0.88rem 600, mono command 0.8rem `--fg-muted` (both `overflow-wrap:anywhere`). Python card highlighted: `--accent-quiet` fill + `#f0a67f` border. Below: rust "Read the install guide" + the beta line.

### 3b — demo site, dark
Same layout; only these change (all verified in the mock):
- Tokens: canvas `#14100f`, surface `#1c1715`, raised `#262019`, code `#100c0b`, lines `#352c28 / #261f1c / #4a3f39`, text `#ede5df / #d8cbc2 / #a4958d / #8b7d75 / #6b5e57`, accent sand `#f09c56` (hover `#f0a67f`), accent-quiet `#3a2620`, status `--err #f07f6d`, `--warn #e3a447`, `--ok #79c2b4`, `--info #b9b2bd` (+ matching quiets).
- **Band ramp shifts one step lighter** (GitHub-Primer-style dark scale): rail/caps/chips/KEY edges run `#f0a67f #f09c56 #db7841 #ce5640 #be3b2e #9b3932 #7d2622` so the deep strata hold on the dark canvas.
- **Elevation by lightening:** spotlight cards fill `--surface-raised #262019`; shadows go black (`rgb(0 0 0 / 62%)` / `40%`) and are secondary.
- **Tints dial down:** chips 26%→20%, KEY columns 18%→14% — saturated washes glow on dark.
- Headlines/links sand `#f09c56`; rust CTA unchanged; chip text `#ede5df`; card borders `#9b3932`.
- Depth pill inverts: sand `#f09c56` fill, `#1c1715` text. Veil `color-mix(in srgb, #14100f 52%, transparent)` with a steel `#c5c3c2` edge.
- **Dark chrome rule:** the mark sits on a `#faf7f5` plate (3px padding, 5px radius) whenever chrome is dark.
- Open decision: theme follows OS vs toggle, and the landing default.

### 4a — docs.laterite.dev
One cookbook page ("Validate a delivery") demonstrating all proposed docs changes:
- **Masthead:** dark `#1c1715`, 2px strata hairline, mark on the `#faf7f5` plate, lockup `laterite | docs · v0.4.2` (product name in UI font per the lockup rule — never slab), search input with a normal light fill (never translucent), rust CTA.
- **Layout:** left nav 230px, content column `max-width 46rem`, right TOC 190px. Canvas `#f4efeb`.
- **Left nav, band-keyed sections:** each top-level section gets an 8px band swatch — Getting started `#f09c56`, Cookbook `#db7841`, Reference `#ce5640`, Support `#9b3932`. Active item: `--accent-quiet` fill + `inset 3px 0 0 <section band>` + maroon 600 text.
- **TOC rail:** the hairline dose — 6px strata strip (four bands `#f09c56 #db7841 #ce5640 #9b3932`) on the TOC's left edge, scroll-progress veil (`#f4efeb` at 82%) and a 12px steel probe. **No depth numbers, no readout** — it is an echo, not an instrument.
- **Prose:** h1 2.3rem Zilla 700 maroon; h3 1.5rem Zilla 600 `--fg`; body 1rem lh 1.7 `--fg-soft`; inline code on `--surface-code` with 1px `--line-subtle` and 3px radius; mono uppercase breadcrumb.
- **Components:** design-system `CodeTabs` (Python/Node/CLI/DuckDB) and `Admonition` (tip, warning) as-is.
- **Catalogue spotlight:** the group catalogue is the docs' one spotlight — `#f0a67f` border, warm shadow, 3px 4-band strata cap, filter input, `174 groups · DICTIONARY EDITION 4.1.1` mono caption, mono pager (`← prev 1 2 3 ⋯ 44 next →`).
- **Constraint:** band colour appears only in the nav swatches/rules and the TOC rail — never in prose or admonitions, so it cannot collide with severity.
- Dark docs inherit 3b's rules.

## Interactions & behaviour
- **Live, no submit** (demo site): emit + validate on every keystroke; Fix applies safe repairs only; findings are click-to-jump with an amber band on the output line. See `ui_kits/demo-site/README.md` in the design-system repo for the full interaction contract — this handoff changes presentation, not behaviour.
- **Rail:** passive scroll listener; positions as specified above. No self-running animation — reduced-motion is unaffected by design.
- Motion elsewhere: the design system's single easing `cubic-bezier(.33,1,.68,1)`, 120–200ms, colour-only hovers, nothing scales or bounces.

## Design tokens
Bundled under `_ds/` (`tokens/*.css` + `styles.css`) — identical to the design-system repo. Values not in the tokens (introduced by this direction): rust action `#ce5640` as `--cta`; the seven-band rail ramps (light `300→900`, dark `200→800`); tint levels 26/18% light, 20/14% dark. **Add Zilla Slab + Public Sans** (Google Fonts, weights 400–700) — the bundle's `fonts.css` currently loads Rokkitt/Plex only.

## Assets
- `assets/laterite-icon-256.png` — light-background mark (use on light chrome and on the plate).
- `assets/laterite-icon-256-white.png` — white-outlined mark for dark/maroon backgrounds without a plate.
Both copied unmodified from the laterite repo's `assets/`.

## Files
- `Demo Site Pairings.dc.html` — the canvas with all frames (4a, 3a, 3b at top; earlier explorations below). Interactive; open in a browser.
- `support.js` — runtime the HTML file needs to render. Reference only.
- `_ds/` — design tokens + component bundle the mock loads. The token files are the source of truth for every value not listed above.
- `assets/` — the two marks.
