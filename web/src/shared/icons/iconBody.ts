/**
 * Strip a vendored Lucide file down to the markup inside its `<svg>`.
 *
 * The vendored files are byte-for-byte upstream (see scripts/sync-icons.mjs),
 * so each one carries a licence comment and a root `<svg>` whose own
 * `width`/`height`/`class` attributes are upstream's, not ours. Rendering that
 * root element directly means every icon arrives at 24px wearing a `lucide`
 * class, and the size has to be undone by string-replacing attributes at render
 * time — which is what the design system's own Icon does, and why it has to.
 *
 * Taking the body instead lets `<Icon>` own the wrapper: one viewBox, our size,
 * our stroke width, and `currentColor` inherited from the caller. The cost is
 * this function, and it runs once per icon at module load rather than per
 * render.
 *
 * Deliberately not a parser. These are eleven known files from one generator
 * with no attribute containing `>`, no CDATA and no nested `<svg>`; a DOM parse
 * would not run in the node test lane anyway. It throws rather than returning
 * something half-right, because a silently empty icon is the failure mode this
 * whole vendoring exists to avoid.
 */
export function iconBody(raw: string): string {
  const open = raw.indexOf("<svg");
  if (open < 0) throw new Error("icon has no <svg> element");

  const afterAttrs = raw.indexOf(">", open);
  if (afterAttrs < 0) throw new Error("icon's <svg> element is unterminated");

  const close = raw.lastIndexOf("</svg>");
  if (close < afterAttrs) throw new Error("icon has no closing </svg>");

  const body = raw.slice(afterAttrs + 1, close).trim();
  if (body === "") throw new Error("icon has an empty <svg> body");
  return body;
}
