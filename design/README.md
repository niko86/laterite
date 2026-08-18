# Design handoffs

Reference material for the three-surface visual direction (tracking **#411**).
These are the two bundles the direction was chosen from, given a home here by
**#394** so the tickets that follow have something to point at.

**They are references, not sources.** Nothing builds from this directory and
nothing here is imported. The shipped artefact is the shared token layer at
`web/src/shared/styles/`, and where the two disagree, the token layer is what
runs.

## What is here

| Directory | What it is |
|---|---|
| `laterite-site-direction/` | The site direction handoff — laterite.dev (light **3a** / dark **3b**) and docs.laterite.dev (**4a**), chosen from four colour/type pairings and two rail concepts. Its `README.md` is the written spec; `Demo Site Pairings.dc.html` is the canvas. |
| `laterite-mobile-design-system/` | The mobile design system — the 2a frozen-key + row-carousel editing model, the mobile page (`Demo Site Mobile.dc.html`), and the foundations artboard. Carries the same `_ds/` token bundle. |

Both carry a `_ds/laterite-design-system-<uuid>/` directory holding
`styles.css` + `tokens/*.css`. **The two copies are byte-identical** — one
export of the same design system — so cite either.

The `.dc.html` files are interactive: open one in a browser and scroll inside a
frame. They reference `_ds/` and `support.js` by relative path, which is why
both bundles are committed exactly as delivered rather than deduplicated to a
single `_ds/`. The only change made to either was deleting a stray `.thumbnail`
preview image.

## Where the shipped tokens deliberately differ

Three departures, all decided after the bundles were produced. They are
recorded in `ags-wiki/design/dec-landing-build-shared-tokens.md`; this is the
short list so a reader comparing the two files is not left guessing.

- **The display face is Figtree, not a slab.** Both documents specify a
  Rockwell-lineage slab (`_ds/…/tokens/fonts.css` picks Rokkitt; the site
  handoff then picks Zilla Slab from four candidates that were all slab-flavoured
  for the same inherited reason). The reasoning given is that a slab is what the
  mark reads as, and the mark is not a slab. **The display scale was re-tuned to
  the geometric sans, not carried across.**
- **`--accent` is maroon `#611a1e`, not brand brick `#9b3932`.** The design
  system sets brick; the site direction resolved links, headlines and active nav
  to maroon, and that is the one that ships. Rust `#ce5640` is a separate token
  (`--cta`) — maroon reads, rust acts.
- **The fonts are self-hosted.** Both documents load theirs from the Google
  Fonts CDN, and a `fonts.googleapis.com` URL survives in the bundled
  `tokens/fonts.css` here. Nothing on any laterite surface fetches it: the apex's
  whole pitch is that it does not phone anywhere.

## Two things the handoffs get wrong for this repo

Neither is a design judgement — both are about the target codebase, and the
tickets carry the corrections.

- The site handoff targets a **Svelte** template in a separate design-system
  repo. That template is reference material, and the design system's own readme
  documents this app as **Solid** and took its component inventory from it. The
  stack is the one already here.
- The design system's mobile policy hides interactive demos below 820px. That
  policy governs marketing surfaces and is **retired for the landing demo**,
  which is editable at 390px.
