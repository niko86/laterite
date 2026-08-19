---
type: decision
title: "laterite.dev gets its own build, and the token layer becomes the shared one (#394)"
status: accepted
tags: [design, decision, web, site, tokens]
decided: "2026-08-18"
supersedes: []
from_gap: []
related: [validator-site, tech-stack-wasm]
sources: []
---

# laterite.dev gets its own build, and the token layer becomes the shared one

## Context

Three surfaces answer on three hosts — the apex, the docs and the browser app —
and until now only one of them was built. The apex was one hand-written file
served as assets by a Worker with no code (`repo:web/landing/wrangler.jsonc`),
which was the right shape for a placeholder whose only job was to stop the
GitHub Pages redirect shim pointing at a hostname with no record.

It is the wrong shape for what #334 describes: a page with four editable AGS4
group tables, a lazily-loaded engine and a live findings pane. And #411 moves
all three surfaces onto one visual direction, which means the question "where do
the tokens live" has to be answered before any of the tickets that consume them
can start.

The failure mode #334 names by name is the landing page quietly becoming a worse
copy of the app. That is the thing this page is arranged against.

## Options considered

**1. Where the landing page builds.**

1. **A second entry in the app's Vite config.** One config, one build, one
   `npm run build`.
2. **Its own Vite config under `web/landing/`, sharing the web project's single
   dependency set.** ← chosen
3. Its own npm project, its own `package.json`, its own lockfile.

**2. Where shared code lives.**

1. A published package, or a workspace package with a build step.
2. **One shared directory under the web source tree, aliased by both builds.** ← chosen
3. Copy the primitives into each surface.

**3. What the token layer is.**

1. A landing-page stylesheet, promoted to shared once a second surface needs it.
2. **The shared artefact from the start, authored where all three surfaces can
   read it.** ← chosen

## Decision

**One dependency set, two builds, one shared directory.**

`repo:web/landing/vite.config.ts` is the apex's own config with its own root,
its own entry and its own output. It shares `repo:web/package.json` and nothing
else. Both configs resolve `@shared` to `repo:web/src/shared`, and the tokens
live there — `repo:web/src/shared/styles/tokens.css` and the three files it
composes.

The pairing's display face is **Figtree**, and no slab is referenced anywhere in
the token layer or the surfaces consuming it.

## Why

**Separate builds are the enforcement mechanism, not a preference.** The app's
config carries machinery that is app-only in every case: a PWA registration
layer, an SPA 404 fallback, and a step that relocates the oversized DuckDB wasm
out to R2 (`repo:web/vite.config.ts`, and [[dec-engine-tiering]] for what that
tiering is protecting). Spanning those across a two-entry build would apply all
three to a page that wants none of them. Worse, shared-chunk splitting between
two entries makes it easy to drag the app's heavy dependencies into the apex
bundle without anyone noticing — which is exactly the failure being designed
against, delivered by the mechanism meant to save effort.

Option 3 was rejected for the opposite reason: a second `package.json` makes the
two surfaces drift on versions, and there is no version skew worth having
between a page and the app it links to.

**Sharing a toolchain is not sharing a bundle — and a check says so.**
`repo:web/landing/appOnlyDependencies.ts` fails the landing build if an app-only
dependency reaches the module graph. The rule is inverted from the obvious one:
rather than a denylist of banned packages, *everything the web project declares
is forbidden on the apex unless explicitly shared*. A denylist has to be
extended by the same person who just added the thing it should have caught;
this way, adding DuckDB, Arrow, ECharts or Leaflet to the app fences the apex off
from it on the same commit, and widening the shared set is the only deliberate
act. Its logic is unit-tested rather than only exercised by the build, because a
guard whose first real execution is the failure it exists to catch is a guard
nobody has read.

**The shared directory has no package boundary and no build hop**, which is the
whole point: there is nothing between writing a primitive and seeing it on both
surfaces. The consequence worth stating plainly is that **a button exists once**
— if the landing page needs a variant the app does not have, the variant goes in
the shared component, not in a second button.

**The tokens EXTEND the app's vocabulary rather than introducing a parallel
one.** `repo:web/src/app.css` already defines a semantic layer — the
canvas/surface/raised ladder, `surface-code`, `chip`, the line trio, the
five-step fg ramp, and `ok`/`warn`/`err`/`accent` — mapped through Tailwind so
the utilities resolve to runtime variables and flip with the theme. The design
bundles use *exactly* those names and add to them: the raw ramps the semantics
are cut from, a quiet variant per status, `info`, and the focus ring. Every name
the app has, the design system has; there are none the other way round. So this
is a widening, not a second layer — which is most of the argument for sharing
the app's toolchain in the first place.

