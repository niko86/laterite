import { Show, type Component } from "solid-js";
import {
  reportIsOnlyFyi,
  reportSeverity,
  type SeverityCounts,
  type ValidationReport,
} from "../../lib/validator";
import { SummaryBanner as Banner, type BannerKind } from "@shared/components";

// The validate pane's reading of a report, rendered by the shared banner (#406).
//
// What used to be here was four copies of the same tinted panel — each with its
// own hand-picked palette classes — differing only in which severity they were
// dressed as. The panel moved to the shared primitive, which the fix and tools
// panes want too; what stays is the part that is genuinely about a
// ValidationReport: which verdict it is, and how to say it.

const RESOLUTION_BLURB: Record<string, string> = {
  forced: "forced by you",
  exact: "exact TRAN_AGS match",
  guessed: "nearest bundled patch",
  fallback: "fallback (TRAN_AGS missing/unknown)",
};

const plural = (n: number, w: string) =>
  `${n.toLocaleString()} ${w}${n === 1 ? "" : "s"}`;

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
  // A file whose ONLY findings are FYI shouldn't look like a failure — warn,
  // not err (see reportIsOnlyFyi for the severity-default rationale).
  const onlyFyi = () => reportIsOnlyFyi(props.report);
  // Headline for the failure banner: the per-severity split ("36 errors · 14
  // informational") when the report is uncapped, else the true grand total
  // (the split would undercount a per-rule-capped report).
  const headline = () => {
    const { counts, exact } = reportSeverity(props.report);
    return exact
      ? breakdown(counts)
      : plural(props.report.finding_count, "finding");
  };
  const against = () =>
    `Validated against AGS ${props.report.dict_version} — ${
      RESOLUTION_BLURB[props.report.resolution] ?? props.report.resolution
    }`;
  const kind = (): BannerKind =>
    props.report.ok ? "ok" : onlyFyi() ? "warn" : "err";

  return (
    <Show
      when={!props.report.error}
      fallback={
        <Banner
          kind="warn"
          headline="Could not validate"
          detail={props.report.error?.message}
          note={`(${props.report.error?.kind ?? ""})`}
        />
      }
    >
      <Banner
        kind={kind()}
        headline={
          <Show when={!props.report.ok} fallback="Clean — 0 findings">
            <Show
              when={!onlyFyi()}
              fallback={`${props.report.finding_count.toLocaleString()} informational (FYI) finding${
                props.report.finding_count === 1 ? "" : "s"
              } — no errors or warnings`}
            >
              {headline()}
            </Show>
          </Show>
        }
        detail={against()}
        note={
          <Show
            when={!props.report.ok && onlyFyi()}
            fallback={
              <Show
                when={
                  !props.report.ok &&
                  props.report.shown_count < props.report.finding_count
                }
              >
                Showing the first {props.report.shown_count.toLocaleString()} of{" "}
                {props.report.finding_count.toLocaleString()} findings (capped
                per rule to keep the page responsive). Download the full report
                below, or for very large files use the{" "}
                <code class="mono">lat-check</code> CLI.
              </Show>
            }
          >
            FYI findings are hidden by default — switch on{" "}
            <span class="mono">fyi</span> in the severity filter to see them.
          </Show>
        }
      />
    </Show>
  );
};
