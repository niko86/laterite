import { theme } from "../shared/lib/theme";

// Chart theming from the shared tokens (#410). ECharts is the one renderer
// that cannot inherit CSS custom properties — it paints a canvas — so the
// tokens are READ at option-build time (the CoordinateMap's #404 pattern) and
// re-read when the theme flips: callers touch `theme()` through here, so a
// memo that spreads these values rebuilds on toggle.
//
// The palette is the FENCED chart vocabulary, not the brand ramp (#434). The
// ramp could never do this job: it is a sequential scale, and a categorical
// channel needs differing hues rather than differing steps. What is here is
// `--chart-1..5`, whose values, their derivation and their cap all live beside
// them in shared/styles/charts.css — read that file before changing this list,
// and re-run the separation gate afterwards rather than trusting either.
//
// TWO LIMITS THIS FILE CANNOT ENFORCE, both real:
//   - Slots 1-3 are the set validated for scatter, where any two marks can sit
//     side by side; all five are only validated where marks merely touch. This
//     list hands the full five to every chart form, so a scatter with four or
//     more series is using a pair that was never checked for it.
//   - Past five series ECharts cycles and colours repeat outright.
// Both want the same fix — cap the series count and fold the tail into "Other"
// — which is behavioural and belongs to the builder, not to a token read.
// Assignment is still ECharts' series order, so a filter that changes the
// series count repaints the survivors; entity-stable colour would need the
// builder to pin colours per value.
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
      readVar("--chart-1"),
      readVar("--chart-2"),
      readVar("--chart-3"),
      readVar("--chart-4"),
      readVar("--chart-5"),
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
