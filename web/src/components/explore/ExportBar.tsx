import { createSignal, For, Show, type Component } from "solid-js";

const MIME: Record<"csv" | "json" | "parquet", string> = {
  csv: "text/csv",
  json: "application/json",
  parquet: "application/octet-stream",
};

function download(bytes: Uint8Array, filename: string, mime: string): void {
  const blob = new Blob([bytes as BlobPart], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/** Export the result of `sql()` to a downloaded file. CSV/JSON are built into
 *  DuckDB; Parquet may need an extension that can't autoload offline, so it
 *  falls back with a notice. */
export const ExportBar: Component<{
  sql: () => string;
  filename: string;
}> = (props) => {
  const [busy, setBusy] = createSignal<string | null>(null);
  const [note, setNote] = createSignal<string | null>(null);

  const doExport = async (format: "csv" | "json" | "parquet") => {
    setBusy(format);
    setNote(null);
    try {
      const { exportQuery } = await import("../../lib/duck");
      const bytes = await exportQuery(props.sql(), format);
      download(bytes, `${props.filename}.${format}`, MIME[format]);
    } catch (e) {
      setNote(
        format === "parquet"
          ? "Parquet needs an extension that may be unavailable offline — use CSV or JSON."
          : `Export failed: ${String(e)}`,
      );
    } finally {
      setBusy(null);
    }
  };

  // The whole keyed relational database — ALL groups (with _id/_parent_id), not
  // just this query's result. The browser counterpart to the library's
  // to_duckdb(); the download is ready to open in DuckDB or diff with read_ags.
  const doExportDb = async () => {
    setBusy("duckdb");
    setNote(null);
    try {
      const { exportDuckdb } = await import("../../lib/duck");
      const bytes = await exportDuckdb();
      download(bytes, `${props.filename}.duckdb`, "application/octet-stream");
    } catch (e) {
      setNote(`DuckDB export failed: ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  return (
    <div class="flex flex-wrap items-center gap-2 text-xs">
      <span class="text-fg-muted">Export:</span>
      <For each={["csv", "json", "parquet"] as const}>
        {(fmt) => (
          <button
            type="button"
            title={`This query's result as ${fmt.toUpperCase()}`}
            class="rounded-sm border border-line-strong px-2 py-1 text-fg-soft hover:bg-chip disabled:opacity-45"
            disabled={busy() !== null}
            onClick={() => void doExport(fmt)}
          >
            {busy() === fmt ? "…" : fmt.toUpperCase()}
          </button>
        )}
      </For>
      <button
        type="button"
        title="The whole keyed database — every group with its _id/_parent_id keys, not just this query"
        class="rounded-sm border border-line-strong px-2 py-1 text-fg-soft hover:bg-chip disabled:opacity-45"
        disabled={busy() !== null}
        onClick={() => void doExportDb()}
      >
        {busy() === "duckdb" ? "…" : "DuckDB"}
      </button>
      <Show when={note()}>
        <span class="text-warn">{note()}</span>
      </Show>
    </div>
  );
};
