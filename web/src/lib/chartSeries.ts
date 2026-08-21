import { scalarText } from "./duckTypes";
import type { ChartFold, ChartType } from "./sqlgen";
import { ALL_PAIRS_CAP, SLOT_COUNT } from "../shared/styles/chartSlots";

// Turning fetched rows into ECharts series, with the colour-by split BOUNDED
// and each colour pinned to a value rather than to a position (#445).
//
// Three defects share this one seam, and all three were invisible from the
// component:
//
//   - Nothing bounded the split. The colour-by control offers any column, so a
//     real delivery — LOCA_TYPE, a geology code, a sample type — routinely has
//     more distinct values than the palette has slots, and past the last slot
//     ECharts CYCLES: it neither fails nor generates a hue, it repaints series
//     one's colour. Two series, one colour, on the same axes, under a legend
//     saying they differ.
//   - The default form is scatter, which is validated only to `ALL_PAIRS_CAP`,
//     not to the whole sequence. So the COMMON case was already drawing pairs
//     the separation gate had never checked for it.
//   - Colour followed series order, which follows first appearance in the
//     fetched rows. Change the row set and the survivors repaint, silently
//     invalidating the mapping the reader carried over from the last view.
//
// This module is pure so all of that is testable: the web unit lane runs
// `environment: "node"` and cannot mount a component, so below e2e this is the
// only altitude the behaviour exists at. The SQL composer beside it is the
// established shape.

/** One plotted point: X is a label or a number depending on the axis. */
type ChartPoint = [unknown, number];

interface ChartSeries {
  name?: string;
  type: ChartType;
  data: ChartPoint[];
  /** Never omitted, and never the root `color` array — see `assembleSeries`. */
  itemStyle: { color: string };
  symbolSize?: number;
  large?: boolean;
  largeThreshold?: number;
  sampling?: string;
}

export interface AssembleOpts {
  /** The plotted rows as the chart query returns them: `x`, `y`, and `c` when
   *  a colour-by is set. */
  rows: readonly Record<string, unknown>[];
  /** The colour-by values that survive, most rows first, as `chartRankSql`
   *  ranked them over the whole table. Empty means no colour-by. Anything
   *  past the form's cap — here, not in the query — folds. */
  ranked: readonly string[];
  chartType: ChartType;
  /** X is plotted as a label rather than a number (bar, or a non-numeric X). */
  categoricalX: boolean;
  palette: readonly string[];
  /** The neutral the fold is painted; never one of the slots. */
  other: string;
}

/** The legend name for the folded tail. */
const OTHER = "Other";

const num = (v: unknown): number =>
  typeof v === "bigint" ? Number(v) : typeof v === "number" ? v : Number(v);

/** How far into the slot sequence a form may spend.
 *
 *  Scatter stops at the head validated on the ALL-PAIRS pairlist, because any
 *  two of its marks can land side by side; bar and line are validated
 *  adjacent-only, since only touching marks have to separate, so they may use
 *  the whole sequence. Both numbers come from the token layer rather than from
 *  here — one definition, and the gate is what validates it, including that
 *  the head is not longer than the sequence it is the head of.
 *
 *  It takes no slot count. The probe's LIMIT and the plotted split are the same
 *  cap, and a parameter is a seam where they could be handed different ones. */
export const seriesCap = (chartType: ChartType): number =>
  chartType === "scatter" ? ALL_PAIRS_CAP : SLOT_COUNT;

/** `OTHER`, unless a surviving value already carries that name.
 *
 *  "Other" is a real value in more than one AGS abbreviation list, and two
 *  ECharts series sharing a name share a legend entry — so the fold steps
 *  aside rather than colliding with a value it is not. Same idiom as
 *  `dedupeOut` in the SQL composer.
 *
 *  Exported because the aggregating bar folds in SQL (#457): that query
 *  materialises this label as DATA, so the literal it emits and the legend
 *  entry here have to be one function's answer over one list. Two guesses at
 *  it would put the fold's own rows in a series named something else. */
export function foldLabel(survivors: readonly string[]): string {
  let label = OTHER;
  for (let n = 2; survivors.includes(label); n++) label = `${OTHER} (${n})`;
  return label;
}

/** The fold an aggregating bar's plot query may be composed with, given the
 *  ranking probe's state — or `undefined`, meaning there is nothing to compose
 *  yet and the query must wait (#457).
 *
 *  Extracted from the component because the guard is invisible there, and was
 *  wrong for exactly that reason. A Solid resource keeps its PREVIOUS value
 *  while refetching — `state: "refreshing"`, `loading: true`, the old value
 *  still readable — so "have we an answer?" cannot be asked of the value alone.
 *  It answers with the last colour column's survivors, an `IN (…)` list naming
 *  a column the query no longer mentions; and before any colour is picked at
 *  all it answers with the empty list the probe's own empty query resolves to,
 *  which composes the unfolded query against exactly the high-cardinality
 *  column the fold exists to bound. Loading is not an answer. */
