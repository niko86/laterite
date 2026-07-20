import { createMemo, createSignal, Show, type Component } from "solid-js";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";

// Formatter / tidy: normalise an AGS4 file's bytes — CRLF line endings
// (AGS4 Rule 2a), a single trailing newline, optional trailing-whitespace
// strip, and UTF-8 BOM removal (Rule 1). Distinct from the Fix tab: it's a
// pure byte tidy that works on any text (it doesn't need the file to parse),
// and it's the inverse of a hand-edited LF file. Fully client-side.

const decode = (b: Uint8Array) =>
  // ignoreBOM:false (default) means the decoder strips a leading BOM, so the
  // decoded text — and thus the re-encoded output — carries no BOM.
  new TextDecoder("utf-8", { fatal: false }).decode(b);

const hasBom = (b: Uint8Array) =>
  b.length >= 3 && b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf;

export const Formatter: Component = () => {
  const [stripTrailing, setStripTrailing] = createSignal(true);

  const computed = createMemo(() => {
    const raw = fileStore.bytes();
    if (!raw) return null;
    const text = decode(raw);
    const lines = text.split(/\r?\n/);
    // A trailing newline yields one empty segment — drop it so we don't
    // emit a double blank line; the final "\r\n" below re-adds exactly one.
    if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();

    // Stats (computed against the original text).
    const lfOnly = (text.match(/(?<!\r)\n/g) || []).length;
    const trailing = lines.filter((l) => /[ \t]+$/.test(l)).length;

    const body = stripTrailing()
      ? lines.map((l) => l.replace(/[ \t]+$/, ""))
      : lines;
    const out = body.join("\r\n") + "\r\n";
    const outBytes = new TextEncoder().encode(out);

    return {
      out,
      outBytes,
      bom: hasBom(raw),
      lfConverted: lfOnly,
      trailingFixed: stripTrailing() ? trailing : 0,
      sizeBefore: raw.length,
      sizeAfter: outBytes.length,
    };
  });

  const save = () => {
    const c = computed();
    if (c)
      downloadBlob(
        c.out,
        `${baseName(fileStore.name())}.ags`,
        "text/plain;charset=utf-8",
      );
  };

  const apply = () => {
    const c = computed();
    if (c) fileStore.setBytes(c.outBytes); // canonical CRLF; originalBytes intact
  };

  return (
    <Show
      when={computed()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Formatter</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab to tidy its bytes — CRLF line
            endings, a trailing newline, trailing-whitespace strip, and BOM
            removal. Nothing is uploaded.
          </p>
        </div>
      }
    >
      {(c) => (
        <div class="flex min-w-0 flex-col gap-3">
          <p class="text-sm text-fg-soft">
            Normalise to CRLF line endings + a single trailing newline, strip a
            UTF-8 BOM, and (optionally) trailing whitespace.
          </p>

          <div class="flex flex-wrap items-center gap-3 text-sm">
            <button
              type="button"
              class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600"
              onClick={save}
            >
              Download tidied
            </button>
            <button
              type="button"
              class="rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip"
              onClick={apply}
            >
              Apply to loaded file
            </button>
            <label class="flex cursor-pointer items-center gap-1.5 text-xs text-fg-muted">
              <input
                type="checkbox"
                checked={stripTrailing()}
                onChange={(e) => setStripTrailing(e.currentTarget.checked)}
              />
              Strip trailing whitespace
            </label>
          </div>

          <ul class="flex flex-col gap-1 text-xs text-fg-soft">
            <li>
              <span class="text-fg-dim">UTF-8 BOM:</span>{" "}
              {c().bom ? (
                <span class="text-warn">present → removed</span>
              ) : (
                <span class="text-ok">none</span>
              )}
            </li>
            <li>
              <span class="text-fg-dim">LF-only line endings:</span>{" "}
              {c().lfConverted > 0 ? (
                <span class="text-warn">
                  {c().lfConverted} → converted to CRLF
                </span>
              ) : (
                <span class="text-ok">all CRLF already</span>
              )}
            </li>
            <li>
              <span class="text-fg-dim">Trailing whitespace:</span>{" "}
              {c().trailingFixed > 0 ? (
                <span class="text-warn">
                  {c().trailingFixed} lines stripped
                </span>
              ) : (
                <span class="text-ok">none</span>
              )}
            </li>
            <li>
              <span class="text-fg-dim">Size:</span> {c().sizeBefore} →{" "}
              {c().sizeAfter} bytes
            </li>
          </ul>
        </div>
      )}
    </Show>
  );
};
