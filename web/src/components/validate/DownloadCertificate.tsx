import { type Component, createSignal, Show } from "solid-js";
import { certify } from "../../lib/validatorClient";
import type { DictVersionOpt, EncodingOpt } from "../../lib/validator";

/** "Download certificate": mints the `.ags.idx` validity certificate for a
 *  clean file entirely client-side (the wasm `certify`) and saves it. Shown
 *  by the ValidatePane only when the file has zero findings — a certificate
 *  attests a clean validation. */
export const DownloadCertificate: Component<{
  bytes: () => Uint8Array | null;
  name: string;
  dict: DictVersionOpt;
  encoding: EncodingOpt;
}> = (props) => {
  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);

  const download = async () => {
    const b = props.bytes();
    if (!b) return;
    setBusy(true);
    setErr(null);
    try {
      const json = await certify(b, props.dict, props.encoding);
      const url = URL.createObjectURL(
        new Blob([json], { type: "application/json" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = `${props.name || "delivery.ags"}.idx`;
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
        onClick={() => void download()}
        class="rounded-md border border-line-strong px-3 py-1.5 text-fg transition-colors hover:border-accent hover:text-accent disabled:opacity-45"
      >
        {busy() ? "Minting…" : "Download certificate (.ags.idx)"}
      </button>
      <span class="text-xs text-fg-faint">
        A validity certificate for this clean file — the same{" "}
        <code class="mono">.ags.idx</code> the CLI and libraries mint, so a
        later re-check can skip an unchanged file.
      </span>
      <Show when={err()}>
        <span class="basis-full text-xs text-err">Certify failed: {err()}</span>
      </Show>
    </div>
  );
};