**There is now exactly one deliberate exception**, and it proves the rule by how
hard it was to justify: [[dec-chart-identity-vocabulary]] (#434) puts the chart
series colours in their own file, fenced so that nothing there is aliased into
the semantic layer and nothing there reads from it. That is not a second
vocabulary for the same job — it is a first vocabulary for a *different* job. The
ramps here are sequential by construction, and a chart legend encodes identity
rather than position, which no arrangement of one hue family can carry. Because
this layer is shared, those tokens land on all three surfaces at once even though
only the app draws charts.

The one genuine rename is the type metrics. `--font-*`, `--text-*`,
`--leading-*` and `--tracking-*` are all **Tailwind theme namespaces**, so the
runtime tokens are `--family-*`, `--size-*`, `--lh-*` and `--track-*`
(`repo:web/src/shared/styles/typography.css`) with
`repo:web/src/shared/styles/tailwind-theme.css` bridging one onto the other.
This is not fussiness: a `--leading-normal` declared at `:root` silently
redefines the `leading-normal` utility for every surface that imports the layer,
and nothing anywhere would fail.

**Authored shared from the start, because promotion never happens on time.** The
app's move was a larger migration (#403), and it landed on *this* file rather
than a copy of it — the values swap the arrangement was built for. Had the
tokens been written under `web/landing/`, the docs ticket and the app ticket
would each have had a reason to copy rather than move. #403 also moved the dark
set (#400's) from the landing stylesheet into the shared `.dark` slot once the
app became its second renderer, and the move surfaced two value repairs in the
dark CTA trio — an accidental hue shift ridden in through the shifted band
ramp, and the one quiet wash the dark pass never dialled down. Both are
documented where they are fixed (`repo:web/src/shared/styles/colors.css`) and
held by the token-contrast gate beside it
(`repo:web/src/shared/styles/contrast.test.ts`), which also pins the light/dark
CTA identity the first dark set broke by inheritance.

### The display face: neither bundle's, and not the handoff's either

Both design documents in `repo:design/README.md` specify a Rockwell-lineage
slab, in the same direction and for the same inherited reason: the design
system's Type section claims a slab is "what the mark reads as", and the site
handoff then chose a different slab from four candidates that were all
slab-flavoured because of that claim.

**The mark is not a slab.** The wordmark in
`repo:assets/laterite-social-preview.png` is a heavy geometric-humanist sans —
flat terminals, no slab serifs anywhere. The design system contradicts itself in
two other sections, describing the wordmark as a licensed geometric-humanist
face and naming Figtree 800 as what to set "laterite" in. Two of its three
statements are right; the wrong one drove the whole candidate round.

So display is Figtree and the slab is retired. This is a deliberate reversal of
the pairing as originally chosen, made once the artwork was actually examined,
and it carries a consequence that is easy to skip: **the display scale had to be
re-tuned, not carried across.** Figtree's x-height is much the larger of the
two, so at equal em size it reads bigger — every display step comes down to land
at the same apparent size. A slab's serifs bridge the gaps between letters and a
geometric sans has nothing doing that job, so tracking goes *further* negative
at the display end rather than less. And that same large x-height eats the
interline gap, so the hero's leading opens rather than staying where the slab
had it. The tool scale is untouched: it was set in the body face, and the body
face did not change.

The pages come out quieter and more engineering-neutral than the slab pairing
would have been. That is a fair trade for a validator. If the real licensed
wordmark face ever becomes available with web-embedding rights it supersedes
Figtree, and the scale gets re-tuned again.

**All three families are self-hosted and subset.** Both documents pull their
fonts from a third-party CDN. The apex is a small page whose whole pitch is that
it does not phone anywhere; a third-party font origin on the page making that
claim is the one place it cannot be true.

### Two calls the primitives forced (#406)

**The token layer grew three families, and two of them deliberately collide.**
The primitives state their control contracts in spacing, radii, elevation and
motion tokens, so "match the control metrics" was not possible without them
(`repo:web/src/shared/styles/metrics.css` and its two neighbours). `--radius-*`,
`--shadow-*`, `--ease-*` and `--spacing` are Tailwind theme namespaces, and
declaring them at `:root` redefines `rounded-md`, `shadow-md` and the spacing
scale for every surface that imports the layer. Here that is the **point** — one
radius scale, and the utilities resolve to it — which is the opposite call from
the one the type metrics needed, where the same collision was accidental and the
tokens were renamed out of the way. Both files say which they are; the failure
mode this design is guarding against is a collision nobody wrote down.

**The filled buttons are rust, not the system's `--accent`.** The design system
fills `primary` with `--accent`, which was brand brick when it was written. This
layer resolved `--accent` to maroon and gave rust its own `--cta`, so
implementing that contract literally would have made every commit button the
colour of the prose around it — the read/act split undone by following the
document that proposed it. The prop names, the variants and the metrics are the
system's; the colour role is the newer decision. `add` and `ghost` keep maroon,
because they read as links rather than commits.

**The icons are vendored, and that is the same argument as the fonts.** The
system's own `Icon` fetches each glyph from a CDN and its readme flags that as
the thing to change for an offline build. The app is a PWA with a precache, so an
icon set that 404s offline is a validator full of unlabelled buttons.
`repo:web/scripts/sync-icons.mjs` refreshes a committed working set from
`lucide-static` and fails on an icon that upstream renamed or one vendored but no
longer named, so the set cannot drift from the package it came from in either
direction.

### Four calls the page forced (#395–#400)

Building the page on this ground surfaced four places where a specification and
the shipped engine disagreed. All four resolved the same way, and it is worth
naming the principle once: **the engine is the authority, and the page follows
it.** A demo that quietly disagrees with the tool it advertises is worse than no
demo, because the reader only discovers the disagreement after installing.

**Fix runs the engine's own fixer.** #398 describes Fix as correcting the decimal
places, the abbreviation case and the missing `TRAN` group. `lat fix` on the
seeded delivery applies exactly one repair (`reformat_numeric`) and reports the
rest as findings. Hand-rolling the other two would make the page repair more than
the real tool does, so a reader who tried the same file on their own machine
would get a different answer from the demo that sold it to them. The page calls
`compute_fixes`/`apply_fixes` and says what it left. The argument #398 wanted — a
validator tells you what is wrong, a human decides what is right — survives
intact and is now true of the shipped tool rather than of the page only.

**No package name is written into markup.**
`repo:tools/gen_install_channels.py` reads each install card's name out of two
independent records and refuses to emit a card the repo contradicts. It refused
twice before shipping: #395 specified "the CLI on crates.io" (`laterite-cli` is
`publish = false`), and the wheel README's Browser row named the web app where
the grid names the npm package. Both were fixed at the record rather than by
weakening the check.

**The 820px hide-rule is retired for this page.** The design bundle contradicts
itself — the foundations artboard hides interactive demos below 820px, and the
options artboard exists precisely to find an editing pattern that works at 390px.
The editing pattern wins. A landing page whose one interactive proof is switched
off for mobile visitors is a landing page that does not land. The policy can
stand for the browser app, which is a different bar with a different job.

**Cards lift by surface step, not shadow.** #396 asks for a warm shadow on the
table cards. The shared elevation layer resolves `--shadow-card` to `none`, on
the argument recorded in its own header that a card reaching for a shadow should
have to overwrite something that says no — and #400 independently requires
exactly that in dark. Following the token keeps one answer instead of two.

## Consequences

**The apex deploy consumes a build.** `repo:.github/workflows/deploy-validator.yml`
gained a landing build step, and `assets.directory` in
`repo:web/landing/wrangler.jsonc` points at output that is gitignored — so a
deploy that skips the build ships nothing rather than something stale. The
honest 404 on unknown paths is unchanged: still no `not_found_handling`, because
the app's single-page-application setting would answer every mistyped path with
the landing page.

**The app's build is untouched.** It gained one `resolve.alias` entry and
nothing else — the PWA layer, the 404 fallback and the wasm relocation behave
exactly as they did.

**The dark set is a values change now, not a restructure.** The `.dark` selector
exists in `repo:web/src/shared/styles/colors.css` with no values in it, which is
the only reason it is there at all: #400 fills it. It is class-toggled rather
than `prefers-color-scheme` because the app already flips `.dark` on `<html>`
from a stored-else-system choice applied before first paint, and reusing that
beats building a second mechanism.

**Class-toggling is two decisions, not one, and only the first is shared.** The
`.dark` selector above switches the VALUES and lives in the shared layer, so
every surface gets it by importing. The `dark:` UTILITY VARIANT is declared per
ENTRY, and Tailwind's default for it is `prefers-color-scheme` — so an entry
that does not override it drives its dark utilities from the OS while its values
follow the class. The two agree only while a reader's OS and their stored choice
agree. #452 found the landing's entry had never declared the override (the app's
had from the start), which left the demo table with no fill for any dark-machine
reader viewing the page in light. Both entries now declare it, and
`repo:web/src/shared/styles/bundle-palette.test.ts` compiles each bundle and
fails on a dark utility that reached it through a media query — per entry,
because a new entry starts from the vendor default.

**One token is expected to be retuned per surface.** `--canvas` — the docs read
long-form on the lighter step, the landing page takes the one below it so cards
lift harder. Everything else is shared, and a surface reaching for a second
override should be read as a token missing from the layer.

**The seven-band strata ramp is the brand ramp**, steps 300–900, not a separate
`--strata-*` set that would be the same colours under new names. Band colour
encodes group identity only, never severity — severity has its own four tokens
and is carried in form as well as colour, so it survives greyscale.

**What this gates.** The shared primitives (#406) are unblocked by the alias and
the tokens together; the landing page proper (#395) and the docs (#401) follow;
the app's move onto the layer (#403) is a values swap rather than a migration
because the names already agree.

## Related

- [[validator-site]] — the app this shares a dependency set with.
- [[tech-stack-wasm]] — the browser engine the apex deliberately does not load.
- [[dec-engine-tiering]] — the app-only build machinery that must not span both.
- #394 (this design) · #334 (the landing page it clears the way for) ·
  #411 (the three-surface direction) · #395 · #400 · #401 · #403 · #406.
