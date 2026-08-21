import { describe, expect, it } from "vitest";
import {
  assembleSeries,
  foldFor,
  foldLabel,
  seriesCap,
  type AssembleOpts,
} from "./chartSeries";
import { ALL_PAIRS_CAP, SLOT_COUNT } from "../shared/styles/chartSlots";

// The colour-by split, at the only altitude below e2e it exists at.
//
// Every case here was reachable in the shipped app: the colour-by control
// offers any column, and the default form is the one with the LOWER cap.

const PALETTE = Array.from({ length: SLOT_COUNT }, (_, i) => `#slot${i + 1}`);
const OTHER_COLOUR = "#neutral";

/** Rows shaped as the chart query returns them. */
const rows = (...cs: string[]): Record<string, unknown>[] =>
  cs.map((c, i) => ({ x: i, y: i * 2, c }));

function assemble(over: Partial<AssembleOpts> = {}) {
  return assembleSeries({
    rows: [],
    ranked: [],
    chartType: "scatter",
    categoricalX: false,
    palette: PALETTE,
    other: OTHER_COLOUR,
    ...over,
  });
}

const names = (s: { name?: string }[]) => s.map((x) => x.name);
const colours = (s: { itemStyle: { color: string } }[]) =>
  s.map((x) => x.itemStyle.color);

describe("seriesCap", () => {
  it("stops scatter at the all-pairs-validated head", () => {
    // Scatter marks can land anywhere near each other, so the pairs that were
    // only checked for ADJACENCY are not available to it.
    expect(seriesCap("scatter")).toBe(ALL_PAIRS_CAP);
  });

  it("lets bar and line spend the whole sequence", () => {
    expect(seriesCap("bar")).toBe(SLOT_COUNT);
    expect(seriesCap("line")).toBe(SLOT_COUNT);
  });
});

describe("assembleSeries — the cap", () => {
  it("never colours more scatter series than the all-pairs head", () => {
    const ranked = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
    const series = assemble({ rows: rows(...ranked), ranked });
    // One per surviving value, plus the fold — and no more, whatever the
    // column's cardinality. Past this ECharts would have cycled.
    expect(series).toHaveLength(ALL_PAIRS_CAP + 1);
    expect(colours(series)).toEqual([
      ...PALETTE.slice(0, ALL_PAIRS_CAP),
      OTHER_COLOUR,
    ]);
  });

  it("lets bar spend every slot before folding", () => {
    const ranked = ["a", "b", "c", "d", "e", "f", "g"];
    const series = assemble({
      rows: rows(...ranked),
      ranked,
      chartType: "bar",
      categoricalX: true,
    });
    expect(series).toHaveLength(SLOT_COUNT + 1);
    expect(colours(series)).toEqual([...PALETTE, OTHER_COLOUR]);
  });

  it("folds the tail into one neutral series, named in the legend", () => {
    const ranked = ["a", "b", "c", "d", "e"];
    // Two rows each for the two values past the scatter cap.
    const series = assemble({
      rows: rows("a", "b", "c", "d", "e", "d", "e"),
      ranked,
    });
    const fold = series.at(-1);
    expect(fold?.name).toBe("Other");
    expect(fold?.itemStyle.color).toBe(OTHER_COLOUR);
    // One series, not one per folded value — and it carries all their points.
    expect(fold?.data).toHaveLength(4);
    expect(
      series.filter((s) => s.itemStyle.color === OTHER_COLOUR),
    ).toHaveLength(1);
  });

  it("takes ONE folded point per category on an aggregated bar", () => {
    // This replaces the pin that recorded the opposite (#457). The aggregating
    // bar query GROUPs BY (x, colour), so folding here left the tail with one
    // point per folded value per category — several bars at the same x, at the
    // same width, one over another, reading as a single bar with no sign that
    // more than one value was under it. That fold now happens in SQL, inside
    // the GROUP BY, so the aggregate is computed over the merged group's own
    // rows and the label arrives as an ORDINARY colour value: absent from the
    // ranking, so it buckets to the fold with no special case here.
    const ranked = ["a", "b", "c", "d", "e"];
    const series = assemble({
      chartType: "bar",
      categoricalX: true,
      ranked,
      rows: [
        { x: "X1", y: 60, c: "Other" },
        { x: "X2", y: 10, c: "Other" },
        { x: "X1", y: 1, c: "a" },
      ],
    });
    const fold = series.at(-1);
    expect(fold?.name).toBe("Other");
    expect(fold?.itemStyle.color).toBe(OTHER_COLOUR);
    expect(fold?.data).toEqual([
      ["X1", 60],
      ["X2", 10],
    ]);
  });

  it("names the fold what the query folded to, on the same input", () => {
    // The legend entry and the literal the SQL emits are ONE string: the query
    // materialises the label as data, so a second guess at it would put the
    // fold's own rows in a series named something else. `foldLabel` is the
    // single answer, and this is the case where it steps aside.
    const ranked = ["Other", "b", "c", "d", "e"];
    const label = foldLabel(ranked);
    expect(label).toBe("Other (2)");
    const series = assemble({
      chartType: "bar",
      categoricalX: true,
      ranked,
      rows: [
        { x: "X1", y: 7, c: label },
        { x: "X1", y: 3, c: "Other" },
      ],
    });
    expect(names(series)).toEqual(["Other", "Other (2)"]);
    // The real value's row stayed with the value; the fold's with the fold.
    expect(series[0]?.data).toEqual([["X1", 3]]);
    expect(series[1]?.data).toEqual([["X1", 7]]);
  });

  it("emits no fold when the values fit", () => {
    const ranked = ["a", "b"];
    const series = assemble({ rows: rows("a", "b", "a"), ranked });
    expect(names(series)).toEqual(["a", "b"]);
    expect(colours(series)).toEqual(PALETTE.slice(0, 2));
  });

  it("steps the fold's label aside from a value that is itself 'Other'", () => {
    // "Other" is a real value in more than one AGS abbreviation list. Two
    // series with one name share a legend entry.
    const ranked = ["Other", "b", "c", "d"];
    const series = assemble({ rows: rows("Other", "b", "c", "d"), ranked });
    expect(names(series)).toEqual(["Other", "b", "c", "Other (2)"]);
    // And the value's OWN rows stayed with the value, not with the fold.
    expect(series[0]?.data).toHaveLength(1);
    expect(series[0]?.itemStyle.color).toBe(PALETTE[0]);
  });
});