export function foldFor(probe: {
  loading: boolean;
  /** `undefined` when the probe failed — see the component's error accessor. */
  values: readonly string[] | undefined;
}): ChartFold | undefined {
  if (probe.loading || probe.values === undefined) return undefined;
  return { keep: probe.values, label: foldLabel(probe.values) };
}

/** Rows + ranked values + form → the ECharts series array.
 *
 *  Every series carries its own `itemStyle.color`, and the option this feeds
 *  sets no root `color`: that array is exactly the palette ECharts walks and
 *  wraps around, so leaving it in place would leave the cycle one un-pinned
 *  series away.
 *
 *  Colour comes from the value's RANK, and what that buys is precise. Nothing
 *  about the plotted SLICE can repaint anything: the row cap, and the arbitrary
 *  rows an un-ordered LIMIT happens to return, are invisible to the ranking.
 *  What CAN reorder it is a change to the population the probe ranks over —
 *  adding a join that fans base rows out, or picking a different X/Y and so a
 *  different `IS NOT NULL` filter. That is the deliberate cost of ranking over
 *  exactly the rows the chart draws (`chartRankSql`): "Other" is then a claim
 *  about the plotted delivery rather than about some other population. */
export function assembleSeries(o: AssembleOpts): ChartSeries[] {
  // No colour-by is one series, which is the same shape as a single ranked
  // value: the empty string — which is also what a NULL colour reads as, so
  // the LEFT-join case that leaves a stratum unmatched takes this path too.
  const ranked = o.ranked.length > 0 ? o.ranked : [""];
  const cap = seriesCap(o.chartType);

  // Driven by the PALETTE rather than by the values, so a slot colour is
  // definite and the cap is applied once. Fewer values than slots is the
  // ordinary case and simply leaves slots unspent.
  const pinned: { value: string; colour: string }[] = [];
  o.palette.slice(0, cap).forEach((colour, i) => {
    const value = ranked[i];
    if (value !== undefined) pinned.push({ value, colour });
  });
  const slotOf = new Map(pinned.map((p, i) => [p.value, i]));

  // Bucketed by SLOT, not by name: `foldLabel` keeps the legend unambiguous,
  // but only an index can keep a value named "Other" out of the fold's DATA.
  //
  // Points are POOLED, never merged, and that is the whole of what this loop
  // may do. For a point cloud pooling IS the fold — scatter and line have
  // nothing to merge. For an aggregated bar it was wrong: the query had already
  // grouped by (x, colour), so folding several values left the tail with a bar
  // per folded value at the same category, at the same width, one over another.
  // The merge was never available here — the right one is the aggregate's own,
  // and avg has none that is honest, since a mean of means is not the mean
  // without each group's row count, which this row set does not carry. So that
  // path now folds in SQL instead, inside the GROUP BY, where the aggregate
  // runs over the merged group's raw rows and avg is as correct as sum (#457).
  // The label arrives here as an ordinary colour value, absent from the
  // ranking, and buckets to the fold below with no special case.
  const FOLD = -1;
  const buckets = new Map<number, ChartPoint[]>();
  for (const row of o.rows) {
    const slot = slotOf.get(scalarText(row.c)) ?? FOLD;
    let points = buckets.get(slot);
    if (points === undefined) {
      points = [];
      buckets.set(slot, points);
    }
    points.push([o.categoricalX ? scalarText(row.x) : num(row.x), num(row.y)]);
  }

  // Perf knobs for big series on weak hardware: `large` batches scatter points
  // into one buffered draw (vs an animated per-symbol render that melts an
  // integrated GPU); LTTB down-samples a dense line without changing its
  // visible shape.
  const knobs =
    o.chartType === "scatter"
      ? { symbolSize: 8, large: true, largeThreshold: 2000 }
      : o.chartType === "line"
        ? { sampling: "lttb" }
        : {};

  const out: ChartSeries[] = [];
  const emit = (slot: number, name: string, colour: string) => {
    const data = buckets.get(slot);
    // A value can rank high over the whole table and still be absent from the
    // capped slice actually plotted. Its slot stays its own; it just has
    // nothing to draw, and an empty legend entry would claim otherwise.
    if (data === undefined) return;
    out.push({
      name: name || undefined,
      type: o.chartType,
      itemStyle: { color: colour },
      data,
      ...knobs,
    });
  };
  pinned.forEach((p, i) => {
    emit(i, p.value, p.colour);
  });
  // Named from the RANKING, not from the slots it filled. `foldFor` labels the
  // SQL literal off the same list, and the two have to be one answer over one
  // list — they coincide only while the palette is exactly as long as the cap
  // and the probe's LIMIT is the same number. Shrink the palette and `pinned`
  // becomes a shorter list, whose `foldLabel` can differ from the string the
  // query already wrote into the rows.
  emit(FOLD, foldLabel(o.ranked), o.other);
  return out;
}
