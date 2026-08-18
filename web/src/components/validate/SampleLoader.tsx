import { For, createSignal, type Component } from "solid-js";
import { ArmedButton } from "@shared/components";
import { SAMPLES } from "../../lib/validator";
import { Disclosure } from "../Disclosure";

export const SampleLoader: Component<{
  onLoad: (bytes: Uint8Array, name: string) => void;
  /** Open the sample list initially (e.g. when the editor is still empty). */
  open?: boolean;
  /** A file is already loaded, so a sample would discard it — arm first (#408). */
  replaces?: boolean;
}> = (props) => {
  const [error, setError] = createSignal<string | null>(null);

  const load = async (file: string) => {
    setError(null);
    try {
      // Served from public/samples/ under the deploy base. BASE_URL
      // already ends in "/", so concatenation is correct at the root and
      // under any subpath base.
      const res = await fetch(`${import.meta.env.BASE_URL}samples/${file}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const buf = await res.arrayBuffer();
      props.onLoad(new Uint8Array(buf), file);
    } catch (e) {
      setError(`Could not load ${file}: ${String(e)}`);
    }
  };

  return (
    <Disclosure
      summary="Or try a sample"
      count={SAMPLES.length}
      open={props.open}
    >
      <div class="flex flex-col gap-2">
        <div class="flex flex-wrap gap-2">
          <For each={SAMPLES}>
            {(s) => (
              <ArmedButton
                confirm="Replace loaded file?"
                armWhen={props.replaces ?? false}
                onConfirm={() => void load(s.file)}
                size="sm"
                title={s.blurb}
              >
                {s.name}
              </ArmedButton>
            )}
          </For>
        </div>
        {error() && <p class="text-xs text-err">{error()}</p>}
      </div>
    </Disclosure>
  );
};
