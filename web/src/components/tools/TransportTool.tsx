import { createSignal, Show, type Component } from "solid-js";
import { fileStore } from "../../lib/fileStore";
import { lock, unlock } from "../../lib/transportClient";
import { downloadBlob } from "../../lib/download";
import { isLowEndDevice } from "../../lib/device";
import { controlCompact } from "../../lib/controls";
import { Spinner } from "../Spinner";

// Tools → Transport: compress + passphrase-encrypt a file to a `.zst.age`
// (and back), fully client-side (#295). The pipeline runs in a Web Worker
// (transport.worker.ts) because the scrypt KDF is deliberately expensive.
// Output is byte-compatible with laterite `lock`/`unlock` (CLI, Python pyrage) —
// zstd 9 + age scrypt log_N 18. Nothing is uploaded to a server.

// A soft, device-tiered size cap (warn-and-allow) — the pipeline is one-shot,
// so peak memory scales with the file. Low-end (≤2 GB / ≤2 cores) gets a lower
// cap. Above it we warn but still let the user proceed.
const CAP_MB = isLowEndDevice() ? 100 : 200;

const fmtSize = (n: number) =>
  n < 1024
    ? `${n} B`
    : n < 1048576
      ? `${(n / 1024).toFixed(1)} KB`
      : `${(n / 1048576).toFixed(1)} MB`;

const friendlyErr = (e: unknown) => {
  const s = String(e);
  return /passphrase|no identity|decrypt|scrypt|MAC|header/i.test(s)
    ? "Wrong passphrase, or not a laterite/age (.zst.age) file."
    : s;
};

