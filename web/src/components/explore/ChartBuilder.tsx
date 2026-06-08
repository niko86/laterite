import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import type { GroupMeta } from "../../lib/duckTypes";
import {
  chartSql,
  type Agg,
  type ChartType,
  type JoinSpec,
  type QualifiedCol,
} from "../../lib/sqlgen";
import { Spinner } from "../Spinner";
import { Chart } from "./Chart";
import { ControlGrid } from "../ControlGrid";
import { Chevron } from "../Chevron";
import { isLowEndDevice } from "../../lib/device";
import { controlClass } from "../../lib/controls";
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

const NUMERIC = /DOUBLE|BIGINT|DECIMAL|HUGEINT|FLOAT|INTEGER|REAL|SMALLINT|TINYINT/i;
const isNumeric = (sqlType: string | undefined) => !!sqlType && NUMERIC.test(sqlType);

const num = (v: unknown): number =>
  typeof v === "bigint" ? Number(v) : typeof v === "number" ? v : Number(v);
const str = (v: unknown): string =>
  v === null || v === undefined ? "" : typeof v === "bigint" ? v.toString() : String(v);

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
    if (!table() && codes().length) setTable(codes()[0]);
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
      ? { baseAlias: BASE, baseCol: dc.col, top: dr.top, base: dr.base, level: dc.level }
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
    if (!joined()) return base;
    const j = joinMeta()!;
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
      const firstNum = cols.find((c) => isNumeric(c.sqlType) && c.key !== cols[0]?.key);
      setYCol(firstNum?.key ?? cols[1]?.key ?? cols[0]?.key ?? "");
    }
    if (colourCol() && !cols.some((c) => c.key === colourCol())) setColourCol("");
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

  const sql = createMemo(() => {
    if (!table()) return "";
    let joins: JoinSpec[] | undefined;
    if (joined() && joinPairs().length) {
      const rp = rangePred();
      joins = [
        {
          table: joinCode(),
          alias: JOIN,
          kind: "LEFT",
          leftAlias: BASE,
          on: joinPairs(),
          range: rp
            ? { baseAlias: rp.baseAlias, baseCol: rp.baseCol, top: rp.top, base: rp.base }
            : undefined,
        },
      ];
    }
    return chartSql({
      table: table(),
      alias: joins ? BASE : undefined,
      joins,
      x: ref(xCol()),
      y: ref(yCol()),
      colour: colourCol() ? ref(colourCol()) : undefined,
      chartType: chartType(),
      agg: agg(),
      rowCap: ROW_CAP,
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

  // Build the ECharts option from the fetched rows + the current controls.
  const option = createMemo<Record<string, unknown> | null>(() => {
    const data = rows();
    if (!data || data.length === 0) return null;
    const cType = chartType();
    const hasColour = !!colourCol();
    const xNumeric = !aggregating() && isNumeric(byKey(xCol())?.sqlType);
    const xName = byKey(xCol())?.col ?? xCol();
    const yName = byKey(yCol())?.col ?? yCol();

    // Split into series by the colour column (one series per distinct value),
    // or a single series when no colour-by.
    const bySeries = new Map<string, [unknown, number][]>();
    for (const r of data) {
      const key = hasColour ? str(r.c) : "";
      const arr = bySeries.get(key) ?? bySeries.set(key, []).get(key)!;
      arr.push([cType === "bar" || !xNumeric ? str(r.x) : num(r.x), num(r.y)]);
    }

    const seriesType = cType === "scatter" ? "scatter" : cType;
    const series = [...bySeries.entries()].map(([name, pts]) => ({
      name: name || undefined,
      type: seriesType,
      symbolSize: cType === "scatter" ? 8 : undefined,
      // Perf knobs for big series on weak hardware: `large` batches scatter
      // points into one buffered draw (vs an animated per-symbol render that
      // melts an integrated GPU); LTTB down-samples a dense line without
      // changing its visible shape.
      ...(cType === "scatter" ? { large: true, largeThreshold: 2000 } : {}),
      ...(cType === "line" ? { sampling: "lttb" } : {}),
      data: pts,
    }));

    const catAxis = cType === "bar" || !xNumeric;
    return {
      // No entry/update animation — on a weak CPU/GPU animating thousands of
      // points (and re-animating on every control tweak) is the jank.
      animation: false,
      tooltip: { trigger: cType === "bar" ? "axis" : "item" },
      legend: hasColour ? { type: "scroll", top: 0 } : undefined,
      grid: { left: 64, right: 24, top: hasColour ? 36 : 16, bottom: 56 },
      xAxis: {
        type: catAxis ? "category" : "value",
        name: xName,
        nameLocation: "middle",
        nameGap: 36,
        scale: !catAxis,
        axisLabel: { rotate: catAxis ? 30 : 0 },
      },
      yAxis: {
        type: "value",
        name: counting() ? "count" : `${agg() !== "none" ? agg() + " " : ""}${yName}`,
        nameLocation: "middle",
        nameGap: 46,
        scale: true,
      },
      dataZoom: catAxis
        ? undefined
        : [
            { type: "inside" },
            { type: "slider", height: 18, bottom: 20 },
          ],
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
    <label class="flex flex-col gap-0.5 text-xs text-fg-muted">
      {sp.label}
      <select
        aria-label={sp.ariaLabel ?? sp.label}
        class={controlClass}
        value={sp.value}
        onChange={(e) => sp.onChange(e.currentTarget.value)}
      >
        <Show when={sp.allowEmpty !== undefined}>
          <option value="">{sp.allowEmpty}</option>
        </Show>
        <For each={sp.options}>
          {(o) => <option value={o.value}>{o.label}</option>}
        </For>
      </select>
    </label>
  );

  const colOptions = () =>
    allCols().map((c) => ({ value: c.key, label: `${c.label} (${c.sqlType})` }));

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
            options={related().map((r) => ({ value: r.code, label: `${r.code} (${r.direction})` }))}
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
        <Select label="X axis" ariaLabel="x axis" value={xCol()} onChange={setXCol} options={colOptions()} />
        <Show when={!counting()}>
          <Select label="Y axis" ariaLabel="y axis" value={yCol()} onChange={setYCol} options={colOptions()} />
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
            <span class="mono">{rp().top}</span>…<span class="mono">{rp().base}</span>{" "}
            ({rp().level}-level) — colour by a {joinCode()} column to band the plot by stratum.
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
        <Show when={rows.error}>
          <p class="text-sm text-err">Chart query error: {String(rows.error)}</p>
        </Show>
        <Show
          when={option()}
          fallback={
            <span class="text-sm text-fg-muted">
              <Show when={rows.loading} fallback="No rows to plot for this selection.">
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
          <pre class="mono mt-1 overflow-x-auto rounded border border-line bg-surface-raised p-2 text-fg-soft">
            {sql()}
          </pre>
        </details>
      </Show>
    </div>
  );
};
