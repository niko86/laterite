# laterite — design system

**laterite** is a modern **AGS4 toolkit**: one Rust engine surfaced natively for
Python, Node.js, a CLI (`lat`), DuckDB and the browser. It validates, reads
(born-typed), queries, builds, fixes, diffs and certifies AGS4 geotechnical
transfer files, and converts to and from Excel. It is not a company — it is one
open-source project (MIT) with three public faces:

| Surface | Where | Built from |
|---|---|---|
| **Docs site** | docs.laterite.dev | MkDocs + Material (`web/docs-site/`) |
| **Web app** | app.laterite.dev | SolidJS + Rust-wasm client-side validator + data explorer (`web/src/`) |
| **Packages** | PyPI · npm · DuckDB community · crates.io | the Rust engine and its bindings |

The audience is geotechnical engineers, data managers and the people who wire
AGS4 into pipelines — technically literate, allergic to marketing.

## Sources this system was built from

Nothing here is invented; every value is lifted from one of these. The reader is
not assumed to have access, but these are the addresses:

- **GitHub — [github.com/niko86/laterite](https://github.com/niko86/laterite)** (branch `main`), specifically:
  - `assets/` — the logo set. **The entire colour system is sampled from `assets/laterite-icon-512.png`.**
  - `web/docs-site/mkdocs.yml`, `docs/index.md`, `docs/cookbook/*`, `docs/stylesheets/catalogue.css` — the docs IA, prose voice and the group catalogue behaviour.
  - `web/src/app.css` — the app's semantic token *names* and elevation/text ladders (the values were cool slate; they are re-tuned here, the names kept).
  - `web/src/App.tsx`, `web/src/components/**` — the component inventory and exact control values.
  - `README.md` — product copy, the surface table, the performance table, tone.
  - Worth exploring further for anyone extending this system: the `web/src/components/explore/` and `tools/` trees hold a dozen more screens than the kit recreates.
- **Attached codebase — `app/`** (a separate Svelte + Vite forms builder by the same author). Adopted: its **UI primitive inventory and exact control metrics** (`src/lab/styleguide/StyleGuide.svelte` is an explicit style guide + primitives audit), its type scale, its Lucide iconography (`src/builder/icons.ts`), its motion contract (`src/motion.ts`), tooltip and toast patterns (`src/app.css`, `src/builder/ToastHost.svelte`). **Not adopted, at the user's instruction: that app's Fugro brand palette and logo.**

Read this alongside the repos: the design system tells you *how* it should look,
the repos tell you *what* the product does.

---

## CONTENT FUNDAMENTALS

The voice is a **precise engineer explaining their own tool**. It is confident
about mechanics and openly modest about maturity.

**Person.** Mostly impersonal, subject-first: "Files come back born-typed."
Second person for instructions and for the reader's own material: "what it hasn't
had is your files", "your file never leaves your machine". First-person plural is
rare and only appears in feedback asks ("Tell us how it goes"). Never "we
believe", never "our mission".

**Casing.** Sentence case everywhere — headings, buttons, labels, nav. UPPERCASE
is reserved for two literal things: AGS4 group/heading codes (`LOCA`, `LOCA_GL`,
`GROUP`, `HEADING`) and micro-labels in the UI (`DICTIONARY EDITION`). Never
Title Case A Heading Like This.

**Sentence shape.** Short declaratives, then an em-dash clause that earns its
keep. Real examples from the source:

> "One Rust engine, surfaced natively for Python, Node.js, the CLI, DuckDB and the browser — the same rules, the same bytes, the same findings on every one."

> "All five are in beta — the engine is tested; what it hasn't had is your files."

> "Runs entirely in your browser — your file never leaves your machine. No server, nothing uploaded."

> "A `2DP` heading is a float, a `DT` a datetime, an `ID` a string — so polars, SQL and the typed graph see real types, not text."

**Numbers do the boasting, prose never does.** "30.0× faster" appears in a table
with the file size, hardware and method; the prose around it says "Reproduce
with…". No "blazingly fast", no "revolutionary", no "seamless".

**Honesty is a house style.** Limitations are stated in the same breath as
capabilities: "in beta", "121 / 131 of python-ags4's own test suite passes
(92 %)", "The CLI is deliberately not mirrored", "10 remaining are deliberate
non-closures". Carry that over: if a design or kit doesn't cover something, say
so in place rather than faking it.

**Comparisons name the other tool respectfully.** python-ags4 is credited as
inspiration before any table compares against it.

**UI copy.** Verdicts lead with a glyph and a count: "✓ Clean — 0 findings",
"✗ 36 errors · 14 informational", "ⓘ 12 informational (FYI) findings — no errors
or warnings". Then a supporting line that says which dictionary edition was used
and how it was resolved. Then the smallest line: caps, hints, what to do next.
Middots (`·`) separate peer facts; em-dashes attach a consequence.
Empty and error states explain the mechanism ("FYI findings are hidden by
default — switch on `fyi` in the severity filter to see them").

**Tooltips** label icon-only controls in a few words. **Long help** is a
sentence with a full stop.

**Emoji.** Not in product UI, ever. The GitHub README uses exactly four as
link markers (📖 🌐 📓 and the tick/dash glyphs in comparison tables) — that
concession stops at the repo. Unicode symbols (☀︎ ☾ ✓ ✗ ⓘ ⋯ ←→) are used as
UI glyphs and are fine.

---

## VISUAL FOUNDATIONS

### The idea

The logo is a **cross-section of laterite soil**: banded strata in rust, brick and
sand, a steel auger driven through it, a crab fossil in the profile, all bound by
a maroon outline. The system takes that literally — **warm ground, cool tool**.
Every colour is soil; the only cool family is the steel of the probe. Interfaces
are dense, flat and instrument-like: this is a tool for reading data files, not a
brochure.

### Colour

Sampled from `assets/laterite-icon-512.png`: `--laterite-900 #611a1e` (outline and
wordmark), `700 #9b3932` (the dominant mass, and the interactive accent),
`600 #be3b2e`, `500 #ce5640`, `400 #db7841`, `300 #f09c56`, `200 #f0a67f`. Steel:
`#c5c3c2` / `#91858e`. Neutrals are a warm **stone** ramp in the same hue family —
never a cool grey, never pure `#fff` or `#000`.

Roles — **no traffic lights.** There is no green in this brand, and green/amber/red
is the palette every generic UI reaches for, so severity runs *down the strata*
instead: `--warn #8f5312` ochre (topsoil), `--err #a51f14` oxide red (bedrock —
hotter and heavier than the brand brick, so a finding never reads as chrome),
`--info #5f5761` the auger's steel for FYI, and `--ok #2f5d5a` a teal-slate that
means *verified* rather than *success*. `--accent` stays brand brick for links,
active tabs and focus. Each role has a `*-quiet` wash for tinted fills. Dark mode
is a warm brown-black (`--canvas #14100f`) with sand as the accent — never
near-black, body text lands about 13:1 rather than a glaring 16:1.

**Severity is encoded in form, not only hue** — solid fill for the loudest state, a
3px coloured left rule ("stratum tick") for the middle, a hairline stencil for the
calm one. A findings list therefore still reads in greyscale and to a colour-blind
reader. Chips and verdict badges are mono, uppercase, 3px radius — instrument
labelling, never soft pastel pills.

Two background colours per surface at most: `--canvas` and `--surface`. The one
literal-colour exception is print/paper (`--page: #fff`), which is artwork
parity and must never be re-themed.

**Dark chrome rule.** Full-width chrome (docs masthead, site nav) is the warm
brown-black `--stone-900`, closed with a 2px strata hairline — the four brand
bands as a left-to-right gradient. Saturated maroon at that size reads as
alarm, and the logo's own maroon outline disappears into it. Deep maroon
`--laterite-900` stays for small dark objects only: toasts and tooltips.
**The mark on any dark chrome sits on a `--stone-50` plate** (3px padding, 5px
radius) using the light-background icon, so its outline always separates.
Search and other inputs inside dark chrome take a normal light control fill —
never a translucent tint, which turns placeholder grey into mud.

### Type

Display/headings and the wordmark voice: **Rokkitt**, a geometric slab in the
Rockwell lineage (a substitution — see *Font substitutions*), 600–800, tight
tracking. **Slab is display only — never body copy.** UI and body:
**IBM Plex Sans**. Code, AGS codes, expressions and SQL: **IBM Plex Mono**,
which shares Plex's skeleton so tables and prose read as one family.

The UI scale is dense and has hard floors, lifted from the app's own type sweep:
`1.25rem` page h1 · `1rem` dialog title · `0.85rem` body · `0.82rem` controls ·
`0.8rem` captions (the floor for sentence text) · `0.72rem` uppercase micro-labels
(the floor for labels) · `0.65rem` count bubbles only. Line-height `1.45` in the
app, `1.7` for docs prose. Root font size scales up on large high-DPI panels
(15.5px ≥1800px, 17px ≥2300px) — a real rule in the app, worth keeping.

### Spacing, radii, layout

Fractional-rem steps (`0.15 / 0.25 / 0.4 / 0.6 / 0.8 / 1 / 1.25 / 1.5 / 2rem`),
**not** a 4/8 grid — that is why the controls read tight and instrument-like. Copy
exact values: a button is `0.3rem 0.8rem`, an input `0.25rem 0.4rem`.

Radii are small and specific: `3px` inputs and add-chips, `4px` small buttons and
tooltips, `5px` buttons and toasts, `6px` menus and popovers, `8px` cards and
dialogs, `10px` chips, pill for count bubbles. Nothing is soft or bubbly.

Layout: app shell `max-width: 80rem` centred with `1.25rem` gutters; docs column
`68rem` with a left nav and a right on-page TOC; a full-width tab bar sits
directly under the header on a hairline. Sticky elements: the docs masthead
(z 30), table headers inside scroll regions, and the toast host (fixed
bottom-left, z 60). Long data regions cap at `60dvh` (`70dvh` from `lg`) and
scroll internally rather than growing the page.

### Backgrounds and imagery

No gradients, no hero photography, no illustrations, no patterns or textures, no
noise. Surfaces are flat token colours with hairlines. The logo art is the only
imagery in the system, used as a mark — never stretched behind content and never
recoloured. If a screen feels empty, that is a layout problem, not a missing
background.

### Elevation, borders, shadows

**Cards never cast a shadow.** They lift by the `canvas → surface → surface-raised`
step plus a 1px `--line`. Shadow means "this floats above the page" and each layer
has one fixed value: tooltip `0 2px 8px /30%`, menu `0 4px 12px /25%`, toast
`0 4px 16px /35%`, popover `0 6px 20px /28%`, dialog `0 12px 40px /35%`, command
palette `0 14px 44px /35%`. Borders: 1px hairline default, `--line-strong` on
controls, dashed `--line-strong` for "+ add" affordances, 2px dashed for a
drop zone, 2px solid accent for an active tab underline.

### Transparency and blur

Used sparingly and only two ways: quiet tint washes (`color-mix` at 10–18%) for
chips, banners and severity bands, and white-alpha borders/hovers on the dark
maroon toast and tooltip (`rgb(255 255 255 / 18%)` border, `/12%` hover).
The scrim is maroon-tinted `--scrim` at 45%. **Nothing is ever blurred** — no
frosted glass, no backdrop-filter.

### Motion

Short, functional, one easing: `cubic-bezier(.33, 1, .68, 1)` (svelte `cubicOut`).
120ms opacity fades, 150ms colour and transform, 180ms enter/exit, 200ms
width/height reveals. The one enter animation in the system is the toast: fly
in 12px from below over 180ms. Nothing bounces, springs, scales in, or parallaxes.
Tooltips appear after a uniform 300ms delay and fade in over 120ms. Every duration
collapses to zero under `prefers-reduced-motion` — that contract is in the tokens.

### States

- **Hover:** text goes muted → `--fg`, or picks up `--accent`; quiet buttons gain a `--chip` fill; ghost buttons go muted → `--err` when destructive; table rows gain `--surface-raised`. Never a shadow lift, never a scale.
- **Press:** colour only. Nothing shrinks or translates.
- **Focus:** `--focus-ring` (3px accent at 30%) or an accent border; visible, never removed.
- **Active/selected:** accent text plus either a 2px accent underline (tabs), a `--accent-quiet` fill (pills, list rows) or a 2px accent left-border (docs nav).
- **Disabled:** `opacity: 0.45` and `cursor: default` — never a grey repaint.
- **Armed confirm:** destructive actions repaint to `--err` on first click and act on the second; the label changes to the question ("Replace current fill?").

### Cards, in one line

8px radius, 1px `--line`, `--surface` fill, `0.8rem` padding, no shadow, no
coloured left border, no header bar unless the content needs one.

**One documented exception — the "spotlight" table.** On the demo site the group
tables *are* the argument of the page, so they take a brand-tinted border
(`--laterite-200`), a lifted shadow
(`0 14px 34px -18px rgb(97 26 30 / 38%)`) and a 3px strata cap. Use it only for
an object the page exists to show; a plain content panel stays a flat Card.

### Mobile policy

The products are desktop instruments — an AGS delivery is a wide table and a
120-line file. Below ~1080px, two-column layouts collapse to one and drawn
connectors become a dashed vertical rule; tables keep **horizontal scroll rather
than shrinking type** (a KEY chain at 12px is unreadable). Below ~820px an
interactive demo is **hidden outright and replaced by a short explanatory block
plus links** — a deliberate policy, not a fallback: better an honest one-pager
than a crippled tool.

---

## ICONOGRAPHY

**Lucide** is the icon set — the attached builder app bundles `@lucide/svelte`
with per-icon deep imports (`src/builder/icons.ts` lists ~45 in use: `shield-check`,
`triangle-alert`, `file-down`, `git-compare-arrows`, `history`, `funnel`,
`grip-vertical`, `search`, `undo-2` / `redo-2`, `trash-2`, `x`, …). Default 16px,
1.5px stroke, `currentColor`, always paired with a label or a tooltip.

Because this system has no bundler, `components/core/Icon.jsx` loads the icon from
the **lucide-static CDN** and paints it with a CSS mask, so it still inherits text
colour and flips with the theme. **This is a CDN link, not vendored files** — flag
it if you need an offline build, and copy `lucide-static/icons/*.svg` into
`assets/icons/` at that point.

The app's own SVGs are two hand-drawn primitives, both kept as components rather
than assets: the disclosure chevron (`Chevron`) and the spinner (`Spinner`).
Unicode is used deliberately for a few glyphs and should stay: `☀︎ ☾` (theme
toggle), `✓ ✗ ⓘ` (verdicts), `⋯` (elided rows), `← →` (paging), `▶` (run).
**No emoji in product UI.** No icon fonts. Never hand-draw a replacement glyph —
if Lucide has no match, say so.

### Product lockups

Product names are **not** set in the display slab. The lockup is always:
mark · **laterite** in `--font-display` (800, `--laterite-900`) · a 1px
`--line-strong` divider · the product name in `--font-ui` (600, `--fg`) · an
optional muted qualifier. So the app header reads
`laterite | AGS4 Validator  + data explorer`, not "AGS4 Validator" in slab —
a long product name in Rockwell-weight slab looks like a cereal box. Slab is for
the brand word and for headings, nothing else.

Logo assets in `assets/` (all copied from the repo, none redrawn):
`laterite.svg`, `laterite-icon-128/256/512.png`, `laterite-icon-256-white.png`
(white-outlined, for dark and maroon backgrounds), `laterite-icon-flat.png` /
`-flat-transparent.png`, `laterite-social-preview.png` / `-white.png` (full
lockup with the wordmark). There is no separate wordmark file — use the social
preview lockup, or set "laterite" in Figtree 800 in `--laterite-900`.

---

## Font substitutions — please confirm

At delivery time the repos shipped **no font binaries** — the site and app fell
back to system stacks, and the wordmark/social-preview headline was set in a
licensed geometric-humanist face that isn't in the source. *(Since overtaken:
the shipped site and app now self-host their faces — Figtree for display,
Public Sans for UI/body, IBM Plex Mono — as tracked woff/woff2 files wired in
`web/docs-site/docs/stylesheets/tokens.css` and `web/src/app.css`. The
substitutions below describe THIS bundle's own `tokens/fonts.css`, which still
loads them from the Google Fonts CDN.)*

- **Rokkitt** (Google Fonts) stands in for the wordmark/display face — the closest free homage to Rockwell, which is what the mark reads as.
- **IBM Plex Sans** replaces the platform system stack for UI and body copy.
- **IBM Plex Mono** replaces the generic `ui-monospace` stack.

All three load from the Google Fonts CDN in `tokens/fonts.css`. Two heavier slab
alternatives are drawn up as live specimens for comparison in
`guidelines/candidates/` (Arvo + Source Sans 3; Zilla Slab + Public Sans).
**If you have the real display face, send the files and I will swap them in and
re-tune the display scale.**

---

## Index

| Path | What it is |
|---|---|
| `styles.css` | the only file consumers link — `@import`s everything below |
| `tokens/colors.css` | brand ramp, steel, stone neutrals, surfaces, text ladder, status, dark theme |
| `tokens/typography.css` | font stacks, the dense UI scale, editorial scale, weights, tracking |
| `tokens/spacing.css` | space steps, radii, control heights, layout maxima |
| `tokens/elevation.css` | the five floating-layer shadows, scrim, z-index ladder |
| `tokens/motion.css` | durations, the one easing, reduced-motion collapse |
| `tokens/fonts.css` | webfont loading + the substitution note |
| `guidelines/*.card.html` | 20 foundation specimen cards (Colors, Type, Spacing, Brand) |
| `guidelines/architecture.md` | **the house stack** — Svelte 5 runes, Vite, TS, vitest/Playwright, lucide, and the contracts every surface honours |
| `components/` | the reusable primitives, grouped by concern |
| `ui_kits/webapp/` | AGS4 Validator recreation — 5 screens, click-through |
| `ui_kits/docs/` | docs.laterite.dev recreation — 4 pages, click-through |
| `ui_kits/demo-site/` | single-page "show, don't tell" site: 4 editable AGS groups → live file + findings + fix. Its README carries the page's **visual & interaction language** |
| `templates/demo-site-svelte/` | the same page as buildable **Svelte 5 + Vite** source — pure engine, rune store, vitest (under `templates/` so the in-browser bundler leaves the npm imports alone) |
| `guidelines/candidates/` | type-pairing specimens, for picking the display face |
| `assets/` | logo marks and lockups, copied from the repo |
| `SKILL.md` | Agent Skills entry point |
| `github.md` | source-repo association for upstream sync |

### Components

Grouped by concern; every one has a `.d.ts` props contract and a `.prompt.md`
usage note beside it.

- **core** — `Button`, `Chip`, `StatusBadge`, `CountBubble`, `Spinner`, `Chevron`, `Icon`
- **forms** — `Input`, `Select`, `Checkbox`, `Field`, `ControlGrid`
- **surfaces** — `Card`, `Disclosure`, `Dialog`, `Admonition`, `CodeTabs`
- **navigation** — `Tabs`, `PillToggle`, `ThemeToggle`
- **feedback** — `Toast`, `Tooltip`, `SummaryBanner`

The inventory comes from the sources, not from a generic checklist: the web app's
own components (`Card`, `Chevron`, `ControlGrid`, `Disclosure`, `PillToggle`,
`Spinner`, `Tabs`, `ThemeToggle`) plus the families the builder app's style-guide
audit enumerates (button families, chips and badges, count bubbles, form
controls, the dialog skeleton, toasts, tooltips), plus the two docs-site
patterns (`Admonition`, `CodeTabs`).

**Intentional additions** (not in either source, flagged as required):

- `Icon` — the builder app imports Lucide glyphs individually; a bundler-free design system needs one wrapper.
- `Field` — the label-above-control pattern is repeated inline in `Controls.tsx`; naming it stops the drift the audit warns about.
- `StatusBadge` — the app's PASS/FAIL badges are literal-hex artwork parity; this is the same idiom on tokens, for UI use.

### What this system deliberately does **not** contain

No Avatar, Toast queue manager, Accordion, Breadcrumb, Pagination or Switch
component — none exists in the sources. Boolean options are labelled checkboxes;
"tabs inside a pane" is `PillToggle`. If you need something new, add it here with
a one-line reason rather than assuming.
