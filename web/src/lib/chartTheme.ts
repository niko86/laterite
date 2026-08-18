import { theme } from "../shared/lib/theme";

// Chart theming from the shared tokens (#410). ECharts is the one renderer
// that cannot inherit CSS custom properties — it paints a canvas — so the
// tokens are READ at option-build time (the CoordinateMap's #404 pattern) and
// re-read when the theme flips: callers touch `theme()` through here, so a
// memo that spreads these values rebuilds on toggle.
//
// THE PALETTE IS TOKEN STEPS, AND THE DARK HALF IS A KNOWN COMPROMISE. The
// sequence was chosen against the dataviz palette validator — the instrument
// to re-run after any ramp edit, never prose to trust — which seated a
// passing head trio on the light surface and could not on the dark one: the
// brand ramp is sequential by construction, and the dark surface squeezes
// adjacent steps together. The extension to six keeps that recorded head
// (500/300/700) and interleaves the remaining band so every added neighbour
// pair clears the floors the head pair cannot; #434 records the verdicts and
// owns the token-extension decision. Step 200 is excluded: dark mode does not
// re-map it, so it collides with dark 300. Beyond six values ECharts cycles
// and colours repeat — the honest fix is folding into "Other", a behavioural
// change this presentation ticket doesn't take. Assignment is ECharts' series
// order; entity-stable colour across filters would need the builder to pin
// colours per value, not taken here either.
const readVar = (name: string): string =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim();

export interface ChartTokens {
  palette: string[];
  fgSoft: string;
  fgDim: string;
  line: string;
  lineSubtle: string;
  familyUi: string;
  tooltipBg: string;
  tooltipBorder: string;
  tooltipFg: string;
}

export function chartTokens(): ChartTokens {
  // Reactive dependency, not a value: the tokens themselves are read from the
  // live cascade, which `.dark` on <html> has already flipped by the time the
  // signal fires.
  void theme();
  return {
    palette: [
      readVar("--laterite-500"),
      readVar("--laterite-300"),
      readVar("--laterite-700"),
      readVar("--laterite-400"),
      readVar("--laterite-900"),
      readVar("--laterite-600"),
    ],
    fgSoft: readVar("--fg-soft"),
    fgDim: readVar("--fg-dim"),
    line: readVar("--line"),
    lineSubtle: readVar("--line-subtle"),
    familyUi: readVar("--family-ui"),
    // The app's own dark-object idiom (toast/tooltip): the deep-brand fill
    // with the fixed on-CTA foreground and the white-alpha hairline.
    tooltipBg: readVar("--laterite-900"),
    tooltipBorder: "rgba(255, 255, 255, 0.18)",
    tooltipFg: readVar("--fg-on-cta"),
  };
}