// The guard that decides whether the aggregating bar has a query to compose at
// all. It lives here rather than in the component because it was WRONG there
// and invisible: the component could only be read as "we wait for the probe",
// which is not what the code did.
describe("foldFor", () => {
  it("treats a REFETCHING probe as no answer, however good its value looks", () => {
    // The regression. A Solid resource keeps its previous value while
    // refetching, so the value alone is the LAST colour column's survivors —
    // an `IN (…)` list naming a column the query no longer mentions. Before
    // any colour is picked it is worse: the empty list the probe's own empty
    // query resolves to, which composes the UNFOLDED query against exactly the
    // high-cardinality column the fold exists to bound.
    expect(foldFor({ loading: true, values: ["a", "b"] })).toBeUndefined();
    expect(foldFor({ loading: true, values: [] })).toBeUndefined();
  });

  it("treats a FAILED probe as no answer", () => {
    expect(foldFor({ loading: false, values: undefined })).toBeUndefined();
  });

  it("hands back the survivors and the label from ONE list", () => {
    // The query materialises the label as data, so this and the legend entry
    // below have to be one function's answer over one list.
    expect(foldFor({ loading: false, values: ["a", "b"] })).toEqual({
      keep: ["a", "b"],
      label: "Other",
    });
    expect(foldFor({ loading: false, values: ["Other", "b"] })).toEqual({
      keep: ["Other", "b"],
      label: "Other (2)",
    });
    // An empty answer is still an ANSWER — the probe found no rows, and the
    // composer has a shape for that.
    expect(foldFor({ loading: false, values: [] })).toEqual({
      keep: [],
      label: "Other",
    });
  });

  it("names the fold what the assembler will name it, off a SHORT palette", () => {
    // The two used to be computed from different lists — the assembler from
    // the slots it filled, the composer from the whole probe answer — which
    // agree only while the palette is exactly as long as the cap. Below that
    // they diverge, and the series carrying the query's own rows is named
    // something the query never wrote.
    const ranked = ["a", "b", "Other"];
    const fold = foldFor({ loading: false, values: ranked });
    const series = assembleSeries({
      rows: [{ x: "X1", y: 1, c: fold?.label }],
      ranked,
      chartType: "bar",
      categoricalX: true,
      palette: PALETTE.slice(0, 2),
      other: OTHER_COLOUR,
    });
    expect(series.at(-1)?.name).toBe(fold?.label);
    expect(series.at(-1)?.itemStyle.color).toBe(OTHER_COLOUR);
  });
});

