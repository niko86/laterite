import { createSignal, type Component } from "solid-js";

/** Read a dropped/selected file into bytes + its name. */
async function readFile(file: File): Promise<{ bytes: Uint8Array; name: string }> {
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

  const handleFiles = async (files: FileList | null | undefined) => {
    const f = files?.[0];
    if (!f) return;
    const { bytes, name } = await readFile(f);
    props.onBytes(bytes, name);
  };

  return (
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-fg-soft">
          AGS4 input{props.name ? ` — ${props.name}` : ""}
        </label>
        <button
          type="button"
          class="rounded border border-line-strong px-2 py-1 text-xs text-fg-soft hover:bg-chip"
          onClick={() => fileInput?.click()}
        >
          Choose file…
        </button>
        {/*
          No `accept` filter on purpose: `.ags` has no registered MIME
          type/UTI, so mobile pickers (esp. iOS) translate any accept list
          to types they know and grey out .ags files — leaving the user
          unable to pick their file. Accepting anything is safe: the
          validator reads raw bytes and reports `not_ags4` for non-AGS
          input, same as the paste path. Don't re-add accept.
        */}
        <input
          ref={fileInput}
          type="file"
          class="hidden"
          onChange={(e) => handleFiles(e.currentTarget.files)}
        />
      </div>

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
          "border-accent bg-accent/5": dragging(),
          "border-line-strong": !dragging(),
        }}
      >
        <textarea
          class="mono h-96 w-full min-w-0 resize-y rounded-lg bg-surface-raised p-3 text-xs leading-relaxed text-fg outline-none placeholder:text-fg-dim"
          placeholder={
            'Drag & drop a .ags file here, or paste AGS4 text…\n\n"GROUP","PROJ"\n"HEADING","PROJ_ID",...'
          }
          spellcheck={false}
          value={props.text()}
          onInput={(e) => props.onText(e.currentTarget.value)}
        />
      </div>
    </div>
  );
};
