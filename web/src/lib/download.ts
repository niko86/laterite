// Client-side file download — build a Blob, click a transient anchor, revoke
// the object URL. Nothing leaves the browser. Used by the Tools utilities
// (template / anonymiser / formatter) and any other "save this" action.

export function downloadBlob(
  data: BlobPart,
  filename: string,
  type = "text/plain",
): void {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

/** Strip a trailing " (suffix)" + extension from a display name, for use as a
 *  download base. `"delivery.ags (fixed)"` → `"delivery"`. */
export function baseName(name: string | undefined, fallback = "ags4"): string {
  const base = (name || fallback)
    .replace(/\s*\(.*\)\s*$/, "")
    .replace(/\.[^./]*$/, "")
    .trim();
  return base || fallback;
}
