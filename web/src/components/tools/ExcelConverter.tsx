import { Button, Checkbox } from "@shared/components";
import { createSignal, For, onMount, Show, type Component } from "solid-js";
import { fileStore } from "../../lib/fileStore";
import {
  excelExport,
  excelImport,
  startTier2Worker,
  EngineUnavailableError,
} from "../../lib/validatorClient";
import { downloadBlob, baseName } from "../../lib/download";
import { engineFailureMessage } from "../../lib/engineFailure";

const XLSX_MIME =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

const count = (n: number, noun: string) => `${n} ${noun}${n === 1 ? "" : "s"}`;

// Tools → Excel: AGS4 ↔ `.xlsx` conversion, fully client-side (the wasm
// laterite-ags4-excel cores; #359). Export turns the loaded AGS4 into a workbook
// (one sheet per group, python-ags4's layout); import turns an uploaded `.xlsx`
// back into AGS4. Nothing is uploaded to a server.
export const ExcelConverter: Component = () => {
  // Both conversions run in the second worker (#354) — the only tool that does.
  // Selecting this tool creates it, so the engine is instantiating while the
  // user is still picking a file instead of after they press the button. It is
  // also the one tier-2 consumer that never waits on DuckDB, so the head start
  // is the whole difference on a slow device.
  onMount(startTier2Worker);

  const [busy, setBusy] = createSignal<"export" | "import" | null>(null);
  // A failed conversion and an engine that never downloaded read differently
  // and end differently: one is about the file, the other is about the network
  // and clears on a retry (#357). `retry` is what tells them apart on screen.
  const [err, setErr] = createSignal<{ text: string; retry: boolean } | null>(
    null,
  );
  const [warnings, setWarnings] = createSignal<string[]>([]);
  const [note, setNote] = createSignal<string | null>(null);
  const [formatNumeric, setFormatNumeric] = createSignal(true);

  // The last conversion attempted, so "Try again" repeats it — an import's
  // workbook is gone from the file input by then (it's cleared on change so the
  // same file can be re-picked), so a retry has to close over it rather than
  // re-read it. Not a signal: only the click handler reads it, and it changes
  // nothing on screen by itself.
  let lastAttempt: (() => Promise<void>) | null = null;

  const reset = () => {
    setErr(null);
    setWarnings([]);
    setNote(null);
  };

  // The copy is the shared engine-failure voice — engineFailureMessage's doc
  // carries the shared rationale (#391, #414). Only the converter's own says
  // stay here: the untyped override reads "Conversion failed" because such an
  // error is about the workbook, not the engine — and it keeps `String(e)`,
  // #415's doubled prefix included, because #414 pinned the rendered lines
  // byte-for-byte; the crash suffix is honest only beside this pane's Try
  // again button; and `retry` derives beside the call.
  const failed = (e: unknown) => {
    const retry = e instanceof EngineUnavailableError;
    const text = engineFailureMessage(
      e,
      "The converter's engine",
      `Conversion failed: ${String(e)}`,
    );
    setErr({
      text:
        retry && e.reason === "crash"
          ? `${text} Trying again starts a fresh one.`
          : text,
      retry,
    });
  };

  const runExport = async () => {
    lastAttempt = runExport;
    setBusy("export");
    reset();
    const b = fileStore.bytes();
    if (!b) {
      // Only reachable from "Try again" — the button itself renders behind a
      // loaded file. Saying so beats the early return this replaced, which left
      // the failure and the retry button exactly as they were and made the
      // click look broken.
      setErr({
        text: "There's no file loaded any more — load one in the Validate tab and export again.",
        retry: false,
      });
      setBusy(null);
      return;
    }
    try {
      const r = await excelExport(b);
      downloadBlob(r.bytes, `${baseName(fileStore.name())}.xlsx`, XLSX_MIME);
      setWarnings(r.warnings);
      setNote(
        `${count(r.sheets, "sheet")}, ${count(r.rows, "data row")} → .xlsx`,
      );
    } catch (e) {
      failed(e);
    } finally {
      setBusy(null);
    }
  };

  const runImport = async (file: File) => {
    lastAttempt = () => runImport(file);
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
      failed(e);
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
            <Button
              variant="primary"
              disabled={busy() !== null}
              onClick={() => void runExport()}
            >
              {busy() === "export"
                ? "Converting…"
                : "Download as Excel (.xlsx)"}
            </Button>
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
          {/* Native file input inside the bordered picker label — the
              file-selector idiom; no primitive wraps a hidden <input type=file>. */}
          <label class="cursor-pointer rounded-md border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip">
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
          <Checkbox
            label="Re-format numeric columns to their TYPE"
            checked={formatNumeric()}
            onChange={(e) => setFormatNumeric(e.currentTarget.checked)}
          />
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
        {(e) => (
          <div class="flex max-w-prose flex-col items-start gap-2">
            <p class="text-xs text-err">{e().text}</p>
            {/* Repeats the attempt, which re-creates the worker the channel
              retired when its engine failed — so this really re-fetches
              rather than re-reading a settled rejection (#357). */}
            <Show when={e().retry}>
              <Button
                variant="outline"
                disabled={busy() !== null}
                onClick={() => void lastAttempt?.()}
              >
                Try again
              </Button>
            </Show>
          </div>
        )}
      </Show>
      <Show when={warnings().length > 0}>
        <div class="rounded-lg border border-warn/45 bg-warn-quiet p-3">
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
