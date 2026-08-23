/* Saving the delivery as a real .ags file (#639).
 *
 * The bytes are the EMITTER'S OWN — the same text the file pane shows —
 * never a re-serialization: the download exists so a reader can run the
 * delivery they just edited through their own validator and compare
 * findings, and bytes this page invented would poison that comparison.
 *
 * Chromium's save-as picker is the decided front door: a reader names and
 * places the file. Where the platform has no picker (Safari, Firefox), the
 * anchor download falls back under the decided name `laterite.ags`. A
 * cancelled picker is a change of mind, not a failure — AbortError reports
 * "cancelled" and nothing downloads. Any other failure after the picker
 * reports "failed", because the reader just chose a location in an OS
 * dialog and silence there reads as saved.
 *
 * The webapp bundle has this fallback's twin (web/src/lib/download.ts,
 * downloadBlob) — a local copy, not an import, because the landing takes no
 * runtime code from the app bundle (#394; sharedAlias.test.ts holds the
 * boundary). A fix to either anchor path should visit the other.
 */

/** The decided fallback name — also the picker's suggestion. */
export const DELIVERY_FILENAME = "laterite.ags";

type SaveFilePicker = (options: {
  suggestedName: string;
  types: { description: string; accept: Record<string, string[]> }[];
}) => Promise<{
  createWritable: () => Promise<{
    write: (data: string) => Promise<void>;
    close: () => Promise<void>;
    abort: () => Promise<void>;
  }>;
}>;

export type SaveOutcome = "saved" | "cancelled" | "failed";

export async function saveDelivery(text: string): Promise<SaveOutcome> {
  // Called THROUGH the window object: a detached reference to a native
  // picker throws "Illegal invocation" — the check and the call share one
  // receiver.
  const w = window as Window & { showSaveFilePicker?: SaveFilePicker };
  if (w.showSaveFilePicker) {
    let handle;
    try {
      handle = await w.showSaveFilePicker({
        suggestedName: DELIVERY_FILENAME,
        types: [
          { description: "AGS4 delivery", accept: { "text/plain": [".ags"] } },
        ],
      });
    } catch (e) {
      if (e instanceof DOMException && e.name === "AbortError")
        return "cancelled";
      return "failed";
    }
    try {
      const writable = await handle.createWritable();
      try {
        await writable.write(text);
        await writable.close();
      } catch (e) {
        // Discard the swap file so nothing half-written can land; the
        // abort's own failure changes nothing about the outcome.
        try {
          await writable.abort();
        } catch {
          /* already reporting failed */
        }
        throw e;
      }
      return "saved";
    } catch {
      return "failed";
    }
  }
  const url = URL.createObjectURL(new Blob([text], { type: "text/plain" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = DELIVERY_FILENAME;
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Deferred, not immediate: revoking on the same tick races the download
  // START in the very browsers this branch serves (a recorded
  // Firefox/Safari failure mode) — the URL is a courtesy cleanup, not a
  // resource worth that race.
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, 30_000);
  return "saved";
}
