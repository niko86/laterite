import { Show, type Component } from "solid-js";
import {
  reportIsOnlyFyi,
  reportSeverity,
  type SeverityCounts,
  type ValidationReport,
} from "../../lib/validator";

const RESOLUTION_BLURB: Record<string, string> = {
  forced: "forced by you",
  exact: "exact TRAN_AGS match",
  guessed: "nearest bundled patch",
  fallback: "fallback (TRAN_AGS missing/unknown)",
};

const plural = (n: number, w: string) => `${n.toLocaleString()} ${w}${n === 1 ? "" : "s"}`;

/** "36 errors · 14 informational" — only the non-zero severities, so a pure
 *  error file just reads "36 errors". */
const breakdown = (c: SeverityCounts): string => {
  const parts: string[] = [];
  if (c.error) parts.push(plural(c.error, "error"));
  if (c.warning) parts.push(plural(c.warning, "warning"));
  if (c.fyi) parts.push(`${c.fyi.toLocaleString()} informational`);
  return parts.join(" · ");
};

export const SummaryBanner: Component<{
  report: ValidationReport;
  name: string;
}> = (props) => {
  // FYI findings are informational (extended-ASCII, etc.), not violations.
  // A file whose ONLY findings are FYI shouldn't look like a failure — amber,
  // not red (see reportIsOnlyFyi for the severity-default rationale).
  const onlyFyi = () => reportIsOnlyFyi(props.report);
  // Headline for the red banner: the per-severity split ("36 errors · 14
  // informational") when the report is uncapped, else the true grand total
  // (the split would undercount a per-rule-capped report).
  const headline = () => {
    const { counts, exact } = reportSeverity(props.report);
    return exact
      ? breakdown(counts)
      : `${plural(props.report.finding_count, "finding")}`;
  };
  return (
    <Show
      when={!props.report.error}
      fallback={
        <div class="rounded-lg border border-amber-600/50 bg-amber-500/10 p-4">
          <p class="font-medium text-warn">Could not validate</p>
          <p class="mt-1 text-sm text-warn">
            {props.report.error?.message}
          </p>
          <p class="mt-1 text-xs text-warn">
            ({props.report.error?.kind})
          </p>
        </div>
      }
    >
      <Show
        when={props.report.ok}
        fallback={
          <Show
            when={!onlyFyi()}
            fallback={
              <div class="rounded-lg border border-amber-600/50 bg-amber-500/10 p-4">
                <p class="font-medium text-warn">
                  ⓘ {props.report.finding_count.toLocaleString()} informational
                  (FYI) finding
                  {props.report.finding_count === 1 ? "" : "s"} — no errors or
                  warnings
                </p>
                <p class="mt-1 text-sm text-fg-soft">
                  Validated against AGS {props.report.dict_version} —{" "}
                  {RESOLUTION_BLURB[props.report.resolution] ??
                    props.report.resolution}
                </p>
                <p class="mt-1 text-xs text-fg-dim">
                  FYI findings are hidden by default — switch on{" "}
                  <span class="mono">fyi</span> in the severity filter to see
                  them.
                </p>
              </div>
            }
          >
            <div class="rounded-lg border border-red-600/50 bg-red-500/10 p-4">
              <p class="font-medium text-err">✗ {headline()}</p>
              <p class="mt-1 text-sm text-fg-soft">
                Validated against AGS {props.report.dict_version} —{" "}
                {RESOLUTION_BLURB[props.report.resolution] ??
                  props.report.resolution}
              </p>
              <Show when={props.report.shown_count < props.report.finding_count}>
                <p class="mt-2 text-xs text-warn">
                  Showing the first {props.report.shown_count.toLocaleString()}{" "}
                  of {props.report.finding_count.toLocaleString()} findings
                  (capped per rule to keep the page responsive). Download the
                  full report below, or for very large files use the{" "}
                  <code class="mono">lat-check</code> CLI.
                </p>
              </Show>
            </div>
          </Show>
        }
      >
        <div class="rounded-lg border border-emerald-600/50 bg-emerald-500/10 p-4">
          <p class="font-medium text-ok">✓ Clean — 0 findings</p>
          <p class="mt-1 text-sm text-fg-soft">
            Validated against AGS {props.report.dict_version} —{" "}
            {RESOLUTION_BLURB[props.report.resolution] ??
              props.report.resolution}
          </p>
        </div>
      </Show>
    </Show>
  );
};
