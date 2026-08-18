// The table-of-contents strata rail (#401).
//
// The apex's signature is a borehole rail: a strata strip down the page, veiled
// below the reading position, with a steel probe at it, so the layers uncover as
// you descend. The docs take the HAIRLINE DOSE of it — 6px on the table of
// contents' left edge, four bands, no depth pill and no numbers. It is an echo
// of the instrument, not a second instrument; a docs page has no depth, and a
// readout claiming one would be a decoration pretending to be data.
//
// Vanilla, like catalogue.js beside it, and for the same reason: the docs load
// no third-party origin.
//
// Everything visual is in stylesheets/laterite.css. This file owns two numbers —
// how far down the page you are, and where that puts the probe.

// One listener for the life of the document, retargeted on navigation. Material's
// `document$` re-fires on every instant navigation, so subscribing a fresh
// scroll listener there would stack one per page visited.
let rail = null;
let veil = null;
let probe = null;

function progress() {
  const scrollable = document.documentElement.scrollHeight - window.innerHeight;
  if (scrollable <= 0) return 0;
  return Math.min(1, Math.max(0, window.scrollY / scrollable));
}

function position() {
  if (!rail) return;
  const p = progress();
  const height = rail.clientHeight;
  // The veil covers what is still BELOW you, so it shrinks as you descend.
  veil.style.height = `${(1 - p) * height}px`;
  probe.style.top = `${p * height}px`;
}

function mount() {
  rail = veil = probe = null;
  const nav = document.querySelector(".md-sidebar--secondary .md-nav--secondary");
  // A page whose only heading is its h1 gets an EMPTY secondary nav rather than
  // none at all — Material still renders the <nav>, just with no list inside. A
  // rail there is a zero-height element tracking a table of contents that is not
  // on the page, so the count is what decides, not the container.
  if (!nav || !nav.querySelector(".md-nav__item")) return;

  const existing = nav.querySelector(".md-rail");
  if (existing) {
    rail = existing;
    veil = rail.querySelector(".md-rail__veil");
    probe = rail.querySelector(".md-rail__probe");
    position();
    return;
  }

  rail = document.createElement("div");
  rail.className = "md-rail";
  // Decorative: the strip repeats information the TOC already carries in text,
  // so a screen reader gains nothing and loses its place.
  rail.setAttribute("aria-hidden", "true");

  veil = document.createElement("div");
  veil.className = "md-rail__veil";
  probe = document.createElement("div");
  probe.className = "md-rail__probe";

  rail.append(veil, probe);
  nav.prepend(rail);
  position();
}

window.addEventListener("scroll", position, { passive: true });
window.addEventListener("resize", position, { passive: true });

// Prefer Material's `document$` observable (re-fires on instant navigation);
// fall back to a one-shot DOM-ready hook if it is somehow absent.
if (typeof document$ !== "undefined") {
  document$.subscribe(mount);
} else if (document.readyState !== "loading") {
  mount();
} else {
  document.addEventListener("DOMContentLoaded", mount);
}
