import { For, Show, type Component } from "solid-js";
import { PillToggle } from "../PillToggle";
import { toolsTool as tool, setToolsTool as setTool } from "../../lib/settings";
import { DictionaryBrowser } from "./DictionaryBrowser";
import { RuleExplainer } from "./RuleExplainer";
import { RevisionDiff } from "./RevisionDiff";
import { TemplateGenerator } from "./TemplateGenerator";
import { Anonymiser } from "./Anonymiser";
import { Formatter } from "./Formatter";
import { CoordinateTool } from "./CoordinateTool";

export type Tool =
  | "dictionary"
  | "rules"
  | "revision"
  | "template"
  | "anonymiser"
  | "formatter"
  | "coords";

// Grouped so it's obvious which tools need the loaded file and which don't.
type Group = "Reference" | "This file" | "Compare";
const GROUPS: Group[] = ["Reference", "This file", "Compare"];
const TOOLS: { id: Tool; label: string; group: Group }[] = [
  { id: "dictionary", label: "Dictionary", group: "Reference" },
  { id: "rules", label: "Rules", group: "Reference" },
  { id: "template", label: "Template", group: "Reference" },
  { id: "anonymiser", label: "Anonymiser", group: "This file" },
  { id: "formatter", label: "Formatter", group: "This file" },
  { id: "coords", label: "Coordinates", group: "This file" },
  { id: "revision", label: "Revision diff", group: "Compare" },
];

// The Tools tab — client-side AGS4 utilities, grouped by what they act on:
// "Reference" needs no file (dictionary / rules / a blank template),
// "This file" acts on the loaded file (anonymise / format / coordinates),
// "Compare" takes its own two files (revision diff). Nothing is uploaded.
export const ToolsPane: Component = () => {
  return (
    <div class="flex min-w-0 flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <For each={GROUPS}>
          {(grp) => (
            <div class="flex flex-wrap items-center gap-1 text-sm">
              <span class="mr-1 w-16 shrink-0 text-xs font-medium uppercase tracking-wide text-fg-dim">
                {grp}
              </span>
              <For each={TOOLS.filter((t) => t.group === grp)}>
                {(t) => (
                  <PillToggle
                    label={t.label}
                    active={tool() === t.id}
                    onClick={() => setTool(t.id)}
                  />
                )}
              </For>
            </div>
          )}
        </For>
      </div>
      <Show when={tool() === "dictionary"}>
        <DictionaryBrowser />
      </Show>
      <Show when={tool() === "rules"}>
        <RuleExplainer />
      </Show>
      <Show when={tool() === "revision"}>
        <RevisionDiff />
      </Show>
      <Show when={tool() === "template"}>
        <TemplateGenerator />
      </Show>
      <Show when={tool() === "anonymiser"}>
        <Anonymiser />
      </Show>
      <Show when={tool() === "formatter"}>
        <Formatter />
      </Show>
      <Show when={tool() === "coords"}>
        <CoordinateTool />
      </Show>
    </div>
  );
};
