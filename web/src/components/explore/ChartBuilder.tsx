import { chartTokens } from "../../lib/chartTheme";
import { Field, Select as SelectControl } from "@shared/components";
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { scalarText, type GroupMeta } from "../../lib/duckTypes";
import {
  chartSql,
  chartRankSql,
  type Agg,
  type ChartType,
  type JoinSpec,
  type QualifiedCol,
} from "../../lib/sqlgen";
import { assembleSeries, seriesCap } from "../../lib/chartSeries";
import { Spinner } from "../Spinner";
import { Chart } from "./Chart";
import { ControlGrid } from "../ControlGrid";
import { Chevron } from "../Chevron";
import { isLowEndDevice } from "../../lib/device";
import {
  relatedGroups,
  joinKeys,
  depthRangeOf,
  depthColumnFor,
  type DictMap,
} from "../../lib/relationships";

// A general-purpose chart builder over the ingested typed DuckDB tables: pick a
// table (+ optional related group) → chart type → X → Y (+ aggregate) → optional
// colour/series-by, and it composes the SQL, runs it, and renders an ECharts
// plot. With a related group, X/Y/colour can come from either table — e.g. a
// test value vs depth coloured by GEOL_LEG (the stratum, via the depth band).

const NUMERIC =
  /DOUBLE|BIGINT|DECIMAL|HUGEINT|FLOAT|INTEGER|REAL|SMALLINT|TINYINT/i;
const isNumeric = (sqlType: string | undefined) =>
  !!sqlType && NUMERIC.test(sqlType);

// Plotting thousands of points per-symbol with entry animation melts a weak
// GPU/CPU — halve the cap on a low-end device (export/SQL still see everything).
const ROW_CAP = isLowEndDevice() ? 2000 : 5000;

const BASE = "c"; // base-table alias
const JOIN = "j"; // related-table alias

interface ColRef {
  alias: string;
  col: string;
  sqlType: string;
  key: string; // always "alias.col" so a join toggle doesn't churn X/Y picks
  label: string;
}

