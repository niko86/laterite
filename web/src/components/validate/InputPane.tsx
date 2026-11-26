import { createSignal, Show, type Component } from "solid-js";
import { Button } from "@shared/components";
import { isAgsFileName } from "../../lib/agsFileName";

/** Read a dropped/selected file into bytes + its name. */
async function readFile(
  file: File,
): Promise<{ bytes: Uint8Array; name: string }> {
  const buf = await file.arrayBuffer();
  return { bytes: new Uint8Array(buf), name: file.name };
}

export const InputPane: Component<{
  text: () => string;
  name: string;
  onText: (s: string) => void;
  onBytes: (b: Uint8Array, name: string) => void;
}> = (props) => {
  const [dragging, setDragging] = createSignal(false);
  let fileInput: HTMLInputElement | undefined;

  // The name of a file that was refused, so the reader is told why nothing
  // loaded. Cleared by the next file that is accepted.
  const [refused, setRefused] = createSignal<string | null>(null);

  const handleFiles = async (files: FileList | null | undefined) => {
    const f = files?.[0];
    if (!f) return;
    // Checked by NAME, after the pick, and not with an `accept` list on the
    // input — see the note there.
    if (!isAgsFileName(f.name)) {
      setRefused(f.name);
      return;
    }
    setRefused(null);
    const { bytes, name } = await readFile(f);
    props.onBytes(bytes, name);
  };

  return (
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-fg-soft">
          AGS4 input{props.name ? ` — ${props.name}` : ""}
        </label>
        <Button size="sm" onClick={() => fileInput?.click()}>
          Choose file…
        </Button>
        {/*
          No `accept` filter on purpose: `.ags` has no registered MIME
          type/UTI, so mobile pickers (esp. iOS) translate any accept list
          to types they know and grey out .ags files — leaving the user
          unable to pick their file. The restriction to `.ags` is applied to
          the chosen file's name instead (handleFiles), which works on every
          picker and covers a dropped file too. Don't re-add accept.
        */}
        <input
          ref={fileInput}
          type="file"
          class="hidden"
          onChange={(e) => {
            const picked = e.currentTarget.files;
            void handleFiles(picked).finally(() => {
              // Let the same file be picked again: the input only fires on a
              // CHANGE, and a refused file should be re-pickable once renamed.
              if (fileInput) fileInput.value = "";
            });
          }}
        />
      </div>

      <Show when={refused()}>
        {(name) => (
          <p role="alert" class="text-sm text-err">
            "{name()}" is not an .ags file, so it was not loaded. Choose a file
            whose name ends in .ags, or paste the text below.
          </p>
        )}
      </Show>

      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragging(false);
          void handleFiles(e.dataTransfer?.files);
        }}
        class="rounded-lg border-2 border-dashed transition-colors"
        classList={{
          "border-accent bg-accent-quiet": dragging(),
          "border-line-strong": !dragging(),
        }}
      >
        {/* Focus is the ring only — the textarea is borderless (the drop-zone
            wrapper carries the border), so the accent-border half of the focus
            contract has nothing to paint on. */}
        <textarea
          class="mono h-96 w-full min-w-0 resize-y rounded-xs bg-surface-raised p-3 text-xs leading-relaxed text-fg outline-hidden focus-visible:[box-shadow:var(--focus-ring)] placeholder:text-fg-dim"
          placeholder={
            'Drag & drop a .ags file here, or paste AGS4 text…\n\n"GROUP","PROJ"\n"HEADING","PROJ_ID",...'
          }
          spellcheck={false}
          value={props.text()}
          onInput={(e) => {
            props.onText(e.currentTarget.value);
          }}
        />
      </div>
    </div>
  );
};
