import { For, type Component } from "solid-js";
import type { DictVersionOpt, EncodingOpt } from "../../lib/validator";
import { DICT_VERSIONS as DICT_VERSION_VALUES } from "../../lib/editions";
import { Card } from "../Card";
import { ControlGrid } from "../ControlGrid";
import { Field, Select } from "@shared/components";

// Built from the generated editions SSOT (laterite-dev#529); only the "auto" label is UI copy
// and stays here. Was a hand-listed array edited in lockstep with three other copies.
const DICT_VERSIONS: { value: DictVersionOpt; label: string }[] =
  DICT_VERSION_VALUES.map((v) => ({
    value: v,
    label: v === "auto" ? "Auto (from TRAN_AGS)" : v,
  }));

const ENCODINGS: { value: EncodingOpt; label: string }[] = [
  { value: "utf-8", label: "UTF-8" },
  { value: "windows-1252", label: "Windows-1252 / Latin-1" },
];

export const Controls: Component<{
  dictVersion: DictVersionOpt;
  onDictVersion: (v: DictVersionOpt) => void;
  encoding: EncodingOpt;
  onEncoding: (v: EncodingOpt) => void;
}> = (props) => {
  return (
    <Card>
      <ControlGrid>
        <Field label="Dictionary edition">
          <Select
            value={props.dictVersion}
            onChange={(e) => {
              props.onDictVersion(e.currentTarget.value as DictVersionOpt);
            }}
          >
            <For each={DICT_VERSIONS}>
              {(d) => <option value={d.value}>{d.label}</option>}
            </For>
          </Select>
        </Field>

        <Field label="Encoding">
          <Select
            value={props.encoding}
            onChange={(e) => {
              props.onEncoding(e.currentTarget.value as EncodingOpt);
            }}
          >
            <For each={ENCODINGS}>
              {(en) => <option value={en.value}>{en.label}</option>}
            </For>
          </Select>
        </Field>
      </ControlGrid>
    </Card>
  );
};
