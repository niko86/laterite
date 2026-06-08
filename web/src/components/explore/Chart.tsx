import { createEffect, onCleanup, onMount, type Component } from "solid-js";

// Generic ECharts host. echarts is multi-MB, so it's dynamically imported
// (tree-shaken via echarts/core + .use of only the charts/components we need)
// — it loads only when the Charts view first mounts, never on the validate or
// table-browse paths. The instance is typed loosely (`any`) — it's a 3rd-party
// canvas object, and the tree-shaken core types are awkward to pin.
export const Chart: Component<{
  option: () => Record<string, unknown> | null;
  height?: string;
}> = (props) => {
  let el!: HTMLDivElement;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let chart: any = null;

  onMount(() => {
    let disposed = false;
    let ro: ResizeObserver | undefined;
    void Promise.all([
      import("echarts/core"),
      import("echarts/charts"),
      import("echarts/components"),
      import("echarts/renderers"),
    ]).then(([core, charts, components, renderers]) => {
      if (disposed) return;
      core.use([
        charts.ScatterChart,
        charts.LineChart,
        charts.BarChart,
        charts.CustomChart,
        components.GridComponent,
        components.TooltipComponent,
        components.DataZoomComponent,
        components.LegendComponent,
        renderers.CanvasRenderer,
      ]);
      chart = core.init(el);
      const o = props.option();
      if (o) chart.setOption(o, { lazyUpdate: true });
      ro = new ResizeObserver(() => chart?.resize());
      ro.observe(el);
    });
    onCleanup(() => {
      disposed = true;
      ro?.disconnect();
      chart?.dispose();
      chart = null;
    });
  });

  // Re-render on option change once the (async-loaded) chart exists. notMerge
  // (full replace) + lazyUpdate (coalesce into the next frame) keeps control
  // tweaks cheap on a weak CPU instead of a synchronous re-layout each change.
  createEffect(() => {
    const o = props.option();
    if (chart && o) chart.setOption(o, { notMerge: true, lazyUpdate: true });
  });

  return (
    <div
      ref={el}
      style={{ width: "100%", height: props.height ?? "340px" }}
    />
  );
};
