import { For, createSignal, type Component } from "solid-js";
import { SAMPLES } from "../../lib/validator";
import { Disclosure } from "../Disclosure";

export const SampleLoader: Component<{
  onLoad: (bytes: Uint8Array, name: string) => void;
  /** Open the sample list initially (e.g. when the editor is still empty). */
  open?: boolean;
}> = (props) => {
  const [error, setError] = createSignal<string | null>(null);

  const load = async (file: string) => {
    setError(null);
    try {
      // Served from public/samples/ under the deploy base. BASE_URL
      // already ends in "/", so concatenation is correct under both
      // "/" (dev) and "/ags5_concept/" (Pages).
      const res = await fetch(`${import.meta.env.BASE_URL}samples/${file}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const buf = await res.arrayBuffer();
      props.onLoad(new Uint8Array(buf), file);
    } catch (e) {
      setError(`Could not load ${file}: ${String(e)}`);
    }
  };

  return (
    <Disclosure summary="Or try a sample" count={SAMPLES.length} open={props.open}>
      <div class="flex flex-col gap-2">
        <div class="flex flex-wrap gap-2">
          <For each={SAMPLES}>
            {(s) => (
              <button
                type="button"
                onClick={() => void load(s.file)}
                class="rounded border border-line-strong px-2.5 py-1 text-xs text-fg-soft transition-colors hover:border-accent hover:text-accent"
                title={s.blurb}
              >
                {s.name}
              </button>
            )}
          </For>
        </div>
        {error() && <p class="text-xs text-err">{error()}</p>}
      </div>
    </Disclosure>
  );
};
