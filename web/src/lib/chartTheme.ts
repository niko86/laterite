import { theme } from "../shared/lib/theme";
import { SLOT_COUNT } from "../shared/styles/chartSlots";

// Chart theming from the shared tokens (#410). ECharts is the one renderer
// that cannot inherit CSS custom properties — it paints a canvas — so the
// tokens are READ at option-build time (the CoordinateMap's #404 pattern) and
// re-read when the theme flips: callers touch `theme()` through here, so a
// memo that spreads these values rebuilds on toggle.
//
// The palette is the FENCED chart vocabulary, not the brand ramp (#434). The
// ramp could never do this job: it is a sequential scale, and a categorical
// channel needs differing hues rather than differing steps. What is here is the
// numbered slots plus the neutral the tail folds into, whose values and their
// derivation live beside them in shared/styles/charts.css — read that file
// before changing this one, and re-run the separation gate afterwards rather
// than trusting either.
//
// This is a READ, and how far into the palette a chart form may spend is not
// something a read can decide: that belongs to the builder, and it takes both
// ceilings from `shared/styles/chartSlots.ts` (#445). Counting the slots here
// rather than listing them is what keeps the two from being two numbers.
const readVar = (name: string): string =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim();

export interface ChartTokens {
  palette: string[];
  /** Where the series past the form's cap go — a neutral, never a sixth hue. */
  other: string;
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
    palette: Array.from({ length: SLOT_COUNT }, (_, i) =>
      readVar(`--chart-${i + 1}`),
    ),
    other: readVar("--chart-other"),
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
