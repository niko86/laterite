import { createSignal, For, Show, type Component } from "solid-js";
import { fileStore } from "../../lib/fileStore";
import { excelExport, excelImport } from "../../lib/validatorClient";
import { downloadBlob, baseName } from "../../lib/download";

const XLSX_MIME =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

const count = (n: number, noun: string) => `${n} ${noun}${n === 1 ? "" : "s"}`;

// Tools → Excel: AGS4 ↔ `.xlsx` conversion, fully client-side (the wasm
// laterite-excel cores; #359). Export turns the loaded AGS4 into a workbook
// (one sheet per group, python-ags4's layout); import turns an uploaded `.xlsx`
// back into AGS4. Nothing is uploaded to a server.
export const ExcelConverter: Component = () => {
  const [busy, setBusy] = createSignal<"export" | "import" | null>(null);
  const [err, setErr] = createSignal<string | null>(null);
  const [warnings, setWarnings] = createSignal<string[]>([]);
  const [note, setNote] = createSignal<string | null>(null);
  const [formatNumeric, setFormatNumeric] = createSignal(true);

  const reset = () => {
    setErr(null);
    setWarnings([]);
    setNote(null);
  };

  const runExport = async () => {
    const b = fileStore.bytes();
    if (!b) return;
    setBusy("export");
    reset();
    try {
      const r = await excelExport(b);
      downloadBlob(r.bytes, `${baseName(fileStore.name())}.xlsx`, XLSX_MIME);
      setWarnings(r.warnings);
      setNote(
        `${count(r.sheets, "sheet")}, ${count(r.rows, "data row")} → .xlsx`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  };

  const runImport = async (file: File) => {
    setBusy("import");
    reset();
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const r = await excelImport(bytes, formatNumeric());
      downloadBlob(
        r.bytes,
        `${baseName(file.name)}.ags`,
        "text/plain;charset=utf-8",
      );
      setWarnings(r.warnings);
      setNote(
        `${count(r.sheets, "sheet")}, ${count(r.rows, "data row")} → .ags`,
      );
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <div class="flex min-w-0 flex-col gap-4">
      <p class="text-sm text-fg-soft">
        Convert between AGS4 and Excel (<code class="mono">.xlsx</code>) — one
        worksheet per group, python-ags4's layout (HEADING / UNIT / TYPE / DATA
        rows). Fully in-browser; nothing is uploaded.
      </p>

      {/* AGS4 → Excel: acts on the file loaded in the Validate tab. */}
      <div class="flex flex-col gap-2 rounded-lg border border-line bg-surface p-3">
        <p class="text-sm font-medium text-fg-soft">AGS4 → Excel</p>
        <Show
          when={fileStore.bytes()}
          fallback={
            <p class="text-xs text-fg-faint">
              Load an AGS4 file in the Validate tab to export it as a workbook.
            </p>
          }
        >
          <div class="flex flex-wrap items-center gap-3 text-sm">
            <button
              type="button"
              disabled={busy() !== null}
              class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-50"
              onClick={() => void runExport()}
            >
              {busy() === "export"
                ? "Converting…"
                : "Download as Excel (.xlsx)"}
            </button>
            <span class="text-xs text-fg-faint">
              from {fileStore.name() || "the loaded file"}
            </span>
          </div>
        </Show>
      </div>

      {/* Excel → AGS4: takes its own uploaded workbook. */}
      <div class="flex flex-col gap-2 rounded-lg border border-line bg-surface p-3">
        <p class="text-sm font-medium text-fg-soft">Excel → AGS4</p>
        <div class="flex flex-wrap items-center gap-3 text-sm">
          <label class="cursor-pointer rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip">
            {busy() === "import" ? "Converting…" : "Choose an .xlsx file…"}
            <input
              type="file"
              accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
              class="hidden"
              disabled={busy() !== null}
              onChange={(e) => {
                const f = e.currentTarget.files?.[0];
                e.currentTarget.value = ""; // allow re-selecting the same file
                if (f) void runImport(f);
              }}
            />
          </label>
          <label class="flex cursor-pointer items-center gap-1.5 text-xs text-fg-muted">
            <input
              type="checkbox"
              checked={formatNumeric()}
              onChange={(e) => setFormatNumeric(e.currentTarget.checked)}
            />
            Re-format numeric columns to their TYPE
          </label>
        </div>
        <p class="text-xs text-fg-faint">
          Each sheet needs a <code class="mono">HEADING</code> column; columns
          that don't match AGS4 Rule 19 and non-UNIT/TYPE/DATA rows are dropped.
        </p>
      </div>

      <Show when={note()}>
        <p class="text-xs text-ok">✓ {note()}</p>
      </Show>
      <Show when={err()}>
        <p class="text-xs text-err">Conversion failed: {err()}</p>
      </Show>
      <Show when={warnings().length > 0}>
        <div class="rounded-lg border border-amber-500/30 bg-amber-500/5 p-3">
          <p class="text-xs font-medium text-warn">
            {count(warnings().length, "warning")}:
          </p>
          <ul class="mt-1 flex flex-col gap-0.5 text-xs text-fg-soft">
            <For each={warnings()}>{(w) => <li>· {w}</li>}</For>
          </ul>
        </div>
      </Show>
    </div>
  );
};
