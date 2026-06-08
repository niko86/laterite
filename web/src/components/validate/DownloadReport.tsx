import { createSignal, Show, type Component } from "solid-js";
import { validateGzip } from "../../lib/validatorClient";
import type {
  DictVersionOpt,
  EncodingOpt,
  ValidationReport,
} from "../../lib/validator";

// Above this many findings, even producing the download in a browser tab
// is at the edge of what's sane (a multi-hundred-MB JSON is ~2× that as a
// UTF-16 string), so we point at the native CLI instead of pretending the
// tab can always do it.
const HUGE = 500_000;

/** "Download full report": re-runs validation UNCAPPED in the worker,
 *  which gzips the JSON there (the big string never reaches this thread)
 *  and transfers back the compressed bytes — saved as `.json.gz`. */
export const DownloadReport: Component<{
  report: ValidationReport;
  bytes: () => Uint8Array | null;
  name: string;
  dict: DictVersionOpt;
  encoding: EncodingOpt;
  includeFyi: boolean;
}> = (props) => {
  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);

  const download = async () => {
    const b = props.bytes();
    if (!b) return;
    setBusy(true);
    setErr(null);
    try {
      const { bytes } = await validateGzip(
        b,
        props.dict,
        props.includeFyi,
        props.encoding,
      );
      const url = URL.createObjectURL(
        new Blob([bytes], { type: "application/gzip" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = `${(props.name || "report").replace(/\.[^./]*$/, "")}.report.json.gz`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-line bg-surface p-3 text-sm">
      <button
        type="button"
        disabled={busy()}
        onClick={download}
        class="rounded border border-line-strong px-3 py-1.5 text-fg transition-colors hover:border-accent hover:text-accent disabled:cursor-not-allowed disabled:opacity-50"
      >
        {busy() ? "Preparing…" : "Download full report (.json.gz)"}
      </button>
      <span class="text-xs text-fg-faint">
        Every finding, uncapped — gzipped JSON.
      </span>
      <Show when={props.report.finding_count > HUGE}>
        <span class="basis-full text-xs text-warn">
          This file has {props.report.finding_count.toLocaleString()} findings;
          an in-browser download may be slow or fail. For the complete report,
          prefer the native{" "}
          <code class="mono">ags4-check delivery.ags --json</code> CLI (or
          laterite).
        </span>
      </Show>
      <Show when={err()}>
        <span class="basis-full text-xs text-err">
          Download failed: {err()}
        </span>
      </Show>
    </div>
  );
};