export const ChartBuilder: Component<{
  groups: GroupMeta[];
  dict: DictMap | undefined;
}> = (props) => {
  const codes = () => props.groups.map((g) => g.code);
  const [table, setTable] = createSignal<string>("");
  const [joinCode, setJoinCode] = createSignal(""); // "" = single-table
  createEffect(() => {
    const first = codes()[0];
    if (!table() && first !== undefined) setTable(first);
  });

  const baseMeta = () => props.groups.find((g) => g.code === table());
  const joinMeta = () => props.groups.find((g) => g.code === joinCode());
  const joined = () => !!joinCode() && !!joinMeta();

  const related = createMemo(() =>
    props.dict && table() ? relatedGroups(table(), codes(), props.dict) : [],
  );
  const joinPairs = createMemo(() => {
    const b = baseMeta();
    const j = joinMeta();
    if (!props.dict || !joined() || !b || !j) return [];
    return joinKeys(
      { code: b.code, cols: b.headings },
      { code: j.code, cols: j.headings },
      props.dict,
    );
  });
  const rangePred = createMemo(() => {
    const b = baseMeta();
    const j = joinMeta();
    if (!props.dict || !joined() || !b || !j) return null;
    // cols-aware: only a band whose top+base columns the related table actually
    // carries (so we never emit a predicate on a missing *_BASE column).
    const dr = depthRangeOf(joinCode(), props.dict, j.headings);
    const dc = dr ? depthColumnFor(b.code, b.headings, props.dict) : null;
    return dr && dc
      ? {
          baseAlias: BASE,
          baseCol: dc.col,
          top: dr.top,
          base: dr.base,
          level: dc.level,
        }
      : null;
  });

  // Columns keyed by "alias.col" (base alias c; related alias j when joined).
  const allCols = createMemo<ColRef[]>(() => {
    const b = baseMeta();
    if (!b) return [];
    const base = b.headings.map((col, i) => ({
      alias: BASE,
      col,
      sqlType: b.sql_types[i] ?? "VARCHAR",
      key: `${BASE}.${col}`,
      label: joined() ? `${b.code}.${col}` : col,
    }));
    const j = joinMeta();
    // `joined()` is `!!joinCode() && !!joinMeta()`, inlined here so `j` narrows.
    if (!joinCode() || !j) return base;
    return [
      ...base,
      ...j.headings.map((col, i) => ({
        alias: JOIN,
        col,
        sqlType: j.sql_types[i] ?? "VARCHAR",
        key: `${JOIN}.${col}`,
        label: `${j.code}.${col}`,
      })),
    ];
  });
  const byKey = (k: string) => allCols().find((c) => c.key === k);

  const [chartType, setChartType] = createSignal<ChartType>("scatter");
  const [xCol, setXCol] = createSignal("");
  const [yCol, setYCol] = createSignal("");
  const [agg, setAgg] = createSignal<Agg>("none");
  const [colourCol, setColourCol] = createSignal("");

  // Seed/repair X/Y when the available columns change (table or join change):
  // X = first column, Y = first numeric after it. Only reseeds a pick that's no
  // longer valid, so adding a join keeps the existing base-column picks.
  createEffect(() => {
    const cols = allCols();
    if (!cols.length) return;
    if (!cols.some((c) => c.key === xCol())) setXCol(cols[0]?.key ?? "");
    if (!cols.some((c) => c.key === yCol())) {
      const firstNum = cols.find(
        (c) => isNumeric(c.sqlType) && c.key !== cols[0]?.key,
      );
      setYCol(firstNum?.key ?? cols[1]?.key ?? cols[0]?.key ?? "");
    }
    if (colourCol() && !cols.some((c) => c.key === colourCol()))
      setColourCol("");
  });

  const aggregating = () => chartType() === "bar" && agg() !== "none";
  const counting = () => agg() === "count";

  // Resolve a stored key to what chartSql wants: a QualifiedCol when joined, the
  // bare column name (string) in single-table mode.
  const ref = (k: string): string | QualifiedCol => {
    const c = byKey(k);
    if (!c) return k;
    return joined() ? { alias: c.alias, col: c.col } : c.col;
  };

  const joins = createMemo<JoinSpec[] | undefined>(() => {
    if (!joined() || !joinPairs().length) return undefined;
    const rp = rangePred();
    return [
      {
        table: joinCode(),
        alias: JOIN,
        kind: "LEFT",
        leftAlias: BASE,
        on: joinPairs(),
        range: rp
          ? {
              baseAlias: rp.baseAlias,
              baseCol: rp.baseCol,
              top: rp.top,
              base: rp.base,
            }
          : undefined,
      },
    ];
  });

  // The half both queries below share. The probe that ranks the colour values
  // has to run over the SAME population as the plot — same table, joins,
  // aliasing and row filter — or it would fold on a count of rows the chart
  // never draws (#445).
  const queryBase = () => ({
    table: table(),
    alias: joins() ? BASE : undefined,
    joins: joins(),
    x: ref(xCol()),
    y: ref(yCol()),
    chartType: chartType(),
    agg: agg(),
  });

  const sql = createMemo(() => {
    if (!table()) return "";
    return chartSql({
      ...queryBase(),
      colour: colourCol() ? ref(colourCol()) : undefined,
      rowCap: ROW_CAP,
    });
  });

  // Which colour values keep a palette slot is a question about the whole
  // table, not about the sampled rows: the scatter query is a bare row LIMIT
  // with no ORDER BY, so its values are an arbitrary slice and a legend saying
  // "Other" over them would claim something the sample cannot support.
  const rankSql = createMemo(() => {
    const colour = colourCol();
    if (!table() || !colour) return "";
    return chartRankSql({
      ...queryBase(),
      colour: ref(colour),
      cap: seriesCap(chartType()),
    });
  });

  const [rows] = createResource(
    () => sql(),
    async (s) => {
      if (!s) return null;
      const { run } = await import("../../lib/duck");
      const t = await run(s);
      return t.toArray() as Record<string, unknown>[];
    },
  );

  const [ranked] = createResource(
    () => rankSql(),
    async (s) => {
      if (!s) return [];
      const { run } = await import("../../lib/duck");
      const t = await run(s);
      // Through the same coercion the plotted rows take, so the two sides of
      // the match agree on what a NULL colour is.
      return (t.toArray() as Record<string, unknown>[]).map((r) =>
        scalarText(r.c),
      );
    },
  );

  // Reading `rows` after a failed query THROWS (a Solid resource re-reads its
  // error), and `option` below is an eager memo — unguarded, the throw took
  // the update down before the sibling error banner in the JSX could render
  // it (#359, same shape as ExplorePane's accessor).
  const fetched = () => (rows.error ? undefined : rows());
  const rankedValues = () => (ranked.error ? undefined : ranked());

  // Build the ECharts option from the fetched rows + the current controls.
  const option = createMemo<Record<string, unknown> | null>(() => {
    const data = fetched();
    if (!data || data.length === 0) return null;
    // With a colour-by the plot WAITS for the probe rather than splitting on
    // what the sample happens to contain — the ranking is the only thing that
    // can say which values keep a slot.
    const ranking = colourCol() ? rankedValues() : [];
    if (ranking === undefined) return null;
    const cType = chartType();
    const hasColour = !!colourCol();
    const xNumeric = !aggregating() && isNumeric(byKey(xCol())?.sqlType);
    const xName = byKey(xCol())?.col ?? xCol();
    const yName = byKey(yCol())?.col ?? yCol();
    const catAxis = cType === "bar" || !xNumeric;

    // Token-resolved theme (#410): reading chartTokens() here makes this memo
    // rebuild on theme flip, so the canvas repaints with the flipped values.
    const tk = chartTokens();
    const series = assembleSeries({
      rows: data,
      ranked: ranking,
      chartType: cType,
      categoricalX: catAxis,
      palette: tk.palette,
      other: tk.other,
    });
    const axisText = { color: tk.fgDim };
    const axisName = { color: tk.fgSoft };
    const axisLine = { lineStyle: { color: tk.line } };
    const splitLine = { lineStyle: { color: tk.lineSubtle } };
    return {
      // No entry/update animation — on a weak CPU/GPU animating thousands of
      // points (and re-animating on every control tweak) is the jank.
      animation: false,
      // No root `color`. That array is exactly the palette ECharts walks and
      // wraps around — past its last entry it repaints slot one rather than
      // failing — so every series carries its own pinned colour instead.
      textStyle: { color: tk.fgSoft, fontFamily: tk.familyUi },
      tooltip: {
        trigger: cType === "bar" ? "axis" : "item",
        backgroundColor: tk.tooltipBg,
        borderColor: tk.tooltipBorder,
        textStyle: { color: tk.tooltipFg },
      },
      legend: hasColour
        ? { type: "scroll", top: 0, textStyle: { color: tk.fgSoft } }
        : undefined,
      grid: { left: 64, right: 24, top: hasColour ? 36 : 16, bottom: 56 },
      xAxis: {
        type: catAxis ? "category" : "value",
        name: xName,
        nameLocation: "middle",
        nameGap: 36,
        scale: !catAxis,
        axisLabel: { rotate: catAxis ? 30 : 0, ...axisText },
        nameTextStyle: axisName,
        axisLine,
        splitLine,
      },
      yAxis: {
        type: "value",
        name: counting()
          ? "count"
          : `${agg() !== "none" ? agg() + " " : ""}${yName}`,
        nameLocation: "middle",
        nameGap: 46,
        scale: true,
        axisLabel: axisText,
        nameTextStyle: axisName,
        axisLine,
        splitLine,
      },
      dataZoom: catAxis
        ? undefined
        : [{ type: "inside" }, { type: "slider", height: 18, bottom: 20 }],
      series,
    };
  });

  const Select: Component<{
    label: string;
    value: string;
    onChange: (v: string) => void;
    options: { value: string; label: string }[];
    allowEmpty?: string;
    ariaLabel?: string;
  }> = (sp) => (
    <Field label={sp.label}>
      <SelectControl
        aria-label={sp.ariaLabel ?? sp.label}
        value={sp.value}
        onChange={(e) => {
          sp.onChange(e.currentTarget.value);
        }}
      >
        <Show when={sp.allowEmpty !== undefined}>
          <option value="">{sp.allowEmpty}</option>
        </Show>
        <For each={sp.options}>
          {(o) => <option value={o.value}>{o.label}</option>}
        </For>
      </SelectControl>
    </Field>
  );

  const colOptions = () =>
    allCols().map((c) => ({
      value: c.key,
      label: `${c.label} (${c.sqlType})`,
    }));

  return (
    <div class="flex min-w-0 flex-col gap-4">
      <ControlGrid>
        <Select
          label="Table"
          value={table()}
          onChange={setTable}
          options={codes().map((c) => ({ value: c, label: c }))}
        />
        <Show when={related().length > 0}>
          <Select
            label="Join related"
            ariaLabel="related group"
            value={joinCode()}
            onChange={setJoinCode}
            options={related().map((r) => ({
              value: r.code,
              label: `${r.code} (${r.direction})`,
            }))}
            allowEmpty="(none)"
          />
        </Show>
        <Select
          label="Chart"
          value={chartType()}
          onChange={(v) => setChartType(v as ChartType)}
          options={[
            { value: "scatter", label: "Scatter" },
            { value: "line", label: "Line" },
            { value: "bar", label: "Bar" },
          ]}
        />
        <Select
          label="X axis"
          ariaLabel="x axis"
          value={xCol()}
          onChange={setXCol}
          options={colOptions()}
        />
        <Show when={!counting()}>
          <Select
            label="Y axis"
            ariaLabel="y axis"
            value={yCol()}
            onChange={setYCol}
            options={colOptions()}
          />
        </Show>
        <Show when={chartType() === "bar"}>
          <Select
            label="Aggregate"
            value={agg()}
            onChange={(v) => setAgg(v as Agg)}
            options={[
              { value: "none", label: "none (raw)" },
              { value: "count", label: "count" },
              { value: "sum", label: "sum" },
              { value: "avg", label: "avg" },
              { value: "min", label: "min" },
              { value: "max", label: "max" },
            ]}
          />
        </Show>
        <Select
          label="Colour by"
          ariaLabel="colour by"
          value={colourCol()}
          onChange={setColourCol}
          options={colOptions()}
          allowEmpty="(none)"
        />
      </ControlGrid>

      <Show when={rangePred()}>
        {(rp) => (
          <p class="text-xs text-fg-faint">
            Depth-band join: <span class="mono">{rp().baseCol}</span> within{" "}
            <span class="mono">{rp().top}</span>…
            <span class="mono">{rp().base}</span> ({rp().level}-level) — colour
            by a {joinCode()} column to band the plot by stratum.
          </p>
        )}
      </Show>

      <Show
        when={sql()}
        fallback={
          <p class="text-sm text-fg-muted">
            Pick a table, X and Y column to build a chart.
          </p>
        }
      >
        <Show when={Boolean(rows.error) || Boolean(ranked.error)}>
          <p class="text-sm text-err">
            Chart query error: {String(rows.error ?? ranked.error)}
          </p>
        </Show>
        <Show
          when={option()}
          fallback={
            <span class="text-sm text-fg-muted">
              <Show
                when={rows.loading || ranked.loading}
                fallback="No rows to plot for this selection."
              >
                <Spinner label="Querying…" />
              </Show>
            </span>
          }
        >
          <Chart option={option} height="420px" />
        </Show>
        <details class="group text-xs">
          <summary class="flex cursor-pointer list-none select-none items-center gap-1.5 text-fg-dim [&::-webkit-details-marker]:hidden">
            <Chevron />
            SQL
          </summary>
          <pre class="mono mt-1 overflow-x-auto rounded-sm border border-line bg-surface-raised p-2 text-fg-soft">
            {sql()}
          </pre>
        </details>
      </Show>
    </div>
  );
};
