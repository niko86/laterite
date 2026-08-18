import { theme } from "../shared/lib/theme";

// Chart theming from the shared tokens (#410). ECharts is the one renderer
// that cannot inherit CSS custom properties — it paints a canvas — so the
// tokens are READ at option-build time (the CoordinateMap's #404 pattern) and
// re-read when the theme flips: callers touch `theme()` through here, so a
// memo that spreads these values rebuilds on toggle.
//
// THE PALETTE IS TOKEN STEPS, VALIDATED, AND THE DARK HALF IS A KNOWN
// COMPROMISE. Light `laterite-500/300/700` passes the dataviz validator's six
// checks on the light surface (one sanctioned contrast WARN on sand — the
// legend and the results table beneath the chart are its required relief).
// The same steps resolve through the dark-shifted ramp for dark, where the
// validator shows no fully-passing sequence EXISTS in this vocabulary: the
// brand ramp is sequential by construction, and the dark surface squeezes it
// below the normal-vision separation floor. Maximum-separation steps ship;
// the token-extension decision is #434. Fixed assignment, never re-ranked:
// colour follows the series, not its position after a filter.
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