describe("assembleSeries — colour follows the value, not the position", () => {
  it("keeps a value's colour when the row set changes under it", () => {
    // The ranking is computed over the whole table, so filtering the plotted
    // rows cannot move a survivor's slot. Before this, series order — first
    // appearance in the fetched rows — decided colour, and any filter that
    // dropped a value repainted every value after it.
    const ranked = ["CLAY", "SAND", "PEAT"];
    const all = assemble({ rows: rows("CLAY", "SAND", "PEAT"), ranked });
    const filtered = assemble({ rows: rows("SAND", "PEAT"), ranked });

    const colourOf = (s: ReturnType<typeof assemble>, name: string) =>
      s.find((x) => x.name === name)?.itemStyle.color;
    expect(colourOf(filtered, "SAND")).toBe(colourOf(all, "SAND"));
    expect(colourOf(filtered, "PEAT")).toBe(colourOf(all, "PEAT"));
    // The point of the assertion: SAND is now the FIRST series drawn and still
    // is not slot 1.
    expect(filtered[0]?.name).toBe("SAND");
    expect(filtered[0]?.itemStyle.color).toBe(PALETTE[1]);
  });

  it("ignores first-appearance order when it disagrees with the ranking", () => {
    // The scatter query is a bare row LIMIT with NO ORDER BY, so the order
    // values appear in the fetched slice is arbitrary. CLAY is the commonest
    // value in the table and the LAST to appear in this sample.
    const series = assemble({
      rows: rows("PEAT", "SAND", "CLAY"),
      ranked: ["CLAY", "SAND", "PEAT"],
    });
    expect(names(series)).toEqual(["CLAY", "SAND", "PEAT"]);
    expect(colours(series)).toEqual(PALETTE.slice(0, 3));
  });

  it("leaves a ranked value out when the plotted slice has none of it", () => {
    // Its slot is still its own — SAND keeps colour 2 — but an empty series
    // would put a value in the legend that the chart does not draw.
    const series = assemble({
      rows: rows("SAND"),
      ranked: ["CLAY", "SAND", "PEAT"],
    });
    expect(names(series)).toEqual(["SAND"]);
    expect(colours(series)).toEqual([PALETTE[1]]);
  });
});

describe("assembleSeries — the shapes that must not change", () => {
  it("makes one unnamed series in slot 1 with no colour-by", () => {
    const series = assemble({ rows: [{ x: 1, y: 2 }] });
    expect(series).toHaveLength(1);
    expect(series[0]?.name).toBeUndefined();
    expect(series[0]?.itemStyle.color).toBe(PALETTE[0]);
  });

  it("treats a NULL colour as its own value rather than folding it", () => {
    // The LEFT join that leaves a sample below every stratum unmatched is the
    // live case: `scalarText(null)` is the empty string on both sides of the
    // probe, so it ranks and plots like any other value.
    const series = assemble({ rows: [{ x: 1, y: 2, c: null }], ranked: [""] });
    expect(series).toHaveLength(1);
    expect(series[0]?.name).toBeUndefined();
    expect(series[0]?.itemStyle.color).toBe(PALETTE[0]);
  });

  it("returns nothing when there is nothing to plot", () => {
    expect(assemble()).toEqual([]);
  });

  it("reads X as a label or a number as the axis asks, and copes with bigint", () => {
    // BIGINT comes back from DuckDB-wasm as a JS bigint, which every arithmetic
    // and JSON path downstream throws on.
    const raw = [{ x: 10n, y: 3n, c: "a" }];
    expect(assemble({ rows: raw, ranked: ["a"] })[0]?.data).toEqual([[10, 3]]);
    expect(
      assemble({ rows: raw, ranked: ["a"], categoricalX: true })[0]?.data,
    ).toEqual([["10", 3]]);
    // DECIMAL arrives as a string on a value axis, and a chart drawn from
    // strings puts every point at the same place.
    expect(
      assemble({ rows: [{ x: "1.5", y: "2.5", c: "a" }], ranked: ["a"] })[0]
        ?.data,
    ).toEqual([[1.5, 2.5]]);
  });

  it("carries each form's own render knobs", () => {
    const one = (chartType: "scatter" | "line" | "bar") =>
      assemble({ rows: rows("a"), ranked: ["a"], chartType })[0];
    expect(one("scatter")).toMatchObject({
      type: "scatter",
      symbolSize: 8,
      large: true,
      largeThreshold: 2000,
    });
    expect(one("line")).toMatchObject({ type: "line", sampling: "lttb" });
    const bar = one("bar");
    expect(bar?.type).toBe("bar");
    expect(bar?.sampling).toBeUndefined();
    expect(bar?.large).toBeUndefined();
  });
});