export const TransportTool: Component = () => {
  const [busy, setBusy] = createSignal<"lock" | "unlock" | null>(null);
  const [err, setErr] = createSignal<string | null>(null);
  const [note, setNote] = createSignal<string | null>(null);

  const [lockPass, setLockPass] = createSignal("");
  const [lockFile, setLockFile] = createSignal<File | null>(null);
  const [unlockPass, setUnlockPass] = createSignal("");
  const [unlockFile, setUnlockFile] = createSignal<File | null>(null);

  // What Lock acts on: an uploaded file wins, else the file loaded in Validate.
  const lockSource = () => {
    const f = lockFile();
    if (f)
      return {
        size: f.size,
        name: f.name,
        bytes: async () => new Uint8Array(await f.arrayBuffer()),
      };
    const b = fileStore.bytes();
    if (b)
      return {
        size: b.length,
        name: fileStore.name() || "delivery.ags",
        bytes: () => Promise.resolve(b),
      };
    return null;
  };
  const overCap = (n: number) => n > CAP_MB * 1048576;

  const reset = () => {
    setErr(null);
    setNote(null);
  };

  const doLock = async () => {
    const src = lockSource();
    if (!src || !lockPass()) return;
    setBusy("lock");
    reset();
    try {
      const out = await lock(await src.bytes(), lockPass());
      downloadBlob(
        out.slice().buffer,
        `${src.name}.zst.age`,
        "application/octet-stream",
      );
      setNote(
        `Encrypted ${src.name} → ${src.name}.zst.age (${fmtSize(out.length)})`,
      );
    } catch (e) {
      setErr(friendlyErr(e));
    } finally {
      setBusy(null);
    }
  };

  const doUnlock = async () => {
    const f = unlockFile();
    if (!f || !unlockPass()) return;
    setBusy("unlock");
    reset();
    try {
      const bytes = new Uint8Array(await f.arrayBuffer());
      const out = await unlock(bytes, unlockPass());
      const name =
        f.name.replace(/\.zst\.age$/i, "").replace(/\.age$/i, "") ||
        "decrypted";
      downloadBlob(out.slice().buffer, name, "application/octet-stream");
      setNote(`Decrypted ${f.name} → ${name} (${fmtSize(out.length)})`);
    } catch (e) {
      setErr(friendlyErr(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <div class="flex min-w-0 flex-col gap-4">
      <p class="text-sm text-fg-soft">
        Compress + passphrase-encrypt a file to a{" "}
        <code class="mono">.zst.age</code> you can share, and decrypt one back.
        Byte-compatible with <code class="mono">lat unlock</code> / the{" "}
        <code class="mono">laterite</code> library /{" "}
        <code class="mono">pyrage</code>. Runs entirely in your browser.
      </p>

      {/* Lock: encrypt the loaded file (or an uploaded one). */}
      <div class="flex flex-col gap-2 rounded-lg border border-line bg-surface p-3">
        <p class="text-sm font-medium text-fg-soft">Encrypt → .zst.age</p>
        <Show
          when={lockSource()}
          fallback={
            <p class="text-xs text-fg-faint">
              Load an AGS4 file in the Validate tab, or choose any file below.
            </p>
          }
        >
          {(src) => (
            <p class="text-xs text-fg-faint">
              File: <span class="mono text-fg-soft">{src().name}</span> (
              {fmtSize(src().size)})
              <Show when={overCap(src().size)}>
                <span class="ml-2 text-warn">
                  · over {CAP_MB} MB — encryption may be slow / memory-heavy on
                  this device
                </span>
              </Show>
            </p>
          )}
        </Show>
        <div class="flex flex-wrap items-center gap-3 text-sm">
          <input
            type="password"
            placeholder="Passphrase"
            autocomplete="new-password"
            class={`w-48 ${controlCompact}`}
            value={lockPass()}
            onInput={(e) => setLockPass(e.currentTarget.value)}
          />
          <button
            type="button"
            disabled={busy() !== null || !lockSource() || !lockPass()}
            class="rounded bg-cta px-3 py-1.5 font-medium text-fg-on-cta hover:bg-cta-hover disabled:cursor-not-allowed disabled:opacity-40"
            onClick={() => void doLock()}
          >
            {busy() === "lock" ? "Encrypting…" : "Encrypt & download"}
          </button>
          <label class="cursor-pointer text-xs text-fg-muted hover:text-fg-soft">
            choose a different file…
            <input
              type="file"
              class="hidden"
              onChange={(e) => {
                setLockFile(e.currentTarget.files?.[0] ?? null);
                e.currentTarget.value = "";
              }}
            />
          </label>
          <Show when={lockFile()}>
            <button
              type="button"
              class="text-xs text-fg-faint underline hover:text-fg-soft"
              onClick={() => setLockFile(null)}
            >
              use loaded file
            </button>
          </Show>
        </div>
      </div>

      {/* Unlock: decrypt an uploaded .zst.age. */}
      <div class="flex flex-col gap-2 rounded-lg border border-line bg-surface p-3">
        <p class="text-sm font-medium text-fg-soft">Decrypt ← .zst.age</p>
        <div class="flex flex-wrap items-center gap-3 text-sm">
          <label class="cursor-pointer rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip">
            <Show when={unlockFile()} fallback="Choose a .zst.age file…">
              {(f) => f().name}
            </Show>
            <input
              type="file"
              accept=".age,.zst,application/octet-stream"
              class="hidden"
              onChange={(e) => {
                setUnlockFile(e.currentTarget.files?.[0] ?? null);
                e.currentTarget.value = "";
              }}
            />
          </label>
          <input
            type="password"
            placeholder="Passphrase"
            autocomplete="new-password"
            class={`w-48 ${controlCompact}`}
            value={unlockPass()}
            onInput={(e) => setUnlockPass(e.currentTarget.value)}
          />
          <button
            type="button"
            disabled={busy() !== null || !unlockFile() || !unlockPass()}
            class="rounded border border-line-strong px-3 py-1.5 text-fg hover:border-accent hover:text-accent disabled:cursor-not-allowed disabled:opacity-40"
            onClick={() => void doUnlock()}
          >
            {busy() === "unlock" ? "Decrypting…" : "Decrypt & download"}
          </button>
        </div>
      </div>

      <Show when={busy()}>
        <Spinner
          label={
            busy() === "lock" ? "Encrypting (deriving key)…" : "Decrypting…"
          }
        />
      </Show>
      <Show when={note()}>
        <p class="text-xs text-ok">✓ {note()}</p>
      </Show>
      <Show when={err()}>
        <p class="text-xs text-err">{err()}</p>
      </Show>
    </div>
  );
};
