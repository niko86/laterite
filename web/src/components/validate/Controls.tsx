import { For, type Component } from "solid-js";
import type { DictVersionOpt, EncodingOpt } from "../../lib/validator";
import { Card } from "../Card";
import { ControlGrid } from "../ControlGrid";
import { controlClass } from "../../lib/controls";

const DICT_VERSIONS: { value: DictVersionOpt; label: string }[] = [
  { value: "auto", label: "Auto (from TRAN_AGS)" },
  { value: "4.0.3", label: "4.0.3" },
  { value: "4.0.4", label: "4.0.4" },
  { value: "4.1", label: "4.1" },
  { value: "4.1.1", label: "4.1.1" },
  { value: "4.2", label: "4.2" },
];

const ENCODINGS: { value: EncodingOpt; label: string }[] = [
  { value: "utf-8", label: "UTF-8" },
  { value: "windows-1252", label: "Windows-1252 / Latin-1" },
];

export const Controls: Component<{
  dictVersion: DictVersionOpt;
  onDictVersion: (v: DictVersionOpt) => void;
  encoding: EncodingOpt;
  onEncoding: (v: EncodingOpt) => void;
  aligned: boolean;
  onAligned: (v: boolean) => void;
}> = (props) => {
  const selectClass = controlClass;

  return (
    <Card>
      <ControlGrid>
      <label class="flex flex-col gap-1 text-xs text-fg-muted">
        Dictionary edition
        <select
          class={selectClass}
          value={props.dictVersion}
          onChange={(e) =>
            props.onDictVersion(e.currentTarget.value as DictVersionOpt)
          }
        >
          <For each={DICT_VERSIONS}>
            {(d) => <option value={d.value}>{d.label}</option>}
          </For>
        </select>
      </label>

      <label class="flex flex-col gap-1 text-xs text-fg-muted">
        Encoding
        <select
          class={selectClass}
          value={props.encoding}
          onChange={(e) =>
            props.onEncoding(e.currentTarget.value as EncodingOpt)
          }
        >
          <For each={ENCODINGS}>
            {(en) => <option value={en.value}>{en.label}</option>}
          </For>
        </select>
      </label>

      <label class="flex items-center gap-2 text-sm text-fg-soft sm:self-end sm:pb-1.5">
        <input
          type="checkbox"
          class="h-4 w-4 rounded border-line-strong bg-surface-raised"
          checked={props.aligned}
          onChange={(e) => props.onAligned(e.currentTarget.checked)}
        />
        Aligned columns
      </label>
      </ControlGrid>
    </Card>
  );
};
