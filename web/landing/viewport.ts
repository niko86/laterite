/* Which PRESENTATION a region gets is a width question, not a modality one
 * (#592) — the inverse of pointer.ts's split. A stack of cards costs vertical
 * space, and a narrow window pays that cost whether its pointer is a finger
 * or a mouse. Each signal is named for the layout boundary it reads, and the
 * boundary is always one the page's own grids already flip at — the content
 * changes dress exactly where the page changes shape.
 *
 * Module scope with no owner, like pointer.ts: these signals ARE the page's
 * lifetime, and a media-query listener holds no DOM to leak.
 */

import { createSignal, type Accessor } from "solid-js";

function matches(query: string): Accessor<boolean> {
  const q = window.matchMedia(query);
  const [on, setOn] = createSignal(q.matches);
  q.addEventListener("change", (e) => setOn(e.matches));
  return on;
}

const wide = matches("(min-width: 64rem)");
const gridded = matches("(min-width: 38rem)");

/** True below the page's two-column breakpoint (the pairing grids flip at
 *  64rem) — the finding carousels' audience (#592). */
export const narrowViewport = () => !wide();

/** True below the install grid's own first column break (38rem), where its
 *  five cards would stack full-height — the install deck's audience (#595). */
export const phoneViewport = () => !gridded();
